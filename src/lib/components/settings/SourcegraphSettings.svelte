<script lang="ts">
	import { tokenStore } from "$lib/tokenStore"
	import * as ipc from "$lib/ipc"
	import { onMount } from "svelte"
	import { CheckCircle2, XCircle, Loader2 } from "lucide-svelte"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let sourcegraphEndpoint = $state("")
	let sourcegraphToken = $state("")
	let sourcegraphEndpointMasked = $state("")
	let sourcegraphTokenMasked = $state("")
	let editingEndpoint = $state(false)
	let editingToken = $state(false)
	let loading = $state(true)
	let healthCheck = $state<ipc.HealthCheckResult | null>(null)
	let checking = $state(false)

	onMount(async () => {
		try {
			const allTokens = await tokenStore.getAllTokensMasked()
			sourcegraphEndpointMasked = allTokens.sourcegraphEndpoint || ""
			sourcegraphTokenMasked = allTokens.sourcegraphToken || ""
		} finally {
			loading = false
		}
	})

	async function testConnection() {
		checking = true
		healthCheck = null
		try {
			healthCheck = await ipc.healthCheckSourcegraph()
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

	async function saveEndpoint() {
		try {
			if (sourcegraphEndpoint.trim()) {
				await tokenStore.setToken("sourcegraph_endpoint", sourcegraphEndpoint.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				sourcegraphEndpointMasked = allTokens.sourcegraphEndpoint || ""
				sourcegraphEndpoint = ""
				editingEndpoint = false
			} else {
				await tokenStore.deleteToken("sourcegraph_endpoint")
				sourcegraphEndpointMasked = ""
				editingEndpoint = false
			}
			onStatusChange({ type: "success", message: "Sourcegraph endpoint saved securely" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function saveToken() {
		try {
			if (sourcegraphToken.trim()) {
				await tokenStore.setToken("sourcegraph_token", sourcegraphToken.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				sourcegraphTokenMasked = allTokens.sourcegraphToken || ""
				sourcegraphToken = ""
				editingToken = false
			} else {
				await tokenStore.deleteToken("sourcegraph_token")
				sourcegraphTokenMasked = ""
				editingToken = false
			}
			onStatusChange({
				type: "success",
				message: "Sourcegraph token saved securely to system keyring",
			})
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function deleteEndpoint() {
		try {
			await tokenStore.deleteToken("sourcegraph_endpoint")
			sourcegraphEndpoint = ""
			sourcegraphEndpointMasked = ""
			editingEndpoint = false
			onStatusChange({ type: "success", message: "Sourcegraph endpoint deleted" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}

	async function deleteToken() {
		try {
			await tokenStore.deleteToken("sourcegraph_token")
			sourcegraphToken = ""
			sourcegraphTokenMasked = ""
			editingToken = false
			onStatusChange({ type: "success", message: "Sourcegraph token deleted from system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}
</script>

<div>
	<h3 class="text-lg font-semibold mb-4">Sourcegraph</h3>
	<p class="text-sm text-muted-foreground mb-6">
		Configure Sourcegraph for repository search and code intelligence
	</p>

	{#if loading}
		<p class="text-sm text-muted-foreground">Loading...</p>
	{:else}
		<div class="space-y-6">
			<div>
				<label for="sourcegraph-endpoint" class="block text-sm font-medium mb-2">Instance URL</label
				>
				<p class="text-xs text-muted-foreground mb-2">Your Sourcegraph instance endpoint</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingEndpoint}
						<div class="flex-1">
							<input
								id="sourcegraph-endpoint"
								type="text"
								bind:value={sourcegraphEndpoint}
								placeholder="https://sourcegraph.com"
								class="w-full px-3 py-2 border rounded-md bg-background"
							/>
						</div>
						<button
							type="button"
							onclick={saveEndpoint}
							class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
						>
							Save
						</button>
						<button
							type="button"
							onclick={() => {
								editingEndpoint = false
								sourcegraphEndpoint = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={sourcegraphEndpointMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingEndpoint = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{sourcegraphEndpointMasked ? "Update" : "Set"}
						</button>
						{#if sourcegraphEndpointMasked}
							<button
								type="button"
								onclick={deleteEndpoint}
								class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
							>
								Delete
							</button>
						{/if}
					{/if}
				</div>
				<p class="text-xs text-muted-foreground mt-2">
					Example: <code>https://sourcegraph.com</code> or your self-hosted instance
				</p>
			</div>

			<div>
				<label for="sourcegraph-token" class="block text-sm font-medium mb-2">Access Token</label>
				<p class="text-xs text-muted-foreground mb-2">Required for repository search API access</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingToken}
						<div class="flex-1">
							<input
								id="sourcegraph-token"
								type="text"
								bind:value={sourcegraphToken}
								placeholder="Enter Sourcegraph access token"
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
								sourcegraphToken = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={sourcegraphTokenMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingToken = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{sourcegraphTokenMasked ? "Update" : "Set"}
						</button>
						{#if sourcegraphTokenMasked}
							<button
								type="button"
								onclick={deleteToken}
								class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
							>
								Delete
							</button>
						{/if}
					{/if}
				</div>
				<p class="text-xs text-muted-foreground mt-2">
					Generate at your Sourcegraph instance: Settings → Access tokens
				</p>
				{#if sourcegraphTokenMasked && sourcegraphEndpointMasked && !editingToken && !editingEndpoint}
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
									<span class="text-sm">Connected as {healthCheck.username}</span>
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
