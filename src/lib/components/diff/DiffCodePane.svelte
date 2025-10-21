<script lang="ts">
	let { children, class: className = '' }: { children: any; class?: string } = $props()
</script>

<div class="flex-1 overflow-auto {className}">
	<div class="font-mono text-sm whitespace-pre-wrap p-4">
		{@render children()}
	</div>
</div>
