#[path = "../src/state.rs"]
mod state;

use state::{clear_last_container, load_last_container, save_last_container};
use std::env;
use tempfile::tempdir;

fn setup_temp_home() -> tempfile::TempDir {
    let dir = tempdir().expect("failed to create temp dir");
    env::set_var("HOME", dir.path());
    dir
}

#[test]
fn test_load_nonexistent_container() {
    let _dir = setup_temp_home();
    let loaded = load_last_container().expect("load should succeed");
    assert!(loaded.is_none());
}

#[test]
fn test_save_and_load_last_container() {
    let _dir = setup_temp_home();
    save_last_container("my_container").expect("save should succeed");
    let loaded = load_last_container().expect("load should succeed");
    assert_eq!(loaded, Some("my_container".to_string()));
}

#[test]
fn test_clear_last_container() {
    let _dir = setup_temp_home();
    save_last_container("to_clear").expect("save should succeed");
    clear_last_container().expect("clear should succeed");
    let loaded = load_last_container().expect("load should succeed");
    assert!(loaded.is_none());
}

