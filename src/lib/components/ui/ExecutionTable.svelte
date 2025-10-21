<script lang="ts">
	import type { Execution, Repository } from '$lib/types'
	import ExecutionTableHeader from './ExecutionTableHeader.svelte'
	import ExecutionRow from './ExecutionRow.svelte'
	import BulkActionBar from './BulkActionBar.svelte'
	import { useSelection } from '$lib/composables/useSelection.svelte'
	import { useExecutionFilters } from '$lib/composables/useExecutionFilters.svelte'
	import { executionStore } from '$lib/stores/executionBus'
	
	const props: {
		executions: Execution[]
		repositories: Map<string, Repository>
		hasValidationPrompt?: boolean
		onDeleteExecution: (execution: Execution, repoName: string) => void
		onStartExecution: (execution: Execution) => void
		onValidateExecution: (execution: Execution) => void
		onStopExecution: (execution: Execution) => void
		onStopValidation: (execution: Execution) => void
		onResumeExecution: (execution: Execution) => void
		onReviewChanges: (executionId: string) => void
		onPushExecution: (execution: Execution) => void
		onRefreshCi: (execution: Execution) => void
		onBulkDelete: (executions: Execution[]) => void
		onBulkStart: (executions: Execution[]) => void
		onBulkRestart: (executions: Execution[]) => void
		onBulkStartValidations: (executions: Execution[]) => void
		onBulkRevalidate: (executions: Execution[]) => void
		onExecuteAll?: () => void
		onStopAll?: () => void
		onStopAllValidations?: () => void
		onRefreshAllCi?: () => void
		onAnalyzeExecutions?: () => void
		onAnalyzeValidations?: () => void
		pushingExecutions: Set<string>
		refreshingCi: Set<string>
		analyzingExecutions: boolean
		analyzingValidations: boolean
		bulkStarting: boolean
		bulkRestarting: boolean
		bulkValidating: boolean
		bulkRevalidating: boolean
		bulkDeleting: boolean
	} = $props()
	
	let sortColumn = $state<string | null>(null)
	let sortDirection = $state<'asc' | 'desc'>('asc')
	
	const selection = useSelection()
	
	// Merge event bus updates with static execution data
	let executionsLive = $derived.by(() => {
		const updates = $executionStore
		return props.executions.map(e => {
			const data = updates.get(e.id)
			if (!data) return e
			return {
				...e,
				...(data.sessionId && { sessionId: data.sessionId }),
				...(data.threadUrl && { threadUrl: data.threadUrl }),
				...(data.status && { status: data.status }),
				...(data.validationStatus && { validationStatus: data.validationStatus }),
				...(data.validationThreadUrl && { validationThreadUrl: data.validationThreadUrl }),
				...(data.commitStatus && { commitStatus: data.commitStatus }),
				...(data.commitSha && { commitSha: data.commitSha }),
				...(data.committedAt && { committedAt: data.committedAt }),
				...(data.ciStatus && { ciStatus: data.ciStatus }),
				...(data.ciUrl && { ciUrl: data.ciUrl }),
				...(data.progressMessage && { progressMessage: data.progressMessage })
			}
		})
	})
	
	// Use filter composable
	const executionFilters = useExecutionFilters({
		executions: () => executionsLive,
		repositories: () => props.repositories
	})
	
	let executionIds = $derived(executionFilters.filteredExecutions.map(e => e.id))
	let selectedExecutions = $derived(selection.getSelected(executionsLive))
	
	let allSelected = $derived(executionFilters.filteredExecutions.length > 0 && selection.selectedIds.size === executionIds.length)
	let someSelected = $derived(selection.selectedIds.size > 0 && !allSelected)
	
	let hasRunning = $derived(executionsLive.some(e => e.status === 'running'))
	let hasRunningValidations = $derived(executionsLive.some(e => e.validationStatus === 'running'))
	let hasFailedExecutions = $derived(executionsLive.some(e => e.status === 'failed'))
	let hasFailedValidations = $derived(executionsLive.some(e => e.validationStatus === 'failed'))
	let hasCommitted = $derived(executionsLive.some(e => e.commitStatus === 'committed'))
	
	let sortedExecutions = $derived.by(() => {
		if (!sortColumn) return executionFilters.filteredExecutions

		return [...executionFilters.filteredExecutions].sort((a, b) => {
			let aVal: any
			let bVal: any

			switch (sortColumn) {
				case 'repository':
					aVal = getRepoName(a.repositoryId).toLowerCase()
					bVal = getRepoName(b.repositoryId).toLowerCase()
					break
				case 'status':
					aVal = a.status
					bVal = b.status
					break
				case 'validationStatus':
					aVal = a.validationStatus || ''
					bVal = b.validationStatus || ''
					break
				case 'commitStatus':
					aVal = a.commitStatus || ''
					bVal = b.commitStatus || ''
					break
				default:
					return 0
			}

			if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1
			if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1
			return 0
		})
	})
	
	function getRepoName(repoId: string): string {
		return props.repositories.get(repoId)?.providerId || repoId
	}
	
	function handleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc'
		} else {
			sortColumn = column
			sortDirection = 'asc'
		}
	}
	
	function handleToggleSelectAll() {
		selection.toggleAll(executionIds)
	}
	
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

<div class="flex-1 min-h-0 flex flex-col overflow-hidden">
	<div class="flex-1 overflow-auto">
	<ExecutionTableHeader
		{allSelected}
		{someSelected}
		{sortColumn}
		{sortDirection}
		{hasRunning}
		{hasRunningValidations}
		{hasFailedExecutions}
		{hasFailedValidations}
		{hasCommitted}
		filters={executionFilters.filters}
		executions={executionsLive}
		executionCount={executionsLive.length}
		onToggleSelectAll={handleToggleSelectAll}
		onSort={handleSort}
		onFilterChange={executionFilters.setFilters}
		onExecuteAll={props.onExecuteAll}
		onStopAll={props.onStopAll}
		onStopAllValidations={props.onStopAllValidations}
		onRefreshAllCi={props.onRefreshAllCi}
		onAnalyzeExecutions={props.onAnalyzeExecutions}
		onAnalyzeValidations={props.onAnalyzeValidations}
		analyzingExecutions={props.analyzingExecutions}
		analyzingValidations={props.analyzingValidations}
	/>
	
	<div class="divide-y divide-border/40">
		{#each sortedExecutions as execution (execution.id)}
			<ExecutionRow
				{execution}
				repoName={getRepoName(execution.repositoryId)}
				hasValidationPrompt={props.hasValidationPrompt}
				selected={selection.selectedIds.has(execution.id)}
				onToggleSelect={() => selection.toggle(execution.id)}
				onDelete={() => props.onDeleteExecution(execution, getRepoName(execution.repositoryId))}
				onStart={() => props.onStartExecution(execution)}
				onValidate={() => props.onValidateExecution(execution)}
				onStop={() => props.onStopExecution(execution)}
				onStopValidation={() => props.onStopValidation(execution)}
				onResume={() => props.onResumeExecution(execution)}
				onReviewChanges={() => props.onReviewChanges(execution.id)}
				onPush={() => props.onPushExecution(execution)}
				onRefreshCi={() => props.onRefreshCi(execution)}
				fileCount={(execution.filesAdded || 0) + (execution.filesRemoved || 0) + (execution.filesModified || 0)}
				additions={execution.linesAdded || 0}
				deletions={execution.linesRemoved || 0}
				progressMessage={execution.progressMessage}
				isPushing={props.pushingExecutions.has(execution.id)}
				isRefreshingCi={props.refreshingCi.has(execution.id)}
			/>
		{/each}
	</div>
	</div>
</div>
