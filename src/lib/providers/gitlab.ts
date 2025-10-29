import type { Repository, RepositoryProvider } from "./types"
import { tokenStore } from "$lib/tokenStore"
import { logger } from "$lib/logger"

interface GitLabProject {
	id: number
	path_with_namespace: string
	name: string
	namespace: {
		path: string
	}
	web_url: string
	description: string | null
}

export class GitLabProvider implements RepositoryProvider {
	name = "GitLab"
	private token: string | null = null
	private endpoint: string | null = null
	private initialized = false
	private initPromise: Promise<void> | null = null

	constructor() {
		// Don't auto-initialize in constructor
	}

	async initialize(): Promise<void> {
		if (this.initialized) {
			return
		}

		if (this.initPromise) {
			return this.initPromise
		}

		this.initPromise = this.initializeToken()
		await this.initPromise
	}

	private async initializeToken(): Promise<void> {
		try {
			const tokens = await tokenStore.getAllTokens()
			if (tokens.gitlabToken) {
				this.token = tokens.gitlabToken
				this.endpoint = (tokens.gitlabInstanceUrl || "https://gitlab.com").replace(/\/$/, "")
			}
		} finally {
			this.initialized = true
		}
	}

	isConfigured(): boolean {
		return this.token !== null
	}

	private async gitlabFetch(path: string): Promise<Response> {
		if (!this.token || !this.endpoint) {
			throw new Error("GitLab token not configured")
		}

		const url = `${this.endpoint}/api/v4${path}`
		return fetch(url, {
			headers: {
				"PRIVATE-TOKEN": this.token,
			},
		})
	}

	async searchRepositories(query: string): Promise<Repository[]> {
		await this.initialize()

		if (!this.token) {
			throw new Error("GitLab token not configured")
		}

		if (!query.trim()) {
			return this.getUserRepositories()
		}

		try {
			const response = await this.gitlabFetch(
				`/projects?search=${encodeURIComponent(query)}&membership=true&per_page=20&order_by=updated_at`
			)

			if (!response.ok) {
				const errorText = await response.text()
				logger.error(`GitLab search failed: ${response.status} ${errorText}`)
				return []
			}

			const data: GitLabProject[] = await response.json()

			return data.map((project) => ({
				provider: "gitlab" as const,
				fullName: project.path_with_namespace,
				name: project.name,
				owner: project.namespace.path,
				url: project.web_url,
				description: project.description || undefined,
			}))
		} catch (error) {
			logger.error(`GitLab repository search failed: ${error}`)
			return []
		}
	}

	async getUserRepositories(): Promise<Repository[]> {
		await this.initialize()

		if (!this.token) {
			throw new Error("GitLab token not configured")
		}

		try {
			const response = await this.gitlabFetch(
				"/projects?membership=true&per_page=100&order_by=updated_at"
			)

			if (!response.ok) {
				return []
			}

			const data: GitLabProject[] = await response.json()

			return data.map((project) => ({
				provider: "gitlab" as const,
				fullName: project.path_with_namespace,
				name: project.name,
				owner: project.namespace.path,
				url: project.web_url,
				description: project.description || undefined,
			}))
		} catch (error) {
			return []
		}
	}
}
