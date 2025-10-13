<script lang="ts">
	import { Play, Square, Trash2, RotateCw } from 'lucide-svelte';
	import UiTooltip from './UiTooltip.svelte';
	import { toShortHash } from '$lib/utils';
	import type { PromptRevision } from '$lib/types';

	let {
		revision,
		hasRunning = false,
		hasRunningValidations = false,
		stats,
		onExecuteAll,
		onStopAll,
		onStopAllValidations,
		onDelete,
		onRefreshAllCi
	}: {
		revision: PromptRevision;
		hasRunning?: boolean;
		hasRunningValidations?: boolean;
		stats?: { total: number; running: number; completed: number; validationPassed: number };
		onExecuteAll: () => void;
		onStopAll: () => void;
		onStopAllValidations: () => void;
		onDelete: () => void;
		onRefreshAllCi: () => void;
	} = $props();
	
	let completionPercent = $derived(stats && stats.total > 0 ? (stats.completed / stats.total) * 100 : 0);
	let validationPercent = $derived(stats && stats.total > 0 ? (stats.validationPassed / stats.total) * 100 : 0);
</script>

<div class="h-full flex items-center gap-3 px-4 py-3 bg-card">
	<div class="flex flex-col gap-2 min-w-0">
		<div class="flex items-center gap-2">
			<h2 class="text-sm font-bold text-foreground">
				Revision {toShortHash(revision.id)}
			</h2>
			<span class="text-[10px] text-muted-foreground">
				{new Date(revision.createdAt).toLocaleString()}
			</span>
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
				
				<!-- Actions inline with stats -->
				<div class="flex items-center gap-2 ml-2">
					<UiTooltip content="Execute on all repos">
						{#snippet children({ props })}
							<button
								{...props}
								onclick={onExecuteAll}
								class="text-green-600 hover:text-green-700 transition-colors"
								aria-label="Execute on all repos"
							>
								<Play class="w-4 h-4" />
							</button>
						{/snippet}
					</UiTooltip>

					{#if hasRunning}
						<UiTooltip content="Stop all running executions">
							{#snippet children({ props })}
								<button
									{...props}
									onclick={onStopAll}
									class="text-orange-600 hover:text-orange-700 transition-colors"
									aria-label="Stop all running executions"
								>
									<Square class="w-4 h-4" />
								</button>
							{/snippet}
						</UiTooltip>
					{/if}

					{#if hasRunningValidations}
						<UiTooltip content="Stop all running validations">
							{#snippet children({ props })}
								<button
									{...props}
									onclick={onStopAllValidations}
									class="text-orange-600 hover:text-orange-700 transition-colors"
									aria-label="Stop all running validations"
								>
									<Square class="w-4 h-4 fill-current" />
								</button>
							{/snippet}
						</UiTooltip>
					{/if}

					<UiTooltip content="Refresh all CI statuses">
						{#snippet children({ props })}
							<button
								{...props}
								onclick={onRefreshAllCi}
								class="text-blue-600 hover:text-blue-700 transition-colors"
								aria-label="Refresh all CI statuses"
							>
								<RotateCw class="w-4 h-4" />
							</button>
						{/snippet}
					</UiTooltip>

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
			</div>
		{:else}
			<!-- Show execute button when no stats -->
			<div class="flex items-center gap-2 text-xs">
				<UiTooltip content="Execute on all repos">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onExecuteAll}
							class="text-green-600 hover:text-green-700 transition-colors"
							aria-label="Execute on all repos"
						>
							<Play class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>

				<UiTooltip content="Refresh all CI statuses">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onRefreshAllCi}
							class="text-blue-600 hover:text-blue-700 transition-colors"
							aria-label="Refresh all CI statuses"
						>
							<RotateCw class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>

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
		{/if}
	</div>
</div>
