<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount, onDestroy } from 'svelte';
	import DiffViewer from '$lib/components/DiffViewer.svelte';
	import RevisionHeader from '$lib/components/ui/RevisionHeader.svelte';
	import PromptConsole from '$lib/components/ui/PromptConsole.svelte';
	import ExecutionTable from '$lib/components/ui/ExecutionTable.svelte';
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
			...(updates?.ciStatus && { ciStatus: updates.ciStatus }),
			...(updates?.ciUrl && { ciUrl: updates.ciUrl }),
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
	let tabsHeight = $state(400);
	let isResizingTabs = $state(false);

	const promptsetId = $derived($page.params.id);
	const revisionParam = $derived($page.url.searchParams.get('revision'));
	
	// Reload prompt set when promptsetId changes
	$effect(() => {
		if (promptsetId) {
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
	
	// Watch for revision parameter changes
	$effect(() => {
		const param = revisionParam;
		if (!param || revisions.length === 0) return;
		
		const revision = revisions.find(r => r.id === param || toShortHash(r.id) === param);
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
		ciStatus: null,
		ciCheckedAt: null,
		ciUrl: null,
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
			showToast('Pushing commit to remote...', 'info');
			await api.executions.push(execution.id, false); // false = not force push
			showToast('Push completed successfully', 'success');
			// CI status will update via event bus after push
		} catch (err) {
			showToast('Failed to push: ' + err, 'error');
		}
	}

	async function refreshCiManually(execution: Execution) {
		try {
			showToast('Refreshing CI status...', 'info');
			await api.ci.refreshStatus(execution.id);
			// CI status will update via event bus
			showToast('CI status refreshed', 'success');
		} catch (err) {
			showToast('Failed to refresh CI: ' + err, 'error');
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
	}

	async function bulkStartExecutions(selectedExecutions: Execution[]) {
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

	async function bulkStartValidations(selectedExecutions: Execution[]) {
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
			showToast(`${successCount} validation${successCount > 1 ? 's' : ''} revalidated`, 'info');
		}
		if (failCount > 0) {
			showToast(`Failed to revalidate ${failCount} validation${failCount > 1 ? 's' : ''}`, 'error');
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
			const analysisId = await ipc.createAnalysis(
				revision.id,
				'execution',
				failedExecutions.map(e => e.id)
			);
			await ipc.runAnalysis(analysisId);
			showToast(`Started analyzing ${failedExecutions.length} failed execution${failedExecutions.length > 1 ? 's' : ''}`, 'info');
			
			// Refresh analyses to include the new one
			analyses = await ipc.getAnalysesByRevision(revision.id);
		} catch (err) {
			showToast('Failed to start analysis', 'error');
			console.error('Analysis error:', err);
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
			const analysisId = await ipc.createAnalysis(
				revision.id,
				'validation',
				failedValidations.map(e => e.id)
			);
			await ipc.runAnalysis(analysisId);
			showToast(`Started analyzing ${failedValidations.length} failed validation${failedValidations.length > 1 ? 's' : ''}`, 'info');
			
			// Refresh analyses to include the new one
			analyses = await ipc.getAnalysesByRevision(revision.id);
		} catch (err) {
			showToast('Failed to start analysis', 'error');
			console.error('Analysis error:', err);
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

	function handleTabsResizeStart(e: MouseEvent) {
		isResizingTabs = true;
		e.preventDefault();
	}

	function handleTabsResizeMove(e: MouseEvent) {
		if (!isResizingTabs) return;
		const maxHeight = window.innerHeight * 0.7;
		const newHeight = window.innerHeight - e.clientY;
		tabsHeight = Math.max(200, Math.min(maxHeight, newHeight));
	}

	function handleTabsResizeEnd() {
		isResizingTabs = false;
	}

	$effect(() => {
		if (isResizingTabs) {
			document.addEventListener('mousemove', handleTabsResizeMove);
			document.addEventListener('mouseup', handleTabsResizeEnd);
			return () => {
				document.removeEventListener('mousemove', handleTabsResizeMove);
				document.removeEventListener('mouseup', handleTabsResizeEnd);
			};
		}
	});

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

		<!-- Main Content Area: PromptConsole + Tabs with resizable divider -->
		<div class="flex-1 min-h-0 flex flex-col overflow-hidden">
			{#if currentRevision}
				<!-- Prompt Console (top, flexible) -->
				<div class="flex-1 min-h-0 overflow-hidden">
					<PromptConsole
						revision={currentRevision}
						validationPrompt={currentPromptSet.validationPrompt}
						autoValidate={currentPromptSet.autoValidate}
						onSaveValidation={saveValidationPrompt}
					/>
				</div>

				<!-- Resize Handle -->
				<button
					onmousedown={handleTabsResizeStart}
					class="flex-shrink-0 h-1.5 bg-border/40 hover:bg-primary/40 transition-colors cursor-ns-resize group relative"
					aria-label="Resize tabs area"
				>
					<div class="absolute inset-x-0 -top-1 -bottom-1"></div>
				</button>

				<!-- Tabs Area (bottom, fixed height) -->
				<div class="flex-shrink-0 flex flex-col overflow-hidden bg-background" style="height: {tabsHeight}px;">
					<Tabs.Root bind:value={activeTab} class="flex flex-col flex-1 min-h-0 overflow-hidden">
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
						
						<Tabs.Content value="executions" class="flex-1 flex flex-col overflow-auto @container/table">
							<ExecutionTable
								executions={executionsWithUpdates.filter(e => e.revisionId === currentRevision!.id)}
								{repositories}
								hasValidationPrompt={!!currentPromptSet.validationPrompt}
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
								onBulkDelete={bulkDeleteExecutions}
								onBulkStart={bulkStartExecutions}
								onBulkRestart={bulkRestartExecutions}
								onBulkStartValidations={bulkStartValidations}
								onBulkRevalidate={bulkRevalidateExecutions}
								onExecuteAll={() => executeRevision(currentRevision!)}
								onStopAll={stopAllExecutions}
								onStopAllValidations={stopAllValidations}
								onRefreshAllCi={refreshAllCiManually}
								onAnalyzeExecutions={() => handleAnalyzeExecutions(currentRevision!)}
								onAnalyzeValidations={() => handleAnalyzeValidations(currentRevision!)}
							/>
						</Tabs.Content>
						
						<Tabs.Content value="analyses" class="flex-1 min-h-0 overflow-auto">
							<AnalysisList
								{analyses}
								onDelete={(analysis) => handleDeleteAnalysis(analysis.id)}
								onRerun={handleRerunAnalysis}
							/>
						</Tabs.Content>
					</Tabs.Root>
				</div>
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