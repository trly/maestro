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
	import IconButton from './IconButton.svelte';
	import { openInEditor, copyWorktreePath } from '$lib/utils/worktree';
	import { executionStore } from '$lib/stores/executionBus';
	import { getExecutionStatusConfig, getValidationStatusConfig, getCommitStatusConfig } from '$lib/utils/statusConfig';

	const props: {
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
		isPushing?: boolean;
		isRefreshingCi?: boolean;
	} = $props();

	// Merge prop with live updates from event bus
	let liveExecution = $derived.by(() => {
		const updates = $executionStore
		const data = updates.get(props.execution.id);
		if (!data) return props.execution;
		return {
			...props.execution,
			...(data.sessionId && { sessionId: data.sessionId }),
			...(data.threadUrl && { threadUrl: data.threadUrl }),
			...(data.status && { status: data.status }),
			...(data.validationStatus && { validationStatus: data.validationStatus }),
			...(data.validationThreadUrl && { validationThreadUrl: data.validationThreadUrl }),
			...(data.commitStatus && { commitStatus: data.commitStatus }),
			...(data.commitSha && { commitSha: data.commitSha }),
			...(data.committedAt && { committedAt: data.committedAt }),
			...(data.ciStatus && { ciStatus: data.ciStatus }),
			...(data.ciUrl && { ciUrl: data.ciUrl }),
			...(data.progressMessage && { progressMessage: data.progressMessage })
		};
	});

	// Reactive icon/color for execution status
	let executionIcon = $derived(getExecutionStatusConfig(liveExecution.status));

	// Reactive icon/color for validation status
	let validationIcon = $derived(getValidationStatusConfig(liveExecution.validationStatus ?? null));

	// Reactive icon/color for commit status
	let commitIcon = $derived(getCommitStatusConfig(liveExecution.commitStatus));

	// Execution action states
	let canStart = $derived(
		props.onStart &&
		liveExecution.status === 'pending' &&
		!liveExecution.sessionId &&
		!liveExecution.threadUrl
	);

	let canStop = $derived(
		props.onStop &&
		liveExecution.status === 'running'
	);

	let canRestart = $derived(
		props.onResume &&
		(liveExecution.status === 'completed' || 
		 liveExecution.status === 'failed' || 
		 liveExecution.status === 'cancelled')
	);

	// Validation action states
	let canValidate = $derived(
		props.hasValidationPrompt && 
		props.onValidate && 
		(liveExecution.status === 'completed' || liveExecution.status === 'cancelled') && 
		!liveExecution.validationStatus
	);

	let canRevalidate = $derived(
		props.hasValidationPrompt &&
		props.onValidate &&
		liveExecution.validationStatus &&
		liveExecution.validationStatus !== 'running'
	);

	let canStopValidation = $derived(
		props.onStopValidation &&
		liveExecution.validationStatus === 'running'
	);

	// Other action states
	let canPush = $derived(
		props.onPush && 
		liveExecution.commitStatus === 'committed' &&
		liveExecution.commitSha
	);

	let isRunning = $derived(
		liveExecution.status === 'running' || liveExecution.validationStatus === 'running'
	);

	let rowClass = $derived.by(() => {
		if (props.selected) return 'bg-primary/10 border-l-4 border-l-primary';
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
		onclick={props.onToggleSelect}
		disabled={isRunning}
		class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2 {props.selected ? 'border-primary bg-primary' : 'border-muted-foreground/30 hover:border-primary/50'} transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
		aria-label={props.selected ? 'Deselect' : 'Select'}
	>
		{#if props.selected}
			<CheckCircle2 class="w-4 h-4 text-primary-foreground" />
		{/if}
	</button>

	<!-- Repository Name -->
	<div class="overflow-hidden min-w-0">
		<span class="text-sm font-medium text-foreground truncate block">{props.repoName}</span>
	</div>

	<!-- Execution Status -->
	<div class="flex flex-col gap-1">
	<div class="flex items-center gap-2">
	<UiTooltip content={liveExecution.progressMessage || `Execution: ${liveExecution.status}`}>
	{#snippet children({ props: slotProps })}
	{@const Icon = executionIcon.Icon}
	<div {...slotProps}>
	<Icon class={`w-4 h-4 ${executionIcon.class}`} />
	</div>
	{/snippet}
	</UiTooltip>
	
	<!-- Start/Stop/Restart execution actions -->
	{#if canStart}
	<IconButton icon={PlayCircle} tooltip="Start execution" onclick={() => props.onStart?.()} variant="success" />
	{:else if canStop}
	<IconButton icon={Square} tooltip="Stop execution" onclick={() => props.onStop?.()} variant="warning" />
	{:else if canRestart}
	<IconButton icon={RotateCw} tooltip="Restart execution" onclick={() => props.onResume?.()} variant="primary" />
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
	{#snippet children({ props: slotProps })}
	{@const Icon = validationIcon.Icon}
	<div {...slotProps}>
	<Icon class={`w-4 h-4 ${validationIcon.class}`} />
	</div>
	{/snippet}
	</UiTooltip>
	
	<!-- Validation actions -->
	{#if canStopValidation}
	<UiTooltip content="Stop validation">
	{#snippet children({ props: slotProps })}
	<button
	{...slotProps}
	onclick={() => props.onStopValidation?.()}
	class="text-warning hover:text-warning/90 transition-colors"
	aria-label="Stop validation"
	>
	<X class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{:else if canValidate}
	<UiTooltip content="Start validation">
	{#snippet children({ props: slotProps })}
	<button
	{...slotProps}
	onclick={() => props.onValidate?.()}
	class="text-success hover:text-success/90 transition-colors"
	aria-label="Start validation"
	>
	<CheckCircle2 class="w-4 h-4" />
	</button>
	{/snippet}
	</UiTooltip>
	{:else if canRevalidate}
	<UiTooltip content="Revalidate">
	{#snippet children({ props: slotProps })}
	<button
	{...slotProps}
	onclick={() => props.onValidate?.()}
	class="text-primary hover:text-primary/90 transition-colors"
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
			onclick={() => props.onReviewChanges?.()}
			disabled={!props.onReviewChanges}
			class="flex items-center gap-2 hover:text-primary transition-colors cursor-pointer disabled:cursor-default disabled:hover:text-current"
			>
				{#if (props.fileCount ?? 0) > 0}
					<UiTooltip content={`Click to view ${props.fileCount} file${props.fileCount !== 1 ? 's' : ''} changed`}>
						{#snippet children({ props: slotProps })}
							<div {...slotProps} class="flex items-center gap-1">
								<FileText class="w-3.5 h-3.5 text-muted-foreground" />
								<span class="text-warning">{props.fileCount}</span>
							</div>
						{/snippet}
					</UiTooltip>
					<span class="text-success">+{props.additions ?? 0}</span>
					<span class="text-destructive">-{props.deletions ?? 0}</span>
				{:else}
					<span class="text-muted-foreground">No changes</span>
				{/if}
			</button>
			
			<!-- Push action -->
			{#if canPush}
				<IconButton icon={Upload} tooltip={props.isPushing ? "Pushing..." : "Push to remote"} onclick={() => props.onPush?.()} variant="accent" disabled={props.isPushing} loading={props.isPushing} />
			{/if}
		</div>
		
		<!-- Commit SHA (if exists) -->
		{#if liveExecution.commitSha && commitIcon}
			<div class="flex items-center gap-1">
				<UiTooltip content={`Commit: ${liveExecution.commitSha.slice(0, 7)}`}>
					{#snippet children({ props: slotProps })}
						{@const Icon = commitIcon.Icon}
						<div {...slotProps}>
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
				onRefresh={props.onRefreshCi}
				isRefreshing={props.isRefreshingCi}
			/>
		{:else if liveExecution.commitStatus === 'committed' && props.onRefreshCi}
			<UiTooltip content="Check CI status">
				{#snippet children({ props: slotProps })}
					<button
						{...slotProps}
						onclick={props.onRefreshCi}
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
			<IconButton icon={Code} tooltip="Open in editor" onclick={handleOpenInEditor} variant="primary" />
			<IconButton icon={Copy} tooltip="Copy worktree path" onclick={handleCopyPath} variant="ghost" />
			<IconButton icon={Trash2} tooltip="Delete execution" onclick={props.onDelete} variant="destructive" />
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
								onSelect={() => props.onStart?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<PlayCircle class="w-4 h-4" />
								Start execution
							</DropdownMenu.Item>
						{/if}
						
						{#if canRestart}
							<DropdownMenu.Item
								onSelect={() => props.onResume?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<RotateCw class="w-4 h-4" />
								Restart execution
							</DropdownMenu.Item>
						{/if}
						
						{#if canStop}
							<DropdownMenu.Item
								onSelect={() => props.onStop?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<Square class="w-4 h-4" />
								Stop execution
							</DropdownMenu.Item>
						{/if}
						
						{#if canStopValidation}
							<DropdownMenu.Item
								onSelect={() => props.onStopValidation?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<X class="w-4 h-4" />
								Stop validation
							</DropdownMenu.Item>
						{/if}
						
						{#if canValidate}
							<DropdownMenu.Item
								onSelect={() => props.onValidate?.()}
								class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-accent hover:text-accent-foreground rounded transition-colors cursor-pointer"
							>
								<CheckCircle2 class="w-4 h-4" />
								Start validation
							</DropdownMenu.Item>
						{/if}
						
						{#if canRevalidate}
							<DropdownMenu.Item
								onSelect={() => props.onValidate?.()}
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
						onSelect={props.onDelete}
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
