use git2::{Error as Git2Error, FetchOptions, PushOptions, RemoteCallbacks, Repository};
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

    pub(crate) fn fetch(
        repo: &Repository,
        remote_name: &str,
        refspecs: &[&str],
    ) -> Result<(), Git2Error> {
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

    pub(crate) fn get_parent_sha(
        repo: &Repository,
        refspec: &str,
    ) -> Result<Option<String>, Git2Error> {
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

    pub(crate) fn delete_local_branch(
        repo: &Repository,
        name: &str,
        force: bool,
    ) -> Result<(), Git2Error> {
        use git2::BranchType;
        let mut branch = repo.find_branch(name, BranchType::Local)?;

        if !force {
            if !branch.is_head() {
                branch.delete()?;
            } else {
                return Err(Git2Error::from_str(
                    "Cannot delete the currently checked out branch",
                ));
            }
        } else {
            branch.delete()?;
        }

        Ok(())
    }

    /// Push a branch to remote using SSH authentication via ssh-agent.
    ///
    /// Prerequisites:
    /// - SSH agent must be running
    /// - SSH key must be added to agent
    /// - Public key must be added to GitHub account
    ///
    /// # Arguments
    /// * `repo` - Repository to push from
    /// * `remote_name` - Name of remote (usually "origin")
    /// * `branch_name` - Name of local branch to push
    /// * `force` - Whether to force push (use with caution)
    pub(crate) fn push_branch(
        repo: &Repository,
        remote_name: &str,
        branch_name: &str,
        force: bool,
    ) -> Result<(), Git2Error> {
        let mut remote = repo.find_remote(remote_name)?;
        let mut callbacks = RemoteCallbacks::new();

        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        let refspec = if force {
            format!("+refs/heads/{}:refs/heads/{}", branch_name, branch_name)
        } else {
            format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)
        };

        remote.push(&[&refspec], Some(&mut push_options))?;
        Ok(())
    }
}

/// Check if a commit exists on the remote
///
/// # Arguments
/// * `repo` - Repository to check
/// * `remote_name` - Name of remote (usually "origin")
/// * `commit_sha` - SHA of the commit to check
pub(crate) fn is_commit_on_remote(
    repo: &Repository,
    remote_name: &str,
    commit_sha: &str,
) -> Result<bool, Git2Error> {
    let remote = repo.find_remote(remote_name)?;
    let remote_name_str = remote
        .name()
        .ok_or_else(|| Git2Error::from_str("Invalid remote name"))?;

    // Try to find the commit in any remote tracking branch
    let branches = repo.branches(Some(git2::BranchType::Remote))?;

    for branch_result in branches {
        let (branch, _) = branch_result?;
        let branch_name = branch.name()?.unwrap_or("");

        // Only check branches for this remote
        if !branch_name.starts_with(&format!("{}/", remote_name_str)) {
            continue;
        }

        if let Some(oid) = branch.get().target() {
            // Walk the commit history of this remote branch
            let mut revwalk = repo.revwalk()?;
            revwalk.push(oid)?;

            for rev_oid in revwalk {
                if rev_oid?.to_string() == commit_sha {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}
