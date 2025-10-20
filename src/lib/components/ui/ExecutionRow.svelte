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
		MoreVertical,
		Upload,
		Code,
		Copy
	} from 'lucide-svelte';
	import CiStatusBadge from './CiStatusBadge.svelte';
	import { openInEditor, copyWorktreePath } from '$lib/utils/worktree';
	import { settingsStore } from '$lib/stores/settingsStore';
	import { executionStore } from '$lib/stores/executionBus';

	let {
		execution,
		repoName,
		hasValidationPrompt = false,
		selected = false,
		onToggleSelect,
		onDelete,
		onStart,
		onValidate,
		onStop,
		onStopValidation,
		onResume,
		onReviewChanges,
		onPush,
		onRefreshCi,
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
		onStart?: () => void;
		onValidate?: () => void;
		onStop?: () => void;
		onStopValidation?: () => void;
		onResume?: () => void;
		onReviewChanges?: () => void;
		onPush?: () => void;
		onRefreshCi?: () => void;
		fileCount?: number;
		additions?: number;
		deletions?: number;
		progressMessage?: string;
	} = $props();

	// Merge prop with live updates from event bus
	let liveExecution = $derived.by(() => {
		const updates = $executionStore.get(execution.id);
		if (!updates) return execution;
		return {
			...execution,
			...(updates.sessionId && { sessionId: updates.sessionId }),
			...(updates.threadUrl && { threadUrl: updates.threadUrl }),
			...(updates.status && { status: updates.status }),
			...(updates.validationStatus && { validationStatus: updates.validationStatus }),
			...(updates.validationThreadUrl && { validationThreadUrl: updates.validationThreadUrl }),
			...(updates.commitStatus && { commitStatus: updates.commitStatus }),
			...(updates.commitSha && { commitSha: updates.commitSha }),
			...(updates.committedAt && { committedAt: updates.committedAt }),
			...(updates.ciStatus && { ciStatus: updates.ciStatus }),
			...(updates.ciUrl && { ciUrl: updates.ciUrl })
		};
	});

	// Reactive icon/color for execution status
	let executionIcon = $derived.by(() => {
		switch (liveExecution.status) {
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
		if (!liveExecution.validationStatus) return null;
		switch (liveExecution.validationStatus) {
			case 'running': return { Icon: Loader2, class: 'text-blue-600 animate-spin' };
			case 'passed': return { Icon: CheckCircle2, class: 'text-green-600' };
			case 'failed': return { Icon: XCircle, class: 'text-red-600' };
			default: return null;
		}
	});

	// Reactive icon/color for commit status
	let commitIcon = $derived.by(() => {
		switch (liveExecution.commitStatus) {
			case 'committed': return { Icon: GitCommit, class: 'text-green-600' };
			case 'uncommitted': return { Icon: GitBranch, class: 'text-orange-600' };
			default: return null;
		}
	});

	// Execution action states
	let canStart = $derived(
		onStart &&
		liveExecution.status === 'pending' &&
		!liveExecution.sessionId &&
		!liveExecution.threadUrl
	);

	let canStop = $derived(
		onStop &&
		liveExecution.status === 'running'
	);

	let canRestart = $derived(
		onResume &&
		(liveExecution.status === 'completed' || 
		 liveExecution.status === 'failed' || 
		 liveExecution.status === 'cancelled')
	);

	// Validation action states
	let canValidate = $derived(
		hasValidationPrompt && 
		onValidate && 
		(liveExecution.status === 'completed' || liveExecution.status === 'cancelled') && 
		!liveExecution.validationStatus
	);

	let canRevalidate = $derived(
		hasValidationPrompt &&
		onValidate &&
		liveExecution.validationStatus &&
		liveExecution.validationStatus !== 'running'
	);

	let canStopValidation = $derived(
		onStopValidation &&
		liveExecution.validationStatus === 'running'
	);

	// Other action states
	let canPush = $derived(
		onPush && 
		liveExecution.commitStatus === 'committed' &&
		liveExecution.commitSha
	);

	let isRunning = $derived(
		liveExecution.status === 'running' || liveExecution.validationStatus === 'running'
	);

	let rowClass = $derived.by(() => {
		if (selected) return 'bg-primary/10 border-l-4 border-l-primary';
		return '';
	});

	async function handleOpenInEditor() {
		try {
			await openInEditor(liveExecution);
		} catch (error) {
			console.error('Failed to open in editor:', error);
		}
	}

	async function handleCopyPath() {
		try {
			await copyWorktreePath(liveExecution);
		} catch (error) {
			console.error('Failed to copy worktree path:', error);
		}
	}
</script>

<div
	class="grid gap-3 px-4 py-2.5 border-b border-border/10 bg-card hover:bg-muted/30 transition-all group items-center {rowClass}
	       [grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]
	       @max-lg/table:[grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.2fr)_minmax(0,_0.8fr)_minmax(0,_0.8fr)]
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
	<div class="flex flex-col gap-1">
	<div class="flex items-center gap-2">
	<UiTooltip content={progressMessage || `Execution: ${liveExecution.status}`}>
	{#snippet children({ props })}
	{@const Icon = executionIcon.Icon}
	<div {...props}>
	<Icon class={`w-4 h-4 ${executionIcon.class}`} />
	</div>
	{/snippet}
	</UiTooltip>
	
	<!-- Start/Stop/Restart execution actions -->
	{#if canStart}
	<UiTooltip content="Start execution">
	{#snippet children({ props })}
	<button
	{...props}
	onclick={() => onStart?.()}
	class="text-green-600 hover:text-green-700 transition-colors"
	aria-label="Start execution"
	>
	<PlayCircle class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{:else if canStop}
	<UiTooltip content="Stop execution">
	{#snippet children({ props })}
	<button
	{...props}
	onclick={() => onStop?.()}
	class="text-orange-600 hover:text-orange-700 transition-colors"
	aria-label="Stop execution"
	>
	<Square class="w-4 h-4 fill-current" />
	</button>
	{/snippet}
	</UiTooltip>
	{:else if canRestart}
	<UiTooltip content="Restart execution">
	{#snippet children({ props })}
	<button
	{...props}
	onclick={() => onResume?.()}
	class="text-blue-600 hover:text-blue-700 transition-colors"
	aria-label="Restart execution"
	>
	<RotateCw class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{/if}
	</div>
	
	<!-- Execution Thread Link -->
	{#if liveExecution.threadUrl}
	<button
	onclick={() => openInBrowser(liveExecution.threadUrl!)}
	class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
	>
	<span>Thread</span>
	<ExternalLink class="w-3 h-3" />
	</button>
	{/if}
	</div>

	<!-- Validation Status -->
	<div class="flex flex-col gap-1">
	{#if validationIcon}
	<div class="flex items-center gap-2">
	<UiTooltip content={`Validation: ${liveExecution.validationStatus}`}>
	{#snippet children({ props })}
	{@const Icon = validationIcon.Icon}
	<div {...props}>
	<Icon class={`w-4 h-4 ${validationIcon.class}`} />
	</div>
	{/snippet}
	</UiTooltip>
	
	<!-- Validation actions -->
	{#if canStopValidation}
	<UiTooltip content="Stop validation">
	{#snippet children({ props })}
	<button
	{...props}
	onclick={() => onStopValidation?.()}
	class="text-orange-600 hover:text-orange-700 transition-colors"
	aria-label="Stop validation"
	>
	<X class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{:else if canValidate}
	<UiTooltip content="Start validation">
	{#snippet children({ props })}
	<button
	{...props}
	onclick={() => onValidate?.()}
	class="text-green-600 hover:text-green-700 transition-colors"
	aria-label="Start validation"
	>
	<CheckCircle2 class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{:else if canRevalidate}
	<UiTooltip content="Revalidate">
	{#snippet children({ props })}
	<button
	{...props}
	onclick={() => onValidate?.()}
	class="text-blue-600 hover:text-blue-700 transition-colors"
	aria-label="Revalidate"
	>
	<RotateCw class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{/if}
	</div>
	
	<!-- Validation Thread Link -->
	{#if liveExecution.validationThreadUrl}
	<button
	onclick={() => openInBrowser(liveExecution.validationThreadUrl!)}
	class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
	>
	<span>Thread</span>
	<ExternalLink class="w-3 h-3" />
	</button>
	{/if}
	{:else}
	<span class="text-xs text-muted-foreground">—</span>
	{/if}
	</div>

	<!-- Combined: Changes & Commit -->
	<div class="flex flex-col gap-1 @max-md/table:hidden">
		<!-- Changes Stats -->
		<div class="flex items-center gap-2 text-xs">
			<button
				onclick={() => onReviewChanges?.()}
				disabled={!onReviewChanges}
				class="flex items-center gap-2 hover:text-blue-600 transition-colors cursor-pointer disabled:cursor-default disabled:hover:text-current"
			>
				{#if fileCount > 0}
					<UiTooltip content={`Click to view ${fileCount} file${fileCount !== 1 ? 's' : ''} changed`}>
						{#snippet children({ props })}
							<div {...props} class="flex items-center gap-1">
								<FileText class="w-3.5 h-3.5 text-muted-foreground" />
								<span class="text-yellow-600">{fileCount}</span>
							</div>
						{/snippet}
					</UiTooltip>
					<span class="text-green-600">+{additions}</span>
					<span class="text-red-600">-{deletions}</span>
				{:else}
					<span class="text-muted-foreground">No changes</span>
				{/if}
			</button>
			
			<!-- Push action -->
			{#if canPush}
				<UiTooltip content="Push to remote">
					{#snippet children({ props })}
						<button
							{...props}
							onclick={() => onPush?.()}
							class="text-purple-600 hover:text-purple-700 transition-colors"
							aria-label="Push to remote"
						>
							<Upload class="w-4 h-4" />
						</button>
					{/snippet}
				</UiTooltip>
			{/if}
		</div>
		
		<!-- Commit SHA (if exists) -->
		{#if liveExecution.commitSha && commitIcon}
			<div class="flex items-center gap-1">
				<UiTooltip content={`Commit: ${liveExecution.commitSha.slice(0, 7)}`}>
					{#snippet children({ props })}
						{@const Icon = commitIcon.Icon}
						<div {...props}>
							<Icon class={`w-4 h-4 ${commitIcon.class}`} />
						</div>
					{/snippet}
				</UiTooltip>
				<span class="text-xs font-mono text-muted-foreground">{liveExecution.commitSha.slice(0, 7)}</span>
			</div>
		{/if}
	</div>

	<!-- CI Status -->
	<div class="flex items-center gap-2 @max-md/table:hidden">
		{#if liveExecution.ciStatus}
			<CiStatusBadge 
				ciStatus={liveExecution.ciStatus} 
				ciUrl={liveExecution.ciUrl}
				onRefresh={onRefreshCi}
			/>
		{:else if liveExecution.commitStatus === 'committed' && onRefreshCi}
			<UiTooltip content="Check CI status">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={onRefreshCi}
						class="flex items-center gap-1.5 text-muted-foreground hover:text-foreground transition-colors"
					>
						<RefreshCw class="w-3.5 h-3.5" />
						<span class="text-xs">Check CI</span>
					</button>
				{/snippet}
			</UiTooltip>
		{:else}
			<span class="text-xs text-muted-foreground">—</span>
		{/if}
	</div>

	<!-- Actions -->
	<div class="flex items-center gap-1 justify-end">
		<div class="hidden @md/table:flex items-center gap-1">
			<UiTooltip content="Open in editor">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={handleOpenInEditor}
						class="text-blue-600 hover:text-blue-700 transition-colors"
						aria-label="Open in editor"
					>
						<Code class="w-4 h-4" />
					</button>
				{/snippet}
			</UiTooltip>
			
			<UiTooltip content="Copy worktree path">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={handleCopyPath}
						class="text-gray-600 hover:text-gray-700 transition-colors"
						aria-label="Copy worktree path"
					>
						<Copy class="w-4 h-4" />
					</button>
				{/snippet}
			</UiTooltip>
			
			<UiTooltip content="Delete execution">
				{#snippet children({ props })}
					<button
						{...props}
						onclick={onDelete}
						class="text-red-600 hover:text-red-700 transition-colors"
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
					class="min-w-[180px] max-w-[250px] bg-card text-card-foreground rounded-lg shadow-xl border border-border/20 p-1"
					align="end"
					sideOffset={4}
					strategy="fixed"
				>
					<!-- Only show execution/validation actions in dropdown on small screens where columns are hidden -->
					<div class="@md/table:hidden">
						{#if canStart}
							<DropdownMenu.Item
								onSelect={() => onStart?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<PlayCircle class="w-4 h-4" />
								Start execution
							</DropdownMenu.Item>
						{/if}
						
						{#if canRestart}
							<DropdownMenu.Item
								onSelect={() => onResume?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<RotateCw class="w-4 h-4" />
								Restart execution
							</DropdownMenu.Item>
						{/if}
						
						{#if canStop}
							<DropdownMenu.Item
								onSelect={() => onStop?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<Square class="w-4 h-4" />
								Stop execution
							</DropdownMenu.Item>
						{/if}
						
						{#if canStopValidation}
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
								<CheckCircle2 class="w-4 h-4" />
								Start validation
							</DropdownMenu.Item>
						{/if}
						
						{#if canRevalidate}
							<DropdownMenu.Item
								onSelect={() => onValidate?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<RotateCw class="w-4 h-4" />
								Revalidate
							</DropdownMenu.Item>
						{/if}
						
						{#if canStart || canRestart || canStop || canStopValidation || canValidate || canRevalidate}
							<DropdownMenu.Separator class="h-px bg-border/40 my-1" />
						{/if}
					</div>
					
					<DropdownMenu.Item
						onSelect={handleOpenInEditor}
						class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
					>
						<Code class="w-4 h-4" />
						Open in editor
					</DropdownMenu.Item>
					
					<DropdownMenu.Item
						onSelect={handleCopyPath}
						class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
					>
						<Copy class="w-4 h-4" />
						Copy worktree path
					</DropdownMenu.Item>
					
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
