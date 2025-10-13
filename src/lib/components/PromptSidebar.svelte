<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { Plus, PanelLeftClose, PanelLeftOpen, Settings } from 'lucide-svelte';
	import UiTooltip from '$lib/components/ui/UiTooltip.svelte';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import { confirm } from '$lib/ui/confirm';
	import type { PromptSet } from '$lib/types';

	let { collapsed = false, onToggleCollapse } = $props();

	let allPromptSets = $state<PromptSet[]>([]);
	let isLoading = $state(true);

	async function loadPromptSets() {
		try {
			allPromptSets = await api.promptSets.getAll();
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

	onMount(loadPromptSets);
	
	$effect(() => {
		$page.url;
		loadPromptSets();
	});

	const isActive = (id: string) => $page.url.pathname.startsWith(`/promptsets/${id}`);
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
						class="p-1.5 rounded-md transition-colors {$page.url.pathname === '/settings' ? 'bg-accent text-accent-foreground' : 'hover:bg-accent'}"
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
							class="p-1.5 rounded-md transition-colors mx-auto {$page.url.pathname === '/settings' ? 'bg-accent text-accent-foreground' : 'hover:bg-accent'}"
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
		<div class="flex-1 overflow-y-auto p-2 space-y-1">
		{#if isLoading}
			<div class="space-y-1.5">
				{#each Array(3) as _}
					<div class="p-2 border border-border/20 rounded-lg animate-pulse">
						<div class="h-3 bg-gray-200 rounded w-3/4 mb-1.5"></div>
						<div class="h-2 bg-gray-200 rounded w-1/2"></div>
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
			{#each allPromptSets as ps}
				<div class="group rounded-md border cursor-pointer transition-all {isActive(ps.id) ? 'bg-primary/10 border-primary' : 'border-transparent hover:border-muted hover:bg-muted/50'}">
					<div class="flex items-start justify-between gap-1.5 p-2">
						<button
							onclick={() => goto(`/promptsets/${ps.id}`)}
							class="flex-1 text-left min-w-0"
						>
							<h3 class="font-medium text-foreground text-xs truncate">
								{ps.name}
							</h3>
							<p class="text-[10px] text-muted-foreground mt-0.5">
								{ps.repositoryIds.length} {ps.repositoryIds.length === 1 ? 'repo' : 'repos'}
							</p>
						</button>
						<UiTooltip content="Delete prompt set">
							{#snippet children({ props })}
								<button
									{...props}
									onclick={(e) => {
										e.stopPropagation();
										deletePromptSetWithConfirm(ps);
									}}
									class="opacity-0 group-hover:opacity-100 p-1 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all flex-shrink-0"
									aria-label="Delete prompt set"
								>
									<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
									</svg>
								</button>
							{/snippet}
						</UiTooltip>
					</div>
				</div>
			{/each}
		{/if}
		</div>
	{/if}
</div>
