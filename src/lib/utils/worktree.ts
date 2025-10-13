import * as ipc from '$lib/ipc'
import type { Execution } from '$lib/types'

/**
 * Get the worktree path for an execution
 */
export async function getWorktreePath(execution: Execution): Promise<string> {
	const configPaths = await ipc.getConfigPaths()
	return `${configPaths.worktreeDir}/${execution.promptsetId}/${execution.id}`
}

/**
 * Open the execution worktree in the configured editor
 */
export async function openInEditor(execution: Execution, editorCommand: string): Promise<void> {
	await ipc.openWorktreeInEditor(execution.promptsetId, execution.id, editorCommand)
}

/**
 * Copy the worktree path to clipboard
 */
export async function copyWorktreePath(execution: Execution): Promise<void> {
	const worktreePath = await getWorktreePath(execution)
	await navigator.clipboard.writeText(worktreePath)
}
