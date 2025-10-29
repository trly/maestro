import { GitHubProvider } from "./github"
import { GitLabProvider } from "./gitlab"
import type { RepositoryProvider } from "./types"

export async function getConfiguredProviders(): Promise<RepositoryProvider[]> {
	const providers: RepositoryProvider[] = []

	const github = new GitHubProvider()
	await github.initialize()
	if (github.isConfigured()) {
		providers.push(github)
	}

	const gitlab = new GitLabProvider()
	await gitlab.initialize()
	if (gitlab.isConfigured()) {
		providers.push(gitlab)
	}

	return providers
}

export * from "./types"
export { GitHubProvider, GitLabProvider }
