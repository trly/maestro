<script lang="ts">
	import PromptDiff from "$lib/components/PromptDiff.svelte"
	import FormField from "$lib/components/ui/FormField.svelte"
	import UiTextarea from "$lib/components/ui/UiTextarea.svelte"

	let {
		promptText = $bindable(""),
		oldPromptText = null,
		validationPromptText = $bindable(""),
		showValidationPrompt = false,
		isProcessing = false,
		hasChanges = true,
		onCreateOnly,
		onCreateAndExecute,
		onCancel,
	} = $props<{
		promptText: string
		oldPromptText?: string | null
		validationPromptText?: string
		showValidationPrompt?: boolean
		isProcessing?: boolean
		hasChanges?: boolean
		onCreateOnly: () => void
		onCreateAndExecute: () => void
		onCancel: () => void
	}>()

	let isDisabled = $derived(isProcessing || !promptText.trim() || !hasChanges)
</script>

<div class="space-y-5">
	<!-- Prompt Editor -->
	{#if oldPromptText}
		<PromptDiff
			oldText={oldPromptText}
			newText={promptText}
			onupdate={(text) => (promptText = text)}
		/>
	{:else}
		<FormField
			label="Prompt"
			htmlFor="prompt-text"
			required
			helperText="This prompt will be executed across all repositories in the set"
		>
			<UiTextarea
				id="prompt-text"
				bind:value={promptText}
				placeholder="Enter your prompt to execute across all repositories..."
				rows={12}
			/>
		</FormField>
	{/if}

	<!-- Validation Prompt (optional) -->
	{#if showValidationPrompt}
		<FormField
			label="Validation Prompt"
			htmlFor="validation-prompt"
			helperText="This prompt will run against the modified branch after each successful execution"
		>
			<UiTextarea
				id="validation-prompt"
				bind:value={validationPromptText}
				placeholder="Enter a prompt to validate each execution after it completes..."
				rows={6}
			/>
		</FormField>
	{/if}

	<!-- Action Buttons -->
	<div class="flex flex-col sm:flex-row gap-3">
		<button
			onclick={onCreateOnly}
			disabled={isDisabled}
			class="w-full sm:w-auto px-6 py-3.5 border border-primary/50 text-primary rounded-md hover:bg-primary/10 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
			title={!hasChanges ? "No changes to prompt text" : ""}
		>
			{isProcessing ? "Creating..." : "Create Only"}
		</button>
		<button
			onclick={onCreateAndExecute}
			disabled={isDisabled}
			class="flex-1 px-8 py-3.5 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
			title={!hasChanges ? "No changes to prompt text" : ""}
		>
			{isProcessing ? "Creating..." : "Create & Execute"}
		</button>
		<button
			onclick={onCancel}
			disabled={isProcessing}
			class="w-full sm:w-auto px-6 py-3.5 border border-border/30 text-foreground rounded-md hover:bg-accent transition-all font-semibold"
		>
			Cancel
		</button>
	</div>
</div>
