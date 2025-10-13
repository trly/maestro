<script lang="ts">
	let {
		value = $bindable(''),
		scrollElement = $bindable(null),
		oninput,
		onscroll,
		title = 'Edit',
		placeholder = ''
	}: {
		value: string
		scrollElement?: HTMLTextAreaElement | null
		oninput: (value: string) => void
		onscroll?: (e: Event) => void
		title?: string
		placeholder?: string
	} = $props()

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement
		value = target.value
		oninput(target.value)
	}
</script>

<div class="rounded-md border border-primary/30 overflow-hidden shadow-sm">
	<div class="bg-primary/10 px-4 py-2 border-b border-primary/10">
		<h4 class="text-xs font-semibold text-foreground">{title}</h4>
	</div>
	<textarea
		bind:this={scrollElement}
		{value}
		oninput={handleInput}
		onscroll={onscroll}
		{placeholder}
		class="w-full h-96 px-4 py-3 font-mono text-sm border-0 focus:ring-0 resize-none bg-background text-foreground"
	></textarea>
</div>
