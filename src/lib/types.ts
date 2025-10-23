export type RepositoryProvider = 'github' | 'gitlab';

export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export type ValidationStatus = 'pending' | 'running' | 'passed' | 'failed' | 'cancelled';

export type FileStatus = 'added' | 'modified' | 'deleted' | 'renamed';

export interface Repository {
	id: string;
	provider: RepositoryProvider;
	providerId: string;
	name: string | null;
	lastSyncedAt: number | null;
	createdAt: number;
}

export interface PromptSet {
	id: string;
	name: string;
	repositoryIds: string[];
	validationPrompt: string | null;
	autoValidate: boolean;
	createdAt: number;
	stats?: {
		totalExecutions: number;
		totalCompletions: number;
		totalValidations: number;
		totalRevisions: number;
	};
}

export interface PromptRevision {
	id: string;
	promptsetId: string;
	promptText: string;
	parentRevisionId: string | null;
	createdAt: number;
	executionStats?: {
		total: number;
		completed: number;
		validationPassed: number;
	};
}

export type PromptStatus = 'passed' | 'failed' | null;

export type CommitStatus = 'none' | 'uncommitted' | 'committed';

export type CiStatus = 'pending' | 'passed' | 'failed' | 'skipped' | 'not_configured';

export interface Execution {
	id: string;
	promptsetId: string;
	revisionId: string;
	repositoryId: string;
	sessionId: string | null;
	threadUrl: string | null;
	status: ExecutionStatus;
	promptStatus: PromptStatus | null;
	promptResult: string | null;
	validationStatus: ValidationStatus | null;
	validationThreadUrl: string | null;
	validationResult: string | null;
	filesAdded: number;
	filesRemoved: number;
	filesModified: number;
	linesAdded: number;
	linesRemoved: number;
	commitStatus: CommitStatus;
	commitSha: string | null;
	committedAt: number | null;
	parentSha: string | null;
	branch: string | null;
	ciStatus: CiStatus | null;
	ciCheckedAt: number | null;
	ciUrl: string | null;
	createdAt: number;
	completedAt: number | null;
	progressMessage?: string;
}

export type AnalysisType = 'execution' | 'validation';

export type AnalysisStatus = 'pending' | 'running' | 'completed' | 'failed';

export interface Analysis {
	id: string;
	revisionId: string;
	type: AnalysisType;
	status: AnalysisStatus;
	analysisPrompt: string;
	analysisResult: string | null;
	ampThreadUrl: string | null;
	ampSessionId: string | null;
	errorMessage: string | null;
	executionCount: number;
	createdAt: number;
	updatedAt: number;
	completedAt: number | null;
}
