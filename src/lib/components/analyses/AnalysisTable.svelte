<script lang="ts">
	import type { Analysis } from '$lib/types'
	import type { SortSpec, ColumnFilters } from './types'
	import AnalysisFilters from './AnalysisFilters.svelte'
	import AnalysisList from './AnalysisList.svelte'

	const props = $props<{
		analyses: Analysis[]
		revisionId?: string
		onDeleteAnalysis: (analysis: Analysis) => void
		onViewThread: (analysis: Analysis) => void
		onViewAnalysis: (analysis: Analysis) => void
	}>()

	let filters = $state<ColumnFilters>({})
	let sort = $state<SortSpec>({ key: 'createdAt', dir: 'desc' })

	$effect(() => {
		props.revisionId
		filters = {}
	})

	function setFilters(next: ColumnFilters) {
		const norm = (v?: string | null) => (v == null || v === '' || v === 'all' ? undefined : v)
		const f = { ...next }
		f.type = norm(f.type as any) as any
		f.status = norm(f.status as any) as any

		for (const k of Object.keys(filters)) delete (filters as any)[k]
		Object.assign(filters, f)
	}

	let analysesById = $derived.by(() => {
		const map = new Map<string, Analysis>()
		for (const a of props.analyses) map.set(a.id, a)
		return map
	})

	let filteredAnalyses = $derived.by(() => {
		return props.analyses.filter((a: Analysis) => {
			if (filters.type && a.type !== filters.type) return false
			if (filters.status && a.status !== filters.status) return false
			return true
		})
	})

	const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })

	let sortedAnalyses = $derived.by(() => {
		const s = [...filteredAnalyses]
		const dir = sort.dir === 'asc' ? 1 : -1
		const cmp = {
			type: (a: Analysis, b: Analysis) => collator.compare(a.type, b.type),
			status: (a: Analysis, b: Analysis) => collator.compare(a.status, b.status),
			createdAt: (a: Analysis, b: Analysis) => (a.createdAt || 0) - (b.createdAt || 0),
			completedAt: (a: Analysis, b: Analysis) => (a.completedAt || 0) - (b.completedAt || 0)
		}[sort.key]
		s.sort((a, b) => {
			const r = cmp(a, b)
			return r === 0 ? dir * collator.compare(a.id, b.id) : dir * r
		})
		return s
	})

	let filteredSortedIds = $derived(sortedAnalyses.map((a) => a.id))

	function handleChangeSort(key: SortSpec['key']) {
		if (sort.key === key) {
			sort.dir = sort.dir === 'asc' ? 'desc' : 'asc'
		} else {
			sort.key = key
			sort.dir = 'asc'
		}
	}

	function getAnalysis(id: string) {
		return analysesById.get(id)!
	}
</script>

<div class="flex-1 min-h-0 flex flex-col overflow-hidden @container/table">
	<AnalysisFilters {filters} onFilterChange={setFilters} />

	<AnalysisList
		ids={filteredSortedIds}
		{analysesById}
		{sort}
		onChangeSort={handleChangeSort}
		onDelete={(id) => props.onDeleteAnalysis(getAnalysis(id))}
		onViewThread={(id) => props.onViewThread(getAnalysis(id))}
		onView={(id) => props.onViewAnalysis(getAnalysis(id))}
	/>
</div>
