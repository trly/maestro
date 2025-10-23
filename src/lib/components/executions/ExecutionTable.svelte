<script lang="ts">
	import type { Execution, Repository } from '$lib/types'
	import type { SortSpec, ColumnFilters } from './types'
	import { useSelection } from '$lib/composables/useSelection.svelte'
	import ExecutionFilters from './ExecutionFilters.svelte'
	import ExecutionList from './ExecutionList.svelte'
	import BulkActionBar from '$lib/components/ui/BulkActionBar.svelte'

	const props = $props<{
	executions: Execution[]
	repositories: Map<string, Repository>
	hasValidationPrompt?: boolean
	executionsVersion?: number
	revisionId?: string
	// Loading states
	pushingExecutions: Set<string>
	refreshingCi: Set<string>
	loadingStats: Set<string>
	bulkStarting: boolean
	bulkRestarting: boolean
	bulkValidating: boolean
	bulkRevalidating: boolean
	bulkDeleting: boolean
	// Analysis
	onAnalyzeExecutions?: () => void
	onAnalyzeValidations?: () => void
	analyzingExecutions?: boolean
	analyzingValidations?: boolean
	// Action handlers
	onDeleteExecution: (execution: Execution, repoName: string) => void
	onStartExecution: (execution: Execution) => void
	onValidateExecution: (execution: Execution) => void
	onStopExecution: (execution: Execution) => void
	onStopValidation: (execution: Execution) => void
	onResumeExecution: (execution: Execution) => void
	onReviewChanges: (executionId: string) => void
	onPushExecution: (execution: Execution) => void
	onRefreshCi: (execution: Execution) => void
	onLoadStats: (executionId: string, status: string) => void
	onBulkDelete: (executions: Execution[]) => void
	onBulkStart: (executions: Execution[]) => void
	onBulkRestart: (executions: Execution[]) => void
	onBulkStartValidations: (executions: Execution[]) => void
	onBulkRevalidate: (executions: Execution[]) => void
}>()

	// Filters + sort state
	let filters = $state<ColumnFilters>({})
	let sort = $state<SortSpec>({ key: 'createdAt', dir: 'desc' })

	// Reset filters when switching revisions
	$effect(() => {
		const _revisionId = props.revisionId
		filters = {}
	})

	function setFilters(next: ColumnFilters) {
		// Normalize filters
		const norm = (v?: string | null) => (v == null || v === '' || v === 'all' ? undefined : v)
		const f = { ...next }
		f.repository = typeof f.repository === 'string' ? f.repository.trim() || undefined : undefined
		f.status = norm(f.status as any) as any
		f.validationStatus = norm(f.validationStatus as any) as any
		f.ciStatus = norm(f.ciStatus as any) as any
		f.changes = norm(f.changes as any) as any

		// Mutate in place for rune stability
		for (const k of Object.keys(filters)) delete (filters as any)[k]
		Object.assign(filters, f)
	}

	// Derived helpers
	let executionsById = $derived.by(() => {
		const _v = props.executionsVersion
		const map = new Map<string, Execution>()
		for (const e of props.executions) map.set(e.id, e)
		return map
	})

	let filteredExecutions = $derived.by(() => {
		const _repoSize = props.repositories.size
		const repoContains = (repoId: string, q: string) => {
			const name = props.repositories.get(repoId)?.providerId || repoId
			return name.toLowerCase().includes(q.toLowerCase())
		}
		return props.executions.filter((e: Execution) => {
			if (filters.repository && !repoContains(e.repositoryId, filters.repository)) return false
			if (filters.status && e.status !== filters.status) return false
			if (filters.validationStatus && e.validationStatus !== filters.validationStatus) return false
			if (filters.ciStatus && e.ciStatus !== filters.ciStatus) return false
			if (filters.changes) {
				const files = (e.filesAdded || 0) + (e.filesRemoved || 0) + (e.filesModified || 0)
				if (filters.changes === 'has-changes' && files === 0) return false
				if (filters.changes === 'no-changes' && files > 0) return false
			}
			return true
		})
	})

	const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })

	let sortedExecutions = $derived.by(() => {
		const s = [...filteredExecutions]
		const dir = sort.dir === 'asc' ? 1 : -1
		const cmp = {
			repo: (a: Execution, b: Execution) => {
				const an = props.repositories.get(a.repositoryId)?.providerId || a.repositoryId
				const bn = props.repositories.get(b.repositoryId)?.providerId || b.repositoryId
				return collator.compare(an, bn)
			},
			status: (a: Execution, b: Execution) => collator.compare(a.status, b.status),
			validation: (a: Execution, b: Execution) =>
				collator.compare(a.validationStatus || 'pending', b.validationStatus || 'pending'),
			ci: (a: Execution, b: Execution) =>
				collator.compare(a.ciStatus || 'not_configured', b.ciStatus || 'not_configured'),
			commit: (a: Execution, b: Execution) => collator.compare(a.commitStatus, b.commitStatus),
			diff: (a: Execution, b: Execution) => {
				const da = (a.linesAdded || 0) + (a.linesRemoved || 0)
				const db = (b.linesAdded || 0) + (b.linesRemoved || 0)
				return da - db
			},
			createdAt: (a: Execution, b: Execution) => (a.createdAt || 0) - (b.createdAt || 0),
			completedAt: (a: Execution, b: Execution) => (a.completedAt || 0) - (b.completedAt || 0)
		}[sort.key]
		s.sort((a, b) => {
			const r = cmp(a, b)
			return r === 0 ? dir * collator.compare(a.id, b.id) : dir * r
		})
		return s
	})

	let filteredSortedIds = $derived(sortedExecutions.map((e) => e.id))

	// Selection
	const selection = useSelection()

	// Bulk actions
	let selectedExecutions = $derived.by(() => {
		return Array.from(selection.selectedIds)
			.map(id => executionsById.get(id))
			.filter((e): e is Execution => e !== undefined)
	})

	function handleBulkDelete() {
		props.onBulkDelete(selectedExecutions)
		selection.clear()
	}

	function handleBulkStart() {
		props.onBulkStart(selectedExecutions)
		selection.clear()
	}

	function handleBulkRestart() {
		props.onBulkRestart(selectedExecutions)
		selection.clear()
	}

	function handleBulkStartValidations() {
		props.onBulkStartValidations(selectedExecutions)
		selection.clear()
	}

	function handleBulkRevalidate() {
		props.onBulkRevalidate(selectedExecutions)
		selection.clear()
	}

	function handleToggleSelectAll() {
		selection.toggleAll(filteredSortedIds)
	}

	function handleChangeSort(key: SortSpec['key']) {
		if (sort.key === key) {
			sort.dir = sort.dir === 'asc' ? 'desc' : 'asc'
		} else {
			sort.key = key
			sort.dir = 'asc'
		}
	}

	// Execution helpers
	function getExecution(id: string) {
		return executionsById.get(id)!
	}

	function getRepoName(execution: Execution): string {
		return props.repositories.get(execution.repositoryId)?.providerId || execution.repositoryId
	}
</script>

{#if selection.selectedIds.size > 0}
	<BulkActionBar
		selectedCount={selection.selectedIds.size}
		hasValidationPrompt={props.hasValidationPrompt}
		onBulkDelete={handleBulkDelete}
		onBulkStart={handleBulkStart}
		onBulkRestart={handleBulkRestart}
		onBulkStartValidations={props.hasValidationPrompt ? handleBulkStartValidations : undefined}
		onBulkRevalidate={props.hasValidationPrompt ? handleBulkRevalidate : undefined}
		onClear={selection.clear}
		isStarting={props.bulkStarting}
		isRestarting={props.bulkRestarting}
		isValidating={props.bulkValidating}
		isRevalidating={props.bulkRevalidating}
		isDeleting={props.bulkDeleting}
	/>
{/if}

<div class="flex-1 min-h-0 flex flex-col overflow-hidden @container/table">
	<ExecutionFilters {filters} onFilterChange={setFilters} />

	<ExecutionList
	ids={filteredSortedIds}
	{executionsById}
	repositories={props.repositories}
	{selection}
	pushingExecutions={props.pushingExecutions}
	refreshingCi={props.refreshingCi}
	loadingStats={props.loadingStats}
	{sort}
	hasValidationPrompt={props.hasValidationPrompt}
	onAnalyzeExecutions={props.onAnalyzeExecutions}
	onAnalyzeValidations={props.onAnalyzeValidations}
	analyzingExecutions={props.analyzingExecutions}
	analyzingValidations={props.analyzingValidations}
	onToggleSelectAll={handleToggleSelectAll}
	onChangeSort={handleChangeSort}
	onLoadStats={(id) => {
	const execution = getExecution(id)
	props.onLoadStats(id, execution.status)
	}}
	onLoadCi={(id) => {
	 const execution = getExecution(id)
	 if (execution.commitStatus === 'committed') {
	  props.onRefreshCi(execution)
	 }
	}}
	onStart={(id) => props.onStartExecution(getExecution(id))}
	onStop={(id) => props.onStopExecution(getExecution(id))}
	onRestart={(id) => props.onResumeExecution(getExecution(id))}
	onValidate={(id) => props.onValidateExecution(getExecution(id))}
	onStopValidation={(id) => props.onStopValidation(getExecution(id))}
	onDelete={(id) => {
	 const execution = getExecution(id)
	 props.onDeleteExecution(execution, getRepoName(execution))
	}}
	onReviewChanges={(id) => props.onReviewChanges(id)}
	onPush={(id) => props.onPushExecution(getExecution(id))}
	onRefreshCi={(id) => props.onRefreshCi(getExecution(id))}
/>
</div>
