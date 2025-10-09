export interface Repository {
	provider: 'github' | 'gitlab' | 'bitbucket';
	fullName: string;
	name: string;
	owner: string;
	url: string;
	description?: string;
}

export interface RepositoryProvider {
	name: string;
	isConfigured(): boolean;
	searchRepositories(query: string): Promise<Repository[]>;
	getUserRepositories(): Promise<Repository[]>;
}
