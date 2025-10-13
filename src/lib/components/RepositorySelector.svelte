<script lang="ts">
	import { Combobox } from 'bits-ui';
	import { getConfiguredProviders, type Repository } from '../providers';
	import { onMount } from 'svelte';
	import { Search, Check } from 'lucide-svelte';

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

	let providers = $state<Awaited<ReturnType<typeof getConfiguredProviders>>>([]);

	onMount(async () => {
		providers = await getConfiguredProviders();
	});

	// Clear search when dropdown closes
	$effect(() => {
		if (!open && searchQuery) {
			searchQuery = '';
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

	function onInput(value: string) {
		searchQuery = value;
		clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => handleSearch(value), 300) as unknown as number;
	}

	function selectRepo(repo: Repository) {
		if (!selectedRepos.find(r => r.fullName === repo.fullName)) {
			selectedRepos = [...selectedRepos, repo];
		}
		searchResults = [];
		open = false;
	}

	function removeRepo(repo: Repository) {
		selectedRepos = selectedRepos.filter(r => r.fullName !== repo.fullName);
	}

	let filteredResults = $derived(
		searchResults.filter(r => !selectedRepos.find(s => s.fullName === r.fullName))
	);
</script>

<div>
	{#if providers.length === 0}
		<div class="bg-card border border-border/30 rounded-lg p-4">
			<p class="font-semibold text-foreground">No repository providers configured</p>
			<p class="text-sm mt-1 text-muted-foreground">Configure GitHub token in Settings to enable GitHub integration</p>
		</div>
	{:else}
		<div class="flex flex-wrap gap-2 mb-4">
			{#each selectedRepos as repo}
				<div class="flex items-center gap-2 bg-primary text-primary-foreground px-4 py-2 rounded-lg transition-all">
					<span class="text-sm font-semibold">{repo.fullName}</span>
					<button
						onclick={() => removeRepo(repo)}
						class="hover:bg-primary-foreground/20 rounded-full p-1 transition-all"
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
	{/if}
</div>
