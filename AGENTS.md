# AGENTS.md - Maestro Codebase Guide

**Hybrid SvelteKit + Tauri 2.0 desktop app** for orchestrating AI prompts across multiple repositories using Amp AI.

## Quick Reference

### Commands
- `bun run dev` - Start Tauri desktop app in development mode
- `bun run build` - Build production installer  
- `bun run check` - TypeScript type checking (run before commits)
- `cargo test` - Run Rust tests in `src-tauri/` directory
- `cargo test <test_name>` - Run specific Rust test

### Architecture Overview
- **Frontend**: SvelteKit (Svelte 5) + adapter-static at `src/`
- **Backend**: Tauri 2.0 (Rust) at `src-tauri/src/`
- **Database**: SQLite via rusqlite at `src-tauri/src/db/`
- **Git**: git2-rs for native operations at `src-tauri/src/git/`
- **AI**: @sourcegraph/amp-sdk for execution agent
- **UI**: bits-ui primitives + lucide-svelte icons + Tailwind 4
- **Types**: TypeScript enums mirror Rust enums via serde

### Core Domain Models
1. **Prompt Sets** - Collections of prompt revisions with repository associations
2. **Prompt Revisions** - Versioned prompts executed against repositories
3. **Executions** - Individual runs of a revision with worktree isolation
4. **Validations** - Automated validation of execution results

## Key Architectural Patterns

### IPC Layer
All frontend-backend communication uses typed wrappers in `src/lib/ipc.ts`:
```typescript
// Never use invoke() directly - use typed wrappers
import { getExecution, executePrompt } from '$lib/ipc'
const execution = await getExecution(id)
```
See @docs/ipc-guide.md for complete API reference.

### Event Bus
Centralized execution events via `src/lib/stores/executionBus.ts`:
```typescript
// Subscribe once at app init in +layout.svelte
import { subscribeToExecutions } from '$lib/stores/executionBus'
subscribeToExecutions()
```
See @docs/execution-event-bus.md for event patterns.

### Diff Architecture
Unified diff access through backend module and frontend store:
- Backend: `src-tauri/src/git/diff.rs` for all diff operations
- Frontend: `src/lib/stores/diffStore.ts` for caching
See @docs/diff-architecture.md for details.

### Status Types
Use type-safe enums (not strings) for all statuses:
```rust
// Rust
use crate::types::{ExecutionStatus, ValidationStatus, CommitStatus};
```
```typescript
// TypeScript
import type { ExecutionStatus, ValidationStatus } from '$lib/types'
```

### UUID Strategy
- **Storage**: Full UUIDs in database and file paths
- **Display**: 8-char short hashes via `toShortHash()` utility
- **Git branches**: `maestro/{promptsetId:8}/{revisionId:8}/{executionId:8}`

### File System Layout
Maestro uses Tauri's app data directory for storage:
- **macOS**: `~/Library/Application Support/dev.trly.maestro/`
- **Linux**: `~/.local/share/maestro/`
- **Windows**: `%APPDATA%\dev.trly.maestro\`
- **Override**: Set `MAESTRO_CONFIG` env var for custom location

```
{app_data_dir}/
├── repos/                    # Admin clones (permanent)
│   └── owner/repo/.git/
├── executions/              # Worktrees (ephemeral)
│   └── {promptsetId}/
│       └── {executionId}/
└── maestro.db               # SQLite database
```

## Code Conventions

### TypeScript/Svelte
- Strict mode, tabs, no semicolons
- Use `$lib/` alias for imports
- camelCase for variables/functions
- PascalCase for components
- All types in `src/lib/types.ts`
- Never use native HTML dialogs/checkboxes - use bits-ui primitives

#### Svelte 5 Runes Mode (Critical)
- **ALWAYS** reference https://svelte.dev/llms-full.txt when working with Svelte components
- **ALWAYS** reference https://bits-ui.com/docs/llms.txt when working with bits-ui components
- See @docs/reactivity.md for detailed patterns and examples
- Use `$state` for reactive component state (NOT traditional stores)
- Use `$derived` for computed values
- Use `$props()` for component inputs
- **NEVER** create stores inside `{#each}` loops - use `$derived` instead
- **NEVER** subscribe to stores in scoped blocks - make data reactive at component top level

### Rust
- Standard rustfmt
- Use `anyhow::Result` for error handling
- snake_case for functions/variables
- Derive `Serialize, Deserialize` with `#[serde(rename_all = "camelCase")]` for IPC types
- Use utility helpers from `src-tauri/src/util/` (never duplicate parsing/path logic)

### Git Operations
- **Always** use `git2` library (never shell out to `git` command)
- **Always** use git worktrees for isolation (via `git worktree add/remove`)
- **Never** use `rm -rf` on worktree directories
- Use `REPO_LOCKS` mutex for concurrent safety
- See @docs/ssh-authentication.md for SSH key setup

## State Management Patterns

### Prevent Race Conditions
- Check `ACTIVE_EXECUTIONS`/`ACTIVE_VALIDATIONS` before starting work
- Use repo-level locking via `REPO_LOCKS` mutex
- Namespace child processes: `exec:{id}` for executions, `val:{id}` for validations
- App startup calls `reconcile_on_startup()` to reset stuck states

### Diff & Commit Tracking
- Store commit metadata (SHA, parent SHA, branch) immediately after worktree creation
- Regenerate diffs on-demand from admin repo (ephemeral approach)
- Worktrees can be safely cleaned up via `cleanup_execution()` command

### Diff Stats Calculation
- **Always calculated on-demand** - never stored in database
- **Committed executions**: Stats calculated from `get_committed_diff(parent_sha, commit_sha)`
- **Uncommitted executions**: Stats calculated from `get_worktree_diff(worktree_path)`
- **Frontend caching**: Stats cached per execution in `executionStats.ts` to avoid redundant IPC calls
- See @docs/diff-stats.md for complete architecture

### Reactivity with Event Bus
- Use `$derived` to merge event bus updates with local state arrays
- Never subscribe to stores per-item in loops - compute derived state at top level
- See @docs/reactivity.md for complete patterns and anti-patterns

## Documentation Index

For detailed guidance on specific topics, see:
- @docs/README.md - Documentation index
- @docs/reactivity.md - Svelte 5 runes mode and reactive patterns
- @docs/ipc-guide.md - IPC command reference and patterns
- @docs/execution-event-bus.md - Event handling architecture
- @docs/diff-architecture.md - Diff access patterns
- @docs/ssh-authentication.md - SSH setup for private repos

## Common Tasks

### Adding a new Tauri command
1. Add command handler in `src-tauri/src/commands/`
2. Register in `src-tauri/src/lib.rs` invoke_handler
3. Add typed wrapper in `src/lib/ipc.ts`
4. Use wrapper in components (never direct `invoke()`)

### Adding a new execution event
1. Emit event in `src-tauri/src/commands/executor_events.rs`
2. Add event type in `src/lib/stores/executionBus.ts`
3. Subscribe in `executionBus.ts` and update store

### Creating a new UI component
1. Use bits-ui primitives (Dialog, Checkbox, etc.) - never manual implementations
2. Place in `src/lib/components/ui/` or `src/lib/components/`
3. Import icons from `lucide-svelte`
4. Use Tailwind classes for styling
