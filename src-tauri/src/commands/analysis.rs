use anyhow::Result;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use super::executor::execute_with_amp;
use crate::commands::executor_events::{emit_analysis_result, emit_analysis_status};
use crate::db::store::Store;
use crate::types::{Analysis, AnalysisStatus, AnalysisType};
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
    store
        .create_analysis(
            &analysis_id,
            &revision_id,
            analysis_type_enum,
            "", // Empty placeholder - actual prompt generated with thread data
        )
        .map_err(|e| format!("Failed to create analysis: {}", e))?;

    store
        .add_analysis_executions(&analysis_id, execution_ids)
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
            log::error!(
                "[run_analysis] Analysis {} failed: {}",
                analysis_id_clone,
                e
            );

            let error_msg = e.to_string();
            let store_state = app_for_error.state::<Mutex<Store>>();
            let store = store_state.lock().unwrap();
            let _ = store.update_analysis_status(
                &analysis_id_clone,
                AnalysisStatus::Failed,
                Some(error_msg.clone()),
            );

            // Emit failure event
            emit_analysis_status(
                &app_for_error,
                &analysis_id_clone,
                "failed",
                Some(&error_msg),
            );
        }
    });

    Ok(())
}

async fn run_analysis_impl(analysis_id: String, app: AppHandle) -> Result<()> {
    let (analysis_type, execution_ids) = {
        let store_state = app.state::<Mutex<Store>>();
        let store_guard = store_state.lock().unwrap();
        let analysis = store_guard
            .get_analysis(&analysis_id)?
            .ok_or_else(|| anyhow::anyhow!("Analysis not found"))?;

        let executions = store_guard.get_analysis_executions(&analysis_id)?;
        let execution_ids: Vec<String> = executions.iter().map(|e| e.id.clone()).collect();

        (analysis.analysis_type, execution_ids)
    };

    let mut thread_list = String::new();

    for execution_id in &execution_ids {
        let thread_url = {
            let store_state = app.state::<Mutex<Store>>();
            let store_guard = store_state.lock().unwrap();
            let execution = store_guard
                .get_execution(execution_id)?
                .ok_or_else(|| anyhow::anyhow!("Execution not found: {}", execution_id))?;

            match analysis_type {
                AnalysisType::Execution => execution.thread_url,
                AnalysisType::Validation => execution.validation_thread_url,
            }
        };

        if let Some(url) = thread_url {
            thread_list.push_str(&format!("\n- {} ({})", url, execution_id));
        }
    }

    let analysis_prompt = format!(
        "You are analyzing failed {} threads to identify common failure patterns.\n\n\
        STEP 1: For EACH thread URL listed below, use the read_thread tool to extract:\n\
        - The original task/goal that was attempted\n\
        - All error messages and stack traces\n\
        - The failure cause and context\n\
        - Any relevant tool outputs or file contents\n\n\
        Thread URLs to analyze:{}\n\n\
        STEP 2: After reading ALL threads, create a comprehensive analysis that includes:\n\
        1. A markdown table categorizing failure patterns with columns: Pattern | Count | Example Thread | Root Cause\n\
        2. Specific, actionable suggestions to modify the original prompt to prevent these failures\n\
        3. Any common environmental or setup issues discovered\n\n\
        IMPORTANT:\n\
        - You MUST use read_thread for every URL listed above\n\
        - Do NOT write any files to disk\n\
        - Return your complete analysis as markdown\n\
        - Be specific about which execution IDs exhibited which patterns",
        match analysis_type {
            AnalysisType::Execution => "execution",
            AnalysisType::Validation => "validation",
        },
        thread_list
    );

    // Store the generated prompt in the database
    {
        let store_state = app.state::<Mutex<Store>>();
        let store_guard = store_state.lock().unwrap();
        store_guard.update_analysis_prompt(&analysis_id, &analysis_prompt)?;
    }

    let temp_dir = std::env::temp_dir().join(format!("maestro-analysis-{}", analysis_id));
    std::fs::create_dir_all(&temp_dir)?;

    // Emit running status event (but don't update DB since there's no Running enum variant)
    emit_analysis_status(&app, &analysis_id, "running", None);

    let execution_result = execute_with_amp(
        &temp_dir,
        &analysis_prompt,
        None,
        None,
        None::<fn(&str)>,
        &app,
    )
    .await;

    let _ = std::fs::remove_dir_all(&temp_dir);

    match execution_result {
        Ok((session_id, result_message)) => {
            let amp_thread_url = format!("https://ampcode.com/threads/{}", session_id);
            let completed_at = now_ms();
            let result = result_message.unwrap_or_default();

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
            emit_analysis_result(
                &app,
                &analysis_id,
                &result,
                Some(&amp_thread_url),
                completed_at,
            );
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
    store
        .get_analysis(&analysis_id)
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

    store
        .get_analyses_by_revision(&revision_id, analysis_type_enum)
        .map_err(|e| format!("Failed to get analyses: {}", e))
}

#[tauri::command]
pub fn delete_analysis(analysis_id: String, store: State<Mutex<Store>>) -> Result<bool, String> {
    let store = store.lock().unwrap();
    store
        .delete_analysis(&analysis_id)
        .map_err(|e| format!("Failed to delete analysis: {}", e))
}
