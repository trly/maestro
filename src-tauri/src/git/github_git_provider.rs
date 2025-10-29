use super::git_provider::{GitProvider, GitProviderContext, RepoMetadata};
use anyhow::Result;
use octocrab::Octocrab;

/// GitHub-specific configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubGitConfig {
    pub owner: String,
    pub repo: String,
}

#[derive(Clone)]
pub struct GitHubGitProvider {
    octocrab: Octocrab,
}

impl GitHubGitProvider {
    pub fn new(token: String) -> Result<Self> {
        let octocrab = Octocrab::builder().personal_token(token).build()?;
        Ok(Self { octocrab })
    }
}

#[async_trait::async_trait]
impl GitProvider for GitHubGitProvider {
    fn id(&self) -> &'static str {
        "github"
    }

    fn display_name(&self) -> &'static str {
        "GitHub"
    }

    async fn get_repo_metadata(&self, ctx: &GitProviderContext) -> Result<RepoMetadata> {
        let cfg: GitHubGitConfig = ctx.cfg()?;
        let repo_info = self.octocrab.repos(&cfg.owner, &cfg.repo).get().await?;

        Ok(RepoMetadata {
            default_branch: repo_info
                .default_branch
                .unwrap_or_else(|| "main".to_string()),
        })
    }
}
