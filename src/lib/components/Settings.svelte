<script lang="ts">
	import { onMount } from "svelte"
	import { settingsStore } from "$lib/stores/settingsStore"
	import * as ipc from "$lib/ipc"

	import AmpSettings from "./settings/AmpSettings.svelte"
	import GitHubSettings from "./settings/GitHubSettings.svelte"
	import GitLabSettings from "./settings/GitLabSettings.svelte"
	import SourcegraphSettings from "./settings/SourcegraphSettings.svelte"
	import EditorSettings from "./settings/EditorSettings.svelte"
	import CiSettings from "./settings/CiSettings.svelte"
	import ThemeSettings from "./settings/ThemeSettings.svelte"
	import StartupSettings from "./settings/StartupSettings.svelte"

	let activeSection = $state<"general" | "agents" | "integrations">("general")
	let loading = $state(true)
	let saveStatus = $state<{ type: "success" | "error"; message: string } | null>(null)
	let availableEditors = $state<ipc.AppInfo[]>([])
	let availableTerminals = $state<ipc.TerminalInfo[]>([])

	onMount(async () => {
		try {
			await settingsStore.load()
			availableEditors = await ipc.getAvailableEditors()
			availableTerminals = await ipc.getAvailableTerminals()
		} finally {
			loading = false
		}
	})

	function handleStatusChange(status: { type: "success" | "error"; message: string }) {
		saveStatus = status
		setTimeout(() => (saveStatus = null), status.type === "success" ? 3000 : 5000)
	}
</script>

<div class="h-full flex">
	<!-- Sidebar Navigation -->
	<nav class="w-56 border-r bg-card flex-shrink-0">
		<div class="p-4">
			<h2 class="text-lg font-semibold mb-4">Settings</h2>
			<div class="space-y-1">
				<button
					onclick={() => (activeSection = "general")}
					class="w-full text-left px-3 py-2 rounded-md text-sm transition-colors {activeSection ===
					'general'
						? 'bg-primary text-primary-foreground'
						: 'hover:bg-muted'}"
				>
					General
				</button>
				<button
					onclick={() => (activeSection = "agents")}
					class="w-full text-left px-3 py-2 rounded-md text-sm transition-colors {activeSection ===
					'agents'
						? 'bg-primary text-primary-foreground'
						: 'hover:bg-muted'}"
				>
					Agents
				</button>
				<button
					onclick={() => (activeSection = "integrations")}
					class="w-full text-left px-3 py-2 rounded-md text-sm transition-colors {activeSection ===
					'integrations'
						? 'bg-primary text-primary-foreground'
						: 'hover:bg-muted'}"
				>
					Integrations
				</button>
			</div>
		</div>
	</nav>

	<!-- Content Area -->
	<div class="flex-1 overflow-y-auto">
		<div class="p-4 sm:p-6 lg:p-8 max-w-4xl">
			{#if saveStatus}
				<div
					class="mb-6 p-4 rounded-lg {saveStatus.type === 'success'
						? 'bg-success/10 text-success'
						: 'bg-destructive/10 text-destructive'}"
				>
					{saveStatus.message}
				</div>
			{/if}

			{#if loading}
				<div class="text-muted-foreground">Loading...</div>
			{:else if activeSection === "general"}
				<div class="mb-6">
					<h1 class="text-2xl font-bold mb-2">General</h1>
					<p class="text-sm text-muted-foreground">
						Configure editor, terminal, CI monitoring, and appearance
					</p>
				</div>
				<div class="space-y-6">
					<EditorSettings
						{availableEditors}
						{availableTerminals}
						onStatusChange={handleStatusChange}
					/>
					<CiSettings onStatusChange={handleStatusChange} />
					<ThemeSettings />
					<StartupSettings onStatusChange={handleStatusChange} />
				</div>
			{:else if activeSection === "agents"}
				<div class="mb-6">
					<h1 class="text-2xl font-bold mb-2">Agents</h1>
					<p class="text-sm text-muted-foreground">
						Configure Amp AI agent for prompt execution and analysis
					</p>
				</div>
				<div class="space-y-6">
					<AmpSettings onStatusChange={handleStatusChange} />
				</div>
			{:else if activeSection === "integrations"}
				<div class="mb-6">
					<h1 class="text-2xl font-bold mb-2">Integrations</h1>
					<p class="text-sm text-muted-foreground">
						Configure Git providers, code search, and CI monitoring
					</p>
				</div>
				<div class="space-y-6">
					<GitHubSettings onStatusChange={handleStatusChange} />
					<GitLabSettings onStatusChange={handleStatusChange} />
					<SourcegraphSettings onStatusChange={handleStatusChange} />
				</div>
			{/if}
		</div>
	</div>
</div>
