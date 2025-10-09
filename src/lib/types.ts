export type RepositoryProvider = 'github' | 'gitlab';

export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed';

export type ValidationStatus = 'pending' | 'running' | 'passed' | 'failed' | null;

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
	createdAt: number;
}

export interface PromptRevision {
	id: string;
	promptsetId: string;
	promptText: string;
	parentRevisionId: string | null;
	createdAt: number;
}

export type PromptStatus = 'passed' | 'failed' | null;

export interface Execution {
	id: string;
	promptsetId: string;
	revisionId: string;
	repositoryId: string;
	sessionId: string | null;
	threadUrl: string | null;
	status: ExecutionStatus;
	promptStatus: PromptStatus;
	promptResult: string | null;
	validationStatus: ValidationStatus;
	validationThreadUrl: string | null;
	validationResult: string | null;
	filesAdded: number;
	filesRemoved: number;
	filesModified: number;
	linesAdded: number;
	linesRemoved: number;
	createdAt: number;
	completedAt: number | null;
}
