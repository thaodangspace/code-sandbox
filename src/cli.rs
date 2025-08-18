use clap::Parser;

#[derive(Parser)]
#[command(name = "codesandbox")]
#[command(about = "Code Sandbox - Docker container manager")]
pub struct Cli {
    #[arg(long, help = "Resume the last created container")]
    pub continue_: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}