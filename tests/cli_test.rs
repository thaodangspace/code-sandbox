use clap::Parser;

#[path = "../src/cli.rs"]
mod cli;

use cli::{Cli, Commands};

#[test]
fn parse_continue_flag() {
    let cli = Cli::parse_from(["codesandbox", "--continue"]);
    assert!(cli.continue_);
    assert!(!cli.cleanup);
}

#[test]
fn parse_cleanup_flag() {
    let cli = Cli::parse_from(["codesandbox", "--cleanup"]);
    assert!(cli.cleanup);
    assert!(!cli.continue_);
}

#[test]
fn parse_ls_subcommand() {
    let cli = Cli::parse_from(["codesandbox", "ls"]);
    assert!(matches!(cli.command, Some(Commands::Ls)));
}

#[test]
fn conflicting_flags_error() {
    let result = Cli::try_parse_from(["codesandbox", "--continue", "--cleanup"]);
    assert!(result.is_err());
}
