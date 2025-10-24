pub(crate) mod provider;
mod github_ci_provider;
mod status_checker;

pub use provider::{CiProvider, CiContext, CiCheck};
pub use github_ci_provider::GitHubCiProvider;
pub use status_checker::{check_ci_once, poll_ci_until_terminal};
