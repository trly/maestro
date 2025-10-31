use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(target_os = "macos")]
use tauri::menu::PredefinedMenuItem;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager, Wry};

pub mod ci;
mod commands;
mod db;
mod git;
mod sourcegraph;
pub mod types;
mod util;

fn build_menu(app: &AppHandle<Wry>) -> tauri::Result<()> {
    // Create About menu item
    let about = MenuItemBuilder::with_id("about", "About Maestro").build(app)?;

    #[cfg(target_os = "macos")]
    {
        // On macOS, create app menu with About and standard items
        let app_menu = SubmenuBuilder::new(app, "Maestro")
            .item(&about)
            .separator()
            .item(&PredefinedMenuItem::separator(app)?)
            .item(&PredefinedMenuItem::hide(app, None)?)
            .item(&PredefinedMenuItem::hide_others(app, None)?)
            .item(&PredefinedMenuItem::show_all(app, None)?)
            .separator()
            .item(&PredefinedMenuItem::quit(app, None)?)
            .build()?;

        // Edit menu with standard shortcuts (critical for copy/paste)
        let edit_menu = SubmenuBuilder::new(app, "Edit")
            .item(&PredefinedMenuItem::undo(app, None)?)
            .item(&PredefinedMenuItem::redo(app, None)?)
            .separator()
            .item(&PredefinedMenuItem::cut(app, None)?)
            .item(&PredefinedMenuItem::copy(app, None)?)
            .item(&PredefinedMenuItem::paste(app, None)?)
            .item(&PredefinedMenuItem::select_all(app, None)?)
            .build()?;

        // Window menu with standard window management
        let window_menu = SubmenuBuilder::new(app, "Window")
            .item(&PredefinedMenuItem::minimize(app, None)?)
            .item(&PredefinedMenuItem::maximize(app, None)?)
            .separator()
            .item(&PredefinedMenuItem::close_window(app, None)?)
            .build()?;

        let menu = MenuBuilder::new(app)
            .item(&app_menu)
            .item(&edit_menu)
            .item(&window_menu)
            .build()?;

        app.set_menu(menu)?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On Windows/Linux, add About to Help menu
        let help_menu = SubmenuBuilder::new(app, "Help").item(&about).build()?;

        let menu = MenuBuilder::new(app).item(&help_menu).build()?;

        app.set_menu(menu)?;
    }

    // Handle menu events
    app.on_menu_event(move |app_handle, event| {
        if event.id() == "about" {
            // Navigate to about page in existing window
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.eval("window.location.href = '/about'");
            }
        }
    });

    Ok(())
}

#[derive(Clone)]
pub struct Paths {
    pub admin_repo_dir: PathBuf,
    pub worktree_dir: PathBuf,
    pub db_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize logging first so all subsequent operations can use it
            let log_level = std::env::var("MAESTRO_LOG_LEVEL")
                .ok()
                .and_then(|level| match level.to_lowercase().as_str() {
                    "trace" => Some(log::LevelFilter::Trace),
                    "debug" => Some(log::LevelFilter::Debug),
                    "info" => Some(log::LevelFilter::Info),
                    "warn" => Some(log::LevelFilter::Warn),
                    "error" => Some(log::LevelFilter::Error),
                    _ => None,
                })
                .unwrap_or(log::LevelFilter::Info); // Default to Info for file logging

            let mut builder = tauri_plugin_log::Builder::default()
                .level(log_level) // File logging level
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(10))
                .max_file_size(5 * 1024 * 1024) // 5MB per file
                .targets(vec![tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: None },
                )]);

            // Add stdout target in debug builds or if MAESTRO_LOG_STDOUT is set
            let enable_stdout = cfg!(debug_assertions)
                || std::env::var("MAESTRO_LOG_STDOUT")
                    .ok()
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(false);

            if enable_stdout {
                builder = builder.target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ));
            }

            // Initialize logging plugin - handle errors gracefully
            if let Err(e) = app.handle().plugin(builder.build()) {
                eprintln!("Failed to initialize logging plugin: {}", e);
            }

            // Ensure SSH_AUTH_SOCK is set for 1Password/ssh-agent compatibility
            #[cfg(target_os = "macos")]
            {
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

            // Build native menu
            build_menu(app.handle())?;

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
            commands::settings::get_first_run_complete,
            commands::settings::set_first_run_complete,
            commands::settings::get_show_first_run_dialog,
            commands::settings::set_show_first_run_dialog,
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
            commands::health_check::health_check_github,
            commands::health_check::health_check_gitlab,
            commands::health_check::health_check_sourcegraph,
            commands::health_check::health_check_git,
            commands::app_info::get_app_info,
        ])
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
