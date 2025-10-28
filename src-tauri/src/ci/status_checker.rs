use anyhow::{Context, Result};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::time::{sleep, Duration};

use crate::ci::{CiProvider, CiContext};
use crate::commands::executor_events;
use crate::db::store::{Store, ExecutionUpdates};
use std::sync::Mutex;
use crate::types::CiStatus;

/// Check CI once and return aggregated status
pub async fn check_ci_once(
	provider: Arc<dyn CiProvider>,
	ctx: CiContext,
) -> Result<(Option<CiStatus>, Option<String>)> {
	// Always provide the commit checks URL for user clickthrough
	let ci_url = Some(format!(
		"https://github.com/{}/{}/commit/{}/checks",
		ctx.owner, ctx.repo, ctx.commit_sha
	));
	
	let checks = provider.poll(&ctx).await?;
	
	if checks.is_empty() {
		return Ok((None, ci_url));
	}
	
	// Aggregate all checks into overall status
	let mut has_pending = false;
	let mut has_failed = false;
	let mut has_passed = false;
	
	for check in &checks {
		match check.state {
			CiStatus::Pending => has_pending = true,
			CiStatus::Failed => has_failed = true,
			CiStatus::Passed => has_passed = true,
			CiStatus::Skipped => {}, // Skipped doesn't affect overall status
			CiStatus::NotConfigured => {}, // Not configured doesn't affect overall status
			CiStatus::NotPushed => {}, // Not pushed doesn't affect overall status
		}
	}
	
	let overall_status = if has_failed {
		CiStatus::Failed
	} else if has_pending {
		CiStatus::Pending
	} else if has_passed {
		CiStatus::Passed
	} else {
		CiStatus::Skipped
	};
	
	Ok((Some(overall_status), ci_url))
}

/// Poll CI until terminal state (passed/failed/skipped) with exponential backoff
pub async fn poll_ci_until_terminal(
	provider: Arc<dyn CiProvider>,
	ctx: CiContext,
	execution_id: String,
	app: AppHandle,
) -> Result<()> {
	let backoff_delays: &[u64] = &[10, 20, 40, 80, 120]; // seconds
	let mut attempts = 0;
	
	for (i, delay_secs) in backoff_delays.iter().enumerate() {
		attempts += 1;
		
		log::info!("CI check attempt {} for execution {}", attempts, execution_id);
		
		match check_ci_once(provider.clone(), ctx.clone()).await {
			Ok((Some(status), ci_url)) => {
				// Update database
				let now = chrono::Utc::now().timestamp_millis();
				let store = app.state::<Mutex<Store>>();
				let _ = store.lock().unwrap().update_execution(
					&execution_id,
					ExecutionUpdates {
						ci_status: Some(status),
						ci_checked_at: Some(now),
						ci_url: ci_url.clone(),
						..Default::default()
					},
				);
				
				// Emit event (serialize status properly using serde)
				let status_str = serde_json::to_value(status)
					.ok()
					.and_then(|v| v.as_str().map(|s| s.to_string()))
					.unwrap_or_else(|| format!("{:?}", status).to_lowercase());
				
				executor_events::emit_execution_ci(
					&app,
					&execution_id,
					&status_str,
					ci_url.as_deref(),
				);
				
				// Check if terminal
				match status {
				CiStatus::Passed | CiStatus::Failed | CiStatus::Skipped | CiStatus::NotConfigured | CiStatus::NotPushed => {
				log::info!(
				"CI reached terminal state {:?} for execution {}",
				status,
				execution_id
				);
				return Ok(());
				}
				CiStatus::Pending => {
				log::info!("CI still pending for execution {}, will retry", execution_id);
				}
				}
			}
			Ok((None, ci_url)) => {
				log::info!(
					"No CI checks found for execution {} (attempt {})",
					execution_id,
					attempts
				);
				
				// Only mark skipped at the final attempt
				if i == backoff_delays.len() - 1 {
					let now = chrono::Utc::now().timestamp_millis();
					let store = app.state::<Mutex<Store>>();
					let _ = store.lock().unwrap().update_execution(
						&execution_id,
						ExecutionUpdates {
							ci_status: Some(CiStatus::Skipped),
							ci_checked_at: Some(now),
							ci_url: ci_url.clone(),
							..Default::default()
						},
					);
					
					executor_events::emit_execution_ci(&app, &execution_id, "skipped", ci_url.as_deref());
					
					log::info!("No CI configured for execution {}, marking as skipped", execution_id);
					return Ok(());
				}
			}
			Err(e) => {
				log::warn!("Error checking CI for execution {}: {:?}", execution_id, e);
				
				// Only mark skipped if we exhaust attempts
				if i == backoff_delays.len() - 1 {
					let now = chrono::Utc::now().timestamp_millis();
					let store = app.state::<Mutex<Store>>();
					let _ = store.lock().unwrap().update_execution(
						&execution_id,
						ExecutionUpdates {
							ci_status: Some(CiStatus::Skipped),
							ci_checked_at: Some(now),
							..Default::default()
						},
					);
					
					executor_events::emit_execution_ci(&app, &execution_id, "skipped", None);
					
					return Err(e).context(format!("Failed to check CI for execution {}", execution_id));
				}
			}
		}
		
		// Wait before next attempt (unless this was the last one)
		if i < backoff_delays.len() - 1 {
			log::info!("Waiting {}s before next CI check for execution {}", delay_secs, execution_id);
			sleep(Duration::from_secs(*delay_secs)).await;
		}
	}
	
	// If we exhausted all attempts and never reached terminal state, log it
	log::warn!(
		"CI polling exhausted for execution {} after {} attempts, leaving in current state",
		execution_id,
		attempts
	);
	
	Ok(())
}
