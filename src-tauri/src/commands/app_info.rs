use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    name: String,
    identifier: String,
    copyright: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        identifier: "dev.trly.maestro".to_string(),
        copyright: "Copyright © 2025 Travis Lyons".to_string(),
    }
}
