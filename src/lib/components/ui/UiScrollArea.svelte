<script lang="ts">
	import { ScrollArea } from 'bits-ui';
	import type { Snippet } from 'svelte';

	let {
		class: className = '',
		viewportClass = '',
		children,
		...restProps
	}: {
		class?: string;
		viewportClass?: string;
		children: Snippet;
		[key: string]: any;
	} = $props();
</script>

<ScrollArea.Root class="relative overflow-hidden rounded-md border border-border/20 {className}" {...restProps}>
	<ScrollArea.Viewport class="h-full w-full rounded-[inherit] {viewportClass}">
		{@render children()}
	</ScrollArea.Viewport>
	<ScrollArea.Scrollbar
		orientation="vertical"
		class="flex w-2.5 touch-none select-none rounded-full p-px transition-all bg-transparent hover:bg-muted/50 data-[state=visible]:animate-in data-[state=hidden]:animate-out data-[state=hidden]:fade-out-0 data-[state=visible]:fade-in-0"
	>
		<ScrollArea.Thumb class="relative flex-1 rounded-full bg-border hover:bg-border/80" />
	</ScrollArea.Scrollbar>
	<ScrollArea.Corner />
</ScrollArea.Root>
