<script lang="ts">
	import { Tabs } from 'bits-ui'
	import type { DiffItem } from '$lib/types/diff'
	import DiffUnified from './DiffUnified.svelte'
	import DiffSplit from './DiffSplit.svelte'

	let {
		items,
		viewMode = $bindable('split'),
		scrollElement = $bindable(null),
		onscroll,
		commitStatus,
		parentSha,
		commitSha,
		fileStatus
	}: { 
		items: DiffItem[]
		viewMode?: 'split' | 'unified'
		scrollElement?: HTMLDivElement | null
		onscroll?: (e: Event) => void
		commitStatus?: 'uncommitted' | 'committed'
		parentSha?: string | null
		commitSha?: string | null
		fileStatus?: string
	} = $props()
</script>

<Tabs.Root bind:value={viewMode}>
	<div class="mb-3 flex items-center justify-between">
		<Tabs.List class="inline-flex rounded-sm bg-muted p-1">
			<Tabs.Trigger value="split" class="px-3 py-1.5 text-sm font-medium rounded-sm transition-all data-[state=active]:bg-background data-[state=active]:shadow-sm">
				Split
			</Tabs.Trigger>
			<Tabs.Trigger value="unified" class="px-3 py-1.5 text-sm font-medium rounded-sm transition-all data-[state=active]:bg-background data-[state=active]:shadow-sm">
				Unified
			</Tabs.Trigger>
		</Tabs.List>
	</div>

	<Tabs.Content value="split">
		<DiffSplit {items} {commitStatus} {parentSha} {commitSha} {fileStatus} />
	</Tabs.Content>

	<Tabs.Content value="unified">
		<div class="rounded-md border border-border/30 overflow-hidden">
		<div class="bg-muted/50 px-4 py-2 border-b border-border/10">
				<h4 class="text-xs font-medium text-muted-foreground">Previous Version (Read-only)</h4>
			</div>
			<div bind:this={scrollElement} onscroll={onscroll} class="h-96 bg-muted/30 overflow-auto opacity-75">
				<DiffUnified {items} />
			</div>
		</div>
	</Tabs.Content>
</Tabs.Root>
