<script lang="ts">
	let { type, text }: { type: "committed" | "uncommitted"; text: string } = $props()
	const classes = type === "committed" ? "bg-success/10 text-success" : "bg-warning/10 text-warning"
</script>

<span class="px-3 py-1.5 rounded-sm text-xs font-bold uppercase tracking-wide {classes}">
	{text}
</span>
