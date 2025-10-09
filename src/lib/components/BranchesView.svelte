<script lang="ts">
	import type { ScanResult, SyncResult } from '../maestroScanner';

	let branches = $state<ScanResult[]>([]);
	let loading = $state(false);
	let syncing = $state(false);
	let lastRefresh = $state<number>(0);
	let syncResult = $state<SyncResult | null>(null);

	async function loadBranches(refresh = false) {
		loading = true;
		syncResult = null;
		try {
			const url = refresh ? '/api/maestro/branches?refresh=1' : '/api/maestro/branches';
			const res = await fetch(url);
			branches = await res.json();
			lastRefresh = Date.now();
		} catch (err) {
			console.error('Failed to load branches:', err);
		} finally {
			loading = false;
		}
	}

	async function syncRepos() {
		if (!confirm('Sync repositories?\n\nThis will:\n- Delete cloned repos not in the database\n- Delete unused repos from the database (not in prompt sets)\n\nThis action cannot be undone.')) {
			return;
		}

		syncing = true;
		try {
			const res = await fetch('/api/maestro/sync', { method: 'POST' });
			syncResult = await res.json();
			await loadBranches(true);
		} catch (err) {
			console.error('Failed to sync repositories:', err);
			alert('Failed to sync repositories');
		} finally {
			syncing = false;
		}
	}

	$effect(() => {
		loadBranches();
	});
</script>

<div class="bg-white rounded-2xl shadow-lg border border-gray-100 p-8">
	<div class="flex items-center justify-between mb-6">
		<div>
			<h2 class="text-2xl font-bold text-gray-800">Maestro Repositories & Branches</h2>
			<p class="text-sm text-gray-500 mt-1">
				View all cloned repositories and their associated maestro branches
			</p>
			{#if lastRefresh > 0}
				<p class="text-xs text-gray-400 mt-1">
					Last updated: {new Date(lastRefresh).toLocaleTimeString()}
				</p>
			{/if}
		</div>
		<div class="flex gap-2">
			<button
				onclick={() => loadBranches(true)}
				disabled={loading || syncing}
				class="px-4 py-2 border-2 border-indigo-300 text-indigo-700 rounded-lg hover:bg-indigo-50 transition-all font-semibold text-sm disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{loading ? 'Refreshing...' : 'Refresh'}
			</button>
			<button
				onclick={syncRepos}
				disabled={loading || syncing}
				class="px-4 py-2 border-2 border-amber-300 text-amber-700 rounded-lg hover:bg-amber-50 transition-all font-semibold text-sm disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{syncing ? 'Syncing...' : 'Sync'}
			</button>
		</div>
	</div>

	{#if syncResult}
		<div class="mb-6 p-4 bg-blue-50 border-2 border-blue-200 rounded-xl">
			<h3 class="text-sm font-bold text-blue-900 mb-2">Sync Results</h3>
			<div class="space-y-1 text-sm">
				{#if syncResult.deletedFromDisk.length > 0}
					<p class="text-blue-800">
						<span class="font-semibold">Deleted from disk:</span> {syncResult.deletedFromDisk.join(', ')}
					</p>
				{/if}
				{#if syncResult.deletedFromDb.length > 0}
					<p class="text-blue-800">
						<span class="font-semibold">Deleted from database:</span> {syncResult.deletedFromDb.join(', ')}
					</p>
				{/if}
				{#if syncResult.errors.length > 0}
					<div class="text-red-700">
						<p class="font-semibold">Errors:</p>
						{#each syncResult.errors as error}
							<p class="text-xs ml-2">• {error}</p>
						{/each}
					</div>
				{/if}
				{#if syncResult.deletedFromDisk.length === 0 && syncResult.deletedFromDb.length === 0 && syncResult.errors.length === 0}
					<p class="text-blue-800">Everything is already in sync!</p>
				{/if}
			</div>
		</div>
	{/if}

	{#if loading && branches.length === 0}
		<div class="text-center py-12">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600 mx-auto"></div>
			<p class="text-gray-500 mt-4">Scanning repositories...</p>
		</div>
	{:else if branches.length === 0}
		<div class="text-center py-12 text-gray-500">
			<p class="text-lg mb-2">No repositories found</p>
			<p class="text-sm">Execute a prompt set to create cloned repositories</p>
		</div>
	{:else}
		<div class="space-y-6">
			{#each branches as { repo, branches: repoBranches }}
				<div class="border-2 border-gray-200 rounded-xl p-6">
					<div class="flex items-start justify-between mb-4">
						<div>
							<h3 class="text-lg font-bold text-gray-800">
								{repo.name || repo.providerId}
							</h3>
							<div class="flex gap-2 mt-2">
								<span class={`text-xs px-2 py-1 rounded-full font-semibold ${
									repo.existsInDb ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'
								}`}>
									{repo.existsInDb ? 'In Database' : 'Not in DB'}
								</span>
								<span class={`text-xs px-2 py-1 rounded-full font-semibold ${
									repo.existsOnDisk ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-600'
								}`}>
									{repo.existsOnDisk ? 'Cloned' : 'Not Cloned'}
								</span>
							</div>
							<p class="text-xs text-gray-400 mt-1 font-mono">{repo.path}</p>
						</div>
					</div>

					{#if repoBranches.length === 0}
						<p class="text-sm text-gray-500 italic">No maestro branches</p>
					{:else}
						<div class="space-y-3">
							<h4 class="text-sm font-semibold text-gray-700">Branches ({repoBranches.length})</h4>
							{#each repoBranches as branch}
								<div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
									<div class="flex items-start justify-between">
										<div class="flex-1">
											<p class="font-mono text-sm text-gray-700 mb-2">{branch.name}</p>
											{#if branch.ids}
												<div class="space-y-1.5 text-xs">
													{#if branch.promptset}
														<div>
															<span class="text-gray-500">Prompt Set:</span>
															<span class="font-semibold text-gray-800 ml-1">{branch.promptset.name}</span>
															<span class="text-gray-400 ml-1 font-mono">({branch.promptset.id.slice(0, 8)})</span>
														</div>
													{:else}
														<div class="text-amber-600">
															⚠ Prompt set not found (ID: {branch.ids.promptsetId.slice(0, 8)})
														</div>
													{/if}

													{#if branch.revision}
														<div>
															<span class="text-gray-500">Revision:</span>
															<span class="text-gray-700 ml-1 font-mono">{branch.revision.id.slice(0, 8)}</span>
															<span class="text-gray-400 ml-1">
																({new Date(branch.revision.createdAt).toLocaleDateString()})
															</span>
														</div>
													{:else}
														<div class="text-amber-600">
															⚠ Revision not found (ID: {branch.ids.revisionId.slice(0, 8)})
														</div>
													{/if}

													{#if branch.execution}
														<div>
															<span class="text-gray-500">Execution:</span>
															<span class={`ml-1 px-2 py-0.5 rounded-full font-semibold ${
																branch.execution.status === 'completed' ? 'bg-green-100 text-green-700' :
																branch.execution.status === 'failed' ? 'bg-red-100 text-red-700' :
																branch.execution.status === 'running' ? 'bg-blue-100 text-blue-700' :
																'bg-gray-100 text-gray-700'
															}`}>
																{branch.execution.status}
															</span>
															{#if branch.execution.threadUrl}
																<a
																	href={branch.execution.threadUrl}
																	target="_blank"
																	rel="noopener noreferrer"
																	class="ml-2 text-indigo-600 hover:text-indigo-800 underline"
																>
																	View Thread →
																</a>
															{/if}
														</div>
														{#if branch.execution.validationStatus}
															<div>
																<span class="text-gray-500">Validation:</span>
																<span class={`ml-1 px-2 py-0.5 rounded-full font-semibold ${
																	branch.execution.validationStatus === 'passed' ? 'bg-green-100 text-green-700' :
																	branch.execution.validationStatus === 'failed' ? 'bg-red-100 text-red-700' :
																	branch.execution.validationStatus === 'running' ? 'bg-blue-100 text-blue-700' :
																	'bg-gray-100 text-gray-700'
																}`}>
																	{branch.execution.validationStatus}
																</span>
																{#if branch.execution.validationThreadUrl}
																	<a
																		href={branch.execution.validationThreadUrl}
																		target="_blank"
																		rel="noopener noreferrer"
																		class="ml-2 text-indigo-600 hover:text-indigo-800 underline"
																	>
																		View Validation →
																	</a>
																{/if}
															</div>
														{/if}
													{:else}
														<div class="text-amber-600">
															⚠ Execution not found (ID: {branch.ids.executionId.slice(0, 8)})
														</div>
													{/if}
												</div>
											{:else}
												<p class="text-xs text-amber-600">Invalid branch name format</p>
											{/if}
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
