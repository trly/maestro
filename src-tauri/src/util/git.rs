use anyhow::{bail, Result};

/// Parse provider_id in format "github.com/owner/repo", "gitlab.com/owner/repo", or "owner/repo"
/// Returns (owner, repo) tuple
pub fn parse_provider_id(provider_id: &str) -> Result<(String, String)> {
    let trimmed = provider_id
        .trim_start_matches("github.com/")
        .trim_start_matches("gitlab.com/");
    let parts: Vec<&str> = trimmed.split('/').collect();

    if parts.len() != 2 {
        bail!("Invalid provider_id format. Expected 'owner/repo', 'github.com/owner/repo', or 'gitlab.com/owner/repo', got '{}'", provider_id);
    }

    if parts[0].is_empty() || parts[1].is_empty() {
        bail!("Invalid provider_id: owner and repo cannot be empty");
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Build provider_cfg JSON for CI/Git provider context
/// - GitHub: {"owner": "...", "repo": "..."}
/// - GitLab: {"project_id": "owner/repo", "slug": "owner/repo", "web_base_url": "..."}
pub fn build_provider_cfg(provider: &str, provider_id: &str) -> Result<serde_json::Value> {
    let (owner, repo) = parse_provider_id(provider_id)?;

    match provider {
        "github" => Ok(serde_json::json!({
            "owner": owner,
            "repo": repo,
        })),
        "gitlab" => {
            use crate::commands::tokens::get_token_value;

            // GitLab API accepts project_id as either numeric ID or "owner/repo" slug
            let slug = format!("{}/{}", owner, repo);

            // Get custom GitLab instance URL if configured
            let web_base_url = get_token_value("gitlab_instance_url")
                .ok()
                .flatten()
                .unwrap_or_else(|| "https://gitlab.com".to_string());

            Ok(serde_json::json!({
                "project_id": slug.clone(),
                "slug": slug,
                "web_base_url": web_base_url,
            }))
        }
        _ => bail!("Unsupported provider: {}", provider),
    }
}

/// Generate maestro branch name from short hashes
/// Format: maestro/{promptset}/{revision}/{execution}
pub fn maestro_branch_name(promptset_id: &str, revision_id: &str, execution_id: &str) -> String {
    let promptset_short = &promptset_id[..8.min(promptset_id.len())];
    let revision_short = &revision_id[..8.min(revision_id.len())];
    let execution_short = &execution_id[..8.min(execution_id.len())];
    format!(
        "maestro/{}/{}/{}",
        promptset_short, revision_short, execution_short
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_provider_id_valid() {
        let (owner, repo) = parse_provider_id("owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_provider_id_with_github_prefix() {
        let (owner, repo) = parse_provider_id("github.com/owner/repo").unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_provider_id_invalid_format() {
        assert!(parse_provider_id("invalid").is_err());
        assert!(parse_provider_id("too/many/parts").is_err());
        assert!(parse_provider_id("/repo").is_err());
        assert!(parse_provider_id("owner/").is_err());
    }

    #[test]
    fn test_maestro_branch_name() {
        let branch = maestro_branch_name(
            "12345678-1234-1234-1234-123456789012",
            "87654321-4321-4321-4321-210987654321",
            "abcdefab-abcd-abcd-abcd-abcdefabcdef",
        );
        assert_eq!(branch, "maestro/12345678/87654321/abcdefab");
    }
}
