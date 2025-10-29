<script lang="ts">
	import type { Analysis } from "$lib/types"
	import { TableState } from "$lib/table/TableState.svelte"
	import { normalizeAnalysisFilters } from "$lib/table/filters"
	import AnalysisFilters from "./AnalysisFilters.svelte"
	import AnalysisList from "./AnalysisList.svelte"

	const props = $props<{
		analyses: Analysis[]
		revisionId?: string
		onDeleteAnalysis: (analysis: Analysis) => void
		onViewThread: (analysis: Analysis) => void
		onViewAnalysis: (analysis: Analysis) => void
	}>()

	const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" })

	const table = new TableState<Analysis>(
		(a) => a.id,
		(filters, a) => {
			if (filters.type && a.type !== filters.type) return false
			if (filters.status && a.status !== filters.status) return false
			return true
		},
		{
			type: (a, b) => collator.compare(a.type, b.type),
			status: (a, b) => collator.compare(a.status, b.status),
			createdAt: (a, b) => (a.createdAt || 0) - (b.createdAt || 0),
			completedAt: (a, b) => (a.completedAt || 0) - (b.completedAt || 0),
		},
		normalizeAnalysisFilters,
		{ key: "createdAt", dir: "desc" }
	)

	$effect(() => {
		table.items = props.analyses
	})

	$effect(() => {
		props.revisionId
		table.clearFilters()
	})

	function getAnalysis(id: string) {
		return table.itemsById.get(id)!
	}
</script>

<div class="flex-1 min-h-0 flex flex-col overflow-hidden @container/table">
	<AnalysisFilters filters={table.filters} onFilterChange={(f) => table.setFilters(f)} />

	<AnalysisList
		ids={table.sortedIds}
		analysesById={table.itemsById}
		sort={table.sort}
		onChangeSort={(key) => table.toggleSort(key)}
		onDelete={(id) => props.onDeleteAnalysis(getAnalysis(id))}
		onViewThread={(id) => props.onViewThread(getAnalysis(id))}
		onView={(id) => props.onViewAnalysis(getAnalysis(id))}
	/>
</div>
