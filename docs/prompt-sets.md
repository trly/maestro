# Prompt Sets

A **Prompt Set** is the top-level organizational unit in Maestro. It groups prompt revisions and defines which repositories to target for execution.

## Domain Model

```rust
struct PromptSet {
    id: String,                   // UUID
    name: String,                 // User-friendly name
    repository_ids: Vec<String>,  // Target repositories
    validation_prompt: Option<String>,
    auto_validate: bool,
    created_at: i64,
}
```

## Lifecycle

### 1. Creation

```typescript
import * as ipc from '$lib/ipc'

const promptSet = await ipc.createPromptSet(
  'Fix authentication bugs',
  ['repo-uuid-1', 'repo-uuid-2'],  // Target repositories
  'Check if authentication works correctly'  // Optional validation prompt
)
```

**Database:**
```sql
INSERT INTO prompt_sets (id, name, repository_ids, validation_prompt, auto_validate, created_at)
VALUES (?, ?, ?, ?, ?, ?)
```

### 2. Adding Repositories

```typescript
await ipc.updatePromptSetRepositories(
  promptSet.id,
  ['repo-uuid-1', 'repo-uuid-2', 'repo-uuid-3']
)
```

Repositories are stored as a JSON array in SQLite:
```sql
UPDATE prompt_sets 
SET repository_ids = ? 
WHERE id = ?
```

### 3. Configuring Validation

```typescript
await ipc.updatePromptSetValidation(
  promptSet.id,
  'Verify all tests pass and no regressions introduced'
)
```

**Validation Prompt Usage:**
- If set, executions can trigger validation after completion
- Validation creates a new Amp session with the prompt and diff context
- Results stored in `validation_status` and `validation_result`

### 4. Creating Revisions

```typescript
const revision = await ipc.createPromptRevision(
  promptSet.id,
  'Refactor authentication to use JWT tokens',
  null  // No parent (first revision)
)
```

See [Prompt Revisions](#prompt-revisions) below.

### 5. Deletion

```typescript
await ipc.deletePromptSet(promptSet.id)
```

**Cascade Behavior:**
- Deletes all prompt revisions
- Deletes all executions
- Does NOT delete repositories (they're independent)

## Prompt Revisions

Revisions represent versions of a prompt within a set. They form a DAG through parent relationships.

```rust
struct PromptRevision {
    id: String,
    promptset_id: String,
    prompt_text: String,
    parent_revision_id: Option<String>,  // Enables DAG structure
    created_at: i64,
}
```

### Revision DAG

Revisions can have parent revisions, creating a version history:

```
      ┌─────────────┐
      │  Revision A │
      │ "Add login" │
      └──────┬──────┘
             │
        ┌────┴─────┐
        ▼          ▼
┌──────────┐  ┌──────────┐
│Revision B│  │Revision C│
│"Use JWT" │  │"Use OAuth"│
└──────────┘  └──────────┘
```

### Creating Child Revisions

```typescript
// Create a child revision from Revision A
const revisionB = await ipc.createPromptRevision(
  promptSet.id,
  'Use JWT tokens instead of sessions',
  revisionA.id  // Parent
)
```

### Executing a Revision

```typescript
// Execute against all repositories in the prompt set
const executionIds = await ipc.executePromptSet(
  promptSet.id,
  revision.id
)

// Or execute against a subset
const executionIds = await ipc.executePromptSet(
  promptSet.id,
  revision.id,
  ['repo-uuid-1']  // Only this repository
)
```

**What Happens:**
1. Creates one `Execution` record per target repository
2. For each execution:
   - Creates git worktree from admin repo
   - Launches Amp session with prompt
   - Tracks changes in worktree
   - Emits real-time status events

See [executions.md](./executions.md) for complete execution lifecycle.

## Repository Management

### Listing Repositories

```typescript
const repos = await ipc.getAllRepositories()

// Display in UI
repos.forEach(repo => {
  console.log(`${repo.name || repo.provider_id}`)
})
```

### Adding a Repository

```typescript
const repo = await ipc.createRepository('github', 'sourcegraph/maestro')
```

**Backend Flow:**
1. Checks if repository already exists (`provider` + `provider_id`)
2. Creates database record
3. Clones admin repo to `{app_data_dir}/repos/sourcegraph/maestro/`
4. Stores default branch

### Repository Clone

```rust
// src-tauri/src/git/service.rs
pub fn clone_repo(url: &str, local_path: &Path) -> Result<()> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
    });
    
    Repository::clone(&url, local_path)?;
    Ok(())
}
```

Uses SSH authentication via ssh-agent. See [ssh-authentication.md](./ssh-authentication.md).

## Query API

### Get Single Prompt Set

```typescript
const promptSet = await ipc.getPromptSet(id)
if (!promptSet) {
  throw new Error('Prompt set not found')
}
```

### Find by ID Prefix

```typescript
// Find by short hash (e.g., "abc12345")
const promptSet = await ipc.findPromptSetByPrefix('abc12345')
```

Useful for CLI-style references where users type shortened UUIDs.

### Get All Prompt Sets

```typescript
const promptSets = await ipc.getAllPromptSets()

// With statistics
promptSets.forEach(ps => {
  console.log(`${ps.name}: ${ps.stats?.total_executions} executions`)
})
```

**Stats included:**
- `total_executions` - Total executions across all revisions
- `total_completions` - Executions with status "completed"
- `total_validations` - Executions with validation_status "passed" or "failed"
- `total_revisions` - Number of revisions in the set

### Get Revisions for a Prompt Set

```typescript
const revisions = await ipc.getPromptSetRevisions(promptSet.id)

// Ordered by creation date (newest first)
revisions.forEach(r => {
  console.log(`${r.id.substring(0, 8)}: ${r.prompt_text.substring(0, 50)}...`)
})
```

### Get Executions for a Prompt Set

```typescript
const executions = await ipc.getExecutionsByPromptSet(promptSet.id)

// Group by revision
const byRevision = executions.reduce((acc, ex) => {
  if (!acc[ex.revision_id]) acc[ex.revision_id] = []
  acc[ex.revision_id].push(ex)
  return acc
}, {})
```

## UI Components

### PromptSetList.svelte

Displays all prompt sets with statistics:

```svelte
<script>
  import { onMount } from 'svelte'
  import * as ipc from '$lib/ipc'
  
  let promptSets = $state<PromptSet[]>([])
  
  onMount(async () => {
    promptSets = await ipc.getAllPromptSets()
  })
</script>

{#each promptSets as ps (ps.id)}
  <div>
    <h3>{ps.name}</h3>
    <p>{ps.stats?.total_executions} executions</p>
    <p>{ps.repository_ids.length} repositories</p>
  </div>
{/each}
```

### PromptSetDetail.svelte

Shows revisions and executions for a specific prompt set:

```svelte
<script>
  import { onMount } from 'svelte'
  
  let { promptsetId } = $props<{ promptsetId: string }>()
  
  let promptSet = $state<PromptSet | null>(null)
  let revisions = $state<PromptRevision[]>([])
  let executions = $state<Execution[]>([])
  
  onMount(async () => {
    [promptSet, revisions, executions] = await Promise.all([
      ipc.getPromptSet(promptsetId),
      ipc.getPromptSetRevisions(promptsetId),
      ipc.getExecutionsByPromptSet(promptsetId)
    ])
  })
</script>
```

## Best Practices

### Naming Conventions

- **Descriptive names**: "Fix authentication bugs" vs "Prompt 1"
- **Action-oriented**: Start with verb ("Add", "Fix", "Refactor")
- **Scoped**: Include feature or area ("Auth: Add JWT support")

### Repository Selection

- **Monorepo**: Create separate prompt sets for different concerns
- **Multi-repo**: Group related repos (e.g., "frontend + backend" for full-stack changes)
- **Experimental**: Start with subset, expand after validation

### Validation Prompts

Good validation prompts:
- ✅ "Check if all tests pass"
- ✅ "Verify no TypeScript errors introduced"
- ✅ "Ensure authentication flow works end-to-end"

Poor validation prompts:
- ❌ "Is this good?" (too vague)
- ❌ "Fix it if broken" (should be separate execution)

### Revision Strategy

**Linear history:**
```
A → B → C → D
```
Use when iterating on a single approach.

**Branching:**
```
    ┌→ B (JWT)
A ──┤
    └→ C (OAuth)
```
Use when exploring multiple approaches.

## Implementation Reference

**Backend:**
- `src-tauri/src/db/store.rs` - Database operations
- `src-tauri/src/commands/promptsets.rs` - Tauri commands

**Frontend:**
- `src/lib/ipc.ts` - Type-safe IPC wrappers
- `src/routes/(app)/promptsets/` - UI routes
- `src/lib/components/PromptSet*.svelte` - UI components

## Related Documentation

- **[Architecture](./architecture.md)** - Overall system design
- **[Executions & Validations](./executions.md)** - Execution lifecycle
- **[IPC Guide](./ipc-guide.md)** - API reference
