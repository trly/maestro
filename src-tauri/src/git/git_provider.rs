use anyhow::Result;

/// Repository metadata fetched from git hosting provider
#[derive(Debug, Clone)]
pub struct RepoMetadata {
	pub default_branch: String,
	#[allow(dead_code)]
	pub description: Option<String>,
}

/// Context for git provider operations
#[derive(Debug, Clone)]
pub struct GitProviderContext {
	pub owner: String,
	pub repo: String,
}

/// Trait for git hosting provider integrations (GitHub, GitLab, etc.)
#[async_trait::async_trait]
pub trait GitProvider: Send + Sync {
	/// Unique identifier for this provider (e.g., "github", "gitlab")
	#[allow(dead_code)]
	fn id(&self) -> &'static str;
	
	/// Human-readable display name (e.g., "GitHub", "GitLab")
	#[allow(dead_code)]
	fn display_name(&self) -> &'static str;
	
	/// Fetch repository metadata (default branch, description, etc.)
	async fn get_repo_metadata(&self, ctx: &GitProviderContext) -> Result<RepoMetadata>;
	
	/// Convenience method to fetch just the default branch
	async fn fetch_default_branch(&self, ctx: &GitProviderContext) -> Result<String> {
		let metadata = self.get_repo_metadata(ctx).await?;
		Ok(metadata.default_branch)
	}
}

/// Factory function to create a git provider
pub fn create_git_provider(provider: &str, _provider_id: &str) -> Result<Box<dyn GitProvider>> {
	use crate::commands::tokens::get_token_value;
	use crate::git::GitHubGitProvider;
	
	match provider {
		"github" => {
			let token = get_token_value("github_token")
				.map_err(|e| anyhow::anyhow!("Failed to access GitHub token: {}", e))?
				.ok_or_else(|| anyhow::anyhow!("GitHub token not configured"))?;
			
			let provider = GitHubGitProvider::new(token)?;
			Ok(Box::new(provider))
		}
		_ => Err(anyhow::anyhow!("Unsupported git provider: {}", provider))
	}
}
