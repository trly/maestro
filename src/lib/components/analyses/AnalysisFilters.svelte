<script lang="ts">
	import type { ColumnFilters } from './types'
	import FilterSelect from '$lib/components/ui/FilterSelect.svelte'

	const props = $props<{
		filters: ColumnFilters
		onFilterChange: (filters: ColumnFilters) => void
	}>()

	function update<K extends keyof ColumnFilters>(k: K, v: ColumnFilters[K]) {
		const next = { ...props.filters, [k]: v }
		props.onFilterChange(next)
	}

	const typeOptions = [
		{ value: 'all', label: 'All types' },
		{ value: 'execution', label: 'Execution' },
		{ value: 'validation', label: 'Validation' }
	]

	const statusOptions = [
		{ value: 'all', label: 'All statuses' },
		{ value: 'pending', label: 'Pending' },
		{ value: 'running', label: 'Running' },
		{ value: 'completed', label: 'Completed' },
		{ value: 'failed', label: 'Failed' }
	]
</script>

<div class="flex items-center gap-2 px-4 py-2 border-b border-border bg-muted/20">
	<!-- Type Filter -->
	<div class="w-36">
		<FilterSelect
			value={props.filters.type || 'all'}
			options={typeOptions}
			placeholder="Type"
			onValueChange={(v) => update('type', v as any)}
		/>
	</div>

	<!-- Status Filter -->
	<div class="w-36">
		<FilterSelect
			value={props.filters.status || 'all'}
			options={statusOptions}
			placeholder="Status"
			onValueChange={(v) => update('status', v as any)}
		/>
	</div>

	<!-- Clear Filters Button -->
	{#if Object.keys(props.filters).length > 0}
		<button
			onclick={() => props.onFilterChange({})}
			class="px-2 py-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
		>
			Clear
		</button>
	{/if}
</div>
