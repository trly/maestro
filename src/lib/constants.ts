import * as ipc from './ipc'

export type ConfigPaths = ipc.ConfigPaths

let configPathsCache: ConfigPaths | null = null
let cacheWasFromFallback = false

function isTauriApp(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export async function getConfigPaths(): Promise<ConfigPaths> {
	// If we have a cached value from Tauri, use it
	if (configPathsCache && !cacheWasFromFallback) {
		return configPathsCache
	}

	// If we're in Tauri, always try to get fresh paths (invalidates fallback cache)
	if (isTauriApp()) {
		try {
			configPathsCache = await ipc.getConfigPaths()
			cacheWasFromFallback = false
			return configPathsCache
		} catch (err) {
		}
	}

	// Only use cached fallback if we're still not in Tauri
	if (configPathsCache && cacheWasFromFallback) {
		return configPathsCache;
	}

	const homeDir = typeof process !== 'undefined' 
		? process.env.HOME 
		: (import.meta.env.HOME || '~');

	configPathsCache = {
		adminRepoDir: typeof process !== 'undefined' 
			? (process.env.VITE_MAESTRO_ADMIN_REPO_DIR || `${process.env.HOME}/maestro/repos`)
			: (import.meta.env.VITE_MAESTRO_ADMIN_REPO_DIR || `${homeDir}/maestro/repos`),
		worktreeDir: typeof process !== 'undefined'
			? (process.env.VITE_MAESTRO_WORKTREE_DIR || `${process.env.HOME}/maestro/executions`)
			: (import.meta.env.VITE_MAESTRO_WORKTREE_DIR || `${homeDir}/maestro/executions`),
		dbPath: 'maestro.db'
	};
	cacheWasFromFallback = true;

	return configPathsCache;
}

export const ADMIN_REPO_DIR = typeof process !== 'undefined' 
	? (process.env.VITE_MAESTRO_ADMIN_REPO_DIR || `${process.env.HOME}/maestro/repos`)
	: (import.meta.env.VITE_MAESTRO_ADMIN_REPO_DIR || `${import.meta.env.HOME || '~'}/maestro/repos`);

export const WORKTREE_DIR = typeof process !== 'undefined'
	? (process.env.VITE_MAESTRO_WORKTREE_DIR || `${process.env.HOME}/maestro/executions`)
	: (import.meta.env.VITE_MAESTRO_WORKTREE_DIR || `${import.meta.env.HOME || '~'}/maestro/executions`);
