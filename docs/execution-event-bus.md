# Execution Event Bus

Detailed architecture of the real-time event system for execution state updates. **For common usage patterns, see AGENTS.md Event Bus section.**

## When to Read This

- Understanding event bus implementation details
- Debugging event subscription issues
- Adding new event types to the system

## Architecture

### Event Bus (`executionBus.ts`)

Single writable store keyed by execution ID:

```typescript
// Internal store structure
{
  [executionId]: {
    sessionId?: string,
    sessionUrl?: string,
    status?: ExecutionStatus,
    validationStatus?: ValidationStatus,
    validationThreadUrl?: string,
    commitStatus?: CommitStatus,
    commitSha?: string,
    committedAt?: string,
    progress?: string
  }
}
```

### API

```typescript
// Initialize (called once in +layout.svelte)
await subscribeToExecutions()

// Get current state
const data = getExecutionStatus(executionId)

// Subscribe to updates
const unsubscribe = onExecutionUpdate(executionId, (data) => { ... })

// Cleanup specific execution
clearExecutionData(executionId)

// Shutdown
unsubscribeFromExecutions()
```

### Helper Store (`executions.ts`)

Provides derived stores that merge execution data with live updates:

```typescript
export function getExecutionWithUpdates(execution: Execution) {
	return derived(executionStore, ($updates) => ({
		...execution,
		...($updates[execution.id] || {}),
	}))
}
```

## Backend Events

Events emitted from Rust backend (`executor_events.rs`):

```rust
emit_execution_session(app, execution_id, session_id, thread_url)
emit_execution_status(app, execution_id, status)
emit_execution_validation(app, execution_id, validation_status, validation_thread_url?)
emit_execution_commit(app, execution_id, commit_status, commit_sha?, committed_at?)
emit_execution_ci(app, execution_id, ci_status, ci_url?)
```

All events include `executionId` for store keying.

## Benefits

- **Single Source of Truth**: One store for all execution state
- **Memory Safe**: Proper cleanup prevents listener leaks
- **Reactive**: Svelte stores auto-update components
- **Type Safe**: Strong TypeScript types for all events
- **Easy Debugging**: Inspect all execution state in one place
