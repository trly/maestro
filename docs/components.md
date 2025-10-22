# Component Architecture

Frontend component organization and patterns.

## Directory Structure

```
src/lib/components/
├── executions/           # Execution table components
│   ├── ExecutionTable.svelte      # Top-level container
│   ├── ExecutionFilters.svelte    # Filter controls
│   ├── ExecutionList.svelte       # Virtualized list + header
│   ├── ExecutionRow.svelte        # Individual execution row
│   └── types.ts                   # Shared types
├── ui/                   # Reusable UI primitives
│   ├── BulkActionBar.svelte
│   ├── FilterInput.svelte
│   ├── FilterSelect.svelte
│   ├── IconButton.svelte
│   └── RevisionHeader.svelte
└── DiffViewer.svelte    # Diff display component
```

## Execution Components

### ExecutionTable.svelte

**Purpose:** Top-level container that manages filtering, sorting, selection, and bulk operations.

**Responsibilities:**
- Filter state management and normalization
- Sort state management (key + direction)
- Selection state via `useSelection` composable
- Bulk action coordination
- Data transformation (filtering, sorting)

**Key Props:**
```typescript
{
  executions: Execution[]
  repositories: Map<string, Repository>
  hasValidationPrompt?: boolean
  executionsVersion?: number
  revisionId?: string
  // Loading states for optimistic UI
  pushingExecutions: Set<string>
  refreshingCi: Set<string>
  loadingStats: Set<string>
  bulkStarting: boolean
  // Action handlers
  onDeleteExecution: (execution, repoName) => void
  onStartExecution: (execution) => void
  onBulkDelete: (executions) => void
  // ... 15+ action handlers
}
```

**State Management:**
- Filters reset when `revisionId` changes
- Selection cleared after bulk operations
- Maintains `filteredSortedIds` for list rendering

### ExecutionFilters.svelte

**Purpose:** Filter controls for repository search and status filters.

**Responsibilities:**
- Repository text search (debounced)
- Status dropdowns (execution, validation, CI)
- Changes filter (has-changes/no-changes)
- Filter normalization (converts 'all' to undefined)

**Key Props:**
```typescript
{
  filters: ColumnFilters
  onFilterChange: (filters: ColumnFilters) => void
}
```

**UI Components:**
- `FilterInput` - Text search with clear button
- `FilterSelect` - bits-ui Select wrapper

### ExecutionList.svelte

**Purpose:** Virtualized list container with header and rows.

**Responsibilities:**
- Render table header with sort controls
- Virtual scrolling for performance (uses intersection observer)
- Pass through action handlers to rows
- Coordinate select-all checkbox

**Key Props:**
```typescript
{
  ids: string[]                          // Filtered + sorted IDs
  executionsById: Map<string, Execution> // Lookup map
  repositories: Map<string, Repository>
  selection: SelectionState
  sort: SortSpec
  // Loading states per execution
  pushingExecutions: Set<string>
  refreshingCi: Set<string>
  loadingStats: Set<string>
  // Action handlers
  onToggleSelectAll: () => void
  onChangeSort: (key: SortKey) => void
  onStart: (id: string) => void
  // ... 10+ action handlers
}
```

**Virtual Scrolling:**
- Uses `intersect` action (intersection observer)
- Only renders visible rows + buffer
- Maintains scroll position during updates

### ExecutionRow.svelte

**Purpose:** Individual execution row with status indicators and actions.

**Columns:**
1. **Checkbox** - Multi-select
2. **Repository** - Provider ID (e.g., "owner/repo")
3. **Status** - Execution status with icon
4. **Validation** - Validation status (if configured)
5. **CI** - CI status badge (if pushed)
6. **Commit** - Commit SHA with copy button
7. **Changes** - File count + line diff stats
8. **Actions** - Context menu with available actions

**Action Buttons:**
- Start/Stop/Restart execution
- Validate/Stop validation
- Review changes (diff viewer)
- Commit changes (via Amp)
- Push to remote
- Refresh CI status
- Delete execution

**Loading States:**
- Individual loading spinners for async operations
- Optimistic UI updates from event bus
- Disabled state during bulk operations

## Reusable UI Components

### BulkActionBar.svelte

Fixed toolbar shown when executions are selected.

**Actions:**
- Bulk Start - Start pending executions
- Bulk Restart - Resume failed/cancelled executions
- Bulk Validate - Start validations
- Bulk Revalidate - Restart validations
- Bulk Delete - Delete executions

### IconButton.svelte

Consistent button component for actions.

**Variants:**
- Default (primary color)
- Success (green)
- Warning (orange)
- Destructive (red)

**States:**
- Loading (spinner)
- Disabled
- Tooltip support

### FilterInput.svelte

Text input with clear button.

**Features:**
- Optional debounce
- Clear button (X)
- Placeholder text

### FilterSelect.svelte

Dropdown filter using bits-ui Select.

**Features:**
- "All" option clears filter
- Consistent styling
- Keyboard navigation

## Data Flow Patterns

### Parent → Child (Props)

```svelte
<!-- +page.svelte -->
<ExecutionTable
  executions={executionsWithUpdates}
  repositories={repositories}
  onStartExecution={handleStart}
/>
```

### Event Bus Integration

```typescript
// Merge live updates from event bus
let executionsWithUpdates = $derived.by(() => {
  const updates = $executionStore
  return executions.map(execution => ({
    ...execution,
    ...(updates.get(execution.id))
  }))
})
```

### Optimistic UI

```typescript
// Track loading state per operation
let pushingExecutions = $state<Set<string>>(new Set())

async function handlePush(execution: Execution) {
  pushingExecutions.add(execution.id)
  try {
    await api.executions.push(execution.id)
  } finally {
    pushingExecutions.delete(execution.id)
  }
}
```

## Performance Optimizations

### Virtual Scrolling

Only render visible rows using intersection observer:

```svelte
{#each visibleIds as id}
  <div use:intersect on:intersect={() => markVisible(id)}>
    <ExecutionRow ... />
  </div>
{/each}
```

### Derived State

Use `$derived.by()` for expensive computations:

```typescript
let filteredExecutions = $derived.by(() => {
  return executions.filter(e => matchesFilters(e))
})

let sortedExecutions = $derived.by(() => {
  return [...filteredExecutions].sort(comparator)
})
```

### Map-Based Lookups

Pass lookup maps instead of arrays for O(1) access:

```typescript
let executionsById = $derived.by(() => {
  const map = new Map()
  for (const e of executions) map.set(e.id, e)
  return map
})

// In child component
const execution = props.executionsById.get(id)
```

## Common Patterns

### Props Pattern (Avoid Destructuring)

```svelte
<!-- ✅ DO: Keep props as object -->
<script lang="ts">
  const props = $props<{ executions: Execution[] }>()
  
  // Access via props.X everywhere
  let filtered = $derived(props.executions.filter(...))
</script>

<!-- ❌ DON'T: Destructure (breaks reactivity) -->
<script lang="ts">
  let { executions } = $props()
  // Won't update when parent changes executions!
</script>
```

### Selection State

```typescript
// In parent
const selection = useSelection()

// Pass to children
<ExecutionList {selection} />

// In child
function handleToggle(id: string) {
  props.selection.toggle(id)
}
```

### Loading States

```typescript
// Track by ID
let loadingStats = $state<Set<string>>(new Set())

// In handler
async function handleLoadStats(id: string) {
  loadingStats.add(id)
  try {
    await fetchStats(id)
  } finally {
    loadingStats.delete(id)
  }
}

// In component
<IconButton
  loading={loadingStats.has(execution.id)}
  onclick={() => handleLoadStats(execution.id)}
/>
```

## Testing Considerations

### Component Isolation

Components should be testable in isolation:
- Accept all data via props
- Emit events for user actions
- No direct store access (except composables)

### Mock Data

Create factory functions for test data:

```typescript
function mockExecution(overrides?: Partial<Execution>): Execution {
  return {
    id: crypto.randomUUID(),
    status: 'completed',
    ...overrides
  }
}
```

## Related Documentation

- [Reactivity Patterns](./reactivity.md) - Svelte 5 runes best practices
- [Execution Event Bus](./execution-event-bus.md) - Real-time updates
- [Change Tracking](./change-tracking-stats.md) - Stats display
- [CI Integration](./ci-tracking.md) - CI status display
