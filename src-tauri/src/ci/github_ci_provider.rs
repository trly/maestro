use super::provider::{CiCheck, CiContext, CiProvider};
use crate::types::CiStatus;
use anyhow::Result;
use octocrab::models::checks::CheckRun;
use octocrab::Octocrab;

#[derive(Clone)]
pub struct GitHubCiProvider {
    octocrab: Octocrab,
}

impl GitHubCiProvider {
    pub fn new(token: String) -> Result<Self> {
        let octocrab = Octocrab::builder().personal_token(token).build()?;
        Ok(Self { octocrab })
    }

    async fn get_all_checks(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        let mut checks = Vec::new();
        let mut statuses_err: Option<anyhow::Error> = None;
        let mut runs_err: Option<anyhow::Error> = None;

        match self.get_commit_statuses(ctx).await {
            Ok(mut statuses) => checks.append(&mut statuses),
            Err(e) => {
                log::warn!(
                    "Failed to fetch commit statuses for {}/{} @ {}: {:?}",
                    ctx.owner,
                    ctx.repo,
                    ctx.commit_sha,
                    e
                );
                statuses_err = Some(e);
            }
        }

        match self.get_check_runs(ctx).await {
            Ok(mut runs) => checks.append(&mut runs),
            Err(e) => {
                log::warn!(
                    "Failed to fetch check runs for {}/{} @ {}: {:?}",
                    ctx.owner,
                    ctx.repo,
                    ctx.commit_sha,
                    e
                );
                runs_err = Some(e);
            }
        }

        if checks.is_empty() && statuses_err.is_some() && runs_err.is_some() {
            return Err(anyhow::anyhow!("Both status and checks API calls failed"));
        }

        Ok(checks)
    }

    async fn get_commit_statuses(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        let route = format!(
            "/repos/{}/{}/commits/{}/status",
            ctx.owner, ctx.repo, ctx.commit_sha
        );

        #[derive(serde::Deserialize)]
        struct CombinedStatus {
            #[allow(dead_code)]
            state: String,
            statuses: Vec<CommitStatus>,
        }

        #[derive(serde::Deserialize)]
        struct CommitStatus {
            id: Option<i64>,
            context: Option<String>,
            state: String,
            description: Option<String>,
            target_url: Option<String>,
        }

        let combined: CombinedStatus = self.octocrab.get(route, None::<&()>).await?;

        let mut checks = Vec::new();
        for status in combined.statuses {
            let context = status
                .context
                .clone()
                .unwrap_or_else(|| "unknown".to_string());
            let state = match status.state.as_str() {
                "success" => CiStatus::Passed,
                "failure" | "error" => CiStatus::Failed,
                "pending" => CiStatus::Pending,
                _ => CiStatus::Pending,
            };

            checks.push(CiCheck {
                provider: "github".to_string(),
                context: format!("status:{}", context),
                name: context,
                state,
                target_url: status.target_url,
                description: status.description,
                external_id: status.id.map(|id| id.to_string()),
                raw_json: None,
            });
        }

        Ok(checks)
    }

    async fn get_check_runs(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        let route = format!(
            "/repos/{}/{}/commits/{}/check-runs",
            ctx.owner, ctx.repo, ctx.commit_sha
        );

        #[derive(serde::Deserialize)]
        struct CheckRunsResponse {
            #[allow(dead_code)]
            total_count: u32,
            check_runs: Vec<CheckRun>,
        }

        let check_runs: CheckRunsResponse = self.octocrab.get(route, None::<&()>).await?;

        let mut checks = Vec::new();
        for run in check_runs.check_runs {
            checks.push(CiCheck {
                provider: "github".to_string(),
                context: format!("check:{}", run.name),
                name: run.name.clone(),
                state: map_check_run_to_state(&run),
                target_url: run.html_url.clone(),
                description: None,
                external_id: Some(run.id.to_string()),
                raw_json: None,
            });
        }

        Ok(checks)
    }
}

#[async_trait::async_trait]
impl CiProvider for GitHubCiProvider {
    fn id(&self) -> &'static str {
        "github"
    }

    fn display_name(&self) -> &'static str {
        "GitHub"
    }

    async fn poll(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        self.get_all_checks(ctx).await
    }

    fn get_commit_url(&self, commit_sha: &str) -> String {
        format!("https://github.com/commit/{}", commit_sha)
    }
}

fn map_check_run_to_state(run: &CheckRun) -> CiStatus {
    match run.conclusion.as_deref() {
        Some("success") | Some("neutral") => CiStatus::Passed,
        Some("skipped") | Some("stale") => CiStatus::Skipped,
        Some("failure") | Some("cancelled") | Some("timed_out") | Some("action_required") => {
            CiStatus::Failed
        }
        None => CiStatus::Pending,
        Some(_) => CiStatus::Pending,
    }
}
