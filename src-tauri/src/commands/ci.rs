use std::sync::Mutex;
use tauri::AppHandle;

use crate::ci::{CiContext, check_ci_once};
use crate::commands::executor_events;
use crate::db::store::{Store, ExecutionUpdates};
use crate::types::CiStatus;
use crate::util::paths::admin_repo_path;
use crate::Paths;

/// Start CI checking for an execution (spawns background polling)
#[tauri::command]
pub async fn start_ci_check(
	execution_id: String,
	app: AppHandle,
	store: tauri::State<'_, Mutex<Store>>,
	paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
	// Get execution details
	let execution = {
		let store = store.lock().map_err(|e| e.to_string())?;
		store
			.get_execution(&execution_id)
			.map_err(|e| e.to_string())?
			.ok_or_else(|| format!("Execution {} not found", execution_id))?
	};
	
	// Ensure execution has been committed
	let commit_sha = execution
		.commit_sha
		.ok_or_else(|| "Execution must be committed before checking CI".to_string())?;
	
	let branch = execution
		.branch
		.ok_or_else(|| "Execution branch not found".to_string())?;
	
	// Get repository details
	let repository = {
		let store = store.lock().map_err(|e| e.to_string())?;
		store
			.get_repository(&execution.repository_id)
			.map_err(|e| e.to_string())?
			.ok_or_else(|| format!("Repository {} not found", execution.repository_id))?
	};
	
	// Parse owner/repo from provider_id
	let (owner, repo_name) = crate::util::git::parse_provider_id(&repository.provider_id)
		.map_err(|e| format!("Failed to parse provider ID: {}", e))?;
	
	// Check if commit has been pushed to remote
	let repo_path = admin_repo_path(&paths, &owner, &repo_name);
	let is_pushed = crate::git::is_commit_pushed(&repo_path, &commit_sha)
	.unwrap_or(false);

	if !is_pushed {
		// Set status to not_pushed and return
		let store = store.lock().unwrap();
		store.update_execution(
			&execution_id,
			ExecutionUpdates {
				ci_status: Some(CiStatus::NotPushed),
				..Default::default()
			},
		).map_err(|e| format!("Failed to update status: {}", e))?;
		
		executor_events::emit_execution_ci(&app, &execution_id, "not_pushed", None);
		return Ok(());
	}
	
	// Create CI provider using the provider trait
	let provider = crate::ci::provider::create_ci_provider(&repository.provider, &repository.provider_id)
		.await
		.map_err(|e| format!("Failed to create CI provider: {}", e))?;
	
	let ctx = CiContext {
		owner,
		repo: repo_name,
		commit_sha: commit_sha.clone(),
		branch: branch.clone(),
		provider_cfg: serde_json::json!({}),
	};
	
	// Get initial CI URL from provider
	let ci_url = provider.get_commit_url(&commit_sha);
	
	// Emit initial pending status
	executor_events::emit_execution_ci(&app, &execution_id, "pending", Some(&ci_url));
	
	// Update database with pending status
	{
		let store = store.lock().map_err(|e| e.to_string())?;
		let now = chrono::Utc::now().timestamp_millis();
		store
			.update_execution(
				&execution_id,
				ExecutionUpdates {
					ci_status: Some(CiStatus::Pending),
					ci_checked_at: Some(now),
					ci_url: Some(ci_url.clone()),
					..Default::default()
				},
			)
			.map_err(|e| e.to_string())?;
	}
	
	// Spawn background polling task
	let app_clone = app.clone();
	let exec_id = execution_id.clone();
	tokio::spawn(async move {
		if let Err(e) = crate::ci::poll_ci_until_terminal(
			provider,
			ctx,
			exec_id,
			app_clone,
		).await {
			log::warn!("CI polling error: {:?}", e);
		}
	});
	
	Ok(())
}

/// Refresh CI status once (no polling)
#[tauri::command]
pub async fn refresh_ci_status(
	execution_id: String,
	app: AppHandle,
	store: tauri::State<'_, Mutex<Store>>,
	paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
	// Get execution details
	let execution = {
		let store = store.lock().map_err(|e| e.to_string())?;
		store
			.get_execution(&execution_id)
			.map_err(|e| e.to_string())?
			.ok_or_else(|| format!("Execution {} not found", execution_id))?
	};
	
	// Ensure execution has been committed
	let commit_sha = execution
		.commit_sha
		.ok_or_else(|| "Execution must be committed before checking CI".to_string())?;
	
	let branch = execution
		.branch
		.unwrap_or_else(|| "main".to_string());
	
	// Get repository details
	let repository = {
		let store = store.lock().map_err(|e| e.to_string())?;
		store
			.get_repository(&execution.repository_id)
			.map_err(|e| e.to_string())?
			.ok_or_else(|| format!("Repository {} not found", execution.repository_id))?
	};
	
	// Parse owner/repo from provider_id
	let (owner, repo_name) = crate::util::git::parse_provider_id(&repository.provider_id)
		.map_err(|e| format!("Failed to parse provider ID: {}", e))?;
	
	// Check if commit has been pushed to remote
	let repo_path = admin_repo_path(&paths, &owner, &repo_name);
	let is_pushed = crate::git::is_commit_pushed(&repo_path, &commit_sha)
	.unwrap_or(false);

	if !is_pushed {
		// Set status to not_pushed and return
		let store = store.lock().unwrap();
		store.update_execution(
			&execution_id,
			ExecutionUpdates {
				ci_status: Some(CiStatus::NotPushed),
				..Default::default()
			},
		).map_err(|e| format!("Failed to update status: {}", e))?;
		
		executor_events::emit_execution_ci(&app, &execution_id, "not_pushed", None);
		return Ok(());
	}

	// Create CI provider using the provider trait
	let provider = crate::ci::provider::create_ci_provider(&repository.provider, &repository.provider_id)
		.await
		.map_err(|e| format!("Failed to create CI provider: {}", e))?;
	
	let ctx = CiContext {
		owner,
		repo: repo_name,
		commit_sha: commit_sha.clone(),
		branch,
		provider_cfg: serde_json::json!({}),
	};
	
	// Check CI once
	let (status_opt, ci_url_opt) = check_ci_once(provider, ctx)
		.await
		.map_err(|e| e.to_string())?;
	
	let status = status_opt.unwrap_or(CiStatus::NotConfigured);
	
	// Update database
	let now = chrono::Utc::now().timestamp_millis();
	{
		let store = store.lock().map_err(|e| e.to_string())?;
		store
			.update_execution(
				&execution_id,
				ExecutionUpdates {
					ci_status: Some(status),
					ci_checked_at: Some(now),
					ci_url: ci_url_opt.clone(),
					..Default::default()
				},
			)
			.map_err(|e| e.to_string())?;
	}
	
	// Emit event (serialize status properly using serde)
	let status_str = serde_json::to_value(status)
		.ok()
		.and_then(|v| v.as_str().map(|s| s.to_string()))
		.unwrap_or_else(|| format!("{:?}", status).to_lowercase());
	
	executor_events::emit_execution_ci(
		&app,
		&execution_id,
		&status_str,
		ci_url_opt.as_deref(),
	);
	
	Ok(())
}
