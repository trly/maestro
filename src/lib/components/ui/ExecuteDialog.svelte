<script lang="ts">
	import Modal from './Modal.svelte';
	import RepositoryCheckboxList from './RepositoryCheckboxList.svelte';
	import type { Repository as DBRepository } from '../../types';
	import type { Snippet } from 'svelte';

	interface Props {
		repositories: Map<string, DBRepository>;
		repositoryIds: string[];
		isOpen: boolean;
		isExecuting?: boolean;
		onClose: () => void;
		onExecute: (selectedRepoIds: string[]) => void;
	}

	let { repositories, repositoryIds, isOpen = $bindable(), isExecuting = false, onClose, onExecute }: Props = $props();

	let selectedRepoIds = $state<string[]>(repositoryIds);

	function handleExecute() {
		if (selectedRepoIds.length === 0) return;
		onExecute(selectedRepoIds);
	}

	function handleCancel() {
		isOpen = false;
		onClose();
	}

	$effect(() => {
		if (isOpen) {
			selectedRepoIds = repositoryIds;
		}
	});
</script>

<Modal bind:open={isOpen} title="Execute Revision">
	{#snippet children()}
		<div class="p-6">
			<RepositoryCheckboxList
				{repositories}
				{repositoryIds}
				selectedIds={selectedRepoIds}
				onSelect={(ids) => selectedRepoIds = ids}
			/>
		</div>
	{/snippet}
	
	{#snippet footer()}
		<div class="flex gap-3">
			<button
				onclick={handleExecute}
				disabled={selectedRepoIds.length === 0 || isExecuting}
				class="flex-1 px-6 py-3 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
			>
				{isExecuting ? 'Executing...' : `Execute on ${selectedRepoIds.length} ${selectedRepoIds.length === 1 ? 'Repository' : 'Repositories'}`}
			</button>
			<button
				onclick={handleCancel}
				disabled={isExecuting}
				class="px-6 py-3 border border-border text-foreground rounded-md hover:bg-accent transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
			>
				Cancel
			</button>
		</div>
	{/snippet}
</Modal>
