import fs from 'node:fs/promises';
import path from 'node:path';
import type { Store } from './db/store';

const CLONE_DIR = process.env.VITE_MAESTRO_CLONE_DIR || `${process.env.HOME}/maestro/repos`;

export type SyncResult = {
	deletedFromDisk: string[];
	deletedFromDb: string[];
	errors: string[];
};

export type ScanResult = {
	repo: {
		provider: 'github';
		providerId: string;
		id?: string;
		name?: string | null;
		existsInDb: boolean;
		existsOnDisk: boolean;
		path: string;
	};
	branches: Array<{
		name: string;
		ids?: { promptsetId: string; revisionId: string; executionId: string };
		promptset?: { id: string; name: string };
		revision?: { id: string; createdAt: number };
		execution?: {
			id: string;
			status: string;
			threadUrl: string | null;
			validationStatus: string | null;
			validationThreadUrl: string | null;
			createdAt: number;
			completedAt: number | null;
		};
	}>;
};

let cache: { ts: number; data: ScanResult[] } | null = null;
const TTL_MS = 30_000;

async function listMaestroBranches(repoPath: string): Promise<string[]> {
	const proc = Bun.spawn(['git', 'for-each-ref', '--format=%(refname:short)', 'refs/heads/maestro'], {
		cwd: repoPath,
		stdout: 'pipe',
		stderr: 'pipe',
	});
	await proc.exited;
	if (proc.exitCode !== 0) return [];
	const out = await new Response(proc.stdout).text();
	return out.split('\n').map(s => s.trim()).filter(Boolean);
}

function parseBranch(name: string) {
	const parts = name.split('/');
	if (parts.length === 4 && parts[0] === 'maestro') {
		const [_, promptsetIdShort, revisionIdShort, executionIdShort] = parts;
		return { promptsetId: promptsetIdShort, revisionId: revisionIdShort, executionId: executionIdShort };
	}
	return null;
}

async function safeReadDir(dir: string): Promise<string[]> {
	try {
		const entries = await fs.readdir(dir, { withFileTypes: true });
		return entries.filter(e => e.isDirectory()).map(e => e.name);
	} catch {
		return [];
	}
}

export async function scanMaestroBranches(store: Store, opts?: { refresh?: boolean }): Promise<ScanResult[]> {
	const now = Date.now();
	if (!opts?.refresh && cache && now - cache.ts < TTL_MS) {
		return cache.data;
	}

	const results: ScanResult[] = [];
	const owners = await safeReadDir(CLONE_DIR);

	for (const owner of owners) {
		const ownerPath = path.join(CLONE_DIR, owner);
		const repos = await safeReadDir(ownerPath);

		for (const repo of repos) {
			const repoPath = path.join(ownerPath, repo);
			const providerId = `${owner}/${repo}`;
			const repoInDb = store.findRepository('github', providerId);
			const branchNames = await listMaestroBranches(repoPath);

			const branches = await Promise.all(
				branchNames.map(async (name) => {
					const ids = parseBranch(name) || undefined;
					const promptset = ids?.promptsetId ? store.findPromptSetByPrefix(ids.promptsetId) : null;
					const revision = ids?.revisionId ? store.findPromptRevisionByPrefix(ids.revisionId) : null;
					const execution = ids?.executionId ? store.findExecutionByPrefix(ids.executionId) : null;

					return {
						name,
						ids,
						promptset: promptset ? { id: promptset.id, name: promptset.name } : undefined,
						revision: revision ? { id: revision.id, createdAt: revision.createdAt } : undefined,
						execution: execution
							? {
									id: execution.id,
									status: execution.status,
									threadUrl: execution.threadUrl,
									validationStatus: execution.validationStatus,
									validationThreadUrl: execution.validationThreadUrl,
									createdAt: execution.createdAt,
									completedAt: execution.completedAt,
								}
							: undefined,
					};
				}),
			);

			results.push({
				repo: {
					provider: 'github',
					providerId,
					id: repoInDb?.id,
					name: repoInDb?.name ?? null,
					existsInDb: !!repoInDb,
					existsOnDisk: true,
					path: repoPath,
				},
				branches,
			});
		}
	}

	const all = store.getAllRepositories();
	const onDiskSet = new Set(results.map((r) => r.repo.providerId));
	for (const r of all) {
		if (r.provider === 'github' && !onDiskSet.has(r.providerId)) {
			results.push({
				repo: {
					provider: 'github',
					providerId: r.providerId,
					id: r.id,
					name: r.name,
					existsInDb: true,
					existsOnDisk: false,
					path: path.join(CLONE_DIR, r.providerId),
				},
				branches: [],
			});
		}
	}

	cache = { ts: now, data: results };
	return results;
}

async function deleteDirectory(dirPath: string): Promise<void> {
	await fs.rm(dirPath, { recursive: true, force: true });
}

export async function syncRepositories(store: Store): Promise<SyncResult> {
	const result: SyncResult = {
		deletedFromDisk: [],
		deletedFromDb: [],
		errors: [],
	};

	const onDiskRepos = new Map<string, string>();
	const owners = await safeReadDir(CLONE_DIR);

	for (const owner of owners) {
		const ownerPath = path.join(CLONE_DIR, owner);
		const repos = await safeReadDir(ownerPath);
		for (const repo of repos) {
			const repoPath = path.join(ownerPath, repo);
			const providerId = `${owner}/${repo}`;
			onDiskRepos.set(providerId, repoPath);
		}
	}

	const dbRepos = store.getAllRepositories();
	const dbRepoIds = new Set(dbRepos.filter(r => r.provider === 'github').map(r => r.providerId));

	for (const [providerId, repoPath] of onDiskRepos.entries()) {
		if (!dbRepoIds.has(providerId)) {
			try {
				await deleteDirectory(repoPath);
				result.deletedFromDisk.push(providerId);
			} catch (err) {
				result.errors.push(`Failed to delete ${providerId} from disk: ${err}`);
			}
		}
	}

	const promptSets = store.getAllPromptSets();
	const usedRepoIds = new Set(promptSets.flatMap(ps => ps.repositoryIds));

	for (const repo of dbRepos) {
		if (repo.provider !== 'github') continue;
		
		const isUsed = usedRepoIds.has(repo.id);
		
		if (!onDiskRepos.has(repo.providerId) && !isUsed) {
			try {
				store.deleteRepository(repo.id);
				result.deletedFromDb.push(repo.providerId);
			} catch (err) {
				result.errors.push(`Failed to delete ${repo.providerId} from DB: ${err}`);
			}
		}
	}

	cache = null;

	return result;
}
