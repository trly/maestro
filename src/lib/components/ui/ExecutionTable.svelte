<script lang="ts">
	import type { Execution, Repository } from '$lib/types'
	import ExecutionTableHeader from './ExecutionTableHeader.svelte'
	import ExecutionRow from './ExecutionRow.svelte'
	import BulkActionBar from './BulkActionBar.svelte'
	import { useSelection } from '$lib/composables/useSelection.svelte'
	import { executionStore } from '$lib/stores/executionBus'
	
	let {
		executions,
		repositories,
		hasValidationPrompt = false,
		onDeleteExecution,
		onStartExecution,
		onValidateExecution,
		onStopExecution,
		onStopValidation,
		onResumeExecution,
		onReviewChanges,
		onPushExecution,
		onRefreshCi,
		onBulkDelete,
		onBulkStart,
		onBulkRestart,
		onBulkStartValidations,
		onBulkRevalidate,
		onExecuteAll,
		onStopAll,
		onStopAllValidations,
		onRefreshAllCi,
		onAnalyzeExecutions,
		onAnalyzeValidations
	}: {
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
	} = $props()
	
	let sortColumn = $state<string | null>(null)
	let sortDirection = $state<'asc' | 'desc'>('asc')
	
	const selection = useSelection()
	
	// Merge event bus updates with static execution data
	let executionsLive = $derived(
		executions.map(e => {
			const updates = $executionStore.get(e.id)
			if (!updates) return e
			return {
				...e,
				...(updates.sessionId && { sessionId: updates.sessionId }),
				...(updates.threadUrl && { threadUrl: updates.threadUrl }),
				...(updates.status && { status: updates.status }),
				...(updates.validationStatus && { validationStatus: updates.validationStatus }),
				...(updates.validationThreadUrl && { validationThreadUrl: updates.validationThreadUrl }),
				...(updates.commitStatus && { commitStatus: updates.commitStatus }),
				...(updates.commitSha && { commitSha: updates.commitSha }),
				...(updates.committedAt && { committedAt: updates.committedAt }),
				...(updates.ciStatus && { ciStatus: updates.ciStatus }),
				...(updates.ciUrl && { ciUrl: updates.ciUrl }),
				...(updates.progressMessage && { progressMessage: updates.progressMessage })
			}
		})
	)
	
	let executionIds = $derived(executionsLive.map(e => e.id))
	let selectedExecutions = $derived(selection.getSelected(executionsLive))
	
	let allSelected = $derived(executionsLive.length > 0 && selection.selectedIds.size === executionIds.length)
	let someSelected = $derived(selection.selectedIds.size > 0 && !allSelected)
	
	let hasRunning = $derived(executionsLive.some(e => e.status === 'running'))
	let hasRunningValidations = $derived(executionsLive.some(e => e.validationStatus === 'running'))
	let hasFailedExecutions = $derived(executionsLive.some(e => e.status === 'failed'))
	let hasFailedValidations = $derived(executionsLive.some(e => e.validationStatus === 'failed'))
	let hasCommitted = $derived(executionsLive.some(e => e.commitStatus === 'committed'))
	
	let sortedExecutions = $derived.by(() => {
		if (!sortColumn) return executionsLive

		return [...executionsLive].sort((a, b) => {
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
		return repositories.get(repoId)?.providerId || repoId
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
		onBulkDelete(selectedExecutions)
		selection.clear()
	}
	
	function handleBulkStart() {
		onBulkStart(selectedExecutions)
		selection.clear()
	}
	
	function handleBulkRestart() {
		onBulkRestart(selectedExecutions)
		selection.clear()
	}
	
	function handleBulkStartValidations() {
		onBulkStartValidations(selectedExecutions)
		selection.clear()
	}
	
	function handleBulkRevalidate() {
		onBulkRevalidate(selectedExecutions)
		selection.clear()
	}
</script>

{#if selection.selectedIds.size > 0}
	<BulkActionBar
		selectedCount={selection.selectedIds.size}
		{hasValidationPrompt}
		onBulkDelete={handleBulkDelete}
		onBulkStart={handleBulkStart}
		onBulkRestart={handleBulkRestart}
		onBulkStartValidations={hasValidationPrompt ? handleBulkStartValidations : undefined}
		onBulkRevalidate={hasValidationPrompt ? handleBulkRevalidate : undefined}
		onClear={selection.clear}
	/>
{/if}

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
		executionCount={executionsLive.length}
		onToggleSelectAll={handleToggleSelectAll}
		onSort={handleSort}
		{onExecuteAll}
		{onStopAll}
		{onStopAllValidations}
		{onRefreshAllCi}
		{onAnalyzeExecutions}
		{onAnalyzeValidations}
	/>
	
	<div class="divide-y divide-border/40">
		{#each sortedExecutions as execution (execution.id)}
			<ExecutionRow
				{execution}
				repoName={getRepoName(execution.repositoryId)}
				{hasValidationPrompt}
				selected={selection.selectedIds.has(execution.id)}
				onToggleSelect={() => selection.toggle(execution.id)}
				onDelete={() => onDeleteExecution(execution, getRepoName(execution.repositoryId))}
				onStart={() => onStartExecution(execution)}
				onValidate={() => onValidateExecution(execution)}
				onStop={() => onStopExecution(execution)}
				onStopValidation={() => onStopValidation(execution)}
				onResume={() => onResumeExecution(execution)}
				onReviewChanges={() => onReviewChanges(execution.id)}
				onPush={() => onPushExecution(execution)}
				onRefreshCi={() => onRefreshCi(execution)}
				fileCount={(execution.filesAdded || 0) + (execution.filesRemoved || 0) + (execution.filesModified || 0)}
				additions={execution.linesAdded || 0}
				deletions={execution.linesRemoved || 0}
				progressMessage={execution.progressMessage}
			/>
		{/each}
	</div>
</div>
