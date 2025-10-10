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

<div class="@container/sidebar w-80 @max-6xl/main:w-64 @max-5xl/main:w-48 border-r border-border/40 bg-card flex flex-col">
	<!-- Sidebar Header -->
	<div class="flex items-center justify-between px-4 @max-5xl/main:px-2 py-3 border-b border-border/40 bg-muted/20">
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
					
					<div
						class={`rounded-md transition-all ${
							isSelected ? 'bg-primary/10 border border-primary' : 'hover:bg-muted/50 border border-transparent'
						}`}
					>
						<button
							onclick={() => onSelect(isSelected ? null : revision)}
							class="w-full text-left p-3"
						>
							<!-- Revision Header -->
							<div class="flex items-center justify-between @max-5xl/main:flex-col @max-5xl/main:items-center @max-5xl/main:gap-1 mb-2"
							>
								<span class="font-mono text-xs text-muted-foreground">
									{toShortHash(revision.id)}
								</span>
								<span class="text-xs text-muted-foreground @max-5xl/main:hidden">
									{new Date(revision.createdAt).toLocaleDateString()}
								</span>
							</div>

							<!-- Revision Stats -->
							{#if stats && stats.total > 0}
								<div class="space-y-1">
									<!-- Executions Progress -->
									<div class="flex items-center justify-between text-xs @max-5xl/main:justify-center">
										<span class="text-muted-foreground @max-5xl/main:hidden">Executions</span>
										<span class="font-medium">
											{stats.completed}/{stats.total}
										</span>
									</div>
									
									{#if stats.running > 0}
										<div class="flex items-center gap-1.5 text-xs text-blue-600 @max-5xl/main:justify-center">
											<div class="w-2 h-2 rounded-full bg-blue-600 animate-pulse"></div>
											<span class="@max-5xl/main:hidden">{stats.running} running</span>
										</div>
									{/if}

									{#if hasValidationPrompt && stats.completed > 0}
										<div class="flex items-center justify-between text-xs @max-5xl/main:justify-center">
											<span class="text-muted-foreground @max-5xl/main:hidden">Validated</span>
											<span class="font-medium text-green-600">
												{stats.validationPassed}/{stats.completed}
											</span>
										</div>
									{/if}

									<!-- Progress Bar -->
									<div class="h-1.5 bg-muted rounded-full overflow-hidden">
										<div
											class="h-full bg-green-600 transition-all"
											style={`width: ${stats.total > 0 ? (stats.completed / stats.total) * 100 : 0}%`}
										></div>
									</div>
								</div>
							{:else}
								<p class="text-xs text-muted-foreground italic @max-5xl/main:text-center">Not executed yet</p>
							{/if}
						</button>
					</div>
				{/each}
			</div>
		</UiScrollArea>
	{/if}
</div>
