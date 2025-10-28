import type { Execution, Repository } from "$lib/types"
import type { ColumnFilters } from "$lib/components/executions/types"

interface UseExecutionFiltersOptions {
	executions: () => Execution[]
	repositories: () => Map<string, Repository>
}

export function useExecutionFilters(options: UseExecutionFiltersOptions) {
	let filters = $state<ColumnFilters>({})

	function getRepoName(repoId: string): string {
		return options.repositories().get(repoId)?.providerId || repoId
	}

	// Helper to check if a filter value should be treated as "no filter"
	function isNoFilter(v?: string | null): boolean {
		return v == null || v === "" || v === "all"
	}

	// Normalize filter values - convert 'all' and empty strings to undefined
	function normalizeFilters(f: ColumnFilters): ColumnFilters {
		const normalized = { ...f }

		// Normalize select filters
		if (isNoFilter(normalized.status)) delete normalized.status
		if (isNoFilter(normalized.validationStatus)) delete normalized.validationStatus
		if (isNoFilter(normalized.changes)) delete normalized.changes
		if (isNoFilter(normalized.ciStatus)) delete normalized.ciStatus

		// Normalize text input filters
		if (typeof normalized.repository === "string") {
			normalized.repository = normalized.repository.trim()
			if (normalized.repository === "") delete normalized.repository
		}

		return normalized
	}

	let filteredExecutions = $derived.by(() => {
		const list = options.executions()
		// Read length to react to in-place mutations (defensive)
		const _len = list.length

		// Read Map size to react to repository Map updates
		const repos = options.repositories()
		const _repoSize = repos.size

		return list.filter((execution) => {
			// Repository filter (case insensitive contains)
			if (filters.repository) {
				const repoName = (
					repos.get(execution.repositoryId)?.providerId || execution.repositoryId
				).toLowerCase()
				if (!repoName.includes(filters.repository.toLowerCase())) {
					return false
				}
			}

			// Status filter
			if (filters.status && execution.status !== filters.status) {
				return false
			}

			// Validation status filter
			if (filters.validationStatus && execution.validationStatus !== filters.validationStatus) {
				return false
			}

			// Changes filter
			if (filters.changes) {
				const fileCount =
					(execution.filesAdded || 0) +
					(execution.filesRemoved || 0) +
					(execution.filesModified || 0)
				if (filters.changes === "has-changes" && fileCount === 0) {
					return false
				}
				if (filters.changes === "no-changes" && fileCount > 0) {
					return false
				}
			}

			// CI status filter
			if (filters.ciStatus && execution.ciStatus !== filters.ciStatus) {
				return false
			}

			return true
		})
	})

	function setFilters(newFilters: ColumnFilters) {
		const normalized = normalizeFilters(newFilters)

		// Remove keys that no longer exist
		for (const k of Object.keys(filters) as (keyof ColumnFilters)[]) {
			if (!(k in normalized)) {
				delete filters[k]
			}
		}

		// Assign new/changed keys (mutate in place for reactivity)
		Object.assign(filters, normalized)
	}

	function clearFilters() {
		// Mutate in place to maintain reactivity
		for (const k of Object.keys(filters) as (keyof ColumnFilters)[]) {
			delete filters[k]
		}
	}

	return {
		filters,
		filteredExecutions,
		setFilters,
		clearFilters,
	}
}
