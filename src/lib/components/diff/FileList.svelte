<script lang="ts">
	import { ScrollArea, Checkbox } from 'bits-ui'
	import { Check } from 'lucide-svelte'

	type FileItem = {
		path: string
		status: string
		selected: boolean
		additions?: number
		deletions?: number
	}

	let {
		files = $bindable<FileItem[]>([]),
		selectedIndex = $bindable<number | null>(null),
		onselect,
		ontoggle,
		readonly = false
	}: {
		files: FileItem[]
		selectedIndex?: number | null
		onselect: (index: number) => void
		ontoggle?: (index: number) => void
		readonly?: boolean
	} = $props()

	let selectedCount = $derived(files.filter((f) => f.selected).length)

	function toggleAll() {
		const allSelected = files.every((f) => f.selected)
		files = files.map((f) => ({ ...f, selected: !allSelected }))
	}
</script>

<div class="basis-72 shrink-0 max-w-[40vw] border-r border-border flex flex-col h-full bg-card">
	<div class="flex items-center justify-between px-4 py-3 border-b border-border">
		<h3 class="font-semibold text-sm">
			{#if readonly}
				Modified Files ({files.length})
			{:else}
				Modified Files ({selectedCount}/{files.length})
			{/if}
		</h3>
		{#if !readonly}
			<button class="px-2 py-1 text-xs hover:bg-accent rounded" onclick={toggleAll}>
				{files.every((f) => f.selected) ? 'Deselect All' : 'Select All'}
			</button>
		{/if}
	</div>

	<ScrollArea.Root class="flex-1">
		<ScrollArea.Viewport class="p-2">
			<div class="space-y-0.5">
				{#each files as file, i}
					<button
						type="button"
						class="flex items-center gap-2 p-2 rounded cursor-pointer hover:bg-accent {selectedIndex ===
						i
							? 'bg-accent'
							: ''} text-left w-full"
						onclick={() => onselect(i)}
						aria-pressed={selectedIndex === i}
					>
						{#if !readonly}
							<Checkbox.Root
								checked={file.selected}
								onCheckedChange={(checked) => {
									ontoggle?.(i)
								}}
								onclick={(e) => {
									e.stopPropagation()
								}}
								class="cursor-pointer size-4 rounded border border-border data-[state=checked]:bg-primary flex items-center justify-center"
							>
								{#snippet children({ checked })}
									{#if checked}
										<Check class="size-3 text-primary-foreground" />
									{/if}
								{/snippet}
							</Checkbox.Root>
						{/if}
						<div class="flex-1 min-w-0">
							<div class="text-sm truncate" title={file.path}>{file.path}</div>
							<div class="text-xs text-muted-foreground">{file.status}</div>
						</div>
					</button>
				{/each}
			</div>
		</ScrollArea.Viewport>
		<ScrollArea.Scrollbar orientation="vertical">
			<ScrollArea.Thumb />
		</ScrollArea.Scrollbar>
	</ScrollArea.Root>
</div>
