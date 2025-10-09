import { Database } from 'bun:sqlite';

export interface Migration {
	version: number;
	up: (db: Database) => void;
}

export const migrations: Migration[] = [
	{
		version: 1,
		up: (db: Database) => {
			db.exec(`
				CREATE TABLE repositories (
					id TEXT PRIMARY KEY,
					provider TEXT NOT NULL,
					provider_id TEXT NOT NULL,
					name TEXT,
					last_synced_at INTEGER,
					created_at INTEGER NOT NULL,
					UNIQUE(provider, provider_id)
				);

				CREATE TABLE promptsets (
					id TEXT PRIMARY KEY,
					name TEXT NOT NULL,
					created_at INTEGER NOT NULL
				);

				CREATE TABLE promptset_repositories (
					promptset_id TEXT NOT NULL,
					repository_id TEXT NOT NULL,
					PRIMARY KEY (promptset_id, repository_id),
					FOREIGN KEY (promptset_id) REFERENCES promptsets(id) ON DELETE CASCADE,
					FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
				);

				CREATE TABLE prompt_revisions (
					id TEXT PRIMARY KEY,
					promptset_id TEXT NOT NULL,
					prompt_text TEXT NOT NULL,
					parent_revision_id TEXT,
					created_at INTEGER NOT NULL,
					FOREIGN KEY (promptset_id) REFERENCES promptsets(id) ON DELETE CASCADE,
					FOREIGN KEY (parent_revision_id) REFERENCES prompt_revisions(id)
				);

				CREATE TABLE executions (
					id TEXT PRIMARY KEY,
					promptset_id TEXT NOT NULL,
					revision_id TEXT NOT NULL,
					repository_id TEXT NOT NULL,
					thread_url TEXT,
					status TEXT NOT NULL,
					created_at INTEGER NOT NULL,
					completed_at INTEGER,
					FOREIGN KEY (promptset_id) REFERENCES promptsets(id) ON DELETE CASCADE,
					FOREIGN KEY (revision_id) REFERENCES prompt_revisions(id),
					FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
				);

				CREATE INDEX idx_executions_promptset ON executions(promptset_id);
				CREATE INDEX idx_executions_revision ON executions(revision_id);
				CREATE INDEX idx_executions_repository ON executions(repository_id);
				CREATE INDEX idx_prompt_revisions_promptset ON prompt_revisions(promptset_id);
			`);
		}
	},
	{
		version: 2,
		up: (db: Database) => {
			db.exec(`
				ALTER TABLE executions ADD COLUMN session_id TEXT;
			`);
		}
	},
	{
		version: 3,
		up: (db: Database) => {
			db.exec(`
				ALTER TABLE promptsets ADD COLUMN validation_prompt TEXT;
				ALTER TABLE executions ADD COLUMN validation_status TEXT;
				ALTER TABLE executions ADD COLUMN validation_thread_url TEXT;
			`);
		}
	},
	{
		version: 4,
		up: (db: Database) => {
			db.exec(`
				ALTER TABLE executions ADD COLUMN validation_result TEXT;
			`);
		}
	},
	{
		version: 5,
		up: (db: Database) => {
			db.exec(`
				ALTER TABLE executions ADD COLUMN files_added INTEGER DEFAULT 0;
				ALTER TABLE executions ADD COLUMN files_removed INTEGER DEFAULT 0;
				ALTER TABLE executions ADD COLUMN files_modified INTEGER DEFAULT 0;
				ALTER TABLE executions ADD COLUMN lines_added INTEGER DEFAULT 0;
				ALTER TABLE executions ADD COLUMN lines_removed INTEGER DEFAULT 0;
			`);
		}
	},
	{
		version: 6,
		up: (db: Database) => {
			db.exec(`
				ALTER TABLE executions ADD COLUMN prompt_status TEXT;
				ALTER TABLE executions ADD COLUMN prompt_result TEXT;
			`);
		}
	}
];

export function runMigrations(db: Database): void {
	db.exec(`
		CREATE TABLE IF NOT EXISTS schema_migrations (
			version INTEGER PRIMARY KEY,
			applied_at INTEGER NOT NULL
		);
	`);

	const getCurrentVersion = db.prepare('SELECT MAX(version) as version FROM schema_migrations');
	const currentVersion = (getCurrentVersion.get() as { version: number | null }).version || 0;

	for (const migration of migrations) {
		if (migration.version > currentVersion) {
			migration.up(db);
			db.prepare('INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)').run(
				migration.version,
				Date.now()
			);
		}
	}
}
