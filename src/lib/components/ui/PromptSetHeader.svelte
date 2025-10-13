<script lang="ts">
	import { Pencil, GitBranch, Search } from 'lucide-svelte';
	import { Switch, Dialog } from 'bits-ui';
	import type { PromptSet, Repository } from '$lib/types';
	import * as ipc from '$lib/ipc';
	import { showToast } from '$lib/ui/toast';

	let {
		promptSet,
		repositories,
		onEditRepos,
		onEditValidation
	}: {
		promptSet: PromptSet;
		repositories: Map<string, Repository>;
		onEditRepos: () => void;
		onEditValidation: () => void;
	} = $props();

	let autoValidate = $state(promptSet.autoValidate);
	let showRepoDialog = $state(false);
	let filterText = $state('');

	async function handleAutoValidateChange(checked: boolean) {
		try {
			await ipc.updatePromptSetAutoValidate(promptSet.id, checked);
			promptSet.autoValidate = checked;
		} catch (err) {
			showToast('Failed to update auto-validate setting: ' + err, 'error');
			// Revert on error
			autoValidate = promptSet.autoValidate;
		}
	}

	$effect(() => {
		autoValidate = promptSet.autoValidate;
	});

	let repoCount = $derived(promptSet.repositoryIds.length);
	
	let repoList = $derived(
		promptSet.repositoryIds
			.map(id => repositories.get(id))
			.filter((repo): repo is Repository => repo !== undefined)
	);
	
	let filteredRepos = $derived(
		filterText
			? repoList.filter(repo => 
				repo.providerId.toLowerCase().includes(filterText.toLowerCase())
			)
			: repoList
	);
</script>

<div class="flex-shrink-0 bg-card h-full">
	<div class="px-4 py-3 h-full flex flex-col justify-center gap-2">
		<h1 class="text-sm font-bold text-foreground">{promptSet.name}</h1>
		
		<div class="flex items-center gap-3 text-xs">
			<!-- Repositories -->
			<div class="flex items-center gap-2 min-w-0 flex-1">
				<GitBranch class="w-4 h-4 text-muted-foreground flex-shrink-0" />
				<button
					onclick={() => showRepoDialog = true}
					class="text-muted-foreground hover:text-foreground transition-colors underline decoration-dotted"
				>
					{repoCount} {repoCount === 1 ? 'repository' : 'repositories'}
				</button>
				<button
					onclick={onEditRepos}
					class="w-6 h-6 flex items-center justify-center rounded hover:bg-muted transition-colors flex-shrink-0"
					aria-label="Edit repositories"
				>
					<Pencil class="w-3 h-3 text-muted-foreground" />
				</button>
			</div>


		</div>
	</div>
</div>

<Dialog.Root bind:open={showRepoDialog}>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
		<Dialog.Content class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] max-h-[600px] bg-card border border-border/30 rounded-lg shadow-lg z-50 flex flex-col">
		<div class="px-6 py-4 border-b border-border/10">
				<Dialog.Title class="text-lg font-semibold">
					Repositories ({repoCount})
				</Dialog.Title>
			</div>
			
			<div class="px-6 py-3 border-b border-border/10">
				<div class="relative">
					<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
					<input
						type="text"
						bind:value={filterText}
						placeholder="Filter repositories..."
						class="w-full pl-9 pr-3 py-2 bg-background border border-border/30 rounded text-sm focus:outline-none focus:ring-2 focus:ring-ring"
					/>
				</div>
			</div>
			
			<div class="flex-1 overflow-y-auto px-6 py-3">
				{#if filteredRepos.length === 0}
					<p class="text-sm text-muted-foreground text-center py-8">
						{filterText ? 'No repositories match your filter' : 'No repositories'}
					</p>
				{:else}
					<div class="space-y-2">
						{#each filteredRepos as repo (repo.id)}
							<div class="flex items-center gap-2 px-3 py-2 rounded bg-muted/50">
								<GitBranch class="w-4 h-4 text-muted-foreground flex-shrink-0" />
								<span class="text-sm font-medium">{repo.providerId}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
			
			<div class="px-6 py-4 border-t border-border/10 flex justify-between">
				<button
					onclick={() => {
						showRepoDialog = false;
						onEditRepos();
					}}
					class="px-4 py-2 border border-border/30 rounded hover:bg-muted transition-colors text-sm font-medium flex items-center gap-2"
				>
					<Pencil class="w-3.5 h-3.5" />
					Edit Repositories
				</button>
				<Dialog.Close class="px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90 transition-colors text-sm font-medium">
					Close
				</Dialog.Close>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
