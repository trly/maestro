<script lang="ts">
	import { Combobox } from 'bits-ui';
	import { getConfiguredProviders, type Repository } from '../providers';
	import { onMount, onDestroy } from 'svelte';
	import { Search, Check, Globe } from 'lucide-svelte';
	import { tokenStore } from '$lib/tokenStore';
	import * as ipc from '$lib/ipc';
	import type { SourcegraphRepository } from '$lib/ipc';

	let {
		selectedRepos = $bindable([])
	}: {
		selectedRepos: Repository[];
	} = $props();

	let searchQuery = $state('');
	let searchResults = $state<Repository[]>([]);
	let isSearching = $state(false);
	let open = $state(false);
	let debounceTimer: number;

	let sgQuery = $state('');
	let sgResults = $state<Repository[]>([]);
	let isSgSearching = $state(false);
	let sgOpen = $state(false);
	let sgDebounceTimer: number;

	let providers = $state<Awaited<ReturnType<typeof getConfiguredProviders>>>([]);
	let hasSgConfig = $state(false);

	onMount(async () => {
		providers = await getConfiguredProviders();
		const tokens = await tokenStore.getAllTokens();
		hasSgConfig = !!(tokens.sourcegraphEndpoint && tokens.sourcegraphToken);
	});

	onDestroy(() => {
		clearTimeout(debounceTimer);
		clearTimeout(sgDebounceTimer);
	});

	// Clear search when dropdown closes
	$effect(() => {
		if (!open && searchQuery) {
			searchQuery = '';
		}
	});

	$effect(() => {
		if (!sgOpen && sgQuery) {
			sgQuery = '';
		}
	});

	async function handleSearch(query: string) {
		if (providers.length === 0 || !query.trim()) {
			searchResults = [];
			open = false;
			return;
		}

		isSearching = true;
		open = true;
		try {
			const allResults = await Promise.all(
				providers.map(p => p.searchRepositories(query))
			);
			searchResults = allResults.flat();
		} catch (error) {
			searchResults = [];
		} finally {
			isSearching = false;
		}
	}

	async function handleSgSearch(query: string) {
		if (!query.trim()) {
			sgResults = [];
			sgOpen = false;
			return;
		}

		isSgSearching = true;
		sgOpen = true;
		try {
			const result = await ipc.searchSourcegraphRepositories(query, 20);
			
			// Convert to Repository format
			sgResults = result.repositories.map((r: SourcegraphRepository) => {
				// Sourcegraph returns names like "github.com/owner/repo", strip the prefix
				const repoPath = r.name.replace(/^github\.com\//, '');
				const parts = repoPath.split('/');
				const owner = parts[0] || '';
				const name = parts[1] || repoPath;
				return {
					provider: 'github' as const,
					fullName: repoPath, // "owner/repo"
					name: name,         // "repo"
					owner: owner,       // "owner"
					url: r.url,
					description: r.description || ''
				};
			});
		} catch (error) {
			console.error('Sourcegraph search failed:', error);
			sgResults = [];
		} finally {
			isSgSearching = false;
		}
	}

	function onInput(value: string) {
		searchQuery = value;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => handleSearch(value), 300) as unknown as number;
	}

	function onSgInput(value: string) {
		sgQuery = value;
		clearTimeout(sgDebounceTimer);
		sgDebounceTimer = setTimeout(() => handleSgSearch(value), 300) as unknown as number;
	}

	function selectRepo(repo: Repository) {
		if (!selectedRepos.find(r => r.fullName === repo.fullName)) {
			selectedRepos = [...selectedRepos, repo];
		}
		searchResults = [];
		open = false;
	}

	function selectSgRepo(repo: Repository) {
		if (!selectedRepos.find(r => r.fullName === repo.fullName)) {
			selectedRepos = [...selectedRepos, repo];
		}
		sgResults = [];
		sgOpen = false;
	}

	function addAllSgResults() {
		const newRepos = filteredSgResults.filter(
			r => !selectedRepos.find(s => s.fullName === r.fullName)
		);
		selectedRepos = [...selectedRepos, ...newRepos];
		sgResults = [];
		sgOpen = false;
	}

	function removeRepo(repo: Repository) {
		selectedRepos = selectedRepos.filter(r => r.fullName !== repo.fullName);
	}

	function clearAll() {
		selectedRepos = [];
	}

	let filteredResults = $derived(
		searchResults.filter(r => !selectedRepos.find(s => s.fullName === r.fullName))
	);

	let filteredSgResults = $derived(
		sgResults.filter(r => !selectedRepos.find(s => s.fullName === r.fullName))
	);
</script>

<div>
	{#if providers.length === 0}
		<div class="bg-card border border-border/30 rounded-lg p-4">
			<p class="font-semibold text-foreground">No repository providers configured</p>
			<p class="text-sm mt-1 text-muted-foreground">Configure GitHub token in Settings to enable GitHub integration</p>
		</div>
	{:else}
		<div class="flex items-center justify-between gap-2 mb-2">
			{#if selectedRepos.length > 0}
				<span class="text-sm text-muted-foreground">{selectedRepos.length} {selectedRepos.length === 1 ? 'repository' : 'repositories'} selected</span>
				<button
					onclick={clearAll}
					class="text-sm text-destructive hover:text-destructive/80 transition-colors"
				>
					Clear all
				</button>
			{/if}
		</div>
		<div class="flex flex-wrap gap-2 mb-4">
			{#each selectedRepos as repo}
				<div class="flex items-center gap-1.5 bg-primary text-primary-foreground px-2.5 py-1 rounded-lg transition-all">
					<span class="text-sm font-semibold">{repo.fullName}</span>
					<button
						onclick={() => removeRepo(repo)}
						class="hover:bg-primary-foreground/20 rounded-full p-0.5 transition-all"
						aria-label="Remove {repo.fullName}"
					>
						✕
					</button>
				</div>
			{/each}
		</div>

		<Combobox.Root 
			type="single" 
			bind:open 
			inputValue={searchQuery}
			onValueChange={(val) => {
				if (!val) return;
				const repo = searchResults.find(r => r.fullName === val);
				if (repo) selectRepo(repo);
			}}
		>
			<div class="relative">
				<Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground pointer-events-none" />
				<Combobox.Input
					placeholder="Search repositories..."
					class="w-full h-10 pl-10 pr-4 border border-border/30 rounded-lg bg-background text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:border-border transition-all"
					oninput={(e) => onInput((e.currentTarget as HTMLInputElement).value)}
				/>
			</div>
			<Combobox.Content
				class="z-50 bg-background border border-border/30 rounded-xl shadow-lg max-h-96 overflow-hidden p-1"
				style="width: var(--bits-combobox-anchor-width);"
				sideOffset={8}
			>
				<div class="max-h-80 overflow-y-auto p-1">
					{#if isSearching}
						<div class="px-5 py-8 text-center text-muted-foreground text-sm">
							Searching...
						</div>
					{:else if filteredResults.length === 0 && searchQuery.trim()}
						<div class="px-5 py-8 text-center text-muted-foreground text-sm">
							No repositories found
						</div>
					{:else}
						{#each filteredResults as repo}
							<Combobox.Item
								value={repo.fullName}
								class="flex h-10 cursor-pointer items-center gap-3 rounded-lg px-3 py-2 data-[highlighted]:bg-muted transition-colors"
							>
								<div class="flex-1 min-w-0 font-medium text-sm text-foreground truncate">
									{repo.fullName}
								</div>
								{#if selectedRepos.find(r => r.fullName === repo.fullName)}
									<Check class="size-4 shrink-0 text-primary" />
								{/if}
							</Combobox.Item>
						{/each}
					{/if}
				</div>
			</Combobox.Content>
		</Combobox.Root>

		{#if hasSgConfig}
			<div class="mt-6">
				<div class="flex items-center gap-2 mb-3">
					<Globe class="size-4 text-muted-foreground" />
					<h3 class="text-sm font-semibold text-foreground">Search via Sourcegraph</h3>
				</div>
				<Combobox.Root 
					type="single" 
					bind:open={sgOpen}
					inputValue={sgQuery}
					onValueChange={(val) => {
						if (!val) return;
						const repo = sgResults.find(r => r.fullName === val);
						if (repo) selectSgRepo(repo);
					}}
				>
					<div class="relative">
						<Search class="absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground pointer-events-none" />
						<Combobox.Input
							placeholder="Sourcegraph query (filters to GitHub repos only)..."
							class="w-full h-10 pl-10 pr-4 border border-border/30 rounded-lg bg-background text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:border-border transition-all"
							oninput={(e) => onSgInput((e.currentTarget as HTMLInputElement).value)}
						/>
					</div>
					<Combobox.Content
						class="z-50 bg-background border border-border/30 rounded-xl shadow-lg max-h-96 overflow-hidden p-1"
						style="width: var(--bits-combobox-anchor-width);"
						sideOffset={8}
					>
						<div class="max-h-80 overflow-y-auto p-1">
						{#if isSgSearching}
						<div class="px-5 py-8 text-center text-muted-foreground text-sm">
						Searching Sourcegraph...
						</div>
						{:else if filteredSgResults.length === 0 && sgQuery.trim()}
						<div class="px-5 py-8 text-center text-muted-foreground text-sm">
						No GitHub repositories found
						</div>
						{:else if filteredSgResults.length > 0}
						<div class="p-2 border-b border-border/30">
						<button
						onclick={addAllSgResults}
						disabled={isSgSearching}
						class="w-full h-9 px-4 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed"
						>
						Add all {filteredSgResults.length} {filteredSgResults.length === 1 ? 'repository' : 'repositories'}
						</button>
						</div>
						{#each filteredSgResults as repo}
						<Combobox.Item
						value={repo.fullName}
						 class="flex h-10 cursor-pointer items-center gap-3 rounded-lg px-3 py-2 data-[highlighted]:bg-muted transition-colors"
						 >
						   <div class="flex-1 min-w-0 font-medium text-sm text-foreground truncate">
						     {repo.fullName}
								</div>
								{#if selectedRepos.find(r => r.fullName === repo.fullName)}
									<Check class="size-4 shrink-0 text-primary" />
								{/if}
							</Combobox.Item>
						{/each}
					{/if}
				</div>
					</Combobox.Content>
				</Combobox.Root>
			</div>
		{/if}
	{/if}
</div>
