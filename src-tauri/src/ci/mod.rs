mod github_ci_provider;
mod gitlab_ci_provider;
pub(crate) mod provider;
mod status_checker;

pub use github_ci_provider::GitHubCiProvider;
pub use gitlab_ci_provider::GitLabCiProvider;
pub use provider::{CiCheck, CiContext, CiProvider};
pub use status_checker::{check_ci_once, poll_ci_until_terminal};
