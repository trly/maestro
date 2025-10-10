<script lang="ts">
	import { X, Trash2, CheckSquare, Square as SquareIcon, ChevronDown, ChevronUp, Play } from 'lucide-svelte';
	import UiTooltip from './UiTooltip.svelte';
	import UiScrollArea from './UiScrollArea.svelte';
	import ExecutionRow from './ExecutionRow.svelte';
	import { toShortHash } from '$lib/utils';
	import type { PromptRevision, Execution, Repository } from '$lib/types';

	let {
		revision,
		executions,
		repositories,
		hasValidationPrompt = false,
		onExecuteAll,
		onStopAll,
		onStopAllValidations,
		onDelete,
		onDeleteExecution,
		onValidateExecution,
		onStopExecution,
		onStopValidation,
		onResumeExecution,
		onReviewChanges,
		onBulkDelete,
		onBulkRestart,
		onBulkRevalidate
	}: {
		revision: PromptRevision;
		executions: Execution[];
		repositories: Map<string, Repository>;
		hasValidationPrompt?: boolean;
		onExecuteAll: () => void;
		onStopAll: () => void;
		onStopAllValidations: () => void;
		onDelete: () => void;
		onDeleteExecution: (execution: Execution, repoName: string) => void;
		onValidateExecution: (execution: Execution) => void;
		onStopExecution: (execution: Execution) => void;
		onStopValidation: (execution: Execution) => void;
		onResumeExecution: (execution: Execution) => void;
		onReviewChanges: (executionId: string) => void;
		onBulkDelete: (executions: Execution[]) => void;
		onBulkRestart: (executions: Execution[]) => void;
		onBulkRevalidate: (executions: Execution[]) => void;
	} = $props();

	let selectedIds = $state<Set<string>>(new Set());
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let promptHeight = $state(256); // Default height in pixels
	let isResizing = $state(false);

	function handleResizeStart(e: MouseEvent) {
		isResizing = true;
		e.preventDefault();
	}

	function handleResizeMove(e: MouseEvent) {
		if (!isResizing) return;
		promptHeight = Math.max(100, Math.min(600, e.clientY - 100));
	}

	function handleResizeEnd() {
		isResizing = false;
	}

	$effect(() => {
		if (isResizing) {
			document.addEventListener('mousemove', handleResizeMove);
			document.addEventListener('mouseup', handleResizeEnd);
			return () => {
				document.removeEventListener('mousemove', handleResizeMove);
				document.removeEventListener('mouseup', handleResizeEnd);
			};
		}
	});

	let hasRunning = $derived(executions.some(e => e.status === 'running'));
	let hasRunningValidations = $derived(executions.some(e => e.validationStatus === 'running'));
	let allSelected = $derived(executions.length > 0 && selectedIds.size === executions.length);
	let someSelected = $derived(selectedIds.size > 0 && !allSelected);
	let selectedExecutions = $derived(executions.filter(e => selectedIds.has(e.id)));

	let sortedExecutions = $derived.by(() => {
		if (!sortColumn) return executions;

		return [...executions].sort((a, b) => {
			let aVal: any;
			let bVal: any;

			switch (sortColumn) {
				case 'repository':
					aVal = getRepoName(a.repositoryId).toLowerCase();
					bVal = getRepoName(b.repositoryId).toLowerCase();
					break;
				case 'status':
					aVal = a.status;
					bVal = b.status;
					break;
				case 'validationStatus':
					aVal = a.validationStatus || '';
					bVal = b.validationStatus || '';
					break;
				case 'commitStatus':
					aVal = a.commitStatus || '';
					bVal = b.commitStatus || '';
					break;
				default:
					return 0;
			}

			if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1;
			if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1;
			return 0;
		});
	});

	function getRepoName(repoId: string): string {
		return repositories.get(repoId)?.providerId || repoId;
	}

	function toggleSelectAll() {
		if (allSelected) {
			selectedIds = new Set();
		} else {
			selectedIds = new Set(executions.map(e => e.id));
		}
	}

	function toggleSelect(executionId: string) {
		const newSet = new Set(selectedIds);
		if (newSet.has(executionId)) {
			newSet.delete(executionId);
		} else {
			newSet.add(executionId);
		}
		selectedIds = newSet;
	}

	function handleSort(column: string) {
		if (sortColumn === column) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortColumn = column;
			sortDirection = 'asc';
		}
	}

	function handleBulkDelete() {
		onBulkDelete(selectedExecutions);
		selectedIds = new Set();
	}

	function handleBulkRestart() {
		onBulkRestart(selectedExecutions);
		selectedIds = new Set();
	}

	function handleBulkRevalidate() {
		onBulkRevalidate(selectedExecutions);
		selectedIds = new Set();
	}
</script>

<div class="flex-1 flex flex-col overflow-hidden bg-background">
	<!-- Prompt Console -->
	<div class="flex-shrink-0 border-b border-border/40 overflow-hidden bg-card">
		<!-- Console Header -->
		<div class="bg-muted/20 px-4 py-3 border-b border-border/40 flex items-center justify-between">
			<div class="flex items-center gap-3">
				<h2 class="text-sm font-bold text-foreground">
					Revision {toShortHash(revision.id)}
				</h2>
				<span class="text-xs text-muted-foreground">
					{new Date(revision.createdAt).toLocaleString()}
				</span>
			</div>

			<!-- Actions -->
			<div class="flex items-center gap-1">
				<UiTooltip content="Execute on all repos">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onExecuteAll}
							class="w-7 h-7 flex items-center justify-center rounded-md bg-green-100 text-green-600 hover:bg-green-200 transition-all"
							aria-label="Execute on all repos"
						>
							<Play class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>

				{#if hasRunning}
					<UiTooltip content="Stop all running executions">
						{#snippet children({ props })}
							<button
								{...props}
								onclick={onStopAll}
								class="w-7 h-7 flex items-center justify-center rounded-md bg-orange-100 text-orange-600 hover:bg-orange-200 transition-all"
								aria-label="Stop all executions"
							>
								<X class="w-3.5 h-3.5" />
							</button>
						{/snippet}
					</UiTooltip>
				{/if}

				{#if hasRunningValidations}
					<UiTooltip content="Stop all running validations">
						{#snippet children({ props })}
							<button
								{...props}
								onclick={onStopAllValidations}
								class="w-7 h-7 flex items-center justify-center rounded-md bg-orange-100 text-orange-600 hover:bg-orange-200 transition-all"
								aria-label="Stop all validations"
							>
								<X class="w-3.5 h-3.5" />
							</button>
						{/snippet}
					</UiTooltip>
				{/if}

				<UiTooltip content="Delete revision">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onDelete}
							class="w-7 h-7 flex items-center justify-center rounded-md text-red-600 hover:bg-red-50 transition-all"
							aria-label="Delete revision"
						>
							<Trash2 class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>
			</div>
		</div>

		<!-- Prompt Content -->
		<UiScrollArea viewportClass="overflow-auto" style="max-height: {promptHeight}px;">
			<pre class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{revision.promptText}</pre>
		</UiScrollArea>
	</div>

	<!-- Resize Handle -->
	<button
		onmousedown={handleResizeStart}
		class="flex-shrink-0 h-1.5 bg-border/40 hover:bg-primary/40 transition-colors cursor-ns-resize group relative"
		aria-label="Resize prompt area"
	>
		<div class="absolute inset-x-0 -top-1 -bottom-1"></div>
	</button>

	<!-- Executions Table -->
	{#if executions.length === 0}
		<div class="flex-1 flex items-center justify-center text-muted-foreground">
			<div class="text-center">
				<p class="text-sm mb-2">No executions yet for this revision</p>
				<p class="text-xs">Click "Execute on all repos" in the sidebar to run this prompt</p>
			</div>
		</div>
	{:else}
		<div class="flex-1 flex flex-col overflow-hidden @container/table">
			<!-- Bulk Actions Toolbar -->
			{#if selectedIds.size > 0}
				<div class="flex-shrink-0 flex items-center gap-3 px-4 py-3 bg-primary/5 border-b border-border/40">
					<span class="text-sm font-medium text-foreground">{selectedIds.size} selected</span>
					<div class="flex items-center gap-2">
						<button
							onclick={handleBulkRestart}
							class="px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
						>
							Restart
						</button>
						{#if hasValidationPrompt}
							<button
								onclick={handleBulkRevalidate}
								class="px-3 py-1.5 text-xs font-medium rounded-md bg-green-600 text-white hover:bg-green-700 transition-colors"
							>
								Revalidate
							</button>
						{/if}
						<button
							onclick={handleBulkDelete}
							class="px-3 py-1.5 text-xs font-medium rounded-md bg-red-600 text-white hover:bg-red-700 transition-colors"
						>
							Delete
						</button>
					</div>
					<button
						type="button"
						onclick={() => selectedIds = new Set()}
						class="ml-auto text-xs text-muted-foreground hover:text-foreground underline"
					>
						Clear selection
					</button>
				</div>
			{/if}

			<!-- Table Container (Header + Body) -->
			<UiScrollArea class="flex-1">
				<div>
					<!-- Table Header -->
					<div
						class="sticky top-0 z-10 grid gap-3 px-4 py-2.5 bg-card border-b border-border/40 text-xs font-medium text-muted-foreground items-center
							[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1fr)]
							@max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_0.8fr)_minmax(0,_1fr)_minmax(0,_0.8fr)]
							@max-md/table:[grid-template-columns:auto_minmax(200px,_6fr)_40px_40px_0_0_40px]"
					>
						<button
							type="button"
							onclick={toggleSelectAll}
							class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2 border-muted-foreground/30 hover:border-primary/50 transition-colors"
							aria-label={allSelected ? 'Deselect all' : 'Select all'}
						>
							{#if allSelected}
								<CheckSquare class="w-4 h-4 text-primary" />
							{:else if someSelected}
								<div class="w-2 h-2 bg-primary rounded-sm"></div>
							{:else}
								<SquareIcon class="w-4 h-4 text-muted-foreground/30" />
							{/if}
						</button>
						<button
							onclick={() => handleSort('repository')}
							class="text-left hover:text-foreground transition-colors truncate"
						>
							Repository
							{#if sortColumn === 'repository'}
								{#if sortDirection === 'asc'}
									<ChevronUp class="w-3 h-3 inline ml-1" />
								{:else}
									<ChevronDown class="w-3 h-3 inline ml-1" />
								{/if}
							{/if}
						</button>
						<button
							onclick={() => handleSort('status')}
							class="text-left hover:text-foreground transition-colors truncate"
						>
							Execution
							{#if sortColumn === 'status'}
								{#if sortDirection === 'asc'}
									<ChevronUp class="w-3 h-3 inline ml-1" />
								{:else}
									<ChevronDown class="w-3 h-3 inline ml-1" />
								{/if}
							{/if}
						</button>
						<button
							onclick={() => handleSort('validationStatus')}
							class="text-left hover:text-foreground transition-colors truncate"
						>
							Validation
							{#if sortColumn === 'validationStatus'}
								{#if sortDirection === 'asc'}
									<ChevronUp class="w-3 h-3 inline ml-1" />
								{:else}
									<ChevronDown class="w-3 h-3 inline ml-1" />
								{/if}
							{/if}
						</button>
						<button
							onclick={() => handleSort('commitStatus')}
							class="text-left hover:text-foreground transition-colors truncate @max-md/table:hidden"
						>
							Commit
							{#if sortColumn === 'commitStatus'}
								{#if sortDirection === 'asc'}
									<ChevronUp class="w-3 h-3 inline ml-1" />
								{:else}
									<ChevronDown class="w-3 h-3 inline ml-1" />
								{/if}
							{/if}
						</button>
						<div class="text-left truncate @max-md/table:hidden">Changes</div>
						<div class="text-right truncate">Actions</div>
					</div>

					<!-- Table Body -->
					<div class="divide-y divide-border/40">
						{#each sortedExecutions as execution (execution.id)}
							<ExecutionRow
								{execution}
								repoName={getRepoName(execution.repositoryId)}
								{hasValidationPrompt}
								selected={selectedIds.has(execution.id)}
								onToggleSelect={() => toggleSelect(execution.id)}
								onDelete={() => onDeleteExecution(execution, getRepoName(execution.repositoryId))}
								onValidate={() => onValidateExecution(execution)}
								onStop={() => onStopExecution(execution)}
								onStopValidation={() => onStopValidation(execution)}
								onResume={() => onResumeExecution(execution)}
								onReviewChanges={() => onReviewChanges(execution.id)}
								fileCount={(execution.filesAdded || 0) + (execution.filesRemoved || 0) + (execution.filesModified || 0)}
								additions={execution.linesAdded || 0}
								deletions={execution.linesRemoved || 0}
								progressMessage={execution.progressMessage}
							/>
						{/each}
					</div>
				</div>
			</UiScrollArea>
		</div>
	{/if}
</div>
