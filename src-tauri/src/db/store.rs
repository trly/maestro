use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::migrations::run_migrations;
use crate::types::{ExecutionStatus, ValidationStatus, PromptStatus, CommitStatus, CiStatus, Analysis, AnalysisType, AnalysisStatus};

fn now_ms() -> i64 {
	chrono::Utc::now().timestamp_millis()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
	pub id: String,
	pub provider: String,
	pub provider_id: String,
	pub name: Option<String>,
	pub default_branch: Option<String>,
	pub last_synced_at: Option<i64>,
	pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSetStats {
	pub total_executions: i64,
	pub total_completions: i64,
	pub total_validations: i64,
	pub total_revisions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSet {
	pub id: String,
	pub name: String,
	pub repository_ids: Vec<String>,
	pub validation_prompt: Option<String>,
	pub auto_validate: bool,
	pub created_at: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub stats: Option<PromptSetStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStats {
	pub total: i64,
	pub completed: i64,
	pub validation_passed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptRevision {
	pub id: String,
	pub promptset_id: String,
	pub prompt_text: String,
	pub parent_revision_id: Option<String>,
	pub created_at: i64,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub execution_stats: Option<ExecutionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Execution {
	pub id: String,
	pub promptset_id: String,
	pub revision_id: String,
	pub repository_id: String,
	pub session_id: Option<String>,
	pub thread_url: Option<String>,
	pub status: ExecutionStatus,
	pub prompt_status: Option<PromptStatus>,
	pub prompt_result: Option<String>,
	pub validation_status: Option<ValidationStatus>,
	pub validation_thread_url: Option<String>,
	pub validation_result: Option<String>,
	pub files_added: i64,
	pub files_removed: i64,
	pub files_modified: i64,
	pub lines_added: i64,
	pub lines_removed: i64,
	pub commit_status: CommitStatus,
	pub commit_sha: Option<String>,
	pub committed_at: Option<i64>,
	pub parent_sha: Option<String>,
	pub branch: Option<String>,
	pub ci_status: Option<CiStatus>,
	pub ci_checked_at: Option<i64>,
	pub ci_url: Option<String>,
	pub created_at: i64,
	pub completed_at: Option<i64>,
}

const SELECT_REPOSITORY: &str = "SELECT id, provider, provider_id, name, default_branch, last_synced_at, created_at FROM repositories";

const SELECT_EXECUTION: &str = "
SELECT 
	id, promptset_id, revision_id, repository_id, session_id, thread_url, status,
	prompt_status, prompt_result, validation_status, validation_thread_url, validation_result,
	COALESCE(files_added, 0) AS files_added,
	COALESCE(files_removed, 0) AS files_removed,
	COALESCE(files_modified, 0) AS files_modified,
	COALESCE(lines_added, 0) AS lines_added,
	COALESCE(lines_removed, 0) AS lines_removed,
	COALESCE(commit_status, 'none') AS commit_status,
	commit_sha, committed_at, parent_sha, branch,
	ci_status, ci_checked_at, ci_url,
	created_at, completed_at
FROM executions";

const SELECT_ANALYSIS: &str = "
SELECT 
	id, revision_id, type, status, analysis_prompt, analysis_result,
	amp_thread_url, amp_session_id, error_message,
	created_at, updated_at, completed_at
FROM analyses";

fn map_repository(row: &Row) -> rusqlite::Result<Repository> {
	Ok(Repository {
		id: row.get("id")?,
		provider: row.get("provider")?,
		provider_id: row.get("provider_id")?,
		name: row.get("name")?,
		default_branch: row.get("default_branch")?,
		last_synced_at: row.get("last_synced_at")?,
		created_at: row.get("created_at")?,
	})
}

fn map_execution(row: &Row) -> rusqlite::Result<Execution> {
	Ok(Execution {
		id: row.get("id")?,
		promptset_id: row.get("promptset_id")?,
		revision_id: row.get("revision_id")?,
		repository_id: row.get("repository_id")?,
		session_id: row.get("session_id")?,
		thread_url: row.get("thread_url")?,
		status: row.get("status")?,
		prompt_status: row.get("prompt_status")?,
		prompt_result: row.get("prompt_result")?,
		validation_status: row.get("validation_status")?,
		validation_thread_url: row.get("validation_thread_url")?,
		validation_result: row.get("validation_result")?,
		files_added: row.get("files_added")?,
		files_removed: row.get("files_removed")?,
		files_modified: row.get("files_modified")?,
		lines_added: row.get("lines_added")?,
		lines_removed: row.get("lines_removed")?,
		commit_status: row.get("commit_status")?,
		commit_sha: row.get("commit_sha")?,
		committed_at: row.get("committed_at")?,
		parent_sha: row.get("parent_sha")?,
		branch: row.get("branch")?,
		ci_status: row.get("ci_status")?,
		ci_checked_at: row.get("ci_checked_at")?,
		ci_url: row.get("ci_url")?,
		created_at: row.get("created_at")?,
		completed_at: row.get("completed_at")?,
	})
}

fn map_analysis(row: &Row) -> rusqlite::Result<Analysis> {
	Ok(Analysis {
		id: row.get("id")?,
		revision_id: row.get("revision_id")?,
		analysis_type: row.get("type")?,
		status: row.get("status")?,
		analysis_prompt: row.get("analysis_prompt")?,
		analysis_result: row.get("analysis_result")?,
		amp_thread_url: row.get("amp_thread_url")?,
		amp_session_id: row.get("amp_session_id")?,
		error_message: row.get("error_message")?,
		created_at: row.get("created_at")?,
		updated_at: row.get("updated_at")?,
		completed_at: row.get("completed_at")?,
	})
}

pub struct Store {
	conn: Connection,
}

impl Store {
	pub fn new(db_path: &str) -> Result<Self> {
		let conn = Connection::open(db_path)?;
		conn.execute("PRAGMA foreign_keys = ON;", [])?;
		run_migrations(&conn)?;
		Ok(Self { conn })
	}

	fn hash_prompt(&self, text: &str) -> String {
		let mut hasher = Sha256::new();
		hasher.update(text.as_bytes());
		hex::encode(hasher.finalize())
	}

	pub fn create_repository(&self, provider: &str, provider_id: &str) -> Result<Repository> {
		let id = Uuid::new_v4().to_string();
		let now = now_ms();

		self.conn.execute(
			"INSERT INTO repositories (id, provider, provider_id, name, default_branch, last_synced_at, created_at)
			 VALUES (?1, ?2, ?3, NULL, NULL, NULL, ?4)",
			params![id, provider, provider_id, now],
		)?;

		Ok(Repository {
			id,
			provider: provider.to_string(),
			provider_id: provider_id.to_string(),
			name: None,
			default_branch: None,
			last_synced_at: None,
			created_at: now,
		})
	}

	pub fn update_repository_name(&self, id: &str, name: &str) -> Result<()> {
		self.conn.execute(
			"UPDATE repositories SET name = ?1, last_synced_at = ?2 WHERE id = ?3",
			params![name, now_ms(), id],
		)?;
		Ok(())
	}

	pub fn update_repository_default_branch(&self, id: &str, default_branch: &str) -> Result<()> {
		self.conn.execute(
			"UPDATE repositories SET default_branch = ?1 WHERE id = ?2",
			params![default_branch, id],
		)?;
		Ok(())
	}

	pub fn get_repository(&self, id: &str) -> Result<Option<Repository>> {
		let mut stmt = self.conn.prepare_cached(&format!("{SELECT_REPOSITORY} WHERE id = ?1"))?;
		stmt.query_row([id], map_repository)
			.optional()
			.map_err(Into::into)
	}

	pub fn find_repository(&self, provider: &str, provider_id: &str) -> Result<Option<Repository>> {
		let mut stmt = self.conn.prepare_cached(
			&format!("{SELECT_REPOSITORY} WHERE provider = ?1 AND provider_id = ?2"),
		)?;
		stmt.query_row([provider, provider_id], map_repository)
			.optional()
			.map_err(Into::into)
	}

	pub fn get_all_repositories(&self) -> Result<Vec<Repository>> {
		let mut stmt = self.conn.prepare(&format!("{SELECT_REPOSITORY} ORDER BY created_at DESC"))?;
		let repos = stmt.query_map([], map_repository)?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(repos)
	}

	pub fn create_promptset(
		&mut self,
		name: &str,
		repository_ids: Vec<String>,
		validation_prompt: Option<String>,
		auto_validate: bool,
	) -> Result<PromptSet> {
		let id = Uuid::new_v4().to_string();
		let now = now_ms();

		let tx = self.conn.transaction()?;

		tx.execute(
			"INSERT INTO promptsets (id, name, validation_prompt, auto_validate, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
			params![id, name, validation_prompt, auto_validate as i32, now],
		)?;

		{
			let mut stmt = tx.prepare(
				"INSERT INTO promptset_repositories (promptset_id, repository_id) VALUES (?1, ?2)",
			)?;
			for repo_id in &repository_ids {
				stmt.execute(params![id, repo_id])?;
			}
		}

		tx.commit()?;

		Ok(PromptSet {
			id,
			name: name.to_string(),
			repository_ids,
			validation_prompt,
			auto_validate,
			created_at: now,
			stats: None,
		})
	}

	pub fn get_all_promptsets(&self) -> Result<Vec<PromptSet>> {
		let mut stmt = self.conn.prepare("SELECT id, name, created_at, validation_prompt, auto_validate FROM promptsets ORDER BY created_at DESC")?;
		let mut rows = stmt.query([])?;

		let mut promptsets = Vec::new();
		while let Some(row) = rows.next()? {
			let id: String = row.get(0)?;
			let name: String = row.get(1)?;
			let created_at: i64 = row.get(2)?;
			let validation_prompt: Option<String> = row.get(3)?;
			let auto_validate: bool = row.get::<_, i32>(4)? != 0;

			let mut repo_stmt = self.conn.prepare_cached(
				"SELECT repository_id FROM promptset_repositories WHERE promptset_id = ?1",
			)?;
			let repo_ids: Vec<String> = repo_stmt
				.query_map([&id], |row| row.get(0))?
				.collect::<rusqlite::Result<Vec<_>>>()?;

			let stats = self.conn.query_row(
				"SELECT 
					COUNT(*) AS total_executions,
					COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0) AS total_completions,
					COALESCE(SUM(CASE WHEN validation_status = 'passed' THEN 1 ELSE 0 END), 0) AS total_validations,
					COUNT(DISTINCT revision_id) AS total_revisions
				FROM executions
				WHERE promptset_id = ?1",
				[&id],
				|row| {
					Ok(PromptSetStats {
						total_executions: row.get("total_executions")?,
						total_completions: row.get("total_completions")?,
						total_validations: row.get("total_validations")?,
						total_revisions: row.get("total_revisions")?,
					})
				},
			).optional()?;

			promptsets.push(PromptSet {
				id,
				name,
				repository_ids: repo_ids,
				validation_prompt,
				auto_validate,
				created_at,
				stats,
			});
		}

		Ok(promptsets)
	}

	pub fn get_promptset(&self, id: &str) -> Result<Option<PromptSet>> {
		let mut stmt = self.conn.prepare_cached("SELECT id, name, created_at, validation_prompt, auto_validate FROM promptsets WHERE id = ?1")?;
		let result = stmt.query_row([id], |row| {
			Ok((
				row.get::<_, String>(0)?,
				row.get::<_, String>(1)?,
				row.get::<_, i64>(2)?,
				row.get::<_, Option<String>>(3)?,
				row.get::<_, i32>(4)? != 0,
			))
		}).optional()?;

		if let Some((id, name, created_at, validation_prompt, auto_validate)) = result {
			let mut repo_stmt = self.conn.prepare_cached(
				"SELECT repository_id FROM promptset_repositories WHERE promptset_id = ?1",
			)?;
			let repo_ids: Vec<String> = repo_stmt
				.query_map([&id], |row| row.get(0))?
				.collect::<rusqlite::Result<Vec<_>>>()?;

			Ok(Some(PromptSet {
				id,
				name,
				repository_ids: repo_ids,
				validation_prompt,
				auto_validate,
				created_at,
				stats: None,
			}))
		} else {
			Ok(None)
		}
	}

	pub fn find_promptset_by_prefix(&self, id_prefix: &str) -> Result<Option<PromptSet>> {
		let pattern = format!("{}%", id_prefix);
		let mut stmt = self.conn.prepare_cached("SELECT id, name, created_at, validation_prompt, auto_validate FROM promptsets WHERE id LIKE ?1")?;
		let result = stmt.query_row([pattern], |row| {
			Ok((
				row.get::<_, String>(0)?,
				row.get::<_, String>(1)?,
				row.get::<_, i64>(2)?,
				row.get::<_, Option<String>>(3)?,
				row.get::<_, i32>(4)? != 0,
			))
		}).optional()?;

		if let Some((id, name, created_at, validation_prompt, auto_validate)) = result {
			let mut repo_stmt = self.conn.prepare_cached(
				"SELECT repository_id FROM promptset_repositories WHERE promptset_id = ?1",
			)?;
			let repo_ids: Vec<String> = repo_stmt
				.query_map([&id], |row| row.get(0))?
				.collect::<rusqlite::Result<Vec<_>>>()?;

			Ok(Some(PromptSet {
				id,
				name,
				repository_ids: repo_ids,
				validation_prompt,
				auto_validate,
				created_at,
				stats: None,
			}))
		} else {
			Ok(None)
		}
	}

	pub fn update_promptset_validation(&self, id: &str, validation_prompt: Option<String>) -> Result<()> {
		self.conn.execute(
			"UPDATE promptsets SET validation_prompt = ?1 WHERE id = ?2",
			params![validation_prompt, id],
		)?;
		Ok(())
	}

	pub fn update_promptset_auto_validate(&self, id: &str, auto_validate: bool) -> Result<()> {
		self.conn.execute(
			"UPDATE promptsets SET auto_validate = ?1 WHERE id = ?2",
			params![auto_validate as i32, id],
		)?;
		Ok(())
	}

	pub fn update_promptset_repositories(&mut self, id: &str, repository_ids: Vec<String>) -> Result<()> {
		let tx = self.conn.transaction()?;

		tx.execute("DELETE FROM promptset_repositories WHERE promptset_id = ?1", [id])?;

		{
			let mut stmt = tx.prepare(
				"INSERT INTO promptset_repositories (promptset_id, repository_id) VALUES (?1, ?2)",
			)?;
			for repo_id in repository_ids {
				stmt.execute(params![id, repo_id])?;
			}
		}

		tx.commit()?;
		Ok(())
	}

	pub fn create_prompt_revision(
		&self,
		promptset_id: &str,
		prompt_text: &str,
		parent_revision_id: Option<String>,
	) -> Result<PromptRevision> {
		let id = self.hash_prompt(prompt_text);
		let now = now_ms();

		if let Some(existing) = self.get_prompt_revision(&id)? {
			return Ok(existing);
		}

		self.conn.execute(
			"INSERT INTO prompt_revisions (id, promptset_id, prompt_text, parent_revision_id, created_at)
			 VALUES (?1, ?2, ?3, ?4, ?5)",
			params![id, promptset_id, prompt_text, parent_revision_id, now],
		)?;

		Ok(PromptRevision {
			id,
			promptset_id: promptset_id.to_string(),
			prompt_text: prompt_text.to_string(),
			parent_revision_id,
			created_at: now,
			execution_stats: None,
		})
	}

	pub fn get_prompt_revision(&self, id: &str) -> Result<Option<PromptRevision>> {
		let mut stmt = self.conn.prepare_cached("SELECT * FROM prompt_revisions WHERE id = ?1")?;
		stmt.query_row([id], |row| {
			Ok(PromptRevision {
				id: row.get(0)?,
				promptset_id: row.get(1)?,
				prompt_text: row.get(2)?,
				parent_revision_id: row.get(3)?,
				created_at: row.get(4)?,
				execution_stats: None,
			})
		})
		.optional()
		.map_err(Into::into)
	}

	pub fn find_prompt_revision_by_prefix(&self, id_prefix: &str) -> Result<Option<PromptRevision>> {
		let pattern = format!("{}%", id_prefix);
		let mut stmt = self.conn.prepare_cached("SELECT * FROM prompt_revisions WHERE id LIKE ?1")?;
		stmt.query_row([pattern], |row| {
			Ok(PromptRevision {
				id: row.get(0)?,
				promptset_id: row.get(1)?,
				prompt_text: row.get(2)?,
				parent_revision_id: row.get(3)?,
				created_at: row.get(4)?,
				execution_stats: None,
			})
		})
		.optional()
		.map_err(Into::into)
	}

	pub fn get_promptset_revisions(&self, promptset_id: &str) -> Result<Vec<PromptRevision>> {
		let mut stmt = self.conn.prepare(
			"SELECT * FROM prompt_revisions WHERE promptset_id = ?1 ORDER BY created_at DESC",
		)?;
		let mut rows = stmt.query([promptset_id])?;

		let mut revisions = Vec::new();
		while let Some(row) = rows.next()? {
			let id: String = row.get(0)?;
			let promptset_id: String = row.get(1)?;
			let prompt_text: String = row.get(2)?;
			let parent_revision_id: Option<String> = row.get(3)?;
			let created_at: i64 = row.get(4)?;

			let stats = self.conn.query_row(
			"SELECT 
			COUNT(*) AS total,
			COALESCE(SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END), 0) AS completed,
			COALESCE(SUM(CASE WHEN validation_status = 'passed' THEN 1 ELSE 0 END), 0) AS validation_passed
			FROM executions
			WHERE revision_id = ?1",
			[&id],
			|row| {
			Ok(ExecutionStats {
			total: row.get("total")?,
			completed: row.get("completed")?,
			validation_passed: row.get("validation_passed")?,
			})
			},
			).optional()?;

			revisions.push(PromptRevision {
				id,
				promptset_id,
				prompt_text,
				parent_revision_id,
				created_at,
				execution_stats: stats,
			});
		}

		Ok(revisions)
	}

	pub fn create_execution(
		&self,
		promptset_id: &str,
		revision_id: &str,
		repository_id: &str,
	) -> Result<Execution> {
		let id = Uuid::new_v4().to_string();
		let now = now_ms();

		self.conn.execute(
			"INSERT INTO executions (id, promptset_id, revision_id, repository_id, session_id, thread_url, status, prompt_status, prompt_result, validation_status, validation_thread_url, validation_result, files_added, files_removed, files_modified, lines_added, lines_removed, commit_status, commit_sha, committed_at, parent_sha, branch, ci_status, ci_checked_at, ci_url, created_at, completed_at)
			 VALUES (?1, ?2, ?3, ?4, NULL, NULL, 'pending', NULL, NULL, NULL, NULL, NULL, 0, 0, 0, 0, 0, 'none', NULL, NULL, NULL, NULL, NULL, NULL, NULL, ?5, NULL)",
			params![id, promptset_id, revision_id, repository_id, now],
		)?;

		Ok(Execution {
			id,
			promptset_id: promptset_id.to_string(),
			revision_id: revision_id.to_string(),
			repository_id: repository_id.to_string(),
			session_id: None,
			thread_url: None,
			status: ExecutionStatus::Pending,
			prompt_status: None,
			prompt_result: None,
			validation_status: None,
			validation_thread_url: None,
			validation_result: None,
			files_added: 0,
			files_removed: 0,
			files_modified: 0,
			lines_added: 0,
			lines_removed: 0,
			commit_status: CommitStatus::None,
			commit_sha: None,
			committed_at: None,
			parent_sha: None,
			branch: None,
			ci_status: None,
			ci_checked_at: None,
			ci_url: None,
			created_at: now,
			completed_at: None,
		})
	}

	pub fn update_execution(&self, id: &str, u: ExecutionUpdates) -> Result<()> {
		self.conn.execute(
			"UPDATE executions SET
				status = COALESCE(?1, status),
				session_id = COALESCE(?2, session_id),
				thread_url = COALESCE(?3, thread_url),
				prompt_status = COALESCE(?4, prompt_status),
				prompt_result = COALESCE(?5, prompt_result),
				validation_status = COALESCE(?6, validation_status),
				validation_thread_url = COALESCE(?7, validation_thread_url),
				validation_result = COALESCE(?8, validation_result),
				files_added = COALESCE(?9, files_added),
				files_removed = COALESCE(?10, files_removed),
				files_modified = COALESCE(?11, files_modified),
				lines_added = COALESCE(?12, lines_added),
				lines_removed = COALESCE(?13, lines_removed),
				commit_status = COALESCE(?14, commit_status),
				commit_sha = COALESCE(?15, commit_sha),
				committed_at = COALESCE(?16, committed_at),
				parent_sha = COALESCE(?17, parent_sha),
				branch = COALESCE(?18, branch),
				ci_status = COALESCE(?19, ci_status),
				ci_checked_at = COALESCE(?20, ci_checked_at),
				ci_url = COALESCE(?21, ci_url),
				completed_at = COALESCE(?22, completed_at)
			WHERE id = ?23",
			params![
				u.status,
				u.session_id,
				u.thread_url,
				u.prompt_status,
				u.prompt_result,
				u.validation_status,
				u.validation_thread_url,
				u.validation_result,
				u.files_added,
				u.files_removed,
				u.files_modified,
				u.lines_added,
				u.lines_removed,
				u.commit_status,
				u.commit_sha,
				u.committed_at,
				u.parent_sha,
				u.branch,
				u.ci_status,
				u.ci_checked_at,
				u.ci_url,
				u.completed_at,
				id,
			],
		)?;
		Ok(())
	}

	pub fn get_execution(&self, id: &str) -> Result<Option<Execution>> {
		let mut stmt = self.conn.prepare_cached(&format!("{SELECT_EXECUTION} WHERE id = ?1"))?;
		stmt.query_row([id], map_execution)
			.optional()
			.map_err(Into::into)
	}

	pub fn find_execution_by_prefix(&self, id_prefix: &str) -> Result<Option<Execution>> {
		let pattern = format!("{}%", id_prefix);
		let mut stmt = self.conn.prepare_cached(&format!("{SELECT_EXECUTION} WHERE id LIKE ?1"))?;
		stmt.query_row([pattern], map_execution)
			.optional()
			.map_err(Into::into)
	}

	pub fn get_executions_by_revision(&self, revision_id: &str) -> Result<Vec<Execution>> {
		let mut stmt = self.conn.prepare(&format!(
			"{SELECT_EXECUTION} WHERE revision_id = ?1 ORDER BY created_at DESC"
		))?;
		let executions = stmt.query_map([revision_id], map_execution)?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(executions)
	}

	pub fn get_executions_by_promptset(&self, promptset_id: &str) -> Result<Vec<Execution>> {
		let mut stmt = self.conn.prepare(&format!(
			"{SELECT_EXECUTION} WHERE promptset_id = ?1 ORDER BY created_at DESC"
		))?;
		let executions = stmt.query_map([promptset_id], map_execution)?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(executions)
	}

	pub fn get_all_executions(&self) -> Result<Vec<Execution>> {
		let mut stmt = self.conn.prepare(&format!(
			"{SELECT_EXECUTION} ORDER BY created_at DESC"
		))?;
		let executions = stmt.query_map([], map_execution)?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(executions)
	}

	pub fn delete_execution(&self, id: &str) -> Result<bool> {
		let result = self.conn.execute("DELETE FROM executions WHERE id = ?1", [id])?;
		Ok(result > 0)
	}

	pub fn delete_promptset(&self, id: &str) -> Result<bool> {
		let result = self.conn.execute("DELETE FROM promptsets WHERE id = ?1", [id])?;
		Ok(result > 0)
	}

	pub fn delete_repository(&self, id: &str) -> Result<bool> {
		let result = self.conn.execute("DELETE FROM repositories WHERE id = ?1", [id])?;
		Ok(result > 0)
	}

	pub fn delete_prompt_revision(&mut self, id: &str) -> Result<bool> {
		let tx = self.conn.transaction()?;

		tx.execute(
			"UPDATE prompt_revisions SET parent_revision_id = NULL WHERE parent_revision_id = ?1",
			[id],
		)?;

		let result = tx.execute("DELETE FROM prompt_revisions WHERE id = ?1", [id])?;
		
		tx.commit()?;
		Ok(result > 0)
	}

	// Settings operations
	pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
		self.conn
			.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
				row.get(0)
			})
			.optional()
			.map_err(Into::into)
	}

	pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
		let now = chrono::Utc::now().timestamp_millis();
		self.conn.execute(
			"INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
			rusqlite::params![key, value, now],
		)?;
		Ok(())
	}

	pub fn get_ci_stuck_threshold_minutes(&self) -> Result<i64> {
		let value = self.get_setting("ci_stuck_threshold_minutes")?
			.unwrap_or_else(|| "10".to_string());
		value.parse::<i64>().map_err(|e| anyhow::anyhow!("Invalid CI threshold: {}", e))
	}

	pub fn get_max_concurrent_executions(&self) -> Result<i64> {
		let value = self.get_setting("max_concurrent_executions")?
			.unwrap_or_else(|| "10".to_string());
		value.parse::<i64>().map_err(|e| anyhow::anyhow!("Invalid max concurrent executions: {}", e))
	}

	// Analysis operations
	pub fn create_analysis(
		&self,
		id: &str,
		revision_id: &str,
		analysis_type: AnalysisType,
		analysis_prompt: &str,
	) -> Result<Analysis> {
		let now = now_ms();

		self.conn.execute(
			"INSERT INTO analyses (id, revision_id, type, status, analysis_prompt, analysis_result, amp_thread_url, amp_session_id, error_message, created_at, updated_at, completed_at)
			 VALUES (?1, ?2, ?3, 'pending', ?4, NULL, NULL, NULL, NULL, ?5, ?5, NULL)",
			params![id, revision_id, analysis_type, analysis_prompt, now],
		)?;

		Ok(Analysis {
			id: id.to_string(),
			revision_id: revision_id.to_string(),
			analysis_type,
			status: AnalysisStatus::Pending,
			analysis_prompt: analysis_prompt.to_string(),
			analysis_result: None,
			amp_thread_url: None,
			amp_session_id: None,
			error_message: None,
			created_at: now,
			updated_at: now,
			completed_at: None,
		})
	}

	pub fn get_analysis(&self, id: &str) -> Result<Option<Analysis>> {
		let mut stmt = self.conn.prepare_cached(&format!("{SELECT_ANALYSIS} WHERE id = ?1"))?;
		stmt.query_row([id], map_analysis)
			.optional()
			.map_err(Into::into)
	}

	pub fn get_analyses_by_revision(
		&self,
		revision_id: &str,
		analysis_type: Option<AnalysisType>,
	) -> Result<Vec<Analysis>> {
		let query = match analysis_type {
			Some(_) => format!("{SELECT_ANALYSIS} WHERE revision_id = ?1 AND type = ?2 ORDER BY created_at DESC"),
			None => format!("{SELECT_ANALYSIS} WHERE revision_id = ?1 ORDER BY created_at DESC"),
		};

		let mut stmt = self.conn.prepare(&query)?;
		
		let analyses = if let Some(atype) = analysis_type {
			stmt.query_map(params![revision_id, atype], map_analysis)?
				.collect::<rusqlite::Result<Vec<_>>>()?
		} else {
			stmt.query_map([revision_id], map_analysis)?
				.collect::<rusqlite::Result<Vec<_>>>()?
		};

		Ok(analyses)
	}

	pub fn update_analysis_status(
		&self,
		id: &str,
		status: AnalysisStatus,
		error_message: Option<String>,
	) -> Result<()> {
		let now = now_ms();
		self.conn.execute(
			"UPDATE analyses SET status = ?1, error_message = ?2, updated_at = ?3 WHERE id = ?4",
			params![status, error_message, now, id],
		)?;
		Ok(())
	}

	pub fn update_analysis_prompt(
		&self,
		id: &str,
		analysis_prompt: &str,
	) -> Result<()> {
		let now = now_ms();
		self.conn.execute(
			"UPDATE analyses SET analysis_prompt = ?1, updated_at = ?2 WHERE id = ?3",
			params![analysis_prompt, now, id],
		)?;
		Ok(())
	}

	pub fn update_analysis_result(
		&self,
		id: &str,
		result: &str,
		amp_thread_url: Option<String>,
		amp_session_id: Option<String>,
		completed_at: i64,
	) -> Result<()> {
		let now = now_ms();
		self.conn.execute(
			"UPDATE analyses SET 
				analysis_result = ?1,
				amp_thread_url = ?2,
				amp_session_id = ?3,
				completed_at = ?4,
				updated_at = ?5
			WHERE id = ?6",
			params![result, amp_thread_url, amp_session_id, completed_at, now, id],
		)?;
		Ok(())
	}

	// Analysis-Execution join table operations
	pub fn add_analysis_executions(
		&mut self,
		analysis_id: &str,
		execution_ids: Vec<String>,
	) -> Result<()> {
		let tx = self.conn.transaction()?;

		{
			let mut stmt = tx.prepare(
				"INSERT INTO analysis_executions (analysis_id, execution_id) VALUES (?1, ?2)",
			)?;
			for execution_id in execution_ids {
				stmt.execute(params![analysis_id, execution_id])?;
			}
		}

		tx.commit()?;
		Ok(())
	}

	pub fn get_analysis_executions(&self, analysis_id: &str) -> Result<Vec<Execution>> {
		let mut stmt = self.conn.prepare(&format!(
			"{SELECT_EXECUTION} 
			WHERE id IN (SELECT execution_id FROM analysis_executions WHERE analysis_id = ?1)
			ORDER BY created_at DESC"
		))?;
		let executions = stmt.query_map([analysis_id], map_execution)?
			.collect::<rusqlite::Result<Vec<_>>>()?;
		Ok(executions)
	}

	pub fn delete_analysis(&self, id: &str) -> Result<bool> {
		let rows_affected = self.conn.execute("DELETE FROM analyses WHERE id = ?1", params![id])?;
		Ok(rows_affected > 0)
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionUpdates {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<ExecutionStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub session_id: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub thread_url: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prompt_status: Option<PromptStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prompt_result: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub validation_status: Option<ValidationStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub validation_thread_url: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub validation_result: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub files_added: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub files_removed: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub files_modified: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lines_added: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub lines_removed: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub commit_status: Option<CommitStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub commit_sha: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub committed_at: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parent_sha: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub branch: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ci_status: Option<CiStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ci_checked_at: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ci_url: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub completed_at: Option<i64>,
}
