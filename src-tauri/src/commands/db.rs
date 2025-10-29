use crate::db::store::{Execution, ExecutionUpdates, PromptRevision, PromptSet, Repository, Store};
use crate::git::GitProviderContext;
use std::sync::Mutex;
use tauri::State;

type StoreState<'a> = State<'a, Mutex<Store>>;

#[tauri::command]
pub async fn sync_repository_metadata(store: StoreState<'_>, id: String) -> Result<(), String> {
    let repository = store
        .lock()
        .unwrap()
        .get_repository(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Repository not found".to_string())?;

    // Create the appropriate git provider
    let provider = crate::git::git_provider::create_git_provider(
        &repository.provider,
        &repository.provider_id,
    )
    .await
    .map_err(|e| format!("Failed to create git provider: {}", e))?;

    // Build provider configuration
    let ctx = GitProviderContext {
        provider_cfg: crate::util::git::build_provider_cfg(
            &repository.provider,
            &repository.provider_id,
        )
        .map_err(|e| format!("Failed to build provider config: {}", e))?,
    };

    // Fetch default branch from provider
    let default_branch = provider
        .fetch_default_branch(&ctx)
        .await
        .map_err(|e| format!("Failed to fetch default branch: {}", e))?;

    store
        .lock()
        .unwrap()
        .update_repository_default_branch(&id, &default_branch)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn create_repository(
    store: StoreState<'_>,
    provider: String,
    provider_id: String,
) -> Result<Repository, String> {
    let mut repo = store
        .lock()
        .unwrap()
        .create_repository(&provider, &provider_id)
        .map_err(|e| e.to_string())?;

    // Fetch default branch using GitProvider
    if let Ok(git_provider) =
        crate::git::git_provider::create_git_provider(&provider, &provider_id).await
    {
        if let Ok(provider_cfg) = crate::util::git::build_provider_cfg(&provider, &provider_id) {
            let ctx = GitProviderContext { provider_cfg };
            if let Ok(default_branch) = git_provider.fetch_default_branch(&ctx).await {
                store
                    .lock()
                    .unwrap()
                    .update_repository_default_branch(&repo.id, &default_branch)
                    .map_err(|e| e.to_string())?;
                repo.default_branch = Some(default_branch);
            }
        }
    }

    Ok(repo)
}

#[tauri::command]
pub async fn update_repository_name(
    store: StoreState<'_>,
    id: String,
    name: String,
) -> Result<(), String> {
    store
        .lock()
        .unwrap()
        .update_repository_name(&id, &name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_repository(
    store: StoreState<'_>,
    id: String,
) -> Result<Option<Repository>, String> {
    store
        .lock()
        .unwrap()
        .get_repository(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_repository(
    store: StoreState<'_>,
    provider: String,
    provider_id: String,
) -> Result<Option<Repository>, String> {
    store
        .lock()
        .unwrap()
        .find_repository(&provider, &provider_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_repositories(store: StoreState<'_>) -> Result<Vec<Repository>, String> {
    store
        .lock()
        .unwrap()
        .get_all_repositories()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_promptset(
    store: StoreState<'_>,
    name: String,
    repository_ids: Vec<String>,
    validation_prompt: Option<String>,
    auto_validate: bool,
) -> Result<PromptSet, String> {
    store
        .lock()
        .unwrap()
        .create_promptset(&name, repository_ids, validation_prompt, auto_validate)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_promptsets(store: StoreState<'_>) -> Result<Vec<PromptSet>, String> {
    store
        .lock()
        .unwrap()
        .get_all_promptsets()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_promptset(store: StoreState<'_>, id: String) -> Result<Option<PromptSet>, String> {
    store
        .lock()
        .unwrap()
        .get_promptset(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_promptset_by_prefix(
    store: StoreState<'_>,
    id_prefix: String,
) -> Result<Option<PromptSet>, String> {
    store
        .lock()
        .unwrap()
        .find_promptset_by_prefix(&id_prefix)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_promptset_validation(
    store: StoreState<'_>,
    id: String,
    validation_prompt: Option<String>,
) -> Result<(), String> {
    store
        .lock()
        .unwrap()
        .update_promptset_validation(&id, validation_prompt)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_promptset_auto_validate(
    store: StoreState<'_>,
    id: String,
    auto_validate: bool,
) -> Result<(), String> {
    store
        .lock()
        .unwrap()
        .update_promptset_auto_validate(&id, auto_validate)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_promptset_repositories(
    store: StoreState<'_>,
    id: String,
    repository_ids: Vec<String>,
) -> Result<(), String> {
    store
        .lock()
        .unwrap()
        .update_promptset_repositories(&id, repository_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_prompt_revision(
    store: StoreState<'_>,
    promptset_id: String,
    prompt_text: String,
    parent_revision_id: Option<String>,
) -> Result<PromptRevision, String> {
    store
        .lock()
        .unwrap()
        .create_prompt_revision(&promptset_id, &prompt_text, parent_revision_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_prompt_revision(
    store: StoreState<'_>,
    id: String,
) -> Result<Option<PromptRevision>, String> {
    store
        .lock()
        .unwrap()
        .get_prompt_revision(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_prompt_revision_by_prefix(
    store: StoreState<'_>,
    id_prefix: String,
) -> Result<Option<PromptRevision>, String> {
    store
        .lock()
        .unwrap()
        .find_prompt_revision_by_prefix(&id_prefix)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_promptset_revisions(
    store: StoreState<'_>,
    promptset_id: String,
) -> Result<Vec<PromptRevision>, String> {
    store
        .lock()
        .unwrap()
        .get_promptset_revisions(&promptset_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_execution(
    store: StoreState<'_>,
    promptset_id: String,
    revision_id: String,
    repository_id: String,
) -> Result<Execution, String> {
    store
        .lock()
        .unwrap()
        .create_execution(&promptset_id, &revision_id, &repository_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_execution(
    store: StoreState<'_>,
    id: String,
    updates: ExecutionUpdates,
) -> Result<(), String> {
    store
        .lock()
        .unwrap()
        .update_execution(&id, updates)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_execution(store: StoreState<'_>, id: String) -> Result<Option<Execution>, String> {
    store
        .lock()
        .unwrap()
        .get_execution(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn find_execution_by_prefix(
    store: StoreState<'_>,
    id_prefix: String,
) -> Result<Option<Execution>, String> {
    store
        .lock()
        .unwrap()
        .find_execution_by_prefix(&id_prefix)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_executions_by_revision(
    store: StoreState<'_>,
    revision_id: String,
) -> Result<Vec<Execution>, String> {
    store
        .lock()
        .unwrap()
        .get_executions_by_revision(&revision_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_executions_by_promptset(
    store: StoreState<'_>,
    promptset_id: String,
) -> Result<Vec<Execution>, String> {
    store
        .lock()
        .unwrap()
        .get_executions_by_promptset(&promptset_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_execution(store: StoreState<'_>, id: String) -> Result<bool, String> {
    store
        .lock()
        .unwrap()
        .delete_execution(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_promptset(store: StoreState<'_>, id: String) -> Result<bool, String> {
    store
        .lock()
        .unwrap()
        .delete_promptset(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_repository(store: StoreState<'_>, id: String) -> Result<bool, String> {
    store
        .lock()
        .unwrap()
        .delete_repository(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_prompt_revision(store: StoreState<'_>, id: String) -> Result<bool, String> {
    store
        .lock()
        .unwrap()
        .delete_prompt_revision(&id)
        .map_err(|e| e.to_string())
}
