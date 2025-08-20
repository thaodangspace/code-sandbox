use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "codesandbox")]
#[command(about = "Code Sandbox - Docker container manager")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[arg(
        long,
        help = "Resume the last created container",
        conflicts_with = "cleanup"
    )]
    pub continue_: bool,

    #[arg(
        long,
        help = "Remove all containers created from this directory",
        conflicts_with = "continue_"
    )]
    pub cleanup: bool,

    #[arg(
        long = "add_dir",
        value_name = "DIR",
        help = "Additional directory to mount read-only inside the container"
    )]
    pub add_dir: Option<PathBuf>,

    #[arg(
        long = "worktree",
        value_name = "BRANCH",
        help = "Create and use a git worktree for the specified branch"
    )]
    pub worktree: Option<String>,

    #[arg(
        long,
        value_enum,
        default_value_t = Agent::Claude,
        help = "Agent to start in the container (claude, gemini, codex, qwen, cursor)",
    )]
    pub agent: Agent,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    #[command(about = "List containers for this directory and optionally attach to one")]
    Ls,
    #[command(
        about = "List all running Code Sandbox containers and optionally attach or open their directory"
    )]
    Ps,
    #[command(about = "Start the Code Sandbox API server")]
    Serve {
        #[arg(short = 'd', long = "daemon", help = "Run server in the background")]
        daemon: bool,
        #[arg(long, help = "Stop the running server", conflicts_with = "restart")]
        stop: bool,
        #[arg(long, help = "Restart the server", conflicts_with = "stop")]
        restart: bool,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Agent {
    Claude,
    Gemini,
    Codex,
    Qwen,
    Cursor,
}

impl Agent {
    pub fn command(&self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Gemini => "gemini",
            Agent::Codex => "codex",
            Agent::Qwen => "qwen",
            Agent::Cursor => "cursor",
        }
    }
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Agent::Claude => "Claude",
            Agent::Gemini => "Gemini",
            Agent::Codex => "Codex",
            Agent::Qwen => "Qwen",
            Agent::Cursor => "Cursor",
        };
        write!(f, "{}", name)
    }
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
