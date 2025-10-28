use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

pub mod amp;
pub mod ci;
mod commands;
mod db;
mod git;
mod sourcegraph;
pub mod types;
mod util;

#[derive(Clone)]
pub struct Paths {
    pub admin_repo_dir: PathBuf,
    pub worktree_dir: PathBuf,
    pub db_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Ensure SSH_AUTH_SOCK is set for 1Password/ssh-agent compatibility
    #[cfg(target_os = "macos")]
    {
        // Check 1Password SSH agent first (common setup)
        let op_ssh_sock = format!(
            "{}/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock",
            std::env::var("HOME").unwrap_or_default()
        );
        if std::path::Path::new(&op_ssh_sock).exists() {
            std::env::set_var("SSH_AUTH_SOCK", &op_ssh_sock);
            log::info!("Using 1Password SSH agent at {}", op_ssh_sock);
        } else if std::env::var("SSH_AUTH_SOCK").is_err() {
            log::warn!("No SSH_AUTH_SOCK found and 1Password agent not detected");
        }
    }

    tauri::Builder::default()
        .setup(|app| {
            // Compute paths from app_data_dir or MAESTRO_CONFIG override
            let base_dir = if let Ok(custom_base) = std::env::var("MAESTRO_CONFIG") {
                PathBuf::from(custom_base)
            } else {
                app.path()
                    .app_data_dir()
                    .expect("Failed to get app data directory")
            };

            let paths = Paths {
                admin_repo_dir: base_dir.join("repos"),
                worktree_dir: base_dir.join("executions"),
                db_path: base_dir.join("maestro.db"),
            };

            // Create directories if they don't exist
            std::fs::create_dir_all(&paths.admin_repo_dir)
                .expect("Failed to create admin repo directory");
            std::fs::create_dir_all(&paths.worktree_dir)
                .expect("Failed to create worktree directory");
            if let Some(db_parent) = paths.db_path.parent() {
                std::fs::create_dir_all(db_parent).expect("Failed to create database directory");
            }

            // Initialize database with the computed path
            let store = db::store::Store::new(paths.db_path.to_str().expect("Invalid db path"))
                .expect("Failed to initialize database");

            // Initialize token cache (single keyring access for all tokens)
            commands::tokens::init_token_cache()
                .unwrap_or_else(|e| log::warn!("Failed to initialize token cache: {}", e));

            // Crash recovery: reset any stuck running executions/validations
            commands::executor::reconcile_on_startup(&store)
                .expect("Failed to reconcile execution states");

            // Manage both store and paths as state
            app.manage(Mutex::new(store));
            app.manage(paths);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::db::create_repository,
            commands::db::update_repository_name,
            commands::db::sync_repository_metadata,
            commands::db::get_repository,
            commands::db::find_repository,
            commands::db::get_all_repositories,
            commands::db::create_promptset,
            commands::db::get_all_promptsets,
            commands::db::get_promptset,
            commands::db::find_promptset_by_prefix,
            commands::db::update_promptset_validation,
            commands::db::update_promptset_auto_validate,
            commands::db::update_promptset_repositories,
            commands::db::create_prompt_revision,
            commands::db::get_prompt_revision,
            commands::db::find_prompt_revision_by_prefix,
            commands::db::get_promptset_revisions,
            commands::db::create_execution,
            commands::db::update_execution,
            commands::db::get_execution,
            commands::db::find_execution_by_prefix,
            commands::db::get_executions_by_revision,
            commands::db::get_executions_by_promptset,
            commands::db::delete_execution,
            commands::db::delete_promptset,
            commands::db::delete_repository,
            commands::db::delete_prompt_revision,
            commands::executor::execute_promptset,
            commands::executor::prepare_executions,
            commands::executor::execute_prompt,
            commands::executor::validate_execution,
            commands::executor::resume_execution,
            commands::executor::commit_changes,
            commands::executor::push_commit,
            commands::executor::stop_execution,
            commands::executor::stop_validation,
            commands::executor::stop_all_executions,
            commands::executor::stop_all_validations,
            commands::executor::cleanup_execution,
            commands::executor::get_execution_modified_files,
            commands::executor::get_execution_file_diff,
            commands::executor::reconcile_stuck_ci,
            commands::ci::start_ci_check,
            commands::ci::refresh_ci_status,
            commands::github::get_config_paths,
            commands::tokens::set_token,
            commands::tokens::delete_token,
            commands::tokens::get_all_tokens,
            commands::tokens::get_all_tokens_masked,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::get_ci_stuck_threshold_minutes,
            commands::settings::get_max_concurrent_executions,
            commands::execution_poller::start_execution_polling,
            commands::execution_poller::stop_execution_polling,
            commands::execution_poller::get_pending_executions_count,
            commands::worktree::open_worktree_in_editor,
            commands::worktree::open_worktree_with_terminal,
            commands::app_check::get_available_editors,
            commands::app_check::get_available_terminals,
            commands::app_check::check_app_installed,
            commands::sourcegraph::search_sourcegraph_repositories,
            commands::analysis::create_analysis,
            commands::analysis::run_analysis,
            commands::analysis::get_analysis,
            commands::analysis::get_analyses_by_revision,
            commands::analysis::delete_analysis,
        ])
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
