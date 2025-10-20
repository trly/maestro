use std::sync::Mutex;
use tauri::Manager;

use crate::db::store::Store;
use crate::types::ExecutionStatus;

lazy_static::lazy_static! {
	static ref POLLING_ACTIVE: Mutex<bool> = Mutex::new(false);
	static ref PENDING_QUEUE: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// Start polling for pending executions
#[tauri::command]
pub async fn start_execution_polling(
	app: tauri::AppHandle,
	_store: tauri::State<'_, Mutex<Store>>,
) -> Result<(), String> {
	let mut polling_active = POLLING_ACTIVE.lock().map_err(|e| e.to_string())?;
	
	if *polling_active {
		return Err("Polling is already active".to_string());
	}
	
	*polling_active = true;
	drop(polling_active);
	
	tokio::spawn(async move {
		poll_executions_loop(app).await;
	});
	
	Ok(())
}

/// Stop polling for pending executions
#[tauri::command]
pub async fn stop_execution_polling() -> Result<(), String> {
	let mut polling_active = POLLING_ACTIVE.lock().map_err(|e| e.to_string())?;
	*polling_active = false;
	Ok(())
}

/// Get pending executions count
#[tauri::command]
pub fn get_pending_executions_count(
	store: tauri::State<'_, Mutex<Store>>,
) -> Result<usize, String> {
	let store = store.lock().map_err(|e| e.to_string())?;
	let pending = store.get_all_executions()
		.map_err(|e| e.to_string())?
		.into_iter()
		.filter(|e| e.status == ExecutionStatus::Pending)
		.count();
	Ok(pending)
}

async fn poll_executions_loop(app: tauri::AppHandle) {
	loop {
		// Check if polling should continue
		{
			let polling_active = POLLING_ACTIVE.lock().unwrap();
			if !*polling_active {
				break;
			}
		}
		
		// Get max concurrent executions from settings
		let max_concurrent = {
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();
			store.get_max_concurrent_executions().unwrap_or(10)
		};
		
		// Get currently running executions count
		let running_count = {
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();
			match store.get_all_executions() {
				Ok(execs) => execs.into_iter()
					.filter(|e| e.status == ExecutionStatus::Running)
					.count(),
				Err(e) => {
					log::error!("Failed to get executions: {}", e);
					0
				}
			}
		};
		
		// If we have capacity, start pending executions
		let available_slots = max_concurrent.saturating_sub(running_count as i64);
		
		if available_slots > 0 {
			// Get pending executions
			let pending_executions = {
				let store_state = app.state::<Mutex<Store>>();
				let store = store_state.lock().unwrap();
				match store.get_all_executions() {
					Ok(execs) => execs.into_iter()
						.filter(|e| e.status == ExecutionStatus::Pending)
						.take(available_slots as usize)
						.collect::<Vec<_>>(),
					Err(e) => {
						log::error!("Failed to get pending executions: {}", e);
						Vec::new()
					}
				}
			};
			
			// Start executions
			for execution in pending_executions {
				let execution_id = execution.id.clone();
				let app_clone = app.clone();
				let paths_state = app.state::<crate::Paths>();
				let paths_clone = paths_state.inner().clone();
				
				log::info!("[execution_poller] Starting execution: {}", execution_id);
				
				tokio::spawn(async move {
					if let Err(e) = super::executor::execute_prompt_impl(
						execution_id.clone(),
						app_clone,
						paths_clone
					).await {
						log::error!("[execution_poller] Execution {} failed: {}", execution_id, e);
					}
				});
			}
		}
		
		// Sleep before next poll (5 seconds)
		tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
	}
	
	log::info!("[execution_poller] Polling stopped");
}
