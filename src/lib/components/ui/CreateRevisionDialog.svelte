<script lang="ts">
	import { Dialog } from 'bits-ui';
	import PromptDiff from '$lib/components/PromptDiff.svelte';

	let {
		open = $bindable(false),
		parentPromptText = '',
		onSave
	}: {
		open?: boolean;
		parentPromptText?: string;
		onSave: (prompt: string) => void;
	} = $props();

	let localPrompt = $state(parentPromptText);

	// Reset to parent prompt text when dialog opens
	$effect(() => {
		if (open) {
			localPrompt = parentPromptText;
		}
	});

	function handleSave() {
		if (!localPrompt.trim()) {
			return;
		}
		onSave(localPrompt);
		open = false;
	}

	function handleCancel() {
		open = false;
	}

	function handleUpdate(text: string) {
		localPrompt = text;
	}
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
		<Dialog.Content
			class="fixed left-1/2 top-1/2 z-50 w-full max-w-6xl -translate-x-1/2 -translate-y-1/2 bg-background rounded-lg shadow-2xl border border-border/30 flex flex-col max-h-[85vh]"
		>
			<Dialog.Title class="px-6 py-4 border-b border-border/10 flex-shrink-0">
				<h2 class="text-xl font-bold text-foreground">Create New Revision</h2>
				<p class="text-sm text-muted-foreground mt-1">
					Edit the prompt on the left. Changes from the parent revision are highlighted on the right.
				</p>
			</Dialog.Title>
			
			<div class="flex-1 min-h-0 overflow-hidden px-6 py-4">
				<PromptDiff
					oldText={parentPromptText}
					newText={localPrompt}
					onupdate={handleUpdate}
				/>
			</div>
			
			<div class="px-6 py-4 border-t border-border/10 flex-shrink-0 flex justify-end gap-3">
				<button
					onclick={handleCancel}
					class="px-4 py-2 rounded-md border border-border hover:bg-accent transition-colors"
				>
					Cancel
				</button>
				<button
					onclick={handleSave}
					disabled={!localPrompt.trim()}
					class="px-4 py-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					Create Revision
				</button>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
