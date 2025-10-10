import { writable, type Writable } from 'svelte/store'
import type { ExecutionStatus, ValidationStatus, CommitStatus } from '../types'
import { clearExecutionStats } from './executionStats'

export interface ExecutionSessionEvent {
	executionId: string
	sessionId: string
	threadUrl: string
}

export interface ExecutionStatusEvent {
	executionId: string
	status: ExecutionStatus
}

export interface ExecutionValidationEvent {
	executionId: string
	validationStatus: ValidationStatus
	validationThreadUrl?: string
}

export interface ExecutionCommitEvent {
	executionId: string
	commitStatus: CommitStatus
	commitSha?: string
	committedAt?: number
}

export interface ExecutionProgressEvent {
	executionId: string
	message: string
}

export interface ExecutionData {
	sessionId?: string
	threadUrl?: string
	status?: ExecutionStatus
	validationStatus?: ValidationStatus
	validationThreadUrl?: string
	commitStatus?: CommitStatus
	commitSha?: string
	committedAt?: number
	progressMessage?: string
}

const executionStore: Writable<Map<string, ExecutionData>> = writable(new Map())
let unlisteners: Array<() => void> = []

export async function subscribeToExecutions() {
	const isTauri = typeof window !== 'undefined' && '__TAURI__' in window
	if (!isTauri) return

	const { listen } = await import('@tauri-apps/api/event')

	const unlisten1 = await listen<ExecutionSessionEvent>('execution:session', (event) => {
		const { executionId, sessionId, threadUrl } = event.payload
		executionStore.update((map) => {
			const existing = map.get(executionId) || {}
			map.set(executionId, { ...existing, sessionId, threadUrl })
			return new Map(map)
		})
	})

	const unlisten2 = await listen<ExecutionStatusEvent>('execution:status', (event) => {
		const { executionId, status } = event.payload
		executionStore.update((map) => {
			const existing = map.get(executionId) || {}
			map.set(executionId, { ...existing, status })
			return new Map(map)
		})
		// Clear stats cache when execution completes so they can be refetched
		if (status === 'completed' || status === 'failed' || status === 'cancelled') {
			clearExecutionStats(executionId)
		}
	})

	const unlisten3 = await listen<ExecutionValidationEvent>('execution:validation', (event) => {
		const { executionId, validationStatus, validationThreadUrl } = event.payload
		executionStore.update((map) => {
			const existing = map.get(executionId) || {}
			map.set(executionId, { ...existing, validationStatus, validationThreadUrl })
			return new Map(map)
		})
	})

	const unlisten4 = await listen<ExecutionCommitEvent>('execution:commit', (event) => {
		const { executionId, commitStatus, commitSha, committedAt } = event.payload
		executionStore.update((map) => {
			const existing = map.get(executionId) || {}
			map.set(executionId, { ...existing, commitStatus, commitSha, committedAt })
			return new Map(map)
		})
	})

	const unlisten5 = await listen<ExecutionProgressEvent>('execution:progress', (event) => {
		const { executionId, message } = event.payload
		executionStore.update((map) => {
			const existing = map.get(executionId) || {}
			map.set(executionId, { ...existing, progressMessage: message })
			return new Map(map)
		})
	})

	unlisteners = [unlisten1, unlisten2, unlisten3, unlisten4, unlisten5]
}

export function unsubscribeFromExecutions() {
	unlisteners.forEach((unlisten) => unlisten())
	unlisteners = []
}

export function getExecutionStatus(id: string): ExecutionData | undefined {
	let data: ExecutionData | undefined
	executionStore.subscribe((map) => {
		data = map.get(id)
	})()
	return data
}

export function onExecutionUpdate(id: string, callback: (data: ExecutionData) => void) {
	return executionStore.subscribe((map) => {
		const data = map.get(id)
		if (data) callback(data)
	})
}

export function clearExecutionData(id: string) {
	executionStore.update((map) => {
		map.delete(id)
		return new Map(map)
	})
}

export { executionStore }
