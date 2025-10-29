<script lang="ts">
	import { tokenStore } from "$lib/tokenStore"
	import { onMount } from "svelte"

	interface Props {
		onStatusChange: (status: { type: "success" | "error"; message: string }) => void
	}

	let { onStatusChange }: Props = $props()

	let ampToken = $state("")
	let ampClientId = $state("")
	let ampClientSecret = $state("")
	let ampTokenMasked = $state("")
	let ampClientIdMasked = $state("")
	let ampClientSecretMasked = $state("")
	let editingToken = $state(false)
	let editingClientId = $state(false)
	let editingClientSecret = $state(false)
	let loading = $state(true)

	onMount(async () => {
		try {
			const allTokens = await tokenStore.getAllTokensMasked()
			ampTokenMasked = allTokens.ampToken || ""
			ampClientIdMasked = allTokens.ampClientId || ""
			ampClientSecretMasked = allTokens.ampClientSecret || ""
		} finally {
			loading = false
		}
	})

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

	async function saveClientId() {
		try {
			if (ampClientId.trim()) {
				await tokenStore.setToken("amp_client_id", ampClientId.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				ampClientIdMasked = allTokens.ampClientId || ""
				ampClientId = ""
				editingClientId = false
			} else {
				await tokenStore.deleteToken("amp_client_id")
				ampClientIdMasked = ""
				editingClientId = false
			}
			onStatusChange({ type: "success", message: "Amp OAuth2 Client ID saved securely" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to save: ${error}` })
		}
	}

	async function saveClientSecret() {
		try {
			if (ampClientSecret.trim()) {
				await tokenStore.setToken("amp_client_secret", ampClientSecret.trim())
				const allTokens = await tokenStore.getAllTokensMasked()
				ampClientSecretMasked = allTokens.ampClientSecret || ""
				ampClientSecret = ""
				editingClientSecret = false
			} else {
				await tokenStore.deleteToken("amp_client_secret")
				ampClientSecretMasked = ""
				editingClientSecret = false
			}
			onStatusChange({ type: "success", message: "Amp OAuth2 Client Secret saved securely" })
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

	async function deleteClientIdValue() {
		try {
			await tokenStore.deleteToken("amp_client_id")
			ampClientId = ""
			ampClientIdMasked = ""
			editingClientId = false
			onStatusChange({ type: "success", message: "Amp OAuth2 Client ID deleted" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}

	async function deleteClientSecretValue() {
		try {
			await tokenStore.deleteToken("amp_client_secret")
			ampClientSecret = ""
			ampClientSecretMasked = ""
			editingClientSecret = false
			onStatusChange({ type: "success", message: "Amp OAuth2 Client Secret deleted" })
		} catch (error) {
			onStatusChange({ type: "error", message: `Failed to delete: ${error}` })
		}
	}
</script>

<div class="p-6 border rounded-lg bg-card">
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
			</div>

			<div>
				<label for="amp-client-id" class="block text-sm font-medium mb-2"
					>Amp OAuth2 Client ID</label
				>
				<p class="text-xs text-muted-foreground mb-2">
					Required for accessing Amp V2 API (thread history, failure analysis)
				</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingClientId}
						<div class="flex-1">
							<input
								id="amp-client-id"
								type="text"
								bind:value={ampClientId}
								placeholder="Enter Amp OAuth2 Client ID"
								class="w-full px-3 py-2 border rounded-md bg-background"
							/>
						</div>
						<button
							type="button"
							onclick={saveClientId}
							class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
						>
							Save
						</button>
						<button
							type="button"
							onclick={() => {
								editingClientId = false
								ampClientId = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={ampClientIdMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingClientId = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{ampClientIdMasked ? "Update" : "Set"}
						</button>
						{#if ampClientIdMasked}
							<button
								type="button"
								onclick={deleteClientIdValue}
								class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
							>
								Delete
							</button>
						{/if}
					{/if}
				</div>
				<p class="text-xs text-muted-foreground mt-2">
					Provisioned by Sourcegraph for Enterprise customers
				</p>
			</div>

			<div>
				<label for="amp-client-secret" class="block text-sm font-medium mb-2"
					>Amp OAuth2 Client Secret</label
				>
				<p class="text-xs text-muted-foreground mb-2">
					Required for accessing Amp V2 API (thread history, failure analysis)
				</p>
				<div class="flex flex-col sm:flex-row gap-2">
					{#if editingClientSecret}
						<div class="flex-1">
							<input
								id="amp-client-secret"
								type="password"
								bind:value={ampClientSecret}
								placeholder="Enter Amp OAuth2 Client Secret"
								class="w-full px-3 py-2 border rounded-md bg-background"
							/>
						</div>
						<button
							type="button"
							onclick={saveClientSecret}
							class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
						>
							Save
						</button>
						<button
							type="button"
							onclick={() => {
								editingClientSecret = false
								ampClientSecret = ""
							}}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							Cancel
						</button>
					{:else}
						<div class="flex-1">
							<input
								type="text"
								value={ampClientSecretMasked || "Not set"}
								disabled
								class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
							/>
						</div>
						<button
							type="button"
							onclick={() => (editingClientSecret = true)}
							class="px-3 py-2 border rounded-md hover:bg-muted"
						>
							{ampClientSecretMasked ? "Update" : "Set"}
						</button>
						{#if ampClientSecretMasked}
							<button
								type="button"
								onclick={deleteClientSecretValue}
								class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
							>
								Delete
							</button>
						{/if}
					{/if}
				</div>
				<p class="text-xs text-muted-foreground mt-2">
					Provisioned by Sourcegraph for Enterprise customers
				</p>
			</div>
		</div>
	{/if}
</div>
