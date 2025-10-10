<script lang="ts">
	import type { Execution } from '../../types';
	import UiTooltip from './UiTooltip.svelte';
	import { openInBrowser } from '$lib/utils/browser';
	import { DropdownMenu } from 'bits-ui';
	import { 
		PlayCircle, 
		Square, 
		CheckCircle2, 
		XCircle, 
		Ban, 
		GitCommit, 
		GitBranch,
		ExternalLink,
		FileText,
		X,
		RefreshCw,
		Trash2,
		Loader2,
		Clock,
		RotateCw,
		MoreVertical
	} from 'lucide-svelte';

	let {
		execution,
		repoName,
		hasValidationPrompt = false,
		selected = false,
		onToggleSelect,
		onDelete,
		onValidate,
		onStop,
		onStopValidation,
		onResume,
		onReviewChanges,
		fileCount = 0,
		additions = 0,
		deletions = 0,
		progressMessage
	}: {
		execution: Execution;
		repoName: string;
		hasValidationPrompt?: boolean;
		selected?: boolean;
		onToggleSelect?: () => void;
		onDelete: () => void;
		onValidate?: () => void;
		onStop?: () => void;
		onStopValidation?: () => void;
		onResume?: () => void;
		onReviewChanges?: () => void;
		fileCount?: number;
		additions?: number;
		deletions?: number;
		progressMessage?: string;
	} = $props();



	// Reactive icon/color for execution status
	let executionIcon = $derived.by(() => {
		switch (execution.status) {
			case 'running': return { Icon: Loader2, class: 'text-blue-600 animate-spin' };
			case 'completed': return { Icon: CheckCircle2, class: 'text-green-600' };
			case 'failed': return { Icon: XCircle, class: 'text-red-600' };
			case 'cancelled': return { Icon: Ban, class: 'text-orange-600' };
			case 'pending': return { Icon: Clock, class: 'text-gray-400' };
			default: return { Icon: Clock, class: 'text-gray-400' };
		}
	});

	// Reactive icon/color for validation status
	let validationIcon = $derived.by(() => {
		if (!execution.validationStatus) return null;
		switch (execution.validationStatus) {
			case 'running': return { Icon: Loader2, class: 'text-blue-600 animate-spin' };
			case 'passed': return { Icon: CheckCircle2, class: 'text-green-600' };
			case 'failed': return { Icon: XCircle, class: 'text-red-600' };
			default: return null;
		}
	});

	// Reactive icon/color for commit status
	let commitIcon = $derived.by(() => {
		switch (execution.commitStatus) {
			case 'committed': return { Icon: GitCommit, class: 'text-green-600' };
			case 'uncommitted': return { Icon: GitBranch, class: 'text-orange-600' };
			default: return null;
		}
	});

	let canValidate = $derived(
		hasValidationPrompt && 
		onValidate && 
		(execution.status === 'completed' || execution.status === 'cancelled') && 
		execution.validationStatus !== 'running'
	);

	let isRunning = $derived(
		execution.status === 'running' || execution.validationStatus === 'running'
	);

	let rowClass = $derived.by(() => {
		if (selected) return 'bg-primary/10 border-l-4 border-l-primary';
		return '';
	});
</script>

<div
	class="grid gap-3 px-4 py-2.5 border-b border-border/40 hover:bg-muted/30 transition-all group items-center {rowClass}
	       [grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1fr)]
	       @max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_0.8fr)_minmax(0,_1fr)_minmax(0,_0.8fr)]
	       @max-md/table:[grid-template-columns:auto_minmax(200px,_6fr)_40px_40px_0_0_40px]"
>
	<!-- Checkbox -->
	<button
		type="button"
		onclick={onToggleSelect}
		disabled={isRunning}
		class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2 {selected ? 'border-primary bg-primary' : 'border-muted-foreground/30 hover:border-primary/50'} transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
		aria-label={selected ? 'Deselect' : 'Select'}
	>
		{#if selected}
			<CheckCircle2 class="w-4 h-4 text-primary-foreground" />
		{/if}
	</button>

	<!-- Repository Name -->
	<div class="overflow-hidden min-w-0">
		<span class="text-sm font-medium text-foreground truncate block">{repoName}</span>
	</div>

	<!-- Execution Status -->
	<div class="flex items-center gap-2">
		<UiTooltip content={progressMessage || `Execution: ${execution.status}`}>
			{#snippet children({ props })}
				{@const Icon = executionIcon.Icon}
				{#if execution.threadUrl}
					<button
						{...props}
						onclick={() => openInBrowser(execution.threadUrl!)}
						class="flex items-center gap-1.5 hover:opacity-80 transition-opacity"
					>
						<Icon class={`w-4 h-4 ${executionIcon.class}`} />
						<span class="hidden @lg/table:inline text-xs {execution.status === 'running' ? 'text-blue-600 font-medium' : 'text-muted-foreground'} hover:text-foreground flex items-center gap-1">
							{#if progressMessage}
								{progressMessage}
							{:else if execution.status === 'running'}
								Running
							{:else}
								Thread
							{/if}
							<ExternalLink class="w-3 h-3" />
						</span>
					</button>
				{:else}
					<div {...props} class="flex items-center gap-1.5">
						<Icon class={`w-4 h-4 ${executionIcon.class}`} />
						<span class="hidden @lg/table:inline text-xs {execution.status === 'running' ? 'text-blue-600 font-medium' : 'text-muted-foreground'}">{progressMessage || execution.status}</span>
					</div>
				{/if}
			{/snippet}
		</UiTooltip>
	</div>

	<!-- Validation Status -->
	<div class="flex items-center gap-2">
		{#if validationIcon}
			<UiTooltip content={`Validation: ${execution.validationStatus}`}>
				{#snippet children({ props })}
					{@const Icon = validationIcon.Icon}
					{#if execution.validationThreadUrl}
						<button
							{...props}
							onclick={() => openInBrowser(execution.validationThreadUrl!)}
							class="flex items-center gap-1.5 hover:opacity-80 transition-opacity"
						>
							<Icon class={`w-4 h-4 ${validationIcon.class}`} />
							<span class="hidden @lg/table:inline text-xs {execution.validationStatus === 'running' ? 'text-green-600 font-medium' : 'text-muted-foreground'} hover:text-foreground flex items-center gap-1">
								{#if execution.validationStatus === 'running'}
									Running
								{:else}
									Thread
								{/if}
								<ExternalLink class="w-3 h-3" />
							</span>
						</button>
					{:else}
						<div {...props} class="flex items-center gap-1.5">
							<Icon class={`w-4 h-4 ${validationIcon.class}`} />
							<span class="hidden @lg/table:inline text-xs {execution.validationStatus === 'running' ? 'text-green-600 font-medium' : 'text-muted-foreground'}">{execution.validationStatus}</span>
						</div>
					{/if}
				{/snippet}
			</UiTooltip>
		{:else}
			<span class="text-xs text-muted-foreground">—</span>
		{/if}
	</div>

	<!-- Commit Status -->
	<div class="flex items-center gap-2 @max-md/table:hidden">
		{#if commitIcon}
			<UiTooltip content={execution.commitSha ? `Click to view diff: ${execution.commitSha.slice(0, 7)}` : execution.commitStatus || ''}>
				{#snippet children({ props })}
					{@const Icon = commitIcon.Icon}
					<button
						{...props}
						onclick={() => onReviewChanges?.()}
						class="flex items-center gap-1.5 hover:text-blue-600 transition-colors cursor-pointer"
						disabled={!onReviewChanges}
					>
						<Icon class={`w-4 h-4 ${commitIcon.class}`} />
						{#if execution.commitSha}
							<span class="text-xs font-mono text-muted-foreground hover:text-blue-600">{execution.commitSha.slice(0, 7)}</span>
						{:else}
							<span class="text-xs text-muted-foreground">{execution.commitStatus}</span>
						{/if}
					</button>
				{/snippet}
			</UiTooltip>
		{:else}
			<span class="text-xs text-muted-foreground">—</span>
		{/if}
	</div>

	<!-- Changes Stats -->
	<button
		onclick={() => onReviewChanges?.()}
		disabled={!onReviewChanges}
		class="flex items-center gap-3 text-xs hover:text-blue-600 transition-colors cursor-pointer disabled:cursor-default disabled:hover:text-current @max-md/table:hidden"
	>
		{#if fileCount > 0}
			<UiTooltip content={`Click to view ${fileCount} file${fileCount !== 1 ? 's' : ''} changed`}>
				{#snippet children({ props })}
					<div {...props} class="flex items-center gap-1">
						<FileText class="w-3.5 h-3.5 text-muted-foreground" />
						<span class="text-muted-foreground">{fileCount}</span>
					</div>
				{/snippet}
			</UiTooltip>
			<span class="text-green-600">+{additions}</span>
			<span class="text-red-600">-{deletions}</span>
		{:else}
			<span class="text-muted-foreground">No changes</span>
		{/if}
	</button>

	<!-- Actions -->
	<div class="flex items-center gap-1 justify-end">
		<div class="hidden @md/table:flex items-center gap-1">
			{#if onReviewChanges && fileCount > 0}
				<UiTooltip content="Review changes">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={onReviewChanges}
							class="w-7 h-7 flex items-center justify-center rounded-md text-blue-600 hover:text-blue-700 hover:bg-blue-50 transition-all"
							aria-label="Review changes"
						>
							<FileText class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}

			{#if execution.status === 'completed' || execution.status === 'failed' || execution.status === 'cancelled'}
				<UiTooltip content="Restart execution">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => onResume?.()}
							class="w-7 h-7 flex items-center justify-center rounded-md text-blue-600 hover:bg-blue-50 transition-all"
							aria-label="Restart execution"
						>
							<RotateCw class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}

			{#if execution.status === 'running'}
				<UiTooltip content="Stop execution">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => onStop?.()}
							class="w-7 h-7 flex items-center justify-center rounded-md bg-orange-100 text-orange-700 hover:bg-orange-200 transition-all shadow-sm ring-1 ring-orange-300"
							aria-label="Stop execution"
						>
							<Square class="w-4 h-4 fill-current" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}

			{#if execution.validationStatus === 'running'}
				<UiTooltip content="Stop validation">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => onStopValidation?.()}
							class="w-7 h-7 flex items-center justify-center rounded-md bg-orange-100 text-orange-700 hover:bg-orange-200 transition-all shadow-sm ring-1 ring-orange-300"
							aria-label="Stop validation"
						>
							<X class="w-4 h-4" />
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
							class="w-7 h-7 flex items-center justify-center rounded-md text-green-600 hover:bg-green-50 transition-all"
							aria-label={execution.validationStatus ? 'Revalidate' : 'Validate'}
						>
							{#if execution.validationStatus}
								<RefreshCw class="w-4 h-4" />
							{:else}
								<CheckCircle2 class="w-4 h-4" />
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
						class="w-7 h-7 flex items-center justify-center rounded-md text-red-600 hover:bg-red-50 transition-all"
						aria-label="Delete execution"
					>
						<Trash2 class="w-4 h-4" />
					</button>
				{/snippet}
			</UiTooltip>
		</div>

		<!-- Compact dropdown menu -->
		<div class="flex @md/table:hidden items-center gap-1">
			<DropdownMenu.Root>
				<DropdownMenu.Trigger 
					class="w-7 h-7 flex items-center justify-center rounded-md text-muted-foreground hover:bg-muted transition-all"
					aria-label="Open row actions"
				>
					<MoreVertical class="w-4 h-4" />
				</DropdownMenu.Trigger>
				<DropdownMenu.Content 
					class="min-w-[180px] max-w-[250px] bg-card text-card-foreground rounded-lg shadow-xl border border-border/40 p-1"
					align="end"
					sideOffset={4}
					strategy="fixed"
				>
					{#if onReviewChanges && fileCount > 0}
						<DropdownMenu.Item
							onSelect={onReviewChanges}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
						>
							<FileText class="w-4 h-4" />
							Review changes
						</DropdownMenu.Item>
					{/if}
					
					{#if execution.status === 'completed' || execution.status === 'failed' || execution.status === 'cancelled'}
						<DropdownMenu.Item
							onSelect={() => onResume?.()}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
						>
							<RotateCw class="w-4 h-4" />
							Restart execution
						</DropdownMenu.Item>
					{/if}
					
					{#if execution.status === 'running'}
						<DropdownMenu.Item
							onSelect={() => onStop?.()}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
						>
							<Square class="w-4 h-4" />
							Stop execution
						</DropdownMenu.Item>
					{/if}
					
					{#if execution.validationStatus === 'running'}
						<DropdownMenu.Item
							onSelect={() => onStopValidation?.()}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
						>
							<X class="w-4 h-4" />
							Stop validation
						</DropdownMenu.Item>
					{/if}
					
					{#if canValidate}
						<DropdownMenu.Item
							onSelect={() => onValidate?.()}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
						>
							{#if execution.validationStatus}
								<RefreshCw class="w-4 h-4" />
								Revalidate
							{:else}
								<CheckCircle2 class="w-4 h-4" />
								Validate
							{/if}
						</DropdownMenu.Item>
					{/if}
					
					<DropdownMenu.Separator class="h-px bg-border/40 my-1" />
					
					<DropdownMenu.Item
						onSelect={onDelete}
						class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-destructive hover:text-destructive-foreground rounded transition-colors cursor-pointer"
					>
						<Trash2 class="w-4 h-4" />
						Delete execution
					</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		</div>
	</div>
</div>
