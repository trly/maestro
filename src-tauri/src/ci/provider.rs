use crate::types::CiStatus;
use anyhow::Result;
use std::sync::Arc;

/// Context passed to CI providers for polling
#[derive(Clone, Debug)]
pub struct CiContext {
    pub commit_sha: String,
    pub branch: String,
    pub provider_cfg: serde_json::Value,
}

impl CiContext {
    /// Deserialize provider-specific configuration from provider_cfg
    pub fn cfg<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_value(self.provider_cfg.clone())
            .map_err(|e| anyhow::anyhow!("Invalid provider_cfg: {}", e))
    }
}

/// A single CI check result from a provider
#[derive(Clone, Debug)]
pub struct CiCheck {
    pub provider: String, // e.g., "github", "jenkins"
    pub context: String,  // Unique identifier (e.g., "github_actions:build", "ci/jenkins")
    pub name: String,     // Display name
    pub state: CiStatus,  // Pending | Passed | Failed | Skipped
    pub target_url: Option<String>,
    pub description: Option<String>,
    pub external_id: Option<String>,
    pub raw_json: Option<serde_json::Value>,
}

/// Trait for CI providers
#[async_trait::async_trait]
pub trait CiProvider: Send + Sync {
    /// Unique identifier for this provider
    #[allow(dead_code)]
    fn id(&self) -> &'static str;

    /// Human-readable display name
    #[allow(dead_code)]
    fn display_name(&self) -> &'static str;

    /// Whether this provider supports polling (vs webhook-only)
    #[allow(dead_code)]
    fn supports_polling(&self) -> bool {
        true
    }

    /// Poll for CI status
    /// Returns a list of checks (can be multiple for providers like GitHub that aggregate multiple sources)
    async fn poll(&self, ctx: &CiContext) -> Result<Vec<CiCheck>>;

    /// Get URL for viewing commit CI status
    fn get_commit_url(&self, ctx: &CiContext) -> Result<String>;
}

/// Factory function to create a CI provider
pub async fn create_ci_provider(provider: &str, _provider_id: &str) -> Result<Arc<dyn CiProvider>> {
    use crate::ci::{GitHubCiProvider, GitLabCiProvider};
    use crate::commands::tokens::get_token_value;

    match provider {
        "github" => {
            let token = get_token_value("github_token")
                .map_err(|e| anyhow::anyhow!("Failed to access GitHub token: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("GitHub token not configured"))?;

            let provider = GitHubCiProvider::new(token)?;
            Ok(Arc::new(provider))
        }
        "gitlab" => {
            let token = get_token_value("gitlab_token")
                .map_err(|e| anyhow::anyhow!("Failed to access GitLab token: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("GitLab token not configured"))?;

            let base_url = get_token_value("gitlab_instance_url").ok().flatten();

            let provider = GitLabCiProvider::new(token, base_url).await?;
            Ok(Arc::new(provider))
        }
        _ => Err(anyhow::anyhow!("Unsupported CI provider: {}", provider)),
    }
}
