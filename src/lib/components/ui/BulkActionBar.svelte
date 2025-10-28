<script lang="ts">
	import { Trash2, Play, RotateCw, Check, CheckCheck } from "lucide-svelte"
	import IconButton from "./IconButton.svelte"

	let {
		selectedCount,
		hasValidationPrompt = false,
		onBulkDelete,
		onBulkStart,
		onBulkRestart,
		onBulkStartValidations,
		onBulkRevalidate,
		onClear,
		isStarting = false,
		isRestarting = false,
		isValidating = false,
		isRevalidating = false,
		isDeleting = false,
	}: {
		selectedCount: number
		hasValidationPrompt?: boolean
		onBulkDelete: () => void
		onBulkStart: () => void
		onBulkRestart: () => void
		onBulkStartValidations?: () => void
		onBulkRevalidate?: () => void
		onClear: () => void
		isStarting?: boolean
		isRestarting?: boolean
		isValidating?: boolean
		isRevalidating?: boolean
		isDeleting?: boolean
	} = $props()

	// Determine if any operation is running
	let anyOperationRunning = $derived(
		isStarting || isRestarting || isValidating || isRevalidating || isDeleting
	)
</script>

<div class="flex items-center justify-between px-4 py-2 bg-primary/10 border-b border-border/20">
	<span class="text-sm font-medium text-foreground">
		{selectedCount} selected
	</span>

	<div class="flex items-center gap-2 ml-12">
		<IconButton
			icon={Play}
			tooltip="Start"
			onclick={onBulkStart}
			variant="success"
			disabled={anyOperationRunning}
			loading={isStarting}
		/>
		<IconButton
			icon={RotateCw}
			tooltip="Restart"
			onclick={onBulkRestart}
			variant="primary"
			disabled={anyOperationRunning}
			loading={isRestarting}
		/>
		{#if hasValidationPrompt && onBulkStartValidations}
			<IconButton
				icon={Check}
				tooltip="Start validation"
				onclick={onBulkStartValidations}
				variant="success"
				disabled={anyOperationRunning}
				loading={isValidating}
			/>
		{/if}
		{#if hasValidationPrompt && onBulkRevalidate}
			<IconButton
				icon={CheckCheck}
				tooltip="Revalidate"
				onclick={onBulkRevalidate}
				variant="primary"
				disabled={anyOperationRunning}
				loading={isRevalidating}
			/>
		{/if}
		<IconButton
			icon={Trash2}
			tooltip="Delete"
			onclick={onBulkDelete}
			variant="destructive"
			disabled={anyOperationRunning}
			loading={isDeleting}
		/>
	</div>

	<button
		type="button"
		onclick={onClear}
		class="ml-auto text-xs text-muted-foreground hover:text-foreground underline"
	>
		Clear selection
	</button>
</div>
