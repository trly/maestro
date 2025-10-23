<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { Plus, PanelLeftClose, PanelLeftOpen, Settings, ChevronDown } from 'lucide-svelte';
	import { Accordion } from 'bits-ui';
import UiTooltip from '$lib/components/ui/UiTooltip.svelte';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import { confirm } from '$lib/ui/confirm';
	import { toShortHash } from '$lib/utils';
	import { sidebarStore } from '$lib/stores/sidebarStore';
	import { executionStore } from '$lib/stores/executionBus';
	import type { PromptSet, PromptRevision, Execution } from '$lib/types';

	let { collapsed = false, onToggleCollapse, pathname, searchParams } = $props();

	let allPromptSets = $state<PromptSet[]>([]);
	let revisionsByPromptSet = $state<Map<string, PromptRevision[]>>(new Map());
	let executionsByRevision = $state<Map<string, Execution[]>>(new Map());
	let isLoading = $state(true);
	let accordionValue = $state<string[]>([]);

	async function loadPromptSets() {
		try {
			isLoading = true;
			
			const promptSets = await api.promptSets.getAll();
			
			// Build new Maps to ensure Svelte 5 reactivity
			const nextRevisionsByPromptSet = new Map<string, PromptRevision[]>();
			const nextExecutionsByRevision = new Map<string, Execution[]>();
			
			// Load revisions for each prompt set
			for (const ps of promptSets) {
				const revisions = await api.promptSets.getRevisions(ps.id);
				nextRevisionsByPromptSet.set(ps.id, revisions);
				
				// Load executions for each revision
				for (const revision of revisions) {
					const executions = await api.revisions.getExecutions(revision.id);
					nextExecutionsByRevision.set(revision.id, executions);
				}
			}
			
			// Reassign Maps to trigger reactivity
			allPromptSets = promptSets;
			revisionsByPromptSet = nextRevisionsByPromptSet;
			executionsByRevision = nextExecutionsByRevision;
		} catch (err) {
			showToast('Failed to load prompt sets: ' + err, 'error');
		} finally {
			isLoading = false;
		}
	}

	async function deletePromptSetWithConfirm(ps: PromptSet) {
		const revisionCount = await api.promptSets.getRevisions(ps.id).then(r => r.length);
		const executionCount = await api.promptSets.getExecutions(ps.id).then(e => e.length);
		
		const confirmed = await confirm({
			title: `Delete prompt set "${ps.name}"?`,
			message: 
				`This will delete:\n` +
				`- The prompt set\n` +
				`- ${revisionCount} revision(s)\n` +
				`- ${executionCount} execution(s)\n` +
				`- All associated git branches\n\n` +
				`This action cannot be undone.`,
			confirmText: 'Delete',
			cancelText: 'Cancel'
		});
		
		if (!confirmed) return;
		
		try {
			await api.promptSets.delete(ps.id);
			await loadPromptSets();
			showToast('Prompt set deleted successfully', 'success');
		} catch (err) {
			showToast('Failed to delete prompt set: ' + err, 'error');
		}
	}

	let hasAutoNavigated = $state(false);

	onMount(loadPromptSets);
	
	// Reload when triggered by sidebarStore
	$effect(() => {
		const _ = $sidebarStore; // Subscribe to refresh trigger
		loadPromptSets();
	});

	// Auto-select newest revision when first landing on a prompt set without a revision
	$effect(() => {
		const currentPath = pathname;
		const currentRevisionParam = searchParams.revision;
		
		// Only auto-navigate once when we're on a prompt set page without a revision
		const promptSetMatch = currentPath.match(/^\/promptsets\/([^\/]+)$/);
		if (promptSetMatch && !currentRevisionParam && !hasAutoNavigated) {
			const promptSetId = promptSetMatch[1];
			const revisions = revisionsByPromptSet.get(promptSetId);
			
			// If this prompt set has revisions, navigate to the newest one
			if (revisions && revisions.length > 0) {
				const newestRevision = revisions[0]; // Revisions are already sorted by createdAt desc
				goto(`/promptsets/${promptSetId}?revision=${newestRevision.id}`, { replaceState: true });
				hasAutoNavigated = true;
			}
		}
		
		// Reset flag when navigating away or to a different prompt set
		if (currentRevisionParam || !promptSetMatch) {
			hasAutoNavigated = false;
		}
	});

	// Compute revision stats reactively with live execution updates
	let revisionStats = $derived.by(() => {
		const stats = new Map<string, { total: number; running: number; completed: number; validationPassed: number }>()
		
		for (const [revisionId, executions] of executionsByRevision.entries()) {
			// Merge static executions with live updates
			const executionsWithUpdates = executions.map(e => {
				const data = $executionStore.get(e.id)
				if (!data) return e
				return {
					...e,
					...(data.status && { status: data.status }),
					...(data.validationStatus && { validationStatus: data.validationStatus })
				}
			})
			
			const total = executionsWithUpdates.length
			const running = executionsWithUpdates.filter(e => e.status === 'running').length
			const completed = executionsWithUpdates.filter(e => e.status === 'completed').length
			const validationPassed = executionsWithUpdates.filter(e => e.validationStatus === 'passed').length
			
			stats.set(revisionId, { total, running, completed, validationPassed })
		}
		
		return stats
	})

	const isPromptSetActive = (id: string) => pathname.startsWith(`/promptsets/${id}`);
	const isRevisionActive = (revisionId: string) => searchParams.revision === revisionId || searchParams.revision === toShortHash(revisionId);
</script>

<div class="h-full flex flex-col bg-card">
	<!-- Header -->
	<div class="flex items-center gap-1.5 px-2 py-2.5 border-b border-border/10 flex-shrink-0 bg-muted/20">
		{#if !collapsed}
			<h2 class="text-sm font-semibold text-foreground flex-1">Prompt Sets</h2>
			<UiTooltip content="Create new prompt set">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={() => goto('/create')}
						class="p-1.5 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
						aria-label="Create prompt set"
					>
						<Plus class="w-3.5 h-3.5" />
					</button>
				{/snippet}
			</UiTooltip>
			<UiTooltip content="Settings">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={() => goto('/settings')}
						class="p-1.5 rounded-md transition-colors {pathname === '/settings' ? 'bg-accent text-accent-foreground' : 'hover:bg-accent'}"
						aria-label="Settings"
					>
						<Settings class="w-3.5 h-3.5" />
					</button>
				{/snippet}
			</UiTooltip>
			<button
				onclick={onToggleCollapse}
				class="p-1.5 rounded-md hover:bg-accent transition-colors"
				aria-label="Collapse sidebar"
			>
				<PanelLeftClose class="w-3.5 h-3.5" />
			</button>
		{:else}
			<div class="flex flex-col gap-1.5 w-full">
				<UiTooltip content="Expand sidebar">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onToggleCollapse}
							class="p-1.5 rounded-md hover:bg-accent transition-colors mx-auto"
							aria-label="Expand sidebar"
						>
							<PanelLeftOpen class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>
				<UiTooltip content="Create new prompt set">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => goto('/create')}
							class="p-1.5 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors mx-auto"
							aria-label="Create prompt set"
						>
							<Plus class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>
				<UiTooltip content="Settings">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => goto('/settings')}
							class="p-1.5 rounded-md transition-colors mx-auto {pathname === '/settings' ? 'bg-accent text-accent-foreground' : 'hover:bg-accent'}"
							aria-label="Settings"
							>
							<Settings class="w-3.5 h-3.5" />
						</button>
					{/snippet}
				</UiTooltip>
			</div>
		{/if}
	</div>

	<!-- Content -->
	{#if !collapsed}
		<div class="flex-1 overflow-auto">
			<div class="p-2">
				{#if isLoading}
					<div class="space-y-1.5">
						{#each Array(3) as _}
							<div class="p-2 border border-border/20 rounded-lg animate-pulse">
								<div class="h-3 bg-muted rounded w-3/4 mb-1.5"></div>
								<div class="h-2 bg-muted rounded w-1/2"></div>
							</div>
						{/each}
					</div>
				{:else if allPromptSets.length === 0}
					<div class="text-center py-6 text-muted-foreground">
						<p class="mb-2 text-xs">No prompt sets yet</p>
						<button
							onclick={() => goto('/create')}
							class="px-2.5 py-1.5 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all text-xs"
						>
							Create your first prompt set
						</button>
					</div>
				{:else}
					<Accordion.Root type="multiple" bind:value={accordionValue} class="space-y-1">
						{#each allPromptSets as ps (ps.id)}
							{@const revisions = revisionsByPromptSet.get(ps.id) || []}
							{@const isActive = isPromptSetActive(ps.id)}
							
							<Accordion.Item 
								value={ps.id}
								class="rounded-md border transition-all {isActive ? 'border-primary bg-primary/5' : 'border-transparent'}"
							>
								<Accordion.Header>
									<Accordion.Trigger
										class="flex w-full items-center justify-between gap-2 px-2 py-1.5 text-left hover:bg-muted/50 rounded-md transition-colors group"
									>
										<div class="flex-1 min-w-0">
											<h3 class="font-medium text-foreground text-xs truncate">
												{ps.name}
											</h3>
											<p class="text-[10px] text-muted-foreground mt-0.5">
												{ps.repositoryIds.length} {ps.repositoryIds.length === 1 ? 'repo' : 'repos'}
												· {revisions.length} {revisions.length === 1 ? 'revision' : 'revisions'}
											</p>
										</div>
										
										<div class="flex items-center gap-1">
											<UiTooltip content="Create new revision">
												{#snippet children({ props })}
													<button
														{...props}
														onclick={(e) => {
															e.stopPropagation();
															goto(`/promptsets/${ps.id}/new-revision`);
														}}
														class="p-1 rounded-md text-muted-foreground hover:text-primary hover:bg-primary/10 transition-all"
														aria-label="Create new revision"
													>
														<Plus class="w-3 h-3" />
													</button>
												{/snippet}
											</UiTooltip>
											<UiTooltip content="Delete prompt set">
												{#snippet children({ props })}
													<button
														{...props}
														onclick={(e) => {
															e.stopPropagation();
															deletePromptSetWithConfirm(ps);
														}}
														class="p-1 rounded-md text-destructive hover:bg-destructive/10 transition-all"
														aria-label="Delete prompt set"
													>
														<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
														</svg>
													</button>
												{/snippet}
											</UiTooltip>
											<ChevronDown class="w-3.5 h-3.5 text-muted-foreground transition-transform duration-200 group-data-[state=open]:rotate-180" />
										</div>
									</Accordion.Trigger>
								</Accordion.Header>
								
								<Accordion.Content class="overflow-hidden data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down">
									<div class="pl-3 pr-2 pb-1 pt-1 space-y-0.5">
										{#if revisions.length === 0}
											<div class="px-2 py-2 text-[10px] text-muted-foreground italic text-center">
												No revisions yet
											</div>
										{:else}
											{#each revisions as revision (revision.id)}
												{@const stats = revisionStats.get(revision.id)}
												{@const isRevActive = isRevisionActive(revision.id)}
												
												<button
													onclick={() => goto(`/promptsets/${ps.id}?revision=${revision.id}`)}
													class={`w-full text-left px-2 py-1 rounded-md transition-all flex items-center gap-2 ${
														isRevActive ? 'bg-primary/10 border border-primary' : 'hover:bg-muted/50 border border-transparent'
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
															<div class="flex items-center gap-1 text-primary">
															<div class="w-1 h-1 rounded-full bg-primary animate-pulse"></div>
															<span>{stats.running}</span>
															</div>
															{/if}

															{#if ps.validationPrompt && stats.completed > 0}
															<div class="flex items-center gap-1">
															<span class="text-muted-foreground">V:</span>
															<span class="font-medium text-success">
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
										{/if}
									</div>
								</Accordion.Content>
							</Accordion.Item>
						{/each}
					</Accordion.Root>
				{/if}
			</div>
		</div>
	{/if}
</div>
