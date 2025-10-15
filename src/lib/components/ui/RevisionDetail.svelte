<script lang="ts">
	import { X, Trash2, CheckSquare, Square as SquareIcon, ChevronDown, ChevronUp, Edit, Save, Play, FolderGit, GitMerge } from 'lucide-svelte';
	import { Switch } from 'bits-ui';
	import UiScrollArea from './UiScrollArea.svelte';
	import ExecutionRow from './ExecutionRow.svelte';
	import RepositorySelector from '$lib/components/RepositorySelector.svelte';
	import type { PromptRevision, Execution, Repository } from '$lib/types';
	import type { Repository as ProviderRepository } from '$lib/providers/types';

	let {
		revision,
		executions,
		repositories,
		repositoryIds = [],
		hasValidationPrompt = false,
		validationPrompt = null,
		autoValidate = false,
		onDeleteExecution,
		onValidateExecution,
		onStopExecution,
		onStopValidation,
		onResumeExecution,
		onReviewChanges,
		onPushExecution,
		onRefreshCi,
		onBulkDelete,
		onBulkRestart,
		onBulkRevalidate,
		onSaveValidation,
		onSaveRepositories,
		onExecuteAll
	}: {
		revision: PromptRevision;
		executions: Execution[];
		repositories: Map<string, Repository>;
		repositoryIds?: string[];
		hasValidationPrompt?: boolean;
		validationPrompt?: string | null;
		autoValidate?: boolean;
		onDeleteExecution: (execution: Execution, repoName: string) => void;
		onValidateExecution: (execution: Execution) => void;
		onStopExecution: (execution: Execution) => void;
		onStopValidation: (execution: Execution) => void;
		onResumeExecution: (execution: Execution) => void;
		onReviewChanges: (executionId: string) => void;
		onPushExecution: (execution: Execution) => void;
		onRefreshCi: (execution: Execution) => void;
		onBulkDelete: (executions: Execution[]) => void;
		onBulkRestart: (executions: Execution[]) => void;
		onBulkRevalidate: (executions: Execution[]) => void;
		onSaveValidation: (prompt: string, autoValidate: boolean) => Promise<void>;
		onSaveRepositories: (repositoryIds: string[]) => Promise<void>;
		onExecuteAll: () => void;
	} = $props();

	let selectedIds = $state<Set<string>>(new Set());
	let sortColumn = $state<string | null>(null);
	let sortDirection = $state<'asc' | 'desc'>('asc');
	let promptHeight = $state(256); // Default height in pixels
	let isResizing = $state(false);
	let isEditingValidation = $state(false);
	let editedValidationPrompt = $state(validationPrompt || '');
	let editedAutoValidate = $state(autoValidate);
	let isSaving = $state(false);
	let manuallyResized = $state(false);
	let containerRef = $state<HTMLDivElement | null>(null);
	let isEditingRepositories = $state(false);
	let editedRepositories = $state<ProviderRepository[]>([]);
	let isSavingRepositories = $state(false);

	function handleResizeStart(e: MouseEvent) {
		isResizing = true;
		manuallyResized = true;
		e.preventDefault();
	}

	function handleResizeMove(e: MouseEvent) {
		if (!isResizing) return;
		promptHeight = Math.max(100, Math.min(600, e.clientY - 100));
	}

	function handleResizeEnd() {
		isResizing = false;
	}

	function calculateAutoHeight(): number {
		if (manuallyResized || !containerRef) return promptHeight;
		
		const containerHeight = containerRef.clientHeight;
		const maxHeight = Math.floor(containerHeight * 0.6);
		
		return Math.min(maxHeight, 600);
	}

	let computedHeight = $derived(calculateAutoHeight());

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

	function startEditingValidation() {
		editedValidationPrompt = validationPrompt || '';
		editedAutoValidate = autoValidate;
		isEditingValidation = true;
	}

	function cancelEditingValidation() {
		isEditingValidation = false;
		editedValidationPrompt = validationPrompt || '';
		editedAutoValidate = autoValidate;
	}

	async function saveValidation() {
		isSaving = true;
		try {
			await onSaveValidation(editedValidationPrompt, editedAutoValidate);
			isEditingValidation = false;
		} finally {
			isSaving = false;
		}
	}

	function startEditingRepositories() {
		// Convert current repository IDs to ProviderRepository format
		const converted: ProviderRepository[] = [];
		for (const id of repositoryIds) {
			const repo = repositories.get(id);
			if (!repo) continue;
			converted.push({
				provider: 'github' as const,
				fullName: repo.providerId,
				name: repo.providerId.split('/')[1] || repo.providerId,
				owner: repo.providerId.split('/')[0] || '',
				url: `https://github.com/${repo.providerId}`,
				description: ''
			});
		}
		editedRepositories = converted;
		isEditingRepositories = true;
	}

	function cancelEditingRepositories() {
		isEditingRepositories = false;
		editedRepositories = [];
	}

	async function saveRepositories() {
		isSavingRepositories = true;
		try {
			// Convert ProviderRepository back to repository IDs (provider_id format)
			const repoIds = editedRepositories.map(r => r.fullName);
			await onSaveRepositories(repoIds);
			isEditingRepositories = false;
		} finally {
			isSavingRepositories = false;
		}
	}

	$effect(() => {
		editedValidationPrompt = validationPrompt || '';
		editedAutoValidate = autoValidate;
	});
</script>

<div bind:this={containerRef} class="flex-1 flex flex-col overflow-hidden bg-background">
	<!-- Repository Editor (when active) -->
	{#if isEditingRepositories}
		<div class="flex-shrink-0 border-b border-border/20 bg-card p-4">
			<div class="flex items-center justify-between mb-3">
				<div class="flex items-center gap-2">
					<FolderGit class="w-4 h-4 text-primary" />
					<h3 class="text-sm font-semibold text-foreground">Edit Repositories</h3>
				</div>
				<div class="flex items-center gap-2">
					<button
						onclick={cancelEditingRepositories}
						disabled={isSavingRepositories}
						class="px-3 py-1.5 text-xs font-medium rounded-md text-muted-foreground hover:text-foreground border border-border/30 hover:bg-muted/50 transition-colors disabled:opacity-50"
					>
						Cancel
					</button>
					<button
						onclick={saveRepositories}
						disabled={isSavingRepositories || editedRepositories.length === 0}
						class="px-3 py-1.5 text-xs font-medium rounded-md bg-green-600 text-white hover:bg-green-700 transition-colors disabled:opacity-50"
					>
						{isSavingRepositories ? 'Saving...' : 'Save'}
					</button>
				</div>
			</div>
			<RepositorySelector bind:selectedRepos={editedRepositories} />
		</div>
	{/if}

	<!-- Prompt Console -->
	<div class="flex-shrink-0 border-b border-border/20 overflow-hidden bg-card">
		<!-- Prompt Content - Two Column Layout -->
		<div class="flex divide-x divide-border/20" style="height: {manuallyResized ? promptHeight : computedHeight}px;">
			<!-- Revision Prompt (Left) -->
			<div class="flex-1 flex flex-col min-w-0">
				<div class="px-4 py-2 bg-muted/10 border-b border-border/10">
					<h3 class="text-xs font-semibold text-muted-foreground">Revision Prompt</h3>
				</div>
				<UiScrollArea viewportClass="overflow-auto" class="flex-1">
					<pre class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{revision.promptText}</pre>
				</UiScrollArea>
			</div>
			
			<!-- Validation Prompt (Right) -->
			{#if validationPrompt || isEditingValidation}
				<div class="flex-1 flex flex-col min-w-0">
					<div class="px-4 py-2 bg-muted/10 border-b border-border/10 flex items-center justify-between gap-3">
						<h3 class="text-xs font-semibold text-muted-foreground">Validation Prompt</h3>
						<div class="flex items-center gap-3">
							<!-- Auto-validate Toggle -->
							<div class="flex items-center gap-2">
								<span class="text-xs text-muted-foreground">Auto-validate</span>
								<Switch.Root
									bind:checked={editedAutoValidate}
									disabled={!isEditingValidation || isSaving}
									class="data-[state=checked]:bg-green-600 data-[state=unchecked]:bg-muted inline-flex h-4 w-7 shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
								>
									<Switch.Thumb
										class="pointer-events-none block h-3 w-3 rounded-full bg-white transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0"
									/>
								</Switch.Root>
							</div>
							
							<!-- Edit/Save/Cancel Buttons -->
							{#if isEditingValidation}
								<button
									onclick={cancelEditingValidation}
									disabled={isSaving}
									class="text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
									aria-label="Cancel"
								>
									<X class="w-4 h-4" />
								</button>
								<button
									onclick={saveValidation}
									disabled={isSaving}
									class="text-green-600 hover:text-green-700 transition-colors disabled:opacity-50"
									aria-label="Save validation prompt"
								>
									<Save class="w-4 h-4" />
								</button>
							{:else}
								<button
									onclick={startEditingValidation}
									class="text-blue-600 hover:text-blue-700 transition-colors"
									aria-label="Edit validation prompt"
								>
									<Edit class="w-4 h-4" />
								</button>
							{/if}
						</div>
					</div>
					<UiScrollArea viewportClass="overflow-auto" class="flex-1">
						{#if isEditingValidation}
							<textarea
								bind:value={editedValidationPrompt}
								disabled={isSaving}
								placeholder="Enter a prompt to validate each execution after it completes..."
								class="w-full h-full px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground bg-transparent border-none outline-none resize-none disabled:opacity-50"
							></textarea>
						{:else}
							<pre class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{validationPrompt}</pre>
						{/if}
					</UiScrollArea>
				</div>
			{/if}
		</div>
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
				<p class="text-sm mb-4">No executions yet for this revision</p>
				<div class="flex flex-col items-center gap-3">
					<div class="flex items-center gap-2 px-3 py-1.5 bg-muted/30 rounded-md">
						<GitMerge class="w-4 h-4 text-muted-foreground" />
						<span class="text-sm font-medium text-foreground">{repositoryIds.length} {repositoryIds.length === 1 ? 'repository' : 'repositories'}</span>
					</div>
					<div class="flex flex-col gap-2">
						<button
							onclick={onExecuteAll}
							class="inline-flex items-center gap-2 px-4 py-2 rounded-md bg-green-600 text-white hover:bg-green-700 transition-colors font-medium text-sm"
						>
							<Play class="w-4 h-4" />
							Execute on all repos
						</button>
						<button
							onclick={startEditingRepositories}
							class="inline-flex items-center gap-2 px-4 py-2 rounded-md bg-blue-600 text-white hover:bg-blue-700 transition-colors font-medium text-sm"
						>
							<FolderGit class="w-4 h-4" />
							Edit repositories
						</button>
					</div>
				</div>
			</div>
		</div>
	{:else}
		<div class="flex-1 flex flex-col overflow-hidden @container/table">
			<!-- Toolbar -->
			<div class="flex-shrink-0 flex items-center gap-3 px-4 py-3 bg-muted/5 border-b border-border/10">
				{#if selectedIds.size > 0}
					<span class="text-sm font-medium text-foreground">{selectedIds.size} selected</span>
					<div class="flex items-center gap-2">
						<button
							onclick={handleBulkRestart}
							class="px-3 py-1.5 text-xs font-medium rounded-md bg-blue-600 text-white hover:bg-blue-700 transition-colors"
						>
							Restart
						</button>
						{#if hasValidationPrompt}
							<button
								onclick={handleBulkRevalidate}
								class="px-3 py-1.5 text-xs font-medium rounded-md bg-blue-600 text-white hover:bg-blue-700 transition-colors"
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
				{:else}
					<div class="flex items-center gap-2">
						<div class="flex items-center gap-1.5 px-2 py-1 bg-muted/30 rounded-md">
							<GitMerge class="w-3.5 h-3.5 text-muted-foreground" />
							<span class="text-xs font-medium text-foreground">{repositoryIds.length}</span>
						</div>
						<button
							onclick={startEditingRepositories}
							class="inline-flex items-center gap-2 px-3 py-1.5 text-xs font-medium rounded-md bg-blue-600 text-white hover:bg-blue-700 transition-colors"
						>
							<FolderGit class="w-3.5 h-3.5" />
							Edit repositories
						</button>
					</div>
				{/if}
			</div>

			<!-- Table Container (Header + Body) -->
			<UiScrollArea class="flex-1">
				<div>
					<!-- Table Header -->
					<div
						class="sticky top-0 z-10 grid gap-3 px-4 py-2.5 bg-card border-b border-border/10 text-xs font-medium text-muted-foreground items-center
							[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]
							@max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.2fr)_minmax(0,_0.8fr)_minmax(0,_0.8fr)]
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
							Changes
							{#if sortColumn === 'commitStatus'}
								{#if sortDirection === 'asc'}
									<ChevronUp class="w-3 h-3 inline ml-1" />
								{:else}
									<ChevronDown class="w-3 h-3 inline ml-1" />
								{/if}
							{/if}
						</button>
						<div class="text-left truncate @max-md/table:hidden">CI</div>
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
			</UiScrollArea>
		</div>
	{/if}
</div>
