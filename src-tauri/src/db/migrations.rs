use anyhow::Result;
use rusqlite::Connection;

pub struct Migration {
    pub version: i32,
    pub up: fn(&Connection) -> Result<()>,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        up: migration_1,
    },
    Migration {
        version: 2,
        up: migration_2,
    },
    Migration {
        version: 3,
        up: migration_3,
    },
    Migration {
        version: 4,
        up: migration_4,
    },
    Migration {
        version: 5,
        up: migration_5,
    },
    Migration {
        version: 6,
        up: migration_6,
    },
    Migration {
        version: 7,
        up: migration_7,
    },
    Migration {
        version: 8,
        up: migration_8,
    },
    Migration {
        version: 9,
        up: migration_9,
    },
    Migration {
        version: 10,
        up: migration_10,
    },
    Migration {
        version: 11,
        up: migration_11,
    },
    Migration {
        version: 12,
        up: migration_12,
    },
    Migration {
        version: 13,
        up: migration_13,
    },
];

fn migration_1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
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
	",
    )?;
    Ok(())
}

fn migration_2(conn: &Connection) -> Result<()> {
    conn.execute_batch("ALTER TABLE executions ADD COLUMN session_id TEXT;")?;
    Ok(())
}

fn migration_3(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
		ALTER TABLE promptsets ADD COLUMN validation_prompt TEXT;
		ALTER TABLE executions ADD COLUMN validation_status TEXT;
		ALTER TABLE executions ADD COLUMN validation_thread_url TEXT;
	",
    )?;
    Ok(())
}

fn migration_4(conn: &Connection) -> Result<()> {
    conn.execute_batch("ALTER TABLE executions ADD COLUMN validation_result TEXT;")?;
    Ok(())
}

fn migration_5(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
		ALTER TABLE executions ADD COLUMN files_added INTEGER DEFAULT 0;
		ALTER TABLE executions ADD COLUMN files_removed INTEGER DEFAULT 0;
		ALTER TABLE executions ADD COLUMN files_modified INTEGER DEFAULT 0;
		ALTER TABLE executions ADD COLUMN lines_added INTEGER DEFAULT 0;
		ALTER TABLE executions ADD COLUMN lines_removed INTEGER DEFAULT 0;
	",
    )?;
    Ok(())
}

fn migration_6(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
		ALTER TABLE executions ADD COLUMN prompt_status TEXT;
		ALTER TABLE executions ADD COLUMN prompt_result TEXT;
	",
    )?;
    Ok(())
}

fn migration_7(conn: &Connection) -> Result<()> {
    conn.execute_batch(
		"
		ALTER TABLE executions ADD COLUMN commit_status TEXT CHECK (commit_status IN ('none', 'uncommitted', 'committed')) DEFAULT 'none';
		ALTER TABLE executions ADD COLUMN commit_sha TEXT;
		ALTER TABLE executions ADD COLUMN committed_at INTEGER;
	",
	)?;
    Ok(())
}

fn migration_8(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
		ALTER TABLE executions ADD COLUMN parent_sha TEXT;
		ALTER TABLE executions ADD COLUMN branch TEXT;
	",
    )?;
    Ok(())
}

fn migration_9(conn: &Connection) -> Result<()> {
    conn.execute_batch("ALTER TABLE repositories ADD COLUMN default_branch TEXT DEFAULT 'main';")?;
    Ok(())
}

fn migration_10(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "ALTER TABLE promptsets ADD COLUMN auto_validate INTEGER NOT NULL DEFAULT 0;",
    )?;
    Ok(())
}

fn migration_11(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
ALTER TABLE executions ADD COLUMN ci_status TEXT;
ALTER TABLE executions ADD COLUMN ci_checked_at INTEGER;
ALTER TABLE executions ADD COLUMN ci_url TEXT;
",
    )?;
    Ok(())
}

fn migration_12(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
		CREATE TABLE settings (
			key TEXT PRIMARY KEY,
			value TEXT NOT NULL,
			updated_at INTEGER NOT NULL
		);
		
		-- Set default CI stuck threshold to 10 minutes
		INSERT INTO settings (key, value, updated_at) VALUES ('ci_stuck_threshold_minutes', '10', 0);
		",
    )?;
    Ok(())
}

fn migration_13(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
		-- Failure analyses main table
		CREATE TABLE analyses (
			id TEXT PRIMARY KEY,
			revision_id TEXT NOT NULL,
			type TEXT NOT NULL CHECK (type IN ('execution', 'validation')),
			status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed', 'failed')),
			analysis_prompt TEXT NOT NULL,
			analysis_result TEXT,
			amp_thread_url TEXT,
			amp_session_id TEXT,
			error_message TEXT,
			created_at INTEGER NOT NULL,
			updated_at INTEGER NOT NULL,
			completed_at INTEGER,
			FOREIGN KEY (revision_id) REFERENCES prompt_revisions(id) ON DELETE CASCADE
		);

		-- Join table to link analyses to executions (many-to-many)
		CREATE TABLE analysis_executions (
			analysis_id TEXT NOT NULL,
			execution_id TEXT NOT NULL,
			PRIMARY KEY (analysis_id, execution_id),
			FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE,
			FOREIGN KEY (execution_id) REFERENCES executions(id) ON DELETE CASCADE
		);

		-- Indexes for common queries
		CREATE INDEX idx_analyses_revision_type_created ON analyses (revision_id, type, created_at DESC);
		CREATE INDEX idx_analyses_status_created ON analyses (status, created_at DESC);
		CREATE INDEX idx_analysis_execs_execution ON analysis_executions (execution_id);
		CREATE INDEX idx_analysis_execs_analysis ON analysis_executions (analysis_id);

		-- Trigger to keep updated_at fresh
		CREATE TRIGGER analyses_set_updated_at
		AFTER UPDATE ON analyses
		FOR EACH ROW
		BEGIN
			UPDATE analyses SET updated_at = strftime('%s', 'now') WHERE id = OLD.id;
		END;
		",
    )?;
    Ok(())
}

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
			version INTEGER PRIMARY KEY,
			applied_at INTEGER NOT NULL
		);",
    )?;

    let current_version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version > current_version {
            (migration.up)(conn)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![migration.version, chrono::Utc::now().timestamp_millis()],
            )?;
        }
    }

    Ok(())
}
