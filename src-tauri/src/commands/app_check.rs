use serde::{Deserialize, Serialize};

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
    which::which(command).is_ok()
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
            check_app_installed(cmd)
        })
        .collect()
}
