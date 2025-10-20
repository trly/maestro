<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import PromptDiff from '$lib/components/PromptDiff.svelte';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import { sidebarStore } from '$lib/stores/sidebarStore';
	import type { PromptSet, PromptRevision } from '$lib/types';

	let promptSetId = $derived($page.params.id);
	let promptSet = $state<PromptSet | null>(null);
	let revisions = $state<PromptRevision[]>([]);
	let parentRevision = $state<PromptRevision | null>(null);
	let newPromptText = $state('');
	let isLoading = $state(true);
	let isSaving = $state(false);
	let hasChanges = $derived(
		!parentRevision || newPromptText.trim() !== parentRevision.promptText.trim()
	);

	async function loadData() {
		if (!promptSetId) {
			showToast('Invalid prompt set ID', 'error');
			isLoading = false;
			return;
		}
		
		try {
			[promptSet, revisions] = await Promise.all([
				api.promptSets.get(promptSetId),
				api.promptSets.getRevisions(promptSetId)
			]);

			// Get the most recent revision as parent
			if (revisions.length > 0) {
				parentRevision = revisions[0]; // Already sorted by createdAt desc
				newPromptText = parentRevision.promptText;
			}
		} catch (err) {
			showToast('Failed to load prompt set: ' + err, 'error');
		} finally {
			isLoading = false;
		}
	}

	function handlePromptUpdate(text: string) {
		newPromptText = text;
	}

	async function saveRevision() {
		if (!promptSet) return;
		if (!newPromptText.trim()) {
			showToast('Prompt text cannot be empty', 'error');
			return;
		}

		isSaving = true;
		try {
			const newRevision = await api.promptSets.createRevision(
				promptSet.id,
				newPromptText,
				parentRevision?.id || null
			);
			showToast('Revision created successfully', 'success');
			sidebarStore.refresh(); // Trigger sidebar to reload
			goto(`/promptsets/${promptSet.id}?revision=${newRevision.id}`);
		} catch (err) {
			showToast('Failed to create revision: ' + err, 'error');
		} finally {
			isSaving = false;
		}
	}

	function cancel() {
		goto(`/promptsets/${promptSetId}`);
	}

	onMount(loadData);
</script>

<div class="h-full flex flex-col bg-background">
	{#if isLoading}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-muted-foreground">Loading...</div>
		</div>
	{:else if !promptSet}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-destructive">Prompt set not found</div>
		</div>
	{:else}
		<!-- Header -->
		<div class="flex items-center justify-between px-6 py-4 border-b border-border">
			<div>
				<h1 class="text-lg font-semibold text-foreground">Create New Revision</h1>
				<p class="text-sm text-muted-foreground mt-1">
					{promptSet.name}
					{#if parentRevision}
						<span class="text-muted-foreground/60">
							· Based on revision from {new Date(parentRevision.createdAt).toLocaleString()}
						</span>
					{/if}
				</p>
			</div>
			<div class="flex items-center gap-2">
				<button
					onclick={cancel}
					class="px-3 py-1.5 rounded-md border border-border hover:bg-muted transition-colors text-sm"
					disabled={isSaving}
				>
					Cancel
				</button>
				<button
					onclick={saveRevision}
					class="px-3 py-1.5 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors text-sm disabled:opacity-50 disabled:cursor-not-allowed"
					disabled={isSaving || !newPromptText.trim() || !hasChanges}
					title={!hasChanges ? 'No changes to prompt text' : ''}
				>
					{isSaving ? 'Saving...' : 'Create Revision'}
				</button>
			</div>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-6">
			{#if parentRevision}
				<PromptDiff
					oldText={parentRevision.promptText}
					newText={newPromptText}
					onupdate={handlePromptUpdate}
				/>
			{:else}
				<div class="rounded-md border border-primary/30 overflow-hidden shadow-sm">
					<div class="bg-primary/10 px-4 py-2 border-b border-primary/10">
						<h4 class="text-xs font-semibold text-foreground">New Prompt (No Previous Version)</h4>
					</div>
					<textarea
						bind:value={newPromptText}
						placeholder="Enter your prompt..."
						class="w-full h-96 px-4 py-3 font-mono text-sm border-0 focus:ring-0 resize-none bg-background text-foreground"
					></textarea>
				</div>
			{/if}
		</div>
	{/if}
</div>
