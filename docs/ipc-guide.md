# IPC Guide

Complete API reference for Maestro's type-safe IPC layer. **For common usage patterns, see AGENTS.md IPC Layer section.**

## When to Read This

- You need the complete list of available IPC commands
- You're adding a new command and want to understand the full pattern
- You need detailed parameter documentation for a specific command

**Source of truth:** `src/lib/ipc.ts` - this document may lag behind the code.

## API Reference

### Repositories

```typescript
// Create/Update
await ipc.createRepository(provider, providerId)
await ipc.updateRepositoryName(id, name)

// Read
await ipc.getRepository(id) // => Repository | null
await ipc.findRepository(provider, providerId) // => Repository | null
await ipc.getAllRepositories() // => Repository[]

// Delete
await ipc.deleteRepository(id) // => boolean
```

### Prompt Sets

```typescript
// Create/Update
await ipc.createPromptSet(name, repositoryIds, validationPrompt?)
await ipc.updatePromptSetValidation(id, validationPrompt)
await ipc.updatePromptSetRepositories(id, repositoryIds)

// Read
await ipc.getPromptSet(id)
await ipc.findPromptSetByPrefix(idPrefix)
await ipc.getAllPromptSets()
await ipc.getPromptSetRevisions(promptsetId)
await ipc.getExecutionsByPromptSet(promptsetId)

// Delete
await ipc.deletePromptSet(id)
```

### Prompt Revisions

```typescript
// Create
await ipc.createPromptRevision(promptsetId, promptText, parentRevisionId?)

// Read
await ipc.getPromptRevision(id)
await ipc.findPromptRevisionByPrefix(idPrefix)
await ipc.getExecutionsByRevision(revisionId)

// Execute
await ipc.executePromptSet(promptsetId, revisionId, repositoryIds?)

// Control
await ipc.stopAllExecutions(revisionId)
await ipc.stopAllValidations(revisionId)

// Delete
await ipc.deletePromptRevision(id)
```

### Executions

```typescript
// Create
await ipc.createExecution(promptsetId, revisionId, repositoryId)

// Read
await ipc.getExecution(id)
await ipc.findExecutionByPrefix(idPrefix)
await ipc.getExecutionModifiedFiles(executionId)
await ipc.getExecutionFileDiff(executionId, filePath)

// Control
await ipc.validateExecution(executionId)
await ipc.stopExecution(executionId)
await ipc.stopValidation(executionId)
await ipc.resumeExecution(executionId)

// Commit & Cleanup
await ipc.commitChanges(executionId, files?)
await ipc.cleanupExecution(executionId)
await ipc.deleteExecution(id)
```

### Tokens

```typescript
import type { TokenKey } from '$lib/ipc'

const key: TokenKey = 'amp_token' // or 'github_token'

await ipc.setToken(key, value)
await ipc.getToken(key) // => string | null
await ipc.getTokenMasked(key) // => string | null (e.g., "abc...xyz")
await ipc.deleteToken(key)
await ipc.hasToken(key) // => boolean
```

### Configuration

```typescript
const paths = await ipc.getConfigPaths()
console.log(paths.adminRepoDir) // ~/maestro/repos
console.log(paths.worktreeDir)  // ~/maestro/executions
console.log(paths.dbPath)       // maestro.db
```

## Migration from `invoke()`

### Before (direct invoke)

```typescript
import { invoke } from '@tauri-apps/api/core'

const execution = await invoke<Execution | null>('get_execution', { id })
await invoke('validate_execution', { executionId: id })
```

### After (IPC wrappers)

```typescript
import * as ipc from '$lib/ipc'

const execution = await ipc.getExecution(id)
await ipc.validateExecution(id)
```

## Benefits

- **Type Safety**: Full TypeScript types for all commands and return values
- **Centralized Errors**: Uniform `TauriIPCError` with command context
- **Less Boilerplate**: Clean API vs manual `invoke()` calls
- **Single Source of Truth**: All command signatures in one module

## Advanced

### Using the High-Level API

For common workflows, use `tauriApi` from `$lib/tauri-api` - it provides additional validation and convenience built on the IPC layer.

### Custom Error Handling

```typescript
try {
  await ipc.commitChanges(executionId)
} catch (error) {
  if (error instanceof TauriIPCError) {
    // Show user-friendly message
    showToast(`Failed to commit: ${error.message}`)
  }
}
```
