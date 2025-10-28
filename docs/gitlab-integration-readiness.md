# GitLab Integration Readiness

## Summary

The CI and Git provider interfaces have been refactored to be fully provider-agnostic and ready for GitLab integration.

## Changes Implemented

### 1. Provider-Agnostic Contexts

**CiContext** ([src-tauri/src/ci/provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/ci/provider.rs#L6-L17))
- ✅ Removed GitHub-specific `owner` and `repo` fields
- ✅ Kept only universal fields: `commit_sha`, `branch`, `provider_cfg`
- ✅ Added `cfg<T>()` helper for type-safe provider config deserialization

**GitProviderContext** ([src-tauri/src/git/git_provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/git/git_provider.rs#L11-L22))
- ✅ Removed GitHub-specific `owner` and `repo` fields  
- ✅ Uses only `provider_cfg` for provider-specific data
- ✅ Added `cfg<T>()` helper for type-safe deserialization

### 2. Updated Trait Methods

**CiProvider::get_commit_url()** ([src-tauri/src/ci/provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/ci/provider.rs#L55))
- ✅ Changed signature to accept `&CiContext` instead of just `commit_sha`
- ✅ Now returns `Result<String>` to handle config parsing errors
- ✅ Allows providers to build URLs from their specific config

### 3. GitHub Provider Config Structs

**GitHubCiConfig** ([src-tauri/src/ci/github_ci_provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/ci/github_ci_provider.rs#L8-L14))
```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubCiConfig {
    pub owner: String,
    pub repo: String,
    #[serde(default = "default_github_web_base")]
    pub web_base_url: String,  // Defaults to "https://github.com"
}
```

**GitHubGitConfig** ([src-tauri/src/git/github_git_provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/git/github_git_provider.rs#L6-L10))
```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubGitConfig {
    pub owner: String,
    pub repo: String,
}
```

### 4. All Call Sites Migrated

✅ Updated all `CiContext` creation sites to use `provider_cfg`:
- [commands/ci.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/commands/ci.rs#L77-L83) (2 locations)
- [commands/executor.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/commands/executor.rs#L1496-L1502)

✅ Updated all `GitProviderContext` creation sites:
- [commands/db.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/commands/db.rs#L28-L33) (2 locations)
- [commands/executor.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/commands/executor.rs#L31-L37)

✅ Updated all `get_commit_url()` call sites to pass context

## GitLab Implementation Guide

### Step 1: Create GitLab Config Structs

```rust
// In src-tauri/src/ci/gitlab_ci_provider.rs
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitLabCiConfig {
    pub project_id: u64,
    #[serde(default = "default_gitlab_web_base")]
    pub web_base_url: String,  // Defaults to "https://gitlab.com"
    pub slug: Option<String>,  // e.g., "group/subgroup/project"
}

fn default_gitlab_web_base() -> String {
    "https://gitlab.com".to_string()
}
```

```rust
// In src-tauri/src/git/gitlab_git_provider.rs
#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitLabGitConfig {
    pub project_id: u64,
    #[serde(default = "default_gitlab_api_base")]
    pub api_base_url: String,  // Defaults to "https://gitlab.com"
}

fn default_gitlab_api_base() -> String {
    "https://gitlab.com".to_string()
}
```

### Step 2: Implement GitLab Providers

Use the config structs in your implementations:

```rust
impl GitLabCiProvider {
    async fn poll(&self, ctx: &CiContext) -> Result<Vec<CiCheck>> {
        let cfg: GitLabCiConfig = ctx.cfg()?;
        
        // Use cfg.project_id, cfg.web_base_url, etc.
        let route = format!("/api/v4/projects/{}/pipelines", cfg.project_id);
        // ...
    }
    
    fn get_commit_url(&self, ctx: &CiContext) -> Result<String> {
        let cfg: GitLabCiConfig = ctx.cfg()?;
        
        if let Some(slug) = &cfg.slug {
            Ok(format!(
                "{}{}/-/commit/{}",
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
```

### Step 3: Add Factory Support

Update [src-tauri/src/ci/provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/ci/provider.rs#L56-L71):

```rust
pub async fn create_ci_provider(provider: &str, _provider_id: &str) -> Result<Arc<dyn CiProvider>> {
    use crate::commands::tokens::get_token_value;

    match provider {
        "github" => {
            let token = get_token_value("github_token")?
                .ok_or_else(|| anyhow::anyhow!("GitHub token not configured"))?;
            Ok(Arc::new(GitHubCiProvider::new(token)?))
        }
        "gitlab" => {
            let token = get_token_value("gitlab_token")?
                .ok_or_else(|| anyhow::anyhow!("GitLab token not configured"))?;
            Ok(Arc::new(GitLabCiProvider::new(token)?))
        }
        _ => Err(anyhow::anyhow!("Unsupported CI provider: {}", provider)),
    }
}
```

Update [src-tauri/src/git/git_provider.rs](file:///Users/trly/src/github.com/trly/maestro/src-tauri/src/git/git_provider.rs#L40-L55) similarly.

### Step 4: Configure Repositories

When adding a GitLab repository, populate `provider_cfg`:

```rust
CiContext {
    commit_sha: "abc123".to_string(),
    branch: "main".to_string(),
    provider_cfg: serde_json::json!({
        "project_id": 12345,
        "slug": "group/subgroup/my-repo",
        "web_base_url": "https://gitlab.example.com"  // Optional, for self-hosted
    }),
}
```

### Step 5: Status Mapping

Map GitLab pipeline statuses to `CiStatus`:

```rust
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
```

## Benefits of This Architecture

1. **Clean Separation**: Provider-specific details isolated in `provider_cfg`
2. **Type Safety**: Each provider deserializes its own typed config
3. **Flexibility**: Supports self-hosted instances via `web_base_url`/`api_base_url`
4. **No Coupling**: Shared contexts remain provider-agnostic
5. **Easy Testing**: Provider configs are simple, serializable structs

## Token Storage

GitLab tokens should be stored in the system keyring with key `"gitlab_token"` using the existing `tokens` module.

For multiple GitLab instances, consider using `"gitlab_token:{provider_id}"` with fallback to `"gitlab_token"`.
