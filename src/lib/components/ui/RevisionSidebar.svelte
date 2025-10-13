<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import UiTooltip from './UiTooltip.svelte';
	import UiScrollArea from './UiScrollArea.svelte';
	import { toShortHash } from '$lib/utils';
	import type { PromptRevision } from '$lib/types';

	let {
		revisions,
		currentRevision,
		revisionStats,
		hasValidationPrompt = false,
		onSelect,
		onCreate
	}: {
		revisions: PromptRevision[];
		currentRevision: PromptRevision | null;
		revisionStats: Record<string, { total: number; running: number; completed: number; validationPassed: number }>;
		hasValidationPrompt?: boolean;
		onSelect: (revision: PromptRevision | null) => void;
		onCreate: () => void;
	} = $props();
</script>

<div class="@container/sidebar w-80 @max-6xl/main:w-64 @max-5xl/main:w-48 border-r border-border/20 bg-card flex flex-col">
	<!-- Sidebar Header -->
	<div class="flex items-center justify-between px-4 @max-5xl/main:px-2 py-3 border-b border-border/10 bg-muted/20">
		<h2 class="font-semibold text-foreground @max-5xl/main:hidden">Revisions</h2>
		<UiTooltip content="Create new revision">
			{#snippet children({ props })}
				<button
					{...props}
					onclick={onCreate}
					class="w-7 h-7 flex items-center justify-center rounded-md bg-primary text-primary-foreground hover:opacity-90 transition-all @max-5xl/main:mx-auto"
					aria-label="Create new revision"
				>
					<Plus class="w-4 h-4" />
				</button>
			{/snippet}
		</UiTooltip>
	</div>

	<!-- Revisions List -->
	{#if revisions.length === 0}
		<div class="flex-1 flex items-center justify-center text-muted-foreground p-4">
			<p class="text-sm text-center">No revisions yet.<br/>Create one to get started.</p>
		</div>
	{:else}
		<UiScrollArea class="flex-1">
			<div class="p-2 space-y-1">
				{#each revisions as revision (revision.id)}
					{@const isSelected = currentRevision?.id === revision.id}
					{@const stats = revisionStats[revision.id]}
					
					<button
						onclick={() => onSelect(isSelected ? null : revision)}
						class={`w-full text-left px-2 py-1.5 rounded-md transition-all flex items-center gap-2 ${
							isSelected ? 'bg-primary/10 border border-primary' : 'hover:bg-muted/50 border border-transparent'
						}`}
					>
						<span class="text-[10px] text-muted-foreground flex-shrink-0">
							{new Date(revision.createdAt).toLocaleString()}
						</span>
						
						{#if stats && stats.total > 0}
							<div class="flex items-center gap-2 text-[10px] min-w-0 ml-auto">
								<!-- Executions -->
								<div class="flex items-center gap-1">
									<span class="text-muted-foreground">Ex:</span>
									<span class="font-medium">
										{stats.completed}/{stats.total}
									</span>
								</div>
								
								{#if stats.running > 0}
									<div class="flex items-center gap-1 text-blue-600">
										<div class="w-1 h-1 rounded-full bg-blue-600 animate-pulse"></div>
										<span>{stats.running}</span>
									</div>
								{/if}

								{#if hasValidationPrompt && stats.completed > 0}
									<div class="flex items-center gap-1">
										<span class="text-muted-foreground">V:</span>
										<span class="font-medium text-green-600">
											{stats.validationPassed}/{stats.completed}
										</span>
									</div>
								{/if}
							</div>
						{:else}
							<span class="text-[10px] text-muted-foreground italic ml-auto">Not executed</span>
						{/if}
					</button>
				{/each}
			</div>
		</UiScrollArea>
	{/if}
</div>
