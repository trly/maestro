use super::git_provider::{GitProvider, GitProviderContext, RepoMetadata};
use anyhow::Result;
use gitlab::api::projects::Project;
use gitlab::api::AsyncQuery;
use gitlab::{AsyncGitlab, GitlabBuilder};
use serde::Deserialize;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitLabGitConfig {
    pub project_id: String,
}

#[derive(Debug, Deserialize)]
struct GitLabProject {
    default_branch: Option<String>,
}

#[derive(Clone)]
pub struct GitLabGitProvider {
    client: AsyncGitlab,
}

impl GitLabGitProvider {
    pub async fn new(token: String, base_url: Option<String>) -> Result<Self> {
        let url = base_url.unwrap_or_else(|| "https://gitlab.com".to_string());

        // GitlabBuilder expects URL without protocol prefix (case-insensitive)
        let url = if let Ok(parsed) = reqwest::Url::parse(&url) {
            // Extract host from properly parsed URL
            parsed.host_str().unwrap_or("gitlab.com").to_string()
        } else {
            // Fallback: strip protocol case-insensitively
            url.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '.')
                .to_string()
        };

        let client = GitlabBuilder::new(url, token).build_async().await?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl GitProvider for GitLabGitProvider {
    fn id(&self) -> &'static str {
        "gitlab"
    }

    fn display_name(&self) -> &'static str {
        "GitLab"
    }

    async fn get_repo_metadata(&self, ctx: &GitProviderContext) -> Result<RepoMetadata> {
        let cfg: GitLabGitConfig = ctx.cfg()?;

        let endpoint = Project::builder().project(cfg.project_id).build()?;
        let project: GitLabProject = endpoint.query_async(&self.client).await?;

        Ok(RepoMetadata {
            default_branch: project.default_branch.unwrap_or_else(|| "main".to_string()),
        })
    }
}
