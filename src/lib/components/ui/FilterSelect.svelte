<script lang="ts">
	import { ChevronDown, Check } from 'lucide-svelte'
	import { Select } from 'bits-ui'
	
	const props = $props<{
		value?: string
		options: Array<{ value: string; label: string }>
		placeholder?: string
		onValueChange: (value: string) => void
	}>()
	
	// Local value bound to Select; keep in sync with parent
	let value = $state(props.value || 'all')
	$effect(() => { value = props.value || 'all' })
	$effect(() => { 
		if (value !== undefined && value !== (props.value || 'all')) {
			props.onValueChange(value)
		}
	})
	
	let displayValue = $derived(
		value 
			? props.options.find((o: { value: string; label: string }) => o.value === value)?.label || props.placeholder || 'Select...'
			: props.placeholder || 'Select...'
	)
</script>

<Select.Root type="single" bind:value>
	<Select.Trigger
		class="w-full flex items-center justify-between px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground hover:bg-muted/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
	>
		<span class="truncate">{displayValue}</span>
		<ChevronDown class="w-3 h-3 text-muted-foreground flex-shrink-0" />
	</Select.Trigger>

	<Select.Portal>
		<Select.Content
			class="w-[var(--bits-select-trigger-width)] bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
			sideOffset={4}
		>
			<Select.Viewport class="max-h-60 overflow-auto">
				{#each props.options as item (item.value)}
					<Select.Item
						value={item.value}
						label={item.label}
						class="flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
					>
						<span>{item.label}</span>
						{#if value === item.value}
							<Check class="w-3 h-3 text-primary" />
						{/if}
					</Select.Item>
				{/each}
			</Select.Viewport>
		</Select.Content>
	</Select.Portal>
</Select.Root>
