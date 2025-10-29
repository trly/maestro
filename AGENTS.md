# AGENTS.md - Maestro Codebase Guide

**Hybrid SvelteKit + Tauri 2.0 desktop app** for orchestrating AI prompts across multiple repositories using Amp AI.

## Quick Reference

### Commands

**Primary (use these):**

- `make dev` or `cargo tauri dev` - Start Tauri desktop app in development mode
- `make build` or `cargo tauri build` - Build production installer
- `make test` or `cd src-tauri && cargo test` - Run Rust test suite
- `make check` - TypeScript/Rust checks (run before commits)
- make tidy - TypeScript/Rust checks (run after a successful check and prior to commits)
- `make icon` or `cargo tauri icon` - Generate app icons

**Advanced:**

- `cargo test <test_name>` - Run specific Rust test in `src-tauri/` directory
- `make test-watch` - Run Rust tests in watch mode (requires cargo-watch)
- `make clean` - Clean all build artifacts
- `make help` - Show all available commands

### Architecture Overview

- **Frontend**: SvelteKit (Svelte 5) + adapter-static at `src/`
- **Backend**: Tauri 2.0 (Rust) at `src-tauri/src/`
- **Database**: SQLite via rusqlite at `src-tauri/src/db/`
- **Git**: git2-rs for native operations at `src-tauri/src/git/`
- **AI**: @sourcegraph/amp-sdk for execution agent, Amp V2 API for thread analysis
- **UI**: bits-ui primitives + lucide-svelte icons + Tailwind 4
- **Types**: TypeScript enums mirror Rust enums via serde

### Core Domain Models

1. **Prompt Sets** - Collections of prompt revisions with repository associations
2. **Prompt Revisions** - Versioned prompts executed against repositories
3. **Executions** - Individual runs of a revision with worktree isolation
4. **Validations** - Automated validation of execution results
5. **Analyses** - Failure analysis across multiple executions using Amp V2 API
6. **Settings** - User preferences for editors, terminals, CI, and appearance

## Key Architectural Patterns

### IPC Layer

All frontend-backend communication uses typed wrappers in `src/lib/ipc.ts`:

**Rule:** Never use `invoke()` directly - always use typed wrappers. Source of truth is `src/lib/ipc.ts`.

### Event Bus

Centralized execution events via `src/lib/stores/executionBus.ts`:

**Usage Pattern:**

```typescript
import { getExecutionWithUpdates } from "$lib/stores/executions"

const executionStore = getExecutionWithUpdates(execution)
$: status = $executionStore.status // Auto-updates with events
```

### Diff Architecture

Unified diff access through backend module and frontend store:

- Backend: `src-tauri/src/git/diff.rs` for all diff operations
- Frontend: `src/lib/stores/diffStore.ts` for caching

**Key Behavior:**

- **Committed executions**: Diffs regenerated from `get_committed_diff(parent_sha, commit_sha)` in admin repo
- **Uncommitted executions**: Diffs from `get_worktree_diff(worktree_path)`
- Worktrees can be safely cleaned up after commit (diffs regenerated from git history)

### Status Types

Use type-safe enums (not strings) for all statuses:

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

## Styling & Theming

Maestro uses **Gruvbox color scheme** with Tailwind 4's CSS variable system for automatic light/dark mode switching.

**Two-Layer System:**

1. **Raw Gruvbox Palette** ([tailwind.config.js](file:///Users/trly/src/github.com/sourcegraph/maestro/tailwind.config.js))
   - `gruvbox.light.*` and `gruvbox.dark.*` color definitions
   - Background layers: `bg0` (base) → `bg4` (elevated)
   - Foreground layers: `fg0` (primary text) → `fg4` (muted text)
   - Accent colors: `red`, `green`, `yellow`, `blue`, `purple`, `aqua`, `orange`, `gray`

2. **Semantic CSS Variables** ([app.css](file:///Users/trly/src/github.com/sourcegraph/maestro/src/app.css))
   - Maps Gruvbox colors to semantic tokens using `@theme` directive
   - Light mode defaults (e.g., `--color-background: #f9f5d7`)
   - Dark mode overrides via `.dark` class
   - Tokens: `background`, `foreground`, `border`, `card`, `muted`, `primary`, `secondary`, `destructive`, `warning`, `success`

**Always use semantic tokens, never raw Gruvbox colors:**

Theme managed by [themeStore.svelte.ts](file:///Users/trly/src/github.com/trly/maestro/src/lib/stores/themeStore.svelte.ts):

### Typography

- **Font**: JetBrains Mono (monospace)
- **Base size**: `0.8125rem` (13px)
- **Line height**: `1.25rem`

### Status Icon/Color Mappings (Use These Everywhere)

**Execution Status:**

- `pending` → `text-muted-foreground` (gray) + `Clock` icon
- `running` → `text-primary animate-spin` (blue) + `Loader2` icon
- `completed` → `text-success` (green) + `CheckCircle2` icon
- `failed` → `text-destructive` (red) + `XCircle` icon
- `cancelled` → `text-warning` (orange) + `Ban` icon

**Validation Status:**

- `pending` → `text-muted-foreground` (gray) + `Clock` icon
- `running` → `text-primary animate-spin` (blue) + `Loader2` icon
- `passed` → `text-success` (green) + `CheckCircle2` icon
- `failed` → `text-destructive` (red) + `XCircle` icon

**Action Button Colors:**

- Start/Validate → `text-success hover:text-success/90`
- Stop/Cancel → `text-warning hover:text-warning/90`
- Delete → `text-destructive hover:text-destructive/90`
- Restart/Retry/Edit/Push/Analyze → `text-primary hover:text-primary/90`

**Note:** `text-accent` should only be used for disabled button states, never for active icon buttons.

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
  - components should always be built on bits-ui primitives before writing a custom implementation
- Use `$state` for reactive component state (NOT traditional stores)
- Use `$derived` for computed values
- Use `$props()` for component inputs
- Use `$effect` for side effects with automatic cleanup
- **Components are dynamic by default** - use `<Component />` directly instead of `<svelte:component this={Component} />`

### Rust

- Standard rustfmt
- Use `anyhow::Result` for error handling
- snake_case for functions/variables
- Derive `Serialize, Deserialize` with `#[serde(rename_all = "camelCase")]` for IPC types
- Use utility helpers from `src-tauri/src/util/` (never duplicate parsing/path logic)

#### Enum Serialization Rules (Critical)

When adding status enums that cross the Rust/TypeScript boundary:

- Use `#[serde(rename_all = "snake_case")]` for enums with **any** multi-word variants
- Use `#[serde(rename_all = "lowercase")]` only for enums with **all** single-word variants
- TypeScript types must exactly match serialized format (e.g., `'not_configured'` not `'notConfigured'`)
- Database storage (ToSql/FromSql) must match serde serialization
- When adding new enum variants, verify TypeScript type alignment

#### Logging Conventions

Use logging sparingly - only for critical information, not routine operations:

- Use `log::error!` for critical failures that need investigation
- Use `log::warn!` for recoverable issues or API failures
- Use `log::info!` sparingly for important milestones (e.g., repository configuration changes)
- **Never** log routine state changes, routine API calls, or success confirmations
- Prefix error logs with `[function_name]` for context (e.g., `[execute_prompt]`)
- User-facing status updates should go through the event bus, not logs

#### Thread-Safe Shared Ownership with Arc

Use `Arc<T>` (Atomic Reference Counted) for sharing data across async tasks/threads:

**When to use Arc:**

- Passing trait objects across async boundaries: `Arc<dyn CiProvider>`
- Sharing read-only data across multiple async tasks
- Required by functions that might spawn concurrent operations

### Git Operations

- **Always** use `git2` library (never shell out to `git` command)
- **Always** use git worktrees for isolation (via `git worktree add/remove`)
- **Never** use `rm -rf` on worktree directories
- Use `REPO_LOCKS` mutex for concurrent safety
- SSH authentication via `git2::Cred::ssh_key_from_agent()` - requires SSH key in ssh-agent

## Settings & Configuration

### Settings Store

User settings are loaded globally at app startup in `+layout.svelte`:

#### Secure Token Storage

Maestro uses the system keyring (via `keyring` crate) for secure credential storage:

- **Service Name**: `dev.trly.maestro` (identifies app in keyring)
- **Token Keys**: `amp_token`, `github_token`, `sourcegraph_endpoint`, `sourcegraph_token`, `amp_client_id`, `amp_client_secret`
- **Never** store tokens in env vars, config files, or database
- **Always** retrieve tokens at command execution time via `tokens::get_token_value(key)`

#### Token Access Pattern

```rust
// In Tauri commands - retrieve token from keyring
use crate::commands::tokens::get_token_value;

let github_token = get_token_value("github_token")
    .map_err(|e| format!("Failed to access token: {}", e))?
    .ok_or_else(|| "GitHub token not configured".to_string())?;
```

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

### CI Integration with GitHub API

#### Octocrab Usage

- **GitHub API client**: Use `octocrab` crate for all GitHub API interactions
- **Authentication**: Create provider with personal access token from keyring
- **Provider pattern**: Wrap API logic in `GitHubProvider` (see `src-tauri/src/ci/github_provider.rs`)
- **Arc wrapping**: Wrap provider in `Arc` when passing to async functions that expect `Arc<dyn CiProvider>`

#### CI Status Retrieval

- **Dual API approach**: Check both GitHub Status API and Checks API
- **Status API**: Legacy CI systems (Jenkins, CircleCI, external webhooks)
- **Checks API**: GitHub Actions and modern integrations
- **Aggregation**: Merge results from both APIs to determine overall CI status

#### CI Events via Event Bus

- CI status updates emit events like execution status: `emit_execution_ci(app, execution_id, status, url)`
- Frontend subscribes via `executionBus` and updates UI reactively
- CI status is persisted to database for historical tracking
