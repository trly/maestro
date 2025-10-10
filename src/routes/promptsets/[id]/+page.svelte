<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import DiffViewer from '$lib/components/DiffViewer.svelte';
	import PromptSetHeader from '$lib/components/ui/PromptSetHeader.svelte';
	import RevisionSidebar from '$lib/components/ui/RevisionSidebar.svelte';
	import RevisionDetail from '$lib/components/ui/RevisionDetail.svelte';
	import EditValidationDialog from '$lib/components/ui/EditValidationDialog.svelte';
	import EditRepositoriesDialog from '$lib/components/ui/EditRepositoriesDialog.svelte';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import { confirm } from '$lib/ui/confirm';
	import { pollExecutions } from '$lib/polling';
	import { toShortHash } from '$lib/utils';
	import type { PromptSet, PromptRevision, Execution, Repository as DBRepository } from '$lib/types';
	import type { Repository as ProviderRepository } from '$lib/providers/types';
	import { executionStore } from '$lib/stores/executionBus';
	import { fetchExecutionStats, type ExecutionStats } from '$lib/stores/executionStats';

	let currentPromptSet = $state<PromptSet | null>(null);
	let revisions = $state<PromptRevision[]>([]);
	let currentRevision = $state<PromptRevision | null>(null);
	let executions = $state<Execution[]>([]); // All executions for the prompt set
	let repositories = $state<Map<string, DBRepository>>(new Map());
	let liveStats = $state<Map<string, ExecutionStats>>(new Map());
	
	// Merge executions with live updates from the event bus and live stats
	let executionsWithUpdates = $derived(
		executions.map(execution => {
			const updates = $executionStore.get(execution.id);
			const stats = liveStats.get(execution.id);
			
			// Use live stats if available, otherwise fall back to DB stats
			const effectiveStats = stats || {
				filesAdded: execution.filesAdded,
				filesRemoved: execution.filesRemoved,
				filesModified: execution.filesModified,
				linesAdded: execution.linesAdded,
				linesRemoved: execution.linesRemoved,
			};
			
			return {
			...execution,
			...(updates?.sessionId && { sessionId: updates.sessionId }),
			...(updates?.threadUrl && { threadUrl: updates.threadUrl }),
			...(updates?.status && { status: updates.status }),
			...(updates?.validationStatus && { validationStatus: updates.validationStatus }),
			...(updates?.validationThreadUrl && { validationThreadUrl: updates.validationThreadUrl }),
			...(updates?.commitStatus && { commitStatus: updates.commitStatus }),
			...(updates?.commitSha && { commitSha: updates.commitSha }),
			...(updates?.committedAt && { committedAt: updates.committedAt }),
			...(updates?.progressMessage && { progressMessage: updates.progressMessage }),
			filesAdded: effectiveStats.filesAdded,
			filesRemoved: effectiveStats.filesRemoved,
			filesModified: effectiveStats.filesModified,
			linesAdded: effectiveStats.linesAdded,
			 linesRemoved: effectiveStats.linesRemoved,
		};
		})
	);

	// Compute reactive execution stats per revision (from all executions, not just current)
	let revisionStats = $derived(
		revisions.reduce((acc, revision) => {
			// Use all executions for the prompt set, not just the current revision
			const allExecutions = executionsWithUpdates.filter(e => e.revisionId === revision.id);
			const total = allExecutions.length;
			const running = allExecutions.filter(e => e.status === 'running').length;
			const completed = allExecutions.filter(e => e.status === 'completed').length;
			const validationPassed = allExecutions.filter(e => e.validationStatus === 'passed').length;
			
			acc[revision.id] = {
				total,
				running,
				completed,
				validationPassed,
			};
			return acc;
		}, {} as Record<string, { total: number; running: number; completed: number; validationPassed: number }>)
	);
	let editValidationOpen = $state(false);
	let editReposOpen = $state(false);
	let editingRepos = $state<ProviderRepository[]>([]);
	let diffViewerOpen = $state(false);
	let diffViewerExecutionId = $state<string | null>(null);
	let stopPolling = $state<(() => void) | null>(null);

	const promptsetId = $derived($page.params.id);
	const revisionParam = $derived($page.url.searchParams.get('revision'));

	async function loadPromptSet() {
		if (!promptsetId) return;

		try {
			currentPromptSet = await api.promptSets.get(promptsetId);
			revisions = await api.promptSets.getRevisions(promptsetId);
			executions = await api.promptSets.getExecutions(promptsetId);

			const newRepos = new Map(repositories);
			for (const repoId of currentPromptSet.repositoryIds) {
			if (!newRepos.has(repoId)) {
			const repo = await api.repositories.get(repoId);
			 newRepos.set(repoId, repo);
			 }
		}
		repositories = newRepos;

			await backfillMissingStats();

			if (revisionParam) {
				const revision = revisions.find(r => r.id === revisionParam || toShortHash(r.id) === revisionParam);
				if (revision) {
					await viewRevisionExecutions(revision);
				}
			} else if (revisions.length > 0) {
				// Auto-select most recent revision (first in list)
				await viewRevisionExecutions(revisions[0]);
			}
		} catch (err) {
			showToast('Failed to load prompt set: ' + err, 'error');
			goto('/');
		}
	}

	async function viewRevisionExecutions(revision: PromptRevision) {
		currentRevision = revision;
		
		// Fetch latest executions for this revision to refresh data
		const revisionExecutions = await api.revisions.getExecutions(revision.id);
		
		// Update the main executions array: replace matching executions, add new ones
		const updatedExecutions = executions.filter(e => e.revisionId !== revision.id)
			.concat(revisionExecutions);
		executions = updatedExecutions;
		
		await backfillMissingStats();
		
		// Load live stats for executions with no stats in DB
		await loadLiveStats(revisionExecutions);

		if (revisionExecutions.some(e => 
			e.status === 'pending' || 
			e.status === 'running' || 
			e.validationStatus === 'pending' || 
			e.validationStatus === 'running'
		)) {
			startPolling(revision.id);
		}
	}

	async function loadLiveStats(executionsToLoad: Execution[]) {
		// Load stats for all completed executions (stats are always calculated on-demand)
		const needsStats = executionsToLoad.filter(e => e.status === 'completed');

		if (needsStats.length === 0) return;

		const newStats = new Map(liveStats);
		await Promise.all(
			needsStats.map(async (execution) => {
				try {
					const stats = await fetchExecutionStats(execution.id);
					newStats.set(execution.id, stats);
				} catch (err) {
					console.error(`Failed to load stats for ${execution.id}:`, err);
				}
			})
		);
		liveStats = newStats;
	}

	function startPolling(revisionId: string) {
		if (stopPolling) stopPolling();
		
		stopPolling = pollExecutions(revisionId, (execs: Execution[]) => {
			// Merge polled executions into main array, don't replace it
			const updatedExecutions = executions.filter(e => e.revisionId !== revisionId)
				.concat(execs);
			executions = updatedExecutions;
			
			const hasActiveExecution = execs.some(e => 
				e.status === 'pending' || 
				e.status === 'running' || 
				e.validationStatus === 'pending' || 
				e.validationStatus === 'running'
			);
			
			if (!hasActiveExecution && stopPolling) {
				stopPolling();
				stopPolling = null;
			}
		});
	}

	async function backfillMissingStats() {
		const missingStats = executions.filter(e => 
			e.status === 'completed' && 
			e.filesAdded === 0 && 
			e.filesRemoved === 0 && 
			e.filesModified === 0 && 
			e.linesAdded === 0 && 
			e.linesRemoved === 0
		);

		if (missingStats.length === 0) return;
		
		for (const execution of missingStats) {
			try {
				const updated = await api.executions.backfillStats(execution.id);
				const index = executions.findIndex(e => e.id === execution.id);
				if (index !== -1) {
					executions[index] = updated;
				}
			} catch (err) {
			}
		}
	}

	async function executeRevision(revision: PromptRevision) {
		if (!currentPromptSet) return;

		currentRevision = revision;
		
		// Optimistic update - create pending executions for each repo
		const newExecutions = currentPromptSet.repositoryIds.map(repoId => ({
		id: crypto.randomUUID(), // Temporary ID
		promptsetId: currentPromptSet!.id,
		revisionId: revision.id,
		repositoryId: repoId,
		status: 'pending' as const,
		promptStatus: null,
		promptResult: null,
		validationStatus: null,
		validationThreadUrl: null,
		validationResult: null,
		filesAdded: 0,
		filesRemoved: 0,
		filesModified: 0,
		 linesAdded: 0,
		linesRemoved: 0,
		commitStatus: 'none' as const,
		sessionId: null,
		threadUrl: null,
		commitSha: null,
		committedAt: null,
		parentSha: null,
		branch: null,
		createdAt: Date.now(),
		completedAt: null,
	}));
		executions = [...executions, ...newExecutions];
		
		try {
			await api.revisions.execute(revision.id);
			// Reload to get real execution IDs from backend
			executions = await api.promptSets.getExecutions(currentPromptSet.id);
			startPolling(revision.id);
			showToast('Execution started', 'info');
		} catch (err) {
			// Revert optimistic update on error
			executions = executions.filter(e => !newExecutions.some(ne => ne.id === e.id));
			showToast('Failed to execute revision: ' + err, 'error');
		}
	}

	async function getRepoName(repoId: string): Promise<string> {
		if (repositories.has(repoId)) {
			return repositories.get(repoId)!.providerId;
		}
		try {
			const repo = await api.repositories.get(repoId);
			repositories.set(repoId, repo);
			return repo.providerId;
		} catch {
			return repoId;
		}
	}

	async function deleteExecutionWithConfirm(execution: Execution, repoName: string) {
		const confirmed = await confirm({
			title: `Delete execution for ${repoName}?`,
			message: `This will:\n- Delete the database record\n- Delete the git branch\n\nThis action cannot be undone.`,
			confirmText: 'Delete',
			cancelText: 'Cancel'
		});
		
		if (!confirmed) return;
		
		try {
			await api.executions.delete(execution.id);
			executions = executions.filter(e => e.id !== execution.id);
			showToast('Execution deleted successfully', 'success');
		} catch (err) {
			showToast('Failed to delete execution: ' + err, 'error');
		}
	}

	async function stopExecutionManually(execution: Execution) {
		try {
			// Optimistic update
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, status: 'cancelled' as const }
					: e
			);
			
			await api.executions.stop(execution.id);
			showToast('Execution stopped', 'success');
			
			const revisionId = currentRevision?.id || execution.revisionId;
			if (revisionId) {
				startPolling(revisionId);
			}
		} catch (err) {
			// Revert optimistic update on error (restore running state)
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, status: 'running' as const }
					: e
			);
			showToast('Failed to stop execution: ' + err, 'error');
		}
	}

	async function stopValidationManually(execution: Execution) {
		try {
			// Optimistic update
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, validationStatus: 'cancelled' as const }
					: e
			);
			
			await api.executions.stopValidation(execution.id);
			showToast('Validation stopped', 'success');
			
			const revisionId = currentRevision?.id || execution.revisionId;
			if (revisionId) {
				startPolling(revisionId);
			}
		} catch (err) {
			// Revert optimistic update on error (restore running state)
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, validationStatus: 'running' as const }
					: e
			);
			showToast('Failed to stop validation: ' + err, 'error');
		}
	}

	async function stopAllExecutions() {
		if (!currentRevision) return;
		
		try {
			const result = await api.revisions.stopAll(currentRevision.id);
			showToast(result.message, 'success');
			startPolling(currentRevision.id);
		} catch (err) {
			showToast('Failed to stop executions: ' + err, 'error');
		}
	}

	async function stopAllValidations() {
		if (!currentRevision) return;
		
		try {
			const result = await api.revisions.stopAllValidations(currentRevision.id);
			showToast(result.message, 'success');
			startPolling(currentRevision.id);
		} catch (err) {
			showToast('Failed to stop validations: ' + err, 'error');
		}
	}

	async function validateExecutionManually(execution: Execution) {
		try {
			// Optimistic update
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, validationStatus: 'running' as const }
					: e
			);
			
			await api.executions.validate(execution.id);
			const revisionId = currentRevision?.id || execution.revisionId;
			if (revisionId) {
				startPolling(revisionId);
			}
			showToast('Validation started', 'info');
		} catch (err) {
			// Revert optimistic update on error
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, validationStatus: null }
					: e
			);
			showToast('Failed to start validation: ' + err, 'error');
		}
	}

	async function resumeExecutionManually(execution: Execution) {
		try {
			// Optimistic update
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, status: 'running' as const }
					: e
			);
			
			await api.executions.resume(execution.id);
			const revisionId = currentRevision?.id || execution.revisionId;
			if (revisionId) {
				startPolling(revisionId);
			}
			showToast('Execution resumed', 'info');
		} catch (err) {
			// Revert optimistic update on error (restore cancelled state)
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, status: 'cancelled' as const }
					: e
			);
			showToast('Failed to resume execution: ' + err, 'error');
		}
	}

	async function bulkDeleteExecutions(selectedExecutions: Execution[]) {
		const confirmed = await confirm({
			title: `Delete ${selectedExecutions.length} execution${selectedExecutions.length > 1 ? 's' : ''}?`,
			message: `This will:\n- Delete ${selectedExecutions.length} database record${selectedExecutions.length > 1 ? 's' : ''}\n- Delete the associated git branches\n\nThis action cannot be undone.`,
			confirmText: 'Delete',
			cancelText: 'Cancel'
		});
		
		if (!confirmed) return;
		
		let successCount = 0;
		let failCount = 0;
		
		for (const execution of selectedExecutions) {
			try {
				await api.executions.delete(execution.id);
				executions = executions.filter(e => e.id !== execution.id);
				successCount++;
			} catch (err) {
				failCount++;
			}
		}
		
		if (successCount > 0) {
			showToast(`${successCount} execution${successCount > 1 ? 's' : ''} deleted successfully`, 'success');
		}
		if (failCount > 0) {
			showToast(`Failed to delete ${failCount} execution${failCount > 1 ? 's' : ''}`, 'error');
		}
	}

	async function bulkRestartExecutions(selectedExecutions: Execution[]) {
		let successCount = 0;
		let failCount = 0;
		
		for (const execution of selectedExecutions) {
			try {
				executions = executions.map(e => 
					e.id === execution.id 
						? { ...e, status: 'running' as const }
						: e
				);
				await api.executions.resume(execution.id);
				successCount++;
			} catch (err) {
				executions = executions.map(e => 
					e.id === execution.id 
						? { ...e, status: execution.status }
						: e
				);
				failCount++;
			}
		}
		
		if (currentRevision) {
			startPolling(currentRevision.id);
		}
		
		if (successCount > 0) {
			showToast(`${successCount} execution${successCount > 1 ? 's' : ''} restarted`, 'info');
		}
		if (failCount > 0) {
			showToast(`Failed to restart ${failCount} execution${failCount > 1 ? 's' : ''}`, 'error');
		}
	}

	async function bulkRevalidateExecutions(selectedExecutions: Execution[]) {
		let successCount = 0;
		let failCount = 0;
		
		for (const execution of selectedExecutions) {
			try {
				executions = executions.map(e => 
					e.id === execution.id 
						? { ...e, validationStatus: 'running' as const }
						: e
				);
				await api.executions.validate(execution.id);
				successCount++;
			} catch (err) {
				executions = executions.map(e => 
					e.id === execution.id 
						? { ...e, validationStatus: execution.validationStatus }
						: e
				);
				failCount++;
			}
		}
		
		if (currentRevision) {
			startPolling(currentRevision.id);
		}
		
		if (successCount > 0) {
			showToast(`${successCount} validation${successCount > 1 ? 's' : ''} started`, 'info');
		}
		if (failCount > 0) {
			showToast(`Failed to start ${failCount} validation${failCount > 1 ? 's' : ''}`, 'error');
		}
	}



	async function deleteRevisionWithConfirm(revision: PromptRevision) {
		const executionCount = await api.revisions.getExecutions(revision.id).then(e => e.length);
		
		const confirmed = await confirm({
			title: `Delete revision ${toShortHash(revision.id)}?`,
			message: 
				`This will delete:\n` +
				`- The revision\n` +
				`- ${executionCount} execution(s)\n` +
				`- All associated git branches\n\n` +
				`This action cannot be undone.`,
			confirmText: 'Delete',
			cancelText: 'Cancel'
		});
		
		if (!confirmed) return;
		
		try {
			await api.revisions.delete(revision.id);
			if (currentRevision?.id === revision.id) {
				currentRevision = null;
				executions = [];
			}
			await loadPromptSet();
			showToast('Revision deleted successfully', 'success');
		} catch (err) {
			showToast('Failed to delete revision: ' + err, 'error');
		}
	}

	async function saveValidationPrompt(prompt: string, autoValidate: boolean) {
		if (!currentPromptSet) return;
		
		await Promise.all([
			api.promptSets.update(currentPromptSet.id, {
				validationPrompt: prompt || null
			}),
			api.promptSets.updateAutoValidate(currentPromptSet.id, autoValidate)
		]);
		currentPromptSet.validationPrompt = prompt;
		currentPromptSet.autoValidate = autoValidate;
		showToast('Validation settings updated', 'success');
	}

	async function loadEditingRepos() {
		if (!currentPromptSet) return [];
		
		return await Promise.all(
			currentPromptSet.repositoryIds.map(async (id) => {
				const repo = await api.repositories.get(id);
				return {
					provider: repo.provider,
					fullName: repo.providerId,
					name: repo.name || '',
					owner: repo.providerId.split('/')[0] || '',
					url: `https://${repo.provider === 'github' ? 'github.com' : 'gitlab.com'}/${repo.providerId}`
				};
			})
		);
	}

	async function saveRepositories(repos: ProviderRepository[]) {
		if (!currentPromptSet || repos.length === 0) return;

		const repoPromises = repos.map(async (repo) => {
			try {
				const dbRepo = await api.repositories.find(repo.provider, repo.fullName)
					.catch(async (findErr) => {
						return await api.repositories.create(repo.provider, repo.fullName, repo.name || undefined);
					});
				repositories.set(dbRepo.id, dbRepo);
				return dbRepo.id;
			} catch (err) {
				return null;
			}
		});

		const repoIds = (await Promise.all(repoPromises)).filter(Boolean) as string[];

		if (repoIds.length === 0) {
			showToast('Failed to persist selected repositories', 'error');
			return;
		}

		await api.promptSets.update(currentPromptSet.id, {
			repositoryIds: repoIds
		});
		currentPromptSet.repositoryIds = repoIds;
		showToast('Repositories updated', 'success');
		await loadPromptSet();
	}

	let unlistenCommit: (() => void) | null = null;

	onMount(async () => {
		loadPromptSet();

		const { listen } = await import('@tauri-apps/api/event');
		unlistenCommit = await listen<{ executionId: string; commitStatus: string; commitSha?: string; committedAt?: number | null }>('execution:commit', (event) => {
			const idx = executions.findIndex(e => e.id === event.payload.executionId);
			if (idx !== -1) {
				executions[idx] = {
					...executions[idx],
					commitStatus: event.payload.commitStatus as 'none' | 'uncommitted' | 'committed',
					commitSha: event.payload.commitSha || executions[idx].commitSha,
					committedAt: event.payload.committedAt ?? executions[idx].committedAt
				};
			}
		});
	});

	// Watch for execution status changes and refetch stats when executions complete
	$effect(() => {
		const executionUpdates = $executionStore;
		
		executionUpdates.forEach((updates, executionId) => {
			// When execution completes, fetch fresh stats
			if (updates.status === 'completed' && !liveStats.has(executionId)) {
				fetchExecutionStats(executionId).then(stats => {
					liveStats = new Map(liveStats).set(executionId, stats);
				}).catch(err => {
					console.error(`Failed to fetch stats for ${executionId}:`, err);
				});
			}
		});
	});

	onDestroy(() => {
		if (stopPolling) {
			stopPolling();
			stopPolling = null;
		}
		if (unlistenCommit) {
			unlistenCommit();
		}
	});
</script>

{#if currentPromptSet}
	<!-- Desktop App Layout -->
	<div class="flex flex-col h-full overflow-hidden">
		<!-- Top Header -->
		<div class="flex-shrink-0 border-b border-border/40">
			<PromptSetHeader
				promptSet={currentPromptSet}
				{repositories}
				onEditRepos={async () => {
					editingRepos = await loadEditingRepos();
					editReposOpen = true;
				}}
				onEditValidation={() => editValidationOpen = true}
			/>
		</div>

		<!-- Main Content: Sidebar + Detail -->
		<div class="flex-1 flex min-h-0">
			<!-- Left Sidebar: Revisions List -->
			<RevisionSidebar
				{revisions}
				{currentRevision}
				{revisionStats}
				hasValidationPrompt={!!currentPromptSet.validationPrompt}
				onSelect={(revision) => {
				if (revision) {
				viewRevisionExecutions(revision);
				} else {
				currentRevision = null;
				}
				}}
				onCreate={() => goto(`/create?promptset=${promptsetId}`)}
			/>

			<!-- Main Content Area: Selected Revision Detail -->
			{#if currentRevision}
				<RevisionDetail
					revision={currentRevision}
					executions={executionsWithUpdates.filter(e => e.revisionId === currentRevision!.id)}
					{repositories}
					hasValidationPrompt={!!currentPromptSet.validationPrompt}
					onExecuteAll={() => executeRevision(currentRevision!)}
					onStopAll={stopAllExecutions}
					onStopAllValidations={stopAllValidations}
					onDelete={() => deleteRevisionWithConfirm(currentRevision!)}
					onDeleteExecution={deleteExecutionWithConfirm}
					onValidateExecution={validateExecutionManually}
					onStopExecution={stopExecutionManually}
					onStopValidation={stopValidationManually}
					onResumeExecution={resumeExecutionManually}
					onReviewChanges={(executionId) => {
						diffViewerExecutionId = executionId;
						diffViewerOpen = true;
					}}
					onBulkDelete={bulkDeleteExecutions}
					onBulkRestart={bulkRestartExecutions}
					onBulkRevalidate={bulkRevalidateExecutions}
				/>
			{:else}
				<div class="flex-1 flex items-center justify-center">
					<div class="text-center text-muted-foreground max-w-md">
						<h3 class="text-lg font-semibold mb-2">Select a revision</h3>
						<p class="text-sm">Choose a revision from the sidebar to view its executions and prompt details.</p>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Dialogs -->
	<EditValidationDialog
		bind:open={editValidationOpen}
		validationPrompt={currentPromptSet.validationPrompt || ''}
		autoValidate={currentPromptSet.autoValidate}
		onSave={saveValidationPrompt}
	/>

	<EditRepositoriesDialog
		bind:open={editReposOpen}
		currentRepos={editingRepos}
		onSave={saveRepositories}
	/>
{/if}

{#if diffViewerExecutionId}
	<DiffViewer executionId={diffViewerExecutionId} bind:open={diffViewerOpen} />
{/if}
