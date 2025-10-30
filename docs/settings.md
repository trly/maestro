# Settings & Configuration

Maestro provides user-configurable settings for development tools, CI monitoring, and appearance preferences. **For common settings patterns, see AGENTS.md Settings & Configuration section.**

## When to Read This

- Detailed settings domain model
- OS-specific editor/terminal behavior
- Understanding settings persistence and loading lifecycle

## Settings Domain

### Available Settings

| Setting Key                  | Type     | Default  | Description                                                   |
| ---------------------------- | -------- | -------- | ------------------------------------------------------------- |
| `ci_stuck_threshold_minutes` | `i64`    | `10`     | Minutes before pending CI is marked as "not_configured"       |
| `editor_command`             | `string` | `"code"` | Command for opening worktrees (legacy, use `selected_editor`) |
| `selected_editor`            | `string` | `"code"` | Preferred editor from available options                       |
| `selected_terminal`          | `string` | `""`     | Terminal app for vim/nvim (macOS only)                        |

### Backend Storage

Settings are stored in a key-value database table.

**IPC Commands:**

- `get_setting(key)` - Retrieve setting value
- `set_setting(key, value)` - Update setting
- `get_ci_stuck_threshold_minutes()` - Get CI timeout threshold
- `get_max_concurrent_executions()` - Get execution concurrency limit

## Frontend Store

Settings are loaded once at app startup via `settingsStore.load()` in `+layout.svelte` and cached in a Svelte store for reactive access throughout the app.

**Critical:** Settings must be loaded before components that depend on them (e.g., opening editors) are used. Otherwise, they fall back to defaults.

## Editor & Terminal Detection

Maestro detects installed editors and terminals on the user's system using the `app_check` module.

### Backend Detection

**Editor Detection:**

The system checks the PATH for known editors (vim, nvim, code, cursor, zed) and returns only installed ones.

Each editor provides:

- Command name (e.g., "nvim")
- Display name (e.g., "Neovim")
- Terminal requirement flag (true for vim/nvim)

**Terminal Detection:**

- macOS: Terminal.app, Ghostty
- Linux/Windows: To be implemented

Detection uses the system PATH to verify application availability.

### Supported Editors

| Editor  | Command  | Terminal Required | Platform |
| ------- | -------- | ----------------- | -------- |
| Vim     | `vim`    | ✅ Yes            | All      |
| Neovim  | `nvim`   | ✅ Yes            | All      |
| VS Code | `code`   | ❌ No             | All      |
| Cursor  | `cursor` | ❌ No             | All      |
| Zed     | `zed`    | ❌ No             | All      |

### Supported Terminals (macOS)

| Terminal     | Command            | Notes                           |
| ------------ | ------------------ | ------------------------------- |
| Terminal.app | `open -a Terminal` | Native macOS terminal           |
| Ghostty      | `ghostty`          | Modern GPU-accelerated terminal |

## Opening Worktrees in Editor

### Frontend Utility

```typescript
// src/lib/utils/worktree.ts
export async function openInEditor(
	execution: Execution,
	editorCommandFallback?: string
): Promise<void> {
	const settings = get(settingsStore)

	// Use configured editor
	const selectedEditor = settings.selectedEditor || "code"
	const selectedTerminal = settings.selectedTerminal

	// Check if editor needs terminal
	const availableEditors = await ipc.getAvailableEditors()
	const editorInfo = availableEditors.find((e) => e.command === selectedEditor)

	if (editorInfo?.needsTerminal && selectedTerminal) {
		// Launch vim/nvim in terminal
		await ipc.openWorktreeWithTerminal(
			execution.promptsetId,
			execution.id,
			selectedEditor,
			selectedTerminal
		)
	} else {
		// Direct launch (VS Code, Cursor, Zed)
		await ipc.openWorktreeInEditor(execution.promptsetId, execution.id, selectedEditor)
	}
}
```

### Backend Launch Strategies

**Direct Editor Launch:**

For editors that don't need a terminal (VS Code, Cursor, Zed), the system spawns the editor process directly with the worktree path as an argument.

**Terminal-Wrapped Launch:**

For terminal editors (vim, nvim), the system:

1. Determines the appropriate terminal emulator
2. Spawns the terminal with the editor command
3. Passes the worktree path to the editor

**Supported Launch Methods:**

- Ghostty: Direct launch with `-e` flag
- Terminal.app: AppleScript-based launch
- Other terminals: Platform-specific implementations

## Secure Token Storage

Maestro uses the **platform keyring** for secure credential storage, not the settings database. This keeps sensitive data encrypted at the OS level.

### Platform Storage

Maestro stores tokens in the operating system's secure credential storage:

- **macOS**: Keychain (Keychain Access.app)
- **Linux**: Secret Service API (GNOME Keyring, KWallet)
- **Windows**: Credential Manager

### Supported Tokens

- `amp_token` - Amp API authentication token
- `github_token` - GitHub Personal Access Token
- `gitlab_token` - GitLab Personal Access Token
- `gitlab_instance_url` - GitLab instance URL
- `sourcegraph_endpoint` - Sourcegraph instance URL
- `sourcegraph_token` - Sourcegraph access token

### Token Operations

Access tokens through the IPC layer:

```typescript
import * as ipc from "$lib/ipc"

// Retrieve all tokens
const tokens = await ipc.getAllTokens()
const tokensMasked = await ipc.getAllTokensMasked() // Shows "abc...xyz" format

// Set/delete individual tokens
await ipc.setToken("github_token", value)
await ipc.deleteToken("github_token")
```

### Security Benefits

1. **OS-Level Encryption**: Tokens encrypted by operating system
2. **No File Storage**: Never written to config files, env vars, or database
3. **Access Control**: OS manages app permissions to keyring
4. **Secure Retrieval**: Tokens retrieved at command execution time only
5. **Automatic Cleanup**: Tokens removed when deleted from keyring

**Critical:** Tokens are retrieved on-demand for each operation and never cached in memory.

## Settings UI

### Settings Page

```svelte
<!-- src/lib/components/Settings.svelte -->
<script lang="ts">
	import { settingsStore } from "$lib/stores/settingsStore"
	import * as ipc from "$lib/ipc"

	let availableEditors = $state<ipc.AppInfo[]>([])
	let availableTerminals = $state<ipc.TerminalInfo[]>([])

	onMount(async () => {
		// Load available options
		availableEditors = await ipc.getAvailableEditors()
		availableTerminals = await ipc.getAvailableTerminals()
	})

	async function handleEditorChange(value: string) {
		await settingsStore.setSelectedEditor(value)
	}
</script>

<!-- Editor dropdown populated with detected editors -->
<Select.Root
	value={selectedEditor}
	onValueChange={handleEditorChange}
	items={availableEditors.map((e) => ({
		value: e.command,
		label: e.displayName,
	}))}
>
	<!-- ... -->
</Select.Root>
```

### Settings Sections

1. **API Tokens** - Amp, GitHub, and Sourcegraph tokens (stored in keyring)
2. **Development Tools** - Editor and terminal preferences
3. **CI Monitoring** - CI stuck timeout threshold
4. **Appearance** - Theme selection (light/dark/auto)

## Common Patterns

### Reading Settings

```typescript
import { get } from "svelte/store"
import { settingsStore } from "$lib/stores/settingsStore"

const settings = get(settingsStore)
const threshold = settings.ciStuckThresholdMinutes
```

### Updating Settings

```typescript
await settingsStore.setSelectedEditor("nvim")
// Automatically persists to backend and updates store
```

### Reactive Settings

```svelte
<script lang="ts">
	import { settingsStore } from "$lib/stores/settingsStore"

	// Subscribe to settings changes
	$: ciThreshold = $settingsStore.ciStuckThresholdMinutes
</script>
```

## CI Stuck Threshold

The CI stuck threshold determines when pending CI checks are marked as "not_configured":

**Default:** 10 minutes

**Use case:** Prevents false positives for repos without CI, while allowing slow CI workflows to start.

The backend retrieves this setting and defaults to 10 minutes if not configured or invalid.

See [ci-tracking.md](./ci-tracking.md) for how this is used in CI status checking.

## Best Practices

### Loading Settings

✅ **DO:** Load settings in `+layout.svelte` on app startup

```typescript
onMount(async () => {
	await settingsStore.load()
})
```

❌ **DON'T:** Load settings lazily in components

```typescript
// This causes race conditions
onMount(async () => {
	await settingsStore.load() // Too late if other components already ran
	await openInEditor(execution)
})
```

### Editor Detection

✅ **DO:** Check if editor needs terminal before launching

```typescript
const editorInfo = availableEditors.find((e) => e.command === selectedEditor)
if (editorInfo?.needsTerminal && !selectedTerminal) {
	throw new Error(`${selectedEditor} requires a terminal`)
}
```

❌ **DON'T:** Assume terminal is available

```typescript
// This fails if user hasn't selected a terminal
await ipc.openWorktreeWithTerminal(id, id, "nvim", "")
```

### Fallback Behavior

When settings aren't loaded or are empty, Maestro falls back to:

- Editor: `'code'` (VS Code)
- Terminal: `''` (none)
- CI threshold: `10` minutes

## Implementation Reference

**Backend Modules:**

- Settings commands - IPC interface
- App detection - Editor/terminal availability
- Worktree commands - Editor launch
- Database operations - Settings persistence

**Frontend Modules:**

- Settings store - Cached settings state
- Worktree utilities - Editor launch logic
- Settings UI - Configuration interface
- App layout - Settings initialization

## Related Documentation

- **[IPC Guide](./ipc-guide.md)** - Settings IPC commands
- **[CI Tracking](./ci-tracking.md)** - CI stuck threshold usage
- **[Sourcegraph Integration](./sourcegraph-integration.md)** - Repository search configuration
- **[Architecture](./architecture.md)** - Overall system design
