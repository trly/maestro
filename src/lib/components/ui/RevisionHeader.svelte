<script lang="ts">
	import { Trash2, Loader2, Edit, FolderGit2 } from 'lucide-svelte';
	import UiTooltip from './UiTooltip.svelte';
	import { toShortHash } from '$lib/utils';
	import type { PromptRevision, Analysis } from '$lib/types';

	let {
		revision,
		stats,
		analyses = [],
		repositoryCount = 0,
		onDelete,
		onEditRepositories
	}: {
		revision: PromptRevision;
		stats?: { total: number; running: number; completed: number; validationPassed: number };
		analyses?: Analysis[];
		repositoryCount?: number;
		onDelete: () => void;
		onEditRepositories?: () => void;
	} = $props();
	
	let completionPercent = $derived(stats && stats.total > 0 ? (stats.completed / stats.total) * 100 : 0);
	let validationPercent = $derived(stats && stats.total > 0 ? (stats.validationPassed / stats.total) * 100 : 0);
	let hasActiveAnalysis = $derived(analyses.some(a => a.status === 'pending' || a.status === 'running'));
	let activeAnalysisCount = $derived(analyses.filter(a => a.status === 'pending' || a.status === 'running').length);
</script>

<div class="h-full flex items-center gap-3 px-4 py-3 bg-card">
	<div class="flex flex-col gap-2 min-w-0 flex-1">
		<div class="flex items-center gap-2">
			<h2 class="text-sm font-bold text-foreground">
				Revision {toShortHash(revision.id)}
			</h2>
			<span class="text-[10px] text-muted-foreground">
				{new Date(revision.createdAt).toLocaleString()}
			</span>
			{#if repositoryCount > 0}
				<UiTooltip content="{repositoryCount} {repositoryCount === 1 ? 'repository' : 'repositories'}">
					{#snippet children({ props })}
						<div {...props} class="flex items-center gap-1 px-2 py-0.5 bg-muted/40 rounded text-muted-foreground">
							<FolderGit2 class="w-3 h-3" />
							<span class="text-[10px] font-medium">{repositoryCount}</span>
						</div>
					{/snippet}
				</UiTooltip>
			{/if}
			{#if onEditRepositories}
				<UiTooltip content="Edit repositories">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onEditRepositories}
							class="text-blue-600 hover:text-blue-700 transition-colors"
							aria-label="Edit repositories"
						>
							<Edit class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}
			{#if hasActiveAnalysis}
				<UiTooltip content="{activeAnalysisCount} analysis {activeAnalysisCount === 1 ? 'running' : 'running'}">
					{#snippet children({ props })}
						<div {...props} class="flex items-center gap-1 px-2 py-0.5 bg-purple-600/10 rounded text-purple-600">
							<Loader2 class="w-3 h-3 animate-spin" />
							<span class="text-[10px] font-medium">{activeAnalysisCount}</span>
						</div>
					{/snippet}
				</UiTooltip>
			{/if}
			<UiTooltip content="Delete this revision">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={onDelete}
						class="text-red-600 hover:text-red-700 transition-colors"
						aria-label="Delete revision"
					>
						<Trash2 class="w-4 h-4" />
					</button>
				{/snippet}
			</UiTooltip>
		</div>
		
		{#if stats && stats.total > 0}
			<div class="flex items-center gap-4 text-xs">
				<!-- Execution Progress -->
				<div class="flex items-center gap-2">
					<span class="text-muted-foreground">Executions:</span>
					<div class="flex items-center gap-1.5">
						<div class="w-24 h-1.5 bg-muted rounded-full overflow-hidden">
							<div 
								class="h-full bg-green-600 transition-all duration-300"
								style="width: {completionPercent}%"
							></div>
						</div>
						<span class="text-foreground font-medium">
							{stats.completed}/{stats.total}
						</span>
					</div>
				</div>
				
				<!-- Validation Progress -->
				<div class="flex items-center gap-2">
					<span class="text-muted-foreground">Validated:</span>
					<div class="flex items-center gap-1.5">
						<div class="w-24 h-1.5 bg-muted rounded-full overflow-hidden">
							<div 
								class="h-full bg-blue-600 transition-all duration-300"
								style="width: {validationPercent}%"
							></div>
						</div>
						<span class="text-foreground font-medium">
							{stats.validationPassed}/{stats.total}
						</span>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>
