#[path = "../src/config.rs"]
mod config;

#[path = "../src/cli.rs"]
mod cli;

#[path = "../src/settings.rs"]
mod settings;

#[path = "../src/language.rs"]
mod language;

#[path = "../src/container/mod.rs"]
mod container;

use cli::Agent;
use container::{auto_remove_old_containers, generate_container_name};
use std::{env, fs, process::Command, sync::Mutex};
use tempfile::tempdir;

static DOCKER_LOCK: Mutex<()> = Mutex::new(());

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
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .status()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .status()
        .expect("git config name");
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

    let name = generate_container_name(&repo_path, &Agent::Claude);
    let prefix = "csb-claude-my-repo--feature-test-";
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

    let name = generate_container_name(&dir_path, &Agent::Claude);
    let prefix = "csb-claude-another-repo-unknown-";
    assert!(name.starts_with(prefix));
    let ts = &name[prefix.len()..];
    assert_eq!(ts.len(), 10);
    assert!(ts.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_auto_remove_old_containers() {
    let _lock = DOCKER_LOCK.lock().unwrap();
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

#[test]
fn test_list_all_containers() {
    let _lock = DOCKER_LOCK.lock().unwrap();
    let tmp = tempdir().expect("temp dir");
    let bin_dir = tmp.path();
    let docker_path = bin_dir.join("docker");
    let script = r#"#!/bin/bash
cmd="$1"
shift
case "$cmd" in
  ps)
    echo "csb-claude-proj-main-123456"
    echo "unrelated"
    ;;
  inspect)
    name="${!#}"
    if [ "$name" = "csb-claude-proj-main-123456" ]; then
      echo "/projects/proj"
    fi
    ;;
  *)
    exit 1
    ;;
esac
"#;
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

    let containers = container::list_all_containers().unwrap();

    env::set_var("PATH", original_path);

    assert_eq!(containers.len(), 1);
    assert_eq!(containers[0].0, "proj");
    assert_eq!(containers[0].1, "csb-claude-proj-main-123456");
    assert_eq!(containers[0].2.as_deref(), Some("/projects/proj"));
}

#[tokio::test]
async fn create_container_masks_only_existing_env_files() {
    let _lock = DOCKER_LOCK.lock().unwrap();
    let tmp = tempdir().expect("temp dir");
    let project_dir = tmp.path().join("proj");
    fs::create_dir(&project_dir).expect("create project dir");
    fs::write(project_dir.join(".env"), "SECRET=1").expect("write env");

    let bin_dir = tmp.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let run_log = tmp.path().join("run.log");
    let script = format!(
        "#!/bin/bash\ncmd=\"$1\"; shift\ncase \"$cmd\" in\n  build) exit 0 ;;\n  run) echo \"$@\" > \"{}\"; exit 0 ;;\n  exec) exit 0 ;;\n  *) exit 0 ;;\nesac\n",
        run_log.display()
    );
    let docker_path = bin_dir.join("docker");
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

    container::create_container("test", &project_dir, None, &Agent::Claude, None, false)
        .await
        .unwrap();

    env::set_var("PATH", original_path);

    let run_args = fs::read_to_string(&run_log).unwrap();
    assert!(run_args.contains(&project_dir.join(".env").display().to_string()));
    assert!(!run_args.contains(&project_dir.join(".env.local").display().to_string()));
    assert!(!run_args.contains(
        &project_dir
            .join(".env.development.local")
            .display()
            .to_string()
    ));
    assert!(!run_args.contains(&project_dir.join(".env.test.local").display().to_string()));
    assert!(!run_args.contains(
        &project_dir
            .join(".env.production.local")
            .display()
            .to_string()
    ));

    // Ensure no new env files were created on the host
    assert!(!project_dir.join(".env.local").exists());
    assert!(!project_dir.join(".env.development.local").exists());
    assert!(!project_dir.join(".env.test.local").exists());
    assert!(!project_dir.join(".env.production.local").exists());
}
