<script lang="ts">
	import { Trash2, Play, RotateCw, CheckCircle2 } from 'lucide-svelte'
	
	let {
		selectedCount,
		hasValidationPrompt = false,
		onBulkDelete,
		onBulkStart,
		onBulkRestart,
		onBulkStartValidations,
		onBulkRevalidate,
		onClear
	}: {
		selectedCount: number
		hasValidationPrompt?: boolean
		onBulkDelete: () => void
		onBulkStart: () => void
		onBulkRestart: () => void
		onBulkStartValidations?: () => void
		onBulkRevalidate?: () => void
		onClear: () => void
	} = $props()
</script>

<div class="flex items-center justify-between px-4 py-2 bg-primary/10 border-b border-border/20">
	<span class="text-sm font-medium text-foreground">
		{selectedCount} selected
	</span>
	
	<div class="flex items-center gap-2">
		<button
			onclick={onBulkStart}
			class="px-3 py-1.5 text-xs font-medium rounded-md bg-green-600 text-white hover:bg-green-700 transition-colors"
		>
			Start
		</button>
		<button
			onclick={onBulkRestart}
			class="px-3 py-1.5 text-xs font-medium rounded-md bg-blue-600 text-white hover:bg-blue-700 transition-colors"
		>
			Restart
		</button>
		{#if hasValidationPrompt && onBulkStartValidations}
			<button
				onclick={onBulkStartValidations}
				class="px-3 py-1.5 text-xs font-medium rounded-md bg-green-600 text-white hover:bg-green-700 transition-colors"
			>
				Start validation
			</button>
		{/if}
		{#if hasValidationPrompt && onBulkRevalidate}
			<button
				onclick={onBulkRevalidate}
				class="px-3 py-1.5 text-xs font-medium rounded-md bg-blue-600 text-white hover:bg-blue-700 transition-colors"
			>
				Revalidate
			</button>
		{/if}
		<button
			onclick={onBulkDelete}
			class="px-3 py-1.5 text-xs font-medium rounded-md bg-red-600 text-white hover:bg-red-700 transition-colors"
		>
			Delete
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
