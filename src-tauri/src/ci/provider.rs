use anyhow::Result;
use std::sync::Arc;
use crate::types::CiStatus;

/// Context passed to CI providers for polling
#[derive(Clone, Debug)]
pub struct CiContext {
    pub owner: String,
    pub repo: String,
    pub commit_sha: String,
    #[allow(dead_code)]
    pub branch: String,
    #[allow(dead_code)]
    pub provider_cfg: serde_json::Value,
}

/// A single CI check result from a provider
#[derive(Clone, Debug)]
pub struct CiCheck {
    pub provider: String,      // e.g., "github", "jenkins"
    pub context: String,        // Unique identifier (e.g., "github_actions:build", "ci/jenkins")
    pub name: String,           // Display name
    pub state: CiStatus,        // Pending | Passed | Failed | Skipped
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
    fn get_commit_url(&self, commit_sha: &str) -> String;
}

/// Factory function to create a CI provider
pub async fn create_ci_provider(provider: &str, _provider_id: &str) -> Result<Arc<dyn CiProvider>> {
    use crate::commands::tokens::get_token_value;
    use crate::ci::GitHubCiProvider;
    
    match provider {
        "github" => {
            let token = get_token_value("github_token")
                .map_err(|e| anyhow::anyhow!("Failed to access GitHub token: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("GitHub token not configured"))?;
            
            let provider = GitHubCiProvider::new(token)?;
            Ok(Arc::new(provider))
        }
        _ => Err(anyhow::anyhow!("Unsupported CI provider: {}", provider))
    }
}
