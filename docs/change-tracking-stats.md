# Diff Stats Architecture

How file change statistics are calculated and displayed in Maestro.

## Overview

Diff stats (files added/removed/modified, lines added/removed) are **always calculated on-demand** from git state. They are never stored in the database - only the `parent_sha` and `commit_sha` are stored, allowing stats to be regenerated at any time.

This ensures:

- Stats are always accurate and reflect current git state
- No stale data in database
- Stats available even after worktree cleanup (for committed changes)

## Backend Calculation

### On-Demand Stats Retrieval

When viewing diffs, stats are calculated live:

```rust
// src-tauri/src/commands/executor.rs - get_execution_modified_files
if execution.commit_status == CommitStatus::Committed {
    // Use committed diff from admin repo
    return get_committed_diff(&admin_repo_path, parent_sha, commit_sha);
}

// Use worktree diff for uncommitted changes
return get_worktree_diff(&worktree_path);
```

## Frontend Display

### ExecutionRow Component

Stats are displayed in two columns:

1. **Commit Status** - Shows commit SHA (clickable to view diff)
2. **Changes Stats** - Shows file count and line changes (clickable to view diff)

```svelte
<!-- src/lib/components/executions/ExecutionRow.svelte -->
<ExecutionRow
	{execution}
	fileCount={(execution.filesAdded || 0) +
		(execution.filesRemoved || 0) +
		(execution.filesModified || 0)}
	additions={execution.linesAdded || 0}
	deletions={execution.linesRemoved || 0}
	onReviewChanges={() => openDiffViewer(execution.id)}
/>
```

### DiffViewer Component

When opened, fetches live diff data:

```svelte
<!-- src/lib/components/DiffViewer.svelte -->
const filesData = await fetchDiff(executionId); // filesData.files contains per-file stats // Aggregated
stats calculated from filesData.files
```

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ Stats Calculation Flow                                          │
└─────────────────────────────────────────────────────────────────┘

1. Frontend loads executions
   ↓
   loadLiveStats() fetches stats for completed executions
   ↓
   fetchExecutionStats(id) calls getExecutionModifiedFiles(id)
   ↓
   Backend: get_execution_modified_files
   ├─ Committed: get_committed_diff(parent_sha, commit_sha) from admin repo
   └─ Uncommitted: get_worktree_diff(worktree_path)
   ↓
   Returns ModifiedFilesResponse with file list (status, additions, deletions)
   ↓
   Frontend aggregates stats from file list
   ↓
   Stats cached in executionStats store
   ↓
   ExecutionRow displays stats
```

## Key Design Decisions

### Why Not Store Stats in Database?

Storing only `parent_sha` and `commit_sha` provides:

- **Always accurate**: Stats calculated from actual git state
- **No stale data**: Can't get out of sync with git history
- **Simpler**: No need to update stats on every change
- **Works after cleanup**: Committed stats available even after worktree deletion

### Why Frontend Caching?

- **Performance**: Avoids redundant IPC calls for same execution
- **User experience**: Instant display when revisiting executions
- **Lightweight**: Simple Map-based cache in memory

## Common Patterns

### Accessing Stats in Components

```typescript
// Get stats from execution object (from DB)
const totalFiles = execution.filesAdded + execution.filesRemoved + execution.filesModified
const netLines = execution.linesAdded - execution.linesRemoved

// Or calculate live from diff response
const diffResponse = await fetchDiff(executionId)
const totalFiles = diffResponse.files.length
const totalAdditions = diffResponse.files.reduce((sum, f) => sum + (f.additions || 0), 0)
```

### Invalidating Cached Diffs

```typescript
import { clearDiffCache } from "$lib/stores/diffStore"

// After commit, clear cache to force fresh fetch
await api.executions.commit(executionId)
clearDiffCache(executionId)
```

## Troubleshooting

### Stats Show Zero

**Cause**: Execution was created before stats calculation was implemented

**Solution**: Run a new execution or commit the execution (triggers recalculation)

### Stats Don't Match Diff Viewer

**Cause**: Cache may be stale after commit

**Solution**: Cache is automatically cleared after commit. If issue persists, check backend logs for diff calculation errors.

### Uncommitted Changes Not Showing

**Cause**: `diff_summary` was using tree-to-tree comparison

**Fixed**: Now uses `diff_tree_to_workdir_with_index` to capture uncommitted changes

## Implementation Reference

**Backend:**

- `src-tauri/src/git/diff.rs` - `get_committed_diff`, `get_worktree_diff`
- `src-tauri/src/commands/executor.rs` - `get_execution_modified_files`

**Frontend:**

- `src/lib/stores/executionStats.ts` - On-demand stats fetching and caching
- `src/lib/stores/diffStore.ts` - Caching layer for diff file lists
- `src/lib/components/executions/ExecutionRow.svelte` - Stats display
- `src/lib/components/DiffViewer.svelte` - Detailed diff viewer
