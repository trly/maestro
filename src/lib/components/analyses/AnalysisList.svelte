<script lang="ts">
	import type { Analysis } from '$lib/types'
	import type { SortSpec } from './types'
	import AnalysisRow from './AnalysisRow.svelte'
	import { ChevronUp, ChevronDown } from 'lucide-svelte'

	const props = $props<{
		ids: string[]
		analysesById: Map<string, Analysis>
		sort: SortSpec
		onChangeSort: (key: SortSpec['key']) => void
		onDelete: (id: string) => void
		onViewThread: (id: string) => void
		onView: (id: string) => void
	}>()

	function getSortIcon(key: SortSpec['key']) {
		if (props.sort.key !== key) return null
		return props.sort.dir === 'asc' ? ChevronUp : ChevronDown
	}
</script>

<div class="flex flex-col min-h-0 overflow-hidden">
	<!-- Header -->
	<div class="grid gap-3 px-4 py-2 border-b-2 border-border bg-muted/30 items-center sticky top-0 z-10
	            [grid-template-columns:minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_2fr)_minmax(0,_1fr)_auto]"
	>
		<!-- Type -->
		<button
			type="button"
			onclick={() => props.onChangeSort('type')}
			class="text-left text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
		>
			Type
			{#if getSortIcon('type')}
				{@const Icon = getSortIcon('type')}
				<Icon size={12} />
			{/if}
		</button>

		<!-- Status -->
		<button
			type="button"
			onclick={() => props.onChangeSort('status')}
			class="text-left text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
		>
			Status
			{#if getSortIcon('status')}
				{@const Icon = getSortIcon('status')}
				<Icon size={12} />
			{/if}
		</button>

		<!-- Prompts -->
		<div class="text-left text-xs font-semibold text-muted-foreground">
			Prompts
		</div>

		<!-- Created -->
		<button
			type="button"
			onclick={() => props.onChangeSort('createdAt')}
			class="text-left text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors flex items-center gap-1"
		>
			Created
			{#if getSortIcon('createdAt')}
				{@const Icon = getSortIcon('createdAt')}
				<Icon size={12} />
			{/if}
		</button>

		<!-- Actions -->
		<div class="text-left text-xs font-semibold text-muted-foreground">
			Actions
		</div>
	</div>

	<!-- Body -->
	<div class="flex-1 overflow-y-auto">
		{#if props.ids.length === 0}
			<div class="px-4 py-8 text-center text-sm text-muted-foreground">
				No analyses found
			</div>
		{:else}
			{#each props.ids as id (id)}
				{@const analysis = props.analysesById.get(id)}
				{#if analysis}
					<AnalysisRow
						{analysis}
						onDelete={() => props.onDelete(id)}
						onViewThread={() => props.onViewThread(id)}
						onView={() => props.onView(id)}
					/>
				{/if}
			{/each}
		{/if}
	</div>
</div>
