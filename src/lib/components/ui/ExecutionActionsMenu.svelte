<script lang="ts">
	import UiTooltip from './UiTooltip.svelte';
	import type { Execution } from '../../types';
	import { X, PlayCircle, Check, RefreshCw, Trash2 } from 'lucide-svelte';

	let {
		execution,
		repoName,
		hasValidationPrompt = false,
		onDelete,
		onValidate,
		onStop,
		onStopValidation,
		onResume
	}: {
		execution: Execution;
		repoName: string;
		hasValidationPrompt?: boolean;
		onDelete: () => void;
		onValidate?: () => void;
		onStop?: () => void;
		onStopValidation?: () => void;
		onResume?: () => void;
	} = $props();

	let canValidate = $derived(
		hasValidationPrompt && 
		onValidate && 
		(execution.status === 'completed' || execution.status === 'cancelled') && 
		execution.validationStatus !== 'running'
	);

	let canStopExecution = $derived(
		onStop && execution.status === 'running'
	);

	let canStopValidation = $derived(
		onStopValidation && execution.validationStatus === 'running'
	);

	let canResume = $derived(
		onResume && execution.status === 'cancelled' && execution.sessionId
	);
</script>

<div class="flex items-center gap-1">
	{#if canStopExecution}
		<UiTooltip content="Stop execution">
			{#snippet children({ props })}
				<button
				{...props}
				onclick={() => onStop?.()}
				class="w-7 h-7 flex items-center justify-center rounded-md text-warning hover:bg-warning/10 transition-all"
				aria-label="Stop execution"
				>
				<X class="w-4 h-4" />
				</button>
			{/snippet}
		</UiTooltip>
	{/if}
	{#if canStopValidation}
		<UiTooltip content="Stop validation">
			{#snippet children({ props })}
				<button
				{...props}
				onclick={() => onStopValidation?.()}
				class="w-7 h-7 flex items-center justify-center rounded-md text-warning hover:bg-warning/10 transition-all"
				aria-label="Stop validation"
				>
				<X class="w-4 h-4" />
				</button>
			{/snippet}
		</UiTooltip>
	{/if}
	{#if canResume}
		<UiTooltip content="Resume execution">
			{#snippet children({ props })}
				<button
				{...props}
				onclick={() => onResume?.()}
				class="w-7 h-7 flex items-center justify-center rounded-md text-primary hover:bg-primary/10 transition-all"
				aria-label="Resume execution"
				>
				<PlayCircle class="w-4 h-4" />
				</button>
			{/snippet}
		</UiTooltip>
	{/if}
	{#if canValidate}
		<UiTooltip content={execution.validationStatus ? 'Revalidate' : 'Validate'}>
			{#snippet children({ props })}
				<button
				{...props}
				onclick={() => onValidate?.()}
				class="w-7 h-7 flex items-center justify-center rounded-md text-success hover:bg-success/10 transition-all"
				aria-label={execution.validationStatus ? 'Revalidate' : 'Validate'}
				>
				{#if execution.validationStatus}
				<RefreshCw class="w-4 h-4" />
				{:else}
				<Check class="w-4 h-4" />
				{/if}
				</button>
			{/snippet}
		</UiTooltip>
	{/if}
	<UiTooltip content="Delete execution">
		{#snippet children({ props })}
			<button
			{...props}
			onclick={onDelete}
			class="w-7 h-7 flex items-center justify-center rounded-md text-destructive hover:bg-destructive/10 transition-all"
			aria-label="Delete execution"
			>
			<Trash2 class="w-4 h-4" />
			</button>
		{/snippet}
	</UiTooltip>
</div>
