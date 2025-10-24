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