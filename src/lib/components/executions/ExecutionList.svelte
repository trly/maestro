<script lang="ts">
	import type { Execution, Repository } from "$lib/types"
	import type { SortSpec } from "$lib/table/types"
	import type { SelectionState } from "$lib/composables/useSelection.svelte"
	import ExecutionRow from "./ExecutionRow.svelte"
	import IconButton from "$lib/components/ui/IconButton.svelte"
	import { ChevronUp, ChevronDown, Brain } from "lucide-svelte"

	const props = $props<{
		ids: string[]
		executionsById: Map<string, Execution>
		repositories: Map<string, Repository>
		selection: SelectionState
		pushingExecutions: Set<string>
		refreshingCi: Set<string>
		loadingStats: Set<string>
		sort: SortSpec
		hasValidationPrompt?: boolean
		onAnalyzeExecutions?: () => void
		onAnalyzeValidations?: () => void
		analyzingExecutions?: boolean
		analyzingValidations?: boolean
		onToggleSelectAll: () => void
		onChangeSort: (key: string) => void
		onLoadStats: (id: string) => void
		onLoadCi: (id: string) => void
		onStart: (id: string) => void
		onStop: (id: string) => void
		onRestart: (id: string) => void
		onValidate: (id: string) => void
		onStopValidation: (id: string) => void
		onDelete: (id: string) => void
		onReviewChanges: (id: string) => void
		onPush: (id: string) => void
		onRefreshCi: (id: string) => void
	}>()

	let allVisibleSelected = $derived.by(() => {
		const ids = props.ids
		if (ids.length === 0) return false
		return ids.every((id: string) => props.selection.selectedIds.has(id))
	})

	let someVisibleSelected = $derived(props.selection.selectedIds.size > 0 && !allVisibleSelected)

	function getSortIcon(key: string) {
		if (props.sort.key !== key) return null
		return props.sort.dir === "asc" ? ChevronUp : ChevronDown
	}
</script>

<div class="flex flex-col min-h-0 overflow-hidden">
	<!-- Header -->
	<div
		class="grid gap-3 px-4 py-2 border-b-2 border-border bg-muted/30 items-center sticky top-0 z-10
	            [grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]"
	>
		<!-- Select All Checkbox -->
		<button
			type="button"
			onclick={props.onToggleSelectAll}
			class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2
			       {allVisibleSelected
				? 'border-primary bg-primary'
				: someVisibleSelected
					? 'border-primary bg-primary/50'
					: 'border-muted-foreground/30 hover:border-primary/50'} 
			       transition-colors"
			aria-label="Select all"
		>
			{#if allVisibleSelected || someVisibleSelected}
				<div class="w-2 h-2 bg-primary-foreground rounded-sm"></div>
			{/if}
		</button>

		<!-- Column Headers -->
		<button
			class="text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors text-left flex items-center gap-1"
			onclick={() => props.onChangeSort("repo")}
		>
			<span>Repository</span>
			{#if getSortIcon("repo")}
				{@const Icon = getSortIcon("repo")}
				<Icon class="w-3 h-3" />
			{/if}
		</button>

		<div class="flex items-center gap-1.5">
			<button
				class="text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors text-left flex items-center gap-1"
				onclick={() => props.onChangeSort("status")}
			>
				<span>Execution</span>
				{#if getSortIcon("status")}
					{@const Icon = getSortIcon("status")}
					<Icon class="w-3 h-3" />
				{/if}
			</button>
			{#if props.onAnalyzeExecutions}
				<IconButton
					icon={Brain}
					tooltip="Analyze execution failures"
					onclick={props.onAnalyzeExecutions}
					variant="primary"
					size="sm"
					loading={props.analyzingExecutions}
				/>
			{/if}
		</div>

		<div class="flex items-center gap-1.5">
			<button
				class="text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors text-left flex items-center gap-1"
				onclick={() => props.onChangeSort("validation")}
			>
				<span>Validation</span>
				{#if getSortIcon("validation")}
					{@const Icon = getSortIcon("validation")}
					<Icon class="w-3 h-3" />
				{/if}
			</button>
			{#if props.onAnalyzeValidations}
				<IconButton
					icon={Brain}
					tooltip="Analyze validation failures"
					onclick={props.onAnalyzeValidations}
					variant="primary"
					size="sm"
					loading={props.analyzingValidations}
				/>
			{/if}
		</div>

		<button
			class="text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors text-left flex items-center gap-1"
			onclick={() => props.onChangeSort("diff")}
		>
			<span>Changes</span>
			{#if getSortIcon("diff")}
				{@const Icon = getSortIcon("diff")}
				<Icon class="w-3 h-3" />
			{/if}
		</button>

		<button
			class="text-xs font-semibold text-muted-foreground hover:text-foreground transition-colors text-left flex items-center gap-1"
			onclick={() => props.onChangeSort("ci")}
		>
			<span>CI</span>
			{#if getSortIcon("ci")}
				{@const Icon = getSortIcon("ci")}
				<Icon class="w-3 h-3" />
			{/if}
		</button>

		<div class="text-xs font-semibold text-muted-foreground text-right">Actions</div>
	</div>

	<!-- Rows -->
	<div class="flex-1 overflow-auto">
		{#if props.ids.length === 0}
			<div class="px-4 py-8 text-center text-muted-foreground">
				<p>No executions found</p>
			</div>
		{:else}
			{#each props.ids as id (id)}
				<ExecutionRow
					{id}
					executionsById={props.executionsById}
					repositories={props.repositories}
					selected={props.selection.selectedIds.has(id)}
					pushing={props.pushingExecutions.has(id)}
					refreshingCi={props.refreshingCi.has(id)}
					loadingStats={props.loadingStats.has(id)}
					hasValidationPrompt={props.hasValidationPrompt}
					onToggleSelected={() => props.selection.toggle(id)}
					onLoadStats={() => props.onLoadStats(id)}
					onLoadCi={() => props.onLoadCi(id)}
					onStart={() => props.onStart(id)}
					onStop={() => props.onStop(id)}
					onRestart={() => props.onRestart(id)}
					onValidate={() => props.onValidate(id)}
					onStopValidation={() => props.onStopValidation(id)}
					onDelete={() => props.onDelete(id)}
					onReviewChanges={() => props.onReviewChanges(id)}
					onPush={() => props.onPush(id)}
					onRefreshCi={() => props.onRefreshCi(id)}
				/>
			{/each}
		{/if}
	</div>
</div>
