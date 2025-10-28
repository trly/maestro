use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub command: String,
    pub display_name: String,
    pub needs_terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalInfo {
    pub command: String,
    pub display_name: String,
}

#[tauri::command]
pub fn check_app_installed(command: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        // For macOS apps, check if they exist in typical locations
        match command {
            "code" => check_command_exists("code"),
            "cursor" => check_command_exists("cursor"),
            "zed" => check_command_exists("zed"),
            "vim" => check_command_exists("vim"),
            "nvim" => check_command_exists("nvim"),
            _ => check_command_exists(command),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        check_command_exists(command)
    }
}

fn check_command_exists(command: &str) -> bool {
    let mut cmd = Command::new("which");
    cmd.arg(command);

    // Set PATH to include common installation directories
    #[cfg(target_os = "macos")]
    {
        let path = std::env::var("PATH").unwrap_or_default();
        let extended_path = format!(
            "{}:/usr/local/bin:/opt/homebrew/bin:{}/.local/bin",
            path,
            std::env::var("HOME").unwrap_or_default()
        );
        cmd.env("PATH", extended_path);
    }

    cmd.output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[tauri::command]
pub fn get_available_editors() -> Vec<AppInfo> {
    let editors = vec![
        AppInfo {
            command: "vim".to_string(),
            display_name: "Vim".to_string(),
            needs_terminal: true,
        },
        AppInfo {
            command: "nvim".to_string(),
            display_name: "Neovim".to_string(),
            needs_terminal: true,
        },
        AppInfo {
            command: "code".to_string(),
            display_name: "VS Code".to_string(),
            needs_terminal: false,
        },
        AppInfo {
            command: "cursor".to_string(),
            display_name: "Cursor".to_string(),
            needs_terminal: false,
        },
        AppInfo {
            command: "zed".to_string(),
            display_name: "Zed".to_string(),
            needs_terminal: false,
        },
    ];

    editors
        .into_iter()
        .filter(|editor| check_app_installed(&editor.command))
        .collect()
}

#[tauri::command]
pub fn get_available_terminals() -> Vec<TerminalInfo> {
    #[cfg(target_os = "macos")]
    let terminals = vec![
        TerminalInfo {
            command: "open -a Terminal".to_string(),
            display_name: "Terminal".to_string(),
        },
        TerminalInfo {
            command: "ghostty".to_string(),
            display_name: "Ghostty".to_string(),
        },
    ];

    #[cfg(not(target_os = "macos"))]
    let terminals: Vec<TerminalInfo> = vec![];

    terminals
        .into_iter()
        .filter(|terminal| {
            let cmd = terminal.command.split_whitespace().next().unwrap_or("");
            check_command_exists(cmd)
        })
        .collect()
}
