use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::commands::tokens::get_token_value;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckResult {
    pub success: bool,
    pub username: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn health_check_github() -> Result<HealthCheckResult, String> {
    let token = get_token_value("github_token")
        .map_err(|e| format!("Failed to access token: {}", e))?
        .ok_or_else(|| "GitHub token not configured".to_string())?;

    match Octocrab::builder().personal_token(token).build() {
        Ok(client) => match client.current().user().await {
            Ok(user) => Ok(HealthCheckResult {
                success: true,
                username: Some(user.login),
                error: None,
            }),
            Err(e) => Ok(HealthCheckResult {
                success: false,
                username: None,
                error: Some(format!("Failed to get user info: {}", e)),
            }),
        },
        Err(e) => Ok(HealthCheckResult {
            success: false,
            username: None,
            error: Some(format!("Failed to create GitHub client: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn health_check_git() -> Result<HealthCheckResult, String> {
    match Command::new("git").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_str = version
                    .trim()
                    .strip_prefix("git version ")
                    .unwrap_or(version.trim());
                Ok(HealthCheckResult {
                    success: true,
                    username: Some(version_str.to_string()),
                    error: None,
                })
            } else {
                Ok(HealthCheckResult {
                    success: false,
                    username: None,
                    error: Some("Git command failed".to_string()),
                })
            }
        }
        Err(e) => Ok(HealthCheckResult {
            success: false,
            username: None,
            error: Some(format!("Git not found: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn health_check_amp() -> Result<HealthCheckResult, String> {
    match Command::new("amp").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_str = version.trim();
                Ok(HealthCheckResult {
                    success: true,
                    username: Some(version_str.to_string()),
                    error: None,
                })
            } else {
                Ok(HealthCheckResult {
                    success: false,
                    username: None,
                    error: Some("Amp command failed".to_string()),
                })
            }
        }
        Err(e) => Ok(HealthCheckResult {
            success: false,
            username: None,
            error: Some(format!("Amp not found: {}", e)),
        }),
    }
}

#[derive(Deserialize)]
struct GitLabUser {
    username: String,
}

#[tauri::command]
pub async fn health_check_gitlab() -> Result<HealthCheckResult, String> {
    let token = get_token_value("gitlab_token")
        .map_err(|e| format!("Failed to access token: {}", e))?
        .ok_or_else(|| "GitLab token not configured".to_string())?;

    let endpoint = get_token_value("gitlab_instance_url")
        .map_err(|e| format!("Failed to access endpoint: {}", e))?
        .unwrap_or_else(|| "https://gitlab.com".to_string());

    let endpoint = endpoint.trim_end_matches('/');
    let url = format!("{}/api/v4/user", endpoint);

    let client = reqwest::Client::new();
    match client.get(&url).header("PRIVATE-TOKEN", token).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GitLabUser>().await {
                    Ok(user) => Ok(HealthCheckResult {
                        success: true,
                        username: Some(user.username),
                        error: None,
                    }),
                    Err(e) => Ok(HealthCheckResult {
                        success: false,
                        username: None,
                        error: Some(format!("Failed to parse user info: {}", e)),
                    }),
                }
            } else {
                Ok(HealthCheckResult {
                    success: false,
                    username: None,
                    error: Some(format!(
                        "HTTP {}: {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    )),
                })
            }
        }
        Err(e) => Ok(HealthCheckResult {
            success: false,
            username: None,
            error: Some(format!("Request failed: {}", e)),
        }),
    }
}

#[derive(Deserialize)]
struct SourcegraphCurrentUserResponse {
    data: SourcegraphCurrentUserData,
}

#[derive(Deserialize)]
struct SourcegraphCurrentUserData {
    #[serde(rename = "currentUser")]
    current_user: Option<SourcegraphUser>,
}

#[derive(Deserialize)]
struct SourcegraphUser {
    username: String,
}

#[tauri::command]
pub async fn health_check_sourcegraph() -> Result<HealthCheckResult, String> {
    let token = get_token_value("sourcegraph_token")
        .map_err(|e| format!("Failed to access token: {}", e))?
        .ok_or_else(|| "Sourcegraph token not configured".to_string())?;

    let endpoint = get_token_value("sourcegraph_endpoint")
        .map_err(|e| format!("Failed to access endpoint: {}", e))?
        .ok_or_else(|| "Sourcegraph endpoint not configured".to_string())?;

    let endpoint = endpoint.trim_end_matches('/');
    let url = format!("{}/.api/graphql", endpoint);

    let query = r#"
        query {
            currentUser {
                username
            }
        }
    "#;

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "query": query
    });

    match client
        .post(&url)
        .header("Authorization", format!("token {}", token))
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SourcegraphCurrentUserResponse>().await {
                    Ok(resp) => {
                        if let Some(user) = resp.data.current_user {
                            Ok(HealthCheckResult {
                                success: true,
                                username: Some(user.username),
                                error: None,
                            })
                        } else {
                            Ok(HealthCheckResult {
                                success: false,
                                username: None,
                                error: Some("No current user returned".to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(HealthCheckResult {
                        success: false,
                        username: None,
                        error: Some(format!("Failed to parse response: {}", e)),
                    }),
                }
            } else {
                Ok(HealthCheckResult {
                    success: false,
                    username: None,
                    error: Some(format!(
                        "HTTP {}: {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    )),
                })
            }
        }
        Err(e) => Ok(HealthCheckResult {
            success: false,
            username: None,
            error: Some(format!("Request failed: {}", e)),
        }),
    }
}
