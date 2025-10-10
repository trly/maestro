<script lang="ts">
	import { ScrollArea } from 'bits-ui'
	
	let { children, class: className = '' }: { children: any; class?: string } = $props()
</script>

<ScrollArea.Root class="flex-1 {className}">
	<ScrollArea.Viewport class="h-full">
		<div class="font-mono text-sm whitespace-pre-wrap p-4">
			{@render children()}
		</div>
	</ScrollArea.Viewport>
	<ScrollArea.Scrollbar orientation="vertical">
		<ScrollArea.Thumb />
	</ScrollArea.Scrollbar>
</ScrollArea.Root>
