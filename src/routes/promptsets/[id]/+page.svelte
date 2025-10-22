<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import DiffViewer from '$lib/components/DiffViewer.svelte';
	import RevisionHeader from '$lib/components/ui/RevisionHeader.svelte';
	import PromptConsole from '$lib/components/ui/PromptConsole.svelte';
	import ExecutionTable from '$lib/components/executions/ExecutionTable.svelte';
	import AnalysisList from '$lib/components/ui/AnalysisList.svelte';
	import { Tabs } from 'bits-ui';
	import EditRepositoriesDialog from '$lib/components/ui/EditRepositoriesDialog.svelte';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import { confirm } from '$lib/ui/confirm';
	import { pollExecutions } from '$lib/polling';
	import { toShortHash } from '$lib/utils';
	import { sidebarStore } from '$lib/stores/sidebarStore';
	import type { PromptSet, PromptRevision, Execution, Repository as DBRepository, Analysis } from '$lib/types';
	import type { Repository as ProviderRepository } from '$lib/providers/types';
	import { executionStore } from '$lib/stores/executionBus';
	import { fetchExecutionStats, type ExecutionStats } from '$lib/stores/executionStats';
	import * as ipc from '$lib/ipc';

	let currentPromptSet = $state<PromptSet | null>(null);
	let revisions = $state<PromptRevision[]>([]);
	let currentRevision = $state<PromptRevision | null>(null);
	let executions = $state<Execution[]>([]); // All executions for the prompt set
	let repositories = $state<Map<string, DBRepository>>(new Map());
	let liveStats = $state<Map<string, ExecutionStats>>(new Map());
	let analyses = $state<Analysis[]>([]); // Analyses for current revision
	
	// Loading states for async operations
	let pushingExecutions = $state<Set<string>>(new Set());
	let refreshingCi = $state<Set<string>>(new Set());
	let loadingStats = $state<Set<string>>(new Set());
	let analyzingExecutions = $state(false);
	let analyzingValidations = $state(false);
	
	// Bulk operation loading states
	let bulkStarting = $state(false);
	let bulkRestarting = $state(false);
	let bulkValidating = $state(false);
	let bulkRevalidating = $state(false);
	let bulkDeleting = $state(false);
	
	// Merge executions with live updates from the event bus and live stats
	let executionsWithUpdates = $derived.by(() => {
		const updates = $executionStore;
		return executions.map(execution => {
			const executionUpdates = updates.get(execution.id);
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
			...(executionUpdates?.sessionId && { sessionId: executionUpdates.sessionId }),
			...(executionUpdates?.threadUrl && { threadUrl: executionUpdates.threadUrl }),
			...(executionUpdates?.status && { status: executionUpdates.status }),
			...(executionUpdates?.validationStatus && { validationStatus: executionUpdates.validationStatus }),
			...(executionUpdates?.validationThreadUrl && { validationThreadUrl: executionUpdates.validationThreadUrl }),
			...(executionUpdates?.commitStatus && { commitStatus: executionUpdates.commitStatus }),
			...(executionUpdates?.commitSha && { commitSha: executionUpdates.commitSha }),
			...(executionUpdates?.committedAt && { committedAt: executionUpdates.committedAt }),
			...(executionUpdates?.ciStatus && { ciStatus: executionUpdates.ciStatus }),
			...(executionUpdates?.ciUrl && { ciUrl: executionUpdates.ciUrl }),
			...(executionUpdates?.progressMessage && { progressMessage: executionUpdates.progressMessage }),
			filesAdded: effectiveStats.filesAdded,
			filesRemoved: effectiveStats.filesRemoved,
			filesModified: effectiveStats.filesModified,
			linesAdded: effectiveStats.linesAdded,
			linesRemoved: effectiveStats.linesRemoved,
		};
		})
	});

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
	
	// Compute revision header props reactively
	let revisionHeaderProps = $derived.by(() => {
		if (!currentRevision || !currentPromptSet) return null;
		
		const stats = revisionStats[currentRevision.id];
		
		return {
			revision: currentRevision,
			stats,
			analyses,
			repositoryCount: currentPromptSet.repositoryIds.length,
			onDelete: () => deleteRevisionWithConfirm(currentRevision!),
			onEditRepositories: () => openEditRepositories()
		};
	});
	
	let editReposOpen = $state(false);
	let editingRepos = $state<ProviderRepository[]>([]);
	let diffViewerOpen = $state(false);
	let diffViewerExecutionId = $state<string | null>(null);
	let stopPolling = $state<(() => void) | null>(null);
	let activeTab = $state<string>('executions');

	const promptsetId = $derived($page.params.id);
	const revisionParam = $derived($page.url.searchParams.get('revision'));
	
	// Reload prompt set when promptsetId changes or when navigating with a new revision
	$effect(() => {
		// Access both to create dependencies
		const id = promptsetId;
		const rev = revisionParam;
		
		if (id) {
			loadPromptSet();
		}
	});

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

			// After loading revisions, check if we need to load a specific revision
			if (revisionParam) {
				const revision = revisions.find(r => r.id === revisionParam || toShortHash(r.id) === revisionParam);
				if (revision) {
					await viewRevisionExecutions(revision);
				}
			} else if (revisions.length > 0 && !currentRevision) {
				// Auto-select the first (most recent) revision if none is selected
				await viewRevisionExecutions(revisions[0]);
			}
		} catch (err) {
			showToast('Failed to load prompt set: ' + err, 'error');
			goto('/');
		}
	}
	
	// Watch for revision parameter changes (when switching between already-loaded revisions)
	$effect(() => {
		// Access dependencies explicitly
		const param = revisionParam;
		const revs = revisions;
		
		if (!param || revs.length === 0) return;
		
		const revision = revs.find(r => r.id === param || toShortHash(r.id) === param);
		if (revision && (!currentRevision || revision.id !== currentRevision.id)) {
			viewRevisionExecutions(revision);
		}
	});

	async function viewRevisionExecutions(revision: PromptRevision) {
		currentRevision = revision;
		
		// Fetch latest executions and analyses for this revision to refresh data
		const [revisionExecutions, revisionAnalyses] = await Promise.all([
			api.revisions.getExecutions(revision.id),
			ipc.getAnalysesByRevision(revision.id)
		]);
		
		// Update the main executions array: replace matching executions, add new ones
		const updatedExecutions = executions.filter(e => e.revisionId !== revision.id)
			.concat(revisionExecutions);
		executions = updatedExecutions;
		analyses = revisionAnalyses;
		
		await backfillMissingStats();
		
		// Stats will be loaded lazily when each row becomes visible

		if (revisionExecutions.some(e => 
			e.status === 'pending' || 
			e.status === 'running' || 
			e.validationStatus === 'pending' || 
			e.validationStatus === 'running'
		)) {
			startPolling(revision.id);
		}
	}

	async function loadSingleExecutionStats(executionId: string, status: string) {
		// Only load stats for completed executions
		if (status !== 'completed') return;
		
		// Skip if already loaded or currently loading
		if (liveStats.has(executionId) || loadingStats.has(executionId)) return;

		// Mark as loading
		loadingStats = new Set(loadingStats).add(executionId);

		try {
			const stats = await fetchExecutionStats(executionId);
			liveStats = new Map(liveStats).set(executionId, stats);
		} catch (err) {
			console.error(`Failed to load stats for ${executionId}:`, err);
		} finally {
			// Remove from loading set
			const updatedLoading = new Set(loadingStats);
			updatedLoading.delete(executionId);
			loadingStats = updatedLoading;
		}
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
		
		// Check if executions already exist for this revision
		const existingExecutions = executions.filter(e => e.revisionId === revision.id);
		const existingRepoIds = new Set(existingExecutions.map(e => e.repositoryId));
		
		// Separate existing executions by state
		// Start: pending with no session_id (never executed, but worktree exists from prepare)
		const toStart = existingExecutions.filter(e => 
			e.status === 'pending' && !e.sessionId
		);
		// Resume: cancelled/failed, or pending with session_id (was started before)
		const toResume = existingExecutions.filter(e => 
			e.status === 'cancelled' || e.status === 'failed' || 
			(e.status === 'pending' && e.sessionId)
		);
		
		// Skip repos with running or completed executions
		const reposWithActiveExecution = new Set(
			existingExecutions
				.filter(e => e.status === 'running' || e.status === 'completed')
				.map(e => e.repositoryId)
		);
		
		// Only create new executions for repos that don't have any execution yet
		const reposNeedingNewExecution = currentPromptSet.repositoryIds.filter(
			repoId => !existingRepoIds.has(repoId)
		);
		
		try {
			let startedCount = 0;
			let resumedCount = 0;
			
			// Start existing pending executions (no worktree yet)
			if (toStart.length > 0) {
				const results = await Promise.allSettled(
					toStart.map(execution => api.executions.start(execution.id))
				);
				startedCount = results.filter(r => r.status === 'fulfilled').length;
			}
			
			// Resume existing failed/cancelled executions (worktree exists)
			if (toResume.length > 0) {
				const results = await Promise.allSettled(
					toResume.map(execution => api.executions.resume(execution.id))
				);
				resumedCount = results.filter(r => r.status === 'fulfilled').length;
			}
			
			// Create new executions only for repos without any execution
			if (reposNeedingNewExecution.length > 0) {
				await api.revisions.execute(revision.id, reposNeedingNewExecution);
			}
			
			// Reload to get updated state
			executions = await api.promptSets.getExecutions(currentPromptSet.id);
			sidebarStore.refresh(); // Trigger sidebar to reload
			startPolling(revision.id);
			
			// Show appropriate message
			const totalStarted = startedCount + reposNeedingNewExecution.length;
			const skippedCount = reposWithActiveExecution.size;
			
			const parts = [];
			if (totalStarted > 0) parts.push(`Started ${totalStarted}`);
			if (resumedCount > 0) parts.push(`resumed ${resumedCount}`);
			if (skippedCount > 0) parts.push(`skipped ${skippedCount} active`);
			
			if (parts.length > 0) {
				showToast(parts.join(', '), 'info');
			} else {
				showToast('All executions already running or completed', 'info');
			}
		} catch (err) {
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
			// Remove from state arrays
			executions = executions.filter(e => e.id !== execution.id);
			// Clear from event bus and stats cache
			executionStore.update(map => {
				map.delete(execution.id);
				return new Map(map);
			});
			liveStats.delete(execution.id);
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

	async function startExecutionManually(execution: Execution) {
		try {
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, status: 'running' as const }
					: e
			);
			
			await api.executions.start(execution.id);
			const revisionId = currentRevision?.id || execution.revisionId;
			if (revisionId) {
				startPolling(revisionId);
			}
			showToast('Execution started', 'info');
		} catch (err) {
			executions = executions.map(e => 
				e.id === execution.id 
					? { ...e, status: execution.status }
					: e
			);
			showToast('Failed to start execution: ' + err, 'error');
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
			sidebarStore.refresh(); // Trigger sidebar to reload
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

	async function pushExecutionManually(execution: Execution) {
		try {
			pushingExecutions.add(execution.id);
			pushingExecutions = new Set(pushingExecutions); // Trigger reactivity
			
			showToast('Pushing commit to remote...', 'info');
			await api.executions.push(execution.id, false); // false = not force push
			showToast('Push completed successfully', 'success');
			// CI status will update via event bus after push
		} catch (err) {
			showToast('Failed to push: ' + err, 'error');
		} finally {
			pushingExecutions.delete(execution.id);
			pushingExecutions = new Set(pushingExecutions); // Trigger reactivity
		}
	}

	async function refreshCiManually(execution: Execution) {
		try {
			refreshingCi.add(execution.id);
			refreshingCi = new Set(refreshingCi); // Trigger reactivity
			
			await api.ci.refreshStatus(execution.id);
			// CI status will update via event bus
			showToast('CI status refreshed', 'success');
		} catch (err) {
			showToast('Failed to refresh CI: ' + err, 'error');
		} finally {
			refreshingCi.delete(execution.id);
			refreshingCi = new Set(refreshingCi); // Trigger reactivity
		}
	}

	async function refreshAllCiManually() {
		if (!currentRevision) return;
		
		const revisionExecutions = executionsWithUpdates.filter(
			e => e.revisionId === currentRevision?.id && e.commitStatus === 'committed'
		);
		
		if (revisionExecutions.length === 0) {
			showToast('No committed executions to refresh', 'info');
			return;
		}
		
		try {
			showToast(`Refreshing CI status for ${revisionExecutions.length} execution(s)...`, 'info');
			await Promise.all(
				revisionExecutions.map(execution => api.ci.refreshStatus(execution.id))
			);
			showToast('All CI statuses refreshed', 'success');
		} catch (err) {
			showToast('Failed to refresh CI: ' + err, 'error');
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
		
		try {
			bulkDeleting = true;
			let successCount = 0;
			let failCount = 0;
			const deletedIds: string[] = [];
			
			for (const execution of selectedExecutions) {
				try {
					await api.executions.delete(execution.id);
					executions = executions.filter(e => e.id !== execution.id);
					deletedIds.push(execution.id);
					successCount++;
				} catch (err) {
					failCount++;
				}
			}
			
			// Clean up event bus and stats for deleted executions
			if (deletedIds.length > 0) {
				executionStore.update(map => {
					deletedIds.forEach(id => {
						map.delete(id);
						liveStats.delete(id);
					});
					return new Map(map);
				});
			}
			
			if (successCount > 0) {
				showToast(`${successCount} execution${successCount > 1 ? 's' : ''} deleted successfully`, 'success');
			}
			if (failCount > 0) {
				showToast(`Failed to delete ${failCount} execution${failCount > 1 ? 's' : ''}`, 'error');
			}
		} finally {
			bulkDeleting = false;
		}
	}

	async function bulkStartExecutions(selectedExecutions: Execution[]) {
		try {
			bulkStarting = true;
			let successCount = 0;
			let failCount = 0;
			
			for (const execution of selectedExecutions) {
				try {
					executions = executions.map(e => 
						e.id === execution.id 
							? { ...e, status: 'running' as const }
							: e
					);
					await api.executions.start(execution.id);
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
				showToast(`${successCount} execution${successCount > 1 ? 's' : ''} started`, 'info');
			}
			if (failCount > 0) {
				showToast(`Failed to start ${failCount} execution${failCount > 1 ? 's' : ''}`, 'error');
			}
		} finally {
			bulkStarting = false;
		}
	}

	async function bulkRestartExecutions(selectedExecutions: Execution[]) {
		try {
			bulkRestarting = true;
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
		} finally {
			bulkRestarting = false;
		}
	}

	async function bulkStartValidations(selectedExecutions: Execution[]) {
		try {
			bulkValidating = true;
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
		} finally {
			bulkValidating = false;
		}
	}

	async function bulkRevalidateExecutions(selectedExecutions: Execution[]) {
		try {
			bulkRevalidating = true;
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
				showToast(`${successCount} validation${successCount > 1 ? 's' : ''} revalidated`, 'info');
			}
			if (failCount > 0) {
				showToast(`Failed to revalidate ${failCount} validation${failCount > 1 ? 's' : ''}`, 'error');
			}
		} finally {
			bulkRevalidating = false;
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
			sidebarStore.refresh(); // Trigger sidebar to reload
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

	async function saveRepositoriesByIds(repoProviderIds: string[]) {
		if (!currentPromptSet || !currentRevision || repoProviderIds.length === 0) return;

		// Convert provider IDs (owner/repo) to database IDs, creating repos if needed
		const repoPromises = repoProviderIds.map(async (providerId) => {
			try {
				const dbRepo = await api.repositories.find('github', providerId)
					.catch(async () => {
						return await api.repositories.create('github', providerId);
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

		// Update prompt set repositories
		await api.promptSets.update(currentPromptSet.id, {
			repositoryIds: repoIds
		});
		currentPromptSet.repositoryIds = repoIds;
		
		// Find new repositories (not in existing executions)
		const existingRepoIds = new Set(
			executions
				.filter(e => e.revisionId === currentRevision!.id)
				.map(e => e.repositoryId)
		);
		const newRepoIds = repoIds.filter(id => !existingRepoIds.has(id));
		
		// Create pending executions for new repositories (without starting them)
		if (newRepoIds.length > 0) {
			const newExecutions = await Promise.all(
				newRepoIds.map(repoId => 
					api.executions.create(currentPromptSet!.id, currentRevision!.id, repoId)
				)
			);
			
			// Add new executions to state
			executions = [...executions, ...newExecutions];
			
			showToast(`Repositories updated, ${newRepoIds.length} new execution${newRepoIds.length > 1 ? 's' : ''} created`, 'success');
		} else {
			showToast('Repositories updated', 'success');
		}
		
		await loadPromptSet();
	}

	async function loadEditingRepos() {
		if (!currentPromptSet) return [];
		
		const repos = await Promise.all(
			currentPromptSet.repositoryIds.map(async (id) => {
				const repo = await api.repositories.get(id);
				if (!repo?.providerId) return null;
				return {
					provider: repo.provider,
					fullName: repo.providerId,
					name: repo.name || '',
					owner: repo.providerId.split('/')[0] || '',
					url: `https://${repo.provider === 'github' ? 'github.com' : 'gitlab.com'}/${repo.providerId}`
				};
			})
		);
		return repos.filter((r): r is NonNullable<typeof r> => r !== null);
	}

	async function openEditRepositories() {
		editingRepos = await loadEditingRepos();
		editReposOpen = true;
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

	async function handleAnalyzeExecutions(revision: PromptRevision) {
		const failedExecutions = executionsWithUpdates.filter(
			e => e.revisionId === revision.id && e.status === 'failed'
		);
		
		if (failedExecutions.length === 0) {
			showToast('No failed executions to analyze', 'info');
			return;
		}
		
		try {
			analyzingExecutions = true;
			showToast(`Analyzing ${failedExecutions.length} failed execution${failedExecutions.length > 1 ? 's' : ''}...`, 'info');
			
			const analysisId = await ipc.createAnalysis(
				revision.id,
				'execution',
				failedExecutions.map(e => e.id)
			);
			await ipc.runAnalysis(analysisId);
			
			// Refresh analyses to include the new one
			analyses = await ipc.getAnalysesByRevision(revision.id);
			showToast('Analysis started', 'success');
		} catch (err) {
			showToast('Failed to start analysis', 'error');
			console.error('Analysis error:', err);
		} finally {
			analyzingExecutions = false;
		}
	}

	async function handleAnalyzeValidations(revision: PromptRevision) {
		const failedValidations = executionsWithUpdates.filter(
			e => e.revisionId === revision.id && e.validationStatus === 'failed'
		);
		
		if (failedValidations.length === 0) {
			showToast('No failed validations to analyze', 'info');
			return;
		}
		
		try {
			analyzingValidations = true;
			showToast(`Analyzing ${failedValidations.length} failed validation${failedValidations.length > 1 ? 's' : ''}...`, 'info');
			
			const analysisId = await ipc.createAnalysis(
				revision.id,
				'validation',
				failedValidations.map(e => e.id)
			);
			await ipc.runAnalysis(analysisId);
			
			// Refresh analyses to include the new one
			analyses = await ipc.getAnalysesByRevision(revision.id);
			showToast('Analysis started', 'success');
		} catch (err) {
			showToast('Failed to start analysis', 'error');
			console.error('Analysis error:', err);
		} finally {
			analyzingValidations = false;
		}
	}

	async function handleDeleteAnalysis(analysisId: string) {
		try {
			await ipc.deleteAnalysis(analysisId);
			showToast('Analysis deleted', 'success');
			// Refresh analyses list
			if (currentRevision) {
				analyses = await ipc.getAnalysesByRevision(currentRevision.id);
			}
		} catch (err) {
			showToast('Failed to delete analysis', 'error');
			console.error('Delete analysis error:', err);
		}
	}

	async function handleRerunAnalysis(analysis: Analysis) {
		try {
			await ipc.runAnalysis(analysis.id);
			showToast('Re-running analysis', 'info');
			// Refresh analyses to see updated status
			if (currentRevision) {
				analyses = await ipc.getAnalysesByRevision(currentRevision.id);
			}
		} catch (err) {
			showToast('Failed to re-run analysis', 'error');
			console.error('Re-run analysis error:', err);
		}
	}

	let unlistenCommit: (() => void) | null = null;

	onMount(async () => {
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
	<div class="flex flex-col flex-1 min-h-0 overflow-hidden">
		<!-- Top Header: Revision Header Only -->
		{#if revisionHeaderProps}
			<div class="flex-shrink-0 border-b border-border/20">
				<RevisionHeader {...revisionHeaderProps} />
			</div>
		{/if}

		<!-- Main Content Area: PromptConsole + Tabs -->
		<div class="flex-1 min-h-0 flex flex-col bg-background">
			{#if currentRevision}
				<!-- Prompt Console (resizable) -->
				<PromptConsole
					revision={currentRevision}
					validationPrompt={currentPromptSet.validationPrompt}
					autoValidate={currentPromptSet.autoValidate}
					onSaveValidation={saveValidationPrompt}
				/>

				<!-- Tabs Area (fixed below PromptConsole) -->
				<Tabs.Root bind:value={activeTab} class="flex flex-col flex-1 min-h-0">
						<Tabs.List class="flex-shrink-0 flex items-center gap-1 px-4 py-2 bg-muted/5 border-b border-border/10">
							<Tabs.Trigger
								value="executions"
								class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors
									data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm
									data-[state=inactive]:text-muted-foreground data-[state=inactive]:hover:text-foreground"
							>
								Executions {#if executionsWithUpdates.filter(e => e.revisionId === currentRevision!.id).length > 0}({executionsWithUpdates.filter(e => e.revisionId === currentRevision!.id).length}){/if}
							</Tabs.Trigger>
							<Tabs.Trigger
								value="analyses"
								class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors
									data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm
									data-[state=inactive]:text-muted-foreground data-[state=inactive]:hover:text-foreground"
							>
								Analyses {#if analyses.length > 0}({analyses.length}){/if}
							</Tabs.Trigger>
						</Tabs.List>
						
						<Tabs.Content value="executions" class="flex-1 flex flex-col min-h-0 overflow-hidden @container/table data-[state=active]:flex data-[state=inactive]:hidden">
							<ExecutionTable
								executions={executionsWithUpdates.filter(e => e.revisionId === currentRevision!.id)}
								{repositories}
								hasValidationPrompt={!!currentPromptSet.validationPrompt}
								revisionId={currentRevision?.id}
								onDeleteExecution={deleteExecutionWithConfirm}
								onStartExecution={startExecutionManually}
								onValidateExecution={validateExecutionManually}
								onStopExecution={stopExecutionManually}
								onStopValidation={stopValidationManually}
								onResumeExecution={resumeExecutionManually}
								onReviewChanges={(executionId) => {
									diffViewerExecutionId = executionId;
									diffViewerOpen = true;
								}}
								onPushExecution={pushExecutionManually}
								onRefreshCi={refreshCiManually}
								onLoadStats={loadSingleExecutionStats}
								onBulkDelete={bulkDeleteExecutions}
								onBulkStart={bulkStartExecutions}
								onBulkRestart={bulkRestartExecutions}
								onBulkStartValidations={bulkStartValidations}
								onBulkRevalidate={bulkRevalidateExecutions}
								{pushingExecutions}
								{refreshingCi}
								{loadingStats}
								{bulkStarting}
								{bulkRestarting}
								{bulkValidating}
								{bulkRevalidating}
								{bulkDeleting}
							/>
						</Tabs.Content>
						
						<Tabs.Content value="analyses" class="flex-1 min-h-0 overflow-hidden data-[state=active]:flex data-[state=inactive]:hidden">
							<AnalysisList
								{analyses}
								onDelete={(analysis) => handleDeleteAnalysis(analysis.id)}
								onRerun={handleRerunAnalysis}
							/>
						</Tabs.Content>
					</Tabs.Root>
			{:else}
				<div class="flex items-center justify-center h-full">
					<div class="text-center text-muted-foreground max-w-md">
						<h3 class="text-lg font-semibold mb-2">Select a revision</h3>
						<p class="text-sm">Choose a revision from the sidebar to view its executions and prompt details.</p>
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Dialogs -->
	<EditRepositoriesDialog
		bind:open={editReposOpen}
		currentRepos={editingRepos}
		onSave={saveRepositories}
	/>
{/if}

{#if diffViewerExecutionId}
	<DiffViewer executionId={diffViewerExecutionId} bind:open={diffViewerOpen} />
{/if}