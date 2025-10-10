<script lang="ts">
	import { Checkbox } from 'bits-ui';
	import { Check, Minus } from 'lucide-svelte';
	import type { Repository as DBRepository } from '../../types';

	interface Props {
		repositories: Map<string, DBRepository>;
		repositoryIds: string[];
		selectedIds?: string[];
		onSelect: (ids: string[]) => void;
	}

	let { repositories, repositoryIds, selectedIds = [], onSelect }: Props = $props();

	let selected = $state<Set<string>>(new Set(selectedIds.length > 0 ? selectedIds : repositoryIds));

	let selectAllChecked = $derived(selected.size === repositoryIds.length);
	let selectAllIndeterminate = $derived(selected.size > 0 && selected.size < repositoryIds.length);

	function toggleRepository(id: string) {
		if (selected.has(id)) {
			selected.delete(id);
		} else {
			selected.add(id);
		}
		onSelect(Array.from(selected));
	}

	function handleSelectAllChange(checked: boolean) {
		if (checked) {
			selected = new Set(repositoryIds);
		} else {
			selected.clear();
		}
		onSelect(Array.from(selected));
	}
</script>

<div class="space-y-3">
	<div class="flex items-center justify-between mb-4">
		<h4 class="text-sm font-semibold text-foreground">Select Repositories</h4>
		<label for="select-all-repos" class="flex items-center gap-2 cursor-pointer">
			<Checkbox.Root
				id="select-all-repos"
				checked={selectAllChecked}
				indeterminate={selectAllIndeterminate}
				onCheckedChange={handleSelectAllChange}
				class="size-4 rounded border border-border data-[state=checked]:bg-primary data-[state=indeterminate]:bg-primary flex items-center justify-center"
			>
				{#snippet children({ checked, indeterminate })}
					{#if indeterminate}
						<Minus class="size-3 text-primary-foreground" />
					{:else if checked}
						<Check class="size-3 text-primary-foreground" />
					{/if}
				{/snippet}
			</Checkbox.Root>
			<span class="text-sm text-primary hover:underline">
				{selected.size === repositoryIds.length ? 'Deselect All' : 'Select All'}
			</span>
		</label>
	</div>
	
	<div class="max-h-96 overflow-y-auto space-y-2">
		{#each repositoryIds as repoId}
			{@const repo = repositories.get(repoId)}
			{#if repo}
				<label for="repo-{repoId}" class="flex items-center gap-3 p-3 border border-border rounded-lg hover:bg-accent/50 cursor-pointer transition-all">
					<Checkbox.Root
						id="repo-{repoId}"
						checked={selected.has(repoId)}
						onCheckedChange={() => toggleRepository(repoId)}
						class="size-4 rounded border border-border data-[state=checked]:bg-primary flex items-center justify-center"
					>
						{#snippet children({ checked })}
							{#if checked}
								<Check class="size-3 text-primary-foreground" />
							{/if}
						{/snippet}
					</Checkbox.Root>
					<span class="text-sm text-foreground">{repo.providerId}</span>
				</label>
			{/if}
		{/each}
	</div>
	
	<div class="pt-3 border-t border-border text-sm text-muted-foreground">
		{selected.size} of {repositoryIds.length} repositories selected
	</div>
</div>
