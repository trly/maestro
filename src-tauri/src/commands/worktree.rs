use crate::util::paths::execution_worktree_path;
use crate::Paths;
use std::process::Command;

#[tauri::command]
pub fn open_worktree_in_editor(
    promptset_id: String,
    execution_id: String,
    editor_command: String,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);

    Command::new(&editor_command)
        .arg(worktree_path)
        .spawn()
        .map_err(|e| format!("Failed to launch editor '{}': {}", editor_command, e))?;

    Ok(())
}

#[tauri::command]
pub fn open_worktree_with_terminal(
    promptset_id: String,
    execution_id: String,
    editor_command: String,
    terminal_command: String,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);
    let worktree_str = worktree_path
        .to_str()
        .ok_or_else(|| "Invalid worktree path".to_string())?;

    // Parse terminal command (may have args like "open -a Terminal")
    let terminal_parts: Vec<&str> = terminal_command.split_whitespace().collect();
    if terminal_parts.is_empty() {
        return Err("Empty terminal command".to_string());
    }

    let terminal_cmd = terminal_parts[0];
    let terminal_args = &terminal_parts[1..];

    // Terminal launch strategies
    match terminal_cmd {
        "ghostty" => {
            Command::new("ghostty")
                .arg("-e")
                .arg(&editor_command)
                .arg(worktree_str)
                .spawn()
                .map_err(|e| format!("Failed to launch Ghostty: {}", e))?;
        }
        "open" if terminal_args.contains(&"-a") && terminal_args.contains(&"Terminal") => {
            // macOS Terminal.app
            let shell_script = format!(
                "cd {} && exec {}",
                shell_escape::escape(worktree_str.into()),
                editor_command
            );

            let applescript = format!(
                "tell application \"Terminal\"\n\
				 \tactivate\n\
				 \tdo script \"{}\"\n\
				 end tell",
                shell_script.replace("\\", "\\\\").replace("\"", "\\\\\"")
            );

            Command::new("osascript")
                .arg("-e")
                .arg(applescript)
                .spawn()
                .map_err(|e| format!("Failed to launch Terminal.app: {}", e))?;
        }
        _ => {
            return Err(format!("Unsupported terminal: {}", terminal_command));
        }
    }

    Ok(())
}
