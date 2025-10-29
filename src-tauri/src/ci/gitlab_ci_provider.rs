use super::provider::{CiCheck, CiContext, CiProvider};
use crate::types::CiStatus;
use anyhow::Result;
use gitlab::api::projects::pipelines::Pipelines;
use gitlab::api::AsyncQuery;
use gitlab::{AsyncGitlab, GitlabBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitLabCiConfig {
    pub project_id: String,
    #[serde(default = "default_gitlab_web_base")]
    pub web_base_url: String,
    pub slug: Option<String>,
}

fn default_gitlab_web_base() -> String {
    "https://gitlab.com".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
struct Pipeline {
    id: u64,
    status: String,
    web_url: String,
    #[serde(rename = "ref")]
    ref_name: Option<String>,
}

#[derive(Clone)]
pub struct GitLabCiProvider {
    client: AsyncGitlab,
}

impl GitLabCiProvider {
    pub async fn new(token: String, base_url: Option<String>) -> Result<Self> {
        let url = base_url.unwrap_or_else(|| "https://gitlab.com".to_string());
        // GitlabBuilder expects URL without protocol prefix
        let url = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(&url)
            .to_string();
        let client = GitlabBuilder::new(url, token).build_async().await?;
        Ok(Self { client })
    }

    async fn get_pipeline_status(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        let cfg: GitLabCiConfig = ctx.cfg()?;

        let endpoint = Pipelines::builder()
            .project(cfg.project_id)
            .sha(&ctx.commit_sha)
            .build()?;

        let pipelines: Vec<Pipeline> = endpoint.query_async(&self.client).await?;

        let checks: Vec<CiCheck> = pipelines
            .into_iter()
            .map(|pipeline| {
                let status = map_gitlab_status(&pipeline.status);
                CiCheck {
                    provider: "gitlab".to_string(),
                    context: format!("gitlab:pipeline:{}", pipeline.id),
                    name: format!("Pipeline #{}", pipeline.id),
                    state: status,
                    target_url: Some(pipeline.web_url.clone()),
                    description: pipeline.ref_name.clone(),
                    external_id: Some(pipeline.id.to_string()),
                    raw_json: serde_json::to_value(&pipeline).ok(),
                }
            })
            .collect();

        Ok(checks)
    }
}

#[async_trait::async_trait]
impl CiProvider for GitLabCiProvider {
    fn id(&self) -> &'static str {
        "gitlab"
    }

    fn display_name(&self) -> &'static str {
        "GitLab"
    }

    async fn poll(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        self.get_pipeline_status(ctx).await
    }

    fn get_commit_url(&self, ctx: &CiContext) -> Result<String> {
        let cfg: GitLabCiConfig = ctx.cfg()?;

        if let Some(slug) = &cfg.slug {
            Ok(format!(
                "{}/{}/-/commit/{}",
                cfg.web_base_url.trim_end_matches('/'),
                slug,
                ctx.commit_sha
            ))
        } else {
            Err(anyhow::anyhow!(
                "provider_cfg.slug required to build commit URL"
            ))
        }
    }
}

fn map_gitlab_status(status: &str) -> CiStatus {
    match status {
        "success" => CiStatus::Passed,
        "failed" => CiStatus::Failed,
        "canceled" => CiStatus::Failed,
        "skipped" => CiStatus::Skipped,
        "running" | "pending" | "created" | "manual" | "scheduled" => CiStatus::Pending,
        _ => CiStatus::Pending,
    }
}
