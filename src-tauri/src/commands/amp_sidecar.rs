use anyhow::{Context, Result};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_shell::{process::CommandChild, process::CommandEvent, ShellExt};

pub fn node_path(app: &tauri::AppHandle) -> String {
    if let Ok(resource_dir) = app.path().resource_dir() {
        resource_dir
            .join("amp-executor/node_modules")
            .to_string_lossy()
            .to_string()
    } else {
        // Dev fallback
        std::env::current_dir()
            .ok()
            .map(|cwd| {
                cwd.join("src-tauri/resources/amp-executor/node_modules")
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_default()
    }
}

pub async fn run_amp(
    repo_path: &std::path::Path,
    prompt_text: &str,
    continue_session_id: Option<&str>,
    execution_key: Option<&str>,
    abort_flag: Option<std::sync::Arc<Mutex<bool>>>,
    on_session_start: Option<impl Fn(&str)>,
    app: &tauri::AppHandle,
) -> Result<(String, Option<String>)> {
    let mut session_id = String::new();
    let mut stdout_buf = String::new();
    let mut exit_code: Option<i32> = None;

    // Resolve the amp-executor sidecar binary
    let sidecar = app
        .shell()
        .sidecar("amp-executor")
        .context("Failed to resolve amp-executor sidecar")?;

    let mut args = vec![
        repo_path.to_string_lossy().to_string(),
        prompt_text.to_string(),
    ];

    if let Some(continue_id) = continue_session_id {
        args.push("--continue".to_string());
        args.push(continue_id.to_string());
    }

    if let Some(key) = execution_key {
        args.push("--execution-key".to_string());
        args.push(key.to_string());
    }

    // Build NODE_PATH to bundled amp-executor node_modules
    let node_path_value = node_path(app);

    let (mut rx, child) = sidecar
        .args(&args)
        .current_dir(repo_path)
        .env(
            "AMP_API_KEY",
            std::env::var("AMP_API_KEY").unwrap_or_default(),
        )
        .env("NODE_PATH", node_path_value)
        .spawn()?;

    // Store child handle for potential abort
    let child_handle: std::sync::Arc<Mutex<Option<CommandChild>>> =
        std::sync::Arc::new(Mutex::new(Some(child)));

    let child_handle_clone = child_handle.clone();
    let abort_flag_clone = abort_flag.clone();

    // Spawn abort monitor if abort_flag is provided
    if abort_flag.is_some() {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                if let Some(ref flag) = abort_flag_clone {
                    let should_abort = *flag.lock().unwrap();
                    if should_abort {
                        if let Ok(mut child_opt) = child_handle_clone.lock() {
                            if let Some(child) = child_opt.take() {
                                let _ = child.kill();
                            }
                        }
                        break;
                    }
                }
            }
        });
    }

    // Process events from the sidecar
    while let Some(event) = rx.recv().await {
        // Check abort flag
        if let Some(ref flag) = abort_flag {
            if *flag.lock().unwrap() {
                if let Ok(mut child_opt) = child_handle.lock() {
                    if let Some(child) = child_opt.take() {
                        let _ = child.kill();
                    }
                }
                anyhow::bail!("Execution aborted by user");
            }
        }

        match event {
            CommandEvent::Stdout(bytes) => {
                stdout_buf.push_str(&String::from_utf8_lossy(&bytes));
            }
            CommandEvent::Stderr(bytes) => {
                let line = String::from_utf8_lossy(&bytes);
                // Parse structured JSON messages from stderr
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                    match msg.get("type").and_then(|v| v.as_str()) {
                        Some("session_id") => {
                            if let Some(sid) = msg.get("sessionId").and_then(|v| v.as_str()) {
                                session_id = sid.to_string();
                                if let Some(ref callback) = on_session_start {
                                    callback(&session_id);
                                }
                            }
                        }
                        Some("abort") => {
                            if let Some(message) = msg.get("message").and_then(|v| v.as_str()) {
                                log::warn!("[amp-executor] Aborted: {}", message);
                            }
                        }
                        Some("error") => {
                            log::error!("[amp-executor] {}", line.trim());
                        }
                        _ => {}
                    }
                } else {
                    // Log non-JSON stderr as error
                    log::error!("[amp-executor] stderr: {}", line.trim());
                }
            }
            CommandEvent::Error(err) => {
                log::warn!("[amp-executor] stream error: {}", err);
            }
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                break;
            }
            _ => {}
        }
    }

    // Clear child handle
    {
        let mut child_opt = child_handle.lock().unwrap();
        *child_opt = None;
    }

    // Parse stdout JSON
    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout_buf) {
        if let Some(sid) = result.get("sessionId").and_then(|v| v.as_str()) {
            if session_id.is_empty() {
                session_id = sid.to_string();
            }
        }

        let result_message = result
            .get("resultMessage")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Always return error if exit code is non-zero
        let code = exit_code.unwrap_or_default();
        if code != 0 {
            let error_msg = result_message
                .as_deref()
                .unwrap_or(stdout_buf.trim())
                .to_string();
            anyhow::bail!("Amp execution failed (exit {}): {}", code, error_msg);
        }

        return Ok((session_id, result_message));
    }

    // Always return error if exit code is non-zero
    let code = exit_code.unwrap_or_default();
    if code != 0 {
        anyhow::bail!(
            "Amp execution failed with exit code {}: {}",
            code,
            stdout_buf.trim()
        );
    }

    Ok((session_id, None))
}
