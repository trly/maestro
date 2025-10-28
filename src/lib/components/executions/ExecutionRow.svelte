<script lang="ts">
	import type { Execution, Repository } from "$lib/types"
	import { intersectOnce } from "$lib/actions/intersect"
	import {
		getExecutionStatusConfig,
		getValidationStatusConfig,
		getCommitStatusConfig,
	} from "$lib/utils/statusConfig"
	import UiTooltip from "$lib/components/ui/UiTooltip.svelte"
	import IconButton from "$lib/components/ui/IconButton.svelte"
	import CiStatusBadge from "$lib/components/ui/CiStatusBadge.svelte"
	import {
		CirclePlay,
		RotateCw,
		Square,
		CircleCheck,
		Code,
		Copy,
		Trash2,
		ExternalLink,
		FileText,
		LoaderCircle,
		Upload,
		X,
	} from "lucide-svelte"
	import { openInBrowser } from "$lib/utils/browser"
	import { openInEditor, copyWorktreePath } from "$lib/utils/worktree"

	const props = $props<{
		id: string
		executionsById: Map<string, Execution>
		repositories: Map<string, Repository>
		selected: boolean
		pushing: boolean
		refreshingCi: boolean
		loadingStats: boolean
		hasValidationPrompt?: boolean
		onToggleSelected: () => void
		onLoadStats: () => void
		onLoadCi: () => void
		onStart: () => void
		onStop: () => void
		onRestart: () => void
		onValidate: () => void
		onStopValidation: () => void
		onDelete: () => void
		onReviewChanges: () => void
		onPush: () => void
		onRefreshCi: () => void
	}>()

	let execution = $derived(props.executionsById.get(props.id)!)
	let repoName = $derived(
		props.repositories.get(execution.repositoryId)?.providerId || execution.repositoryId
	)

	let executionIcon = $derived(getExecutionStatusConfig(execution.status))
	let validationIcon = $derived(getValidationStatusConfig(execution.validationStatus ?? null))
	let commitIcon = $derived(getCommitStatusConfig(execution.commitStatus))

	let visible = $state(false)
	function onVisible() {
		if (!visible) {
			visible = true
			props.onLoadStats()
			props.onLoadCi()
		}
	}

	let canStart = $derived(
		execution.status === "pending" && !execution.sessionId && !execution.threadUrl
	)
	let canStop = $derived(execution.status === "running")
	let canRestart = $derived(
		execution.status === "completed" ||
			execution.status === "failed" ||
			execution.status === "cancelled"
	)
	let canValidate = $derived(
		props.hasValidationPrompt &&
			(execution.status === "completed" || execution.status === "cancelled") &&
			!execution.validationStatus
	)
	let canRevalidate = $derived(
		props.hasValidationPrompt &&
			execution.validationStatus &&
			execution.validationStatus !== "running"
	)
	let canStopValidation = $derived(execution.validationStatus === "running")
	let canPush = $derived(execution.commitStatus === "committed" && execution.commitSha)

	let fileCount = $derived(
		(execution.filesAdded || 0) + (execution.filesRemoved || 0) + (execution.filesModified || 0)
	)
	let additions = $derived(execution.linesAdded || 0)
	let deletions = $derived(execution.linesRemoved || 0)

	let rowClass = $derived(props.selected ? "bg-primary/10 border-l-4 border-l-primary" : "")

	async function handleOpenInEditor() {
		try {
			await openInEditor(execution)
		} catch (error) {
			console.error("Failed to open in editor:", error)
		}
	}

	async function handleCopyPath() {
		try {
			await copyWorktreePath(execution)
		} catch (error) {
			console.error("Failed to copy worktree path:", error)
		}
	}
</script>

<div
	use:intersectOnce={{ rootMargin: "200px", onEnter: onVisible }}
	class="grid gap-3 px-4 py-2.5 border-b border-border/10 bg-card hover:bg-muted/30 transition-all group items-center {rowClass}
	       [grid-template-columns:auto_minmax(0,_2fr)_minmax(0,_1fr)_minmax(0,_1fr)_minmax(0,_1.5fr)_minmax(0,_1fr)_minmax(0,_1fr)]"
>
	<!-- Checkbox -->
	<button
		type="button"
		onclick={props.onToggleSelected}
		class="flex-shrink-0 w-5 h-5 flex items-center justify-center rounded border-2 {props.selected
			? 'border-primary bg-primary'
			: 'border-muted-foreground/30 hover:border-primary/50'} transition-colors"
		aria-label={props.selected ? "Deselect" : "Select"}
	>
		{#if props.selected}
			<CircleCheck class="w-4 h-4 text-primary-foreground" />
		{/if}
	</button>

	<!-- Repository -->
	<div class="overflow-hidden min-w-0">
		<span class="text-sm font-medium text-foreground truncate block">{repoName}</span>
	</div>

	<!-- Execution Status -->
	<div class="flex flex-col gap-1">
		<div class="flex items-center gap-2">
			<UiTooltip content={execution.progressMessage || `Execution: ${execution.status}`}>
				{#snippet children({ props: slotProps })}
					{@const Icon = executionIcon.Icon}
					<div {...slotProps}>
						<Icon class="w-4 h-4 {executionIcon.class}" />
					</div>
				{/snippet}
			</UiTooltip>

			{#if canStart}
				<IconButton
					icon={CirclePlay}
					tooltip="Start execution"
					onclick={props.onStart}
					variant="success"
				/>
			{:else if canStop}
				<IconButton
					icon={Square}
					tooltip="Stop execution"
					onclick={props.onStop}
					variant="warning"
				/>
			{:else if canRestart}
				<IconButton
					icon={RotateCw}
					tooltip="Restart execution"
					onclick={props.onRestart}
					variant="primary"
				/>
			{/if}
		</div>

		{#if execution.threadUrl}
			<button
				onclick={() => openInBrowser(execution.threadUrl!)}
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
				<UiTooltip content={`Validation: ${execution.validationStatus}`}>
					{#snippet children({ props: slotProps })}
						{@const Icon = validationIcon.Icon}
						<div {...slotProps}>
							<Icon class="w-4 h-4 {validationIcon.class}" />
						</div>
					{/snippet}
				</UiTooltip>

				{#if canStopValidation}
					<IconButton
						icon={X}
						tooltip="Stop validation"
						onclick={props.onStopValidation}
						variant="warning"
					/>
				{:else if canValidate}
					<IconButton
						icon={CircleCheck}
						tooltip="Start validation"
						onclick={props.onValidate}
						variant="success"
					/>
				{:else if canRevalidate}
					<IconButton
						icon={RotateCw}
						tooltip="Revalidate"
						onclick={props.onValidate}
						variant="primary"
					/>
				{/if}
			</div>

			{#if execution.validationThreadUrl}
				<button
					onclick={() => openInBrowser(execution.validationThreadUrl!)}
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

	<!-- Changes & Commit -->
	<div class="flex flex-col gap-1">
		<div class="flex items-center gap-2 text-xs">
			{#if props.loadingStats}
				<div class="flex items-center gap-1.5 text-muted-foreground">
					<LoaderCircle class="w-3.5 h-3.5 animate-spin" />
					<span class="text-xs">Loading...</span>
				</div>
			{:else}
				<button
					onclick={props.onReviewChanges}
					class="flex items-center gap-2 hover:text-primary transition-colors cursor-pointer"
				>
					{#if fileCount > 0}
						<UiTooltip
							content={`Click to view ${fileCount} file${fileCount !== 1 ? "s" : ""} changed`}
						>
							{#snippet children({ props: slotProps })}
								<div {...slotProps} class="flex items-center gap-1">
									<FileText class="w-3.5 h-3.5 text-muted-foreground" />
									<span class="text-warning">{fileCount}</span>
								</div>
							{/snippet}
						</UiTooltip>
						<span class="text-success">+{additions}</span>
						<span class="text-destructive">-{deletions}</span>
					{:else}
						<span class="text-muted-foreground">No changes</span>
					{/if}
				</button>
			{/if}

			{#if canPush}
				<IconButton
					icon={Upload}
					tooltip={props.pushing ? "Pushing..." : "Push to remote"}
					onclick={props.onPush}
					variant="primary"
					disabled={props.pushing}
					loading={props.pushing}
				/>
			{/if}
		</div>

		{#if execution.commitSha && commitIcon}
			<div class="flex items-center gap-1">
				<UiTooltip content={`Commit: ${execution.commitSha.slice(0, 7)}`}>
					{#snippet children({ props: slotProps })}
						{@const Icon = commitIcon.Icon}
						<div {...slotProps}>
							<Icon class="w-4 h-4 {commitIcon.class}" />
						</div>
					{/snippet}
				</UiTooltip>
				<span class="text-xs font-mono text-muted-foreground"
					>{execution.commitSha.slice(0, 7)}</span
				>
			</div>
		{/if}
	</div>

	<!-- CI Status -->
	<div class="flex items-center gap-2">
		{#if execution.ciStatus}
			<CiStatusBadge
				ciStatus={execution.ciStatus}
				ciUrl={execution.ciUrl}
				onRefresh={props.onRefreshCi}
				isRefreshing={props.refreshingCi}
			/>
		{:else if execution.commitStatus === "committed"}
			<button
				onclick={props.onRefreshCi}
				class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
			>
				<span>Check CI</span>
			</button>
		{:else}
			<span class="text-xs text-muted-foreground">—</span>
		{/if}
	</div>

	<!-- Actions -->
	<div class="flex items-center gap-1 justify-end">
		<IconButton
			icon={Code}
			tooltip="Open in editor"
			onclick={handleOpenInEditor}
			variant="primary"
		/>
		<IconButton icon={Copy} tooltip="Copy worktree path" onclick={handleCopyPath} variant="ghost" />
		<IconButton
			icon={Trash2}
			tooltip="Delete execution"
			onclick={props.onDelete}
			variant="destructive"
		/>
	</div>
</div>
