<script lang="ts">
	import RepositorySelector from './lib/components/RepositorySelector.svelte';
	import PromptDiff from './lib/components/PromptDiff.svelte';
	import BranchesView from './lib/components/BranchesView.svelte';
	import type { Repository } from './lib/providers';
	import { api } from './lib/api';
	import type { PromptSet, PromptRevision, Execution, Repository as DBRepository } from './lib/types';

	let currentPromptSet = $state<PromptSet | null>(null);
	let revisions = $state<PromptRevision[]>([]);
	let currentRevision = $state<PromptRevision | null>(null);
	let executions = $state<Execution[]>([]);
	let repositories = $state<Map<string, DBRepository>>(new Map());

	let view = $state<'list' | 'create' | 'detail' | 'branches'>('list');
	let step = $state<'create' | 'prompt' | 'execute'>('create');
	let promptSetName = $state('');
	let selectedRepos = $state<Repository[]>([]);
	let promptText = $state('');
	let validationPromptText = $state('');
	let isRunning = $state(false);
	let allPromptSets = $state<PromptSet[]>([]);
	let isEditingValidation = $state(false);

	async function createPromptSet() {
		if (!promptSetName.trim() || selectedRepos.length === 0) {
			return;
		}

		const repoPromises = selectedRepos.map(async (repo) => {
			try {
				const dbRepo = await api.repositories.find(repo.provider, repo.fullName)
					.catch(() => api.repositories.create(repo.provider, repo.fullName, repo.name));
				repositories.set(dbRepo.id, dbRepo);
				return dbRepo.id;
			} catch (err) {
				console.error('Failed to create repository:', err);
				return null;
			}
		});

		const repoIds = (await Promise.all(repoPromises)).filter(Boolean) as string[];

		currentPromptSet = await api.promptSets.create(
			promptSetName, 
			repoIds, 
			null
		);
		revisions = await api.promptSets.getRevisions(currentPromptSet.id);
		view = 'create';
		step = 'prompt';
	}

	async function saveRevision() {
		if (!currentPromptSet || !promptText.trim()) {
			return;
		}

		const parentId = revisions.length > 0 ? revisions[0].id : null;
		
		if (revisions.length === 0 && validationPromptText.trim()) {
			await api.promptSets.update(currentPromptSet.id, {
				validationPrompt: validationPromptText.trim()
			});
			currentPromptSet.validationPrompt = validationPromptText.trim();
		}
		
		currentRevision = await api.revisions.create(
			currentPromptSet.id,
			promptText,
			parentId
		);
		
		revisions = await api.promptSets.getRevisions(currentPromptSet.id);
		step = 'execute';
		
		isRunning = true;
		await api.revisions.execute(currentRevision.id);
		await pollExecutions(currentRevision.id);
		isRunning = false;
	}

	async function executePrompt() {
		if (!currentPromptSet || !currentRevision) {
			return;
		}

		isRunning = true;
		await api.revisions.execute(currentRevision.id);
		await pollExecutions(currentRevision.id);
		isRunning = false;
	}

	function newRevision() {
		step = 'prompt';
	}

	function startOver() {
		view = 'list';
		step = 'create';
		promptSetName = '';
		selectedRepos = [];
		promptText = '';
		validationPromptText = '';
		currentPromptSet = null;
		currentRevision = null;
		revisions = [];
		executions = [];
	}

	function showCreateForm() {
		view = 'create';
		step = 'create';
		promptSetName = '';
		selectedRepos = [];
		promptText = '';
		validationPromptText = '';
	}

	async function loadPromptSets() {
		allPromptSets = await api.promptSets.getAll();
	}

	async function viewPromptSet(promptSet: PromptSet) {
		currentPromptSet = promptSet;
		revisions = await api.promptSets.getRevisions(promptSet.id);
		executions = await api.promptSets.getExecutions(promptSet.id);
		view = 'detail';
		validationPromptText = promptSet.validationPrompt || '';
		isEditingValidation = false;
		
		for (const repoId of promptSet.repositoryIds) {
			if (!repositories.has(repoId)) {
				const repo = await api.repositories.get(repoId);
				repositories.set(repoId, repo);
			}
		}

		await backfillMissingStats();
	}

	async function viewRevisionExecutions(revision: PromptRevision) {
		currentRevision = revision;
		executions = await api.revisions.getExecutions(revision.id);
		await backfillMissingStats();
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

		console.log(`Backfilling stats for ${missingStats.length} executions`);
		
		for (const execution of missingStats) {
			try {
				const updated = await api.executions.backfillStats(execution.id);
				const index = executions.findIndex(e => e.id === execution.id);
				if (index !== -1) {
					executions[index] = updated;
				}
			} catch (err) {
				console.warn(`Failed to backfill stats for ${execution.id}:`, err);
			}
		}
	}

	async function executeRevision(revision: PromptRevision) {
		if (!currentPromptSet) return;

		currentRevision = revision;
		isRunning = true;
		await api.revisions.execute(revision.id);
		await pollExecutions(revision.id);
		isRunning = false;
	}

	$effect(() => {
		if (view === 'list') {
			loadPromptSets();
		}
	});

	async function getRepoName(repoId: string): Promise<string> {
		if (repositories.has(repoId)) {
			return repositories.get(repoId)!.name || repoId;
		}
		try {
			const repo = await api.repositories.get(repoId);
			repositories.set(repoId, repo);
			return repo.name || repoId;
		} catch {
			return repoId;
		}
	}

	async function deleteExecutionWithConfirm(execution: Execution, repoName: string) {
		if (!confirm(`Delete execution for ${repoName}?\n\nThis will:\n- Delete the database record\n- Delete the git branch\n\nThis action cannot be undone.`)) {
			return;
		}
		
		try {
			await api.executions.delete(execution.id);
			executions = executions.filter(e => e.id !== execution.id);
		} catch (err) {
			alert('Failed to delete execution: ' + err);
		}
	}

	async function validateExecutionManually(execution: Execution) {
		try {
			await api.executions.validate(execution.id);
			if (currentRevision) {
				await pollExecutions(currentRevision.id);
			}
		} catch (err) {
			alert('Failed to start validation: ' + err);
		}
	}

	async function pollExecutions(revisionId: string) {
		executions = await api.revisions.getExecutions(revisionId);
		while (executions.some(e => 
			e.status === 'pending' || 
			e.status === 'running' || 
			e.validationStatus === 'pending' || 
			e.validationStatus === 'running'
		)) {
			await new Promise(r => setTimeout(r, 1500));
			executions = await api.revisions.getExecutions(revisionId);
		}
	}

	async function deletePromptSetWithConfirm(promptSet: PromptSet) {
		const revisionCount = await api.promptSets.getRevisions(promptSet.id).then(r => r.length);
		const executionCount = await api.promptSets.getExecutions(promptSet.id).then(e => e.length);
		
		if (!confirm(
			`Delete prompt set "${promptSet.name}"?\n\n` +
			`This will delete:\n` +
			`- The prompt set\n` +
			`- ${revisionCount} revision(s)\n` +
			`- ${executionCount} execution(s)\n` +
			`- All associated git branches\n\n` +
			`This action cannot be undone.`
		)) {
			return;
		}
		
		try {
			await api.promptSets.delete(promptSet.id);
			allPromptSets = allPromptSets.filter(p => p.id !== promptSet.id);
			if (currentPromptSet?.id === promptSet.id) {
				startOver();
			}
		} catch (err) {
			alert('Failed to delete prompt set: ' + err);
		}
	}

	async function saveValidationPrompt() {
		if (!currentPromptSet) return;
		
		try {
			const updated = await api.promptSets.update(currentPromptSet.id, {
				validationPrompt: validationPromptText.trim() || null
			});
			currentPromptSet = updated;
			isEditingValidation = false;
		} catch (err) {
			alert('Failed to update validation prompt: ' + err);
		}
	}

	function cancelEditValidation() {
		if (currentPromptSet) {
			validationPromptText = currentPromptSet.validationPrompt || '';
		}
		isEditingValidation = false;
	}
</script>

<div class="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 p-8">
	<div class="max-w-6xl mx-auto">
		<div class="mb-10">
			<div class="flex items-center justify-between mb-4">
				<div>
					<h1 class="text-5xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent mb-2">Maestro</h1>
					<p class="text-gray-600 text-lg">Orchestrate AI prompts across multiple repositories</p>
				</div>
				{#if view === 'list'}
					<button
						onclick={showCreateForm}
						class="px-6 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl hover:from-blue-700 hover:to-indigo-700 transition-all font-semibold shadow-lg hover:shadow-xl"
					>
						New Prompt Set
					</button>
				{:else}
					<button
						onclick={startOver}
						class="px-6 py-3 border-2 border-gray-300 text-gray-700 rounded-xl hover:bg-gray-50 transition-all font-semibold"
					>
						← Back to List
					</button>
				{/if}
			</div>
			<div class="flex gap-2">
				<button
					onclick={() => view = 'list'}
					class={`px-4 py-2 rounded-lg font-semibold transition-all ${
						view === 'list' || view === 'create' || view === 'detail'
							? 'bg-indigo-100 text-indigo-700'
							: 'bg-gray-100 text-gray-600 hover:bg-gray-200'
					}`}
				>
					Prompt Sets
				</button>
				<button
					onclick={() => view = 'branches'}
					class={`px-4 py-2 rounded-lg font-semibold transition-all ${
						view === 'branches'
							? 'bg-indigo-100 text-indigo-700'
							: 'bg-gray-100 text-gray-600 hover:bg-gray-200'
					}`}
				>
					Branches
				</button>
			</div>
		</div>

		{#if view === 'list'}
			<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8">
				<h2 class="text-2xl font-bold text-gray-800 mb-6">Prompt Sets</h2>
				{#if allPromptSets.length === 0}
					<div class="text-center py-12 text-gray-500">
						<p class="text-lg mb-4">No prompt sets yet</p>
						<button
							onclick={showCreateForm}
							class="px-6 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl hover:from-blue-700 hover:to-indigo-700 transition-all font-semibold shadow-lg hover:shadow-xl"
						>
							Create Your First Prompt Set
						</button>
					</div>
				{:else}
					<div class="space-y-4">
						{#each allPromptSets as promptSet}
							<div class="relative group">
								<button
									onclick={() => viewPromptSet(promptSet)}
									class="w-full p-6 border-2 border-gray-200 rounded-xl hover:border-indigo-300 hover:bg-indigo-50 transition-all text-left"
								>
									<div class="flex items-center justify-between mb-2">
										<h3 class="text-xl font-bold text-gray-800">{promptSet.name}</h3>
										<span class="bg-blue-100 text-blue-700 px-3 py-1 rounded-full text-sm font-semibold">
											{promptSet.repositoryIds.length} repos
										</span>
									</div>
									<p class="text-sm text-gray-500">
										Created {new Date(promptSet.createdAt).toLocaleDateString()}
									</p>
								</button>
								<button
									onclick={(e) => { e.stopPropagation(); deletePromptSetWithConfirm(promptSet); }}
									class="absolute top-4 right-4 px-3 py-1.5 text-red-600 hover:bg-red-50 rounded-lg transition-all text-xs font-semibold opacity-0 group-hover:opacity-100"
									title="Delete prompt set"
								>
									Delete
								</button>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}

		{#if view === 'detail' && currentPromptSet}
			<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 mb-6">
				<div class="mb-6 flex items-start justify-between">
					<div>
						<h2 class="text-2xl font-bold text-gray-800 mb-2">{currentPromptSet.name}</h2>
						<div class="flex gap-4 text-sm text-gray-600">
							<span>{currentPromptSet.repositoryIds.length} repositories</span>
							<span>•</span>
							<span>{revisions.length} revisions</span>
						</div>
					</div>
					<button
						onclick={() => deletePromptSetWithConfirm(currentPromptSet)}
						class="px-4 py-2 text-red-600 hover:bg-red-50 rounded-lg transition-all text-sm font-semibold border-2 border-red-200"
						title="Delete prompt set"
					>
						Delete Prompt Set
					</button>
				</div>

				<div class="mb-6">
					<h3 class="text-lg font-bold text-gray-800 mb-3">Repositories</h3>
					<div class="flex flex-wrap gap-2">
						{#each currentPromptSet.repositoryIds as repoId}
							{#await getRepoName(repoId)}
								<span class="px-3 py-1.5 bg-gray-100 rounded-lg text-sm animate-pulse">Loading...</span>
							{:then repoName}
								<span class="px-3 py-1.5 bg-gray-100 rounded-lg text-sm font-medium">{repoName}</span>
							{/await}
						{/each}
					</div>
				</div>

				<div class="mb-6">
					<div class="flex items-center justify-between mb-3">
						<h3 class="text-lg font-bold text-gray-800">Validation Prompt</h3>
						{#if !isEditingValidation}
							<button
								onclick={() => isEditingValidation = true}
								class="px-3 py-1.5 text-indigo-600 hover:bg-indigo-50 rounded-lg transition-all text-sm font-semibold border border-indigo-300"
							>
								{currentPromptSet.validationPrompt ? 'Edit' : 'Add'}
							</button>
						{/if}
					</div>
					
					{#if isEditingValidation}
						<div class="space-y-3">
							<textarea
								bind:value={validationPromptText}
								placeholder="Enter a prompt to validate each execution after it completes..."
								rows="8"
								class="w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all resize-y font-mono text-sm"
							></textarea>
							<p class="text-xs text-gray-500">
								This prompt will run against the modified branch after each successful execution
							</p>
							<div class="flex gap-2">
								<button
									onclick={saveValidationPrompt}
									class="px-4 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all font-semibold text-sm"
								>
									Save
								</button>
								<button
									onclick={cancelEditValidation}
									class="px-4 py-2 border-2 border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-all font-semibold text-sm"
								>
									Cancel
								</button>
							</div>
						</div>
					{:else}
						{#if currentPromptSet.validationPrompt}
							<pre class="text-sm text-gray-700 whitespace-pre-wrap font-mono bg-gray-50 p-4 rounded-lg">{currentPromptSet.validationPrompt}</pre>
						{:else}
							<p class="text-sm text-gray-500 italic">No validation prompt set. Add one to enable automatic validation of executions.</p>
						{/if}
					{/if}
				</div>

				<div>
					<div class="flex items-center justify-between mb-4">
						<h3 class="text-lg font-bold text-gray-800">Revisions</h3>
						<button
							onclick={() => { view = 'create'; step = 'prompt'; }}
							class="px-4 py-2 border-2 border-indigo-300 text-indigo-700 rounded-lg hover:bg-indigo-50 transition-all font-semibold text-sm"
						>
							New Revision
						</button>
					</div>
					{#if revisions.length === 0}
						<div class="text-center py-8 text-gray-500">
							<p>No revisions yet</p>
						</div>
					{:else}
						<div class="space-y-4">
							{#each revisions as revision}
								<div class="border-2 border-gray-200 rounded-xl p-4">
									<div class="flex items-start justify-between mb-3">
										<div class="flex-1">
											<div class="flex items-center gap-2 mb-1">
												<span class="font-mono text-sm text-gray-600">{revision.id.slice(0, 8)}</span>
												<span class="text-xs text-gray-400">
													{new Date(revision.createdAt).toLocaleString()}
												</span>
											</div>
											<pre class="text-sm text-gray-700 whitespace-pre-wrap font-mono bg-gray-50 p-3 rounded-lg mt-2 max-h-32 overflow-y-auto">{revision.promptText}</pre>
										</div>
									</div>
									<div class="flex gap-2">
										<button
											onclick={() => viewRevisionExecutions(revision)}
											class="flex-1 px-4 py-2 bg-indigo-100 text-indigo-700 rounded-lg hover:bg-indigo-200 transition-all font-semibold text-sm"
										>
											View Executions ({executions.filter(e => e.revisionId === revision.id).length})
										</button>
										<button
											onclick={() => executeRevision(revision)}
											disabled={isRunning}
											class="px-4 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all disabled:from-gray-300 disabled:to-gray-400 disabled:cursor-not-allowed font-semibold text-sm"
										>
											Execute
										</button>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			{#if currentRevision && executions.length > 0}
				<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8">
					<h3 class="text-xl font-bold text-gray-800 mb-4">
						Executions for Revision {currentRevision.id.slice(0, 8)}
					</h3>
					<div class="space-y-4">
						{#each executions.filter(e => e.revisionId === currentRevision.id) as execution}
							{#await getRepoName(execution.repositoryId)}
								<div class="p-4 border-2 border-gray-200 rounded-xl animate-pulse">
									<div class="h-4 bg-gray-200 rounded w-1/3"></div>
								</div>
							{:then repoName}
								<div class="p-4 border-2 border-gray-200 rounded-xl">
									<div class="flex items-center justify-between mb-2">
										<h4 class="font-semibold text-gray-800">{repoName}</h4>
										<div class="flex items-center gap-2">
											<span class={`px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wide ${
												execution.status === 'completed' ? 'bg-green-100 text-green-700 ring-2 ring-green-200' :
												execution.status === 'failed' ? 'bg-red-100 text-red-700 ring-2 ring-red-200' :
												execution.status === 'running' ? 'bg-blue-100 text-blue-700 ring-2 ring-blue-200' :
												'bg-gray-100 text-gray-700 ring-2 ring-gray-200'
											}`}>
												{execution.status}
											</span>
											{#if execution.validationStatus}
												<span class={`px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wide ${
													execution.validationStatus === 'passed' ? 'bg-emerald-100 text-emerald-700 ring-2 ring-emerald-200' :
													execution.validationStatus === 'failed' ? 'bg-orange-100 text-orange-700 ring-2 ring-orange-200' :
													execution.validationStatus === 'running' ? 'bg-blue-100 text-blue-700 ring-2 ring-blue-200 animate-pulse' :
													'bg-yellow-100 text-yellow-700 ring-2 ring-yellow-200'
												}`} title="Validation status">
													✓ {execution.validationStatus}
												</span>
											{/if}
											{#if currentPromptSet?.validationPrompt}
												<button
													onclick={() => validateExecutionManually(execution)}
													disabled={execution.status !== 'completed' || execution.validationStatus === 'running'}
													class="px-3 py-1.5 text-emerald-600 hover:bg-emerald-50 rounded-lg transition-all text-xs font-semibold disabled:opacity-50 disabled:cursor-not-allowed border border-emerald-300"
													title={
														execution.status !== 'completed' ? 'Execution must complete first' :
														execution.validationStatus === 'running' ? 'Validation in progress' :
														execution.validationStatus ? 'Re-run validation' : 'Run validation'
													}
												>
													{execution.validationStatus ? '↻ Revalidate' : '✓ Validate'}
												</button>
											{/if}
											<button
												onclick={() => deleteExecutionWithConfirm(execution, repoName)}
												class="px-2 py-1 text-red-600 hover:bg-red-50 rounded-lg transition-all text-xs font-semibold"
												title="Delete execution"
											>
												Delete
											</button>
										</div>
									</div>
									<div class="flex flex-col gap-1">
										{#if execution.threadUrl}
											<a
												href={execution.threadUrl}
												target="_blank"
												rel="noopener noreferrer"
												class="text-sm text-indigo-600 hover:text-indigo-800 underline"
											>
												View Execution Thread →
											</a>
										{/if}
										{#if execution.validationThreadUrl}
											<a
												href={execution.validationThreadUrl}
												target="_blank"
												rel="noopener noreferrer"
												class="text-sm text-emerald-600 hover:text-emerald-800 underline"
											>
												View Validation Thread →
											</a>
										{/if}
									</div>
									{#if execution.status === 'completed' && (execution.linesAdded > 0 || execution.linesRemoved > 0 || execution.filesAdded > 0 || execution.filesRemoved > 0 || execution.filesModified > 0)}
										<div class="mt-3 pt-3 border-t border-gray-200">
											<div class="flex items-center gap-4 text-xs">
												{#if execution.filesAdded > 0}
													<span class="text-green-600 font-mono">+{execution.filesAdded} files</span>
												{/if}
												{#if execution.filesRemoved > 0}
													<span class="text-red-600 font-mono">-{execution.filesRemoved} files</span>
												{/if}
												{#if execution.filesModified > 0}
													<span class="text-amber-600 font-mono">~{execution.filesModified} files</span>
												{/if}
												{#if execution.linesAdded > 0}
													<span class="text-green-600 font-mono">+{execution.linesAdded} lines</span>
												{/if}
												{#if execution.linesRemoved > 0}
													<span class="text-red-600 font-mono">-{execution.linesRemoved} lines</span>
												{/if}
											</div>
										</div>
									{/if}
									<p class="text-xs text-gray-500 mt-2">
										Started {new Date(execution.createdAt).toLocaleString()}
									</p>
								</div>
							{/await}
						{/each}
					</div>
				</div>
			{/if}
		{/if}

		{#if view === 'create' && step === 'create'}
			<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 mb-6 transition-all hover:shadow-xl">
				<h2 class="text-2xl font-bold text-gray-800 mb-5">Create Prompt Set</h2>
				
				<div class="mb-6">
					<label for="promptset-name" class="block text-sm font-semibold text-gray-700 mb-2">Name</label>
					<input
						id="promptset-name"
						type="text"
						bind:value={promptSetName}
						placeholder="e.g., Update Documentation"
						class="w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all"
					/>
				</div>

				<div class="mb-6">
					<div class="flex items-center justify-between mb-3">
						<h3 class="text-sm font-semibold text-gray-700">Repositories</h3>
						{#if selectedRepos.length > 0}
							<span class="bg-blue-100 text-blue-700 px-3 py-1 rounded-full text-sm font-semibold">
								{selectedRepos.length} selected
							</span>
						{/if}
					</div>
					<RepositorySelector bind:selectedRepos />
				</div>

				<button
					onclick={createPromptSet}
					disabled={!promptSetName.trim() || selectedRepos.length === 0}
					class="w-full px-8 py-3.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl hover:from-blue-700 hover:to-indigo-700 transition-all disabled:from-gray-300 disabled:to-gray-400 disabled:cursor-not-allowed font-semibold shadow-lg hover:shadow-xl disabled:shadow-none"
				>
					Continue to Prompt
				</button>
			</div>
		{/if}

		{#if view === 'create' && step === 'prompt' && currentPromptSet}
			<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 mb-6 transition-all hover:shadow-xl">
				<div class="flex items-center justify-between mb-5">
					<h2 class="text-2xl font-bold text-gray-800">{currentPromptSet.name}</h2>
					<span class="text-sm text-gray-500">
						{currentPromptSet.repositoryIds.length} repositories
					</span>
				</div>

				{#if revisions.length === 0}
					<div class="mb-4">
						<label for="initial-prompt" class="block text-sm font-semibold text-gray-700 mb-2">Initial Prompt</label>
						<textarea
							id="initial-prompt"
							bind:value={promptText}
							placeholder="Enter your prompt to execute across all repositories..."
							rows="12"
							class="w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all resize-y font-mono text-sm"
						></textarea>
					</div>
					<div class="mb-4">
						<label for="validation-prompt" class="block text-sm font-semibold text-gray-700 mb-2">
							Validation Prompt <span class="text-gray-500 font-normal">(optional)</span>
						</label>
						<textarea
							id="validation-prompt"
							bind:value={validationPromptText}
							placeholder="Enter a prompt to validate each execution after it completes..."
							rows="6"
							class="w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all resize-y font-mono text-sm"
						></textarea>
						<p class="text-xs text-gray-500 mt-1">
							This prompt will run against the modified branch after each successful execution
						</p>
					</div>
				{:else}
					<PromptDiff
						oldText={revisions[0].promptText}
						newText={promptText}
						onupdate={(text) => promptText = text}
					/>
				{/if}

				<div class="flex gap-3 mt-5">
					<button
						onclick={saveRevision}
						disabled={!promptText.trim()}
						class="flex-1 px-8 py-3.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl hover:from-blue-700 hover:to-indigo-700 transition-all disabled:from-gray-300 disabled:to-gray-400 disabled:cursor-not-allowed font-semibold shadow-lg hover:shadow-xl disabled:shadow-none"
					>
						{revisions.length === 0 ? 'Create & Execute' : 'Save Revision & Execute'}
					</button>
					<button
						onclick={startOver}
						class="px-6 py-3.5 border-2 border-gray-300 text-gray-700 rounded-xl hover:bg-gray-50 transition-all font-semibold"
					>
						Cancel
					</button>
				</div>
			</div>
		{/if}

		{#if view === 'create' && step === 'execute' && currentPromptSet && currentRevision}
			<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 mb-6 transition-all hover:shadow-xl">
				<div class="flex items-center justify-between mb-5">
					<div>
						<h2 class="text-2xl font-bold text-gray-800">{currentPromptSet.name}</h2>
						<p class="text-sm text-gray-500 mt-1">Revision: {currentRevision.id.slice(0, 8)}</p>
					</div>
					<div class="flex gap-2">
						<button
							onclick={newRevision}
							class="px-6 py-2.5 border-2 border-indigo-300 text-indigo-700 rounded-xl hover:bg-indigo-50 transition-all font-semibold"
						>
							New Revision
						</button>
						<button
							onclick={startOver}
							class="px-6 py-2.5 border-2 border-gray-300 text-gray-700 rounded-xl hover:bg-gray-50 transition-all font-semibold"
						>
							New Prompt Set
						</button>
					</div>
				</div>

				<button
					onclick={executePrompt}
					disabled={isRunning}
					class="w-full mb-6 px-8 py-3.5 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl hover:from-blue-700 hover:to-indigo-700 transition-all disabled:from-gray-300 disabled:to-gray-400 disabled:cursor-not-allowed font-semibold shadow-lg hover:shadow-xl disabled:shadow-none"
				>
					{isRunning ? 'Executing...' : 'Execute Across All Repos'}
				</button>

				{#if executions.length > 0}
					<div class="space-y-4">
						<h3 class="text-xl font-bold text-gray-800">Executions</h3>
						{#each executions as execution}
							{#await getRepoName(execution.repositoryId)}
								<div class="p-4 border-2 border-gray-200 rounded-xl animate-pulse">
									<div class="h-4 bg-gray-200 rounded w-1/3"></div>
								</div>
							{:then repoName}
								<div class="p-4 border-2 border-gray-200 rounded-xl">
									<div class="flex items-center justify-between mb-2">
										<h4 class="font-semibold text-gray-800">{repoName}</h4>
										<div class="flex items-center gap-2">
											<span class={`px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wide ${
												execution.status === 'completed' ? 'bg-green-100 text-green-700 ring-2 ring-green-200' :
												execution.status === 'failed' ? 'bg-red-100 text-red-700 ring-2 ring-red-200' :
												execution.status === 'running' ? 'bg-blue-100 text-blue-700 ring-2 ring-blue-200' :
												'bg-gray-100 text-gray-700 ring-2 ring-gray-200'
											}`}>
												{execution.status}
											</span>
											<button
												onclick={() => deleteExecutionWithConfirm(execution, repoName)}
												class="px-2 py-1 text-red-600 hover:bg-red-50 rounded-lg transition-all text-xs font-semibold"
												title="Delete execution"
											>
												Delete
											</button>
										</div>
									</div>
									{#if execution.threadUrl}
										<a
											href={execution.threadUrl}
											target="_blank"
											rel="noopener noreferrer"
											class="text-sm text-indigo-600 hover:text-indigo-800 underline"
										>
											View Thread →
										</a>
									{/if}
								</div>
							{/await}
						{/each}
					</div>
				{/if}
			</div>
		{/if}

		{#if view === 'branches'}
			<BranchesView />
		{/if}
	</div>
</div>
