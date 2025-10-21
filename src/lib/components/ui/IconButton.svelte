<script lang="ts">
	import type { ComponentType } from 'svelte'
	import UiTooltip from './UiTooltip.svelte'

	import { Loader2 } from 'lucide-svelte'

	let {
	icon,
	tooltip,
	onclick,
	variant = 'default',
	disabled = false,
	size = 'default',
	 loading = false,
	class: className = '',
	...props
	}: {
	icon: ComponentType
	tooltip: string
	onclick?: () => void
	variant?: 'default' | 'success' | 'destructive' | 'warning' | 'primary' | 'accent' | 'ghost'
	disabled?: boolean
	 size?: 'sm' | 'default' | 'lg'
		loading?: boolean
		class?: string
		[key: string]: any
	} = $props()

	const variantClasses = {
		default: 'text-foreground hover:text-foreground/90',
		success: 'text-success hover:text-success/90',
		destructive: 'text-destructive hover:text-destructive/90',
		warning: 'text-warning hover:text-warning/90',
		primary: 'text-primary hover:text-primary/90',
		accent: 'text-accent hover:text-accent/90',
		ghost: 'text-muted-foreground hover:text-foreground'
	}

	const sizeClasses = {
		sm: 'w-3.5 h-3.5',
		default: 'w-4 h-4',
		lg: 'w-5 h-5'
	}

	const buttonClass = $derived(
		`${variantClasses[variant]} transition-colors ${className} ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`
	)

	let Icon = $derived(icon)
</script>

<UiTooltip content={tooltip}>
	{#snippet children({ props: tooltipProps })}
		<button
			{...tooltipProps}
			{...props}
			{onclick}
			{disabled}
			class={buttonClass}
			aria-label={tooltip}
		>
			{#if loading}
				<Loader2 class={sizeClasses[size]} />
			{:else}
				<Icon class={sizeClasses[size]} />
			{/if}
		</button>
	{/snippet}
</UiTooltip>
