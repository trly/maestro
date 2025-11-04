<script lang="ts">
	import { tokenStore } from "$lib/tokenStore"
	import * as ipc from "$lib/ipc"
	import { onMount } from "svelte"
	import { CheckCircle2, XCircle, Loader2 } from "lucide-svelte"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let ampToken = $state("")
	let ampTokenMasked = $state("")
	let editingToken = $state(false)
	let loading = $state(true)
	let healthCheck = $state<ipc.HealthCheckResult | null>(null)
	let checking = $state(false)

	onMount(async () => {
		try {
			const allTokens = await tokenStore.getAllTokensMasked()
			ampTokenMasked = allTokens.ampToken || ""
		} finally {
			loading = false
		}
	})

	async function testConnection() {
		checking = true
		healthCheck = null
		try {
			healthCheck = await ipc.healthCheckAmpToken()
		} catch (error) {
			healthCheck = {
				success: false,
				username: null,
				error: error instanceof Error ? error.message : String(error),
			}
		} finally {
			checking = false
		}
	}

	async function saveToken() {
		try {
			if (ampToken.trim()) {
				await tokenStore.setToken("amp_token", ampToken.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				ampTokenMasked = allTokens.ampToken || ""
				ampToken = ""
				editingToken = false
			} else {
				await tokenStore.deleteToken("amp_token")
				ampTokenMasked = ""
				editingToken = false
			}
			onStatusChange({ type: "success", message: "Amp token saved securely to system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function deleteTokenValue() {
		try {
			await tokenStore.deleteToken("amp_token")
			ampToken = ""
			ampTokenMasked = ""
			editingToken = false
			onStatusChange({ type: "success", message: "Amp token deleted from system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}
</script>

<div>
	<h3 class="text-lg font-semibold mb-4">Amp Configuration</h3>
	<p class="text-sm text-muted-foreground mb-6">
		Configure Amp AI agent for prompt execution and analysis
	</p>

	{#if loading}
		<p class="text-sm text-muted-foreground">Loading...</p>
	{:else}
		<div class="space-y-6">
			<div>
				<label for="amp-token" class="block text-sm font-medium mb-2">Amp API Token</label>
				<p class="text-xs text-muted-foreground mb-2">Required for executing prompts with Amp</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingToken}
						<div class="flex-1">
							<input
								id="amp-token"
								type="text"
								bind:value={ampToken}
								placeholder="Enter Amp API token"
								class="w-full px-3 py-2 border rounded-md bg-background"
							/>
						</div>
						<button
							type="button"
							onclick={saveToken}
							class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
						>
							Save
						</button>
						<button
							type="button"
							onclick={() => {
								editingToken = false
								ampToken = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={ampTokenMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingToken = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{ampTokenMasked ? "Update" : "Set"}
						</button>
						{#if ampTokenMasked}
							<button
								type="button"
								onclick={deleteTokenValue}
								class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
							>
								Delete
							</button>
						{/if}
					{/if}
				</div>
				<p class="text-xs text-muted-foreground mt-2">
					Generate at: <a
						href="https://ampcode.com/settings/profile"
						target="_blank"
						class="text-primary hover:underline">ampcode.com/settings/profile</a
					>
				</p>
				{#if ampTokenMasked && !editingToken}
					<div class="mt-3 flex items-center gap-2">
						<button
							type="button"
							onclick={testConnection}
							disabled={checking}
							class="px-3 py-1.5 text-sm border rounded-md hover:bg-muted transition-colors disabled:opacity-50"
						>
							{checking ? "Testing..." : "Test Connection"}
						</button>
						{#if checking}
							<Loader2 class="w-4 h-4 animate-spin text-primary" />
						{:else if healthCheck}
							{#if healthCheck.success}
								<div class="flex items-center gap-1.5 text-success">
									<CheckCircle2 class="w-4 h-4" />
									<span class="text-sm">{healthCheck.username}</span>
								</div>
							{:else}
								<div class="flex items-center gap-1.5 text-destructive">
									<XCircle class="w-4 h-4" />
									<span class="text-sm">{healthCheck.error}</span>
								</div>
							{/if}
						{/if}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
