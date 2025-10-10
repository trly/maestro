<script lang="ts">
	import { Dialog } from 'bits-ui'
	import type { Snippet } from 'svelte'

	let {
		open = $bindable(false),
		title,
		children,
		footer
	}: {
		open?: boolean
		title: string
		children: Snippet
		footer?: Snippet
	} = $props()
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 z-50 bg-black/50" />
		<Dialog.Content
			class="fixed inset-4 z-50 bg-background rounded-lg shadow-2xl border border-border flex flex-col"
		>
			<Dialog.Title class="px-6 py-4 border-b border-border flex-shrink-0">
				<h2 class="text-xl font-bold text-foreground">{title}</h2>
			</Dialog.Title>
			<div class="flex-1 min-h-0 overflow-hidden">
				{@render children()}
			</div>
			{#if footer}
				<div class="px-6 py-4 border-t border-border flex-shrink-0">
					{@render footer()}
				</div>
			{/if}
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
