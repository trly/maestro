import * as ipc from './ipc'
import type { Repository, PromptSet, PromptRevision, Execution } from './types'

export const tauriApi = {
	repositories: {
		create: async (provider: string, providerId: string, name?: string) => {
			const repo = await ipc.createRepository(provider, providerId)
			if (name && name !== repo.name) {
				await ipc.updateRepositoryName(repo.id, name)
				return { ...repo, name }
			}
			return repo
		},

		get: async (id: string) => {
			const repo = await ipc.getRepository(id)
			if (!repo) {
				throw new Error('Repository not found')
			}
			return repo
		},

		find: async (provider: string, providerId: string) => {
			const repo = await ipc.findRepository(provider, providerId)
			if (!repo) {
				throw new Error('Repository not found')
			}
			return repo
		}
	},

	promptSets: {
		getAll: () => ipc.getAllPromptSets(),

		create: (name: string, repositoryIds: string[], validationPrompt?: string | null) =>
			ipc.createPromptSet(name, repositoryIds, validationPrompt),

		get: async (id: string) => {
			const promptSet = await ipc.getPromptSet(id)
			if (!promptSet) {
				throw new Error('Prompt set not found')
			}
			return promptSet
		},

		update: async (id: string, updates: { validationPrompt?: string | null; repositoryIds?: string[] }) => {
			if ('validationPrompt' in updates) {
				await ipc.updatePromptSetValidation(id, updates.validationPrompt ?? null)
			}
			if ('repositoryIds' in updates && updates.repositoryIds) {
				await ipc.updatePromptSetRepositories(id, updates.repositoryIds)
			}
			return tauriApi.promptSets.get(id)
		},

		updateAutoValidate: (id: string, autoValidate: boolean) => ipc.updatePromptSetAutoValidate(id, autoValidate),

		getRevisions: (id: string) => ipc.getPromptSetRevisions(id),

		getExecutions: (id: string) => ipc.getExecutionsByPromptSet(id),

		delete: async (id: string) => {
			const success = await ipc.deletePromptSet(id)
			return { success }
		}
	},

	revisions: {
		create: (promptsetId: string, promptText: string, parentRevisionId: string | null = null) =>
			ipc.createPromptRevision(promptsetId, promptText, parentRevisionId),

		get: async (id: string) => {
			const revision = await ipc.getPromptRevision(id)
			if (!revision) {
				throw new Error('Revision not found')
			}
			return revision
		},

		getExecutions: (id: string) => ipc.getExecutionsByRevision(id),

		execute: async (id: string, repositoryIds?: string[]) => {
			const revision = await tauriApi.revisions.get(id)
			return await ipc.executePromptSet(revision.promptsetId, id, repositoryIds)
		},

		delete: async (id: string) => {
			const success = await ipc.deletePromptRevision(id)
			return { success }
		},

		stopAll: async (id: string) => {
			const stopped = await ipc.stopAllExecutions(id)
			return { stopped, message: `Stopped ${stopped} execution(s)` }
		},

		stopAllValidations: async (id: string) => {
			const stopped = await ipc.stopAllValidations(id)
			return { stopped, message: `Stopped ${stopped} validation(s)` }
		}
	},

	executions: {
		get: async (id: string) => {
			const execution = await ipc.getExecution(id)
			if (!execution) {
				throw new Error('Execution not found')
			}
			return execution
		},

		delete: async (id: string) => {
			const success = await ipc.deleteExecution(id)
			return { success }
		},

		validate: async (id: string) => {
			await ipc.validateExecution(id)
		},

		backfillStats: async (id: string) => {
			// Backfill stats not implemented yet - return execution as-is
			return tauriApi.executions.get(id)
		},

		stop: async (id: string) => {
			const stopped = await ipc.stopExecution(id)
			return { stopped }
		},

		stopValidation: async (id: string) => {
			const stopped = await ipc.stopValidation(id)
			return { stopped }
		},

		resume: async (id: string) => {
			await ipc.resumeExecution(id)
		},

		getModifiedFiles: async (id: string) => {
			return await ipc.getExecutionModifiedFiles(id)
		},

		getFileDiff: async (id: string, file: string) => {
			return await ipc.getExecutionFileDiff(id, file)
		},

		commit: async (id: string, files?: string[]) => {
			await ipc.commitChanges(id, files)
		}
	}
}
