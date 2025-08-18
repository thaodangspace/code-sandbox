#[path = "../src/config.rs"]
mod config;

#[path = "../src/container.rs"]
mod container;

use container::generate_container_name;
use tempfile::tempdir;
use std::fs;
use std::process::Command;

#[test]
fn test_generate_container_name_with_git_repo() {
    // Create a temp directory with a special name to test sanitization
    let tmp = tempdir().expect("create temp dir");
    let repo_path = tmp.path().join("My Repo!");
    fs::create_dir(&repo_path).expect("create repo dir");

    // Initialize a git repository with a custom branch
    Command::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .status()
        .expect("git init");
    Command::new("git")
        .args(["checkout", "-b", "Feature/Test"])
        .current_dir(&repo_path)
        .status()
        .expect("git checkout");
    fs::write(repo_path.join("file.txt"), "test").expect("write file");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .status()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(&repo_path)
        .status()
        .expect("git commit");

    let name = generate_container_name(&repo_path);
    let prefix = "csb-my-repo--feature-test-";
    assert!(name.starts_with(prefix));
    let ts = &name[prefix.len()..];
    assert_eq!(ts.len(), 10);
    assert!(ts.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_generate_container_name_without_git_repo() {
    let tmp = tempdir().expect("create temp dir");
    let dir_path = tmp.path().join("Another Repo");
    fs::create_dir(&dir_path).expect("create dir");

    let name = generate_container_name(&dir_path);
    let prefix = "csb-another-repo-unknown-";
    assert!(name.starts_with(prefix));
    let ts = &name[prefix.len()..];
    assert_eq!(ts.len(), 10);
    assert!(ts.chars().all(|c| c.is_ascii_digit()));
}
