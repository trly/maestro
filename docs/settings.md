# Settings & Configuration

Maestro provides user-configurable settings for development tools, CI monitoring, and appearance preferences. Settings are stored in the SQLite database and loaded globally at app startup.

## Settings Domain

### Available Settings

| Setting Key | Type | Default | Description |
|-------------|------|---------|-------------|
| `ci_stuck_threshold_minutes` | `i64` | `10` | Minutes before pending CI is marked as "not_configured" |
| `editor_command` | `string` | `"code"` | Command for opening worktrees (legacy, use `selected_editor`) |
| `selected_editor` | `string` | `"code"` | Preferred editor from available options |
| `selected_terminal` | `string` | `""` | Terminal app for vim/nvim (macOS only) |

### Backend Storage

Settings are stored in a key-value table:

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

**Commands:**

```rust
// src-tauri/src/commands/settings.rs
#[tauri::command]
pub fn get_setting(key: String, store: State<Mutex<Store>>) 
    -> Result<Option<String>, String>

#[tauri::command]
pub fn set_setting(key: String, value: String, store: State<Mutex<Store>>) 
    -> Result<(), String>

#[tauri::command]
pub fn get_ci_stuck_threshold_minutes(store: State<Mutex<Store>>) 
    -> Result<i64, String>
```

## Frontend Store

### Settings Store Pattern

```typescript
// src/lib/stores/settingsStore.ts
import { writable } from 'svelte/store';
import * as ipc from '$lib/ipc';

export interface Settings {
  ciStuckThresholdMinutes: number;
  editorCommand: string;
  selectedEditor: string;
  selectedTerminal: string;
}

const settingsStore = createSettingsStore();

// Load settings from backend
await settingsStore.load();

// Update individual settings
await settingsStore.setCiStuckThreshold(15);
await settingsStore.setSelectedEditor('nvim');
await settingsStore.setSelectedTerminal('ghostty');
```

**Key Pattern:** Settings are loaded once at app startup and cached in a Svelte store for reactive access throughout the app.

## App Initialization

Settings must be loaded during app initialization to ensure they're available when needed:

```svelte
<!-- src/routes/+layout.svelte -->
<script lang="ts">
  import { settingsStore } from '$lib/stores/settingsStore';
  import { onMount } from 'svelte';

  onMount(async () => {
    await settingsStore.load();  // Load settings on startup
    // Settings now available throughout app
  });
</script>
```

**Critical:** Settings must be loaded in `+layout.svelte` before components that depend on them (e.g., opening editors) are used. Otherwise, they fall back to defaults.

## Editor & Terminal Detection

Maestro detects installed editors and terminals on the user's system using the `app_check` module.

### Backend Detection

```rust
// src-tauri/src/commands/app_check.rs
pub struct AppInfo {
    pub command: String,        // e.g., "nvim"
    pub display_name: String,   // e.g., "Neovim"
    pub needs_terminal: bool,   // true for vim/nvim
}

#[tauri::command]
pub fn get_available_editors() -> Vec<AppInfo> {
    // Checks PATH for: vim, nvim, code, cursor, zed
    // Returns only installed editors
}

#[tauri::command]
pub fn get_available_terminals() -> Vec<TerminalInfo> {
    // macOS: Terminal.app, Ghostty
    // Linux/Windows: TBD
}
```

**Detection method:** Uses `which` command to check if editor/terminal exists in PATH.

### Supported Editors

| Editor | Command | Terminal Required | Platform |
|--------|---------|-------------------|----------|
| Vim | `vim` | ✅ Yes | All |
| Neovim | `nvim` | ✅ Yes | All |
| VS Code | `code` | ❌ No | All |
| Cursor | `cursor` | ❌ No | All |
| Zed | `zed` | ❌ No | All |

### Supported Terminals (macOS)

| Terminal | Command | Notes |
|----------|---------|-------|
| Terminal.app | `open -a Terminal` | Native macOS terminal |
| Ghostty | `ghostty` | Modern GPU-accelerated terminal |

## Opening Worktrees in Editor

### Frontend Utility

```typescript
// src/lib/utils/worktree.ts
export async function openInEditor(
  execution: Execution, 
  editorCommandFallback?: string
): Promise<void> {
  const settings = get(settingsStore);
  
  // Use configured editor
  const selectedEditor = settings.selectedEditor || 'code';
  const selectedTerminal = settings.selectedTerminal;
  
  // Check if editor needs terminal
  const availableEditors = await ipc.getAvailableEditors();
  const editorInfo = availableEditors.find(e => e.command === selectedEditor);
  
  if (editorInfo?.needsTerminal && selectedTerminal) {
    // Launch vim/nvim in terminal
    await ipc.openWorktreeWithTerminal(
      execution.promptsetId,
      execution.id,
      selectedEditor,
      selectedTerminal
    );
  } else {
    // Direct launch (VS Code, Cursor, Zed)
    await ipc.openWorktreeInEditor(
      execution.promptsetId,
      execution.id,
      selectedEditor
    );
  }
}
```

### Backend Launch Strategies

```rust
// src-tauri/src/commands/worktree.rs
#[tauri::command]
pub fn open_worktree_in_editor(
    promptset_id: String,
    execution_id: String,
    editor_command: String,
    paths: State<'_, Paths>
) -> Result<(), String> {
    let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);
    
    Command::new(&editor_command)
        .arg(worktree_path)
        .spawn()?;
    
    Ok(())
}

#[tauri::command]
pub fn open_worktree_with_terminal(
    promptset_id: String,
    execution_id: String,
    editor_command: String,
    terminal_command: String,
    paths: State<'_, Paths>
) -> Result<(), String> {
    let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);
    
    match terminal_command.as_str() {
        "ghostty" => {
            Command::new("ghostty")
                .arg("-e")
                .arg(&editor_command)
                .arg(worktree_path)
                .spawn()?;
        },
        "open -a Terminal" => {
            // Use AppleScript to launch Terminal.app
            let shell_script = format!("cd {} && exec {}", worktree_path, editor_command);
            let applescript = format!("tell application \"Terminal\" ...");
            Command::new("osascript")
                .arg("-e")
                .arg(applescript)
                .spawn()?;
        },
        _ => return Err(format!("Unsupported terminal: {}", terminal_command))
    }
    
    Ok(())
}
```

## Secure Token Storage

Maestro uses the **platform keyring** for secure credential storage, not the settings database. This keeps sensitive data encrypted at the OS level.

### Keyring Architecture

```rust
// src-tauri/src/commands/tokens.rs
use keyring::Entry;

const SERVICE_NAME: &str = "dev.trly.maestro";

fn get_entry(key: &str) -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, key)
        .map_err(|e| format!("Failed to access keyring: {}", e))
}
```

**Platform-Specific Storage:**
- **macOS**: Keychain (Keychain Access.app)
- **Linux**: Secret Service API (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

**Token Keys:**
- `amp_token` - Amp API authentication token
- `github_token` - GitHub Personal Access Token

### Token Operations

```rust
#[tauri::command]
pub fn set_token(key: String, value: String) -> Result<(), String> {
    let entry = get_entry(&key)?;
    entry.set_password(&value)
        .map_err(|e| format!("Failed to save token: {}", e))
}

#[tauri::command]
pub fn get_token(key: String) -> Result<Option<String>, String> {
    let entry = get_entry(&key)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve token: {}", e))
    }
}

#[tauri::command]
pub fn get_token_masked(key: String) -> Result<Option<String>, String> {
    // Returns "abc...xyz" format for display
}

#[tauri::command]
pub fn delete_token(key: String) -> Result<(), String> {
    let entry = get_entry(&key)?;
    entry.delete_credential()
}
```

### Frontend Token Store

```typescript
// src/lib/tokenStore.ts
import * as ipc from '$lib/ipc';

export type TokenKey = 'amp_token' | 'github_token';

export const tokenStore = {
  async getToken(key: TokenKey): Promise<string | null> {
    return await ipc.getToken(key);
  },
  
  async getTokenMasked(key: TokenKey): Promise<string | null> {
    return await ipc.getTokenMasked(key);
  },
  
  async setToken(key: TokenKey, value: string): Promise<void> {
    return await ipc.setToken(key, value);
  },
  
  async deleteToken(key: TokenKey): Promise<void> {
    return await ipc.deleteToken(key);
  }
};
```

### Security Benefits

1. **OS-Level Encryption**: Tokens encrypted by operating system
2. **No File Storage**: Never written to config files, env vars, or database
3. **Access Control**: OS manages app permissions to keyring
4. **Secure Retrieval**: Tokens retrieved at command execution time only
5. **Automatic Cleanup**: Tokens removed when deleted from keyring

### Usage Pattern

```rust
// In Tauri commands - retrieve token from keyring
use crate::commands::tokens::get_token_value;

let github_token = get_token_value("github_token")
    .map_err(|e| format!("Failed to access token: {}", e))?
    .ok_or_else(|| "GitHub token not configured".to_string())?;

// Use token for API call
let provider = GitHubProvider::new(github_token)?;
```

**Critical:** Always retrieve tokens at command execution time, never cache in memory or store in structs.

## Settings UI

### Settings Page

```svelte
<!-- src/lib/components/Settings.svelte -->
<script lang="ts">
  import { settingsStore } from '$lib/stores/settingsStore';
  import * as ipc from '$lib/ipc';
  
  let availableEditors = $state<ipc.AppInfo[]>([]);
  let availableTerminals = $state<ipc.TerminalInfo[]>([]);
  
  onMount(async () => {
    // Load available options
    availableEditors = await ipc.getAvailableEditors();
    availableTerminals = await ipc.getAvailableTerminals();
  });
  
  async function handleEditorChange(value: string) {
    await settingsStore.setSelectedEditor(value);
  }
</script>

<!-- Editor dropdown populated with detected editors -->
<Select.Root
  value={selectedEditor}
  onValueChange={handleEditorChange}
  items={availableEditors.map(e => ({ 
    value: e.command, 
    label: e.displayName 
  }))}
>
  <!-- ... -->
</Select.Root>
```

### Settings Sections

1. **API Tokens** - Amp and GitHub tokens (stored in keyring)
2. **Development Tools** - Editor and terminal preferences
3. **CI Monitoring** - CI stuck timeout threshold
4. **Appearance** - Theme selection (light/dark/auto)

## Common Patterns

### Reading Settings

```typescript
import { get } from 'svelte/store';
import { settingsStore } from '$lib/stores/settingsStore';

const settings = get(settingsStore);
const threshold = settings.ciStuckThresholdMinutes;
```

### Updating Settings

```typescript
await settingsStore.setSelectedEditor('nvim');
// Automatically persists to backend and updates store
```

### Reactive Settings

```svelte
<script lang="ts">
  import { settingsStore } from '$lib/stores/settingsStore';
  
  // Subscribe to settings changes
  $: ciThreshold = $settingsStore.ciStuckThresholdMinutes;
</script>
```

## CI Stuck Threshold

The CI stuck threshold determines when pending CI checks are marked as "not_configured":

**Default:** 10 minutes

**Use case:** Prevents false positives for repos without CI, while allowing slow CI workflows to start.

**Implementation:**

```rust
// src-tauri/src/db/store.rs
pub fn get_ci_stuck_threshold_minutes(&self) -> Result<i64> {
    let value = self.get_setting("ci_stuck_threshold_minutes")?
        .unwrap_or_else(|| "10".to_string());
    
    value.parse::<i64>()
        .or(Ok(10))  // Default to 10 if invalid
}
```

See [ci-tracking.md](./ci-tracking.md) for how this is used in CI status checking.

## Best Practices

### Loading Settings

✅ **DO:** Load settings in `+layout.svelte` on app startup
```typescript
onMount(async () => {
  await settingsStore.load();
});
```

❌ **DON'T:** Load settings lazily in components
```typescript
// This causes race conditions
onMount(async () => {
  await settingsStore.load();  // Too late if other components already ran
  await openInEditor(execution);
});
```

### Editor Detection

✅ **DO:** Check if editor needs terminal before launching
```typescript
const editorInfo = availableEditors.find(e => e.command === selectedEditor);
if (editorInfo?.needsTerminal && !selectedTerminal) {
  throw new Error(`${selectedEditor} requires a terminal`);
}
```

❌ **DON'T:** Assume terminal is available
```typescript
// This fails if user hasn't selected a terminal
await ipc.openWorktreeWithTerminal(id, id, 'nvim', '');
```

### Fallback Behavior

When settings aren't loaded or are empty, Maestro falls back to:
- Editor: `'code'` (VS Code)
- Terminal: `''` (none)
- CI threshold: `10` minutes

## Implementation Reference

**Backend:**
- `src-tauri/src/commands/settings.rs` - Settings commands
- `src-tauri/src/commands/app_check.rs` - Editor/terminal detection
- `src-tauri/src/commands/worktree.rs` - Editor launch
- `src-tauri/src/db/store.rs` - Database operations

**Frontend:**
- `src/lib/stores/settingsStore.ts` - Settings store
- `src/lib/utils/worktree.ts` - Editor launch logic
- `src/lib/components/Settings.svelte` - Settings UI
- `src/routes/+layout.svelte` - Settings initialization

## Related Documentation

- **[IPC Guide](./ipc-guide.md)** - Settings IPC commands
- **[CI Tracking](./ci-tracking.md)** - CI stuck threshold usage
- **[Architecture](./architecture.md)** - Overall system design
