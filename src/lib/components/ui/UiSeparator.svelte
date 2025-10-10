<script lang="ts">
	import { Separator } from 'bits-ui';

	let {
		orientation = 'horizontal',
		decorative = true,
		class: className = ''
	}: {
		orientation?: 'horizontal' | 'vertical';
		decorative?: boolean;
		class?: string;
	} = $props();

	const baseClasses = orientation === 'horizontal' 
		? 'h-px w-full bg-border my-8'
		: 'w-px h-full bg-border mx-8';
</script>

<Separator.Root
	{orientation}
	{decorative}
	class="{baseClasses} {className}"
/>
