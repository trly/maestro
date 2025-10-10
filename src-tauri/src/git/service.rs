use git2::{
	Error as Git2Error, FetchOptions,
	RemoteCallbacks, Repository,
};
use std::path::Path;

pub(crate) struct GitService;

impl GitService {
	pub(crate) fn open(path: &Path) -> Result<Repository, Git2Error> {
		Repository::open(path)
	}

	/// Clone a repository using SSH authentication via ssh-agent.
	/// 
	/// Prerequisites:
	/// - SSH agent must be running
	/// - SSH key must be added to agent (ssh-add ~/.ssh/id_rsa or ~/.ssh/id_ed25519)
	/// - Public key must be added to GitHub account
	/// 
	/// # Arguments
	/// * `url` - SSH URL format: git@github.com:owner/repo.git
	/// * `path` - Destination path for cloned repository
	pub(crate) fn clone_repo(url: &str, path: &Path) -> Result<Repository, Git2Error> {
		let mut callbacks = RemoteCallbacks::new();
		callbacks.credentials(|_url, username_from_url, _allowed_types| {
			git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
		});

		let mut fetch_options = FetchOptions::new();
		fetch_options.remote_callbacks(callbacks);

		let mut builder = git2::build::RepoBuilder::new();
		builder.fetch_options(fetch_options);
		builder.clone(url, path)
	}

	pub(crate) fn fetch(repo: &Repository, remote_name: &str, refspecs: &[&str]) -> Result<(), Git2Error> {
		let mut remote = repo.find_remote(remote_name)?;
		let mut callbacks = RemoteCallbacks::new();
		callbacks.credentials(|_url, username_from_url, _allowed_types| {
			git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
		});

		let mut fetch_options = FetchOptions::new();
		fetch_options.remote_callbacks(callbacks);

		remote.fetch(refspecs, Some(&mut fetch_options), None)?;
		Ok(())
	}

	pub(crate) fn rev_parse(repo: &Repository, spec: &str) -> Result<String, Git2Error> {
		let obj = repo.revparse_single(spec)?;
		Ok(obj.id().to_string())
	}

	pub(crate) fn has_uncommitted_changes(repo: &Repository) -> Result<bool, Git2Error> {
		use git2::Status;
		let statuses = repo.statuses(None)?;
		
		let has_changes = statuses.iter().any(|entry| {
			let status = entry.status();
			// Ignore IGNORED files - they're not uncommitted changes
			if status.contains(Status::IGNORED) {
				return false;
			}
			
			true
		});
		
		Ok(has_changes)
	}

	pub(crate) fn get_commit_timestamp(repo: &Repository, refspec: &str) -> Result<i64, Git2Error> {
		let obj = repo.revparse_single(refspec)?;
		let commit = obj.peel_to_commit()?;
		Ok(commit.time().seconds() * 1000)
	}

	pub(crate) fn get_parent_sha(repo: &Repository, refspec: &str) -> Result<Option<String>, Git2Error> {
		let obj = repo.revparse_single(refspec)?;
		let commit = obj.peel_to_commit()?;
		
		if commit.parent_count() > 0 {
			Ok(Some(commit.parent(0)?.id().to_string()))
		} else {
			Ok(None)
		}
	}

	pub(crate) fn get_current_branch(repo: &Repository) -> Result<Option<String>, Git2Error> {
		let head = repo.head()?;
		
		if head.is_branch() {
			Ok(head.shorthand().map(|s| s.to_string()))
		} else {
			Ok(None)
		}
	}

	pub(crate) fn delete_local_branch(repo: &Repository, name: &str, force: bool) -> Result<(), Git2Error> {
		use git2::BranchType;
		let mut branch = repo.find_branch(name, BranchType::Local)?;
		
		if !force {
			if !branch.is_head() {
				branch.delete()?;
			} else {
				return Err(Git2Error::from_str("Cannot delete the currently checked out branch"));
			}
		} else {
			branch.delete()?;
		}

		Ok(())
	}
}
