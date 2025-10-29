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

### Direct Store Access

Components access execution updates directly from the centralized `executionStore`:

```typescript
import { executionStore } from "$lib/stores/executionBus"

// In Svelte 5 runes mode, access updates directly
let executionsWithUpdates = $derived(
	executions.map((execution) => {
		const updates = $executionStore.get(execution.id)
		return updates ? { ...execution, ...updates } : execution
	})
)
```

## Backend Events

Events emitted from backend:

- `execution:session` - Session ID and thread URL
- `execution:status` - Execution status changes
- `execution:validation` - Validation status and thread URL
- `execution:commit` - Commit status, SHA, and timestamp
- `execution:ci` - CI status and results URL

All events include `executionId` for store keying.

## Benefits

- **Single Source of Truth**: One store for all execution state
- **Memory Safe**: Proper cleanup prevents listener leaks
- **Reactive**: Svelte stores auto-update components
- **Type Safe**: Strong TypeScript types for all events
- **Easy Debugging**: Inspect all execution state in one place
