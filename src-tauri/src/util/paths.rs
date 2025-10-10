use std::path::PathBuf;
use crate::Paths;

/// Construct path to admin repository clone
/// Format: {admin_repo_dir}/{owner}/{repo}
pub fn admin_repo_path(paths: &Paths, owner: &str, repo: &str) -> PathBuf {
	paths.admin_repo_dir.join(owner).join(repo)
}

/// Construct path to execution worktree
/// Format: {worktree_dir}/{promptset_id}/{execution_id}
pub fn execution_worktree_path(paths: &Paths, promptset_id: &str, execution_id: &str) -> PathBuf {
	paths.worktree_dir.join(promptset_id).join(execution_id)
}

/// Construct path to execution worktree (takes references, more flexible)
pub fn worktree_path(worktree_dir: &PathBuf, promptset_id: &str, execution_id: &str) -> PathBuf {
	worktree_dir.join(promptset_id).join(execution_id)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_admin_repo_path() {
		let paths = Paths {
			admin_repo_dir: PathBuf::from("/test/repos"),
			worktree_dir: PathBuf::from("/test/executions"),
			db_path: PathBuf::from("/test/maestro.db"),
		};
		let path = admin_repo_path(&paths, "owner", "repo");
		assert_eq!(path, PathBuf::from("/test/repos/owner/repo"));
	}

	#[test]
	fn test_execution_worktree_path() {
		let paths = Paths {
			admin_repo_dir: PathBuf::from("/test/repos"),
			worktree_dir: PathBuf::from("/test/executions"),
			db_path: PathBuf::from("/test/maestro.db"),
		};
		let path = execution_worktree_path(&paths, "promptset-123", "exec-456");
		assert_eq!(path, PathBuf::from("/test/executions/promptset-123/exec-456"));
	}
}
