<script lang="ts">
	import type { DiffItem } from '$lib/types/diff'
	import DiffCodePane from './DiffCodePane.svelte'

	let { items }: { items: DiffItem[] } = $props()
</script>

<DiffCodePane>
	{#each items as item}
		{#if item.type === 'added'}
			<div class="bg-success/10 text-success">+ {item.newLine}</div>
		{:else if item.type === 'removed'}
			<div class="bg-destructive/10 text-destructive">- {item.oldLine}</div>
		{:else if item.type === 'modified'}
			<div class="bg-destructive/10 text-destructive">
				- {#each item.segments || [] as seg}{#if seg.removed}<span class="bg-destructive/30 font-semibold">{seg.value}</span>{:else if !seg.added}{seg.value}{/if}{/each}
			</div>
			<div class="bg-success/10 text-success">
				+ {#each item.segments || [] as seg}{#if seg.added}<span class="bg-success/30 font-semibold">{seg.value}</span>{:else if !seg.removed}{seg.value}{/if}{/each}
			</div>
		{:else}
			<div class="text-foreground">  {item.oldLine}</div>
		{/if}
	{/each}
</DiffCodePane>
