use crate::git::GitService;
use crate::types::FileStatus;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiedFile {
	pub status: FileStatus,
	pub path: String,
	pub additions: Option<u32>,
	pub deletions: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiedFilesResponse {
	pub files: Vec<ModifiedFile>,
	pub source: String,
	pub commit_sha: Option<String>,
}

pub fn get_committed_diff(
	admin_repo_path: &Path,
	parent_sha: &str,
	commit_sha: &str,
) -> Result<ModifiedFilesResponse> {
	let repo = GitService::open(admin_repo_path)
		.context("Failed to open repository")?;

	let commit = repo
		.find_commit(git2::Oid::from_str(commit_sha)?)
		.context("Failed to find commit")?;
	let parent = repo
		.find_commit(git2::Oid::from_str(parent_sha)?)
		.context("Failed to find parent commit")?;

	let commit_tree = commit.tree().context("Failed to get commit tree")?;
	let parent_tree = parent.tree().context("Failed to get parent tree")?;

	let diff = repo
		.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
		.context("Failed to create diff")?;

	let mut files = Vec::new();

	diff.foreach(
		&mut |delta, _progress| {
			let status = match delta.status() {
				git2::Delta::Added => Some(FileStatus::Added),
				git2::Delta::Deleted => Some(FileStatus::Deleted),
				git2::Delta::Modified => Some(FileStatus::Modified),
				git2::Delta::Renamed => Some(FileStatus::Renamed),
				_ => None,
			};

			if let Some(status) = status {
				let path = delta
					.new_file()
					.path()
					.or_else(|| delta.old_file().path())
					.and_then(|p| p.to_str())
					.unwrap_or("")
					.to_string();

				files.push(ModifiedFile {
					status,
					path,
					additions: None,
					deletions: None,
				});
			}
			true
		},
		None,
		None,
		None,
	)
	.context("Failed to iterate diff")?;

	let diff_stats = diff.stats().context("Failed to get diff stats")?;
	for i in 0..diff_stats.files_changed() {
		if let Some(patch) = git2::Patch::from_diff(&diff, i)? {
			let mut additions = 0u32;
			let mut deletions = 0u32;

			for hunk_idx in 0..patch.num_hunks() {
				if let Ok(num_lines) = patch.num_lines_in_hunk(hunk_idx) {
					for line_idx in 0..num_lines {
						if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
							match line.origin() {
								'+' => additions += 1,
								'-' => deletions += 1,
								_ => {}
							}
						}
					}
				}
			}

			if let Some(file) = files.get_mut(i) {
				file.additions = if additions > 0 { Some(additions) } else { None };
				file.deletions = if deletions > 0 { Some(deletions) } else { None };
			}
		}
	}

	Ok(ModifiedFilesResponse {
		files,
		source: "committed".to_string(),
		commit_sha: Some(commit_sha.to_string()),
	})
}

pub fn get_committed_file_diff(
	admin_repo_path: &Path,
	parent_sha: &str,
	commit_sha: &str,
	file_path: &str,
) -> Result<String> {
	let output = Command::new("git")
		.args(&["diff", parent_sha, commit_sha, "--", file_path])
		.current_dir(admin_repo_path)
		.output()
		.context("Failed to run git diff")?;

	if !output.status.success() {
		anyhow::bail!(
			"git diff failed: {}",
			String::from_utf8_lossy(&output.stderr)
		);
	}

	Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn get_worktree_diff(worktree_path: &Path) -> Result<ModifiedFilesResponse> {
	let repo = GitService::open(worktree_path).context("Failed to open worktree")?;

	let mut status_opts = git2::StatusOptions::new();
	status_opts.include_untracked(true);
	status_opts.recurse_untracked_dirs(true);

	let statuses = repo
		.statuses(Some(&mut status_opts))
		.context("Failed to get statuses")?;

	let diff = repo
		.diff_index_to_workdir(None, None)
		.context("Failed to create diff")?;
	let diff_stats = diff.stats().context("Failed to get diff stats")?;

	let mut files = Vec::new();

	for entry in statuses.iter() {
		let path = entry.path().unwrap_or("").to_string();

		if path.ends_with('/') {
			continue;
		}

		let status_flags = entry.status();
		let status = match status_flags {
			s if s.is_wt_new() || s.is_index_new() => FileStatus::Added,
			s if s.is_wt_deleted() || s.is_index_deleted() => FileStatus::Deleted,
			s if s.is_wt_modified() || s.is_index_modified() => FileStatus::Modified,
			s if s.is_wt_renamed() || s.is_index_renamed() => FileStatus::Renamed,
			_ => continue,
		};

		let (additions, deletions) = if status == FileStatus::Added {
			let file_path = worktree_path.join(&path);
			if let Ok(content) = std::fs::read_to_string(&file_path) {
				let line_count = content.lines().count() as u32;
				(line_count, 0)
			} else {
				(0, 0)
			}
		} else {
			(0..diff_stats.files_changed())
				.find_map(|i| {
					diff.get_delta(i).and_then(|delta| {
						let old_path = delta.old_file().path().and_then(|p| p.to_str());
						let new_path = delta.new_file().path().and_then(|p| p.to_str());

						if old_path == Some(path.as_str()) || new_path == Some(path.as_str()) {
							let patch = git2::Patch::from_diff(&diff, i).ok()??;
							let mut adds = 0;
							let mut dels = 0;

							for hunk_idx in 0..patch.num_hunks() {
								let num_lines = patch.num_lines_in_hunk(hunk_idx).ok()?;
								for line_idx in 0..num_lines {
									if let Ok(line) = patch.line_in_hunk(hunk_idx, line_idx) {
										match line.origin() {
											'+' => adds += 1,
											'-' => dels += 1,
											_ => {}
										}
									}
								}
							}

							Some((adds, dels))
						} else {
							None
						}
					})
				})
				.unwrap_or((0, 0))
		};

		files.push(ModifiedFile {
			status,
			path,
			additions: if additions > 0 { Some(additions) } else { None },
			deletions: if deletions > 0 { Some(deletions) } else { None },
		});
	}

	let commit_sha = match repo.head() {
		Ok(head) => head.target().map(|oid| oid.to_string()),
		Err(_) => None,
	};

	let source = if commit_sha.is_some() {
		"uncommitted"
	} else {
		"none"
	}
	.to_string();

	Ok(ModifiedFilesResponse {
		files,
		source,
		commit_sha,
	})
}

pub fn get_worktree_file_diff(worktree_path: &Path, file_path: &str) -> Result<String> {
	let tracked_check = Command::new("git")
		.args(&["ls-files", "--", file_path])
		.current_dir(worktree_path)
		.output()
		.context("Failed to check if file is tracked")?;

	let is_tracked = !tracked_check.stdout.is_empty();

	let output = if is_tracked {
		Command::new("git")
			.args(&["diff", "HEAD", "--", file_path])
			.current_dir(worktree_path)
			.output()
			.context("Failed to run git diff")?
	} else {
		Command::new("git")
			.args(&["diff", "--no-index", "/dev/null", file_path])
			.current_dir(worktree_path)
			.output()
			.context("Failed to run git diff")?
	};

	if !output.status.success() {
		if !is_tracked && output.status.code() == Some(1) {
		} else {
			anyhow::bail!(
				"git diff failed: {}",
				String::from_utf8_lossy(&output.stderr)
			);
		}
	}

	Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
