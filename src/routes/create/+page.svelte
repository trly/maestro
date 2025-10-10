<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import RepositorySelector from '$lib/components/RepositorySelector.svelte';
	import PromptDiff from '$lib/components/PromptDiff.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import UiInput from '$lib/components/ui/UiInput.svelte';
	import UiTextarea from '$lib/components/ui/UiTextarea.svelte';
	import UiTooltip from '$lib/components/ui/UiTooltip.svelte';
	import type { Repository } from '$lib/providers';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import type { PromptSet, PromptRevision, Repository as DBRepository } from '$lib/types';
	import { pollExecutions } from '$lib/polling';
	import type { Execution } from '$lib/types';

	let step = $state<'create' | 'prompt'>('create');
	let promptSetName = $state('');
	let selectedRepos = $state<Repository[]>([]);
	let promptText = $state('');
	let validationPromptText = $state('');
	let isRunning = $state(false);
	let currentPromptSet = $state<PromptSet | null>(null);
	let revisions = $state<PromptRevision[]>([]);
	let repositories = $state<Map<string, DBRepository>>(new Map());
	let stopPolling = $state<(() => void) | null>(null);

	const promptsetParam = $derived($page.url.searchParams.get('promptset'));

	onMount(async () => {
		if (promptsetParam) {
			try {
				currentPromptSet = await api.promptSets.get(promptsetParam);
				revisions = await api.promptSets.getRevisions(promptsetParam);
				if (revisions.length > 0) {
					promptText = revisions[0].promptText;
				}
				step = 'prompt';
			} catch (err) {
				showToast('Failed to load prompt set: ' + err, 'error');
				goto('/');
			}
		}
	});

	async function createPromptSet() {
		if (!promptSetName.trim() || selectedRepos.length === 0) {
			return;
		}

		const repoPromises = selectedRepos.map(async (repo) => {
			try {
				const dbRepo = await api.repositories.find(repo.provider, repo.fullName)
					.catch(async (findErr) => {
						return await api.repositories.create(repo.provider, repo.fullName, repo.name);
					});
				repositories.set(dbRepo.id, dbRepo);
				return dbRepo.id;
			} catch (err) {
				return null;
			}
		});

		const repoIds = (await Promise.all(repoPromises)).filter(Boolean) as string[];

		if (repoIds.length === 0) {
			showToast('Failed to persist selected repositories', 'error');
			return;
		}

		try {
			currentPromptSet = await api.promptSets.create(
				promptSetName, 
				repoIds, 
				null
			);
			revisions = await api.promptSets.getRevisions(currentPromptSet.id);
			step = 'prompt';
		} catch (err) {
			showToast('Failed to create prompt set: ' + err, 'error');
		}
	}

	async function saveRevision(executeAfterCreate = true) {
		if (!currentPromptSet || !promptText.trim()) {
			return;
		}

		const parentId = revisions.length > 0 ? revisions[0].id : null;
		
		isRunning = true;
		try {
			if (revisions.length === 0 && validationPromptText.trim()) {
				await api.promptSets.update(currentPromptSet.id, {
					validationPrompt: validationPromptText
				});
				currentPromptSet.validationPrompt = validationPromptText;
			}
			
			const currentRevision = await api.revisions.create(
				currentPromptSet.id,
				promptText,
				parentId
			);
			
			revisions = await api.promptSets.getRevisions(currentPromptSet.id);
			
			if (executeAfterCreate) {
				await api.revisions.execute(currentRevision.id);
				showToast('Execution started', 'info');
				
				stopPolling = pollExecutions(currentRevision.id, (executions: Execution[]) => {
					const hasActiveExecution = executions.some(e => 
						e.status === 'pending' || 
						e.status === 'running' || 
						e.validationStatus === 'pending' || 
						e.validationStatus === 'running'
					);
					
					if (!hasActiveExecution && stopPolling) {
						stopPolling();
						stopPolling = null;
					}
				});
				
				goto(`/promptsets/${currentPromptSet.id}?revision=${currentRevision.id}`);
			} else {
				showToast('Revision created successfully', 'success');
				goto(`/promptsets/${currentPromptSet.id}`);
			}
		} catch (err) {
			showToast('Failed to save and execute revision: ' + err, 'error');
		} finally {
			isRunning = false;
		}
	}

	function startOver() {
		goto('/');
	}
</script>

{#if step === 'create'}
	<div class="bg-card border border-border rounded-lg p-8 mb-6">
		<h2 class="text-2xl font-bold text-foreground mb-5">Create Prompt Set</h2>
		
		<FormField
			label="Name"
			htmlFor="promptset-name"
			required
			helperText="A descriptive name for this prompt set"
		>
			<UiInput
				id="promptset-name"
				bind:value={promptSetName}
				placeholder="e.g., Update Documentation"
			/>
		</FormField>

		<div class="mb-6">
			<div class="flex items-center justify-between mb-3">
				<h3 class="text-sm font-semibold text-foreground">Repositories</h3>
				{#if selectedRepos.length > 0}
				<UiTooltip content="Number of repositories selected for this prompt set">
				{#snippet children({ props })}
				<span {...props} class="bg-accent text-accent-foreground px-3 py-1 rounded-full text-sm font-semibold cursor-help">
				  {selectedRepos.length} selected
				  </span>
				  {/snippet}
				</UiTooltip>
			{/if}
			</div>
			<RepositorySelector bind:selectedRepos />
		</div>

		<button
			onclick={createPromptSet}
			disabled={!promptSetName.trim() || selectedRepos.length === 0}
			class="w-full px-8 py-3.5 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
		>
			Continue to Prompt
		</button>
	</div>
{/if}

{#if step === 'prompt' && currentPromptSet}
	<div class="bg-card border border-border rounded-lg p-8 mb-6">
		<div class="flex items-center justify-between mb-5">
			<h2 class="text-2xl font-bold text-foreground">{currentPromptSet.name}</h2>
			<span class="text-sm text-muted-foreground">
				{currentPromptSet.repositoryIds.length} repositories
			</span>
		</div>

		{#if revisions.length === 0}
			<FormField
				label="Initial Prompt"
				htmlFor="initial-prompt"
				required
				helperText="This prompt will be executed across all repositories in the set"
			>
				<UiTextarea
					id="initial-prompt"
					bind:value={promptText}
					placeholder="Enter your prompt to execute across all repositories..."
					rows={12}
				/>
			</FormField>
			
			<FormField
				label="Validation Prompt"
				htmlFor="validation-prompt"
				helperText="This prompt will run against the modified branch after each successful execution"
			>
				<UiTextarea
					id="validation-prompt"
					bind:value={validationPromptText}
					placeholder="Enter a prompt to validate each execution after it completes..."
					rows={6}
				/>
			</FormField>
		{:else}
			<PromptDiff
				oldText={revisions[0].promptText}
				newText={promptText}
				onupdate={(text) => promptText = text}
			/>
		{/if}

		<div class="flex flex-col sm:flex-row gap-3 mt-5">
		<button
		onclick={() => saveRevision(true)}
		disabled={!promptText.trim() || isRunning}
		class="flex-1 px-8 py-3.5 bg-primary text-primary-foreground rounded-md hover:opacity-90 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
		>
		{isRunning ? 'Creating...' : (revisions.length === 0 ? 'Create & Execute' : 'Save Revision & Execute')}
		</button>
		<button
		onclick={() => saveRevision(false)}
		disabled={!promptText.trim() || isRunning}
		class="w-full sm:w-auto px-6 py-3.5 border border-primary text-primary rounded-md hover:bg-primary/10 transition-all disabled:opacity-50 disabled:cursor-not-allowed font-semibold"
		>
		{revisions.length === 0 ? 'Create Only' : 'Save Only'}
		</button>
		<button
		onclick={startOver}
		class="w-full sm:w-auto px-6 py-3.5 border border-border text-foreground rounded-md hover:bg-accent transition-all font-semibold"
		>
		Cancel
			</button>
		</div>
	</div>
{/if}
