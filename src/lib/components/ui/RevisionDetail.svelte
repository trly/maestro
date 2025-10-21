<script lang="ts">
	import { Tabs } from 'bits-ui';
	import PromptConsole from './PromptConsole.svelte';
	import ExecutionTable from './ExecutionTable.svelte';
	import AnalysisList from './AnalysisList.svelte';
	import type { PromptRevision, Execution, Repository, Analysis } from '$lib/types';

	const props: {
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
		// State props for ExecutionTable
		pushingExecutions?: Set<string>;
		refreshingCi?: Set<string>;
		analyzingExecutions?: boolean;
		analyzingValidations?: boolean;
		bulkStarting?: boolean;
		bulkRestarting?: boolean;
		bulkValidating?: boolean;
		bulkRevalidating?: boolean;
		bulkDeleting?: boolean;
	} = $props();

	let selectedTab = $state('executions');
</script>

<div class="flex-1 min-h-0 flex flex-col overflow-hidden bg-background">
	<!-- Prompt Console -->
	<PromptConsole
		revision={props.revision}
		validationPrompt={props.validationPrompt}
		autoValidate={props.autoValidate}
		onSaveValidation={props.onSaveValidation}
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
				Executions {#if props.executions.length > 0}({props.executions.length}){/if}
			</Tabs.Trigger>
			<Tabs.Trigger
				value="analyses"
				class="px-3 py-1.5 text-sm font-medium rounded-md transition-colors
					data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm
					data-[state=inactive]:text-muted-foreground data-[state=inactive]:hover:text-foreground"
			>
				Analyses {#if (props.analyses ?? []).length > 0}({(props.analyses ?? []).length}){/if}
			</Tabs.Trigger>
		</Tabs.List>
		
		<Tabs.Content value="executions" class="flex-1 flex flex-col overflow-auto @container/table">
			<ExecutionTable
				executions={props.executions}
				repositories={props.repositories}
				hasValidationPrompt={props.hasValidationPrompt}
				onDeleteExecution={props.onDeleteExecution}
				onStartExecution={props.onStartExecution}
				onValidateExecution={props.onValidateExecution}
				onStopExecution={props.onStopExecution}
				onStopValidation={props.onStopValidation}
				onResumeExecution={props.onResumeExecution}
				onReviewChanges={props.onReviewChanges}
				onPushExecution={props.onPushExecution}
				onRefreshCi={props.onRefreshCi}
				onBulkDelete={props.onBulkDelete}
				onBulkStart={props.onBulkStart}
				onBulkRestart={props.onBulkRestart}
				onBulkStartValidations={props.onBulkStartValidations}
				onBulkRevalidate={props.onBulkRevalidate}
				onExecuteAll={props.onExecuteAll}
				onStopAll={props.onStopAll}
				onStopAllValidations={props.onStopAllValidations}
				onRefreshAllCi={props.onRefreshAllCi}
				onAnalyzeExecutions={props.onAnalyzeExecutions}
				onAnalyzeValidations={props.onAnalyzeValidations}
				pushingExecutions={props.pushingExecutions ?? new Set()}
				refreshingCi={props.refreshingCi ?? new Set()}
				analyzingExecutions={props.analyzingExecutions ?? false}
				analyzingValidations={props.analyzingValidations ?? false}
				bulkStarting={props.bulkStarting ?? false}
				bulkRestarting={props.bulkRestarting ?? false}
				bulkValidating={props.bulkValidating ?? false}
				bulkRevalidating={props.bulkRevalidating ?? false}
				bulkDeleting={props.bulkDeleting ?? false}
			/>
		</Tabs.Content>
		
		<Tabs.Content value="analyses" class="flex-1 min-h-0 overflow-auto">
			<AnalysisList
				analyses={props.analyses ?? []}
				onDelete={props.onDeleteAnalysis}
				onRerun={props.onRerunAnalysis}
			/>
		</Tabs.Content>
	</Tabs.Root>
</div>