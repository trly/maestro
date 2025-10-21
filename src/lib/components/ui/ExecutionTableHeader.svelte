<script lang="ts">
	import { ChevronDown, ChevronUp, Play, Square, RotateCw, ScanSearch, Loader2, Filter, X, Check, Minus } from 'lucide-svelte'
	import { Select } from 'bits-ui'
	import UiTooltip from './UiTooltip.svelte'
	
	export interface ColumnFilters {
		repository?: string
		status?: string
		validationStatus?: string
		changes?: string
		ciStatus?: string
	}
	
	let {
		allSelected = false,
		someSelected = false,
		sortColumn = null,
		sortDirection = 'asc',
		hasRunning = false,
		hasRunningValidations = false,
		hasFailedExecutions = false,
		hasFailedValidations = false,
		hasCommitted = false,
		executionCount = 0,
		executions = [],
		filters = {},
		onToggleSelectAll,
		onSort,
		onFilterChange,
		onExecuteAll,
		onStopAll,
		onStopAllValidations,
		onRefreshAllCi,
		onAnalyzeExecutions,
		onAnalyzeValidations,
		analyzingExecutions = false,
		analyzingValidations = false
	}: {
		allSelected?: boolean
		someSelected?: boolean
		sortColumn?: string | null
		sortDirection?: 'asc' | 'desc'
		hasRunning?: boolean
		hasRunningValidations?: boolean
		hasFailedExecutions?: boolean
		hasFailedValidations?: boolean
		hasCommitted?: boolean
		executionCount?: number
		executions?: Array<any>
		filters?: ColumnFilters
		onToggleSelectAll: () => void
		onSort: (column: string) => void
		onFilterChange: (filters: ColumnFilters) => void
		onExecuteAll?: () => void
		onStopAll?: () => void
		onStopAllValidations?: () => void
		onRefreshAllCi?: () => void
		onAnalyzeExecutions?: () => void
		onAnalyzeValidations?: () => void
		analyzingExecutions?: boolean
		analyzingValidations?: boolean
	} = $props()
	
	let showFilters = $state(true)
	
	// Extract unique values from executions
	let availableStatuses = $derived(Array.from(new Set(executions.map(e => e.status).filter(Boolean))))
	let availableValidationStatuses = $derived(Array.from(new Set(executions.map(e => e.validationStatus).filter(Boolean))))
	let availableCiStatuses = $derived(Array.from(new Set(executions.map(e => e.ciStatus).filter(Boolean))))
	let hasChanges = $derived(executions.some(e => ((e.filesAdded || 0) + (e.filesRemoved || 0) + (e.filesModified || 0)) > 0))
	let hasNoChanges = $derived(executions.some(e => ((e.filesAdded || 0) + (e.filesRemoved || 0) + (e.filesModified || 0)) === 0))
	
	// Convert filter values for Select components (empty string -> 'all')
	let statusValue = $derived({ value: filters.status || 'all' })
	let validationValue = $derived({ value: filters.validationStatus || 'all' })
	let changesValue = $derived({ value: filters.changes || 'all' })
	let ciStatusValue = $derived({ value: filters.ciStatus || 'all' })
	
	function getSortIcon(column: string) {
		if (sortColumn !== column) return null
		return sortDirection === 'asc' ? ChevronUp : ChevronDown
	}
	
	function updateFilter(column: keyof ColumnFilters, value: string) {
		onFilterChange({ ...filters, [column]: value || undefined })
	}
	
	function clearFilter(column: keyof ColumnFilters) {
		const newFilters = { ...filters }
		delete newFilters[column]
		onFilterChange(newFilters)
	}
	
	function clearAllFilters() {
		onFilterChange({})
	}
	
	let hasActiveFilters = $derived(Object.keys(filters).length > 0)
</script>

<div class="sticky top-0 z-10 bg-card border-b border-border/10">
	<!-- Header Row -->
	<div
		class="grid gap-3 px-4 py-2.5 text-xs font-medium text-muted-foreground items-center
			[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]
			@max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.2fr)_minmax(0,_0.8fr)_minmax(0,_0.8fr)]
			@max-md/table:[grid-template-columns:auto_minmax(200px,_6fr)_40px_40px_0_0_40px]"
	>
		<!-- Select All Checkbox -->
		<div class="flex items-center gap-2">
			<button
			type="button"
			onclick={onToggleSelectAll}
			class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2 {allSelected ? 'border-primary bg-primary' : 'border-muted-foreground/30 hover:border-primary/50'} transition-colors"
			aria-label={allSelected ? 'Deselect all' : 'Select all'}
			>
			{#if allSelected}
			<Check class="w-4 h-4 text-primary-foreground" />
			{:else if someSelected}
			<Minus class="w-4 h-4 text-primary" />
			{:else}
			<!-- Empty for unchecked state -->
			{/if}
			</button>
			<UiTooltip content={showFilters ? "Hide filters" : "Show filters"}>
				{#snippet children({ props: tooltipProps })}
					<button
						{...tooltipProps}
						onclick={() => showFilters = !showFilters}
						class="text-muted-foreground hover:text-foreground transition-colors {hasActiveFilters ? 'text-primary' : ''}"
						aria-label="Toggle filters"
					>
						<Filter class="w-3.5 h-3.5" />
					</button>
				{/snippet}
			</UiTooltip>
			{#if hasActiveFilters}
				<UiTooltip content="Clear all filters">
					{#snippet children({ props: tooltipProps })}
						<button
							{...tooltipProps}
							onclick={clearAllFilters}
							class="text-muted-foreground hover:text-destructive transition-colors"
							aria-label="Clear filters"
						>
							<X class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}
		</div>
		
		<!-- Repository Column -->
		<button
			onclick={() => onSort('repository')}
			class="text-left hover:text-foreground transition-colors truncate"
		>
			Repository
			{#if getSortIcon('repository')}
				{@const Icon = getSortIcon('repository')}
				{#if Icon}
					<Icon class="w-3 h-3 inline ml-1" />
				{/if}
			{/if}
		</button>
	
	<!-- Execution Column -->
	<div class="flex items-center gap-2">
		<button
			onclick={() => onSort('status')}
			class="text-left hover:text-foreground transition-colors truncate"
		>
			Execution
			{#if getSortIcon('status')}
				{@const Icon = getSortIcon('status')}
				{#if Icon}
					<Icon class="w-3 h-3 inline ml-1" />
				{/if}
			{/if}
		</button>
		{#if hasRunning && onStopAll}
			<UiTooltip content="Stop all running executions">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={onStopAll}
						class="text-warning hover:text-warning/90 transition-colors"
						aria-label="Stop all running executions"
					>
						<Square class="w-3.5 h-3.5" />
					</button>
				{/snippet}
			</UiTooltip>
		{:else if onExecuteAll}
			<UiTooltip content="Execute on all repos">
				{#snippet children({ props })}
					<button
					{...props}
					onclick={onExecuteAll}
					class="text-success hover:text-success/90 transition-colors"
					aria-label="Execute on all repos"
					>
					<Play class="w-3.5 h-3.5" />
					</button>
				{/snippet}
			</UiTooltip>
		{/if}
		{#if hasFailedExecutions && onAnalyzeExecutions}
			<UiTooltip content={analyzingExecutions ? "Analyzing..." : "Analyze failed executions"}>
				{#snippet children({ props })}
					<button
					{...props}
					onclick={onAnalyzeExecutions}
					disabled={analyzingExecutions}
					class="text-warning hover:text-warning/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
					aria-label="Analyze failed executions"
					>
					{#if analyzingExecutions}
						<Loader2 class="w-3.5 h-3.5 animate-spin" />
					{:else}
						<ScanSearch class="w-3.5 h-3.5" />
					{/if}
					</button>
				{/snippet}
			</UiTooltip>
		{/if}
	</div>
	
	<!-- Validation Column -->
	<div class="flex items-center gap-2">
		<button
			onclick={() => onSort('validationStatus')}
			class="text-left hover:text-foreground transition-colors truncate"
		>
			Validation
			{#if getSortIcon('validationStatus')}
				{@const Icon = getSortIcon('validationStatus')}
				{#if Icon}
					<Icon class="w-3 h-3 inline ml-1" />
				{/if}
			{/if}
		</button>
		{#if hasRunningValidations && onStopAllValidations}
			<UiTooltip content="Stop all running validations">
				{#snippet children({ props })}
					<button
					{...props}
					onclick={onStopAllValidations}
					class="text-warning hover:text-warning/90 transition-colors"
					aria-label="Stop all running validations"
					>
					<Square class="w-3.5 h-3.5 fill-current" />
					</button>
				{/snippet}
			</UiTooltip>
		{/if}
		{#if hasFailedValidations && onAnalyzeValidations}
			<UiTooltip content={analyzingValidations ? "Analyzing..." : "Analyze failed validations"}>
				{#snippet children({ props })}
					<button
					{...props}
					onclick={onAnalyzeValidations}
					disabled={analyzingValidations}
					class="text-warning hover:text-warning/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
					aria-label="Analyze failed validations"
					>
					{#if analyzingValidations}
						<Loader2 class="w-3.5 h-3.5 animate-spin" />
					{:else}
						<ScanSearch class="w-3.5 h-3.5" />
					{/if}
					</button>
				{/snippet}
			</UiTooltip>
		{/if}
	</div>
	
	<!-- Changes Column -->
	<button
		onclick={() => onSort('commitStatus')}
		class="text-left hover:text-foreground transition-colors truncate @max-md/table:hidden"
	>
		Changes
		{#if getSortIcon('commitStatus')}
			{@const Icon = getSortIcon('commitStatus')}
			{#if Icon}
				<Icon class="w-3 h-3 inline ml-1" />
			{/if}
		{/if}
	</button>
	
	<!-- CI Column -->
	<div class="flex items-center gap-2 @max-md/table:hidden">
		<span class="text-left truncate">CI</span>
		{#if hasCommitted && onRefreshAllCi}
			<UiTooltip content="Refresh all CI statuses">
				{#snippet children({ props })}
					<button
					{...props}
					onclick={onRefreshAllCi}
					class="text-primary hover:text-primary/90 transition-colors"
					aria-label="Refresh all CI statuses"
					>
					<RotateCw class="w-3.5 h-3.5" />
					</button>
				{/snippet}
			</UiTooltip>
		{/if}
	</div>
	
		<!-- Actions Column -->
		<div class="text-right truncate">Actions</div>
	</div>
	
	<!-- Filter Row -->
	{#if showFilters}
		<div
			class="grid gap-3 px-4 py-2 text-xs border-t border-border/10 items-center
				[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]
				@max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.2fr)_minmax(0,_0.8fr)_minmax(0,_0.8fr)]
				@max-md/table:[grid-template-columns:auto_minmax(200px,_6fr)_40px_40px_0_0_40px]"
		>
			<!-- Empty cell for select column -->
			<div></div>
			
			<!-- Repository Filter -->
			<div class="relative">
				<input
					type="text"
					value={filters.repository || ''}
					oninput={(e) => updateFilter('repository', e.currentTarget.value)}
					placeholder="Filter repositories..."
					class="w-full px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
				/>
				{#if filters.repository}
					<button
						onclick={() => clearFilter('repository')}
						class="absolute right-1.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-destructive transition-colors"
						aria-label="Clear repository filter"
					>
						<X class="w-3 h-3" />
					</button>
				{/if}
			</div>
			
			<!-- Execution Status Filter -->
			<Select.Root
				type="single"
				value={statusValue}
				onValueChange={(v) => {
					if (!v?.value) return
					updateFilter('status', v.value === 'all' ? '' : v.value)
				}}
			>
				<Select.Trigger
					class="w-full flex items-center justify-between px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground hover:bg-muted/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
				>
					<span class="truncate">{filters.status ? filters.status.charAt(0).toUpperCase() + filters.status.slice(1) : 'All statuses'}</span>
					<ChevronDown class="w-3 h-3 text-muted-foreground flex-shrink-0" />
				</Select.Trigger>
				<Select.Content
					class="w-[var(--bits-select-trigger-width)] bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
					sideOffset={4}
				>
					{#each [
						{ value: 'all', label: 'All statuses' },
						...availableStatuses.map(s => ({ 
							value: s, 
							label: s.charAt(0).toUpperCase() + s.slice(1) 
						}))
					] as item (item.value)}
						<Select.Item
							value={item.value}
							label={item.label}
							class="flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
						>
							<span>{item.label}</span>
							{#if (filters.status || 'all') === item.value}
								<Check class="w-3 h-3 text-primary" />
							{/if}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
			
			<!-- Validation Status Filter -->
			<Select.Root
				type="single"
				value={validationValue}
				onValueChange={(v) => {
					if (!v?.value) return
					updateFilter('validationStatus', v.value === 'all' ? '' : v.value)
				}}
			>
				<Select.Trigger
					class="w-full flex items-center justify-between px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground hover:bg-muted/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
				>
					<span class="truncate">{filters.validationStatus ? filters.validationStatus.charAt(0).toUpperCase() + filters.validationStatus.slice(1) : 'All statuses'}</span>
					<ChevronDown class="w-3 h-3 text-muted-foreground flex-shrink-0" />
				</Select.Trigger>
				<Select.Content
					class="w-[var(--bits-select-trigger-width)] bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
					sideOffset={4}
				>
					{#each [
						{ value: 'all', label: 'All statuses' },
						...availableValidationStatuses.map(s => ({ 
							value: s, 
							label: s.charAt(0).toUpperCase() + s.slice(1) 
						}))
					] as item (item.value)}
						<Select.Item
							value={item.value}
							label={item.label}
							class="flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
						>
							<span>{item.label}</span>
							{#if (filters.validationStatus || 'all') === item.value}
								<Check class="w-3 h-3 text-primary" />
							{/if}
						</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
			
			<!-- Changes Filter -->
			<div class="@max-md/table:hidden">
				<Select.Root
					type="single"
					value={changesValue}
					onValueChange={(v) => {
						if (!v?.value) return
						updateFilter('changes', v.value === 'all' ? '' : v.value)
					}}
				>
					<Select.Trigger
						class="w-full flex items-center justify-between px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground hover:bg-muted/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
					>
						<span class="truncate">
							{#if filters.changes === 'has-changes'}
								Has changes
							{:else if filters.changes === 'no-changes'}
								No changes
							{:else}
								All
							{/if}
						</span>
						<ChevronDown class="w-3 h-3 text-muted-foreground flex-shrink-0" />
					</Select.Trigger>
					<Select.Content
						class="w-[var(--bits-select-trigger-width)] bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
						sideOffset={4}
					>
						{#each [
							{ value: 'all', label: 'All' },
							...(hasChanges ? [{ value: 'has-changes', label: 'Has changes' }] : []),
							...(hasNoChanges ? [{ value: 'no-changes', label: 'No changes' }] : [])
						] as item (item.value)}
							<Select.Item
								value={item.value}
								label={item.label}
								class="flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
							>
								<span>{item.label}</span>
								{#if (filters.changes || 'all') === item.value}
									<Check class="w-3 h-3 text-primary" />
								{/if}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			
			<!-- CI Status Filter -->
			<div class="@max-md/table:hidden">
				<Select.Root
					type="single"
					value={ciStatusValue}
					onValueChange={(v) => {
						if (!v?.value) return
						updateFilter('ciStatus', v.value === 'all' ? '' : v.value)
					}}
				>
					<Select.Trigger
						class="w-full flex items-center justify-between px-2 py-1.5 text-xs border border-input-border bg-input-background rounded-[var(--radius-sm)] text-foreground hover:bg-muted/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring transition-all"
					>
						<span class="truncate">
							{#if filters.ciStatus === 'not_configured'}
								Not configured
							{:else if filters.ciStatus}
								{filters.ciStatus.charAt(0).toUpperCase() + filters.ciStatus.slice(1)}
							{:else}
								All statuses
							{/if}
						</span>
						<ChevronDown class="w-3 h-3 text-muted-foreground flex-shrink-0" />
					</Select.Trigger>
					<Select.Content
						class="w-[var(--bits-select-trigger-width)] bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
						sideOffset={4}
					>
						{#each [
							{ value: 'all', label: 'All statuses' },
							...availableCiStatuses.map(s => ({ 
								value: s, 
								label: s === 'not_configured' ? 'Not configured' : s.charAt(0).toUpperCase() + s.slice(1) 
							}))
						] as item (item.value)}
							<Select.Item
								value={item.value}
								label={item.label}
								class="flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
							>
								<span>{item.label}</span>
								{#if (filters.ciStatus || 'all') === item.value}
									<Check class="w-3 h-3 text-primary" />
								{/if}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			
			<!-- Empty cell for actions column -->
			<div></div>
		</div>
	{/if}
</div>
