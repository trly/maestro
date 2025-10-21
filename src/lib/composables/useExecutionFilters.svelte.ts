import type { Execution, Repository } from '$lib/types'
import type { ColumnFilters } from '$lib/components/ui/ExecutionTableHeader.svelte'

interface UseExecutionFiltersOptions {
	executions: () => Execution[]
	repositories: () => Map<string, Repository>
}

export function useExecutionFilters(options: UseExecutionFiltersOptions) {
	
	let filters = $state<ColumnFilters>({})
	
	function getRepoName(repoId: string): string {
		return options.repositories().get(repoId)?.providerId || repoId
	}
	
	let filteredExecutions = $derived.by(() => {
		return options.executions().filter(execution => {
			// Repository filter (case insensitive contains)
			if (filters.repository) {
				const repoName = getRepoName(execution.repositoryId).toLowerCase()
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
				const fileCount = (execution.filesAdded || 0) + (execution.filesRemoved || 0) + (execution.filesModified || 0)
				if (filters.changes === 'has-changes' && fileCount === 0) {
					return false
				}
				if (filters.changes === 'no-changes' && fileCount > 0) {
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
		filters = newFilters
	}
	
	function clearFilters() {
		filters = {}
	}
	
	return {
		get filters() {
			return filters
		},
		get filteredExecutions() {
			return filteredExecutions
		},
		setFilters,
		clearFilters
	}
}
