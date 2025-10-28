mod github_ci_provider;
pub(crate) mod provider;
mod status_checker;

pub use github_ci_provider::GitHubCiProvider;
pub use provider::{CiCheck, CiContext, CiProvider};
pub use status_checker::{check_ci_once, poll_ci_until_terminal};
