<script lang="ts">
	import type { Analysis, AnalysisStatus } from '$lib/types'
	import IconButton from '$lib/components/ui/IconButton.svelte'
	import { ExternalLink, Trash2, Clock, LoaderCircle, CircleCheck, CircleX } from 'lucide-svelte'

	let {
		analysis,
		onDelete,
		onViewThread,
		onView
	}: {
		analysis: Analysis
		onDelete: () => void
		onViewThread: () => void
		onView: () => void
	} = $props()

	const statusConfig: Record<AnalysisStatus, { icon: any; color: string }> = {
		pending: { icon: Clock, color: 'text-muted-foreground' },
		running: { icon: LoaderCircle, color: 'text-primary animate-spin' },
		completed: { icon: CircleCheck, color: 'text-success' },
		failed: { icon: CircleX, color: 'text-destructive' }
	}

	let statusIcon = $derived(statusConfig[analysis.status].icon)
	let statusColor = $derived(statusConfig[analysis.status].color)
	
	function formatRelativeTime(timestamp: number): string {
		const now = Date.now()
		const diff = now - timestamp
		const seconds = Math.floor(diff / 1000)
		const minutes = Math.floor(seconds / 60)
		const hours = Math.floor(minutes / 60)
		const days = Math.floor(hours / 24)

		if (days > 0) return `${days}d ago`
		if (hours > 0) return `${hours}h ago`
		if (minutes > 0) return `${minutes}m ago`
		return 'just now'
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault()
			onView()
		}
	}
</script>

<div 
	role="button"
	tabindex="0"
	onclick={onView}
	onkeydown={handleKeyDown}
	class="grid gap-3 px-4 py-2 border-b border-border hover:bg-muted/30 items-center cursor-pointer
            [grid-template-columns:minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_2fr)_minmax(0,_1fr)_auto]"
>
	<!-- Type -->
	<div class="text-xs capitalize">
		{analysis.type}
	</div>

	<!-- Status -->
	<div class="flex items-center gap-2 text-xs">
		{#if statusIcon}
			{@const Icon = statusIcon}
			<Icon size={14} class={statusColor} />
		{/if}
		<span class="capitalize">{analysis.status}</span>
	</div>

	<!-- Prompts -->
	<div class="text-xs text-muted-foreground">
		{analysis.executionCount}
	</div>

	<!-- Created -->
	<div class="text-xs text-muted-foreground">
		{formatRelativeTime(analysis.createdAt)}
	</div>

	<!-- Actions -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="flex items-center gap-1" role="group" onclick={(e) => e.stopPropagation()}>
		{#if analysis.ampThreadUrl}
			<IconButton
				icon={ExternalLink}
				tooltip="View thread"
				onclick={onViewThread}
			/>
		{/if}
		<IconButton
			icon={Trash2}
			tooltip="Delete"
			variant="destructive"
			onclick={onDelete}
		/>
	</div>
</div>