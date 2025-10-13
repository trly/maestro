mod provider;
mod github_provider;
mod status_checker;

pub use provider::{CiProvider, CiContext, CiCheck};
pub use github_provider::GitHubProvider;
pub use status_checker::{check_ci_once, poll_ci_until_terminal};
