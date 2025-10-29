<script lang="ts">
	import { onMount } from "svelte"
	import { open } from "@tauri-apps/plugin-shell"
	import { ExternalLink, Github } from "lucide-svelte"
	import { getAppInfo, type AppMetadata } from "$lib/ipc"

	let appInfo = $state<AppMetadata | null>(null)

	const REPO_URL = "https://github.com/trly/maestro"
	const LICENSE_URL = `${REPO_URL}/blob/main/LICENSE`
	const ISSUES_URL = `${REPO_URL}/issues`

	onMount(async () => {
		appInfo = await getAppInfo()
	})

	async function openExternal(url: string) {
		await open(url)
	}
</script>

<div class="p-6 max-w-2xl mx-auto space-y-6">
	<div class="space-y-2">
		<h1 class="text-2xl font-bold">About Maestro</h1>
		{#if appInfo}
			<p class="text-muted-foreground">Version {appInfo.version}</p>
		{/if}
	</div>

	<div class="space-y-4 border border-border rounded-lg p-6 bg-card">
		<div>
			<h2 class="text-lg font-semibold mb-2">Description</h2>
			<p class="text-foreground/90">
				Maestro orchestrates AI prompt execution across multiple repositories using the Amp SDK.
				Execute prompts at scale, validate changes automatically, and manage CI workflows from a
				single interface.
			</p>
		</div>

		<div class="grid gap-3">
			<button
				type="button"
				onclick={() => openExternal(REPO_URL)}
				class="flex items-center gap-2 text-primary hover:text-primary/90 transition-colors"
			>
				<Github class="w-4 h-4" />
				<span>View on GitHub</span>
				<ExternalLink class="w-3 h-3" />
			</button>

			<button
				type="button"
				onclick={() => openExternal(LICENSE_URL)}
				class="flex items-center gap-2 text-primary hover:text-primary/90 transition-colors"
			>
				<ExternalLink class="w-4 h-4" />
				<span>MIT License</span>
			</button>

			<button
				type="button"
				onclick={() => openExternal(ISSUES_URL)}
				class="flex items-center gap-2 text-primary hover:text-primary/90 transition-colors"
			>
				<ExternalLink class="w-4 h-4" />
				<span>Report an Issue</span>
			</button>
		</div>
	</div>

	{#if appInfo}
		<div class="border border-border rounded-lg p-6 bg-card space-y-2">
			<h2 class="text-lg font-semibold mb-3">Application Details</h2>
			<div class="grid gap-2 text-sm font-mono">
				<div class="flex justify-between">
					<span class="text-muted-foreground">Name:</span>
					<span>{appInfo.name}</span>
				</div>
				<div class="flex justify-between">
					<span class="text-muted-foreground">Version:</span>
					<span>{appInfo.version}</span>
				</div>
				<div class="flex justify-between">
					<span class="text-muted-foreground">Identifier:</span>
					<span>{appInfo.identifier}</span>
				</div>
				<div class="flex justify-between">
					<span class="text-muted-foreground">Copyright:</span>
					<span>{appInfo.copyright}</span>
				</div>
			</div>
		</div>
	{/if}

	<div class="text-xs text-muted-foreground text-center pt-4">
		Built with Tauri, SvelteKit, and Rust
	</div>
</div>
