<script lang="ts">
	import UiLabel from './UiLabel.svelte';

	let {
		label,
		helperText,
		errorText,
		required = false,
		htmlFor,
		class: className = '',
		children
	}: {
		label?: string;
		helperText?: string;
		errorText?: string;
		required?: boolean;
		htmlFor?: string;
		class?: string;
		children: import('svelte').Snippet;
	} = $props();
</script>

<div class="mb-6 {className}">
	{#if label}
		<UiLabel for={htmlFor} {required}>{label}</UiLabel>
	{/if}
	
	{@render children()}
	
	{#if errorText}
		<p class="text-xs text-destructive mt-1">{errorText}</p>
	{:else if helperText}
		<p class="text-xs text-muted-foreground mt-1">{helperText}</p>
	{/if}
</div>
