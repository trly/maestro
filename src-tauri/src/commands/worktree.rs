use std::process::Command;
use crate::util::paths::execution_worktree_path;
use crate::Paths;

#[tauri::command]
pub fn open_worktree_in_editor(
	promptset_id: String,
	execution_id: String,
	editor_command: String,
	paths: tauri::State<'_, Paths>
) -> Result<(), String> {
	let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);
	
	Command::new(&editor_command)
		.arg(worktree_path)
		.spawn()
		.map_err(|e| format!("Failed to launch editor '{}': {}", editor_command, e))?;
	
	Ok(())
}
