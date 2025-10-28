use crate::commands::tokens;
use crate::sourcegraph::{RepositorySearchResult, SourcegraphClient};

#[tauri::command]
pub async fn search_sourcegraph_repositories(
    query: String,
    limit: Option<i32>,
) -> Result<RepositorySearchResult, String> {
    // Get Sourcegraph endpoint from keyring
    let endpoint = tokens::get_token_value("sourcegraph_endpoint")
        .map_err(|e| format!("Failed to access endpoint: {}", e))?
        .ok_or_else(|| {
            "Sourcegraph endpoint not configured. Please set it in Settings.".to_string()
        })?;

    // Get Sourcegraph access token from keyring
    let access_token = tokens::get_token_value("sourcegraph_token")
        .map_err(|e| format!("Failed to access token: {}", e))?
        .ok_or_else(|| {
            "Sourcegraph access token not configured. Please set it in Settings.".to_string()
        })?;

    // Create client
    let client = SourcegraphClient::new(endpoint, access_token)
        .map_err(|e| format!("Failed to create Sourcegraph client: {}", e))?;

    // Search repositories
    let limit = limit.unwrap_or(50);
    let result = client
        .search_repositories(&query, limit)
        .await
        .map_err(|e| format!("Failed to search repositories: {}", e))?;

    Ok(result)
}
