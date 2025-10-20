use std::sync::Mutex;
use crate::db::store::Store;

#[tauri::command]
pub fn get_setting(
	key: String,
	store: tauri::State<Mutex<Store>>,
) -> Result<Option<String>, String> {
	let store = store.lock().map_err(|e| e.to_string())?;
	store.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
	key: String,
	value: String,
	store: tauri::State<Mutex<Store>>,
) -> Result<(), String> {
	let store = store.lock().map_err(|e| e.to_string())?;
	store.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_ci_stuck_threshold_minutes(
	store: tauri::State<Mutex<Store>>,
) -> Result<i64, String> {
	let store = store.lock().map_err(|e| e.to_string())?;
	store.get_ci_stuck_threshold_minutes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_max_concurrent_executions(
	store: tauri::State<Mutex<Store>>,
) -> Result<i64, String> {
	let store = store.lock().map_err(|e| e.to_string())?;
	store.get_max_concurrent_executions().map_err(|e| e.to_string())
}
