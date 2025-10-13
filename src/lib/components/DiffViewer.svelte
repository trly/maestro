<script lang="ts">
	import { api } from '$lib/api'
	import { processPatchDiff } from '$lib/diff'
	import { fetchDiff, fetchFileDiff, clearDiffCache } from '$lib/stores/diffStore'
	import Modal from '$lib/components/ui/Modal.svelte'
	import Badge from '$lib/components/ui/Badge.svelte'
	import FileList from '$lib/components/diff/FileList.svelte'
	import DiffTabs from '$lib/components/diff/DiffTabs.svelte'

	type ModifiedFile = {
		status: string
		path: string
		selected: boolean
	}

	let { executionId, open = $bindable(false) }: { executionId: string; open?: boolean } = $props()

	let files = $state<ModifiedFile[]>([])
	let selectedFileIndex = $state<number | null>(null)
	let diff = $state<string>('')
	let loading = $state(false)
	let committing = $state(false)
	let hasSessionId = $state(false)
	let commitStatus = $state<'uncommitted' | 'committed'>('uncommitted')
	let commitSha = $state<string | null>(null)
	let parentSha = $state<string | null>(null)
	let branch = $state<string | null>(null)

	let diffItems = $derived(diff ? processPatchDiff(diff) : [])
	let selectedFile = $derived(
		selectedFileIndex !== null && selectedFileIndex >= 0 ? files[selectedFileIndex] : null
	)
	let isBinaryFile = $derived(diff.includes('Binary files') || diff.includes('GIT binary patch'))

	async function loadFiles() {
		loading = true
		try {
			const [filesData, executionData] = await Promise.all([
				fetchDiff(executionId),
				api.executions.get(executionId)
			])

			files = filesData.files.map((f: any) => ({ ...f, selected: true }))
			hasSessionId = !!executionData.sessionId
			commitStatus = (
				filesData.source === 'committed' ? 'committed' : 'uncommitted'
			) as 'committed' | 'uncommitted'
			commitSha = filesData.commitSha || null
			parentSha = executionData.parentSha || null
			branch = executionData.branch || null

			if (files.length > 0) {
				await selectFile(0)
			}
		} catch (error) {
		} finally {
			loading = false
		}
	}

	async function selectFile(index: number) {
		selectedFileIndex = index
		const file = files[index]
		if (!file) return

		loading = true
		try {
			diff = await fetchFileDiff(executionId, file.path)
		} catch (error) {
			diff = ''
		} finally {
			loading = false
		}
	}

	function toggleFile(index: number) {
		const file = files[index]
		if (file) {
			file.selected = !file.selected
		}
	}

	async function commitSelectedFiles() {
		const selectedFiles = files.filter((f) => f.selected).map((f) => f.path)
		if (selectedFiles.length === 0) {
			alert('Please select at least one file to commit')
			return
		}

		committing = true
		// Close immediately to show progress in execution row
		open = false
		try {
			await api.executions.commit(executionId, selectedFiles)
			clearDiffCache(executionId)
		} catch (error) {
			alert('Failed to commit files')
		} finally {
			committing = false
		}
	}

	$effect(() => {
		if (open) {
			loadFiles()
		}
	})
</script>

<Modal bind:open title="Review Changes">
	{#snippet children()}
		<div class="flex-1 flex flex-row min-h-0">
			<FileList 
				bind:files 
				bind:selectedIndex={selectedFileIndex} 
				onselect={selectFile} 
				ontoggle={toggleFile}
				readonly={commitStatus === 'committed'}
			/>

			<div class="flex-1 flex flex-col min-w-0 overflow-hidden p-6">
				<div class="flex flex-col gap-2 mb-4">
					<div class="flex items-center gap-3">
						{#if selectedFile}
							<h3 class="font-semibold">{selectedFile.path}</h3>
						{/if}
						{#if commitStatus === 'uncommitted'}
							<Badge type="uncommitted" text="Uncommitted changes" />
						{:else}
							<Badge
								type="committed"
								text="Committed {commitSha ? commitSha.slice(0, 7) : ''}"
							/>
						{/if}
					</div>
					<div class="flex items-center gap-4 text-xs text-muted-foreground">
						{#if parentSha}
							<div>
								Parent: <span class="font-mono">{parentSha.slice(0, 7)}</span>
							</div>
						{/if}
						{#if branch}
							<div>
								Branch: <span class="font-mono">{branch}</span>
							</div>
						{/if}
					</div>
				</div>

				<div class="flex-1 overflow-y-auto">
					{#if loading && !selectedFile}
						<p class="text-sm text-muted-foreground">Loading...</p>
					{:else if !selectedFile}
						<p class="text-sm text-muted-foreground">Select a file to view diff</p>
					{:else if loading}
						<p class="text-sm text-muted-foreground">Loading diff...</p>
					{:else if diffItems.length > 0}
						<DiffTabs items={diffItems} {commitStatus} {parentSha} {commitSha} fileStatus={selectedFile?.status} />
					{:else}
						<p class="text-sm text-muted-foreground">
							No changes to display{isBinaryFile ? ' (binary file)' : ''}
						</p>
					{/if}
				</div>
			</div>
		</div>
	{/snippet}

	{#snippet footer()}
		<div class="flex justify-end gap-2">
			<button
				class="px-4 py-2 border rounded hover:bg-gray-50"
				onclick={() => (open = false)}>Close</button
			>
			{#if hasSessionId}
				<button
					class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
					onclick={commitSelectedFiles}
					disabled={committing ||
						files.filter((f) => f.selected).length === 0 ||
						commitStatus === 'committed'}
					title={commitStatus === 'committed' ? 'Changes already committed' : ''}
				>
					{committing ? 'Committing...' : `Commit ${files.filter((f) => f.selected).length} file(s)`}
				</button>
			{/if}
		</div>
	{/snippet}
</Modal>
