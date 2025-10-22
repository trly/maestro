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

	const statusOptions = [
		{ value: 'all', label: 'All statuses' },
		{ value: 'pending', label: 'Pending' },
		{ value: 'running', label: 'Running' },
		{ value: 'completed', label: 'Completed' },
		{ value: 'failed', label: 'Failed' },
		{ value: 'cancelled', label: 'Cancelled' }
	]

	const validationOptions = [
		{ value: 'all', label: 'All validation' },
		{ value: 'pending', label: 'Pending' },
		{ value: 'running', label: 'Running' },
		{ value: 'passed', label: 'Passed' },
		{ value: 'failed', label: 'Failed' },
		{ value: 'cancelled', label: 'Cancelled' }
	]

	const ciOptions = [
		{ value: 'all', label: 'All CI' },
		{ value: 'pending', label: 'Pending' },
		{ value: 'passed', label: 'Passed' },
		{ value: 'failed', label: 'Failed' },
		{ value: 'skipped', label: 'Skipped' },
		{ value: 'not_configured', label: 'Not configured' }
	]

	const changesOptions = [
		{ value: 'all', label: 'Any changes' },
		{ value: 'has-changes', label: 'Has changes' },
		{ value: 'no-changes', label: 'No changes' }
	]
</script>

<div class="flex items-center gap-2 px-4 py-2 border-b border-border bg-muted/20">
	<!-- Repository Filter -->
	<input
		type="text"
		placeholder="Filter by repository..."
		value={props.filters.repository || ''}
		oninput={(e) => update('repository', e.currentTarget.value)}
		class="flex-1 px-2 py-1.5 text-xs border border-border rounded-[var(--radius-sm)] bg-background text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
	/>

	<!-- Status Filter -->
	<div class="w-36">
		<FilterSelect
			value={props.filters.status || 'all'}
			options={statusOptions}
			placeholder="Status"
			onValueChange={(v) => update('status', v as any)}
		/>
	</div>

	<!-- Validation Filter -->
	<div class="w-36">
		<FilterSelect
			value={props.filters.validationStatus || 'all'}
			options={validationOptions}
			placeholder="Validation"
			onValueChange={(v) => update('validationStatus', v as any)}
		/>
	</div>

	<!-- CI Filter -->
	<div class="w-36">
		<FilterSelect
			value={props.filters.ciStatus || 'all'}
			options={ciOptions}
			placeholder="CI"
			onValueChange={(v) => update('ciStatus', v as any)}
		/>
	</div>

	<!-- Changes Filter -->
	<div class="w-36">
		<FilterSelect
			value={props.filters.changes || 'all'}
			options={changesOptions}
			placeholder="Changes"
			onValueChange={(v) => update('changes', v as any)}
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
