<script lang="ts">
	import { X } from 'lucide-svelte'
	
	const props = $props<{
		value?: string
		placeholder?: string
		onValueChange: (value: string) => void
		onClear?: () => void
	}>()
</script>

<div class="relative">
	<input
		type="text"
		value={props.value || ''}
		oninput={(e) => props.onValueChange(e.currentTarget.value)}
		placeholder={props.placeholder}
		class="w-full px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
	/>
	{#if props.value}
		<button
			onclick={props.onClear}
			class="absolute right-1.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-destructive transition-colors"
			aria-label="Clear filter"
		>
			<X class="w-3 h-3" />
		</button>
	{/if}
</div>
