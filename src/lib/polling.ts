import { api } from "./api"
import type { Execution } from "./types"

export function pollExecutions(
	revisionId: string,
	onUpdate: (executions: Execution[]) => void,
	intervalMs = 1500
): () => void {
	let cancelled = false

	async function tick() {
		if (cancelled) return
		try {
			const execs = await api.revisions.getExecutions(revisionId)
			onUpdate(execs)
		} catch (err) {
		} finally {
			if (!cancelled) {
				setTimeout(tick, intervalMs)
			}
		}
	}

	tick()
	return () => {
		cancelled = true
	}
}
