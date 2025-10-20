<script lang="ts">
	import { Trash2 } from 'lucide-svelte';
	import UiTooltip from './UiTooltip.svelte';
	import { toShortHash } from '$lib/utils';
	import type { PromptRevision } from '$lib/types';

	let {
		revision,
		stats,
		onDelete
	}: {
		revision: PromptRevision;
		stats?: { total: number; running: number; completed: number; validationPassed: number };
		onDelete: () => void;
	} = $props();
	
	let completionPercent = $derived(stats && stats.total > 0 ? (stats.completed / stats.total) * 100 : 0);
	let validationPercent = $derived(stats && stats.total > 0 ? (stats.validationPassed / stats.total) * 100 : 0);
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
