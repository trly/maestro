# Change Tracking

Unified system for tracking file changes (diffs) and statistics from both committed history and live worktrees.

## Overview

Maestro's diff system supports:
- **Committed diffs**: Regenerated from git history after worktree deletion
- **Worktree diffs**: Live changes from active worktrees
- **Frontend caching**: Reduces redundant backend calls

## Backend Module

### `src-tauri/src/git/diff.rs`

**Core Functions:**

```rust
// Committed changes (from git history)
get_committed_diff(admin_repo_path, parent_sha, commit_sha) 
  -> Result<ModifiedFilesResponse>

get_committed_file_diff(admin_repo_path, parent_sha, commit_sha, file_path) 
  -> Result<String>

// Worktree changes (uncommitted)
get_worktree_diff(worktree_path) 
  -> Result<ModifiedFilesResponse>

get_worktree_file_diff(worktree_path, file_path) 
  -> Result<String>
```

**Types:**

```rust
struct ModifiedFile {
    status: FileStatus,
    path: String,
    additions: u32,
    deletions: u32
}

struct ModifiedFilesResponse {
    files: Vec<ModifiedFile>,
    source: DiffSource,  // "worktree" | "committed"
    commit_sha: Option<String>
}
```

### Command Flow

Both IPC commands follow the same logic:

```rust
// 1. Check if execution has commit
if let Some(commit_sha) = execution.commit_sha {
    return get_committed_diff(admin_repo_path, parent_sha, commit_sha)
}

// 2. Otherwise use worktree
if worktree_exists {
    return get_worktree_diff(worktree_path)
}
```

## Frontend Store

### `src/lib/stores/diffStore.ts`

**API:**

```typescript
// Fetch file list (cached)
const response = await fetchDiff(executionId)

// Fetch specific file diff (cached)
const diffText = await fetchFileDiff(executionId, filePath)

// Clear cache
clearDiffCache(executionId) // specific execution
clearDiffCache() // all executions

// Semantic alias
invalidateDiff(executionId)
```

**Caching Strategy:**

- Execution-level cache for file lists
- Execution:file-level cache for individual diffs
- Automatic invalidation after commit

**Types:**

```typescript
interface FileDiff {
    status: 'added' | 'modified' | 'deleted' | 'renamed'
    path: string
    additions: number
    deletions: number
}

interface ModifiedFilesResponse {
    files: FileDiff[]
    source: 'worktree' | 'committed'
    commitSha?: string
}
```

## Usage Examples

### Display File List

```typescript
import { fetchDiff } from '$lib/stores/diffStore'

const response = await fetchDiff(executionId)
console.log(`${response.files.length} files changed`)
console.log(`Source: ${response.source}`)
```

### Display File Diff

```typescript
import { fetchFileDiff } from '$lib/stores/diffStore'

const diffText = await fetchFileDiff(executionId, 'src/main.ts')
// Render unified diff format
```

### Invalidate After Commit

```typescript
import { invalidateDiff } from '$lib/stores/diffStore'

await ipc.commitChanges(executionId)
invalidateDiff(executionId) // Clear cache
await fetchDiff(executionId) // Fetch fresh committed diff
```

## Key Design Decisions

### Ephemeral Worktrees

Worktrees can be safely deleted after commit because:
- Commit SHA is stored in database
- Parent SHA is stored before deletion
- Diffs regenerated via `git diff <parent> <commit>` from admin repo

### Separation of Concerns

- **Backend**: Pure git operations, no caching
- **Frontend**: Caching layer for UI performance
- **Commands**: Route to appropriate diff function based on execution state

## Benefits

- **Code Reduction**: ~160 lines eliminated from commands (71% reduction)
- **Centralization**: All diff logic in one backend module
- **Caching**: Frontend reduces redundant IPC calls
- **Consistency**: Unified error handling and types
- **Testability**: Diff logic can be unit tested independently

## Components Using Diffs

- **`DiffViewer.svelte`**: Main diff display component (uses diffStore)
- **`PromptDiff.svelte`**: Unrelated (text-only prompt comparison)
