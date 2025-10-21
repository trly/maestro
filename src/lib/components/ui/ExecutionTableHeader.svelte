<script lang="ts">
	import { CheckSquare, Square as SquareIcon, ChevronDown, ChevronUp, Play, Square, RotateCw, ScanSearch, Loader2 } from 'lucide-svelte'
	import UiTooltip from './UiTooltip.svelte'
	
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
		onToggleSelectAll,
		onSort,
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
		onToggleSelectAll: () => void
		onSort: (column: string) => void
		onExecuteAll?: () => void
		onStopAll?: () => void
		onStopAllValidations?: () => void
		onRefreshAllCi?: () => void
		onAnalyzeExecutions?: () => void
		onAnalyzeValidations?: () => void
		analyzingExecutions?: boolean
		analyzingValidations?: boolean
	} = $props()
	
	function getSortIcon(column: string) {
		if (sortColumn !== column) return null
		return sortDirection === 'asc' ? ChevronUp : ChevronDown
	}
</script>

<div
	class="sticky top-0 z-10 grid gap-3 px-4 py-2.5 bg-card border-b border-border/10 text-xs font-medium text-muted-foreground items-center
		[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]
		@max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.2fr)_minmax(0,_0.8fr)_minmax(0,_0.8fr)]
		@max-md/table:[grid-template-columns:auto_minmax(200px,_6fr)_40px_40px_0_0_40px]"
>
	<!-- Select All Checkbox -->
	<button
		type="button"
		onclick={onToggleSelectAll}
		class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2 border-muted-foreground/30 hover:border-primary/50 transition-colors"
		aria-label={allSelected ? 'Deselect all' : 'Select all'}
	>
		{#if allSelected}
			<CheckSquare class="w-4 h-4 text-primary" />
		{:else if someSelected}
			<div class="w-2 h-2 bg-primary rounded-sm"></div>
		{:else}
			<SquareIcon class="w-4 h-4 text-muted-foreground/30" />
		{/if}
	</button>
	
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
					class="text-accent hover:text-accent/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
					class="text-accent hover:text-accent/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
