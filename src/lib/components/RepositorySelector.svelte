<script lang="ts">
	import { getConfiguredProviders, type Repository } from '../providers';
	import { onMount } from 'svelte';

	let {
		selectedRepos = $bindable([])
	}: {
		selectedRepos: Repository[];
	} = $props();

	let searchQuery = $state('');
	let searchResults = $state<Repository[]>([]);
	let isSearching = $state(false);
	let showDropdown = $state(false);
	let inputElement = $state<HTMLInputElement>();
	let debounceTimer: number;

	const providers = getConfiguredProviders();



	async function handleSearch() {
		if (providers.length === 0 || !searchQuery.trim()) {
			searchResults = [];
			return;
		}

		isSearching = true;
		showDropdown = true;
		try {
			const allResults = await Promise.all(
				providers.map(p => p.searchRepositories(searchQuery))
			);
			searchResults = allResults.flat();
		} catch (error) {
			console.error('Error searching repos:', error);
			searchResults = [];
		} finally {
			isSearching = false;
		}
	}

	function onInput() {
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(handleSearch, 300) as unknown as number;
	}

	function selectRepo(repo: Repository) {
		if (!selectedRepos.find(r => r.fullName === repo.fullName)) {
			selectedRepos = [...selectedRepos, repo];
		}
		searchQuery = '';
		searchResults = [];
		showDropdown = false;
	}

	function removeRepo(repo: Repository) {
		selectedRepos = selectedRepos.filter(r => r.fullName !== repo.fullName);
	}

	function handleBlur() {
		setTimeout(() => {
			showDropdown = false;
		}, 200);
	}

	let filteredResults = $derived(
		searchResults.filter(r => !selectedRepos.find(s => s.fullName === r.fullName))
	);
</script>

<div class="relative">
	{#if providers.length === 0}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 text-yellow-800">
			<p class="font-semibold">No repository providers configured</p>
			<p class="text-sm mt-1">Set <code class="bg-yellow-100 px-2 py-0.5 rounded">VITE_MAESTRO_GITHUB_TOKEN</code> environment variable to enable GitHub integration</p>
		</div>
	{:else}
		<div class="flex flex-wrap gap-2 mb-4">
			{#each selectedRepos as repo}
				<div class="flex items-center gap-2 bg-gradient-to-r from-blue-500 to-indigo-500 text-white px-4 py-2 rounded-lg shadow-md hover:shadow-lg transition-all">
					<span class="text-sm font-semibold">{repo.fullName}</span>
					<button
						onclick={() => removeRepo(repo)}
						class="hover:bg-white/20 rounded-full p-1 transition-all"
						aria-label="Remove {repo.fullName}"
					>
						✕
					</button>
				</div>
			{/each}
		</div>

		<input
			bind:this={inputElement}
			bind:value={searchQuery}
			oninput={onInput}
			onblur={handleBlur}
			type="text"
			placeholder="Search repositories..."
			class="w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition-all"
		/>

		{#if showDropdown && filteredResults.length > 0}
			<div class="absolute z-10 w-full mt-2 bg-white border-2 border-gray-200 rounded-xl shadow-2xl max-h-80 overflow-y-auto">
				{#if isSearching}
					<div class="px-5 py-4 text-gray-500 text-sm">
						Searching...
					</div>
				{:else}
					{#each filteredResults as repo}
						<button
							onclick={() => selectRepo(repo)}
							class="w-full px-5 py-3.5 text-left hover:bg-gradient-to-r hover:from-blue-50 hover:to-indigo-50 border-b border-gray-100 last:border-b-0 transition-all"
						>
							<div class="font-semibold text-gray-900">
								{repo.fullName}
							</div>
							{#if repo.description}
								<div class="text-sm text-gray-600 mt-1 truncate">{repo.description}</div>
							{/if}
						</button>
					{/each}
				{/if}
			</div>
		{/if}
	{/if}
</div>
