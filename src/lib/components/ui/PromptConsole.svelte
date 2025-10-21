<script lang="ts">
	import { X, Edit, Save } from 'lucide-svelte';
	import { Switch } from 'bits-ui';
	import type { PromptRevision } from '$lib/types';

	let {
		revision,
		validationPrompt = null,
		autoValidate = false,
		onSaveValidation
	}: {
		revision: PromptRevision;
		validationPrompt?: string | null;
		autoValidate?: boolean;
		onSaveValidation: (prompt: string, autoValidate: boolean) => Promise<void>;
	} = $props();

	let promptHeight = $state(256);
	let isResizing = $state(false);
	let isEditingValidation = $state(false);
	let editedValidationPrompt = $state(validationPrompt || '');
	let editedAutoValidate = $state(autoValidate);
	let isSaving = $state(false);
	let manuallyResized = $state(false);
	let promptContentRef = $state<HTMLPreElement | null>(null);
	let validationContentRef = $state<HTMLElement | null>(null);

	function handleResizeStart(e: MouseEvent) {
		isResizing = true;
		manuallyResized = true;
		e.preventDefault();
	}

	function handleResizeMove(e: MouseEvent) {
		if (!isResizing) return;
		promptHeight = Math.max(100, Math.min(window.innerHeight * 0.5, e.clientY - 100));
	}

	function handleResizeEnd() {
		isResizing = false;
	}

	function calculateAutoHeight(): number {
		if (manuallyResized) return promptHeight;
		
		// Calculate based on content height
		const promptContentHeight = promptContentRef?.scrollHeight || 0;
		const validationContentHeight = validationContentRef?.scrollHeight || 0;
		const maxContentHeight = Math.max(promptContentHeight, validationContentHeight);
		
		// Add padding and header space (approx 48px for header)
		const neededHeight = maxContentHeight + 48;
		
		// Cap at 50% of viewport height
		const maxViewportHeight = Math.floor(window.innerHeight * 0.5);
		
		// Use at least 200px, cap at 50% viewport or content height (whichever is smaller)
		return Math.max(200, Math.min(neededHeight, maxViewportHeight));
	}

	let computedHeight = $derived(calculateAutoHeight());

	$effect(() => {
		if (isResizing) {
			document.addEventListener('mousemove', handleResizeMove);
			document.addEventListener('mouseup', handleResizeEnd);
			return () => {
				document.removeEventListener('mousemove', handleResizeMove);
				document.removeEventListener('mouseup', handleResizeEnd);
			};
		}
	});

	function startEditingValidation() {
		editedValidationPrompt = validationPrompt || '';
		editedAutoValidate = autoValidate;
		isEditingValidation = true;
	}

	function cancelEditingValidation() {
		isEditingValidation = false;
		editedValidationPrompt = validationPrompt || '';
		editedAutoValidate = autoValidate;
	}

	async function saveValidation() {
		isSaving = true;
		try {
			await onSaveValidation(editedValidationPrompt, editedAutoValidate);
			isEditingValidation = false;
		} finally {
			isSaving = false;
		}
	}

	$effect(() => {
		editedValidationPrompt = validationPrompt || '';
		editedAutoValidate = autoValidate;
	});
</script>

<div class="flex-shrink-0">
	<!-- Prompt Content -->
	<div class="border-b border-border/20 overflow-hidden bg-card">
		<div class="flex divide-x divide-border/20" style="height: {manuallyResized ? promptHeight : computedHeight}px;">
			<!-- Revision Prompt (Left) -->
			<div class="flex-1 flex flex-col min-w-0">
				<div class="px-4 py-2 bg-muted/10 border-b border-border/10">
					<h3 class="text-xs font-semibold text-muted-foreground">Revision Prompt</h3>
				</div>
				<div class="flex-1 overflow-auto">
					<pre bind:this={promptContentRef} class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{revision.promptText}</pre>
				</div>
			</div>
			
			<!-- Validation Prompt (Right) -->
			{#if validationPrompt || isEditingValidation}
				<div class="flex-1 flex flex-col min-w-0">
					<div class="px-4 py-2 bg-muted/10 border-b border-border/10 flex items-center justify-between gap-3">
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
									class="text-green-600 hover:text-green-700 transition-colors disabled:opacity-50"
									aria-label="Save validation prompt"
								>
									<Save class="w-4 h-4" />
								</button>
							{:else}
								<button
									onclick={startEditingValidation}
									class="text-blue-600 hover:text-blue-700 transition-colors"
									aria-label="Edit validation prompt"
								>
									<Edit class="w-4 h-4" />
								</button>
							{/if}
						</div>
					</div>
					<div class="flex-1 overflow-auto">
					{#if isEditingValidation}
					<textarea
					bind:this={validationContentRef}
					bind:value={editedValidationPrompt}
					disabled={isSaving}
					placeholder="Enter a prompt to validate each execution after it completes..."
					 class="w-full h-full px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground bg-transparent border-none outline-none resize-none disabled:opacity-50"
					 ></textarea>
					{:else}
					 <pre bind:this={validationContentRef} class="px-6 py-6 text-sm whitespace-pre-wrap font-mono leading-relaxed text-foreground">{validationPrompt}</pre>
					 {/if}
				</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Resize Handle -->
	<button
		onmousedown={handleResizeStart}
		class="flex-shrink-0 h-1.5 bg-border/40 hover:bg-primary/40 transition-colors cursor-ns-resize group relative"
		aria-label="Resize prompt area"
	>
		<div class="absolute inset-x-0 -top-1 -bottom-1"></div>
	</button>
</div>
