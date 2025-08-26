use clap::Parser;

#[path = "../src/cli.rs"]
mod cli;

use cli::{Agent, Cli, Commands};

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
fn parse_ps_subcommand() {
    let cli = Cli::parse_from(["codesandbox", "ps"]);
    assert!(matches!(cli.command, Some(Commands::Ps)));
}

#[test]
fn parse_serve_subcommand() {
    let cli = Cli::parse_from(["codesandbox", "serve"]);
    assert!(matches!(
        cli.command,
        Some(Commands::Serve {
            daemon: false,
            stop: false,
            restart: false
        })
    ));
}

#[test]
fn parse_serve_daemon_flag() {
    let cli = Cli::parse_from(["codesandbox", "serve", "-d"]);
    assert!(matches!(
        cli.command,
        Some(Commands::Serve {
            daemon: true,
            stop: false,
            restart: false
        })
    ));
}

#[test]
fn parse_serve_stop_flag() {
    let cli = Cli::parse_from(["codesandbox", "serve", "--stop"]);
    assert!(matches!(
        cli.command,
        Some(Commands::Serve {
            daemon: false,
            stop: true,
            restart: false
        })
    ));
}

#[test]
fn parse_serve_restart_flag() {
    let cli = Cli::parse_from(["codesandbox", "serve", "--restart"]);
    assert!(matches!(
        cli.command,
        Some(Commands::Serve {
            daemon: false,
            stop: false,
            restart: true
        })
    ));
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

#[test]
fn default_agent_is_claude() {
    let cli = Cli::parse_from(["codesandbox"]);
    assert!(matches!(cli.agent, Agent::Claude));
}

#[test]
fn parse_agent_option() {
    let cli = Cli::parse_from(["codesandbox", "--agent", "qwen"]);
    assert!(matches!(cli.agent, Agent::Qwen));
}

#[test]
fn parse_shell_flag() {
    let cli = Cli::parse_from(["codesandbox", "--shell"]);
    assert!(cli.shell);
}

#[test]
fn parse_worktree_option() {
    let cli = Cli::parse_from(["codesandbox", "--worktree", "feature"]);
    assert_eq!(cli.worktree.as_deref(), Some("feature"));
}
