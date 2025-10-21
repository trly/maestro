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

pub(crate) fn emit_execution_ci(app: &tauri::AppHandle, execution_id: &str, ci_status: &str, ci_url: Option<&str>) {
	let payload = if let Some(url) = ci_url {
		json!({
			"executionId": execution_id,
			"ciStatus": ci_status,
			"ciUrl": url
		})
	} else {
		json!({
			"executionId": execution_id,
			"ciStatus": ci_status
		})
	};
	let _ = app.emit("execution:ci", payload);
}

pub(crate) fn emit_analysis_status(app: &tauri::AppHandle, analysis_id: &str, status: &str, error_message: Option<&str>) {
	let payload = if let Some(msg) = error_message {
		json!({
			"analysisId": analysis_id,
			"status": status,
			"errorMessage": msg
		})
	} else {
		json!({
			"analysisId": analysis_id,
			"status": status
		})
	};
	let _ = app.emit("analysis:status", payload);
}

pub(crate) fn emit_analysis_result(
	app: &tauri::AppHandle, 
	analysis_id: &str, 
	result: &str, 
	amp_thread_url: Option<&str>,
	completed_at: i64,
) {
	let payload = if let Some(url) = amp_thread_url {
		json!({
			"analysisId": analysis_id,
			"analysisResult": result,
			"ampThreadUrl": url,
			"completedAt": completed_at
		})
	} else {
		json!({
			"analysisId": analysis_id,
			"analysisResult": result,
			"completedAt": completed_at
		})
	};
	let _ = app.emit("analysis:result", payload);
}
