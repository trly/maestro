import * as ipc from "$lib/ipc"
import { logger } from "$lib/logger"

export interface ExecutionStats {
	filesAdded: number
	filesRemoved: number
	filesModified: number
	linesAdded: number
	linesRemoved: number
}

const statsCache = new Map<string, ExecutionStats>()

/**
 * Fetch execution stats on-demand from the worktree or committed diff
 */
export async function fetchExecutionStats(executionId: string): Promise<ExecutionStats> {
	if (statsCache.has(executionId)) {
		return statsCache.get(executionId)!
	}

	try {
		const diffResponse = await ipc.getExecutionModifiedFiles(executionId)

		const stats: ExecutionStats = {
			filesAdded: diffResponse.files.filter((f) => f.status === "added").length,
			filesRemoved: diffResponse.files.filter((f) => f.status === "deleted").length,
			filesModified: diffResponse.files.filter(
				(f) => f.status === "modified" || f.status === "renamed"
			).length,
			linesAdded: diffResponse.files.reduce((sum, f) => sum + (f.additions || 0), 0),
			linesRemoved: diffResponse.files.reduce((sum, f) => sum + (f.deletions || 0), 0),
		}

		statsCache.set(executionId, stats)
		return stats
	} catch (error) {
		logger.error(`Failed to fetch stats for execution ${executionId}: ${error}`)
		return {
			filesAdded: 0,
			filesRemoved: 0,
			filesModified: 0,
			linesAdded: 0,
			linesRemoved: 0,
		}
	}
}

/**
 * Clear cached stats for an execution (e.g., after commit)
 */
export function clearExecutionStats(executionId?: string) {
	if (executionId) {
		statsCache.delete(executionId)
	} else {
		statsCache.clear()
	}
}
