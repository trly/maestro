pub(crate) mod diff;
pub(crate) mod service;
pub(crate) mod git_provider;
mod github_git_provider;

pub(crate) use diff::{
	get_committed_diff, get_committed_file_diff, get_worktree_diff, get_worktree_file_diff,
	ModifiedFilesResponse,
};
pub(crate) use service::GitService;
pub use git_provider::{GitProvider, GitProviderContext};
pub use github_git_provider::GitHubGitProvider;

/// Check if a commit has been pushed to the remote
pub(crate) fn is_commit_pushed(repo_path: &std::path::Path, commit_sha: &str) -> Result<bool, String> {
	let repo = GitService::open(repo_path)
		.map_err(|e| format!("Failed to open repository: {}", e))?;
	
	service::is_commit_on_remote(&repo, "origin", commit_sha)
		.map_err(|e| format!("Failed to check remote: {}", e))
}