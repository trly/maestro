import { GitHubProvider } from './github';
import type { RepositoryProvider } from './types';

export async function getConfiguredProviders(): Promise<RepositoryProvider[]> {
	const providers: RepositoryProvider[] = [];
	
	const github = new GitHubProvider();
	await github.initialize();
	if (github.isConfigured()) {
		providers.push(github);
	}

	return providers;
}

export * from './types';
export { GitHubProvider };
