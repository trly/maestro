import { invoke } from "@tauri-apps/api/core"
import type {
	Repository,
	PromptSet,
	PromptRevision,
	Execution,
	Analysis,
	AnalysisType,
} from "./types"

/**
 * Custom error class for Tauri IPC errors
 */
export class TauriIPCError extends Error {
	constructor(
		message: string,
		public readonly command: string,
		public readonly originalError?: unknown
	) {
		super(message)
		this.name = "TauriIPCError"
	}
}

/**
 * Generic wrapper for Tauri IPC commands with centralized error handling
 *
 * @param cmd - The Tauri command name
 * @param args - Optional command arguments
 * @returns Promise with the typed result
 * @throws TauriIPCError on failure
 */
export async function invokeCommand<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	try {
		return await invoke<T>(cmd, args)
	} catch (error) {
		// Convert Tauri errors to proper Error objects
		const errorMessage =
			typeof error === "string" ? error : error instanceof Error ? error.message : "Unknown error"

		throw new TauriIPCError(`Command '${cmd}' failed: ${errorMessage}`, cmd, error)
	}
}

// ============================================================================
// Repository Commands
// ============================================================================

/**
 * Create a new repository
 */
export async function createRepository(provider: string, providerId: string): Promise<Repository> {
	return invokeCommand<Repository>("create_repository", {
		provider,
		providerId,
	})
}

/**
 * Update repository name
 */
export async function updateRepositoryName(id: string, name: string): Promise<void> {
	return invokeCommand<void>("update_repository_name", { id, name })
}

/**
 * Sync repository metadata from GitHub API (e.g., default branch)
 */
export async function syncRepositoryMetadata(id: string): Promise<void> {
	return invokeCommand<void>("sync_repository_metadata", { id })
}

/**
 * Get repository by ID
 */
export async function getRepository(id: string): Promise<Repository | null> {
	return invokeCommand<Repository | null>("get_repository", { id })
}

/**
 * Find repository by provider and provider ID
 */
export async function findRepository(
	provider: string,
	providerId: string
): Promise<Repository | null> {
	return invokeCommand<Repository | null>("find_repository", {
		provider,
		providerId,
	})
}

/**
 * Get all repositories
 */
export async function getAllRepositories(): Promise<Repository[]> {
	return invokeCommand<Repository[]>("get_all_repositories")
}

/**
 * Delete repository by ID
 */
export async function deleteRepository(id: string): Promise<boolean> {
	return invokeCommand<boolean>("delete_repository", { id })
}

// ============================================================================
// PromptSet Commands
// ============================================================================

/**
 * Create a new prompt set
 */
export async function createPromptSet(
	name: string,
	repositoryIds: string[],
	validationPrompt?: string | null,
	autoValidate: boolean = false
): Promise<PromptSet> {
	return invokeCommand<PromptSet>("create_promptset", {
		name,
		repositoryIds,
		validationPrompt,
		autoValidate,
	})
}

/**
 * Get all prompt sets
 */
export async function getAllPromptSets(): Promise<PromptSet[]> {
	return invokeCommand<PromptSet[]>("get_all_promptsets")
}

/**
 * Get prompt set by ID
 */
export async function getPromptSet(id: string): Promise<PromptSet | null> {
	return invokeCommand<PromptSet | null>("get_promptset", { id })
}

/**
 * Update prompt set validation prompt
 */
export async function updatePromptSetValidation(
	id: string,
	validationPrompt: string | null
): Promise<void> {
	return invokeCommand<void>("update_promptset_validation", {
		id,
		validationPrompt,
	})
}

/**
 * Update prompt set auto-validate setting
 */
export async function updatePromptSetAutoValidate(
	id: string,
	autoValidate: boolean
): Promise<void> {
	return invokeCommand<void>("update_promptset_auto_validate", {
		id,
		autoValidate,
	})
}

/**
 * Update prompt set repositories
 */
export async function updatePromptSetRepositories(
	id: string,
	repositoryIds: string[]
): Promise<void> {
	return invokeCommand<void>("update_promptset_repositories", {
		id,
		repositoryIds,
	})
}

/**
 * Delete prompt set by ID
 */
export async function deletePromptSet(id: string): Promise<boolean> {
	return invokeCommand<boolean>("delete_promptset", { id })
}

/**
 * Get all revisions for a prompt set
 */
export async function getPromptSetRevisions(promptsetId: string): Promise<PromptRevision[]> {
	return invokeCommand<PromptRevision[]>("get_promptset_revisions", {
		promptsetId,
	})
}

/**
 * Get all executions for a prompt set
 */
export async function getExecutionsByPromptSet(promptsetId: string): Promise<Execution[]> {
	return invokeCommand<Execution[]>("get_executions_by_promptset", {
		promptsetId,
	})
}

// ============================================================================
// Prompt Revision Commands
// ============================================================================

/**
 * Create a new prompt revision
 */
export async function createPromptRevision(
	promptsetId: string,
	promptText: string,
	parentRevisionId: string | null = null
): Promise<PromptRevision> {
	return invokeCommand<PromptRevision>("create_prompt_revision", {
		promptsetId,
		promptText,
		parentRevisionId,
	})
}

/**
 * Get prompt revision by ID
 */
export async function getPromptRevision(id: string): Promise<PromptRevision | null> {
	return invokeCommand<PromptRevision | null>("get_prompt_revision", { id })
}

/**
 * Delete prompt revision by ID
 */
export async function deletePromptRevision(id: string): Promise<boolean> {
	return invokeCommand<boolean>("delete_prompt_revision", { id })
}

/**
 * Get all executions for a revision
 */
export async function getExecutionsByRevision(revisionId: string): Promise<Execution[]> {
	return invokeCommand<Execution[]>("get_executions_by_revision", {
		revisionId,
	})
}

/**
 * Prepare executions: create execution records and worktrees without starting
 */
export async function prepareExecutions(
	promptsetId: string,
	revisionId: string,
	repositoryIds?: string[]
): Promise<string[]> {
	return invokeCommand<string[]>("prepare_executions", {
		promptsetId,
		revisionId,
		repositoryIds,
	})
}

/**
 * Execute a prompt set with a specific revision
 */
export async function executePromptSet(
	promptsetId: string,
	revisionId: string,
	repositoryIds?: string[]
): Promise<string[]> {
	return invokeCommand<string[]>("execute_promptset", {
		promptsetId,
		revisionId,
		repositoryIds,
	})
}

/**
 * Stop all executions for a revision
 */
export async function stopAllExecutions(revisionId: string): Promise<number> {
	return invokeCommand<number>("stop_all_executions", { revisionId })
}

/**
 * Stop all validations for a revision
 */
export async function stopAllValidations(revisionId: string): Promise<number> {
	return invokeCommand<number>("stop_all_validations", { revisionId })
}

// ============================================================================
// Execution Commands
// ============================================================================

/**
 * Get execution by ID
 */
export async function getExecution(id: string): Promise<Execution | null> {
	return invokeCommand<Execution | null>("get_execution", { id })
}

/**
 * Find execution by ID prefix (short hash)
 */
export async function findExecutionByPrefix(idPrefix: string): Promise<Execution | null> {
	return invokeCommand<Execution | null>("find_execution_by_prefix", {
		idPrefix,
	})
}

/**
 * Create a new execution (without starting it)
 */
export async function createExecution(
	promptsetId: string,
	revisionId: string,
	repositoryId: string
): Promise<Execution> {
	return invokeCommand<Execution>("create_execution", {
		promptsetId,
		revisionId,
		repositoryId,
	})
}

/**
 * Delete execution by ID
 */
export async function deleteExecution(id: string): Promise<boolean> {
	return invokeCommand<boolean>("delete_execution", { id })
}

/**
 * Start an execution (new or restart)
 */
export async function startExecution(executionId: string): Promise<void> {
	return invokeCommand<void>("execute_prompt", { executionId })
}

/**
 * Validate an execution
 */
export async function validateExecution(executionId: string): Promise<void> {
	return invokeCommand<void>("validate_execution", { executionId })
}

/**
 * Stop a running execution
 */
export async function stopExecution(executionId: string): Promise<boolean> {
	return invokeCommand<boolean>("stop_execution", { executionId })
}

/**
 * Stop a running validation
 */
export async function stopValidation(executionId: string): Promise<boolean> {
	return invokeCommand<boolean>("stop_validation", { executionId })
}

/**
 * Resume a paused execution
 */
export async function resumeExecution(executionId: string): Promise<void> {
	return invokeCommand<void>("resume_execution", { executionId })
}

/**
 * Get modified files for an execution
 */
export async function getExecutionModifiedFiles(executionId: string): Promise<{
	files: Array<{
		status: string
		path: string
		additions?: number
		deletions?: number
	}>
	source: string
	commitSha: string | null
}> {
	return invokeCommand("get_execution_modified_files", { executionId })
}

/**
 * Get file diff for an execution
 */
export async function getExecutionFileDiff(executionId: string, file: string): Promise<string> {
	return invokeCommand<string>("get_execution_file_diff", {
		executionId,
		file,
	})
}

/**
 * Commit changes for an execution
 */
export async function commitChanges(executionId: string, files?: string[]): Promise<void> {
	return invokeCommand<void>("commit_changes", {
		executionId,
		files: files || null,
	})
}

/**
 * Cleanup execution worktree
 */
export async function cleanupExecution(executionId: string): Promise<void> {
	return invokeCommand<void>("cleanup_execution", { executionId })
}

// ============================================================================
// Token Commands
// ============================================================================

export type TokenKey =
	| "amp_token"
	| "github_token"
	| "gitlab_token"
	| "gitlab_instance_url"
	| "sourcegraph_endpoint"
	| "sourcegraph_token"
	| "amp_client_id"
	| "amp_client_secret"

/**
 * Set a token in the system keyring
 */
export async function setToken(key: TokenKey, value: string): Promise<void> {
	return invokeCommand<void>("set_token", { key, value })
}

/**
 * Delete a token from the system keyring
 */
export async function deleteToken(key: TokenKey): Promise<void> {
	return invokeCommand<void>("delete_token", { key })
}

export interface AllTokens {
	ampToken: string | null
	githubToken: string | null
	gitlabToken: string | null
	gitlabInstanceUrl: string | null
	sourcegraphEndpoint: string | null
	sourcegraphToken: string | null
	ampClientId: string | null
	ampClientSecret: string | null
}

/**
 * Get all tokens from the system keyring (single keychain prompt)
 */
export async function getAllTokens(): Promise<AllTokens> {
	return invokeCommand<AllTokens>("get_all_tokens")
}

/**
 * Get all tokens masked (single keychain prompt)
 */
export async function getAllTokensMasked(): Promise<AllTokens> {
	return invokeCommand<AllTokens>("get_all_tokens_masked")
}

// ============================================================================
// Sourcegraph Commands
// ============================================================================

export interface SourcegraphRepository {
	id: string
	name: string
	description: string | null
	url: string
	language: string | null
	stars: number
	isPrivate: boolean
	isFork: boolean
	isArchived: boolean
	externalRepository: {
		serviceType: string
		serviceId: string
	}
}

export interface RepositorySearchResult {
	repositories: SourcegraphRepository[]
	totalCount: number
	hasNextPage: boolean
}

/**
 * Search Sourcegraph repositories
 */
export async function searchSourcegraphRepositories(
	query: string,
	limit?: number
): Promise<RepositorySearchResult> {
	return invokeCommand<RepositorySearchResult>("search_sourcegraph_repositories", {
		query,
		limit,
	})
}

// ============================================================================
// Config Commands
// ============================================================================

export interface ConfigPaths {
	adminRepoDir: string
	worktreeDir: string
	dbPath: string
}

/**
 * Get configuration paths
 */
export async function getConfigPaths(): Promise<ConfigPaths> {
	return invokeCommand<ConfigPaths>("get_config_paths")
}

/**
 * Open worktree in editor
 */
export async function openWorktreeInEditor(
	promptsetId: string,
	executionId: string,
	editorCommand: string
): Promise<void> {
	return invokeCommand<void>("open_worktree_in_editor", {
		promptsetId,
		executionId,
		editorCommand,
	})
}

/**
 * Open worktree in editor with terminal wrapper
 */
export async function openWorktreeWithTerminal(
	promptsetId: string,
	executionId: string,
	editorCommand: string,
	terminalCommand: string
): Promise<void> {
	return invokeCommand<void>("open_worktree_with_terminal", {
		promptsetId,
		executionId,
		editorCommand,
		terminalCommand,
	})
}

// ============================================================================
// CI Commands
// ============================================================================

/**
 * Start CI checking for an execution (spawns background polling)
 */
export async function startCiCheck(executionId: string): Promise<void> {
	return invokeCommand<void>("start_ci_check", { executionId })
}

/**
 * Refresh CI status once (no polling)
 */
export async function refreshCiStatus(executionId: string): Promise<void> {
	return invokeCommand<void>("refresh_ci_status", { executionId })
}

/**
 * Push committed changes to remote
 */
export async function pushCommit(executionId: string, force: boolean = false): Promise<void> {
	return invokeCommand<void>("push_commit", { executionId, force })
}

// ============================================================================
// Settings Commands
// ============================================================================

/**
 * Get a setting value by key
 */
export async function getSetting(key: string): Promise<string | null> {
	return invokeCommand<string | null>("get_setting", { key })
}

/**
 * Set a setting value
 */
export async function setSetting(key: string, value: string): Promise<void> {
	return invokeCommand<void>("set_setting", { key, value })
}

/**
 * Get the CI stuck threshold in minutes (how long before pending CI is marked as skipped)
 */
export async function getCiStuckThresholdMinutes(): Promise<number> {
	return invokeCommand<number>("get_ci_stuck_threshold_minutes", {})
}

/**
 * Get the maximum number of concurrent executions allowed
 */
export async function getMaxConcurrentExecutions(): Promise<number> {
	return invokeCommand<number>("get_max_concurrent_executions", {})
}

// ============================================================================
// Application Check Commands
// ============================================================================

export interface AppInfo {
	command: string
	displayName: string
	needsTerminal: boolean
}

export interface TerminalInfo {
	command: string
	displayName: string
}

/**
 * Get list of available/installed editors
 */
export async function getAvailableEditors(): Promise<AppInfo[]> {
	return invokeCommand<AppInfo[]>("get_available_editors")
}

/**
 * Get list of available/installed terminal apps
 */
export async function getAvailableTerminals(): Promise<TerminalInfo[]> {
	return invokeCommand<TerminalInfo[]>("get_available_terminals")
}

/**
 * Check if a specific app/command is installed
 */
export async function checkAppInstalled(command: string): Promise<boolean> {
	return invokeCommand<boolean>("check_app_installed", { command })
}

// ============================================================================
// Health Check Commands
// ============================================================================

export interface HealthCheckResult {
	success: boolean
	username?: string
	error?: string
}

/**
 * Check GitHub token validity and get current username
 */
export async function healthCheckGithub(): Promise<HealthCheckResult> {
	return invokeCommand<HealthCheckResult>("health_check_github")
}

/**
 * Check GitLab token validity and get current username
 */
export async function healthCheckGitlab(): Promise<HealthCheckResult> {
	return invokeCommand<HealthCheckResult>("health_check_gitlab")
}

/**
 * Check Sourcegraph token validity and get current username
 */
export async function healthCheckSourcegraph(): Promise<HealthCheckResult> {
	return invokeCommand<HealthCheckResult>("health_check_sourcegraph")
}

// ============================================================================
// Analysis Commands
// ============================================================================

/**
 * Create a new analysis for a revision
 */
export async function createAnalysis(
	revisionId: string,
	analysisType: AnalysisType,
	executionIds: string[]
): Promise<string> {
	return invokeCommand<string>("create_analysis", {
		revisionId,
		analysisType,
		executionIds,
	})
}

/**
 * Run an analysis
 */
export async function runAnalysis(analysisId: string): Promise<void> {
	return invokeCommand<void>("run_analysis", { analysisId })
}

/**
 * Get analysis by ID
 */
export async function getAnalysis(analysisId: string): Promise<Analysis | null> {
	return invokeCommand<Analysis | null>("get_analysis", { analysisId })
}

/**
 * Get all analyses for a revision (optionally filtered by type)
 */
export async function getAnalysesByRevision(
	revisionId: string,
	analysisType?: AnalysisType
): Promise<Analysis[]> {
	return invokeCommand<Analysis[]>("get_analyses_by_revision", {
		revisionId,
		analysisType,
	})
}

/**
 * Delete an analysis
 */
export async function deleteAnalysis(analysisId: string): Promise<boolean> {
	return invokeCommand<boolean>("delete_analysis", { analysisId })
}
