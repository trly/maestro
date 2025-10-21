use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use anyhow::Result;
use uuid::Uuid;

use crate::db::store::Store;
use crate::types::{Analysis, AnalysisType, AnalysisStatus};
use crate::amp::v2_client::AmpV2Client;
use crate::commands::tokens::get_token_value;
use crate::commands::executor_events::{emit_analysis_status, emit_analysis_result};
use crate::Paths;

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[tauri::command]
pub fn create_analysis(
    revision_id: String,
    analysis_type: String,
    execution_ids: Vec<String>,
    store: State<Mutex<Store>>,
) -> Result<String, String> {
    let mut store = store.lock().unwrap();
    
    let analysis_type_enum = match analysis_type.as_str() {
        "execution" => AnalysisType::Execution,
        "validation" => AnalysisType::Validation,
        _ => return Err(format!("Invalid analysis type: {}", analysis_type)),
    };
    
    let analysis_id = Uuid::new_v4().to_string();
    
    // Prompt will be generated and stored during run_analysis_impl
    store.create_analysis(
        &analysis_id,
        &revision_id,
        analysis_type_enum,
        "",  // Empty placeholder - actual prompt generated with thread data
    ).map_err(|e| format!("Failed to create analysis: {}", e))?;
    
    store.add_analysis_executions(&analysis_id, execution_ids)
        .map_err(|e| format!("Failed to link executions to analysis: {}", e))?;
    
    Ok(analysis_id)
}

#[tauri::command]
pub async fn run_analysis(
    analysis_id: String,
    app: AppHandle,
    _store: State<'_, Mutex<Store>>,
    _paths: State<'_, Paths>,
) -> Result<(), String> {
    let app_clone = app.clone();
    let analysis_id_clone = analysis_id.clone();
    
    tokio::spawn(async move {
        let app_for_error = app_clone.clone();
        if let Err(e) = run_analysis_impl(analysis_id_clone.clone(), app_clone).await {
            log::error!("[run_analysis] Analysis {} failed: {}", analysis_id_clone, e);
            
            let error_msg = e.to_string();
            let store_state = app_for_error.state::<Mutex<Store>>();
            let store = store_state.lock().unwrap();
            let _ = store.update_analysis_status(
                &analysis_id_clone,
                AnalysisStatus::Failed,
                Some(error_msg.clone()),
            );
            
            // Emit failure event
            emit_analysis_status(&app_for_error, &analysis_id_clone, "failed", Some(&error_msg));
        }
    });
    
    Ok(())
}

async fn run_analysis_impl(
    analysis_id: String,
    app: AppHandle,
) -> Result<()> {
    
    let (analysis_type, execution_ids) = {
        let store_state = app.state::<Mutex<Store>>();
        let store_guard = store_state.lock().unwrap();
        let analysis = store_guard.get_analysis(&analysis_id)?
            .ok_or_else(|| anyhow::anyhow!("Analysis not found"))?;
        
        let executions = store_guard.get_analysis_executions(&analysis_id)?;
        let execution_ids: Vec<String> = executions.iter().map(|e| e.id.clone()).collect();
        
        (analysis.analysis_type, execution_ids)
    };
    
    let client_id = get_token_value("amp_client_id")
        .map_err(|e| anyhow::anyhow!("Failed to get amp_client_id: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("amp_client_id not configured"))?;
    let client_secret = get_token_value("amp_client_secret")
        .map_err(|e| anyhow::anyhow!("Failed to get amp_client_secret: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("amp_client_secret not configured"))?;
    
    let mut amp_client = AmpV2Client::new(client_id, client_secret);
    
    let mut formatted_threads = String::new();
    
    for execution_id in &execution_ids {
        let thread_url = {
            let store_state = app.state::<Mutex<Store>>();
            let store_guard = store_state.lock().unwrap();
            let execution = store_guard.get_execution(execution_id)?
                .ok_or_else(|| anyhow::anyhow!("Execution not found: {}", execution_id))?;
            
            match analysis_type {
                AnalysisType::Execution => execution.thread_url,
                AnalysisType::Validation => execution.validation_thread_url,
            }
        };
        
        if let Some(url) = thread_url {
            if let Some(thread_id) = AmpV2Client::extract_thread_id(&url) {
                match amp_client.get_thread_messages(&thread_id).await {
                    Ok(messages) => {
                        formatted_threads.push_str(&format!("\n\n## Execution: {}\n", execution_id));
                        formatted_threads.push_str(&format!("Thread URL: {}\n", url));
                        formatted_threads.push_str(&AmpV2Client::format_messages_for_analysis(&messages));
                    }
                    Err(e) => {
                        log::warn!("[run_analysis] Failed to fetch thread {}: {}", thread_id, e);
                        formatted_threads.push_str(&format!("\n\n## Execution: {} (failed to fetch)\n", execution_id));
                    }
                }
            }
        }
    }
    
    let analysis_prompt = format!(
        "Analyze the following failed {} threads and categorize common failure patterns.\n\n\
        As part of the analysis, provide suggested changes to the prompt to reduce these failures over a larger set of similar repositories.\n\n
        IMPORTANT: Do NOT write any analysis to disk. Return your failure categorization as a markdown table.\n\n\
        {}",
        match analysis_type {
            AnalysisType::Execution => "execution",
            AnalysisType::Validation => "validation",
        },
        formatted_threads
    );
    
    // Store the generated prompt in the database
    {
        let store_state = app.state::<Mutex<Store>>();
        let store_guard = store_state.lock().unwrap();
        store_guard.update_analysis_prompt(&analysis_id, &analysis_prompt)?;
    }
    
    let temp_dir = std::env::temp_dir().join(format!("maestro-analysis-{}", analysis_id));
    std::fs::create_dir_all(&temp_dir)?;
    
    let execution_result = execute_analysis_with_amp(&temp_dir, &analysis_prompt, &app).await;
    
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    match execution_result {
        Ok((session_id, result)) => {
            let amp_thread_url = format!("https://ampcode.com/threads/{}", session_id);
            let completed_at = now_ms();
            
            let store_state = app.state::<Mutex<Store>>();
            let store_guard = store_state.lock().unwrap();
            store_guard.update_analysis_result(
                &analysis_id,
                &result,
                Some(amp_thread_url.clone()),
                Some(session_id),
                completed_at,
            )?;
            store_guard.update_analysis_status(&analysis_id, AnalysisStatus::Completed, None)?;
            
            // Emit success events
            emit_analysis_result(&app, &analysis_id, &result, Some(&amp_thread_url), completed_at);
            emit_analysis_status(&app, &analysis_id, "completed", None);
        }
        Err(e) => {
            log::error!("[run_analysis] Amp execution failed: {}", e);
            
            let error_msg = e.to_string();
            let store_state = app.state::<Mutex<Store>>();
            let store_guard = store_state.lock().unwrap();
            store_guard.update_analysis_status(
                &analysis_id,
                AnalysisStatus::Failed,
                Some(error_msg.clone()),
            )?;
            
            // Emit failure event
            emit_analysis_status(&app, &analysis_id, "failed", Some(&error_msg));
            
            return Err(e);
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_analysis(
    analysis_id: String,
    store: State<Mutex<Store>>,
) -> Result<Option<Analysis>, String> {
    let store = store.lock().unwrap();
    store.get_analysis(&analysis_id)
        .map_err(|e| format!("Failed to get analysis: {}", e))
}

#[tauri::command]
pub fn get_analyses_by_revision(
    revision_id: String,
    analysis_type: Option<String>,
    store: State<Mutex<Store>>,
) -> Result<Vec<Analysis>, String> {
    let store = store.lock().unwrap();
    
    let analysis_type_enum = if let Some(atype) = analysis_type {
        let type_enum = match atype.as_str() {
            "execution" => AnalysisType::Execution,
            "validation" => AnalysisType::Validation,
            _ => return Err(format!("Invalid analysis type: {}", atype)),
        };
        Some(type_enum)
    } else {
        None
    };
    
    store.get_analyses_by_revision(&revision_id, analysis_type_enum)
        .map_err(|e| format!("Failed to get analyses: {}", e))
}

#[tauri::command]
pub fn delete_analysis(
    analysis_id: String,
    store: State<Mutex<Store>>,
) -> Result<bool, String> {
    let store = store.lock().unwrap();
    store.delete_analysis(&analysis_id)
        .map_err(|e| format!("Failed to delete analysis: {}", e))
}

async fn execute_analysis_with_amp(
    repo_path: &std::path::Path,
    prompt_text: &str,
    app: &AppHandle,
) -> Result<(String, String)> {
    use std::process::{Command, Stdio};
    use std::io::BufRead;
    
    let mut session_id = String::new();
    
    let bun_path = which::which("bun")?;
    
    let executor_script = if cfg!(debug_assertions) {
        let mut workspace_root = std::env::current_dir()?;
        if workspace_root.ends_with("src-tauri") {
            workspace_root = workspace_root.parent()
                .ok_or_else(|| anyhow::anyhow!("Failed to get workspace root"))?
                .to_path_buf();
        }
        workspace_root.join("src").join("lib").join("amp-executor.ts")
    } else {
        app.path()
            .resource_dir()?
            .join("src/lib/amp-executor.ts")
    };
    
    if !executor_script.exists() {
        anyhow::bail!("amp-executor.ts not found at {:?}", executor_script);
    }
    
    let args = vec![
        "run".to_string(),
        executor_script.to_string_lossy().to_string(),
        repo_path.to_string_lossy().to_string(),
        prompt_text.to_string(),
    ];
    
    let mut child = Command::new(&bun_path)
        .args(&args)
        .env("AMP_API_KEY", std::env::var("AMP_API_KEY").unwrap_or_default())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    
    let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = String::new();
        let _ = std::io::BufReader::new(stdout).read_to_string(&mut buf);
        let _ = stdout_tx.send(buf);
    });
    
    let stderr_reader = std::io::BufReader::new(stderr);
    for line in stderr_reader.lines() {
        let line = line?;
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
            if msg.get("type") == Some(&serde_json::json!("session_id")) {
                if let Some(sid) = msg.get("sessionId").and_then(|v| v.as_str()) {
                    session_id = sid.to_string();
                }
            }
        }
    }
    
    let stdout_content = stdout_rx.recv().unwrap_or_default();
    
    let status = child.wait()?;
    
    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout_content) {
        if let Some(sid) = result.get("sessionId").and_then(|v| v.as_str()) {
            if session_id.is_empty() {
                session_id = sid.to_string();
            }
        }
        
        let result_message = result.get("resultMessage")
            .and_then(|v| v.as_str())
            .unwrap_or(&stdout_content)
            .to_string();
        
        if !status.success() && session_id.is_empty() {
            anyhow::bail!("Amp execution failed with exit code {}: {}", status.code().unwrap_or(-1), result_message);
        }
        
        return Ok((session_id, result_message));
    }
    
    if !status.success() {
        anyhow::bail!("Amp execution failed with exit code {}: {}", status.code().unwrap_or(-1), stdout_content);
    }
    
    Ok((session_id, stdout_content))
}