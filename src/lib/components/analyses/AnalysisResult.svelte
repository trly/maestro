<script lang="ts">
	import type { Analysis } from '$lib/types'
	import { ExternalLink, RotateCw, Trash2, TriangleAlert, LoaderCircle, Clock } from 'lucide-svelte'
	import { openInBrowser } from '$lib/utils/browser'
	import UiTooltip from '$lib/components/ui/UiTooltip.svelte'
	import { marked } from 'marked'
	import { analysisStore } from '$lib/stores/executionBus'
	import { getAnalysisStatusConfig } from '$lib/utils/statusConfig'

	let { 
		analysis,
		onDelete,
		onRerun
	}: {
		analysis: Analysis
		onDelete?: () => void
		onRerun?: () => void
	} = $props()

	// Merge static analysis with live updates from event bus
	let reactiveAnalysis = $derived.by(() => {
		const updates = $analysisStore
		const data = updates.get(analysis.id)
		return {
			...analysis,
			...(data || {})
		}
	})

	let typeLabel = $derived(reactiveAnalysis.type === 'execution' ? 'Execution' : 'Validation')
	let formattedResult = $derived(reactiveAnalysis.analysisResult ? marked.parse(reactiveAnalysis.analysisResult) as string : '')

	// Status icon configuration
	let statusConfig = $derived(getAnalysisStatusConfig(reactiveAnalysis.status))

	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleString()
	}
</script>

<div class="space-y-6">
	<div class="flex items-start justify-between gap-4">
		<div class="flex-1 space-y-2">
			<div class="flex items-center gap-2">
				<h4 class="text-lg font-medium text-foreground">
					{typeLabel} Analysis
				</h4>
				<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md {statusConfig.bgClass} {statusConfig.textClass} text-xs font-medium">
					<statusConfig.Icon class="w-3 h-3" />
					{statusConfig.label}
				</span>
			</div>

			<div class="text-xs text-muted-foreground space-y-1">
				<div>Created: {formatDate(reactiveAnalysis.createdAt)}</div>
				{#if reactiveAnalysis.completedAt}
					<div>Completed: {formatDate(reactiveAnalysis.completedAt)}</div>
				{/if}
			</div>
		</div>

		<div class="flex items-center gap-2">
			{#if reactiveAnalysis.ampThreadUrl}
				<UiTooltip content="View in Amp">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => openInBrowser(reactiveAnalysis.ampThreadUrl || '')}
							class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
						>
							<span>Thread</span>
							<ExternalLink class="w-3 h-3" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}
			
			{#if onRerun && (reactiveAnalysis.status === 'completed' || reactiveAnalysis.status === 'failed')}
				<UiTooltip content="Re-run analysis">
					{#snippet children({ props })}
						<button
						{...props}
						onclick={onRerun}
						class="text-primary hover:text-primary/90 transition-colors"
						aria-label="Re-run analysis"
						>
						<RotateCw class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}
			
			{#if onDelete}
				<UiTooltip content="Delete analysis">
					{#snippet children({ props })}
						<button
						{...props}
						onclick={onDelete}
						class="text-destructive hover:text-destructive/90 transition-colors"
						aria-label="Delete analysis"
						>
						<Trash2 class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}
		</div>
	</div>

	{#if reactiveAnalysis.status === 'failed' && reactiveAnalysis.errorMessage}
		<div class="flex items-start gap-2 text-destructive">
			<TriangleAlert class="w-4 h-4 mt-0.5 flex-shrink-0" />
			<div class="text-sm">
				<div class="font-medium mb-1">Analysis Failed</div>
				<div class="text-xs opacity-90">{reactiveAnalysis.errorMessage}</div>
			</div>
		</div>
	{/if}

	{#if reactiveAnalysis.analysisResult}
		<div class="prose prose-sm max-w-none text-foreground prose-headings:text-foreground prose-p:text-foreground prose-strong:text-foreground prose-code:text-foreground">
			{@html formattedResult}
		</div>
	{:else if reactiveAnalysis.status === 'running'}
		<div class="flex items-center justify-between gap-2 text-sm text-muted-foreground">
			<div class="flex items-center gap-2">
				<LoaderCircle class="w-4 h-4 animate-spin" />
				<span>Analysis in progress...</span>
			</div>
			{#if reactiveAnalysis.ampThreadUrl}
				<button
					onclick={() => openInBrowser(reactiveAnalysis.ampThreadUrl || '')}
					class="flex items-center gap-1 text-xs hover:text-foreground transition-colors"
				>
					<span>View Thread</span>
					<ExternalLink class="w-3 h-3" />
				</button>
			{/if}
		</div>
	{:else if reactiveAnalysis.status === 'pending'}
		<div class="flex items-center gap-2 text-sm text-muted-foreground">
			<Clock class="w-4 h-4" />
			<span>Analysis pending...</span>
		</div>
	{/if}
</div>
