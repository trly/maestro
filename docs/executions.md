# Executions & Validations

An **Execution** represents running a specific prompt revision against a single repository. Executions can optionally be validated after completion.

## Execution Lifecycle

### 1. Creation & Worktree Setup

```typescript
// Execute a revision against selected repositories
const executionIds = await ipc.executePromptSet(
	promptSet.id,
	revision.id,
	["repo-uuid-1", "repo-uuid-2"] // Optional subset
)
```

**Backend Flow:**

1. For each target repository:
   - Creates `Execution` record with status "pending"
   - Acquires repository lock
   - Ensures admin repo exists and is up-to-date
   - Creates worktree at `{app_data_dir}/executions/{promptsetId}/{executionId}/`
   - Stores `parent_sha` (current HEAD before changes)
   - Creates branch `maestro/{promptset:8}/{revision:8}/{execution:8}`
   - Launches Amp session with prompt

### 2. Running State

```rust
ExecutionStatus::Running
```

**Events Emitted:**

- `execution:session` - Session ID and thread URL
- `execution:status` - Status changes
- `execution:progress` - Progress messages from Amp

**Frontend Reactivity:**

```typescript
import { getExecutionWithUpdates } from "$lib/stores"

const executionStore = getExecutionWithUpdates(execution)
$: status = $executionStore.status
$: threadUrl = $executionStore.threadUrl
```

### 3. Completion

```rust
ExecutionStatus::Completed
```

**What Happens:**

- Change statistics calculated from worktree diff
- `completed_at` timestamp recorded
- Auto-validation triggered if `auto_validate: true`

### 4. Validation (Optional)

If prompt set has a `validation_prompt`, executions can be validated:

```typescript
await ipc.validateExecution(execution.id)
```

**Validation Flow:**

1. Fetches worktree diff
2. Creates new Amp session with validation prompt + diff context
3. Sets `validation_status: running`
4. Emits `execution:validation` events
5. Stores validation result and final status (passed/failed)

**Validation Prompt Example:**

```
Check if these changes:
1. Pass all existing tests
2. Don't introduce TypeScript errors
3. Follow code style guidelines

[Diff context automatically included]
```

### 5. Commit

```typescript
// Commit all changes
await ipc.commitChanges(execution.id)

// Or commit specific files
await ipc.commitChanges(execution.id, ["src/main.ts", "package.json"])
```

**Backend Flow:**

1. Generates commit message (from Amp or custom)
2. Stages selected files
3. Creates commit in worktree
4. Stores `commit_sha` in database
5. Sets `commit_status: committed`
6. Emits `execution:commit` event

**Note:** Worktree can be cleaned up after commit - diff regenerated from `parent_sha` and `commit_sha`.

### 6. Push & CI Tracking

```typescript
await ipc.pushCommit(execution.id, false) // force = false
```

**Backend Flow:**

1. Pushes branch to remote via SSH
2. Sets `ci_status: pending`
3. Optionally starts CI polling (manual refresh available)

See [ci-tracking.md](./ci-tracking.md) for CI integration details.

### 7. Cleanup

```typescript
await ipc.cleanupExecution(execution.id)
```

**Removes:**

- Git worktree
- File system directory
- Leaves database record intact

**Safe After:**

- Execution cancelled
- Execution committed (diff can be regenerated from admin repo)

## Execution Domain Model

```rust
struct Execution {
    // Identity
    id: String,
    promptset_id: String,
    revision_id: String,
    repository_id: String,

    // Amp session
    session_id: Option<String>,       // Amp session UUID
    thread_url: Option<String>,        // ampcode.com/threads/T-...

    // Execution state
    status: ExecutionStatus,           // pending, running, completed, failed, cancelled
    prompt_status: Option<PromptStatus>,
    prompt_result: Option<String>,     // Final Amp response

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
    commit_status: CommitStatus,       // uncommitted, committed
    commit_sha: Option<String>,
    committed_at: Option<i64>,
    parent_sha: Option<String>,        // SHA before changes
    branch: Option<String>,

    // CI tracking
    ci_status: Option<CiStatus>,       // pending, passed, failed, not_configured
    ci_checked_at: Option<i64>,
    ci_url: Option<String>,            // GitHub checks page URL

    created_at: i64,
    completed_at: Option<i64>,
}
```

## Status Types

### ExecutionStatus

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### ValidationStatus

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ValidationStatus {
    Pending,
    Running,
    Passed,
    Failed,
}
```

### CommitStatus

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CommitStatus {
    Uncommitted,
    Committed,
}
```

### CiStatus

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CiStatus {
    Pending,
    Passed,
    Failed,
    NotConfigured,  // Serializes as "not_configured"
}
```

## Cancellation & Resume

### Stop Execution

```typescript
await ipc.stopExecution(execution.id)
```

**What Happens:**

- Kills Amp child process (namespace: `exec:{id}`)
- Sets status to "cancelled"
- Preserves worktree (can be resumed or cleaned up)

### Stop Validation

```typescript
await ipc.stopValidation(execution.id)
```

**What Happens:**

- Kills validation Amp child process (namespace: `val:{id}`)
- Sets `validation_status` to null
- Execution state unchanged

### Resume Execution

```typescript
await ipc.resumeExecution(execution.id)
```

**Use Cases:**

- Execution was stopped manually
- Execution failed but worktree is still valid
- Want to re-run with same context

**Requirements:**

- Worktree must exist
- Execution status must be "cancelled" or "failed"

## Bulk Operations

### Stop All Executions in Revision

```typescript
await ipc.stopAllExecutions(revision.id)
```

Stops all running executions for a specific revision.

### Stop All Validations in Revision

```typescript
await ipc.stopAllValidations(revision.id)
```

Stops all running validations for a specific revision.

## Query API

### Get Single Execution

```typescript
const execution = await ipc.getExecution(id)
```

### Find by ID Prefix

```typescript
const execution = await ipc.findExecutionByPrefix("abc12345")
```

### Get Executions by Prompt Set

```typescript
const executions = await ipc.getExecutionsByPromptSet(promptset.id)
```

### Get Executions by Revision

```typescript
const executions = await ipc.getExecutionsByRevision(revision.id)
```

## Real-Time Updates

Subscribe to execution events once at app initialization:

```typescript
// +layout.svelte
import { subscribeToExecutions } from "$lib/stores/executionBus"
import { onMount, onDestroy } from "svelte"

onMount(async () => {
	await subscribeToExecutions()
})

onDestroy(() => {
	unsubscribeFromExecutions()
})
```

Components automatically receive updates:

```svelte
<script>
	import { getExecutionWithUpdates } from "$lib/stores"

	let { execution } = $props()

	const executionStore = getExecutionWithUpdates(execution)
</script>

<div>
	<h3>{$executionStore.status}</h3>
	{#if $executionStore.threadUrl}
		<a href={$executionStore.threadUrl}>View in Amp</a>
	{/if}
</div>
```

See [execution-event-bus.md](./execution-event-bus.md) for complete event bus documentation.

## File System Structure

### Worktree Layout

```
{app_data_dir}/executions/{promptsetId}/{executionId}/
├── .git           # Worktree metadata
├── src/           # Repository files
├── package.json
└── ...
```

### Branch Naming

Format: `maestro/{promptsetId:8}/{revisionId:8}/{executionId:8}`

Example: `maestro/a1b2c3d4/e5f6g7h8/i9j0k1l2`

**Why short hashes?**

- Readable in git logs
- Shorter branch names
- Full UUIDs stored in database

## Concurrency Safety

### Repository Locking

The backend ensures only one execution per repository performs git operations at a time through repository-level locking.

### Active Execution Tracking

The system prevents starting duplicate executions for the same execution ID through active execution tracking.

### Process Isolation

The AI execution system uses namespaces to isolate processes:

- Executions: `exec:{executionId}`
- Validations: `val:{executionId}`

This enables clean cancellation without interfering with other operations.

## Error Handling

### Common Errors

**Git errors:**

- Worktree already exists
- Admin repo not found
- SSH authentication failed

**Amp errors:**

- Token not configured
- Session creation failed
- Session interrupted

**State errors:**

- Execution already running
- Validation triggered without validation_prompt
- Commit without changes

### Reconciliation on Startup

On app startup, the system resets stuck states:

- Running executions → Failed
- Running validations → null

This prevents orphaned "running" states from crashed sessions.

## Best Practices

### When to Commit

✅ **Commit when:**

- Changes are validated (manually or automatically)
- Ready for code review
- Want to trigger CI

❌ **Don't commit if:**

- Execution failed
- Changes are incomplete
- Validation failed (unless intentional)

### When to Cleanup

✅ **Cleanup when:**

- Execution cancelled and won't resume
- After commit (if disk space needed)
- Execution failed and worktree is corrupted

❌ **Don't cleanup if:**

- Might want to resume
- Want to manually inspect changes
- Haven't committed yet

### Validation Strategy

**Fast validation:**

```
Check if code compiles and no lint errors
```

**Thorough validation:**

```
1. Run test suite
2. Check TypeScript types
3. Verify no security issues
4. Ensure follows style guide
```

**Validation with context:**

```
These changes implement JWT authentication.
Verify:
- Tokens are properly validated
- Refresh token flow works
- Tests cover edge cases
```

## Implementation Reference

**Backend Modules:**

- Execution commands - IPC interface for execution lifecycle
- Event emission - Real-time execution updates
- Git service - Worktree and branch management
- AI integration - Execution and validation sessions

**Frontend Modules:**

- Event bus - Real-time execution updates
- Stats fetching - On-demand statistics
- Execution routes - UI pages
- Execution components - Reusable UI elements

## Related Documentation

- **[Architecture](./architecture.md)** - Overall system design
- **[Prompt Sets](./prompt-sets.md)** - Prompt set domain
- **[Change Tracking](./change-tracking.md)** - Diff architecture
- **[CI Tracking](./ci-tracking.md)** - GitHub CI integration
- **[Execution Event Bus](./execution-event-bus.md)** - Real-time updates
