<script lang="ts">
	import { processTextDiff } from "$lib/diff"
	import DiffUnified from "./diff/DiffUnified.svelte"

	interface Props {
		oldText: string
		newText: string
		onupdate: (text: string) => void
	}

	let { oldText, newText, onupdate }: Props = $props()

	let editedText = $state(newText)
	let diffScrollElement: HTMLDivElement | null = null
	let editorScrollElement: HTMLTextAreaElement | null = null

	$effect(() => {
		editedText = newText
	})

	let diffItems = $derived(processTextDiff(oldText, editedText))

	let debounceTimer: ReturnType<typeof setTimeout> | null = null
	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement
		editedText = target.value
		if (debounceTimer) clearTimeout(debounceTimer)
		debounceTimer = setTimeout(() => {
			onupdate(target.value)
		}, 200)
	}

	function handleDiffScroll(e: Event) {
		const target = e.target as HTMLDivElement
		if (editorScrollElement) {
			const scrollPercentage = target.scrollTop / (target.scrollHeight - target.clientHeight)
			editorScrollElement.scrollTop =
				scrollPercentage * (editorScrollElement.scrollHeight - editorScrollElement.clientHeight)
		}
	}

	function handleEditorScroll(e: Event) {
		const target = e.target as HTMLTextAreaElement
		if (diffScrollElement) {
			const scrollPercentage = target.scrollTop / (target.scrollHeight - target.clientHeight)
			diffScrollElement.scrollTop =
				scrollPercentage * (diffScrollElement.scrollHeight - diffScrollElement.clientHeight)
		}
	}
</script>

<div class="grid grid-cols-2 gap-3">
	<div class="rounded-md border border-primary/30 overflow-hidden shadow-sm">
		<div class="bg-primary/10 px-4 py-2 border-b border-primary/10">
			<h4 class="text-xs font-semibold text-foreground">Edit Prompt</h4>
		</div>
		<textarea
			bind:this={editorScrollElement}
			value={editedText}
			oninput={handleInput}
			onscroll={handleEditorScroll}
			placeholder="Edit your prompt..."
			class="w-full h-96 px-4 py-3 font-mono text-sm border-0 focus:ring-0 resize-none bg-background text-foreground"
		></textarea>
	</div>

	<div class="rounded-md border border-border/30 overflow-hidden">
		<div class="bg-muted/50 px-4 py-2 border-b border-border/10">
			<h4 class="text-xs font-medium text-muted-foreground">Changes from Previous Version</h4>
		</div>
		<div
			bind:this={diffScrollElement}
			onscroll={handleDiffScroll}
			class="h-96 bg-muted/30 overflow-auto px-4 py-3"
		>
			<div class="font-mono text-sm">
				<DiffUnified items={diffItems} />
			</div>
		</div>
	</div>
</div>
