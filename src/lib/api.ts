import type { Repository, PromptSet, PromptRevision, Execution } from './types';

const API_BASE = '/api';

async function fetchJSON<T>(url: string, options?: RequestInit): Promise<T> {
	const response = await fetch(`${API_BASE}${url}`, {
		...options,
		headers: {
			'Content-Type': 'application/json',
			...options?.headers
		}
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: response.statusText }));
		throw new Error(error.error || 'Request failed');
	}

	return response.json();
}

export const api = {
	repositories: {
		create: (provider: string, providerId: string, name?: string) =>
			fetchJSON<Repository>('/repositories', {
				method: 'POST',
				body: JSON.stringify({ provider, providerId, name })
			}),

		get: (id: string) =>
			fetchJSON<Repository>(`/repositories?id=${id}`),

		find: (provider: string, providerId: string) =>
			fetchJSON<Repository>(`/repositories?provider=${provider}&providerId=${encodeURIComponent(providerId)}`)
	},

	promptSets: {
		getAll: () =>
			fetchJSON<PromptSet[]>('/promptsets'),

		create: (name: string, repositoryIds: string[], validationPrompt?: string | null) =>
			fetchJSON<PromptSet>('/promptsets', {
				method: 'POST',
				body: JSON.stringify({ name, repositoryIds, validationPrompt })
			}),

		get: (id: string) =>
			fetchJSON<PromptSet>(`/promptsets/${id}`),

		update: (id: string, updates: { validationPrompt?: string | null }) =>
			fetchJSON<PromptSet>(`/promptsets/${id}`, {
				method: 'PATCH',
				body: JSON.stringify(updates)
			}),

		getRevisions: (id: string) =>
			fetchJSON<PromptRevision[]>(`/promptsets/${id}/revisions`),

		getExecutions: (id: string) =>
			fetchJSON<Execution[]>(`/promptsets/${id}/executions`),

		delete: (id: string) =>
			fetchJSON<{ success: boolean }>(`/promptsets/${id}`, {
				method: 'DELETE'
			})
	},

	revisions: {
		create: (promptsetId: string, promptText: string, parentRevisionId: string | null = null) =>
			fetchJSON<PromptRevision>('/revisions', {
				method: 'POST',
				body: JSON.stringify({ promptsetId, promptText, parentRevisionId })
			}),

		get: (id: string) =>
			fetchJSON<PromptRevision>(`/revisions/${id}`),

		getExecutions: (id: string) =>
			fetchJSON<Execution[]>(`/revisions/${id}/executions`),

		execute: (id: string) =>
			fetchJSON<{ executionIds: string[] }>(`/revisions/${id}/execute`, {
				method: 'POST'
			})
	},

	executions: {
		create: (promptsetId: string, revisionId: string, repositoryId: string) =>
			fetchJSON<Execution>('/executions', {
				method: 'POST',
				body: JSON.stringify({ promptsetId, revisionId, repositoryId })
			}),

		get: (id: string) =>
			fetchJSON<Execution>(`/executions/${id}`),

		update: (id: string, updates: Partial<Pick<Execution, 'status' | 'threadUrl' | 'completedAt'>>) =>
			fetchJSON<Execution>(`/executions/${id}`, {
				method: 'PATCH',
				body: JSON.stringify(updates)
			}),

		delete: (id: string) =>
			fetchJSON<{ success: boolean }>(`/executions/${id}`, {
				method: 'DELETE'
			}),

		validate: (id: string) =>
			fetchJSON<{ success: boolean; message: string }>(`/executions/${id}/validate`, {
				method: 'POST'
			}),

		backfillStats: (id: string) =>
			fetchJSON<Execution>(`/executions/${id}/backfill-stats`, {
				method: 'POST'
			})
	}
};
