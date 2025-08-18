use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn create_worktree(base_dir: &Path, branch: &str) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(base_dir)
        .output()
        .context("Failed to determine git repository root")?;
    if !output.status.success() {
        anyhow::bail!("Not a git repository");
    }
    let root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    let worktrees_dir = root.join(".csb-worktrees");
    fs::create_dir_all(&worktrees_dir).context("Failed to create worktrees directory")?;
    let worktree_path = worktrees_dir.join(branch);
    let status = Command::new("git")
        .args([
            "worktree",
            "add",
            "--force",
            worktree_path.to_str().unwrap(),
            branch,
        ])
        .current_dir(&root)
        .status()
        .context("Failed to add git worktree")?;
    if !status.success() {
        anyhow::bail!("git worktree add failed");
    }
    Ok(worktree_path)
}
