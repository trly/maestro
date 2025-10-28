import * as ipc from "$lib/ipc"
import type { Execution } from "$lib/types"
import { settingsStore } from "$lib/stores/settingsStore"
import { get } from "svelte/store"

/**
 * Get the worktree path for an execution
 */
export async function getWorktreePath(execution: Execution): Promise<string> {
	const configPaths = await ipc.getConfigPaths()
	return `${configPaths.worktreeDir}/${execution.promptsetId}/${execution.id}`
}

/**
 * Open the execution worktree in the configured editor with proper terminal wrapping if needed
 */
export async function openInEditor(
	execution: Execution,
	editorCommandFallback?: string
): Promise<void> {
	const settings = get(settingsStore)

	// Use settings if available, otherwise fallback to legacy editorCommand
	const selectedEditor = settings.selectedEditor || editorCommandFallback || "code"
	const selectedTerminal = settings.selectedTerminal

	// Check if editor needs terminal
	const availableEditors = await ipc.getAvailableEditors()
	const editorInfo = availableEditors.find((e) => e.command === selectedEditor)
	const needsTerminal = editorInfo?.needsTerminal ?? false

	if (needsTerminal && selectedTerminal) {
		// Use terminal wrapper
		await ipc.openWorktreeWithTerminal(
			execution.promptsetId,
			execution.id,
			selectedEditor,
			selectedTerminal
		)
	} else if (needsTerminal && !selectedTerminal) {
		// Editor needs terminal but none selected - show error
		throw new Error(`${selectedEditor} requires a terminal. Please select a terminal in Settings.`)
	} else {
		// Direct launch
		await ipc.openWorktreeInEditor(execution.promptsetId, execution.id, selectedEditor)
	}
}

/**
 * Copy the worktree path to clipboard
 */
export async function copyWorktreePath(execution: Execution): Promise<void> {
	const worktreePath = await getWorktreePath(execution)
	await navigator.clipboard.writeText(worktreePath)
}
