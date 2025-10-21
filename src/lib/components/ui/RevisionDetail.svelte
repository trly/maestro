<script lang="ts">
	import { Tabs } from 'bits-ui';
	import PromptConsole from './PromptConsole.svelte';
	import ExecutionTable from './ExecutionTable.svelte';
	import AnalysisList from './AnalysisList.svelte';
	import type { PromptRevision, Execution, Repository, Analysis } from '$lib/types';

	let {
		revision,
		executions,
		analyses = [],
		repositories,
		repositoryIds = [],
		hasValidationPrompt = false,
		validationPrompt = null,
		autoValidate = false,
		onDeleteExecution,
		onStartExecution,
		onValidateExecution,
		onStopExecution,
		onStopValidation,
		onResumeExecution,
		onReviewChanges,
		onPushExecution,
		onRefreshCi,
		onBulkDelete,
		onBulkStart,
		onBulkRestart,
		onBulkStartValidations,
		onBulkRevalidate,
		onSaveValidation,
		onExecuteAll,
		onStopAll,
		onStopAllValidations,
		onRefreshAllCi,
		onAnalyzeExecutions,
		onAnalyzeValidations,
		onDeleteAnalysis,
		onRerunAnalysis
	}: {
		revision: PromptRevision;
		executions: Execution[];
		analyses?: Analysis[];
		repositories: Map<string, Repository>;
		repositoryIds?: string[];
		hasValidationPrompt?: boolean;
		validationPrompt?: string | null;
		autoValidate?: boolean;
		onDeleteExecution: (execution: Execution, repoName: string) => void;
		onStartExecution: (execution: Execution) => void;
		onValidateExecution: (execution: Execution) => void;
		onStopExecution: (execution: Execution) => void;
		onStopValidation: (execution: Execution) => void;
		onResumeExecution: (execution: Execution) => void;
		onReviewChanges: (executionId: string) => void;
		onPushExecution: (execution: Execution) => void;
		onRefreshCi: (execution: Execution) => void;
		onBulkDelete: (executions: Execution[]) => void;
		onBulkStart: (executions: Execution[]) => void;
		onBulkRestart: (executions: Execution[]) => void;
		onBulkStartValidations: (executions: Execution[]) => void;
		onBulkRevalidate: (executions: Execution[]) => void;
		onSaveValidation: (prompt: string, autoValidate: boolean) => Promise<void>;
		onExecuteAll: () => void;
		onStopAll: () => void;
		onStopAllValidations: () => void;
		onRefreshAllCi: () => void;
		onAnalyzeExecutions: () => void;
		onAnalyzeValidations: () => void;
		onDeleteAnalysis: (analysis: Analysis) => void;
		onRerunAnalysis: (analysis: Analysis) => void;
	} = $props();

	let selectedTab = $state('executions');
</script>

<div class="flex-1 min-h-0 flex flex-col overflow-hidden bg-background">
	<!-- Prompt Console -->
	<PromptConsole
		{revision}
		{validationPrompt}
		{autoValidate}
		{onSaveValidation}
	/>

	<!-- Tabs for Executions and Analysis -->
	<Tabs.Root bind:value={selectedTab} class="flex flex-col flex-1 min-h-0 overflow-hidden">
		<Tabs.List class="flex-shrink-0 flex items-center gap-1 px-4 py-2 bg-muted/5 border-b border-border/10">
			<Tabs.Trigger
				value="executions"
				class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors
					data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm
					data-[state=inactive]:text-muted-foreground data-[state=inactive]:hover:text-foreground"
			>
				Executions {#if executions.length > 0}({executions.length}){/if}
			</Tabs.Trigger>
			<Tabs.Trigger
				value="analyses"
				class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors
					data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm
					data-[state=inactive]:text-muted-foreground data-[state=inactive]:hover:text-foreground"
			>
				Analyses {#if analyses.length > 0}({analyses.length}){/if}
			</Tabs.Trigger>
		</Tabs.List>
		
		<Tabs.Content value="executions" class="flex-1 flex flex-col overflow-auto @container/table">
			<ExecutionTable
				{executions}
				{repositories}
				{hasValidationPrompt}
				{onDeleteExecution}
				{onStartExecution}
				{onValidateExecution}
				{onStopExecution}
				{onStopValidation}
				{onResumeExecution}
				{onReviewChanges}
				{onPushExecution}
				{onRefreshCi}
				{onBulkDelete}
				{onBulkStart}
				{onBulkRestart}
				{onBulkStartValidations}
				{onBulkRevalidate}
				{onExecuteAll}
				{onStopAll}
				{onStopAllValidations}
				{onRefreshAllCi}
				{onAnalyzeExecutions}
				{onAnalyzeValidations}
			/>
		</Tabs.Content>
		
		<Tabs.Content value="analyses" class="flex-1 min-h-0 overflow-auto">
			<AnalysisList
				{analyses}
				onDelete={onDeleteAnalysis}
				onRerun={onRerunAnalysis}
			/>
		</Tabs.Content>
	</Tabs.Root>
</div>