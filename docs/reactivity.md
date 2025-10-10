# Reactivity Guide

Maestro uses **Svelte 5 runes mode** for reactive state management. This guide covers the patterns and best practices for building reactive components.

## Required Reading

Before working with Svelte or bits-ui components, **ALWAYS** consult:

- **Svelte 5 Documentation**: https://svelte.dev/llms-full.txt
- **bits-ui Documentation**: https://bits-ui.com/docs/llms.txt

These documents contain the authoritative patterns for Svelte 5 runes mode and bits-ui component usage.

## Core Reactivity Primitives

### `$state` - Reactive State

Use `$state` for all component-local reactive data:

```svelte
<script>
  let count = $state(0);
  let user = $state({ name: 'Alice', age: 30 });
  let items = $state([{ id: 1, text: 'Task 1' }]);
</script>

<button onclick={() => count++}>Count: {count}</button>
```

**Key Features:**
- Deep reactivity for objects and arrays
- Automatic proxy creation for nested properties
- Mutations trigger UI updates

### `$derived` - Computed Values

Use `$derived` for values that depend on other reactive state:

```svelte
<script>
  let firstName = $state('John');
  let lastName = $state('Doe');
  
  // Automatically recomputes when firstName or lastName changes
  let fullName = $derived(`${firstName} ${lastName}`);
</script>
```

### `$props()` - Component Inputs

Use `$props()` to define component props:

```svelte
<script>
  let { execution, onDelete } = $props();
</script>
```

## Maestro-Specific Patterns

### Event Bus Reactivity

Maestro uses a centralized event bus (`executionStore`) for real-time execution updates. The pattern for integrating event bus updates with component state is:

```typescript
import { executionStore } from '$lib/stores/executionBus';

let executions = $state<Execution[]>([]);

// Merge static data with live updates
let executionsWithUpdates = $derived(
  executions.map(execution => {
    const updates = $executionStore.get(execution.id);
    if (!updates) return execution;
    return {
      ...execution,
      ...(updates.sessionId && { sessionId: updates.sessionId }),
      ...(updates.status && { status: updates.status }),
      ...(updates.validationStatus && { validationStatus: updates.validationStatus }),
      // ... other fields
    };
  })
);
```

**Why This Works:**
- `$executionStore` is a store subscription at the top level (allowed)
- `$derived` recomputes when `$executionStore` changes
- No per-item store subscriptions (not allowed in runes mode)
- Clean separation between static data and live updates

### Computing Aggregate Stats

Use `$derived` to compute aggregate statistics from reactive arrays:

```typescript
let revisionStats = $derived(
  revisions.reduce((acc, revision) => {
    const revisionExecutions = executionsWithUpdates.filter(
      e => e.revisionId === revision.id
    );
    const total = revisionExecutions.length;
    const running = revisionExecutions.filter(e => e.status === 'running').length;
    const completed = revisionExecutions.filter(e => e.status === 'completed').length;
    
    acc[revision.id] = { total, running, completed };
    return acc;
  }, {} as Record<string, Stats>)
);
```

**Benefits:**
- Automatically updates when `executionsWithUpdates` changes
- Efficient - only recomputes when dependencies change
- Type-safe with TypeScript

### Using Reactive Data in Templates

```svelte
{#each executionsWithUpdates as execution (execution.id)}
  <div>
    <h3>{execution.name}</h3>
    <StatusBadge status={execution.status} />
    
    {#if execution.status === 'running'}
      <button onclick={() => stop(execution)}>Stop</button>
    {/if}
    
    {#if execution.status === 'completed' && execution.validationStatus !== 'running'}
      <button onclick={() => validate(execution)}>Validate</button>
    {/if}
  </div>
{/each}
```

**Key Points:**
- Use keyed `{#each}` blocks with `(execution.id)` for efficient updates
- Conditionals (`{#if}`) automatically re-evaluate when reactive state changes
- No manual subscriptions needed - Svelte handles it

## Anti-Patterns to Avoid

### ❌ Per-Item Stores in Loops

**DON'T DO THIS:**

```svelte
{#each items as item}
  {@const itemStore = getStoreForItem(item)}
  <div>{$itemStore.value}</div>
{/each}
```

**Problem:** Cannot subscribe to stores inside `{#each}` blocks in runes mode.

**DO THIS INSTEAD:**

```svelte
<script>
  let itemsWithUpdates = $derived(
    items.map(item => {
      const updates = $globalStore.get(item.id);
      return updates ? { ...item, ...updates } : item;
    })
  );
</script>

{#each itemsWithUpdates as item (item.id)}
  <div>{item.value}</div>
{/each}
```

### ❌ Store Subscriptions in Scoped Blocks

**DON'T DO THIS:**

```svelte
{#if condition}
  {#await promise}
    {@const store = createStore()}
    <div>{$store.value}</div>
  {/await}
{/if}
```

**Problem:** Store subscriptions must be at component top level.

**DO THIS INSTEAD:**

```svelte
<script>
  let storeValue = $state(null);
  
  $effect(() => {
    // Handle store subscription in effect at top level
  });
</script>
```

### ❌ Traditional Stores for Component State

**DON'T DO THIS:**

```svelte
<script>
  import { writable } from 'svelte/store';
  const count = writable(0);
</script>

<button onclick={() => $count++}>Count: {$count}</button>
```

**DO THIS INSTEAD:**

```svelte
<script>
  let count = $state(0);
</script>

<button onclick={() => count++}>Count: {count}</button>
```

## Working with bits-ui Components

bits-ui components expose `$bindable` props for state management:

```svelte
<script>
  import { Accordion } from "bits-ui";
  
  let value = $state(["item-1"]);
</script>

<Accordion.Root bind:value={value}>
  <Accordion.Item value="item-1">
    <!-- content -->
  </Accordion.Item>
</Accordion.Root>
```

**Pattern:**
- Declare `$state` for the value you want to control
- Use `bind:value` to two-way bind with bits-ui component
- Component updates reflect in your state automatically

## Deep Reactivity Example

`$state` provides deep reactivity for objects and arrays:

```svelte
<script>
  let todos = $state([
    { id: 1, text: 'Learn Svelte', done: false },
    { id: 2, text: 'Build app', done: false }
  ]);
  
  function toggle(id) {
    // Direct mutation works - $state creates deep proxy
    const todo = todos.find(t => t.id === id);
    if (todo) todo.done = !todo.done;
  }
  
  function add() {
    // Array methods work reactively
    todos.push({ id: Date.now(), text: 'New task', done: false });
  }
</script>
```

## Migration from Svelte 4 Stores

If you encounter legacy Svelte 4 store patterns, migrate to runes:

| Svelte 4 | Svelte 5 Runes |
|----------|----------------|
| `const count = writable(0)` | `let count = $state(0)` |
| `const doubled = derived(count, $c => $c * 2)` | `let doubled = $derived(count * 2)` |
| `$count++` (auto-subscribe) | `count++` (native reactivity) |
| `export let prop` | `let { prop } = $props()` |

## Testing Reactive Components

When testing components with reactive state:

```typescript
import { render, fireEvent } from '@testing-library/svelte';
import MyComponent from './MyComponent.svelte';

test('reactive state updates', async () => {
  const { getByRole } = render(MyComponent);
  const button = getByRole('button');
  
  await fireEvent.click(button);
  
  // State updates are synchronous in runes mode
  expect(button.textContent).toBe('Count: 1');
});
```

## Performance Considerations

- **`$derived` is lazy**: Only recomputes when accessed and dependencies changed
- **Fine-grained updates**: Svelte 5 only updates specific DOM nodes that changed
- **Memoization**: `$derived` automatically memoizes computed values
- **Batch updates**: Multiple state changes in same tick batch into single update

## Debugging Reactive State

Use Svelte DevTools to inspect reactive state:

```svelte
<script>
  let count = $state(0);
  
  // Log when derived value recomputes
  let doubled = $derived.by(() => {
    console.log('Recomputing doubled');
    return count * 2;
  });
</script>
```

## Summary

- ✅ Use `$state` for reactive component state
- ✅ Use `$derived` for computed values  
- ✅ Use `$props()` for component inputs
- ✅ Merge event bus updates at top level with `$derived`
- ✅ Use keyed `{#each}` blocks for arrays
- ❌ Never create stores inside loops
- ❌ Never subscribe to stores in scoped blocks
- ❌ Avoid traditional stores for new component state

**Always consult https://svelte.dev/llms-full.txt and https://bits-ui.com/docs/llms.txt before implementing reactive patterns.**
