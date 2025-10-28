<script lang="ts">
	import { goto } from "$app/navigation"
	import { onMount } from "svelte"
	import RevisionEditor from "$lib/components/ui/RevisionEditor.svelte"
	import { api } from "$lib/api"
	import { showToast } from "$lib/ui/toast"
	import { sidebarStore } from "$lib/stores/sidebarStore"
	import type { PromptSet, PromptRevision } from "$lib/types"

	let { data } = $props()

	let promptSet = $state<PromptSet | null>(null)
	let revisions = $state<PromptRevision[]>([])
	let parentRevision = $state<PromptRevision | null>(null)
	let promptText = $state("")
	let isLoading = $state(true)
	let isSaving = $state(false)
	let hasChanges = $derived(
		!parentRevision || promptText.trim() !== parentRevision.promptText.trim()
	)

	async function loadData() {
		const promptSetId = data.promptsetId
		if (!promptSetId) {
			showToast("Invalid prompt set ID", "error")
			isLoading = false
			return
		}

		try {
			;[promptSet, revisions] = await Promise.all([
				api.promptSets.get(promptSetId),
				api.promptSets.getRevisions(promptSetId),
			])

			// Get the most recent revision as parent
			if (revisions.length > 0) {
				parentRevision = revisions[0] // Already sorted by createdAt desc
				promptText = parentRevision.promptText
			}
		} catch (err) {
			showToast("Failed to load prompt set: " + err, "error")
		} finally {
			isLoading = false
		}
	}

	async function saveRevision(executeAfterCreate = false) {
		if (!promptSet) return
		if (!promptText.trim()) {
			showToast("Prompt text cannot be empty", "error")
			return
		}

		isSaving = true
		try {
			const newRevision = await api.promptSets.createRevision(
				promptSet.id,
				promptText,
				parentRevision?.id || null
			)

			if (executeAfterCreate) {
				await api.revisions.execute(newRevision.id)
				showToast("Revision created and execution started", "success")
			} else {
				// Prepare executions (create worktrees) without starting them
				await api.revisions.prepare(newRevision.id)
				showToast("Revision created successfully", "success")
			}

			sidebarStore.refresh() // Trigger sidebar to reload
			goto(`/promptsets/${promptSet.id}?revision=${newRevision.id}`)
		} catch (err) {
			showToast("Failed to create revision: " + err, "error")
		} finally {
			isSaving = false
		}
	}

	function cancel() {
		goto(`/promptsets/${data.promptsetId}`)
	}

	onMount(loadData)
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
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-6">
			<RevisionEditor
				bind:promptText
				oldPromptText={parentRevision?.promptText || null}
				{hasChanges}
				isProcessing={isSaving}
				onCreateOnly={() => saveRevision(false)}
				onCreateAndExecute={() => saveRevision(true)}
				onCancel={cancel}
			/>
		</div>
	{/if}
</div>
