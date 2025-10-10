<script lang="ts">
	import { onMount } from 'svelte';
	import { tokenStore, type TokenKey } from '$lib/tokenStore';
	import { themeStore, type Theme } from '$lib/stores/themeStore';

	let ampToken = $state('');
	let githubToken = $state('');
	let ampTokenMasked = $state('');
	let githubTokenMasked = $state('');
	let editingAmp = $state(false);
	let editingGithub = $state(false);
	let loading = $state(true);
	let saveStatus = $state<{ type: 'success' | 'error'; message: string } | null>(null);
	let currentTheme = $state<Theme>('auto');

	onMount(async () => {
		try {
			const ampMasked = await tokenStore.getTokenMasked('amp_token');
			const githubMasked = await tokenStore.getTokenMasked('github_token');
			ampTokenMasked = ampMasked || '';
			githubTokenMasked = githubMasked || '';
		} finally {
			loading = false;
		}

		themeStore.subscribe(theme => {
			currentTheme = theme;
		});
	});

	async function setTheme(theme: Theme) {
		await themeStore.setTheme(theme);
	}

	async function saveToken(key: TokenKey, value: string) {
		try {
			if (value.trim()) {
				await tokenStore.setToken(key, value.trim());
				const masked = await tokenStore.getTokenMasked(key);
				if (key === 'amp_token') {
					ampTokenMasked = masked || '';
					ampToken = '';
					editingAmp = false;
				} else {
					githubTokenMasked = masked || '';
					githubToken = '';
					editingGithub = false;
				}
			} else {
				await tokenStore.deleteToken(key);
				if (key === 'amp_token') {
					ampTokenMasked = '';
					editingAmp = false;
				} else {
					githubTokenMasked = '';
					editingGithub = false;
				}
			}
			saveStatus = { type: 'success', message: 'Token saved securely to system keyring' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to save: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
		}
	}

	async function deleteToken(key: TokenKey) {
		try {
			await tokenStore.deleteToken(key);
			if (key === 'amp_token') {
				ampToken = '';
				ampTokenMasked = '';
				editingAmp = false;
			} else {
				githubToken = '';
				githubTokenMasked = '';
				editingGithub = false;
			}
			saveStatus = { type: 'success', message: 'Token deleted from system keyring' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to delete: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
		}
	}

	function startEditing(key: TokenKey) {
		if (key === 'amp_token') {
			editingAmp = true;
			ampToken = '';
		} else {
			editingGithub = true;
			githubToken = '';
		}
	}

	function cancelEditing(key: TokenKey) {
		if (key === 'amp_token') {
			editingAmp = false;
			ampToken = '';
		} else {
			editingGithub = false;
			githubToken = '';
		}
	}
</script>

<div class="h-full overflow-y-auto p-4 sm:p-6 lg:p-8">
	<div class="max-w-2xl mx-auto">
		<div class="mb-8">
			<h2 class="text-3xl font-bold text-foreground mb-2">Settings</h2>
			<p class="text-muted-foreground">Manage your API tokens and configuration</p>
		</div>

		{#if saveStatus}
			<div class="mb-6 p-4 rounded-lg {saveStatus.type === 'success' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}">
				{saveStatus.message}
			</div>
		{/if}

		{#if loading}
			<div class="text-muted-foreground">Loading...</div>
		{:else}
			<div class="space-y-6">
				<div class="p-6 border rounded-lg bg-card">
					<h3 class="text-lg font-semibold mb-4">API Tokens</h3>
					<p class="text-sm text-muted-foreground mb-6">
						Tokens are stored securely in your system keyring
					</p>

					<div class="space-y-6">
						<div>
							<label for="amp-token" class="block text-sm font-medium mb-2">Amp API Token</label>
							<p class="text-xs text-muted-foreground mb-2">Required for executing prompts with Amp</p>
							<div class="flex flex-col sm:flex-row gap-2">
								{#if editingAmp}
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
										onclick={() => saveToken('amp_token', ampToken)}
										class="px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('amp_token')}
										class="px-3 py-2 border rounded-md hover:bg-gray-50"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={ampTokenMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-gray-50 text-gray-600"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('amp_token')}
										class="px-3 py-2 border rounded-md hover:bg-gray-50"
									>
										{ampTokenMasked ? 'Update' : 'Set'}
									</button>
									{#if ampTokenMasked}
										<button
											type="button"
											onclick={() => deleteToken('amp_token')}
											class="px-3 py-2 text-red-600 hover:bg-red-50 rounded-md"
										>
											Delete
										</button>
									{/if}
								{/if}
							</div>
							<p class="text-xs text-muted-foreground mt-2">
								Get your token at <a href="https://ampcode.com/settings" target="_blank" class="underline">ampcode.com/settings</a>
							</p>
						</div>

						<div>
							<label for="github-token" class="block text-sm font-medium mb-2">GitHub Personal Access Token</label>
							<p class="text-xs text-muted-foreground mb-2">Required for repository access</p>
							<div class="flex flex-col sm:flex-row gap-2">
								{#if editingGithub}
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
										onclick={() => saveToken('github_token', githubToken)}
										class="px-3 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('github_token')}
										class="px-3 py-2 border rounded-md hover:bg-gray-50"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={githubTokenMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-gray-50 text-gray-600"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('github_token')}
										class="px-3 py-2 border rounded-md hover:bg-gray-50"
									>
										{githubTokenMasked ? 'Update' : 'Set'}
									</button>
									{#if githubTokenMasked}
										<button
											type="button"
											onclick={() => deleteToken('github_token')}
											class="px-3 py-2 text-red-600 hover:bg-red-50 rounded-md"
										>
											Delete
										</button>
									{/if}
								{/if}
							</div>
							<p class="text-xs text-muted-foreground mt-2">
								Generate at <a href="https://github.com/settings/tokens" target="_blank" class="underline">github.com/settings/tokens</a> (requires <code>repo</code> scope)
							</p>
						</div>
					</div>
				</div>

				<div class="p-6 border rounded-lg bg-card">
					<h3 class="text-lg font-semibold mb-4">Appearance</h3>
					<p class="text-sm text-muted-foreground mb-6">
						Choose your preferred color theme
					</p>

					<div>
						<div class="block text-sm font-medium mb-3">Theme</div>
						<div class="flex gap-3">
							<button
								type="button"
								onclick={() => setTheme('light')}
								class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'light' ? 'bg-primary text-primary-foreground border-primary' : 'bg-card hover:bg-muted'}"
							>
								Light
							</button>
							<button
								type="button"
								onclick={() => setTheme('dark')}
								class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'dark' ? 'bg-primary text-primary-foreground border-primary' : 'bg-card hover:bg-muted'}"
							>
								Dark
							</button>
							<button
								type="button"
								onclick={() => setTheme('auto')}
								class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'auto' ? 'bg-primary text-primary-foreground border-primary' : 'bg-card hover:bg-muted'}"
							>
								Auto
							</button>
						</div>
						<p class="text-xs text-muted-foreground mt-2">
							{currentTheme === 'auto' ? 'Automatically matches your system theme' : `Using ${currentTheme} theme`}
						</p>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>
