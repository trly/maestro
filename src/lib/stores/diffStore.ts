import { writable, get } from "svelte/store"
import * as ipc from "$lib/ipc"

export interface FileDiff {
	status: string
	path: string
	additions?: number
	deletions?: number
}

export interface ModifiedFilesResponse {
	files: FileDiff[]
	source: string
	commitSha: string | null
}

interface DiffCache {
	[executionId: string]: ModifiedFilesResponse
}

interface FileDiffCache {
	[key: string]: string
}

const diffCache = writable<DiffCache>({})
const fileDiffCache = writable<FileDiffCache>({})

export async function fetchDiff(executionId: string): Promise<ModifiedFilesResponse> {
	const cache = get(diffCache)

	if (cache[executionId]) {
		return cache[executionId]
	}

	const result = await ipc.getExecutionModifiedFiles(executionId)

	diffCache.update((c) => {
		c[executionId] = result
		return c
	})

	return result
}

export async function fetchFileDiff(executionId: string, filePath: string): Promise<string> {
	const key = `${executionId}:${filePath}`
	const cache = get(fileDiffCache)

	if (cache[key]) {
		return cache[key]
	}

	const result = await ipc.getExecutionFileDiff(executionId, filePath)

	fileDiffCache.update((c) => {
		c[key] = result
		return c
	})

	return result
}

export function clearDiffCache(executionId?: string) {
	if (executionId) {
		diffCache.update((c) => {
			delete c[executionId]
			return c
		})

		fileDiffCache.update((c) => {
			const keysToDelete = Object.keys(c).filter((k) => k.startsWith(`${executionId}:`))
			for (const key of keysToDelete) {
				delete c[key]
			}
			return c
		})
	} else {
		diffCache.set({})
		fileDiffCache.set({})
	}
}
