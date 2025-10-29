<script lang="ts">
	import { X, Pencil, Save } from "lucide-svelte"
	import { Switch } from "bits-ui"
	import { PaneGroup, Pane, PaneResizer } from "paneforge"
	import { settingsStore } from "$lib/stores/settingsStore"
	import type { PromptRevision } from "$lib/types"

	let {
		revision,
		validationPrompt = null,
		autoValidate = false,
		onSaveValidation,
	}: {
		revision: PromptRevision
		validationPrompt?: string | null
		autoValidate?: boolean
		onSaveValidation: (prompt: string, autoValidate: boolean) => Promise<void>
	} = $props()

	let isEditingValidation = $state(false)
	let editedValidationPrompt = $state(validationPrompt || "")
	let editedAutoValidate = $state(autoValidate)
	let isSaving = $state(false)
	let promptContentRef = $state<HTMLPreElement | null>(null)
	let validationContentRef = $state<HTMLElement | null>(null)
	let promptScrollContainerRef = $state<HTMLDivElement | null>(null)
	let validationScrollContainerRef = $state<HTMLDivElement | null>(null)
	let settings = $state<any>({})
	let isSyncingScroll = $state(false)

	$effect(() => {
		settingsStore.subscribe((s) => (settings = s))
	})

	function handlePromptSplitResize(sizes: number[]) {
		if (sizes[0] !== undefined) {
			settingsStore.updateUI({ promptSplitPct: sizes[0] })
		}
	}

	function startEditingValidation() {
		editedValidationPrompt = validationPrompt || ""
		editedAutoValidate = autoValidate
		isEditingValidation = true
	}

	function cancelEditingValidation() {
		isEditingValidation = false
		editedValidationPrompt = validationPrompt || ""
		editedAutoValidate = autoValidate
	}

	async function saveValidation() {
		isSaving = true
		try {
			await onSaveValidation(editedValidationPrompt, editedAutoValidate)
			isEditingValidation = false
		} finally {
			isSaving = false
		}
	}

	$effect(() => {
		editedValidationPrompt = validationPrompt || ""
		editedAutoValidate = autoValidate
	})

	function handlePromptScroll() {
		if (!promptScrollContainerRef || !validationScrollContainerRef || isSyncingScroll) return
		isSyncingScroll = true
		const scrollPercentage =
			promptScrollContainerRef.scrollTop /
			(promptScrollContainerRef.scrollHeight - promptScrollContainerRef.clientHeight || 1)
		validationScrollContainerRef.scrollTop =
			scrollPercentage *
			(validationScrollContainerRef.scrollHeight - validationScrollContainerRef.clientHeight)
		isSyncingScroll = false
	}

	function handleValidationScroll() {
		if (!promptScrollContainerRef || !validationScrollContainerRef || isSyncingScroll) return
		isSyncingScroll = true
		const scrollPercentage =
			validationScrollContainerRef.scrollTop /
			(validationScrollContainerRef.scrollHeight - validationScrollContainerRef.clientHeight || 1)
		promptScrollContainerRef.scrollTop =
			scrollPercentage *
			(promptScrollContainerRef.scrollHeight - promptScrollContainerRef.clientHeight)
		isSyncingScroll = false
	}
</script>

<div class="h-full flex flex-col overflow-hidden">
	<!-- Prompt Content -->
	<div class="flex-1 overflow-hidden bg-card">
		<div class="h-full flex divide-x divide-border/20">
			<PaneGroup direction="horizontal" onLayoutChange={handlePromptSplitResize}>
				<Pane defaultSize={settings.ui?.promptSplitPct ?? 50}>
					<!-- Revision Prompt (Left) -->
					<div class="flex-1 flex flex-col min-w-0 h-full">
						<div class="px-4 py-2 bg-muted/10 border-b border-border/10">
							<h3 class="text-xs font-semibold text-muted-foreground">Revision Prompt</h3>
						</div>
						<div bind:this={promptScrollContainerRef} class="flex-1 overflow-auto" onscroll={handlePromptScroll}>
							<pre
								bind:this={promptContentRef}
								class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{revision.promptText}</pre>
						</div>
					</div>
				</Pane>
				<PaneResizer
					class="w-1 bg-border/40 hover:bg-primary/40 transition-colors cursor-col-resize"
				/>
				<Pane>
					<!-- Validation Prompt (Right) -->
					{#if !validationPrompt && !isEditingValidation}
						<div class="flex-1 flex flex-col min-w-0 items-center justify-center">
							<button
								onclick={startEditingValidation}
								class="text-sm text-muted-foreground hover:text-foreground transition-colors"
							>
								+ Add validation prompt
							</button>
						</div>
					{:else if validationPrompt || isEditingValidation}
						<div class="flex-1 flex flex-col min-w-0">
							<div
								class="px-4 py-2 bg-muted/10 border-b border-border/10 flex items-center justify-between gap-3"
							>
								<h3 class="text-xs font-semibold text-muted-foreground">Validation Prompt</h3>
								<div class="flex items-center gap-3">
									<div class="flex items-center gap-2">
										<span class="text-xs text-muted-foreground">Auto-validate</span>
										<Switch.Root
											bind:checked={editedAutoValidate}
											disabled={!isEditingValidation || isSaving}
											class="data-[state=checked]:bg-green-600 data-[state=unchecked]:bg-muted inline-flex h-4 w-7 shrink-0 cursor-pointer items-center rounded-full px-[2px] transition-colors focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
										>
											<Switch.Thumb
												class="pointer-events-none block h-3 w-3 rounded-full bg-white transition-transform data-[state=checked]:translate-x-[14px] data-[state=unchecked]:translate-x-0"
											/>
										</Switch.Root>
									</div>

									{#if isEditingValidation}
										<button
											onclick={cancelEditingValidation}
											disabled={isSaving}
											class="text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
											aria-label="Cancel"
										>
											<X class="w-4 h-4" />
										</button>
										<button
											onclick={saveValidation}
											disabled={isSaving}
											class="text-success hover:text-success/90 transition-colors disabled:opacity-50"
											aria-label="Save validation prompt"
										>
											<Save class="w-4 h-4" />
										</button>
									{:else}
										<button
											onclick={startEditingValidation}
											class="text-primary hover:text-primary/90 transition-colors"
											aria-label="Edit validation prompt"
										>
											<Pencil class="w-4 h-4" />
										</button>
									{/if}
								</div>
							</div>
							<div bind:this={validationScrollContainerRef} class="flex-1 overflow-auto" onscroll={handleValidationScroll}>
								{#if isEditingValidation}
									<textarea
										bind:this={validationContentRef}
										bind:value={editedValidationPrompt}
										disabled={isSaving}
										placeholder="Enter a prompt to validate each execution after it completes..."
										class="w-full h-full px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground bg-transparent border-none outline-none resize-none disabled:opacity-50"
									></textarea>
								{:else}
									<pre
										bind:this={validationContentRef}
										class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{validationPrompt}</pre>
								{/if}
							</div>
						</div>
					{/if}
				</Pane>
			</PaneGroup>
		</div>
	</div>
</div>
