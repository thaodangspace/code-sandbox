use clap::{Parser, Subcommand};
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

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List containers for this directory and optionally attach to one")]
    Ls,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
