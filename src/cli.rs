use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "codesandbox")]
#[command(about = "Code Sandbox - Docker container manager")]
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
        long,
        value_enum,
        default_value_t = Agent::Claude,
        help = "Agent to start in the container (claude, gemini, codex, qwen)",
    )]
    pub agent: Agent,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List containers for this directory and optionally attach to one")]
    Ls,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Agent {
    Claude,
    Gemini,
    Codex,
    Qwen,
}

impl Agent {
    pub fn command(&self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Gemini => "gemini",
            Agent::Codex => "codex",
            Agent::Qwen => "qwen",
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
        };
        write!(f, "{}", name)
    }
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
