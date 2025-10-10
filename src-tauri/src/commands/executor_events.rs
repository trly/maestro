use serde_json::json;
use tauri::Emitter;

pub(crate) fn emit_execution_session(app: &tauri::AppHandle, execution_id: &str, session_id: &str, thread_url: &str) {
	let _ = app.emit("execution:session", json!({
		"executionId": execution_id,
		"sessionId": session_id,
		"threadUrl": thread_url
	}));
}

pub(crate) fn emit_execution_status(app: &tauri::AppHandle, execution_id: &str, status: &str) {
	let _ = app.emit("execution:status", json!({
		"executionId": execution_id,
		"status": status
	}));
}

pub(crate) fn emit_execution_validation(app: &tauri::AppHandle, execution_id: &str, validation_status: &str, validation_thread_url: Option<&str>) {
	let payload = if let Some(url) = validation_thread_url {
		json!({
			"executionId": execution_id,
			"validationStatus": validation_status,
			"validationThreadUrl": url
		})
	} else {
		json!({
			"executionId": execution_id,
			"validationStatus": validation_status
		})
	};
	let _ = app.emit("execution:validation", payload);
}

pub(crate) fn emit_execution_commit(app: &tauri::AppHandle, execution_id: &str, commit_status: &str, commit_sha: Option<&str>, committed_at: Option<&i64>) {
	let payload = if let (Some(sha), Some(timestamp)) = (commit_sha, committed_at) {
		json!({
			"executionId": execution_id,
			"commitStatus": commit_status,
			"commitSha": sha,
			"committedAt": timestamp
		})
	} else {
		json!({
			"executionId": execution_id,
			"commitStatus": commit_status
		})
	};
	let _ = app.emit("execution:commit", payload);
}

pub(crate) fn emit_execution_progress(app: &tauri::AppHandle, execution_id: &str, message: &str) {
	let _ = app.emit("execution:progress", json!({
		"executionId": execution_id,
		"message": message
	}));
}
