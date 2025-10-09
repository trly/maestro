import { Store } from './src/lib/db/store';
import { executePromptSet, deleteExecution, deletePromptSet, validateExecution, backfillDiffStats } from './src/lib/executor';
import { scanMaestroBranches, syncRepositories } from './src/lib/maestroScanner';

const store = new Store();
const isDev = process.env.NODE_ENV !== 'production';

const CLONE_DIR = process.env.VITE_MAESTRO_CLONE_DIR || `${process.env.HOME}/maestro/repos`;
await Bun.spawn(["mkdir", "-p", CLONE_DIR]).exited;

const server = Bun.serve({
	port: 3000,
	async fetch(req) {
		const url = new URL(req.url);

		if (url.pathname.startsWith('/api')) {
			return handleAPI(url, req);
		}

		if (isDev) {
			return new Response('Use Vite dev server on port 5173', { status: 404 });
		}

		return serveStatic(url.pathname);
	}
});

async function handleAPI(url: URL, req: Request): Promise<Response> {
	const json = (data: any) => Response.json(data);
	const error = (message: string, status = 400) => 
		Response.json({ error: message }, { status });

	try {
		if (url.pathname === '/api/repositories' && req.method === 'POST') {
			const { provider, providerId, name } = await req.json();
			let repo = store.findRepository(provider, providerId);
			if (!repo) {
				repo = store.createRepository(provider, providerId);
				if (name) store.updateRepositoryName(repo.id, name);
			}
			return json(repo);
		}

		if (url.pathname === '/api/repositories' && req.method === 'GET') {
			const id = url.searchParams.get('id');
			const provider = url.searchParams.get('provider');
			const providerId = url.searchParams.get('providerId');
			
			if (id) {
				const repo = store.getRepository(id);
				return repo ? json(repo) : error('Not found', 404);
			}
			if (provider && providerId) {
				const repo = store.findRepository(provider, providerId);
				return repo ? json(repo) : error('Not found', 404);
			}
			return error('Missing parameters');
		}

		if (url.pathname === '/api/promptsets' && req.method === 'GET') {
			const promptSets = store.getAllPromptSets();
			return json(promptSets);
		}

		if (url.pathname === '/api/promptsets' && req.method === 'POST') {
			const { name, repositoryIds, validationPrompt } = await req.json();
			const promptSet = store.createPromptSet(name, repositoryIds, validationPrompt);
			return json(promptSet);
		}

		if (url.pathname.match(/^\/api\/promptsets\/[\w-]+$/) && req.method === 'GET') {
			const id = url.pathname.split('/')[3];
			const promptSet = store.getPromptSet(id);
			return promptSet ? json(promptSet) : error('Not found', 404);
		}

		if (url.pathname.match(/^\/api\/promptsets\/[\w-]+$/) && req.method === 'PATCH') {
			const id = url.pathname.split('/')[3];
			const promptSet = store.getPromptSet(id);
			if (!promptSet) {
				return error('Not found', 404);
			}
			const { validationPrompt } = await req.json();
			store.updatePromptSetValidation(id, validationPrompt);
			const updated = store.getPromptSet(id);
			return json(updated);
		}

		if (url.pathname.match(/^\/api\/promptsets\/[\w-]+$/) && req.method === 'DELETE') {
			const id = url.pathname.split('/')[3];
			const promptSet = store.getPromptSet(id);
			if (!promptSet) {
				return error('Not found', 404);
			}
			await deletePromptSet(id, store);
			return json({ success: true });
		}

		if (url.pathname === '/api/revisions' && req.method === 'POST') {
			const { promptsetId, promptText, parentRevisionId } = await req.json();
			const revision = await store.createPromptRevision(promptsetId, promptText, parentRevisionId);
			return json(revision);
		}

		if (url.pathname.match(/^\/api\/revisions\/[\w-]+$/) && req.method === 'GET') {
			const id = url.pathname.split('/')[3];
			const revision = store.getPromptRevision(id);
			return revision ? json(revision) : error('Not found', 404);
		}

		if (url.pathname.match(/^\/api\/promptsets\/[\w-]+\/revisions$/) && req.method === 'GET') {
			const id = url.pathname.split('/')[3];
			const revisions = store.getPromptSetRevisions(id);
			return json(revisions);
		}

		if (url.pathname === '/api/executions' && req.method === 'POST') {
			const { promptsetId, revisionId, repositoryId } = await req.json();
			const execution = store.createExecution(promptsetId, revisionId, repositoryId);
			return json(execution);
		}

		if (url.pathname.match(/^\/api\/executions\/[\w-]+$/) && req.method === 'PATCH') {
			const id = url.pathname.split('/')[3];
			const updates = await req.json();
			store.updateExecution(id, updates);
			const execution = store.getExecution(id);
			return execution ? json(execution) : error('Not found', 404);
		}

		if (url.pathname.match(/^\/api\/executions\/[\w-]+$/) && req.method === 'GET') {
			const id = url.pathname.split('/')[3];
			const execution = store.getExecution(id);
			return execution ? json(execution) : error('Not found', 404);
		}

		if (url.pathname.match(/^\/api\/revisions\/[\w-]+\/executions$/) && req.method === 'GET') {
			const revisionId = url.pathname.split('/')[3];
			const executions = store.getExecutionsByRevision(revisionId);
			return json(executions);
		}

		if (url.pathname.match(/^\/api\/promptsets\/[\w-]+\/executions$/) && req.method === 'GET') {
			const promptsetId = url.pathname.split('/')[3];
			const executions = store.getExecutionsByPromptSet(promptsetId);
			return json(executions);
		}

		if (url.pathname.match(/^\/api\/revisions\/[\w-]+\/execute$/) && req.method === 'POST') {
			const revisionId = url.pathname.split('/')[3];
			const revision = store.getPromptRevision(revisionId);
			if (!revision) {
				return error('Revision not found', 404);
			}
			const executionIds = await executePromptSet({ 
				promptsetId: revision.promptsetId, 
				revisionId, 
				store 
			});
			return json({ executionIds });
		}

		if (url.pathname.match(/^\/api\/executions\/[\w-]+$/) && req.method === 'DELETE') {
			const id = url.pathname.split('/')[3];
			const execution = store.getExecution(id);
			if (!execution) {
				return error('Not found', 404);
			}
			await deleteExecution(id, store);
			return json({ success: true });
		}

		if (url.pathname.match(/^\/api\/executions\/[\w-]+\/validate$/) && req.method === 'POST') {
			const id = url.pathname.split('/')[3];
			const execution = store.getExecution(id);
			if (!execution) {
				return error('Not found', 404);
			}
			validateExecution({ executionId: id, store }).catch((err) =>
				console.error(`Manual validation ${id} failed:`, err)
			);
			return json({ success: true, message: 'Validation started' });
		}

		if (url.pathname.match(/^\/api\/executions\/[\w-]+\/backfill-stats$/) && req.method === 'POST') {
			const id = url.pathname.split('/')[3];
			const execution = store.getExecution(id);
			if (!execution) {
				return error('Not found', 404);
			}
			await backfillDiffStats(id, store);
			const updated = store.getExecution(id);
			return json(updated);
		}

		if (url.pathname === '/api/maestro/branches' && req.method === 'GET') {
			const refresh = url.searchParams.get('refresh') === '1';
			const data = await scanMaestroBranches(store, { refresh });
			return json(data);
		}

		if (url.pathname === '/api/maestro/sync' && req.method === 'POST') {
			const result = await syncRepositories(store);
			return json(result);
		}

		return error('Not found', 404);
	} catch (err) {
		console.error('API Error:', err);
		return error('Internal server error', 500);
	}
}

async function serveStatic(pathname: string): Promise<Response> {
	const filePath = pathname === '/' ? '/index.html' : pathname;
	const file = Bun.file(`./dist${filePath}`);
	
	if (await file.exists()) {
		return new Response(file);
	}

	const indexFile = Bun.file('./dist/index.html');
	if (await indexFile.exists()) {
		return new Response(indexFile);
	}

	return new Response('Not found', { status: 404 });
}

console.log(`Server running at http://localhost:${server.port}`);
if (isDev) {
	console.log('Development mode: Run Vite dev server separately on port 5173');
}
