<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import RepositorySelector from '$lib/components/RepositorySelector.svelte';
	import RevisionEditor from '$lib/components/ui/RevisionEditor.svelte';
	import FormField from '$lib/components/ui/FormField.svelte';
	import UiInput from '$lib/components/ui/UiInput.svelte';
	import UiTooltip from '$lib/components/ui/UiTooltip.svelte';
	import type { Repository } from '$lib/providers';
	import { api } from '$lib/api';
	import { showToast } from '$lib/ui/toast';
	import type { PromptSet, PromptRevision, Repository as DBRepository } from '$lib/types';
	import { pollExecutions } from '$lib/polling';
	import type { Execution } from '$lib/types';

	let { data } = $props();

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

	const promptsetParam = $derived(data.promptsetParam);

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

		const newRepos = new Map(repositories);
		const repoPromises = selectedRepos.map(async (repo) => {
			try {
				const dbRepo = await api.repositories.find(repo.provider, repo.fullName)
					.catch(async (findErr) => {
						return await api.repositories.create(repo.provider, repo.fullName, repo.name);
					});
				newRepos.set(dbRepo.id, dbRepo);
				return dbRepo.id;
			} catch (err) {
				return null;
			}
		});

		const repoIds = (await Promise.all(repoPromises)).filter(Boolean) as string[];
		repositories = newRepos;

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
			currentPromptSet = {
			  ...currentPromptSet,
				validationPrompt: validationPromptText
			};
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
				// Prepare executions (create worktrees) without starting them
				await api.revisions.prepare(currentRevision.id);
				showToast('Revision created successfully', 'success');
				goto(`/promptsets/${currentPromptSet.id}?revision=${currentRevision.id}`);
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
	<div class="bg-card border border-border/30 rounded-lg p-8 mb-6">
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
	<div class="bg-card border border-border/30 rounded-lg p-8 mb-6">
		<div class="flex items-center justify-between mb-5">
			<h2 class="text-2xl font-bold text-foreground">{currentPromptSet.name}</h2>
			<span class="text-sm text-muted-foreground">
				{currentPromptSet.repositoryIds.length} repositories
			</span>
		</div>

		<RevisionEditor
			bind:promptText
			bind:validationPromptText
			oldPromptText={revisions.length > 0 ? revisions[0].promptText : null}
			showValidationPrompt={revisions.length === 0}
			isProcessing={isRunning}
			onCreateOnly={() => saveRevision(false)}
			onCreateAndExecute={() => saveRevision(true)}
			onCancel={startOver}
		/>
	</div>
{/if}
