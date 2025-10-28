use super::git_provider::{GitProvider, GitProviderContext, RepoMetadata};
use anyhow::Result;
use octocrab::Octocrab;

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
        let repo_info = self.octocrab.repos(&ctx.owner, &ctx.repo).get().await?;

        Ok(RepoMetadata {
            default_branch: repo_info
                .default_branch
                .unwrap_or_else(|| "main".to_string()),
            description: repo_info.description,
        })
    }
}
