<script lang="ts">
	import { Dialog, Switch } from 'bits-ui';
	import { X } from 'lucide-svelte';

	let {
		open = $bindable(false),
		validationPrompt,
		autoValidate,
		onSave
	}: {
		open?: boolean;
		validationPrompt: string;
		autoValidate: boolean;
		onSave: (prompt: string, autoValidate: boolean) => Promise<void>;
	} = $props();

	let editedPrompt = $state(validationPrompt);
	let editedAutoValidate = $state(autoValidate);
	let saving = $state(false);

	async function handleSave() {
		saving = true;
		try {
			await onSave(editedPrompt, editedAutoValidate);
			open = false;
		} finally {
			saving = false;
		}
	}

	$effect(() => {
		if (open) {
			editedPrompt = validationPrompt;
			editedAutoValidate = autoValidate;
		}
	});
</script>

<Dialog.Root bind:open={open}>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 bg-black/50 z-40" />
		<Dialog.Content class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-background border border-border rounded-lg shadow-xl z-50 w-full max-w-2xl max-h-[80vh] flex flex-col">
			<div class="flex items-center justify-between p-6 border-b border-border">
				<Dialog.Title class="text-lg font-semibold">Edit Validation Prompt</Dialog.Title>
				<Dialog.Close class="w-8 h-8 flex items-center justify-center rounded-md hover:bg-muted transition-colors">
					<X class="w-4 h-4" />
				</Dialog.Close>
			</div>

			<div class="p-6 flex-1 overflow-auto">
				<div class="space-y-4">
					<div>
						<label for="validation-prompt" class="block text-sm font-medium mb-2">Validation Prompt</label>
						<textarea
							id="validation-prompt"
							bind:value={editedPrompt}
							placeholder="Enter a prompt to validate each execution after it completes..."
							rows="12"
							class="w-full px-4 py-3 border-2 border-border rounded-lg focus:ring-2 focus:ring-primary focus:border-primary transition-all resize-y font-mono text-sm"
						></textarea>
					</div>
					<p class="text-xs text-muted-foreground">
						This prompt will run against the modified branch after each successful execution.
						Leave empty to disable validation.
					</p>

					<!-- Auto-validate Toggle -->
					<div class="flex items-center justify-between p-4 bg-muted/30 rounded-lg border border-border">
						<div class="flex-1">
							<label for="auto-validate" class="block text-sm font-medium mb-1">Automatic Validation</label>
							<p class="text-xs text-muted-foreground">
								Run validation automatically after each execution completes successfully
							</p>
						</div>
						<Switch.Root
							id="auto-validate"
							bind:checked={editedAutoValidate}
							class="data-[state=checked]:bg-green-600 data-[state=unchecked]:bg-muted inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
						>
							<Switch.Thumb
								class="pointer-events-none block h-4 w-4 rounded-full bg-white transition-transform data-[state=checked]:translate-x-[18px] data-[state=unchecked]:translate-x-0"
							/>
						</Switch.Root>
					</div>
				</div>
			</div>

			<div class="flex justify-end gap-3 p-6 border-t border-border">
				<Dialog.Close class="px-4 py-2 border border-border rounded-md hover:bg-muted transition-colors">
					Cancel
				</Dialog.Close>
				<button
					onclick={handleSave}
					disabled={saving}
					class="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all disabled:opacity-50"
				>
					{saving ? 'Saving...' : 'Save'}
				</button>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
