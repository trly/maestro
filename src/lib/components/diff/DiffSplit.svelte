<script lang="ts">
	import type { DiffItem } from '$lib/types/diff'
	import DiffCodePane from './DiffCodePane.svelte'

	let {
		items,
		commitStatus,
		parentSha,
		commitSha,
		fileStatus
	}: {
		items: DiffItem[]
		commitStatus?: 'uncommitted' | 'committed'
		parentSha?: string | null
		commitSha?: string | null
		fileStatus?: string
	} = $props()
	
	let leftPane: HTMLDivElement
	let rightPane: HTMLDivElement
	
	function syncScroll(source: HTMLDivElement, target: HTMLDivElement) {
		target.scrollTop = source.scrollTop
	}
	
	let leftHeader = $derived(
		parentSha ? parentSha.slice(0, 7) : (fileStatus === 'added' ? '' : 'Modified')
	)
	let rightHeader = $derived(
		commitStatus === 'committed' && commitSha
			? commitSha.slice(0, 7)
			: fileStatus === 'added' ? 'New' : 'Modified'
	)
</script>

<div class="grid grid-cols-2 gap-2 rounded-md border border-border/30 overflow-hidden">
	<div class="flex flex-col">
		<div class="bg-muted px-4 py-2 border-b border-border/10">
			<h4 class="text-xs font-semibold text-muted-foreground">{leftHeader}</h4>
		</div>
		<div bind:this={leftPane} onscroll={() => syncScroll(leftPane, rightPane)} class="h-96 bg-background overflow-auto">
			<DiffCodePane>
				{#each items as item}
					{#if item.type === 'removed'}
						<div class="bg-destructive/10 text-destructive -mx-4 px-4 flex">
							<span class="text-muted-foreground mr-4 select-none w-12 text-right shrink-0">{item.oldLineNumber || ''}</span>
							<span class="flex-1">{item.oldLine || ' '}</span>
						</div>
					{:else if item.type === 'modified'}
						<div class="bg-destructive/10 text-destructive -mx-4 px-4 flex">
							<span class="text-muted-foreground mr-4 select-none w-12 text-right shrink-0">{item.oldLineNumber || ''}</span>
							<span class="flex-1">
								{#each item.segments || [] as seg}
									{#if seg.removed}
										<span class="bg-destructive/30 font-semibold">{seg.value}</span>
									{:else if !seg.added}
										<span>{seg.value}</span>
									{/if}
								{/each}
							</span>
						</div>
					{:else if item.type === 'unchanged'}
						<div class="text-foreground flex">
							<span class="text-muted-foreground mr-4 select-none w-12 text-right shrink-0">{item.oldLineNumber || ''}</span>
							<span class="flex-1">{item.oldLine || ' '}</span>
						</div>
					{/if}
				{/each}
			</DiffCodePane>
		</div>
	</div>

	<div class="flex flex-col border-l border-border/20">
		<div class="bg-accent px-4 py-2 border-b border-border/10">
			<h4 class="text-xs font-semibold text-accent-foreground">{rightHeader}</h4>
		</div>
		<div bind:this={rightPane} onscroll={() => syncScroll(rightPane, leftPane)} class="h-96 bg-background overflow-auto">
			<DiffCodePane>
				{#each items as item}
					{#if item.type === 'added'}
						<div class="bg-success/10 text-success -mx-4 px-4 flex">
							<span class="text-muted-foreground mr-4 select-none w-12 text-right shrink-0">{item.newLineNumber || ''}</span>
							<span class="flex-1">{item.newLine || ' '}</span>
						</div>
					{:else if item.type === 'modified'}
						<div class="bg-success/10 text-success -mx-4 px-4 flex">
							<span class="text-muted-foreground mr-4 select-none w-12 text-right shrink-0">{item.newLineNumber || ''}</span>
							<span class="flex-1">
								{#each item.segments || [] as seg}
									{#if seg.added}
										<span class="bg-success/30 font-semibold">{seg.value}</span>
									{:else if !seg.removed}
										<span>{seg.value}</span>
									{/if}
								{/each}
							</span>
						</div>
					{:else if item.type === 'unchanged'}
						<div class="text-foreground flex">
							<span class="text-muted-foreground mr-4 select-none w-12 text-right shrink-0">{item.newLineNumber || ''}</span>
							<span class="flex-1">{item.newLine || ' '}</span>
						</div>
					{/if}
				{/each}
			</DiffCodePane>
		</div>
	</div>
</div>
