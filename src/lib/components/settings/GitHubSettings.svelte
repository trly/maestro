<script lang="ts">
	import { tokenStore } from "$lib/tokenStore"
	import * as ipc from "$lib/ipc"
	import { onMount } from "svelte"
	import { CheckCircle2, XCircle, Loader2 } from "lucide-svelte"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let githubToken = $state("")
	let githubTokenMasked = $state("")
	let editing = $state(false)
	let loading = $state(true)
	let healthCheck = $state<ipc.HealthCheckResult | null>(null)
	let checking = $state(false)

	onMount(async () => {
		try {
			const allTokens = await tokenStore.getAllTokensMasked()
			githubTokenMasked = allTokens.githubToken || ""
		} finally {
			loading = false
		}
	})

	async function testConnection() {
		checking = true
		healthCheck = null
		try {
			healthCheck = await ipc.healthCheckGithub()
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
			if (githubToken.trim()) {
				await tokenStore.setToken("github_token", githubToken.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				githubTokenMasked = allTokens.githubToken || ""
				githubToken = ""
				editing = false
			} else {
				await tokenStore.deleteToken("github_token")
				githubTokenMasked = ""
				editing = false
			}
			onStatusChange({ type: "success", message: "GitHub token saved securely to system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function deleteToken() {
		try {
			await tokenStore.deleteToken("github_token")
			githubToken = ""
			githubTokenMasked = ""
			editing = false
			onStatusChange({ type: "success", message: "GitHub token deleted from system keyring" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}
</script>

<div class="p-6 border rounded-lg bg-card">
	<h3 class="text-lg font-semibold mb-4">GitHub</h3>
	<p class="text-sm text-muted-foreground mb-6">
		Configure GitHub integration for CI status checks
	</p>

	{#if loading}
		<p class="text-sm text-muted-foreground">Loading...</p>
	{:else}
		<div>
			<label for="github-token" class="block text-sm font-medium mb-2">Personal Access Token</label>
			<p class="text-xs text-muted-foreground mb-2">Required for CI status monitoring</p>
			<div class="flex flex-col sm:flex-row gap-2">
				{#if editing}
					<div class="flex-1">
						<input
							id="github-token"
							type="text"
							bind:value={githubToken}
							placeholder="Enter GitHub PAT"
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
							editing = false
							githubToken = ""
						}}
						class="px-3 py-2 border rounded-md hover:bg-muted"
					>
						Cancel
					</button>
				{:else}
					<div class="flex-1">
						<input
							type="text"
							value={githubTokenMasked || "Not set"}
							disabled
							class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
						/>
					</div>
					<button
						type="button"
						onclick={() => (editing = true)}
						class="px-3 py-2 border rounded-md hover:bg-muted"
					>
						{githubTokenMasked ? "Update" : "Set"}
					</button>
					{#if githubTokenMasked}
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
				Generate at: <a
					href="https://github.com/settings/tokens"
					target="_blank"
					class="text-primary hover:underline">github.com/settings/tokens</a
				>
			</p>
			{#if githubTokenMasked && !editing}
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
	{/if}
</div>
