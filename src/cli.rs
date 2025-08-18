use clap::Parser;

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

    #[arg(long, help = "Remove all containers created from this directory")]
    pub cleanup: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
