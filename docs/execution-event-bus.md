# Execution Event Bus

Centralized event system for real-time execution state updates across the UI.

## Quick Start

```typescript
import { subscribeToExecutions, getExecutionWithUpdates } from '$lib/stores'

// Initialize once at app startup (in +layout.svelte)
await subscribeToExecutions()

// In your component
const executionStore = getExecutionWithUpdates(execution)
$executionStore // Auto-updates with latest state
```

## Supported Events

1. **session** - Session ID and Amp thread URL
2. **status** - Execution status (pending, running, completed, failed, cancelled)
3. **validation** - Validation status and thread URL
4. **commit** - Commit status, SHA, and timestamp
5. **progress** - Progress messages during execution

## Usage Patterns

### Reactive Execution State

```typescript
import { getExecutionWithUpdates } from '$lib/stores'

const executionStore = getExecutionWithUpdates(execution)

// Automatically reactive
$: status = $executionStore.status
$: sessionUrl = $executionStore.sessionUrl
```

### Manual Event Handling

```typescript
import { onExecutionUpdate } from '$lib/stores/executionBus'

const unsubscribe = onExecutionUpdate(executionId, (data) => {
  console.log('Status:', data.status)
  console.log('Progress:', data.progress)
})

// Cleanup when done
onDestroy(unsubscribe)
```

### Direct State Access

```typescript
import { getExecutionStatus } from '$lib/stores/executionBus'

const currentState = getExecutionStatus(executionId)
if (currentState?.status === 'running') {
  // Handle running state
}
```

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
  return derived(executionStore, $updates => ({
    ...execution,
    ...($updates[execution.id] || {})
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
```

All events include `executionId` for store keying.

## Benefits

- **Single Source of Truth**: One store for all execution state
- **Memory Safe**: Proper cleanup prevents listener leaks
- **Reactive**: Svelte stores auto-update components
- **Type Safe**: Strong TypeScript types for all events
- **Easy Debugging**: Inspect all execution state in one place

## Migration from Direct Listeners

### Before

```typescript
// Multiple separate listeners
await listen('execution:status', (event) => { ... })
await listen('execution:commit', (event) => { ... })
await listen('execution:validation', (event) => { ... })
await listen('execution:session', (event) => { ... })
```

### After

```typescript
// Single subscription
await subscribeToExecutions()

// Automatic updates via derived stores
const executionStore = getExecutionWithUpdates(execution)
```
