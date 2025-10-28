<script lang="ts">
	import type { HTMLTextareaAttributes } from "svelte/elements"

	interface Props extends HTMLTextareaAttributes {
		value?: string
		error?: boolean
		class?: string
	}

	let {
		value = $bindable(""),
		error = false,
		class: className = "",
		...restProps
	}: Props = $props()

	const baseClasses = "w-full px-4 py-3 border rounded-md transition-all resize-y font-mono text-sm"
	const errorClasses = "border-destructive focus-visible:ring-2 focus-visible:ring-ring"
	const normalClasses =
		"border-input-border bg-input-background focus-visible:ring-2 focus-visible:ring-ring"
</script>

<textarea
	bind:value
	class="{baseClasses} {error ? errorClasses : normalClasses} {className}"
	{...restProps}
></textarea>
