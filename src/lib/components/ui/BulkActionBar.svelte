<script lang="ts">
	import { Trash2, Play, RotateCw, CheckCircle2, Loader2 } from 'lucide-svelte'
	
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
		isDeleting = false
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
	let anyOperationRunning = $derived(isStarting || isRestarting || isValidating || isRevalidating || isDeleting)
</script>

<div class="flex items-center justify-between px-4 py-2 bg-primary/10 border-b border-border/20">
	<span class="text-sm font-medium text-foreground">
		{selectedCount} selected
	</span>
	
	<div class="flex items-center gap-2">
		<button
			onclick={onBulkStart}
			disabled={anyOperationRunning}
			class="px-3 py-1.5 text-xs font-medium rounded-md bg-success text-success-foreground hover:bg-success/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
		>
			{#if isStarting}
				<Loader2 class="w-3 h-3 animate-spin" />
			{/if}
			<span>Start</span>
		</button>
		<button
			onclick={onBulkRestart}
			disabled={anyOperationRunning}
			class="px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
		>
			{#if isRestarting}
				<Loader2 class="w-3 h-3 animate-spin" />
			{/if}
			<span>Restart</span>
		</button>
		{#if hasValidationPrompt && onBulkStartValidations}
			<button
				onclick={onBulkStartValidations}
				disabled={anyOperationRunning}
				class="px-3 py-1.5 text-xs font-medium rounded-md bg-success text-success-foreground hover:bg-success/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
			>
				{#if isValidating}
					<Loader2 class="w-3 h-3 animate-spin" />
				{/if}
				<span>Start validation</span>
			</button>
		{/if}
		{#if hasValidationPrompt && onBulkRevalidate}
			<button
				onclick={onBulkRevalidate}
				disabled={anyOperationRunning}
				class="px-3 py-1.5 text-xs font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
			>
				{#if isRevalidating}
					<Loader2 class="w-3 h-3 animate-spin" />
				{/if}
				<span>Revalidate</span>
			</button>
		{/if}
		<button
			onclick={onBulkDelete}
			disabled={anyOperationRunning}
			class="px-3 py-1.5 text-xs font-medium rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
		>
			{#if isDeleting}
				<Loader2 class="w-3 h-3 animate-spin" />
			{/if}
			<span>Delete</span>
		</button>
	</div>
	
	<button
		type="button"
		onclick={onClear}
		class="ml-auto text-xs text-muted-foreground hover:text-foreground underline"
	>
		Clear selection
	</button>
</div>
