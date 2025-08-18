use clap::Parser;

#[path = "../src/cli.rs"]
mod cli;

use cli::{Cli, Commands};

#[test]
fn parse_continue_flag() {
    let cli = Cli::parse_from(["codesandbox", "--continue"]);
    assert!(cli.continue_);
    assert!(!cli.cleanup);
    assert!(cli.add_dir.is_none());
}

#[test]
fn parse_cleanup_flag() {
    let cli = Cli::parse_from(["codesandbox", "--cleanup"]);
    assert!(cli.cleanup);
    assert!(!cli.continue_);
    assert!(cli.add_dir.is_none());
}

#[test]
fn parse_ls_subcommand() {
    let cli = Cli::parse_from(["codesandbox", "ls"]);
    assert!(matches!(cli.command, Some(Commands::Ls)));
    assert!(cli.add_dir.is_none());
}

#[test]
fn conflicting_flags_error() {
    let result = Cli::try_parse_from(["codesandbox", "--continue", "--cleanup"]);
    assert!(result.is_err());
}

#[test]
fn parse_add_dir() {
    let cli = Cli::parse_from(["codesandbox", "--add_dir", "/tmp/foo"]);
    assert_eq!(
        cli.add_dir.as_deref(),
        Some(std::path::Path::new("/tmp/foo"))
    );
}
