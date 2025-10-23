use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
	Pending,
	Running,
	Completed,
	Failed,
	Cancelled,
}

impl FromSql for ExecutionStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"pending" => Ok(ExecutionStatus::Pending),
			"running" => Ok(ExecutionStatus::Running),
			"completed" => Ok(ExecutionStatus::Completed),
			"failed" => Ok(ExecutionStatus::Failed),
			"cancelled" => Ok(ExecutionStatus::Cancelled),
			other => Err(FromSqlError::Other(
				format!("Invalid ExecutionStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for ExecutionStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			ExecutionStatus::Pending => "pending",
			ExecutionStatus::Running => "running",
			ExecutionStatus::Completed => "completed",
			ExecutionStatus::Failed => "failed",
			ExecutionStatus::Cancelled => "cancelled",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
	Pending,
	Running,
	Passed,
	Failed,
	Cancelled,
}

impl FromSql for ValidationStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"pending" => Ok(ValidationStatus::Pending),
			"running" => Ok(ValidationStatus::Running),
			"passed" => Ok(ValidationStatus::Passed),
			"failed" => Ok(ValidationStatus::Failed),
			"cancelled" => Ok(ValidationStatus::Cancelled),
			other => Err(FromSqlError::Other(
				format!("Invalid ValidationStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for ValidationStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			ValidationStatus::Pending => "pending",
			ValidationStatus::Running => "running",
			ValidationStatus::Passed => "passed",
			ValidationStatus::Failed => "failed",
			ValidationStatus::Cancelled => "cancelled",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptStatus {
	Passed,
	Failed,
}

impl FromSql for PromptStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"passed" => Ok(PromptStatus::Passed),
			"failed" => Ok(PromptStatus::Failed),
			other => Err(FromSqlError::Other(
				format!("Invalid PromptStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for PromptStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			PromptStatus::Passed => "passed",
			PromptStatus::Failed => "failed",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommitStatus {
	None,
	Uncommitted,
	Committed,
}

impl FromSql for CommitStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"none" => Ok(CommitStatus::None),
			"uncommitted" => Ok(CommitStatus::Uncommitted),
			"committed" => Ok(CommitStatus::Committed),
			other => Err(FromSqlError::Other(
				format!("Invalid CommitStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for CommitStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			CommitStatus::None => "none",
			CommitStatus::Uncommitted => "uncommitted",
			CommitStatus::Committed => "committed",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiStatus {
	Pending,
	Passed,
	Failed,
	Skipped,
	NotConfigured,
}

impl FromSql for CiStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"pending" => Ok(CiStatus::Pending),
			"passed" => Ok(CiStatus::Passed),
			"failed" => Ok(CiStatus::Failed),
			"skipped" => Ok(CiStatus::Skipped),
			"not_configured" => Ok(CiStatus::NotConfigured),
			other => Err(FromSqlError::Other(
				format!("Invalid CiStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for CiStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			CiStatus::Pending => "pending",
			CiStatus::Passed => "passed",
			CiStatus::Failed => "failed",
			CiStatus::Skipped => "skipped",
			CiStatus::NotConfigured => "not_configured",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
	Added,
	Modified,
	Deleted,
	Renamed,
}

impl FromSql for FileStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"added" => Ok(FileStatus::Added),
			"modified" => Ok(FileStatus::Modified),
			"deleted" => Ok(FileStatus::Deleted),
			"renamed" => Ok(FileStatus::Renamed),
			other => Err(FromSqlError::Other(
				format!("Invalid FileStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for FileStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			FileStatus::Added => "added",
			FileStatus::Modified => "modified",
			FileStatus::Deleted => "deleted",
			FileStatus::Renamed => "renamed",
		};
		Ok(ToSqlOutput::from(s))
	}
}

// ============================================================================
// Analysis Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisType {
	Execution,
	Validation,
}

impl FromSql for AnalysisType {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"execution" => Ok(AnalysisType::Execution),
			"validation" => Ok(AnalysisType::Validation),
			other => Err(FromSqlError::Other(
				format!("Invalid AnalysisType: {}", other).into(),
			)),
		}
	}
}

impl ToSql for AnalysisType {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			AnalysisType::Execution => "execution",
			AnalysisType::Validation => "validation",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalysisStatus {
	Pending,
	Completed,
	Failed,
}

impl FromSql for AnalysisStatus {
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
		match value.as_str()? {
			"pending" => Ok(AnalysisStatus::Pending),
			"completed" => Ok(AnalysisStatus::Completed),
			"failed" => Ok(AnalysisStatus::Failed),
			other => Err(FromSqlError::Other(
				format!("Invalid AnalysisStatus: {}", other).into(),
			)),
		}
	}
}

impl ToSql for AnalysisStatus {
	fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
		let s = match self {
			AnalysisStatus::Pending => "pending",
			AnalysisStatus::Completed => "completed",
			AnalysisStatus::Failed => "failed",
		};
		Ok(ToSqlOutput::from(s))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
	pub id: String,
	pub revision_id: String,
	#[serde(rename = "type")]
	pub analysis_type: AnalysisType,
	pub status: AnalysisStatus,
	pub analysis_prompt: String,
	pub analysis_result: Option<String>,
	pub amp_thread_url: Option<String>,
	pub amp_session_id: Option<String>,
	pub error_message: Option<String>,
	pub execution_count: i64,
	pub created_at: i64,
	pub updated_at: i64,
	pub completed_at: Option<i64>,
}
