<script lang="ts">
	import { Label } from 'bits-ui';

	let {
		for: htmlFor,
		required = false,
		class: className = '',
		children
	}: {
		for?: string;
		required?: boolean;
		class?: string;
		children: import('svelte').Snippet;
	} = $props();
</script>

<Label.Root
	for={htmlFor}
	class="block text-sm font-semibold text-foreground mb-2 {className}"
>
	{@render children()}
	{#if required}
		<span class="text-destructive ml-1">*</span>
	{/if}
</Label.Root>
