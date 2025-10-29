use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::Manager;

use super::executor_events::{
    emit_execution_commit, emit_execution_progress, emit_execution_session, emit_execution_status,
    emit_execution_validation,
};
use crate::db::store::{ExecutionUpdates, Store};
use crate::git::{
    get_committed_diff, get_committed_file_diff, get_worktree_diff, get_worktree_file_diff,
    GitService, ModifiedFilesResponse,
};
use crate::types::{CiStatus, CommitStatus, ExecutionStatus, PromptStatus, ValidationStatus};
use crate::util::git::{maestro_branch_name, parse_provider_id};
use crate::util::paths::{admin_repo_path, execution_worktree_path, worktree_path};
use crate::Paths;

async fn fetch_default_branch(provider: &str, provider_id: &str) -> Result<String> {
    use crate::git::git_provider::create_git_provider;
    use crate::git::GitProviderContext;
    use crate::util::git::build_provider_cfg;

    let git_provider = create_git_provider(provider, provider_id).await?;

    let ctx = GitProviderContext {
        provider_cfg: build_provider_cfg(provider, provider_id)?,
    };

    let metadata = git_provider.get_repo_metadata(&ctx).await?;
    Ok(metadata.default_branch)
}

lazy_static::lazy_static! {
    static ref ACTIVE_EXECUTIONS: Mutex<HashMap<String, std::sync::Arc<Mutex<bool>>>> = Mutex::new(HashMap::new());
    static ref ACTIVE_VALIDATIONS: Mutex<HashMap<String, std::sync::Arc<Mutex<bool>>>> = Mutex::new(HashMap::new());
    static ref ACTIVE_CHILDREN: Mutex<HashMap<String, std::sync::Arc<Mutex<std::process::Child>>>> = Mutex::new(HashMap::new());
    static ref REPO_LOCKS: Mutex<HashMap<String, std::sync::Arc<Mutex<()>>>> = Mutex::new(HashMap::new());
}

fn get_repo_lock(owner: &str, repo: &str) -> std::sync::Arc<Mutex<()>> {
    let repo_key = format!("{}/{}", owner, repo);
    let mut locks = REPO_LOCKS.lock().unwrap();
    locks
        .entry(repo_key)
        .or_insert_with(|| std::sync::Arc::new(Mutex::new(())))
        .clone()
}

async fn ensure_admin_repo_and_fetch(
    admin_repo_dir: &Path,
    provider: &str,
    owner: &str,
    repo: &str,
) -> Result<PathBuf> {
    use crate::commands::tokens::get_token_value;

    let repo_lock = get_repo_lock(owner, repo);
    let _lock = repo_lock.lock().unwrap();

    let admin_repo_path = admin_repo_dir.join(owner).join(repo);

    if !admin_repo_path.join(".git").exists() {
        let parent_dir = admin_repo_dir.join(owner);
        std::fs::create_dir_all(&parent_dir)?;

        let url = match provider {
            "github" => format!("git@github.com:{}/{}.git", owner, repo),
            "gitlab" => {
                // Check for custom GitLab instance URL (try both keys for compatibility)
                let instance_url = get_token_value("gitlab_instance_url")
                    .ok()
                    .flatten()
                    .or_else(|| get_token_value("gitlab_endpoint").ok().flatten())
                    .unwrap_or_else(|| "https://gitlab.com".to_string());

                // Extract hostname from URL (e.g., "https://gitlab.example.com" -> "gitlab.example.com")
                let host = instance_url
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .trim_end_matches('/');

                format!("git@{}:{}/{}.git", host, owner, repo)
            }
            _ => anyhow::bail!("Unsupported provider for SSH clone: {}", provider),
        };

        GitService::clone_repo(&url, &admin_repo_path)
			.context("Failed to clone repository. Ensure SSH key is added to ssh-agent (try: ssh-add ~/.ssh/id_rsa)")?;
    }

    let repo = GitService::open(&admin_repo_path)?;
    GitService::fetch(&repo, "origin", &["+refs/heads/*:refs/remotes/origin/*"])
        .context("Failed to fetch from origin. Ensure SSH authentication is configured.")?;

    Ok(admin_repo_path)
}

struct WorktreeInfo {
    worktree_path: PathBuf,
    branch_name: String,
    base_commit: String,
}

async fn add_worktree(
    admin_repo_path: &PathBuf,
    worktree_dir: &Path,
    promptset_id: &str,
    revision_id: &str,
    execution_id: &str,
    default_branch: &str,
) -> Result<WorktreeInfo> {
    let branch_name = maestro_branch_name(promptset_id, revision_id, execution_id);
    let worktree_path = worktree_path(worktree_dir, promptset_id, execution_id);

    let base_ref = format!("origin/{}", default_branch);

    std::fs::create_dir_all(worktree_path.parent().unwrap())?;

    // If worktree exists, remove it properly using git worktree remove
    if worktree_path.exists() {
        let _ = Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(admin_repo_path)
            .output();

        let _ = Command::new("git")
            .args(["worktree", "prune", "-v"])
            .current_dir(admin_repo_path)
            .output();
    }

    // If branch already exists, delete it
    let branch_check = Command::new("git")
        .args(["rev-parse", "--verify", &branch_name])
        .current_dir(admin_repo_path)
        .output()?;

    if branch_check.status.success() {
        let repo = GitService::open(admin_repo_path)?;
        let _ = GitService::delete_local_branch(&repo, &branch_name, true);
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            &branch_name,
            worktree_path.to_str().unwrap(),
            &base_ref,
        ])
        .current_dir(admin_repo_path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let repo = GitService::open(&worktree_path)?;
    let base_commit = GitService::rev_parse(&repo, "HEAD")?;

    Ok(WorktreeInfo {
        worktree_path,
        branch_name,
        base_commit,
    })
}

#[allow(dead_code)]
async fn remove_worktree(
    admin_repo_path: &PathBuf,
    worktree_dir: &Path,
    promptset_id: &str,
    execution_id: &str,
    branch_name: Option<&str>,
) -> Result<()> {
    let worktree_path = worktree_path(worktree_dir, promptset_id, execution_id);

    let output = Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(admin_repo_path)
        .output()?;

    if !output.status.success() {
        log::error!(
            "[remove_worktree] Failed to remove worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    if let Some(branch) = branch_name {
        let repo = GitService::open(admin_repo_path)?;
        let _ = GitService::delete_local_branch(&repo, branch, true);
    }

    let _ = Command::new("git")
        .args(["worktree", "prune", "-v"])
        .current_dir(admin_repo_path)
        .output();

    Ok(())
}

async fn execute_with_amp(
    repo_path: &Path,
    prompt_text: &str,
    continue_session_id: Option<&str>,
    execution_key: Option<&str>,
    abort_flag: Option<std::sync::Arc<Mutex<bool>>>,
    on_session_start: Option<impl Fn(&str)>,
    app: &tauri::AppHandle,
) -> Result<(String, Option<String>)> {
    let mut session_id = String::new();
    let mut result_message: Option<String> = None;

    let bun_path = which::which("bun").context("bun command not found in PATH")?;

    // In dev mode, use the source file directly
    // In production, use the bundled resource
    let executor_script = if cfg!(debug_assertions) {
        // Development mode - use source file
        let mut workspace_root = std::env::current_dir()?;
        if workspace_root.ends_with("src-tauri") {
            workspace_root = workspace_root
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Failed to get workspace root"))?
                .to_path_buf();
        }
        workspace_root
            .join("src")
            .join("lib")
            .join("amp-executor.ts")
    } else {
        // Production mode - use bundled resource
        app.path()
            .resource_dir()
            .context("Failed to get resource directory")?
            .join("src/lib/amp-executor.ts")
    };

    if !executor_script.exists() {
        anyhow::bail!("amp-executor.ts not found at {:?}", executor_script);
    }

    let mut args = vec![
        "run".to_string(),
        executor_script.to_string_lossy().to_string(),
        repo_path.to_string_lossy().to_string(),
        prompt_text.to_string(),
    ];

    if let Some(continue_id) = continue_session_id {
        args.push(continue_id.to_string());
    }

    let child = Command::new(&bun_path)
        .args(&args)
        .env(
            "AMP_API_KEY",
            std::env::var("AMP_API_KEY").unwrap_or_default(),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn bun process")?;

    // Store child process for potential cancellation
    let child = std::sync::Arc::new(Mutex::new(child));
    if let Some(key) = execution_key {
        let mut active = ACTIVE_CHILDREN.lock().unwrap();
        active.insert(key.to_string(), child.clone());
    }

    // Check for early abort (before we even start reading)
    if let Some(abort) = &abort_flag {
        if *abort.lock().unwrap() {
            let _ = child.lock().unwrap().kill();
            if let Some(key) = execution_key {
                ACTIVE_CHILDREN.lock().unwrap().remove(key);
            }
            anyhow::bail!("Execution aborted");
        }
    }

    // Take stdout and stderr from child
    let stderr = child
        .lock()
        .unwrap()
        .stderr
        .take()
        .context("Failed to capture stderr")?;
    let stdout = child
        .lock()
        .unwrap()
        .stdout
        .take()
        .context("Failed to capture stdout")?;

    // Spawn thread to read stdout concurrently to avoid deadlock
    let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = String::new();
        let _ = std::io::BufReader::new(stdout).read_to_string(&mut buf);
        let _ = stdout_tx.send(buf);
    });

    // Read stderr on main thread to capture session_id
    let stderr_reader = std::io::BufReader::new(stderr);
    use std::io::BufRead;
    for line in stderr_reader.lines() {
        let line = line?;

        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
            if msg.get("type") == Some(&serde_json::json!("session_id")) {
                if let Some(sid) = msg.get("sessionId").and_then(|v| v.as_str()) {
                    session_id = sid.to_string();
                    if let Some(callback) = &on_session_start {
                        callback(sid);
                    }
                }
            }
        }
    }

    // Get stdout content
    let stdout_content = stdout_rx.recv().unwrap_or_default();

    // Check for abort before waiting
    if let Some(abort) = &abort_flag {
        if *abort.lock().unwrap() {
            let _ = child.lock().unwrap().kill();
            if let Some(key) = execution_key {
                ACTIVE_CHILDREN.lock().unwrap().remove(key);
            }
            anyhow::bail!("Execution aborted");
        }
    }

    let status = child
        .lock()
        .unwrap()
        .wait()
        .context("Failed to wait for bun process")?;

    // Remove from active children
    if let Some(key) = execution_key {
        ACTIVE_CHILDREN.lock().unwrap().remove(key);
    }

    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout_content) {
        if let Some(sid) = result.get("sessionId").and_then(|v| v.as_str()) {
            if session_id.is_empty() {
                session_id = sid.to_string();
                if let Some(callback) = &on_session_start {
                    callback(sid);
                }
            }
        }
        if let Some(res) = result.get("resultMessage").and_then(|v| v.as_str()) {
            result_message = Some(res.to_string());
        }
        if let Some(err) = result.get("error").and_then(|v| v.as_str()) {
            anyhow::bail!("Amp execution failed: {}", err);
        }
    }

    if !status.success() {
        anyhow::bail!("Amp process exited with status: {}", status);
    }

    Ok((session_id, result_message))
}

#[tauri::command]
pub async fn execute_promptset(
    promptset_id: String,
    revision_id: String,
    repository_ids: Option<Vec<String>>,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<Vec<String>, String> {
    let promptset = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().map_err(|e| e.to_string())?;
        store
            .get_promptset(&promptset_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("PromptSet {} not found", promptset_id))?
    };

    let repo_ids = if let Some(ids) = repository_ids {
        ids.into_iter()
            .filter(|id| promptset.repository_ids.contains(id))
            .collect()
    } else {
        promptset.repository_ids.clone()
    };

    let mut execution_ids = Vec::new();

    for repository_id in repo_ids {
        let execution = {
            let store_state = app.state::<Mutex<Store>>();
            let store = store_state.lock().map_err(|e| e.to_string())?;
            store
                .create_execution(&promptset_id, &revision_id, &repository_id)
                .map_err(|e| e.to_string())?
        };

        execution_ids.push(execution.id.clone());

        let execution_id = execution.id.clone();
        let app_clone = app.clone();
        let paths_clone = paths.inner().clone();

        tokio::spawn(async move {
            if let Err(e) = execute_prompt_impl(execution_id, app_clone, paths_clone).await {
                log::error!("Execution failed: {}", e);
            }
        });
    }

    Ok(execution_ids)
}

#[tauri::command]
pub async fn prepare_executions(
    promptset_id: String,
    revision_id: String,
    repository_ids: Option<Vec<String>>,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<Vec<String>, String> {
    let promptset = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().map_err(|e| e.to_string())?;
        store
            .get_promptset(&promptset_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("PromptSet {} not found", promptset_id))?
    };

    let repo_ids = if let Some(ids) = repository_ids {
        ids.into_iter()
            .filter(|id| promptset.repository_ids.contains(id))
            .collect()
    } else {
        promptset.repository_ids.clone()
    };

    let mut execution_ids = Vec::new();

    for repository_id in repo_ids {
        let execution = {
            let store_state = app.state::<Mutex<Store>>();
            let store = store_state.lock().map_err(|e| e.to_string())?;
            store
                .create_execution(&promptset_id, &revision_id, &repository_id)
                .map_err(|e| e.to_string())?
        };

        execution_ids.push(execution.id.clone());

        let execution_id = execution.id.clone();
        let app_clone = app.clone();
        let paths_clone = paths.inner().clone();

        tokio::spawn(async move {
            if let Err(e) = prepare_execution_impl(execution_id, app_clone, paths_clone).await {
                log::error!("Execution preparation failed: {}", e);
            }
        });
    }

    Ok(execution_ids)
}

async fn prepare_execution_impl(
    execution_id: String,
    app: tauri::AppHandle,
    paths: Paths,
) -> Result<()> {
    let (execution, repository) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();

        let execution = store
            .get_execution(&execution_id)?
            .ok_or_else(|| anyhow::anyhow!("Execution {} not found", execution_id))?;
        let repository = store
            .get_repository(&execution.repository_id)?
            .ok_or_else(|| anyhow::anyhow!("Repository {} not found", execution.repository_id))?;

        (execution, repository)
    };

    let (owner, repo) = parse_provider_id(&repository.provider_id)?;

    let admin_repo_path =
        ensure_admin_repo_and_fetch(&paths.admin_repo_dir, &repository.provider, &owner, &repo)
            .await?;

    // Use cached default branch or fetch from provider if not cached
    let default_branch = if let Some(cached_branch) = &repository.default_branch {
        cached_branch.clone()
    } else {
        let branch = fetch_default_branch(&repository.provider, &repository.provider_id).await?;
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        let _ = store.update_repository_default_branch(&repository.id, &branch);
        branch
    };

    let worktree_info = add_worktree(
        &admin_repo_path,
        &paths.worktree_dir,
        &execution.promptset_id,
        &execution.revision_id,
        &execution_id,
        &default_branch,
    )
    .await?;

    {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        store.update_execution(
            &execution_id,
            ExecutionUpdates {
                parent_sha: Some(worktree_info.base_commit.clone()),
                branch: Some(worktree_info.branch_name.clone()),
                ..Default::default()
            },
        )?;
    }

    Ok(())
}

#[tauri::command]
pub async fn execute_prompt(
    execution_id: String,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    execute_prompt_impl(execution_id, app, paths.inner().clone())
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn execute_prompt_impl(
    execution_id: String,
    app: tauri::AppHandle,
    paths: Paths,
) -> Result<()> {
    let (execution, repository, revision) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();

        let execution = store
            .get_execution(&execution_id)?
            .ok_or_else(|| anyhow::anyhow!("Execution {} not found", execution_id))?;
        let repository = store
            .get_repository(&execution.repository_id)?
            .ok_or_else(|| anyhow::anyhow!("Repository {} not found", execution.repository_id))?;
        let revision = store
            .get_prompt_revision(&execution.revision_id)?
            .ok_or_else(|| anyhow::anyhow!("Revision {} not found", execution.revision_id))?;

        (execution, repository, revision)
    };

    // Guard against duplicate runs
    {
        let active = ACTIVE_EXECUTIONS.lock().unwrap();
        if active.contains_key(&execution_id) {
            anyhow::bail!("Execution {} is already running", execution_id);
        }
        if execution.status == ExecutionStatus::Running {
            anyhow::bail!("Execution {} is already in running state", execution_id);
        }
    }

    {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        store.update_execution(
            &execution_id,
            ExecutionUpdates {
                status: Some(ExecutionStatus::Running),
                files_added: Some(0),
                files_removed: Some(0),
                files_modified: Some(0),
                lines_added: Some(0),
                lines_removed: Some(0),
                ..Default::default()
            },
        )?;
    }
    emit_execution_status(&app, &execution_id, "running");

    let abort_flag = std::sync::Arc::new(Mutex::new(false));
    {
        let mut active = ACTIVE_EXECUTIONS.lock().unwrap();
        active.insert(execution_id.clone(), abort_flag.clone());
    }

    let result = async {
    let (owner, repo) = parse_provider_id(&repository.provider_id)?;

    let admin_repo_path = ensure_admin_repo_and_fetch(&paths.admin_repo_dir, &repository.provider, &owner, &repo).await?;

    // Use cached default branch or fetch from provider if not cached
	let default_branch = if let Some(cached_branch) = &repository.default_branch {
     cached_branch.clone()
	} else {
     let branch = fetch_default_branch(&repository.provider, &repository.provider_id).await?;
     log::info!("[execute_prompt] Fetched default branch for {}/{}: {}", owner, repo, branch);
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();
			let _ = store.update_repository_default_branch(&repository.id, &branch);
			branch
		};

		let worktree_info = add_worktree(
			&admin_repo_path,
			&paths.worktree_dir,
			&execution.promptset_id,
			&execution.revision_id,
			&execution_id,
			&default_branch,
		)
		.await?;

		// Persist parent_sha and branch immediately after worktree creation
		{
		let store_state = app.state::<Mutex<Store>>();
		let store = store_state.lock().unwrap();
		 store.update_execution(
			&execution_id,
			ExecutionUpdates {
				parent_sha: Some(worktree_info.base_commit.clone()),
				branch: Some(worktree_info.branch_name.clone()),
				..Default::default()
			},
		)?;
	}

		let response_format = "

IMPORTANT: You MUST end your final response with exactly one of these lines on the final line to reflect if the above prompt is considered successful or not:
PROMPT: PASS
PROMPT: FAIL";
		let full_prompt = format!("{}{}", revision.prompt_text, response_format);

		let execution_id_clone = execution_id.clone();
		let app_clone = app.clone();

		let (session_id, result_message) = execute_with_amp(
			&worktree_info.worktree_path,
			&full_prompt,
			None,
			Some(&format!("exec:{}", execution_id)),
			Some(abort_flag.clone()),
			Some(move |sid: &str| {
				let thread_url = format!("https://ampcode.com/threads/{}", sid);
				let store_state = app_clone.state::<Mutex<Store>>();
				let store = store_state.lock().unwrap();
				let _ = store.update_execution(
					&execution_id_clone,
					ExecutionUpdates {
						session_id: Some(sid.to_string()),
						thread_url: Some(thread_url.clone()),
						..Default::default()
					},
				);
				emit_execution_session(&app_clone, &execution_id_clone, sid, &thread_url);
			}),
			&app,
		)
		.await?;

		let thread_url = format!("https://ampcode.com/threads/{}", session_id);

		let prompt_passed = result_message.as_ref().map(|m| m.contains("PROMPT: PASS")).unwrap_or(false);
		let prompt_failed = result_message.as_ref().map(|m| m.contains("PROMPT: FAIL")).unwrap_or(false);
		let prompt_status = if prompt_passed {
			Some(PromptStatus::Passed)
		} else if prompt_failed {
			Some(PromptStatus::Failed)
		} else {
			None
		};

		let execution_status = if prompt_failed {
			ExecutionStatus::Failed
		} else {
			ExecutionStatus::Completed
		};

		let should_validate = {
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();

			store.update_execution(
				&execution_id,
				ExecutionUpdates {
					status: Some(execution_status),
					session_id: Some(session_id.clone()),
					thread_url: Some(thread_url),
					prompt_status,
					prompt_result: result_message.clone(),
					completed_at: Some(chrono::Utc::now().timestamp_millis()),
					..Default::default()
				},
			)?;

			let promptset = store.get_promptset(&execution.promptset_id)?
				.ok_or_else(|| anyhow::anyhow!("PromptSet {} not found", execution.promptset_id))?;

			promptset.validation_prompt.is_some() && prompt_status == Some(PromptStatus::Passed)
		};

		let status_str = match execution_status {
			ExecutionStatus::Failed => "failed",
			_ => "completed",
		};
		emit_execution_status(&app, &execution_id, status_str);

		if should_validate {
			let validate_execution_id = execution_id.clone();
			let app_clone = app.clone();
			let paths_clone = paths.clone();
			tokio::spawn(async move {
				if let Err(e) = validate_execution_impl(validate_execution_id, app_clone, paths_clone).await {
					log::error!("Validation failed: {}", e);
				}
			});
		}

		Ok::<(), anyhow::Error>(())
	}
	.await;

    {
        let mut active = ACTIVE_EXECUTIONS.lock().unwrap();
        active.remove(&execution_id);
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("[execute_prompt] Execution {} failed: {}", execution_id, e);
            let is_aborted = e.to_string().contains("aborted");
            let status = if is_aborted {
                ExecutionStatus::Cancelled
            } else {
                ExecutionStatus::Failed
            };
            let status_str = if is_aborted { "cancelled" } else { "failed" };

            {
                let store_state = app.state::<Mutex<Store>>();
                let store = store_state.lock().unwrap();
                store.update_execution(
                    &execution_id,
                    ExecutionUpdates {
                        status: Some(status),
                        completed_at: Some(chrono::Utc::now().timestamp_millis()),
                        ..Default::default()
                    },
                )?;
            }
            emit_execution_status(&app, &execution_id, status_str);

            if !is_aborted {
                Err(e)
            } else {
                Ok(())
            }
        }
    }
}

#[tauri::command]
pub async fn validate_execution(
    execution_id: String,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    validate_execution_impl(execution_id, app, paths.inner().clone())
        .await
        .map_err(|e| e.to_string())
}

async fn validate_execution_impl(
    execution_id: String,
    app: tauri::AppHandle,
    paths: Paths,
) -> Result<()> {
    let (execution, repository, validation_prompt_text) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();

        let execution = store
            .get_execution(&execution_id)?
            .ok_or_else(|| anyhow::anyhow!("Execution {} not found", execution_id))?;

        if execution.status != ExecutionStatus::Completed
            && execution.status != ExecutionStatus::Cancelled
        {
            return Ok(());
        }

        let promptset = store
            .get_promptset(&execution.promptset_id)?
            .ok_or_else(|| anyhow::anyhow!("PromptSet {} not found", execution.promptset_id))?;

        if promptset.validation_prompt.is_none() {
            store.update_execution(
                &execution_id,
                ExecutionUpdates {
                    validation_status: Some(ValidationStatus::Failed),
                    ..Default::default()
                },
            )?;
            return Ok(());
        }

        let repository = store
            .get_repository(&execution.repository_id)?
            .ok_or_else(|| anyhow::anyhow!("Repository {} not found", execution.repository_id))?;

        (execution, repository, promptset.validation_prompt.unwrap())
    };

    // Guard against duplicate validation runs
    {
        let active = ACTIVE_VALIDATIONS.lock().unwrap();
        if active.contains_key(&execution_id) {
            anyhow::bail!(
                "Validation for execution {} is already running",
                execution_id
            );
        }
        if execution.validation_status == Some(ValidationStatus::Running) {
            anyhow::bail!(
                "Validation for execution {} is already in running state",
                execution_id
            );
        }
    }

    {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        store.update_execution(
            &execution_id,
            ExecutionUpdates {
                validation_status: Some(ValidationStatus::Running),
                ..Default::default()
            },
        )?;
    }
    emit_execution_validation(&app, &execution_id, "running", None);

    let abort_flag = std::sync::Arc::new(Mutex::new(false));
    {
        let mut active = ACTIVE_VALIDATIONS.lock().unwrap();
        active.insert(execution_id.clone(), abort_flag.clone());
    }

    let result = async {
		let (owner, repo) = parse_provider_id(&repository.provider_id)?;

		let _admin_repo_path = admin_repo_path(&paths, &owner, &repo);
		let worktree_path = execution_worktree_path(&paths, &execution.promptset_id, &execution_id);

		let branch_name = maestro_branch_name(&execution.promptset_id, &execution.revision_id, &execution_id);

		if !worktree_path.exists() {
			anyhow::bail!("Worktree not found at {:?}", worktree_path);
		}

		let output = Command::new("git")
			.args(["status", "--porcelain"])
			.current_dir(&worktree_path)
			.output()?;

		let has_changes = !String::from_utf8_lossy(&output.stdout).trim().is_empty();

		if !has_changes {
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();
			store.update_execution(
				&execution_id,
				ExecutionUpdates {
					validation_status: Some(ValidationStatus::Failed),
					validation_result: Some("No pending changes found to validate".to_string()),
					..Default::default()
				},
			)?;
			return Ok(());
		}

		let system_prompt = format!("You are a code change validation reviewer
You are tasked with ensuring the current changes in {}.
You are to review the pending changes in the current branch with the oracle, librarian, and any other tools that will not make any further code changes to ensure that the following is true:

", branch_name);

		let response_format = "

IMPORTANT: You MUST end your response with exactly one of these lines on the final line:
VALIDATION: PASS
VALIDATION: FAIL";

		let full_validation_prompt = format!(
			"{}{}{}",
			system_prompt,
			validation_prompt_text,
			response_format
		);

		let execution_id_clone = execution_id.clone();
		let app_clone = app.clone();

		let (validation_session_id, result_message) = execute_with_amp(
			&worktree_path,
			&full_validation_prompt,
			None,
			Some(&format!("val:{}", execution_id)),
			Some(abort_flag.clone()),
			Some(move |sid: &str| {
				let validation_thread_url = format!("https://ampcode.com/threads/{}", sid);
				let store_state = app_clone.state::<Mutex<Store>>();
				let store = store_state.lock().unwrap();
				let _ = store.update_execution(
					&execution_id_clone,
					ExecutionUpdates {
						validation_thread_url: Some(validation_thread_url.clone()),
						..Default::default()
					},
				);
				emit_execution_session(&app_clone, &execution_id_clone, sid, &validation_thread_url);
			}),
			&app,
		)
		.await?;

		let validation_thread_url = format!("https://ampcode.com/threads/{}", validation_session_id);

		let validation_passed = result_message.as_ref().map(|m| m.contains("VALIDATION: PASS")).unwrap_or(false);
		let validation_status = if validation_passed {
		 ValidationStatus::Passed
		} else {
		 ValidationStatus::Failed
		};
		let validation_status_str = match validation_status {
			ValidationStatus::Passed => "passed",
			ValidationStatus::Failed => "failed",
			_ => "failed",
		};

		{
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();
			store.update_execution(
				&execution_id,
				ExecutionUpdates {
					validation_status: Some(validation_status),
					validation_thread_url: Some(validation_thread_url.clone()),
					validation_result: result_message,
					..Default::default()
				},
			)?;
		}
		emit_execution_validation(&app, &execution_id, validation_status_str, Some(&validation_thread_url));

		Ok::<(), anyhow::Error>(())
	}
	.await;

    {
        let mut active = ACTIVE_VALIDATIONS.lock().unwrap();
        active.remove(&execution_id);
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!(
                "[validate_execution] Validation {} failed: {}",
                execution_id,
                e
            );
            let is_aborted = e.to_string().contains("aborted");
            let status = if is_aborted {
                ValidationStatus::Cancelled
            } else {
                ValidationStatus::Failed
            };
            let status_str = if is_aborted { "cancelled" } else { "failed" };

            {
                let store_state = app.state::<Mutex<Store>>();
                let store = store_state.lock().unwrap();
                store.update_execution(
                    &execution_id,
                    ExecutionUpdates {
                        validation_status: Some(status),
                        ..Default::default()
                    },
                )?;
            }
            emit_execution_validation(&app, &execution_id, status_str, None);

            if !is_aborted {
                Err(e)
            } else {
                Ok(())
            }
        }
    }
}

#[tauri::command]
pub async fn resume_execution(
    execution_id: String,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    resume_execution_impl(execution_id, app, paths.inner().clone())
        .await
        .map_err(|e| e.to_string())
}

async fn resume_execution_impl(
    execution_id: String,
    app: tauri::AppHandle,
    paths: Paths,
) -> Result<()> {
    log::info!(
        "[resume_execution] Starting resume for execution {}",
        execution_id
    );

    // Guard against duplicate runs
    {
        let active = ACTIVE_EXECUTIONS.lock().unwrap();
        if active.contains_key(&execution_id) {
            anyhow::bail!("Execution {} is already running", execution_id);
        }
    }

    let (execution, repository) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();

        let execution = store
            .get_execution(&execution_id)?
            .ok_or_else(|| anyhow::anyhow!("Execution {} not found", execution_id))?;

        log::info!(
            "[resume_execution] Execution {} has status {:?}",
            execution_id,
            execution.status
        );

        // Allow resuming completed executions (to re-run), as well as cancelled/failed
        if execution.status == ExecutionStatus::Running {
            anyhow::bail!("Cannot resume execution {} - already running", execution_id);
        }

        let repository = store
            .get_repository(&execution.repository_id)?
            .ok_or_else(|| anyhow::anyhow!("Repository {} not found", execution.repository_id))?;

        let _ = store
            .get_prompt_revision(&execution.revision_id)?
            .ok_or_else(|| anyhow::anyhow!("Revision {} not found", execution.revision_id))?;

        (execution, repository)
    };

    {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        store.update_execution(
            &execution_id,
            ExecutionUpdates {
                status: Some(ExecutionStatus::Running),
                ..Default::default()
            },
        )?;
    }
    emit_execution_status(&app, &execution_id, "running");

    let abort_flag = std::sync::Arc::new(Mutex::new(false));
    {
        let mut active = ACTIVE_EXECUTIONS.lock().unwrap();
        active.insert(execution_id.clone(), abort_flag.clone());
    }

    let result = async {
		log::info!("[resume_execution] Parsing provider ID for {}", execution_id);
		let (owner, repo) = parse_provider_id(&repository.provider_id)?;

		log::info!("[resume_execution] Ensuring admin repo and fetching for {}/{}", owner, repo);
		let admin_repo_path = ensure_admin_repo_and_fetch(&paths.admin_repo_dir, &repository.provider, &owner, &repo).await?;
		let worktree_path = execution_worktree_path(&paths, &execution.promptset_id, &execution_id);

		let branch_name = maestro_branch_name(&execution.promptset_id, &execution.revision_id, &execution_id);

		// Recreate worktree if it was cleaned up
		if !worktree_path.exists() {
			log::info!("[resume_execution] Worktree doesn't exist, recreating for {}", execution_id);
			// Use stored default branch, or try fetching, or fall back to "main"
			let default_branch = if let Some(branch) = &repository.default_branch {
				branch.clone()
			} else {
				fetch_default_branch(&repository.provider, &repository.provider_id)
					.await
					.unwrap_or_else(|_| "main".to_string())
			};

			log::info!("[resume_execution] Creating worktree on branch {} for {}", default_branch, execution_id);
			let worktree_info = add_worktree(
				&admin_repo_path,
				&paths.worktree_dir,
				&execution.promptset_id,
				&execution.revision_id,
				&execution_id,
				&default_branch,
			)
			.await?;

			// Update parent_sha and branch if they weren't set
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();
			store.update_execution(
				&execution_id,
				ExecutionUpdates {
					parent_sha: Some(worktree_info.base_commit.clone()),
					branch: Some(worktree_info.branch_name.clone()),
					..Default::default()
				},
			)?;
			log::info!("[resume_execution] Worktree created successfully for {}", execution_id);
		} else {
			log::info!("[resume_execution] Worktree already exists for {}", execution_id);
		}

		let response_format = "

IMPORTANT: You MUST end your final response with exactly one of these lines on the final line to reflect if the above prompt is considered successful or not:
PROMPT: PASS
PROMPT: FAIL";
		let resume_prompt = format!("Please continue with the previous task.{}", response_format);

		log::info!("[resume_execution] Starting Amp execution for {}", execution_id);

		let execution_id_clone = execution_id.clone();
		let app_clone = app.clone();

		let (session_id, result_message) = execute_with_amp(
			&worktree_path,
			&resume_prompt,
			execution.session_id.as_deref(),
			Some(&format!("exec:{}", execution_id)),
			Some(abort_flag.clone()),
			Some(move |sid: &str| {
				let thread_url = format!("https://ampcode.com/threads/{}", sid);
				let store_state = app_clone.state::<Mutex<Store>>();
				let store = store_state.lock().unwrap();
				let _ = store.update_execution(
					&execution_id_clone,
					ExecutionUpdates {
						session_id: Some(sid.to_string()),
						thread_url: Some(thread_url.clone()),
						..Default::default()
					},
				);
				emit_execution_session(&app_clone, &execution_id_clone, sid, &thread_url);
			}),
			&app,
		)
		.await?;

		log::info!("[resume_execution] Amp execution completed for {}", execution_id);

		let prompt_passed = result_message.as_ref().map(|m| m.contains("PROMPT: PASS")).unwrap_or(false);
		let prompt_failed = result_message.as_ref().map(|m| m.contains("PROMPT: FAIL")).unwrap_or(false);
		let prompt_status = if prompt_passed {
			Some(PromptStatus::Passed)
		} else if prompt_failed {
			Some(PromptStatus::Failed)
		} else {
			None
		};

		let default_branch = repository.default_branch.as_deref().unwrap_or("main");
		let default_branch_ref = format!("origin/{}", default_branch);

		let merge_base_output = Command::new("git")
			.args(["merge-base", &branch_name, &default_branch_ref])
			.current_dir(&admin_repo_path)
			.output()?;

		let _base_commit = if merge_base_output.status.success() {
			Some(String::from_utf8(merge_base_output.stdout)?.trim().to_string())
		} else {
			None
		};

		let thread_url = format!("https://ampcode.com/threads/{}", session_id);

		let should_validate = {
			let store_state = app.state::<Mutex<Store>>();
			let store = store_state.lock().unwrap();

			store.update_execution(
				&execution_id,
				ExecutionUpdates {
					status: Some(ExecutionStatus::Completed),
					session_id: Some(session_id.clone()),
					thread_url: Some(thread_url),
					prompt_status,
					prompt_result: result_message.clone(),
					completed_at: Some(chrono::Utc::now().timestamp_millis()),
					..Default::default()
				},
			)?;

			let promptset = store.get_promptset(&execution.promptset_id)?
				.ok_or_else(|| anyhow::anyhow!("PromptSet {} not found", execution.promptset_id))?;

			promptset.validation_prompt.is_some() && prompt_status == Some(PromptStatus::Passed)
		};
		emit_execution_status(&app, &execution_id, "completed");

		if should_validate {
			let validate_execution_id = execution_id.clone();
			let app_clone = app.clone();
			let paths_clone = paths.clone();
			tokio::spawn(async move {
				if let Err(e) = validate_execution_impl(validate_execution_id, app_clone, paths_clone).await {
					log::error!("Validation failed: {}", e);
				}
			});
		}

		Ok::<(), anyhow::Error>(())
	}
	.await;

    {
        let mut active = ACTIVE_EXECUTIONS.lock().unwrap();
        active.remove(&execution_id);
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!(
                "[resume_execution] Execution {} failed: {}",
                execution_id,
                e
            );
            let is_aborted = e.to_string().contains("aborted");
            let status = if is_aborted {
                ExecutionStatus::Cancelled
            } else {
                ExecutionStatus::Failed
            };
            let status_str = if is_aborted { "cancelled" } else { "failed" };

            {
                let store_state = app.state::<Mutex<Store>>();
                let store = store_state.lock().unwrap();
                store.update_execution(
                    &execution_id,
                    ExecutionUpdates {
                        status: Some(status),
                        completed_at: Some(chrono::Utc::now().timestamp_millis()),
                        ..Default::default()
                    },
                )?;
            }
            emit_execution_status(&app, &execution_id, status_str);

            if !is_aborted {
                Err(e)
            } else {
                Ok(())
            }
        }
    }
}

#[tauri::command]
pub async fn commit_changes(
    execution_id: String,
    files: Option<Vec<String>>,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    let (session_id, promptset_id, _repository_id) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        let execution = store
            .get_execution(&execution_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", execution_id))?;

        if execution.session_id.is_none() {
            return Err(format!(
                "Cannot commit {} - no session ID found",
                execution_id
            ));
        }
        (
            execution.session_id.clone().unwrap(),
            execution.promptset_id.clone(),
            execution.repository_id.clone(),
        )
    };

    let commit_prompt = if let Some(files) = &files {
        let file_list = files
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "Please commit only the following files with an appropriate commit message:\n{}",
            file_list
        )
    } else {
        "Please commit the current changes with an appropriate commit message.".to_string()
    };

    // Emit progress message to show committing in UI
    emit_execution_progress(&app, &execution_id, "Committing files...");

    let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);
    execute_with_amp(
        &worktree_path,
        &commit_prompt,
        Some(&session_id),
        None, // No execution_key for commit operations
        None, // No abort_flag for commit operations
        None::<fn(&str)>,
        &app,
    )
    .await
    .map_err(|e| e.to_string())?;

    let repo = GitService::open(&worktree_path).map_err(|e| e.to_string())?;
    let has_uncommitted = GitService::has_uncommitted_changes(&repo).map_err(|e| e.to_string())?;

    {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();

        if has_uncommitted {
            store
                .update_execution(
                    &execution_id,
                    ExecutionUpdates {
                        commit_status: Some(CommitStatus::Uncommitted),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;

            emit_execution_commit(&app, &execution_id, "uncommitted", None, None);
        } else {
            let commit_sha = GitService::rev_parse(&repo, "HEAD").map_err(|e| e.to_string())?;
            let committed_at =
                GitService::get_commit_timestamp(&repo, "HEAD").map_err(|e| e.to_string())?;
            let parent_sha =
                GitService::get_parent_sha(&repo, "HEAD").map_err(|e| e.to_string())?;
            let branch = GitService::get_current_branch(&repo).map_err(|e| e.to_string())?;

            store
                .update_execution(
                    &execution_id,
                    ExecutionUpdates {
                        commit_status: Some(CommitStatus::Committed),
                        commit_sha: Some(commit_sha.clone()),
                        committed_at: Some(committed_at),
                        parent_sha,
                        branch,
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;

            emit_execution_commit(
                &app,
                &execution_id,
                "committed",
                Some(&commit_sha),
                Some(&committed_at),
            );
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn push_commit(
    execution_id: String,
    force: bool,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    use tauri::Manager;

    // Get execution details
    let (promptset_id, repository_id, branch, commit_sha) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        let execution = store
            .get_execution(&execution_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", execution_id))?;

        // Ensure execution has been committed
        if execution.commit_status != CommitStatus::Committed {
            return Err("Execution must be committed before pushing".to_string());
        }

        let branch = execution
            .branch
            .clone()
            .ok_or_else(|| "No branch found for execution".to_string())?;

        let commit_sha = execution
            .commit_sha
            .clone()
            .ok_or_else(|| "No commit SHA found for execution".to_string())?;

        (
            execution.promptset_id.clone(),
            execution.repository_id.clone(),
            branch,
            commit_sha,
        )
    };

    // Emit progress message
    emit_execution_progress(&app, &execution_id, "Pushing commit to remote...");

    // Push the branch
    let worktree_path = execution_worktree_path(&paths, &promptset_id, &execution_id);
    let repo = GitService::open(&worktree_path).map_err(|e| e.to_string())?;

    GitService::push_branch(&repo, "origin", &branch, force)
        .map_err(|e| format!("Failed to push branch: {}", e))?;

    emit_execution_progress(&app, &execution_id, "Push completed successfully");

    // Wait for provider to process the push
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Start CI checking automatically after push using CiProvider
    let (provider_name, provider_id) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().unwrap();
        let repository = store
            .get_repository(&repository_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Repository {} not found", repository_id))?;

        (repository.provider.clone(), repository.provider_id.clone())
    };

    // Create CI provider using the provider trait
    if let Ok(provider) =
        crate::ci::provider::create_ci_provider(&provider_name, &provider_id).await
    {
        use crate::ci::CiContext;

        // Build provider configuration
        let provider_cfg = crate::util::git::build_provider_cfg(&provider_name, &provider_id)
            .map_err(|e| e.to_string())?;

        let ctx = CiContext {
            commit_sha: commit_sha.clone(),
            branch: branch.clone(),
            provider_cfg,
        };

        // Check if CI is configured by polling once
        let ci_url = provider.get_commit_url(&ctx).map_err(|e| e.to_string())?;
        let ci_status = match provider.poll(&ctx).await {
            Ok(checks) if checks.is_empty() => {
                // No CI configured
                use crate::types::CiStatus;
                CiStatus::NotConfigured
            }
            Ok(_checks) => {
                // CI exists, set to pending
                use crate::types::CiStatus;
                CiStatus::Pending
            }
            Err(e) => {
                // API error - assume not configured to avoid false positives
                log::warn!("[push_commit] Failed to check CI @ {}: {}", commit_sha, e);
                use crate::types::CiStatus;
                CiStatus::NotConfigured
            }
        };

        // Emit CI status
        let status_str = match ci_status {
            crate::types::CiStatus::NotConfigured => "not_configured",
            crate::types::CiStatus::Pending => "pending",
            _ => "pending",
        };
        use super::executor_events::emit_execution_ci;
        emit_execution_ci(&app, &execution_id, status_str, Some(&ci_url));

        // Update database
        {
            let store_state = app.state::<Mutex<Store>>();
            let store = store_state.lock().unwrap();
            let now = chrono::Utc::now().timestamp_millis();
            store
                .update_execution(
                    &execution_id,
                    ExecutionUpdates {
                        ci_status: Some(ci_status),
                        ci_checked_at: Some(now),
                        ci_url: Some(ci_url.clone()),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn stop_execution(execution_id: String, app: tauri::AppHandle) -> Result<bool, String> {
    let exec_key = format!("exec:{}", execution_id);

    // First, set the abort flag
    let active_execs = ACTIVE_EXECUTIONS.lock().unwrap();
    if let Some(abort_flag) = active_execs.get(&execution_id) {
        *abort_flag.lock().unwrap() = true;
    }
    drop(active_execs);

    // Then kill the child process directly
    let active_children = ACTIVE_CHILDREN.lock().unwrap();
    if let Some(child) = active_children.get(&exec_key) {
        if let Ok(mut child_guard) = child.lock() {
            let _ = child_guard.kill();
        }
        drop(active_children);
        return Ok(true);
    }
    drop(active_children);

    let store_state = app.state::<Mutex<Store>>();
    let store = store_state.lock().map_err(|e| e.to_string())?;
    let execution = store
        .get_execution(&execution_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Execution {} not found", execution_id))?;

    if execution.status == ExecutionStatus::Running {
        store
            .update_execution(
                &execution_id,
                ExecutionUpdates {
                    status: Some(ExecutionStatus::Cancelled),
                    completed_at: Some(chrono::Utc::now().timestamp_millis()),
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?;
        return Ok(true);
    }

    Ok(false)
}

#[tauri::command]
pub fn stop_validation(execution_id: String, app: tauri::AppHandle) -> Result<bool, String> {
    let val_key = format!("val:{}", execution_id);

    let active = ACTIVE_VALIDATIONS.lock().unwrap();

    if let Some(abort_flag) = active.get(&execution_id) {
        *abort_flag.lock().unwrap() = true;
    }
    drop(active);

    // Kill the child process
    let active_children = ACTIVE_CHILDREN.lock().unwrap();
    if let Some(child) = active_children.get(&val_key) {
        if let Ok(mut child_guard) = child.lock() {
            let _ = child_guard.kill();
        }
        drop(active_children);
        return Ok(true);
    }
    drop(active_children);

    let store_state = app.state::<Mutex<Store>>();
    let store = store_state.lock().map_err(|e| e.to_string())?;
    let execution = store
        .get_execution(&execution_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Execution {} not found", execution_id))?;

    if execution.validation_status == Some(ValidationStatus::Running) {
        store
            .update_execution(
                &execution_id,
                ExecutionUpdates {
                    validation_status: Some(ValidationStatus::Cancelled),
                    ..Default::default()
                },
            )
            .map_err(|e| e.to_string())?;
        return Ok(true);
    }

    Ok(false)
}

pub fn reconcile_on_startup(store: &Store) -> Result<()> {
    // Get all executions that might be stuck
    let executions = store.get_all_executions()?;

    let active_execs = ACTIVE_EXECUTIONS.lock().unwrap();
    let active_vals = ACTIVE_VALIDATIONS.lock().unwrap();

    for execution in executions {
        // Reset stuck running executions
        if execution.status == ExecutionStatus::Running && !active_execs.contains_key(&execution.id)
        {
            store.update_execution(
                &execution.id,
                ExecutionUpdates {
                    status: Some(ExecutionStatus::Cancelled),
                    completed_at: Some(chrono::Utc::now().timestamp_millis()),
                    ..Default::default()
                },
            )?;
        }

        // Reset stuck running validations
        if execution.validation_status == Some(ValidationStatus::Running)
            && !active_vals.contains_key(&execution.id)
        {
            store.update_execution(
                &execution.id,
                ExecutionUpdates {
                    validation_status: Some(ValidationStatus::Cancelled),
                    ..Default::default()
                },
            )?;
        }

        // Reset stuck pending CI checks (no active tracking, so check age)
        if execution.ci_status == Some(CiStatus::Pending) {
            if let Some(checked_at) = execution.ci_checked_at {
                let threshold_minutes = store.get_ci_stuck_threshold_minutes().unwrap_or(10);
                let now = chrono::Utc::now().timestamp_millis();
                let age_minutes = (now - checked_at) / 1000 / 60;

                // If stuck for more than threshold, mark as skipped
                if age_minutes > threshold_minutes {
                    log::warn!(
						"Resetting stuck CI check for execution {} (pending for {} minutes, threshold: {})",
						execution.id,
						age_minutes,
						threshold_minutes
					);
                    store.update_execution(
                        &execution.id,
                        ExecutionUpdates {
                            ci_status: Some(CiStatus::Skipped),
                            ..Default::default()
                        },
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Manually reconcile stuck CI checks across all executions
#[tauri::command]
pub fn reconcile_stuck_ci(app: tauri::AppHandle) -> Result<usize, String> {
    let store = app.state::<Mutex<Store>>();
    let store = store.lock().map_err(|e| e.to_string())?;

    let threshold_minutes = store.get_ci_stuck_threshold_minutes().unwrap_or(10);
    let executions = store.get_all_executions().map_err(|e| e.to_string())?;
    let mut fixed = 0;

    for execution in executions {
        // Reset stuck pending CI checks
        if execution.ci_status == Some(CiStatus::Pending) {
            if let Some(checked_at) = execution.ci_checked_at {
                let now = chrono::Utc::now().timestamp_millis();
                let age_minutes = (now - checked_at) / 1000 / 60;

                if age_minutes > threshold_minutes {
                    log::warn!(
						"Resetting stuck CI check for execution {} (pending for {} minutes, threshold: {})",
						execution.id,
						age_minutes,
						threshold_minutes
					);
                    store
                        .update_execution(
                            &execution.id,
                            ExecutionUpdates {
                                ci_status: Some(CiStatus::Skipped),
                                ..Default::default()
                            },
                        )
                        .map_err(|e| e.to_string())?;
                    fixed += 1;
                }
            }
        }
    }

    Ok(fixed)
}

#[tauri::command]
pub fn stop_all_executions(revision_id: String, app: tauri::AppHandle) -> Result<usize, String> {
    let executions = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().map_err(|e| e.to_string())?;
        store
            .get_executions_by_revision(&revision_id)
            .map_err(|e| e.to_string())?
    };

    let mut stopped = 0;
    for execution in executions {
        if execution.status == ExecutionStatus::Running {
            let stop_execution_id = execution.id.clone();
            let result = stop_execution(stop_execution_id, app.clone())?;
            if result {
                stopped += 1;
            }
        }
    }

    Ok(stopped)
}

#[tauri::command]
pub fn stop_all_validations(revision_id: String, app: tauri::AppHandle) -> Result<usize, String> {
    let executions = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().map_err(|e| e.to_string())?;
        store
            .get_executions_by_revision(&revision_id)
            .map_err(|e| e.to_string())?
    };

    let mut stopped = 0;
    for execution in executions {
        if execution.validation_status == Some(ValidationStatus::Running) {
            let stop_execution_id = execution.id;
            if stop_validation(stop_execution_id, app.clone())? {
                stopped += 1;
            }
        }
    }

    Ok(stopped)
}

#[tauri::command]
pub async fn cleanup_execution(
    execution_id: String,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<(), String> {
    let (repository, promptset_id, branch) = {
        let store_state = app.state::<Mutex<Store>>();
        let store = store_state.lock().map_err(|e| e.to_string())?;

        let execution = store
            .get_execution(&execution_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Execution {} not found", execution_id))?;

        let repository = store
            .get_repository(&execution.repository_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Repository {} not found", execution.repository_id))?;

        (
            repository,
            execution.promptset_id.clone(),
            execution.branch.clone(),
        )
    };

    let (owner, repo) = parse_provider_id(&repository.provider_id).map_err(|e| e.to_string())?;

    let admin_repo_path = admin_repo_path(&paths, &owner, &repo);

    remove_worktree(
        &admin_repo_path,
        &paths.worktree_dir,
        &promptset_id,
        &execution_id,
        branch.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_execution_modified_files(
    execution_id: String,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<ModifiedFilesResponse, String> {
    let store_state = app.state::<Mutex<Store>>();
    let store = store_state.lock().map_err(|e| e.to_string())?;
    let execution = store
        .get_execution(&execution_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Execution {} not found", execution_id))?;

    let worktree_path = paths
        .worktree_dir
        .join(&execution.promptset_id)
        .join(&execution_id);

    if execution.commit_status == CommitStatus::Committed
        && execution.commit_sha.is_some()
        && execution.parent_sha.is_some()
    {
        let repository = store
            .get_repository(&execution.repository_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Repository not found".to_string())?;

        let (owner, repo_name) =
            parse_provider_id(&repository.provider_id).map_err(|e| e.to_string())?;
        let admin_repo_path = admin_repo_path(&paths, &owner, &repo_name);

        return get_committed_diff(
            &admin_repo_path,
            execution.parent_sha.as_ref().unwrap(),
            execution.commit_sha.as_ref().unwrap(),
        )
        .map_err(|e| e.to_string());
    }

    if !worktree_path.exists() {
        return Err(format!("Worktree not found at {:?}", worktree_path));
    }

    get_worktree_diff(&worktree_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_execution_file_diff(
    execution_id: String,
    file: String,
    app: tauri::AppHandle,
    paths: tauri::State<'_, Paths>,
) -> Result<String, String> {
    let store_state = app.state::<Mutex<Store>>();
    let store = store_state.lock().map_err(|e| e.to_string())?;
    let execution = store
        .get_execution(&execution_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Execution {} not found", execution_id))?;

    let worktree_path = paths
        .worktree_dir
        .join(&execution.promptset_id)
        .join(&execution_id);

    if execution.commit_status == CommitStatus::Committed
        && execution.commit_sha.is_some()
        && execution.parent_sha.is_some()
    {
        let repository = store
            .get_repository(&execution.repository_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Repository not found".to_string())?;

        let (owner, repo_name) =
            parse_provider_id(&repository.provider_id).map_err(|e| e.to_string())?;
        let admin_repo_path = admin_repo_path(&paths, &owner, &repo_name);

        return get_committed_file_diff(
            &admin_repo_path,
            execution.parent_sha.as_ref().unwrap(),
            execution.commit_sha.as_ref().unwrap(),
            &file,
        )
        .map_err(|e| e.to_string());
    }

    if !worktree_path.exists() {
        return Err(format!("Worktree not found at {:?}", worktree_path));
    }

    get_worktree_file_diff(&worktree_path, &file).map_err(|e| e.to_string())
}
