<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { X } from 'lucide-svelte';
	import RepositorySelector from '$lib/components/RepositorySelector.svelte';
	import type { Repository } from '$lib/providers/types';

	let {
		open = $bindable(false),
		currentRepos,
		onSave
	}: {
		open?: boolean;
		currentRepos: Repository[];
		onSave: (repos: Repository[]) => Promise<void>;
	} = $props();

	let selectedRepos = $state<Repository[]>([]);
	let saving = $state(false);

	async function handleSave() {
		saving = true;
		try {
			await onSave(selectedRepos);
			open = false;
		} finally {
			saving = false;
		}
	}

	$effect(() => {
		if (open) {
			selectedRepos = [...currentRepos];
		}
	});
</script>

<Dialog.Root bind:open={open}>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 bg-black/50 z-40" />
		<Dialog.Content class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-background border border-border/30 rounded-lg shadow-xl z-50 w-full max-w-2xl max-h-[80vh] flex flex-col">
		<div class="flex items-center justify-between p-6 border-b border-border/10">
				<Dialog.Title class="text-lg font-semibold">Edit Repositories</Dialog.Title>
				<Dialog.Close class="w-8 h-8 flex items-center justify-center rounded-md hover:bg-muted transition-colors">
					<X class="w-4 h-4" />
				</Dialog.Close>
			</div>

			<div class="p-6 flex-1 overflow-auto">
				<RepositorySelector bind:selectedRepos={selectedRepos} />
			</div>

			<div class="flex justify-end gap-3 p-6 border-t border-border/10">
			<Dialog.Close class="px-4 py-2 border border-border/30 rounded-md hover:bg-muted transition-colors">
					Cancel
				</Dialog.Close>
				<button
					onclick={handleSave}
					disabled={saving || selectedRepos.length === 0}
					class="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all disabled:opacity-50"
				>
					{saving ? 'Saving...' : 'Save'}
				</button>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
