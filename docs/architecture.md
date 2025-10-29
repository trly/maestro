# Architecture Overview

Maestro is a desktop application for orchestrating AI prompts across multiple repositories using Amp.

## Technology Stack

**Frontend:**

- **SvelteKit** (Svelte 5) with adapter-static
- **TypeScript** with strict mode
- **bits-ui** component primitives
- **Tailwind 4** with Gruvbox theme (semantic tokens only - never raw colors)
- **lucide-svelte** icons

**Backend:**

- **Tauri 2.0** (Rust)
- **SQLite** database
- **VCS Integration Layer** - Git operations, GitHub/GitLab API integration
- **AI Execution SDK** - Amp AI orchestration

## Core Domains

### 1. Repositories

Repositories represent git repositories that executions target. They are stored as "admin clones" - bare repos used as a base for worktrees.

```rust
struct Repository {
    id: String,              // UUID
    provider: String,        // "github"
    provider_id: String,     // "owner/repo"
    name: Option<String>,    // User-friendly name override
    default_branch: Option<String>,
    last_synced_at: Option<i64>,
    created_at: i64,
}
```

**File System:**

- Admin repos: `{app_data_dir}/repos/owner/repo/.git/`
- Accessed via SSH using ssh-agent credentials

### 2. Prompt Sets

A Prompt Set is a named collection that organizes prompts and tracks which repositories to execute against.

```rust
struct PromptSet {
    id: String,                   // UUID
    name: String,
    repository_ids: Vec<String>,  // Target repositories
    validation_prompt: Option<String>,
    auto_validate: bool,
    created_at: i64,
}
```

**Relationships:**

- Has many **Prompt Revisions**
- Has many **Executions** (through revisions)

### 3. Prompt Revisions

A Prompt Revision is a version of a prompt within a Prompt Set. Revisions form a DAG through parent relationships.

```rust
struct PromptRevision {
    id: String,                         // UUID
    promptset_id: String,
    prompt_text: String,
    parent_revision_id: Option<String>, // DAG structure
    created_at: i64,
}
```

**Execution:**

- Each revision can be executed against some or all repositories in the prompt set
- Execution creates one Execution record per repository

### 4. Executions

An Execution represents running a specific prompt revision against a single repository.

```rust
struct Execution {
    id: String,                             // UUID
    promptset_id: String,
    revision_id: String,
    repository_id: String,

    // Amp session
    session_id: Option<String>,
    thread_url: Option<String>,

    // Execution state
    status: ExecutionStatus,                // pending, running, completed, failed, cancelled
    prompt_status: Option<PromptStatus>,
    prompt_result: Option<String>,

    // Validation
    validation_status: Option<ValidationStatus>,  // pending, running, passed, failed
    validation_thread_url: Option<String>,
    validation_result: Option<String>,

    // Change tracking
    files_added: i64,
    files_removed: i64,
    files_modified: i64,
    lines_added: i64,
    lines_removed: i64,

    // Commit tracking
    commit_status: CommitStatus,            // uncommitted, committed
    commit_sha: Option<String>,
    committed_at: Option<i64>,
    parent_sha: Option<String>,             // SHA before worktree changes
    branch: Option<String>,                 // maestro/{promptset:8}/{revision:8}/{execution:8}

    // CI tracking
    ci_status: Option<CiStatus>,            // pending, passed, failed, skipped, not_configured
    ci_checked_at: Option<i64>,
    ci_url: Option<String>,

    created_at: i64,
    completed_at: Option<i64>,
}
```

**Worktree Isolation:**

- Each execution gets its own git worktree at `{app_data_dir}/executions/{promptset_id}/{execution_id}/`
- Worktree is ephemeral - cleaned up after commit or cancellation
- Branch naming: `maestro/{promptsetId:8}/{revisionId:8}/{executionId:8}` (short hashes)

### 5. Validations

Validations are optional automated checks run after execution completes. If a prompt set has a `validation_prompt`, executions can trigger validation through another Amp session.

**State Flow:**

```
Execution completes → Trigger validation → validation_status: running
                                        → validation_thread_url set
                                        → Amp evaluates changes
                                        → validation_status: passed/failed
                                        → validation_result stored
```

## Data Flow Architecture

### 1. Frontend ↔ Backend Communication (IPC)

All communication uses typed Tauri commands through centralized `$lib/ipc.ts`:

```typescript
// Type-safe IPC wrapper
import * as ipc from "$lib/ipc"

const execution = await ipc.getExecution(id)
await ipc.validateExecution(id)
```

See [ipc-guide.md](./ipc-guide.md) for complete API reference.

### 2. Real-Time Updates (Event Bus)

Backend emits events that frontend subscribes to via `executionStore`:

```rust
// Backend emits events
emit_execution_status(app, execution_id, status)
emit_execution_validation(app, execution_id, validation_status, url)
emit_execution_commit(app, execution_id, commit_status, sha, timestamp)
emit_execution_ci(app, execution_id, ci_status, url)
```

```typescript
// Frontend subscribes
import { subscribeToExecutions, executionStore } from "$lib/stores/executionBus"
await subscribeToExecutions()

// Components get reactive updates via $derived
let executionsWithUpdates = $derived(
	executions.map((execution) => {
		const updates = $executionStore.get(execution.id)
		return updates ? { ...execution, ...updates } : execution
	})
)
```

See [execution-event-bus.md](./execution-event-bus.md) for details.

### 3. Svelte 5 Reactivity

Maestro uses Svelte 5 runes mode for reactive state:

```svelte
<script>
	let executions = $state<Execution[]>([])

	// Merge static data with live event bus updates
	let executionsWithUpdates = $derived(
		executions.map((e) => ({
			...e,
			...$executionStore.get(e.id),
		}))
	)
</script>
```

See [reactivity.md](./reactivity.md) for patterns and best practices.

## File System Layout

Maestro uses Tauri's app data directory:

- **macOS**: `~/Library/Application Support/dev.trly.maestro/`
- **Linux**: `~/.local/share/maestro/`
- **Windows**: `%APPDATA%\dev.trly.maestro\`

Override with `MAESTRO_CONFIG` environment variable.

```
{app_data_dir}/
├── repos/                    # Admin clones (permanent)
│   └── owner/
│       └── repo/.git/
├── executions/              # Worktrees (ephemeral)
│   └── {promptsetId}/
│       └── {executionId}/
└── maestro.db               # SQLite database
```

## Security Architecture

### Token Storage

Maestro uses the system keyring (via `keyring` crate) for secure credential storage:

```rust
// Service name: "dev.trly.maestro"
// Token keys: "amp_token", "github_token"

use crate::commands::tokens::get_token_value;

let github_token = get_token_value("github_token")?
    .ok_or_else(|| "GitHub token not configured")?;
```

**Benefits:**

- Tokens never stored in env vars, config files, or database
- OS-level encryption (Keychain on macOS, Secret Service on Linux)
- Retrieved at command execution time

### SSH Authentication

All git operations (clone, fetch, push) use SSH authentication via the system's ssh-agent.

See [ssh-authentication.md](./ssh-authentication.md) for setup.

## Concurrency & Safety

### Repository Locking

```rust
lazy_static! {
    static ref REPO_LOCKS: Mutex<HashMap<String, Arc<Mutex<()>>>> =
        Mutex::new(HashMap::new());
}
```

Ensures only one execution per repository can perform git operations at a time.

### Active Operation Tracking

```rust
lazy_static! {
    static ref ACTIVE_EXECUTIONS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref ACTIVE_VALIDATIONS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}
```

Prevents duplicate concurrent operations on same execution.

### Child Process Namespacing

Amp SDK processes are namespaced:

- Executions: `exec:{id}`
- Validations: `val:{id}`

Enables clean cancellation without interfering with other operations.

## Testing Strategy

### Rust Tests

```bash
make test       # or: cd src-tauri && cargo test
make test-watch # Watch mode (requires cargo-watch)
```

### TypeScript Type Checking

```bash
make check      # TypeScript + Rust checks
```

### Development

```bash
make dev        # or: cargo tauri dev
```

## Related Documentation

- **[IPC Guide](./ipc-guide.md)** - Type-safe backend communication
- **[Execution Event Bus](./execution-event-bus.md)** - Real-time updates
- **[Reactivity Guide](./reactivity.md)** - Svelte 5 patterns
- **[Prompt Sets](./prompt-sets.md)** - Prompt set domain
- **[Executions & Validations](./executions.md)** - Execution lifecycle
- **[Change Tracking](./change-tracking.md)** - Diff and stats
- **[CI Tracking](./ci-tracking.md)** - GitHub CI integration
