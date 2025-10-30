<script lang="ts">
	import { Dialog, Checkbox } from "bits-ui"
	import { CheckCircle2, XCircle, Loader2 } from "lucide-svelte"
	import { onMount } from "svelte"
	import {
		healthCheckGit,
		healthCheckNodejs,
		setFirstRunComplete,
		setShowFirstRunDialog,
	} from "$lib/ipc"
	import type { HealthCheckResult } from "$lib/types"

	type PageContent = {
		heading: string
		description: string
		features: Array<{ title: string; desc: string }>
	}

	interface Props {
		open?: boolean
		onComplete: () => void
	}

	let { open = $bindable(true), onComplete }: Props = $props()

	let page = $state(0)
	let gitCheck = $state<HealthCheckResult | null>(null)
	let nodejsCheck = $state<HealthCheckResult | null>(null)
	let checksComplete = $state(false)
	let showOnStartup = $state(true)

	async function runHealthChecks() {
		const [git, nodejs] = await Promise.all([healthCheckGit(), healthCheckNodejs()])
		gitCheck = git
		nodejsCheck = nodejs
		checksComplete = true
	}

	onMount(() => {
		runHealthChecks()
	})

	const allChecksPassed = $derived(checksComplete && gitCheck?.success && nodejsCheck?.success)

	async function handleFinish() {
		await setFirstRunComplete()
		await setShowFirstRunDialog(showOnStartup)
		open = false
		onComplete()
	}

	const pages: Array<{
		title: string
		content: () => PageContent | Record<string, never>
		isHealthCheck?: boolean
	}> = [
		{
			title: "System Requirements",
			content: () => ({}) as Record<string, never>,
			isHealthCheck: true,
		},
		{
			title: "Welcome to Maestro",
			content: () => ({
				heading: "Prompt Sets",
				description:
					"A Prompt Set is a collection of prompts that work together to accomplish a goal across multiple repositories.",
				features: [
					{
						title: "Repository Selection",
						desc: "Choose which repositories your prompts will be executed against",
					},
					{
						title: "Execution Prompts",
						desc: "The main prompt that Amp AI will execute in each repository",
					},
					{
						title: "Validation Prompts",
						desc: "Optional prompts to validate the execution results automatically",
					},
				],
			}),
		},
		{
			title: "Prompt Revisions",
			content: () => ({
				heading: "Iterate and Improve",
				description:
					"Each time you modify your prompt, Maestro creates a new revision. This allows you to track changes, compare results, and roll back if needed.",
				features: [
					{
						title: "Version History",
						desc: "All prompt changes are tracked automatically",
					},
					{
						title: "Execution Tracking",
						desc: "See which revisions have been executed and their results",
					},
					{
						title: "Easy Comparison",
						desc: "Compare results across different prompt revisions",
					},
				],
			}),
		},
		{
			title: "Execution & Analysis",
			content: () => ({
				heading: "Execute and Validate",
				description:
					"Maestro executes your prompts across repositories in isolated worktrees, validates results, and helps you analyze failures.",
				features: [
					{
						title: "Isolated Execution",
						desc: "Each execution runs in a separate git worktree for safety",
					},
					{
						title: "Automated Validation",
						desc: "Optional automated validation of execution results",
					},
					{
						title: "Failure Analysis",
						desc: "AI-powered analysis of failures across multiple executions",
					},
				],
			}),
		},
	]

	function nextPage() {
		if (page < pages.length - 1) page++
	}

	function prevPage() {
		if (page > 0) page--
	}

	const canProceed = $derived(page === 0 ? allChecksPassed : true)
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 bg-background/80 backdrop-blur-sm z-50" />
		<Dialog.Content
			class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-2xl bg-card border border-border rounded-lg shadow-lg p-6"
		>
			<Dialog.Title class="text-lg font-semibold mb-4">{pages[page].title}</Dialog.Title>

			<div class="min-h-[24rem]">
				{#if page === 0}
					<div class="space-y-4">
						<p class="text-sm text-muted-foreground mb-6">
							Maestro requires the following system dependencies:
						</p>

						<div class="space-y-3">
							<!-- Git Check -->
							<div
								class="flex items-center justify-between p-3 bg-background rounded border border-border"
							>
								<div class="flex items-center gap-3">
									{#if !gitCheck}
										<Loader2 class="h-4 w-4 animate-spin text-primary" />
									{:else if gitCheck.success}
										<CheckCircle2 class="h-4 w-4 text-success" />
									{:else}
										<XCircle class="h-4 w-4 text-destructive" />
									{/if}
									<div>
										<div class="font-medium">Git</div>
										{#if gitCheck?.success}
											<div class="text-xs text-muted-foreground">{gitCheck.username}</div>
										{:else if gitCheck?.error}
											<div class="text-xs text-destructive">{gitCheck.error}</div>
										{/if}
									</div>
								</div>
							</div>

							<!-- Node.js Check -->
							<div
								class="flex items-center justify-between p-3 bg-background rounded border border-border"
							>
								<div class="flex items-center gap-3">
									{#if !nodejsCheck}
										<Loader2 class="h-4 w-4 animate-spin text-primary" />
									{:else if nodejsCheck.success}
										<CheckCircle2 class="h-4 w-4 text-success" />
									{:else}
										<XCircle class="h-4 w-4 text-destructive" />
									{/if}
									<div>
										<div class="font-medium">Node.js</div>
										{#if nodejsCheck?.success}
											<div class="text-xs text-muted-foreground">{nodejsCheck.username}</div>
										{:else if nodejsCheck?.error}
											<div class="text-xs text-destructive">{nodejsCheck.error}</div>
										{/if}
									</div>
								</div>
							</div>
						</div>

						{#if checksComplete && !allChecksPassed}
							<div class="mt-6 p-4 bg-destructive/10 border border-destructive/20 rounded">
								<p class="text-sm text-destructive font-medium">
									Please install the missing dependencies before continuing.
								</p>
							</div>
						{/if}
					</div>
				{:else}
					{@const pageContent = pages[page].content()}
					<div class="space-y-6">
						<div>
							<h3 class="text-base font-semibold mb-2">{pageContent.heading}</h3>
							<p class="text-sm text-muted-foreground">{pageContent.description}</p>
						</div>

						<div class="space-y-4">
							{#each pageContent.features as feature}
								<div class="p-4 bg-background rounded border border-border">
									<div class="font-medium mb-1">{feature.title}</div>
									<div class="text-sm text-muted-foreground">{feature.desc}</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<div class="mt-6 pt-4 border-t border-border space-y-4">
				<label class="flex items-center gap-2 text-sm cursor-pointer">
					<Checkbox.Root
						bind:checked={showOnStartup}
						class="h-4 w-4 rounded border border-border bg-background flex items-center justify-center data-[state=checked]:bg-primary data-[state=checked]:border-primary"
					>
						{#snippet children({ checked })}
							{#if checked}
								<CheckCircle2 class="h-3 w-3 text-primary-foreground" />
							{/if}
						{/snippet}
					</Checkbox.Root>
					<span class="text-muted-foreground">Show this dialog on startup</span>
				</label>

				<div class="flex items-center justify-between">
					<div class="flex gap-2">
						{#each pages as _, idx}
							<button
								class="h-2 w-2 rounded-full transition-colors {idx === page
									? 'bg-primary'
									: 'bg-muted-foreground/30'}"
								onclick={() => (page = idx)}
								disabled={idx === 0 && !allChecksPassed}
								aria-label="Page {idx + 1}"
							></button>
						{/each}
					</div>

					<div class="flex gap-2">
						{#if page > 0}
							<button
								onclick={prevPage}
								class="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-muted rounded"
							>
								Back
							</button>
						{/if}

						{#if page < pages.length - 1}
							<button
								onclick={nextPage}
								disabled={!canProceed}
								class="px-3 py-1.5 text-sm bg-primary text-primary-foreground hover:bg-primary/90 rounded disabled:opacity-50 disabled:cursor-not-allowed"
							>
								Next
							</button>
						{:else}
							<button
								onclick={handleFinish}
								class="px-3 py-1.5 text-sm bg-primary text-primary-foreground hover:bg-primary/90 rounded"
							>
								Get Started
							</button>
						{/if}
					</div>
				</div>
			</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
