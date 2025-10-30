<script lang="ts">
	import { Checkbox } from "bits-ui"
	import { CheckCircle2 } from "lucide-svelte"
	import { onMount } from "svelte"
	import * as ipc from "$lib/ipc"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let loading = $state(true)
	let showFirstRunDialog = $state(true)

	onMount(async () => {
		try {
			showFirstRunDialog = await ipc.getShowFirstRunDialog()
		} catch (error) {
			onStatusChange({
				type: "error",
				message: error instanceof Error ? error.message : String(error),
			})
		} finally {
			loading = false
		}
	})

	async function handleToggle(checked: boolean) {
		try {
			await ipc.setShowFirstRunDialog(checked)
			showFirstRunDialog = checked
			onStatusChange({
				type: "success",
				message: "Startup preference saved",
			})
		} catch (error) {
			onStatusChange({
				type: "error",
				message: error instanceof Error ? error.message : String(error),
			})
		}
	}
</script>

<div>
	<h3 class="text-lg font-semibold mb-4">Startup</h3>
	<p class="text-sm text-muted-foreground mb-6">Configure what happens when you start Maestro</p>

	{#if loading}
		<div class="text-sm text-muted-foreground">Loading...</div>
	{:else}
		<label class="flex items-start gap-3 cursor-pointer group">
			<Checkbox.Root
				bind:checked={showFirstRunDialog}
				onCheckedChange={handleToggle}
				class="h-4 w-4 rounded border border-border bg-background flex items-center justify-center data-[state=checked]:bg-primary data-[state=checked]:border-primary mt-0.5"
			>
				{#snippet children({ checked })}
					{#if checked}
						<CheckCircle2 class="h-3 w-3 text-primary-foreground" />
					{/if}
				{/snippet}
			</Checkbox.Root>
			<div>
				<div class="text-sm font-medium group-hover:text-foreground transition-colors">
					Show welcome dialog on startup
				</div>
				<div class="text-xs text-muted-foreground mt-1">
					Display the first-run dialog with system checks and onboarding information when Maestro
					starts
				</div>
			</div>
		</label>
	{/if}
</div>
