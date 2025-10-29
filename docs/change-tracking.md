# Change Tracking

Maestro tracks file changes from both committed history and live worktrees. **For common usage patterns, see AGENTS.md Diff Architecture section.**

## When to Read This

- Understanding the difference between worktree and committed diffs
- Learning how to access change information in the UI
- Understanding the change tracking lifecycle

## Core Concepts

### Worktree Diff vs. Committed Diff

Maestro provides two types of change tracking:

**Worktree Diff** - Changes in active execution worktrees:
- Tracks uncommitted changes in the working directory
- Shows added, modified, and deleted files
- Includes line-level statistics (additions/deletions)
- Available while execution is in progress or completed but not yet committed

**Committed Diff** - Changes from git history:
- Regenerated from commit metadata stored in the database
- Shows the exact changes between parent and current commit
- Available even after worktree cleanup
- Permanent record of what changed in each execution

### Change Lifecycle

```
Execution starts
  ↓
Worktree created with initial state
  ↓
AI makes changes (tracked via worktree diff)
  ↓
Changes committed to branch
  ↓
Diff source switches to committed history
  ↓
Worktree can be safely cleaned up
  ↓
Changes remain accessible via commit metadata
```

## Frontend API

### Accessing Changes

Changes are accessed through the IPC layer:

```typescript
// Get list of modified files
const response = await ipc.getExecutionModifiedFiles(executionId)
// Returns: { files: [], source: "worktree" | "committed", commitSha? }

// Get diff for specific file
const diffText = await ipc.getExecutionFileDiff(executionId, filePath)
```

### Change Statistics

Statistics are calculated on-demand from the diff:

- **File counts**: Added, modified, deleted, renamed files
- **Line changes**: Total additions and deletions
- **Per-file stats**: Individual file change metrics

Statistics are never stored in the database—they're always computed from the current git state.

## Key Design Decisions

### Ephemeral Worktrees

Worktrees can be safely deleted after commit because:

- Commit SHA is stored in execution metadata
- Parent SHA is stored before deletion
- Diffs are regenerated from the admin repository's git history

This approach:
- Saves disk space
- Ensures accuracy (no stale cached data)
- Allows historical access to changes

### On-Demand Calculation

Change statistics are calculated when requested rather than stored:

**Benefits:**
- Always accurate and reflects current git state
- No risk of stale data
- Simpler database schema
- Works seamlessly after worktree cleanup

**Trade-off:**
- Requires computation on each access
- Frontend caching recommended for performance

## UI Components

### Change Display

Changes are displayed in several places:

- **Execution rows**: Summary statistics (file count, line changes)
- **Diff viewer**: Detailed file-by-file comparison
- **Commit status**: Shows whether changes are committed

### Diff Viewer

The diff viewer provides:
- File list with change status badges
- Line-level statistics per file
- Unified diff view for individual files
- Syntax highlighting

## Common Patterns

### Viewing Changes

1. Click the commit badge or changes summary in an execution row
2. Diff viewer opens with file list
3. Click any file to see detailed diff

### After Committing

1. Changes are committed to a branch
2. Diff source automatically switches to "committed"
3. Stats remain accessible even if worktree is cleaned up

### Caching for Performance

The frontend caches diff data per execution to avoid redundant IPC calls. Cache is automatically invalidated when:
- Execution status changes
- Changes are committed
- Execution is restarted

## Troubleshooting

### Changes Not Showing

**Possible causes:**
- Execution hasn't made any changes yet
- Worktree doesn't exist (was cleaned up)
- Execution was never committed

**Solution:** For completed executions, commit the changes to make them permanently accessible.

### Statistics Show Zero

**Possible causes:**
- Execution completed with no changes
- Diff calculation encountered an error

**Solution:** Check execution logs or try restarting the execution.
