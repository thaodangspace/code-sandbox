use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "codesandbox")]
#[command(about = "Code Sandbox - Docker container manager")]
pub struct Cli {
    #[arg(
        long,
        help = "Resume the last created container",
        conflicts_with_all = ["cleanup", "command"]
    )]
    pub continue_: bool,

    #[arg(
        long,
        help = "Remove all containers created from this directory",
        conflicts_with_all = ["continue_", "command"]
    )]
    pub cleanup: bool,

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
