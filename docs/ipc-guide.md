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
await ipc.syncRepositoryMetadata(id)

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
await ipc.createPromptSet(name, repositoryIds, validationPrompt?, autoValidate?)
await ipc.updatePromptSetValidation(id, validationPrompt)
await ipc.updatePromptSetAutoValidate(id, autoValidate)
await ipc.updatePromptSetRepositories(id, repositoryIds)

// Read
await ipc.getPromptSet(id)
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
await ipc.getExecutionsByRevision(revisionId)

// Execute
await ipc.prepareExecutions(promptsetId, revisionId, repositoryIds?)
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
await ipc.getExecutionFileDiff(executionId, file)

// Control
await ipc.startExecution(executionId)
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
import type { TokenKey } from "$lib/ipc"

const key: TokenKey = "amp_token" // or 'github_token', 'gitlab_token', etc.

await ipc.setToken(key, value)
await ipc.deleteToken(key)
await ipc.getAllTokens() // => { ampToken, githubToken, ... }
await ipc.getAllTokensMasked() // => { ampToken: "abc...xyz", ... }
```

### Configuration

```typescript
// Paths
const paths = await ipc.getConfigPaths()
console.log(paths.adminRepoDir) // ~/maestro/repos
console.log(paths.worktreeDir) // ~/maestro/executions
console.log(paths.dbPath) // maestro.db

// Editor/Terminal
await ipc.openWorktreeInEditor(promptsetId, executionId, editorCommand)
await ipc.openWorktreeWithTerminal(promptsetId, executionId, editorCommand, terminalCommand)

// App Info
const appInfo = await ipc.getAppInfo() // => { version, name, identifier, copyright }
const editors = await ipc.getAvailableEditors() // => AppInfo[]
const terminals = await ipc.getAvailableTerminals() // => TerminalInfo[]
const isInstalled = await ipc.checkAppInstalled(command)

// Health Checks
const ghHealth = await ipc.healthCheckGithub() // => { success, username?, error? }
const glHealth = await ipc.healthCheckGitlab()
const sgHealth = await ipc.healthCheckSourcegraph()

// Settings
await ipc.getSetting(key)
await ipc.setSetting(key, value)
await ipc.getCiStuckThresholdMinutes()
await ipc.getMaxConcurrentExecutions()

// CI
await ipc.startCiCheck(executionId)
await ipc.refreshCiStatus(executionId)
await ipc.pushCommit(executionId, force?)

// Sourcegraph
const result = await ipc.searchSourcegraphRepositories(query, limit?)

// Analysis
const analysisId = await ipc.createAnalysis(revisionId, analysisType, executionIds)
await ipc.runAnalysis(analysisId)
const analysis = await ipc.getAnalysis(analysisId)
const analyses = await ipc.getAnalysesByRevision(revisionId, analysisType?)
await ipc.deleteAnalysis(analysisId)
```

## Migration from `invoke()`

### Before (direct invoke)

```typescript
import { invoke } from "@tauri-apps/api/core"

const execution = await invoke<Execution | null>("get_execution", { id })
await invoke("validate_execution", { executionId: id })
```

### After (IPC wrappers)

```typescript
import * as ipc from "$lib/ipc"

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
