#[path = "../src/config.rs"]
mod config;

#[path = "../src/container.rs"]
mod container;

use container::{auto_remove_old_containers, generate_container_name};
use std::{env, fs, process::Command};
use tempfile::tempdir;

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

#[test]
fn test_auto_remove_old_containers() {
    let tmp = tempdir().expect("temp dir");
    let bin_dir = tmp.path();
    let rm_log = bin_dir.join("rm.log");

    let docker_path = bin_dir.join("docker");
    let script = r#"#!/bin/bash
set -e
cmd="$1"
shift
case "$cmd" in
  ps)
    echo "csb-old"
    echo "csb-recent"
    ;;
  inspect)
    name="${!#}"
    if [ "$name" = "csb-old" ]; then
      echo "1970-01-01T00:00:00Z"
    else
      date -u +"%Y-%m-%dT%H:%M:%SZ"
    fi
    ;;
  logs)
    name="${!#}"
    if [ "$name" = "csb-old" ]; then
      :
    else
      echo "has logs"
    fi
    ;;
  rm)
    name="${!#}"
    echo "$name" >> "__LOG__"
    ;;
  *)
    exit 1
    ;;
esac
"#
    .replace("__LOG__", rm_log.to_str().unwrap());
    fs::write(&docker_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&docker_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&docker_path, perms).unwrap();
    }

    let original_path = env::var("PATH").unwrap_or_default();
    env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    auto_remove_old_containers(1).unwrap();

    env::set_var("PATH", original_path);

    let removed = fs::read_to_string(&rm_log).unwrap();
    assert_eq!(removed.trim(), "csb-old");
}
