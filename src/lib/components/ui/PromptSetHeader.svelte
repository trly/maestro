<script lang="ts">
	import { Pencil, GitBranch } from 'lucide-svelte';
	import { Switch } from 'bits-ui';
	import UiTooltip from './UiTooltip.svelte';
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

	let repoList = $derived(
		promptSet.repositoryIds.map(id => {
			const repo = repositories.get(id);
			return {
				id,
				name: repo?.providerId || '...',
				loading: !repo
			};
		})
	);

	let displayRepos = $derived(repoList.slice(0, 3));
	let remainingCount = $derived(Math.max(0, repoList.length - 3));
	let allRepoNames = $derived(repoList.map(r => r.name).join(', '));
</script>

<div class="flex-shrink-0 bg-card">
	<div class="px-6 py-4">
		<div class="flex items-start justify-between mb-3">
			<h1 class="text-2xl font-bold text-foreground">{promptSet.name}</h1>
		</div>
		
		<div class="flex flex-col @md:flex-row @md:items-center gap-3 @md:gap-6 text-sm">
			<!-- Repositories -->
			<div class="flex items-center gap-2 min-w-0 flex-1">
				<GitBranch class="w-4 h-4 text-muted-foreground flex-shrink-0" />
				<div class="flex items-center gap-1.5 min-w-0 flex-wrap">
					{#each displayRepos as repo}
						<span class="px-2 py-0.5 bg-muted rounded text-xs font-medium text-foreground">
							{repo.name}
						</span>
					{/each}
					{#if remainingCount > 0}
						<UiTooltip content={allRepoNames}>
							{#snippet children({ props })}
								<span 
									{...props}
									class="px-2 py-0.5 bg-muted/50 rounded text-xs text-muted-foreground cursor-help"
								>
									+{remainingCount} more
								</span>
							{/snippet}
						</UiTooltip>
					{/if}
				</div>
				<button
					onclick={onEditRepos}
					class="w-6 h-6 flex items-center justify-center rounded hover:bg-muted transition-colors flex-shrink-0"
					aria-label="Edit repositories"
				>
					<Pencil class="w-3 h-3 text-muted-foreground" />
				</button>
			</div>

			<!-- Validation Toggle -->
			<div class="flex items-center gap-3 flex-shrink-0">
				<span class="text-sm text-muted-foreground">Auto-validate</span>
				<UiTooltip content={autoValidate ? 'Disable automatic validation after execution' : 'Enable automatic validation after execution'}>
					{#snippet children({ props })}
						<div {...props}>
							<Switch.Root
								bind:checked={autoValidate}
								onCheckedChange={handleAutoValidateChange}
								class="data-[state=checked]:bg-green-600 data-[state=unchecked]:bg-muted inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
								aria-label={autoValidate ? 'Disable auto-validate' : 'Enable auto-validate'}
							>
								<Switch.Thumb
									class="pointer-events-none block h-4 w-4 rounded-full bg-white transition-transform data-[state=checked]:translate-x-[18px] data-[state=unchecked]:translate-x-0"
								/>
							</Switch.Root>
						</div>
					{/snippet}
				</UiTooltip>
				<button
					onclick={onEditValidation}
					class="w-6 h-6 flex items-center justify-center rounded hover:bg-muted transition-colors"
					aria-label="Edit validation prompt"
				>
					<Pencil class="w-3 h-3 text-muted-foreground" />
				</button>
			</div>
		</div>
	</div>
</div>
