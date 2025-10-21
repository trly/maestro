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
See @docs/change-tracking.md for details.

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

## Styling & Theming

Maestro uses **Gruvbox color scheme** with Tailwind 4's CSS variable system for automatic light/dark mode switching.

### Theme Architecture

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

### Using Semantic Tokens in Components

**Always use semantic tokens, never raw Gruvbox colors:**

```svelte
<!-- ✅ DO: Use semantic tokens -->
<div class="bg-background text-foreground border-border">
  <button class="bg-primary text-primary-foreground hover:bg-primary/90">
    Save
  </button>
  <span class="text-muted-foreground">Optional</span>
</div>

<!-- ❌ DON'T: Use raw Gruvbox colors -->
<div class="bg-gruvbox-dark-bg0 text-gruvbox-dark-fg0">
```

**Common Semantic Tokens:**
- `bg-background` / `text-foreground` - Base app background/text
- `bg-card` / `text-card-foreground` - Card/panel backgrounds
- `bg-muted` / `text-muted-foreground` - Subtle/disabled elements
- `bg-primary` / `text-primary-foreground` - Primary actions (blue)
- `bg-destructive` / `text-destructive-foreground` - Destructive actions (red)
- `bg-success` / `text-success-foreground` - Success states (green)
- `bg-warning` / `text-warning-foreground` - Warning states (yellow)
- `border-border` - Default borders
- `bg-accent` / `text-accent-foreground` - Hover states

### Theme Switching

Theme managed by [themeStore.svelte.ts](file:///Users/trly/src/github.com/trly/maestro/src/lib/stores/themeStore.svelte.ts):

```typescript
import { themeStore } from '$lib/stores/themeStore.svelte'

// Set theme (stores in localStorage)
await themeStore.setTheme('dark')   // Force dark mode
await themeStore.setTheme('light')  // Force light mode
await themeStore.setTheme('auto')   // Follow system preference

// Initialize on app startup (in +layout.svelte)
await themeStore.init()
```

**How it works:**
- Toggles `.dark` class on `<html>` element
- CSS variables automatically switch via `.dark { ... }` rules
- Syncs with Tauri native window theme
- Listens to system theme changes when in "auto" mode

### Typography

- **Font**: JetBrains Mono (monospace)
- **Base size**: `0.8125rem` (13px)
- **Line height**: `1.25rem`

### Styling Best Practices

1. **Use Tailwind utilities** - Avoid custom CSS when possible
2. **Semantic tokens only** - **NEVER** use raw colors like `text-blue-600`, `bg-red-500`, etc. **ALWAYS** use semantic tokens
3. **Consistent spacing** - Use Tailwind's spacing scale (`p-4`, `gap-2`, etc.)
4. **Hover states** - Use `hover:` variants with opacity modifiers (`bg-primary/90`)
5. **Border radius** - Use CSS variables: `rounded-[var(--radius)]` or `rounded-[var(--radius-sm)]`
6. **Focus rings** - Use `focus-visible:ring-2 focus-visible:ring-ring`

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
- Restart/Retry/Edit → `text-primary hover:text-primary/90`
- Push/Analyze → `text-accent hover:text-accent/90`

**Never Use Raw Colors:**
- ❌ `text-green-600`, `text-red-500`, `text-blue-700`
- ❌ `bg-gray-50`, `bg-green-100`, `bg-red-100`
- ✅ Use semantic tokens instead: `text-success`, `text-destructive`, `bg-muted`

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
- **Components are dynamic by default** - use `<Component />` directly instead of `<svelte:component this={Component} />`
- **NEVER** destructure props if they need to be reactive - keep as single object and access via `props.fieldName`
- **NEVER** create stores inside `{#each}` loops - use `$derived` instead
- **NEVER** subscribe to stores in scoped blocks - make data reactive at component top level

**Props Pattern:**
```svelte
<!-- ✅ DO: Keep props as object for reactivity -->
<script lang="ts">
  let props: { status: string; count: number } = $props();
  let doubled = $derived(props.count * 2);
</script>

<!-- ❌ DON'T: Destructure props (breaks reactivity) -->
<script lang="ts">
  let { status, count } = $props();
  let doubled = $derived(count * 2); // Won't update when count changes!
</script>
```

### Rust
- Standard rustfmt
- Use `anyhow::Result` for error handling
- snake_case for functions/variables
- Derive `Serialize, Deserialize` with `#[serde(rename_all = "camelCase")]` for IPC types
- Use utility helpers from `src-tauri/src/util/` (never duplicate parsing/path logic)

#### Enum Serialization Rules (Critical)
When adding status enums that cross the Rust/TypeScript boundary:

```rust
// ✅ DO: Use snake_case for enums with multi-word variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiStatus {
    Pending,
    Passed,
    Failed,
    NotConfigured,  // Serializes as "not_configured"
}
```

```rust
// ✅ ALSO OK: Use lowercase for single-word variants only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,    // Serializes as "pending"
    Running,    // Serializes as "running"
    Completed,  // Serializes as "completed"
}
```

**Guidelines:**
- Use `#[serde(rename_all = "snake_case")]` for enums with **any** multi-word variants
- Use `#[serde(rename_all = "lowercase")]` only for enums with **all** single-word variants
- TypeScript types must exactly match serialized format (e.g., `'not_configured'` not `'notConfigured'`)
- Database storage (ToSql/FromSql) must match serde serialization
- When adding new enum variants, verify TypeScript type alignment

#### Logging Conventions
Use logging sparingly - only for critical information, not routine operations:

```rust
// ✅ DO: Log errors and warnings
log::error!("[execute_prompt] Execution {} failed: {}", execution_id, e);
log::warn!("Failed to fetch commit statuses for {}/{} @ {}: {:?}", owner, repo, sha, e);

// ✅ DO: Log important milestones (sparingly)
log::info!("[execute_prompt] GitHub default branch for {}/{}: {}", owner, repo, branch);

// ❌ DON'T: Log routine operations
log::info!("Found {} check runs for {}/{}", count, owner, repo);  // Too verbose
log::info!("CI status set to pending");  // Routine state change
log::info!("Started CI checking");  // Redundant with state changes
```

**Guidelines:**
- Use `log::error!` for critical failures that need investigation
- Use `log::warn!` for recoverable issues or API failures
- Use `log::info!` sparingly for important milestones (e.g., repository configuration changes)
- **Never** log routine state changes, routine API calls, or success confirmations
- Prefix error logs with `[function_name]` for context (e.g., `[execute_prompt]`)
- User-facing status updates should go through the event bus, not logs

#### Thread-Safe Shared Ownership with Arc
Use `Arc<T>` (Atomic Reference Counted) for sharing data across async tasks/threads:

```rust
use std::sync::Arc;

// Wrap provider in Arc for thread-safe sharing
let provider = Arc::new(GitHubProvider::new(token)?);

// Can be cloned cheaply (just increments counter)
let provider_clone = Arc::clone(&provider);
```

**When to use Arc:**
- Passing trait objects across async boundaries: `Arc<dyn CiProvider>`
- Sharing read-only data across multiple async tasks
- Required by functions that might spawn concurrent operations

**Key characteristics:**
- **Thread-safe**: Uses atomic operations for reference counting
- **Cheap cloning**: Only increments counter, doesn't copy data
- **Automatic cleanup**: Data freed when last Arc is dropped
- **Immutable by default**: Use `Arc<Mutex<T>>` for mutable shared state

### Git Operations
- **Always** use `git2` library (never shell out to `git` command)
- **Always** use git worktrees for isolation (via `git worktree add/remove`)
- **Never** use `rm -rf` on worktree directories
- Use `REPO_LOCKS` mutex for concurrent safety
- See @docs/ssh-authentication.md for SSH key setup

## Settings & Configuration

### Settings Store
User settings are loaded globally at app startup in `+layout.svelte`:
```typescript
import { settingsStore } from '$lib/stores/settingsStore'

onMount(async () => {
  await settingsStore.load()  // Must be called on startup
})
```

**Available Settings:**
- `ciStuckThresholdMinutes` - CI timeout threshold (default: 10 minutes)
- `selectedEditor` - Preferred editor command (e.g., "nvim", "code")
- `selectedTerminal` - Terminal for vim/nvim (e.g., "ghostty", "open -a Terminal")
- `editorCommand` - Legacy editor setting

### Editor & Terminal Detection
Maestro detects available editors/terminals on startup:
```typescript
// Frontend
const editors = await ipc.getAvailableEditors()  // vim, nvim, code, cursor, zed
const terminals = await ipc.getAvailableTerminals()  // Terminal.app, Ghostty (macOS)

// Check if editor needs terminal
const editorInfo = editors.find(e => e.command === 'nvim')
if (editorInfo?.needsTerminal) {
  // Requires terminal selection
}
```

**Backend:** Uses `which` command to check PATH (`src-tauri/src/commands/app_check.rs`)

### Opening Worktrees
Always use `openInEditor` utility which reads settings:
```typescript
import { openInEditor } from '$lib/utils/worktree'

await openInEditor(execution)  // Uses configured editor/terminal
```

**Critical:** Settings must be loaded in `+layout.svelte` before opening editors, otherwise falls back to default `'code'`.

See @docs/settings.md for complete settings documentation.

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
- See @docs/change-tracking-stats.md for complete architecture

### Reactivity with Event Bus
- Use `$derived` to merge event bus updates with local state arrays
- Never subscribe to stores per-item in loops - compute derived state at top level
- See @docs/reactivity.md for complete patterns and anti-patterns

## Token Management & Security

### Secure Token Storage
Maestro uses the system keyring (via `keyring` crate) for secure credential storage:
- **Service Name**: `dev.trly.maestro` (identifies app in keyring)
- **Token Keys**: `amp_token`, `github_token`, `sourcegraph_endpoint`, `sourcegraph_token`, `amp_client_id`, `amp_client_secret`
- **Never** store tokens in env vars, config files, or database
- **Always** retrieve tokens at command execution time via `tokens::get_token_value(key)`

### Token Access Pattern
```rust
// In Tauri commands - retrieve token from keyring
use crate::commands::tokens::get_token_value;

let github_token = get_token_value("github_token")
    .map_err(|e| format!("Failed to access token: {}", e))?
    .ok_or_else(|| "GitHub token not configured".to_string())?;
```

### CI Integration with GitHub API

#### Octocrab Usage
- **GitHub API client**: Use `octocrab` crate for all GitHub API interactions
- **Authentication**: Create provider with personal access token from keyring
- **Provider pattern**: Wrap API logic in `GitHubProvider` (see `src-tauri/src/ci/github_provider.rs`)
- **Arc wrapping**: Wrap provider in `Arc` when passing to async functions that expect `Arc<dyn CiProvider>`

```rust
use std::sync::Arc;
use crate::ci::GitHubProvider;

// Wrap in Arc for thread-safe sharing across async operations
let provider = Arc::new(GitHubProvider::new(github_token)?);
check_ci_once(provider, ctx).await // Expects Arc<dyn CiProvider>
```

#### CI Status Retrieval
- **Dual API approach**: Check both GitHub Status API and Checks API
- **Status API**: Legacy CI systems (Jenkins, CircleCI, external webhooks)
- **Checks API**: GitHub Actions and modern integrations
- **Aggregation**: Merge results from both APIs to determine overall CI status

#### CI Events via Event Bus
- CI status updates emit events like execution status: `emit_execution_ci(app, execution_id, status, url)`
- Frontend subscribes via `executionBus` and updates UI reactively
- CI status is persisted to database for historical tracking

## Documentation Index

For detailed guidance on specific topics, consult the domain-focused documentation:

**Core Architecture & Domains:**
- @docs/README.md - Complete documentation index with "when to read" guidance
- @docs/architecture.md - System architecture, technology stack, data flow, security, concurrency patterns
- @docs/prompt-sets.md - Prompt sets, revisions, repositories, DAG structure, execution triggering
- @docs/executions.md - Execution lifecycle, validations, commits, cancellation, bulk operations
- @docs/analyses.md - Failure analysis using Amp V2 API, OAuth2 authentication, categorizing failures
- @docs/change-tracking.md - Diff architecture, worktree vs committed diffs, frontend caching
- @docs/ci-tracking.md - GitHub CI integration, status aggregation, polling strategy

**Infrastructure & Patterns:**
- @docs/ipc-guide.md - Type-safe IPC layer, complete API reference, error handling
- @docs/execution-event-bus.md - Real-time event system, subscription patterns, reactive updates
- @docs/reactivity.md - Svelte 5 runes mode, event bus integration, anti-patterns

**Setup & Additional:**
- @docs/settings.md - User settings, editor/terminal detection, configuration patterns
- @docs/ssh-authentication.md - SSH setup for git operations, troubleshooting
- @docs/change-tracking-stats.md - Diff statistics calculation details
- @docs/distribution.md - Code signing, notarization, release process
- @docs/sourcegraph-integration.md - Sourcegraph repository search, GraphQL API, configuration

## Common Tasks

### Adding a new Tauri command
1. Add command handler in `src-tauri/src/commands/`
2. Register in `src-tauri/src/lib.rs` invoke_handler
3. Add typed wrapper in `src/lib/ipc.ts`
4. Add helper in `src/lib/tauri-api.ts` (optional, for convenience)
5. Use wrapper in components (never direct `invoke()`)

**Example**: Adding CI refresh command
- Backend: `commands/ci.rs::refresh_ci_status(execution_id)`
- IPC: `ipc.ts::refreshCiStatus(executionId)`
- API: `tauri-api.ts::api.ci.refreshStatus(id)`
- UI: `+page.svelte::refreshCiManually(execution)`

### Adding a new execution event
1. Emit event in `src-tauri/src/commands/executor_events.rs`
2. Add event type in `src/lib/stores/executionBus.ts`
3. Subscribe in `executionBus.ts` and update store

### Adding external API integration
1. **Add provider trait** in `src-tauri/src/<domain>/provider.rs`
2. **Implement provider** using appropriate API client (`octocrab`, `reqwest`, etc.)
3. **Retrieve tokens from keyring** in command handlers using `tokens::get_token_value()`
4. **Emit events** for UI updates via event bus
5. **Update database** to persist state changes

**Example**: CI status checking (see `src-tauri/src/ci/`)
- Trait: `CiProvider` with `check_status()` method
- Implementation: `GitHubProvider` using `octocrab`
- Token: Retrieved from keyring at command execution time
- Event: `emit_execution_ci()` for real-time UI updates

**Example**: Sourcegraph repository search (see `src-tauri/src/sourcegraph/`)
- Client: `SourcegraphClient` with GraphQL integration
- Implementation: Direct `reqwest` HTTP client for GraphQL API
- Tokens: Both endpoint and access token retrieved from keyring
- Usage: `search_sourcegraph_repositories(query, limit)` command

**Example**: Amp V2 API integration (see `src-tauri/src/amp/`)
- Client: `AmpV2Client` with OAuth2 authentication
- Implementation: `reqwest` HTTP client for REST API
- Tokens: `amp_client_id` and `amp_client_secret` from keyring
- Usage: Fetch thread messages for failure analysis

### Adding analysis features
1. **Create domain model** in `src-tauri/src/types.rs` (e.g., `Analysis` struct)
2. **Add database tables** via migration in `src-tauri/src/db/migrations.rs`
3. **Implement CRUD operations** in `src-tauri/src/db/store.rs`
4. **Add command handlers** in `src-tauri/src/commands/`
5. **Create IPC wrappers** in `src/lib/ipc.ts`
6. **Add TypeScript types** in `src/lib/types.ts`
7. **Build UI component** in `src/lib/components/ui/`
8. **Integrate in parent page** to fetch and display data

**Example**: Failure analysis (see `src-tauri/src/amp/`, `src-tauri/src/commands/analysis.rs`)
- Domain: `Analysis` with type (execution/validation) and status
- Tables: `analyses` + `analysis_executions` join table
- Commands: `create_analysis`, `run_analysis`, `get_analyses_by_revision`, `delete_analysis`
- UI: `AnalysisResult.svelte` component displays results with delete/rerun actions
- Trigger: ScanSearch icon in column headers when failures exist
- Management: Delete and re-run buttons in AnalysisResult component

### Creating a new UI component
1. Use bits-ui primitives (Dialog, Checkbox, etc.) - never manual implementations
2. Place in `src/lib/components/ui/` or `src/lib/components/`
3. Import icons from `lucide-svelte`
4. **ALWAYS** use semantic Tailwind tokens for styling - **NEVER** raw colors (see Status Icon/Color Mappings section above)

### Adding bulk operations
1. **Add handler in parent component** that operates on array of items
2. **Pass handler as prop** to child component with `onBulk*` naming convention
3. **Show bulk action toolbar** when items are selected
4. **Clear selection** after bulk operation completes

**Example**: Bulk CI refresh (see `RevisionDetail.svelte`)
```typescript
async function refreshAllCi() {
  const committed = executions.filter(e => e.commitStatus === 'committed');
  await Promise.all(committed.map(e => api.ci.refreshStatus(e.id)));
}
```
