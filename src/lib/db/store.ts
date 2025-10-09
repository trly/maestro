import { Database } from 'bun:sqlite';
import { runMigrations } from './migrations';
import type { Repository, PromptSet, PromptRevision, Execution } from '../types';

export class Store {
	private db: Database;

	constructor(dbPath: string = 'maestro.db') {
		this.db = new Database(dbPath);
		this.db.exec('PRAGMA foreign_keys = ON;');
		runMigrations(this.db);
	}

	async hashPrompt(text: string): Promise<string> {
		const encoder = new TextEncoder();
		const data = encoder.encode(text);
		const hashBuffer = await crypto.subtle.digest('SHA-256', data);
		const hashArray = Array.from(new Uint8Array(hashBuffer));
		return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
	}

	createRepository(provider: string, providerId: string): Repository {
		const id = crypto.randomUUID();
		const now = Date.now();
		
		this.db.prepare(`
			INSERT INTO repositories (id, provider, provider_id, name, last_synced_at, created_at)
			VALUES (?, ?, ?, NULL, NULL, ?)
		`).run(id, provider, providerId, now);

		return {
			id,
			provider: provider as Repository['provider'],
			providerId,
			name: null,
			lastSyncedAt: null,
			createdAt: now
		};
	}

	updateRepositoryName(id: string, name: string): void {
		this.db.prepare(`
			UPDATE repositories SET name = ?, last_synced_at = ? WHERE id = ?
		`).run(name, Date.now(), id);
	}

	getRepository(id: string): Repository | null {
		const row = this.db.prepare('SELECT * FROM repositories WHERE id = ?').get(id) as any;
		if (!row) return null;

		return {
			id: row.id,
			provider: row.provider,
			providerId: row.provider_id,
			name: row.name,
			lastSyncedAt: row.last_synced_at,
			createdAt: row.created_at
		};
	}

	findRepository(provider: string, providerId: string): Repository | null {
		const row = this.db.prepare('SELECT * FROM repositories WHERE provider = ? AND provider_id = ?').get(provider, providerId) as any;
		if (!row) return null;

		return {
			id: row.id,
			provider: row.provider,
			providerId: row.provider_id,
			name: row.name,
			lastSyncedAt: row.last_synced_at,
			createdAt: row.created_at
		};
	}

	getAllRepositories(): Repository[] {
		const rows = this.db.prepare('SELECT * FROM repositories ORDER BY created_at DESC').all() as any[];
		
		return rows.map(row => ({
			id: row.id,
			provider: row.provider,
			providerId: row.provider_id,
			name: row.name,
			lastSyncedAt: row.last_synced_at,
			createdAt: row.created_at
		}));
	}

	createPromptSet(name: string, repositoryIds: string[], validationPrompt: string | null = null): PromptSet {
		const id = crypto.randomUUID();
		const now = Date.now();

		this.db.prepare('INSERT INTO promptsets (id, name, validation_prompt, created_at) VALUES (?, ?, ?, ?)').run(id, name, validationPrompt, now);

		const insertRepo = this.db.prepare('INSERT INTO promptset_repositories (promptset_id, repository_id) VALUES (?, ?)');
		for (const repoId of repositoryIds) {
			insertRepo.run(id, repoId);
		}

		return { id, name, repositoryIds, validationPrompt, createdAt: now };
	}

	getAllPromptSets(): PromptSet[] {
		const rows = this.db.prepare('SELECT * FROM promptsets ORDER BY created_at DESC').all() as any[];
		
		return rows.map(row => {
			const repos = this.db.prepare('SELECT repository_id FROM promptset_repositories WHERE promptset_id = ?').all(row.id) as any[];
			return {
				id: row.id,
				name: row.name,
				repositoryIds: repos.map(r => r.repository_id),
				validationPrompt: row.validation_prompt,
				createdAt: row.created_at
			};
		});
	}

	getPromptSet(id: string): PromptSet | null {
		const row = this.db.prepare('SELECT * FROM promptsets WHERE id = ?').get(id) as any;
		if (!row) return null;

		const repos = this.db.prepare('SELECT repository_id FROM promptset_repositories WHERE promptset_id = ?').all(id) as any[];
		
		return {
			id: row.id,
			name: row.name,
			repositoryIds: repos.map(r => r.repository_id),
			validationPrompt: row.validation_prompt,
			createdAt: row.created_at
		};
	}

	findPromptSetByPrefix(idPrefix: string): PromptSet | null {
		const row = this.db.prepare('SELECT * FROM promptsets WHERE id LIKE ?').get(`${idPrefix}%`) as any;
		if (!row) return null;

		const repos = this.db.prepare('SELECT repository_id FROM promptset_repositories WHERE promptset_id = ?').all(row.id) as any[];
		
		return {
			id: row.id,
			name: row.name,
			repositoryIds: repos.map(r => r.repository_id),
			validationPrompt: row.validation_prompt,
			createdAt: row.created_at
		};
	}

	updatePromptSetValidation(id: string, validationPrompt: string | null): void {
		this.db.prepare('UPDATE promptsets SET validation_prompt = ? WHERE id = ?').run(validationPrompt, id);
	}

	async createPromptRevision(promptsetId: string, promptText: string, parentRevisionId: string | null = null): Promise<PromptRevision> {
		const id = await this.hashPrompt(promptText);
		const now = Date.now();

		const existing = this.db.prepare('SELECT * FROM prompt_revisions WHERE id = ?').get(id);
		if (existing) {
			return existing as PromptRevision;
		}

		this.db.prepare(`
			INSERT INTO prompt_revisions (id, promptset_id, prompt_text, parent_revision_id, created_at)
			VALUES (?, ?, ?, ?, ?)
		`).run(id, promptsetId, promptText, parentRevisionId, now);

		return { id, promptsetId, promptText, parentRevisionId, createdAt: now };
	}

	getPromptRevision(id: string): PromptRevision | null {
		const row = this.db.prepare('SELECT * FROM prompt_revisions WHERE id = ?').get(id) as any;
		if (!row) return null;

		return {
			id: row.id,
			promptsetId: row.promptset_id,
			promptText: row.prompt_text,
			parentRevisionId: row.parent_revision_id,
			createdAt: row.created_at
		};
	}

	findPromptRevisionByPrefix(idPrefix: string): PromptRevision | null {
		const row = this.db.prepare('SELECT * FROM prompt_revisions WHERE id LIKE ?').get(`${idPrefix}%`) as any;
		if (!row) return null;

		return {
			id: row.id,
			promptsetId: row.promptset_id,
			promptText: row.prompt_text,
			parentRevisionId: row.parent_revision_id,
			createdAt: row.created_at
		};
	}

	getPromptSetRevisions(promptsetId: string): PromptRevision[] {
		const rows = this.db.prepare('SELECT * FROM prompt_revisions WHERE promptset_id = ? ORDER BY created_at DESC').all(promptsetId) as any[];
		
		return rows.map(row => ({
			id: row.id,
			promptsetId: row.promptset_id,
			promptText: row.prompt_text,
			parentRevisionId: row.parent_revision_id,
			createdAt: row.created_at
		}));
	}

	createExecution(promptsetId: string, revisionId: string, repositoryId: string): Execution {
		const id = crypto.randomUUID();
		const now = Date.now();

		this.db.prepare(`
			INSERT INTO executions (id, promptset_id, revision_id, repository_id, session_id, thread_url, status, prompt_status, prompt_result, validation_status, validation_thread_url, validation_result, files_added, files_removed, files_modified, lines_added, lines_removed, created_at, completed_at)
			VALUES (?, ?, ?, ?, NULL, NULL, 'pending', NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 0, 0, ?, NULL)
		`).run(id, promptsetId, revisionId, repositoryId, now);

		return {
			id,
			promptsetId,
			revisionId,
			repositoryId,
			sessionId: null,
			threadUrl: null,
			status: 'pending',
			promptStatus: null,
			promptResult: null,
			validationStatus: null,
			validationThreadUrl: null,
			validationResult: null,
			filesAdded: 0,
			filesRemoved: 0,
			filesModified: 0,
			linesAdded: 0,
			linesRemoved: 0,
			createdAt: now,
			completedAt: null
		};
	}

	updateExecution(id: string, updates: Partial<Pick<Execution, 'status' | 'sessionId' | 'threadUrl' | 'promptStatus' | 'promptResult' | 'validationStatus' | 'validationThreadUrl' | 'validationResult' | 'filesAdded' | 'filesRemoved' | 'filesModified' | 'linesAdded' | 'linesRemoved' | 'completedAt'>>): void {
		const fields: string[] = [];
		const values: any[] = [];

		if (updates.status) {
			fields.push('status = ?');
			values.push(updates.status);
		}
		if (updates.sessionId !== undefined) {
			fields.push('session_id = ?');
			values.push(updates.sessionId);
		}
		if (updates.threadUrl !== undefined) {
			fields.push('thread_url = ?');
			values.push(updates.threadUrl);
		}
		if (updates.promptStatus !== undefined) {
			fields.push('prompt_status = ?');
			values.push(updates.promptStatus);
		}
		if (updates.promptResult !== undefined) {
			fields.push('prompt_result = ?');
			values.push(updates.promptResult);
		}
		if (updates.validationStatus !== undefined) {
			fields.push('validation_status = ?');
			values.push(updates.validationStatus);
		}
		if (updates.validationThreadUrl !== undefined) {
			fields.push('validation_thread_url = ?');
			values.push(updates.validationThreadUrl);
		}
		if (updates.validationResult !== undefined) {
			fields.push('validation_result = ?');
			values.push(updates.validationResult);
		}
		if (updates.filesAdded !== undefined) {
			fields.push('files_added = ?');
			values.push(updates.filesAdded);
		}
		if (updates.filesRemoved !== undefined) {
			fields.push('files_removed = ?');
			values.push(updates.filesRemoved);
		}
		if (updates.filesModified !== undefined) {
			fields.push('files_modified = ?');
			values.push(updates.filesModified);
		}
		if (updates.linesAdded !== undefined) {
			fields.push('lines_added = ?');
			values.push(updates.linesAdded);
		}
		if (updates.linesRemoved !== undefined) {
			fields.push('lines_removed = ?');
			values.push(updates.linesRemoved);
		}
		if (updates.completedAt !== undefined) {
			fields.push('completed_at = ?');
			values.push(updates.completedAt);
		}

		if (fields.length > 0) {
			values.push(id);
			this.db.prepare(`UPDATE executions SET ${fields.join(', ')} WHERE id = ?`).run(...values);
		}
	}

	getExecution(id: string): Execution | null {
		const row = this.db.prepare('SELECT * FROM executions WHERE id = ?').get(id) as any;
		if (!row) return null;

		return {
			id: row.id,
			promptsetId: row.promptset_id,
			revisionId: row.revision_id,
			repositoryId: row.repository_id,
			sessionId: row.session_id,
			threadUrl: row.thread_url,
			status: row.status,
			promptStatus: row.prompt_status,
			promptResult: row.prompt_result,
			validationStatus: row.validation_status,
			validationThreadUrl: row.validation_thread_url,
			validationResult: row.validation_result,
			filesAdded: row.files_added || 0,
			filesRemoved: row.files_removed || 0,
			filesModified: row.files_modified || 0,
			linesAdded: row.lines_added || 0,
			linesRemoved: row.lines_removed || 0,
			createdAt: row.created_at,
			completedAt: row.completed_at
		};
	}

	findExecutionByPrefix(idPrefix: string): Execution | null {
		const row = this.db.prepare('SELECT * FROM executions WHERE id LIKE ?').get(`${idPrefix}%`) as any;
		if (!row) return null;

		return {
			id: row.id,
			promptsetId: row.promptset_id,
			revisionId: row.revision_id,
			repositoryId: row.repository_id,
			sessionId: row.session_id,
			threadUrl: row.thread_url,
			status: row.status,
			promptStatus: row.prompt_status,
			promptResult: row.prompt_result,
			validationStatus: row.validation_status,
			validationThreadUrl: row.validation_thread_url,
			validationResult: row.validation_result,
			filesAdded: row.files_added || 0,
			filesRemoved: row.files_removed || 0,
			filesModified: row.files_modified || 0,
			linesAdded: row.lines_added || 0,
			linesRemoved: row.lines_removed || 0,
			createdAt: row.created_at,
			completedAt: row.completed_at
		};
	}

	getExecutionsByRevision(revisionId: string): Execution[] {
		const rows = this.db.prepare('SELECT * FROM executions WHERE revision_id = ? ORDER BY created_at DESC').all(revisionId) as any[];
		
		return rows.map(row => ({
			id: row.id,
			promptsetId: row.promptset_id,
			revisionId: row.revision_id,
			repositoryId: row.repository_id,
			sessionId: row.session_id,
			threadUrl: row.thread_url,
			status: row.status,
			promptStatus: row.prompt_status,
			promptResult: row.prompt_result,
			validationStatus: row.validation_status,
			validationThreadUrl: row.validation_thread_url,
			validationResult: row.validation_result,
			filesAdded: row.files_added || 0,
			filesRemoved: row.files_removed || 0,
			filesModified: row.files_modified || 0,
			linesAdded: row.lines_added || 0,
			linesRemoved: row.lines_removed || 0,
			createdAt: row.created_at,
			completedAt: row.completed_at
		}));
	}

	getExecutionsByPromptSet(promptsetId: string): Execution[] {
		const rows = this.db.prepare('SELECT * FROM executions WHERE promptset_id = ? ORDER BY created_at DESC').all(promptsetId) as any[];
		
		return rows.map(row => ({
			id: row.id,
			promptsetId: row.promptset_id,
			revisionId: row.revision_id,
			repositoryId: row.repository_id,
			sessionId: row.session_id,
			threadUrl: row.thread_url,
			status: row.status,
			promptStatus: row.prompt_status,
			promptResult: row.prompt_result,
			validationStatus: row.validation_status,
			validationThreadUrl: row.validation_thread_url,
			validationResult: row.validation_result,
			filesAdded: row.files_added || 0,
			filesRemoved: row.files_removed || 0,
			filesModified: row.files_modified || 0,
			linesAdded: row.lines_added || 0,
			linesRemoved: row.lines_removed || 0,
			createdAt: row.created_at,
			completedAt: row.completed_at
		}));
	}

	deleteExecution(id: string): boolean {
		const result = this.db.prepare('DELETE FROM executions WHERE id = ?').run(id);
		return result.changes > 0;
	}

	deletePromptSet(id: string): boolean {
		const result = this.db.prepare('DELETE FROM promptsets WHERE id = ?').run(id);
		return result.changes > 0;
	}

	deleteRepository(id: string): boolean {
		const result = this.db.prepare('DELETE FROM repositories WHERE id = ?').run(id);
		return result.changes > 0;
	}

	close(): void {
		this.db.close();
	}
}
