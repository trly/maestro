import { GitHubProvider } from './github';
import type { RepositoryProvider } from './types';

export function getConfiguredProviders(): RepositoryProvider[] {
	const providers: RepositoryProvider[] = [];
	
	const github = new GitHubProvider();
	if (github.isConfigured()) {
		providers.push(github);
	}

	return providers;
}

export * from './types';
export { GitHubProvider };
