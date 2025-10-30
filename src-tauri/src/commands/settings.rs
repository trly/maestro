use crate::db::store::Store;
use std::sync::Mutex;

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
pub fn get_ci_stuck_threshold_minutes(store: tauri::State<Mutex<Store>>) -> Result<i64, String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    store
        .get_ci_stuck_threshold_minutes()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_max_concurrent_executions(store: tauri::State<Mutex<Store>>) -> Result<i64, String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    store
        .get_max_concurrent_executions()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_first_run_complete(store: tauri::State<Mutex<Store>>) -> Result<bool, String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    match store.get_setting("first_run_complete") {
        Ok(Some(value)) => Ok(value == "true"),
        Ok(None) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_first_run_complete(store: tauri::State<Mutex<Store>>) -> Result<(), String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    store
        .set_setting("first_run_complete", "true")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_show_first_run_dialog(store: tauri::State<Mutex<Store>>) -> Result<bool, String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    match store.get_setting("show_first_run_dialog") {
        Ok(Some(value)) => Ok(value == "true"),
        Ok(None) => Ok(true), // Default to showing the dialog
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn set_show_first_run_dialog(
    enabled: bool,
    store: tauri::State<Mutex<Store>>,
) -> Result<(), String> {
    let store = store.lock().map_err(|e| e.to_string())?;
    store
        .set_setting(
            "show_first_run_dialog",
            if enabled { "true" } else { "false" },
        )
        .map_err(|e| e.to_string())
}
