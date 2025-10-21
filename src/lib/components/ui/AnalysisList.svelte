<script lang="ts">
	import type { Analysis } from '$lib/types'
	import { CheckCircle2, AlertCircle, Clock, Loader2, ExternalLink } from 'lucide-svelte'
	import { Dialog } from 'bits-ui'
	import AnalysisResult from './AnalysisResult.svelte'
	import { analysisStore } from '$lib/stores/executionBus'
	
	let {
		analyses,
		onDelete,
		onRerun
	}: {
		analyses: Analysis[]
		onDelete: (analysis: Analysis) => void
		onRerun: (analysis: Analysis) => void
	} = $props()
	
	let sortedAnalyses = $derived(
		[...analyses].sort((a, b) => b.createdAt - a.createdAt)
	)
	
	let selectedAnalysis = $state<Analysis | null>(null)
	let dialogOpen = $state(false)
	
	function openAnalysis(analysis: Analysis) {
		selectedAnalysis = analysis
		dialogOpen = true
	}
	
	function getStatusIcon(status: string) {
		switch (status) {
			case 'completed': return CheckCircle2
			case 'failed': return AlertCircle
			case 'running': return Loader2
			default: return Clock
		}
	}
	
	function getStatusClass(status: string) {
		switch (status) {
			case 'completed': return 'text-success'
			case 'failed': return 'text-destructive'
			case 'running': return 'text-primary'
			default: return 'text-muted-foreground'
		}
	}
	
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		})
	}
</script>

<div class="flex flex-col gap-2 p-4">
	{#if analyses.length === 0}
		<div class="flex flex-col items-center justify-center py-12 text-center">
			<div class="text-muted-foreground mb-2">
				<svg class="w-12 h-12 mx-auto mb-3 opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
				</svg>
				<p class="text-sm font-medium">No analyses yet</p>
				<p class="text-xs mt-1">Analyze failed executions or validations to identify common patterns</p>
			</div>
		</div>
	{:else}
		{#each sortedAnalyses as analysis (analysis.id)}
			{@const statusIcon = getStatusIcon(analysis.status)}
			{@const statusClass = getStatusClass(analysis.status)}
			{@const updates = $analysisStore.get(analysis.id)}
			{@const currentStatus = updates?.status || analysis.status}
			{@const currentIcon = getStatusIcon(currentStatus)}
			{@const currentClass = getStatusClass(currentStatus)}
			
			<button
				onclick={() => openAnalysis(analysis)}
				class="w-full text-left rounded-[var(--radius)] border border-border bg-card hover:bg-accent/50 transition-colors p-3"
			>
				<div class="flex items-center gap-3">
					<svelte:component this={currentIcon} class="w-4 h-4 {currentClass} {currentStatus === 'running' ? 'animate-spin' : ''}" />
					
					<div class="flex-1 min-w-0">
						<div class="flex items-center gap-2 mb-1">
							<span class="text-sm font-medium text-card-foreground">
								{analysis.type === 'execution' ? 'Execution' : 'Validation'} Analysis
							</span>
						</div>
						<div class="text-xs text-muted-foreground">
							{formatDate(analysis.createdAt)}
						</div>
					</div>
					
					{#if analysis.ampThreadUrl || updates?.ampThreadUrl}
						<ExternalLink class="w-3.5 h-3.5 text-muted-foreground" />
					{/if}
				</div>
			</button>
		{/each}
	{/if}
</div>

{#if selectedAnalysis}
	<Dialog.Root bind:open={dialogOpen}>
		<Dialog.Portal>
			<Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
			<Dialog.Content
				class="fixed inset-0 w-full h-full overflow-auto bg-background z-50 p-6"
			>
				<AnalysisResult
					analysis={selectedAnalysis}
					onDelete={() => {
						onDelete(selectedAnalysis!)
						dialogOpen = false
					}}
					onRerun={() => {
						onRerun(selectedAnalysis!)
					}}
				/>
			</Dialog.Content>
		</Dialog.Portal>
	</Dialog.Root>
{/if}
