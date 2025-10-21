<script lang="ts">
	import { onMount } from 'svelte';
	import { tokenStore, type TokenKey } from '$lib/tokenStore';
	import { themeStore, type Theme } from '$lib/stores/themeStore.svelte';
	import { settingsStore } from '$lib/stores/settingsStore';
	import * as ipc from '$lib/ipc';
	import { Select } from 'bits-ui';
	import { Check, ChevronDown } from 'lucide-svelte';

	let ampToken = $state('');
	let githubToken = $state('');
	let sourcegraphEndpoint = $state('');
	let sourcegraphToken = $state('');
	let ampClientId = $state('');
	let ampClientSecret = $state('');
	let ampTokenMasked = $state('');
	let githubTokenMasked = $state('');
	let sourcegraphEndpointMasked = $state('');
	let sourcegraphTokenMasked = $state('');
	let ampClientIdMasked = $state('');
	let ampClientSecretMasked = $state('');
	let editingAmp = $state(false);
	let editingGithub = $state(false);
	let editingSourcegraphEndpoint = $state(false);
	let editingSourcegraphToken = $state(false);
	let editingAmpClientId = $state(false);
	let editingAmpClientSecret = $state(false);
	let loading = $state(true);
	let saveStatus = $state<{ type: 'success' | 'error'; message: string } | null>(null);
	let ciThreshold = $state(10);
	let editingCiThreshold = $state(false);
	let ciThresholdInput = $state('10');
	let maxConcurrentExecutions = $state(10);
	let editingMaxConcurrent = $state(false);
	let maxConcurrentInput = $state('10');
	let editorCommand = $state('code');
	let editingEditorCommand = $state(false);
	let editorCommandInput = $state('code');
	
	let availableEditors = $state<ipc.AppInfo[]>([]);
	let availableTerminals = $state<ipc.TerminalInfo[]>([]);
	let selectedEditorValue = $state<string>('');
	let selectedTerminalValue = $state<string>('');

	// Derive currentTheme from themeStore
	let currentTheme = $derived(themeStore.current);
	
	// Find if selected editor needs terminal
	let editorNeedsTerminal = $derived(
		availableEditors.find(e => e.command === selectedEditorValue)?.needsTerminal ?? false
	);

	onMount(async () => {
		try {
			// Load all tokens in a single keychain access (prevents multiple prompts)
			const allTokens = await tokenStore.getAllTokensMasked();
			ampTokenMasked = allTokens.ampToken || '';
			githubTokenMasked = allTokens.githubToken || '';
			sourcegraphEndpointMasked = allTokens.sourcegraphEndpoint || '';
			sourcegraphTokenMasked = allTokens.sourcegraphToken || '';
			ampClientIdMasked = allTokens.ampClientId || '';
			ampClientSecretMasked = allTokens.ampClientSecret || '';
			
			await settingsStore.load();
			
			// Load available editors and terminals
			availableEditors = await ipc.getAvailableEditors();
			availableTerminals = await ipc.getAvailableTerminals();
		} finally {
			loading = false;
		}
	});

	// Reactively sync settings store to local state using $effect
	$effect(() => {
		const settings = $settingsStore;
		ciThreshold = settings.ciStuckThresholdMinutes;
		ciThresholdInput = settings.ciStuckThresholdMinutes.toString();
		maxConcurrentExecutions = settings.maxConcurrentExecutions;
		maxConcurrentInput = settings.maxConcurrentExecutions.toString();
		editorCommand = settings.editorCommand;
		editorCommandInput = settings.editorCommand;
		selectedEditorValue = settings.selectedEditor || '';
		selectedTerminalValue = settings.selectedTerminal || '';
	});

	async function setTheme(theme: Theme) {
		await themeStore.setTheme(theme);
	}

	async function saveToken(key: TokenKey, value: string) {
		try {
			if (value.trim()) {
				await tokenStore.setToken(key, value.trim());
				// Refresh all masked tokens after update
				const allTokens = await tokenStore.getAllTokensMasked();
				ampTokenMasked = allTokens.ampToken || '';
				githubTokenMasked = allTokens.githubToken || '';
				sourcegraphEndpointMasked = allTokens.sourcegraphEndpoint || '';
				sourcegraphTokenMasked = allTokens.sourcegraphToken || '';
				ampClientIdMasked = allTokens.ampClientId || '';
				ampClientSecretMasked = allTokens.ampClientSecret || '';
				
				// Clear input and exit editing mode
				if (key === 'amp_token') {
					ampToken = '';
					editingAmp = false;
				} else if (key === 'github_token') {
					githubToken = '';
					editingGithub = false;
				} else if (key === 'sourcegraph_endpoint') {
					sourcegraphEndpoint = '';
					editingSourcegraphEndpoint = false;
				} else if (key === 'sourcegraph_token') {
					sourcegraphToken = '';
					editingSourcegraphToken = false;
				} else if (key === 'amp_client_id') {
					ampClientId = '';
					editingAmpClientId = false;
				} else if (key === 'amp_client_secret') {
					ampClientSecret = '';
					editingAmpClientSecret = false;
				}
			} else {
				await tokenStore.deleteToken(key);
				if (key === 'amp_token') {
					ampTokenMasked = '';
					editingAmp = false;
				} else if (key === 'github_token') {
					githubTokenMasked = '';
					editingGithub = false;
				} else if (key === 'sourcegraph_endpoint') {
					sourcegraphEndpointMasked = '';
					editingSourcegraphEndpoint = false;
				} else if (key === 'sourcegraph_token') {
					sourcegraphTokenMasked = '';
					editingSourcegraphToken = false;
				} else if (key === 'amp_client_id') {
					ampClientIdMasked = '';
					editingAmpClientId = false;
				} else if (key === 'amp_client_secret') {
					ampClientSecretMasked = '';
					editingAmpClientSecret = false;
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
			} else if (key === 'github_token') {
				githubToken = '';
				githubTokenMasked = '';
				editingGithub = false;
			} else if (key === 'sourcegraph_endpoint') {
				sourcegraphEndpoint = '';
				sourcegraphEndpointMasked = '';
				editingSourcegraphEndpoint = false;
			} else if (key === 'sourcegraph_token') {
				sourcegraphToken = '';
				sourcegraphTokenMasked = '';
				editingSourcegraphToken = false;
			} else if (key === 'amp_client_id') {
				ampClientId = '';
				ampClientIdMasked = '';
				editingAmpClientId = false;
			} else if (key === 'amp_client_secret') {
				ampClientSecret = '';
				ampClientSecretMasked = '';
				editingAmpClientSecret = false;
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
		} else if (key === 'github_token') {
			editingGithub = true;
			githubToken = '';
		} else if (key === 'sourcegraph_endpoint') {
			editingSourcegraphEndpoint = true;
			sourcegraphEndpoint = '';
		} else if (key === 'sourcegraph_token') {
			editingSourcegraphToken = true;
			sourcegraphToken = '';
		} else if (key === 'amp_client_id') {
			editingAmpClientId = true;
			ampClientId = '';
		} else if (key === 'amp_client_secret') {
			editingAmpClientSecret = true;
			ampClientSecret = '';
		}
	}

	function cancelEditing(key: TokenKey) {
		if (key === 'amp_token') {
			editingAmp = false;
			ampToken = '';
		} else if (key === 'github_token') {
			editingGithub = false;
			githubToken = '';
		} else if (key === 'sourcegraph_endpoint') {
			editingSourcegraphEndpoint = false;
			sourcegraphEndpoint = '';
		} else if (key === 'sourcegraph_token') {
			editingSourcegraphToken = false;
			sourcegraphToken = '';
		} else if (key === 'amp_client_id') {
			editingAmpClientId = false;
			ampClientId = '';
		} else if (key === 'amp_client_secret') {
			editingAmpClientSecret = false;
			ampClientSecret = '';
		}
	}
	
	async function saveCiThreshold() {
		try {
			const value = parseInt(ciThresholdInput);
			if (isNaN(value) || value < 1) {
				saveStatus = { type: 'error', message: 'CI timeout must be at least 1 minute' };
				setTimeout(() => saveStatus = null, 3000);
				return;
			}
			await settingsStore.setCiStuckThreshold(value);
			editingCiThreshold = false;
			saveStatus = { type: 'success', message: 'CI timeout setting saved' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to save: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
		}
	}
	
	function cancelCiThresholdEdit() {
		ciThresholdInput = ciThreshold.toString();
		editingCiThreshold = false;
	}
	
	async function saveMaxConcurrent() {
		try {
			const value = parseInt(maxConcurrentInput);
			if (isNaN(value) || value < 1) {
				saveStatus = { type: 'error', message: 'Max concurrent executions must be at least 1' };
				setTimeout(() => saveStatus = null, 3000);
				return;
			}
			await settingsStore.setMaxConcurrentExecutions(value);
			editingMaxConcurrent = false;
			saveStatus = { type: 'success', message: 'Concurrency limit saved' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to save: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
		}
	}
	
	function cancelMaxConcurrentEdit() {
		maxConcurrentInput = maxConcurrentExecutions.toString();
		editingMaxConcurrent = false;
	}
	
	async function saveEditorCommand() {
		try {
			const value = editorCommandInput.trim();
			if (!value) {
				saveStatus = { type: 'error', message: 'Editor command cannot be empty' };
				setTimeout(() => saveStatus = null, 3000);
				return;
			}
			await settingsStore.setEditorCommand(value);
			editingEditorCommand = false;
			saveStatus = { type: 'success', message: 'Editor command saved' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to save: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
		}
	}
	
	function cancelEditorCommandEdit() {
		editorCommandInput = editorCommand;
		editingEditorCommand = false;
	}
	
	async function handleEditorChange(value: string | undefined) {
		if (!value) return;
		try {
			await settingsStore.setSelectedEditor(value);
			saveStatus = { type: 'success', message: 'Editor preference saved' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to save: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
		}
	}
	
	async function handleTerminalChange(value: string | undefined) {
		if (!value) return;
		try {
			await settingsStore.setSelectedTerminal(value);
			saveStatus = { type: 'success', message: 'Terminal preference saved' };
			setTimeout(() => saveStatus = null, 3000);
		} catch (error) {
			saveStatus = { type: 'error', message: `Failed to save: ${error}` };
			setTimeout(() => saveStatus = null, 5000);
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
			<div class="mb-6 p-4 rounded-lg {saveStatus.type === 'success' ? 'bg-success/10 text-success' : 'bg-destructive/10 text-destructive'}">
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
									class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
									>
									Save
									</button>
									<button
									type="button"
									onclick={() => cancelEditing('amp_token')}
									class="px-3 py-2 border rounded-md hover:bg-muted"
									>
									Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
										type="text"
										value={ampTokenMasked || 'Not set'}
										disabled
										class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('amp_token')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										{ampTokenMasked ? 'Update' : 'Set'}
									</button>
									{#if ampTokenMasked}
										<button
											type="button"
											onclick={() => deleteToken('amp_token')}
											class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
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
										class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('github_token')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={githubTokenMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('github_token')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										{githubTokenMasked ? 'Update' : 'Set'}
									</button>
									{#if githubTokenMasked}
										<button
											type="button"
											onclick={() => deleteToken('github_token')}
											class="px-3 py-2 text-destructive hover:bg-destructive/10 rounded-md"
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
					<h3 class="text-lg font-semibold mb-4">Sourcegraph</h3>
					<p class="text-sm text-muted-foreground mb-6">
						Configure Sourcegraph instance for repository search
					</p>

					<div class="space-y-6">
						<div>
							<label for="sourcegraph-endpoint" class="block text-sm font-medium mb-2">Sourcegraph Instance</label>
							<p class="text-xs text-muted-foreground mb-2">URL of your Sourcegraph instance</p>
							<div class="flex flex-col sm:flex-row gap-2">
								{#if editingSourcegraphEndpoint}
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
										onclick={() => saveToken('sourcegraph_endpoint', sourcegraphEndpoint)}
										class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('sourcegraph_endpoint')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={sourcegraphEndpointMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('sourcegraph_endpoint')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										{sourcegraphEndpointMasked ? 'Update' : 'Set'}
									</button>
									{#if sourcegraphEndpointMasked}
										<button
											type="button"
											onclick={() => deleteToken('sourcegraph_endpoint')}
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
							<label for="sourcegraph-token" class="block text-sm font-medium mb-2">Sourcegraph Access Token</label>
							<p class="text-xs text-muted-foreground mb-2">Required for repository search API access</p>
							<div class="flex flex-col sm:flex-row gap-2">
								{#if editingSourcegraphToken}
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
										onclick={() => saveToken('sourcegraph_token', sourcegraphToken)}
										class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('sourcegraph_token')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={sourcegraphTokenMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('sourcegraph_token')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										{sourcegraphTokenMasked ? 'Update' : 'Set'}
									</button>
									{#if sourcegraphTokenMasked}
										<button
											type="button"
											onclick={() => deleteToken('sourcegraph_token')}
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
						</div>

						<!-- Amp OAuth2 Client ID -->
						<div>
							<label for="amp-client-id" class="block text-sm font-medium mb-2">Amp OAuth2 Client ID</label>
							<p class="text-xs text-muted-foreground mb-2">
								Required for accessing Amp V2 API (thread history, failure analysis)
							</p>
							<div class="flex flex-col sm:flex-row gap-2">
								{#if editingAmpClientId}
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
										onclick={() => saveToken('amp_client_id', ampClientId)}
										class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('amp_client_id')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={ampClientIdMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('amp_client_id')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										{ampClientIdMasked ? 'Update' : 'Set'}
									</button>
									{#if ampClientIdMasked}
										<button
											type="button"
											onclick={() => deleteToken('amp_client_id')}
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

						<!-- Amp OAuth2 Client Secret -->
						<div>
							<label for="amp-client-secret" class="block text-sm font-medium mb-2">Amp OAuth2 Client Secret</label>
							<p class="text-xs text-muted-foreground mb-2">
								Required for accessing Amp V2 API (thread history, failure analysis)
							</p>
							<div class="flex flex-col sm:flex-row gap-2">
								{#if editingAmpClientSecret}
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
										onclick={() => saveToken('amp_client_secret', ampClientSecret)}
										class="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
									>
										Save
									</button>
									<button
										type="button"
										onclick={() => cancelEditing('amp_client_secret')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										Cancel
									</button>
								{:else}
									<div class="flex-1">
										<input
											type="text"
											value={ampClientSecretMasked || 'Not set'}
											disabled
											class="w-full px-3 py-2 border rounded-md bg-muted text-muted-foreground"
										/>
									</div>
									<button
										type="button"
										onclick={() => startEditing('amp_client_secret')}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										{ampClientSecretMasked ? 'Update' : 'Set'}
									</button>
									{#if ampClientSecretMasked}
										<button
											type="button"
											onclick={() => deleteToken('amp_client_secret')}
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
				</div>

				<div class="p-6 border rounded-lg bg-card">
					<h3 class="text-lg font-semibold mb-4">Development Tools</h3>
					<p class="text-sm text-muted-foreground mb-6">
						Configure editor and development tools
					</p>

					<div class="space-y-6">
						<!-- Editor Selection -->
						<div>
							<label for="editor-select" class="block text-sm font-medium mb-2">Preferred Editor</label>
							<p class="text-xs text-muted-foreground mb-2">
								Choose your preferred editor for opening worktrees
							</p>
							{#if availableEditors.length > 0}
								<Select.Root
									type="single"
									value={selectedEditorValue}
									onValueChange={handleEditorChange}
									items={availableEditors.map(e => ({ value: e.command, label: e.displayName }))}
								>
									<Select.Trigger
										id="editor-select"
										class="w-full flex items-center justify-between px-3 py-2 border rounded-md bg-background hover:bg-muted/30 transition-colors"
									>
										<span>{availableEditors.find(e => e.command === selectedEditorValue)?.displayName || 'Select an editor'}</span>
										<ChevronDown class="w-4 h-4 text-muted-foreground" />
									</Select.Trigger>
									<Select.Content
										class="w-full bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
										sideOffset={4}
									>
										{#each availableEditors as editor (editor.command)}
											<Select.Item
												value={editor.command}
												label={editor.displayName}
												class="flex items-center justify-between px-3 py-2 text-sm rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
											>
												<span>{editor.displayName}</span>
												{#if selectedEditorValue === editor.command}
													<Check class="w-4 h-4 text-primary" />
												{/if}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{:else}
								<p class="text-sm text-muted-foreground">No supported editors found in PATH</p>
							{/if}
						</div>
						
						<!-- Terminal Selection -->
						<div>
							<label for="terminal-select" class="block text-sm font-medium mb-2">Terminal Application</label>
							<p class="text-xs text-muted-foreground mb-2">
								Choose terminal for vim/nvim (macOS only: Terminal or Ghostty)
							</p>
							{#if availableTerminals.length > 0}
								<Select.Root
									type="single"
									value={selectedTerminalValue}
									onValueChange={handleTerminalChange}
									items={availableTerminals.map(t => ({ value: t.command, label: t.displayName }))}
								>
									<Select.Trigger
										id="terminal-select"
										class="w-full flex items-center justify-between px-3 py-2 border rounded-md bg-background hover:bg-muted/30 transition-colors"
									>
										<span>{availableTerminals.find(t => t.command === selectedTerminalValue)?.displayName || 'Select a terminal'}</span>
										<ChevronDown class="w-4 h-4 text-muted-foreground" />
									</Select.Trigger>
									<Select.Content
										class="w-full bg-card border border-border/20 rounded-lg shadow-xl p-1 z-50"
										sideOffset={4}
									>
										{#each availableTerminals as terminal (terminal.command)}
											<Select.Item
												value={terminal.command}
												label={terminal.displayName}
												class="flex items-center justify-between px-3 py-2 text-sm rounded hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors"
											>
												<span>{terminal.displayName}</span>
												{#if selectedTerminalValue === terminal.command}
													<Check class="w-4 h-4 text-primary" />
												{/if}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{:else}
								<p class="text-sm text-muted-foreground">No terminal applications found</p>
							{/if}
						</div>
					</div>
				</div>

				<div class="p-6 border rounded-lg bg-card">
					<h3 class="text-lg font-semibold mb-4">CI Monitoring</h3>
					<p class="text-sm text-muted-foreground mb-6">
						Configure CI status checking behavior
					</p>

					<div class="space-y-6">
						<div>
							<label for="ci-threshold-input" class="block text-sm font-medium mb-2">CI Stuck Timeout</label>
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
										onclick={() => editingCiThreshold = true}
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
							<label for="max-concurrent-input" class="block text-sm font-medium mb-2">Max Concurrent Executions</label>
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
										onclick={() => editingMaxConcurrent = true}
										class="px-3 py-2 border rounded-md hover:bg-muted"
									>
										Update
									</button>
								{/if}
							</div>
							<p class="text-xs text-muted-foreground mt-2">
								Default: 10 executions. Increase for faster bulk operations, decrease to limit resource usage.
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
								class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'light' ? 'bg-primary text-primary-foreground border-primary' : 'bg-card hover:bg-muted border-border/30'}"
							>
								Light
							</button>
							<button
								type="button"
								onclick={() => setTheme('dark')}
								class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'dark' ? 'bg-primary text-primary-foreground border-primary' : 'bg-card hover:bg-muted border-border/30'}"
							>
								Dark
							</button>
							<button
								type="button"
								onclick={() => setTheme('auto')}
								class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'auto' ? 'bg-primary text-primary-foreground border-primary' : 'bg-card hover:bg-muted border-border/30'}"
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
