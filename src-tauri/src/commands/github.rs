use serde::{Deserialize, Serialize};
use crate::Paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPaths {
	pub admin_repo_dir: String,
	pub worktree_dir: String,
	pub db_path: String,
}

#[tauri::command]
pub fn get_config_paths(paths: tauri::State<'_, Paths>) -> Result<ConfigPaths, String> {
	Ok(ConfigPaths {
		admin_repo_dir: paths.admin_repo_dir.to_string_lossy().to_string(),
		worktree_dir: paths.worktree_dir.to_string_lossy().to_string(),
		db_path: paths.db_path.to_string_lossy().to_string(),
	})
}
