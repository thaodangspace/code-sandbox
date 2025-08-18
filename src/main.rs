mod cli;
mod config;
mod container;
mod state;

use anyhow::{Context, Result};
use std::env;

use cli::Cli;
use container::{
    check_docker_availability, cleanup_containers, create_container, generate_container_name,
    resume_container,
};
use state::{clear_last_container, load_last_container, save_last_container};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    check_docker_availability()?;

    if cli.cleanup {
        cleanup_containers(&current_dir)?;
        clear_last_container()?;
        println!(
            "Removed all Code Sandbox containers for directory {}",
            current_dir.display()
        );
        return Ok(());
    }

    if cli.continue_ {
        match load_last_container()? {
            Some(container_name) => {
                resume_container(&container_name).await?;
                return Ok(());
            }
            None => {
                anyhow::bail!("No previous container found. Run without --continue to create a new container.");
            }
        }
    }

    let container_name = generate_container_name(&current_dir);

    println!("Starting Claude Code Sandbox container: {container_name}");

    create_container(&container_name, &current_dir).await?;
    save_last_container(&container_name)?;

    println!("Container {container_name} started successfully!");
    println!("To attach to the container, run: docker exec -it {container_name} /bin/bash");

    Ok(())
}
