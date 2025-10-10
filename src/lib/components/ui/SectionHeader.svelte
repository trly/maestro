<script lang="ts">
	let { title, variant = 'default' }: { title: string; variant?: 'default' | 'accent' } = $props()
	const bgClass = variant === 'accent' ? 'bg-accent' : 'bg-muted'
	const textClass = variant === 'accent' ? 'text-accent-foreground' : 'text-muted-foreground'
</script>

<div class="{bgClass} px-4 py-2 border-b border-border">
	<h4 class="text-xs font-semibold uppercase tracking-wide {textClass}">{title}</h4>
</div>
