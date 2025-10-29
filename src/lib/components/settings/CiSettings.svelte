<script lang="ts">
	import { settingsStore } from "$lib/stores/settingsStore"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let ciThreshold = $state(10)
	let ciThresholdInput = $state("10")
	let maxConcurrentExecutions = $state(10)
	let maxConcurrentInput = $state("10")
	let editingCiThreshold = $state(false)
	let editingMaxConcurrent = $state(false)

	$effect(() => {
		const settings = $settingsStore
		ciThreshold = settings.ciStuckThresholdMinutes
		ciThresholdInput = settings.ciStuckThresholdMinutes.toString()
		maxConcurrentExecutions = settings.maxConcurrentExecutions
		maxConcurrentInput = settings.maxConcurrentExecutions.toString()
	})

	async function saveCiThreshold() {
		try {
			const value = parseInt(ciThresholdInput)
			if (isNaN(value) || value < 1) {
				onStatusChange({ type: "error", message: "CI timeout must be at least 1 minute" })
				return
			}
			await settingsStore.setCiStuckThreshold(value)
			editingCiThreshold = false
			onStatusChange({ type: "success", message: "CI timeout setting saved" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	function cancelCiThresholdEdit() {
		ciThresholdInput = ciThreshold.toString()
		editingCiThreshold = false
	}

	async function saveMaxConcurrent() {
		try {
			const value = parseInt(maxConcurrentInput)
			if (isNaN(value) || value < 1) {
				onStatusChange({ type: "error", message: "Max concurrent executions must be at least 1" })
				return
			}
			await settingsStore.setMaxConcurrentExecutions(value)
			editingMaxConcurrent = false
			onStatusChange({ type: "success", message: "Concurrency limit saved" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	function cancelMaxConcurrentEdit() {
		maxConcurrentInput = maxConcurrentExecutions.toString()
		editingMaxConcurrent = false
	}
</script>

<div>
	<h3 class="text-lg font-semibold mb-4">CI Monitoring</h3>
	<p class="text-sm text-muted-foreground mb-6">Configure CI status checking behavior</p>

	<div class="space-y-6">
		<div>
			<label for="ci-threshold-input" class="block text-sm font-medium mb-2">CI Stuck Timeout</label
			>
			<p class="text-xs text-muted-foreground mb-2">
				How long (in minutes) before pending CI checks are marked as "No CI configured"
			</p>
			<div class="flex flex-col sm:flex-row gap-2">
				{#if editingCiThreshold}
					<div class="flex-1">
						<input
							id="ci-threshold-input"
							type="number"
							bind:value={ciThresholdInput}
							min="1"
							placeholder="Minutes"
							class="w-full px-3 py-2 border rounded-md bg-background"
						/>
					</div>
					<button
						type="button"
						onclick={saveCiThreshold}
						class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
					>
						Save
					</button>
					<button
						type="button"
						onclick={cancelCiThresholdEdit}
						class="px-3 py-2 border rounded-md hover:bg-muted"
					>
						Cancel
					</button>
				{:else}
					<div class="flex-1">
						<input
							type="text"
							value="{ciThreshold} minutes"
							disabled
							class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
						/>
					</div>
					<button
						type="button"
						onclick={() => (editingCiThreshold = true)}
						class="px-3 py-2 border rounded-md hover:bg-muted"
					>
						Update
					</button>
				{/if}
			</div>
			<p class="text-xs text-muted-foreground mt-2">
				Recommended: 10-15 minutes to allow slow CI workflows to start
			</p>
		</div>

		<div>
			<label for="max-concurrent-input" class="block text-sm font-medium mb-2"
				>Max Concurrent Executions</label
			>
			<p class="text-xs text-muted-foreground mb-2">
				Maximum number of executions that can run simultaneously
			</p>
			<div class="flex flex-col sm:flex-row gap-2">
				{#if editingMaxConcurrent}
					<div class="flex-1">
						<input
							id="max-concurrent-input"
							type="number"
							bind:value={maxConcurrentInput}
							min="1"
							placeholder="Count"
							class="w-full px-3 py-2 border rounded-md bg-background"
						/>
					</div>
					<button
						type="button"
						onclick={saveMaxConcurrent}
						class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
					>
						Save
					</button>
					<button
						type="button"
						onclick={cancelMaxConcurrentEdit}
						class="px-3 py-2 border rounded-md hover:bg-muted"
					>
						Cancel
					</button>
				{:else}
					<div class="flex-1">
						<input
							type="text"
							value="{maxConcurrentExecutions} executions"
							disabled
							class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
						/>
					</div>
					<button
						type="button"
						onclick={() => (editingMaxConcurrent = true)}
						class="px-3 py-2 border rounded-md hover:bg-muted"
					>
						Update
					</button>
				{/if}
			</div>
			<p class="text-xs text-muted-foreground mt-2">
				Default: 10 executions. Increase for faster bulk operations, decrease to limit resource
				usage.
			</p>
		</div>
	</div>
</div>
