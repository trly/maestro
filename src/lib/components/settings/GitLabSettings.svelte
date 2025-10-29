<script lang="ts">
	import { tokenStore } from "$lib/tokenStore"
	import * as ipc from "$lib/ipc"
	import { onMount } from "svelte"
	import { CheckCircle2, XCircle, Loader2 } from "lucide-svelte"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let gitlabToken = $state("")
	let gitlabEndpoint = $state("")
	let gitlabTokenMasked = $state("")
	let gitlabEndpointMasked = $state("")
	let editingToken = $state(false)
	let editingEndpoint = $state(false)
	let loading = $state(true)
	let healthCheck = $state<ipc.HealthCheckResult | null>(null)
	let checking = $state(false)

	onMount(async () => {
		try {
			const allTokens = await tokenStore.getAllTokensMasked()
			gitlabTokenMasked = allTokens.gitlabToken || ""
			gitlabEndpointMasked = allTokens.gitlabInstanceUrl || ""
		} finally {
			loading = false
		}
	})

	async function testConnection() {
		checking = true
		healthCheck = null
		try {
			healthCheck = await ipc.healthCheckGitlab()
		} catch (error) {
			healthCheck = {
				success: false,
				error: error instanceof Error ? error.message : String(error),
			}
		} finally {
			checking = false
		}
	}

	async function saveToken() {
		try {
			if (gitlabToken.trim()) {
				await tokenStore.setToken("gitlab_token", gitlabToken.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				gitlabTokenMasked = allTokens.gitlabToken || ""
				gitlabToken = ""
				editingToken = false
			} else {
				await tokenStore.deleteToken("gitlab_token")
				gitlabTokenMasked = ""
				editingToken = false
			}
			onStatusChange({ type: "success", message: "GitLab token saved securely to system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function saveEndpoint() {
		try {
			if (gitlabEndpoint.trim()) {
				await tokenStore.setToken("gitlab_instance_url", gitlabEndpoint.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				gitlabEndpointMasked = allTokens.gitlabInstanceUrl || ""
				gitlabEndpoint = ""
				editingEndpoint = false
			} else {
				await tokenStore.deleteToken("gitlab_instance_url")
				gitlabEndpointMasked = ""
				editingEndpoint = false
			}
			onStatusChange({ type: "success", message: "GitLab endpoint saved securely" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function deleteToken() {
		try {
			await tokenStore.deleteToken("gitlab_token")
			gitlabToken = ""
			gitlabTokenMasked = ""
			editingToken = false
			onStatusChange({ type: "success", message: "GitLab token deleted from system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}

	async function deleteEndpoint() {
		try {
			await tokenStore.deleteToken("gitlab_instance_url")
			gitlabEndpoint = ""
			gitlabEndpointMasked = ""
			editingEndpoint = false
			onStatusChange({ type: "success", message: "GitLab endpoint deleted" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}
</script>

<div class="p-6 border rounded-lg bg-card">
	<h3 class="text-lg font-semibold mb-4">GitLab</h3>
	<p class="text-sm text-muted-foreground mb-6">
		Configure GitLab integration for self-hosted or GitLab.com
	</p>

	{#if loading}
		<p class="text-sm text-muted-foreground">Loading...</p>
	{:else}
		<div class="space-y-6">
			<div>
				<label for="gitlab-endpoint" class="block text-sm font-medium mb-2">Instance URL</label>
				<p class="text-xs text-muted-foreground mb-2">Your GitLab instance endpoint</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingEndpoint}
						<div class="flex-1">
							<input
								id="gitlab-endpoint"
								type="text"
								bind:value={gitlabEndpoint}
								placeholder="https://gitlab.com"
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
								gitlabEndpoint = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={gitlabEndpointMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingEndpoint = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{gitlabEndpointMasked ? "Update" : "Set"}
						</button>
						{#if gitlabEndpointMasked}
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
					Example: <code>https://gitlab.com</code> or your self-hosted instance
				</p>
			</div>

			<div>
				<label for="gitlab-token" class="block text-sm font-medium mb-2">Access Token</label>
				<p class="text-xs text-muted-foreground mb-2">Required for CI status monitoring</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingToken}
						<div class="flex-1">
							<input
								id="gitlab-token"
								type="text"
								bind:value={gitlabToken}
								placeholder="Enter GitLab access token"
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
								gitlabToken = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={gitlabTokenMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingToken = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{gitlabTokenMasked ? "Update" : "Set"}
						</button>
						{#if gitlabTokenMasked}
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
					Generate at your GitLab instance: User Settings → Access Tokens
				</p>
				{#if gitlabTokenMasked && gitlabEndpointMasked && !editingToken && !editingEndpoint}
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
