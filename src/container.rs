use anyhow::{Context, Result};
use chrono::{Local, Utc};
use std::env;
use std::path::Path;
use std::process::Command;

use crate::config::{get_claude_config_dir, get_claude_json_paths};

fn sanitize(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

pub fn generate_container_name(current_dir: &Path) -> String {
    let dir_name = current_dir
        .file_name()
        .and_then(|s| s.to_str())
        .map(sanitize)
        .unwrap_or_else(|| "unknown".to_string());

    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(current_dir)
        .output();
    let branch_name = branch_output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| sanitize(String::from_utf8_lossy(&o.stdout).trim()))
        .unwrap_or_else(|| "unknown".to_string());

    let timestamp = Local::now().format("%y%m%d%H%M").to_string();

    format!("csb-{dir_name}-{branch_name}-{timestamp}")
}

pub fn cleanup_containers(current_dir: &Path) -> Result<()> {
    let dir_name = current_dir
        .file_name()
        .and_then(|s| s.to_str())
        .map(sanitize)
        .unwrap_or_else(|| "unknown".to_string());
    let prefix = format!("csb-{dir_name}-");

    let list_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .context("Failed to list Docker containers")?;

    if !list_output.status.success() {
        anyhow::bail!(
            "Failed to list containers: {}",
            String::from_utf8_lossy(&list_output.stderr)
        );
    }

    let names = String::from_utf8_lossy(&list_output.stdout);
    for name in names.lines().filter(|n| n.starts_with(&prefix)) {
        println!("Removing container {name}");
        let rm_output = Command::new("docker")
            .args(["rm", "-f", name])
            .output()
            .context("Failed to remove container")?;

        if !rm_output.status.success() {
            anyhow::bail!(
                "Failed to remove container {}: {}",
                name,
                String::from_utf8_lossy(&rm_output.stderr)
            );
        }
    }

    Ok(())
}

pub fn list_containers(current_dir: &Path) -> Result<Vec<String>> {
    let dir_name = current_dir
        .file_name()
        .and_then(|s| s.to_str())
        .map(sanitize)
        .unwrap_or_else(|| "unknown".to_string());
    let prefix = format!("csb-{dir_name}-");

    let list_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .context("Failed to list Docker containers")?;

    if !list_output.status.success() {
        anyhow::bail!(
            "Failed to list containers: {}",
            String::from_utf8_lossy(&list_output.stderr)
        );
    }

    let names = String::from_utf8_lossy(&list_output.stdout);
    let containers = names
        .lines()
        .filter(|n| n.starts_with(&prefix))
        .map(|s| s.to_string())
        .collect();
    Ok(containers)
}

pub fn auto_remove_old_containers(minutes: u64) -> Result<()> {
    if minutes == 0 {
        return Ok(());
    }

    let cutoff = Utc::now() - chrono::Duration::minutes(minutes as i64);

    let list_output = Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .context("Failed to list Docker containers")?;

    if !list_output.status.success() {
        anyhow::bail!(
            "Failed to list containers: {}",
            String::from_utf8_lossy(&list_output.stderr)
        );
    }

    let names = String::from_utf8_lossy(&list_output.stdout);
    for name in names.lines().filter(|n| n.starts_with("csb-")) {
        let inspect_output = Command::new("docker")
            .args(["inspect", "-f", "{{.Created}}", name])
            .output()
            .context("Failed to inspect container")?;
        if !inspect_output.status.success() {
            continue;
        }
        let created_str = String::from_utf8_lossy(&inspect_output.stdout)
            .trim()
            .to_string();
        let created = match chrono::DateTime::parse_from_rfc3339(&created_str) {
            Ok(c) => c.with_timezone(&Utc),
            Err(_) => continue,
        };
        if created > cutoff {
            continue;
        }

        let logs_output = Command::new("docker")
            .args(["logs", name])
            .output()
            .context("Failed to check container logs")?;
        if !logs_output.status.success() {
            continue;
        }
        if logs_output.stdout.is_empty() && logs_output.stderr.is_empty() {
            println!("Auto removing unused container {name}");
            let rm_output = Command::new("docker")
                .args(["rm", "-f", name])
                .output()
                .context("Failed to remove container")?;
            if !rm_output.status.success() {
                anyhow::bail!(
                    "Failed to remove container {}: {}",
                    name,
                    String::from_utf8_lossy(&rm_output.stderr)
                );
            }
        }
    }
    Ok(())
}

pub fn check_docker_availability() -> Result<()> {
    let output = Command::new("docker").arg("--version").output().context(
        "Failed to check Docker availability. Make sure Docker is installed and running.",
    )?;

    if !output.status.success() {
        anyhow::bail!("Docker is not available or not running");
    }

    Ok(())
}

pub fn is_container_running(container_name: &str) -> Result<bool> {
    let output = Command::new("docker")
        .args(&["inspect", "-f", "{{.State.Running}}", container_name])
        .output()
        .context("Failed to check container status")?;

    if !output.status.success() {
        return Ok(false);
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let status = output_str.trim();
    Ok(status == "true")
}

pub fn container_exists(container_name: &str) -> Result<bool> {
    let output = Command::new("docker")
        .args(&["inspect", container_name])
        .output()
        .context("Failed to check if container exists")?;

    Ok(output.status.success())
}

pub async fn create_container(
    container_name: &str,
    current_dir: &Path,
    additional_dir: Option<&Path>,
) -> Result<()> {
    let current_user = env::var("USER").unwrap_or_else(|_| "ubuntu".to_string());
    let dockerfile_content = create_dockerfile_content(&current_user);

    let temp_dir = std::env::temp_dir();
    let dockerfile_path = temp_dir.join("Dockerfile.codesandbox");
    std::fs::write(&dockerfile_path, dockerfile_content).context("Failed to write Dockerfile")?;

    println!("Building Docker image with Claude Code...");
    let build_output = Command::new("docker")
        .args(&[
            "build",
            "-t",
            "codesandbox-image",
            "-f",
            dockerfile_path.to_str().unwrap(),
            ".",
        ])
        .current_dir(&temp_dir)
        .output()
        .context("Failed to build Docker image")?;

    if !build_output.status.success() {
        anyhow::bail!(
            "Docker build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    let mut docker_run = Command::new("docker");
    docker_run.args(&[
        "run",
        "-d",
        "-it",
        "--name",
        container_name,
        "-v",
        &format!("{}:{}", current_dir.display(), current_dir.display()),
    ]);

    if let Some(dir) = additional_dir {
        docker_run.args(&["-v", &format!("{}:{}:ro", dir.display(), dir.display())]);
        println!("Mounting additional directory read-only: {}", dir.display());
    }

    if let Some(claude_config_dir) = get_claude_config_dir() {
        if claude_config_dir.exists() {
            docker_run.args(&[
                "-v",
                &format!(
                    "{}:/home/{}/.claude",
                    claude_config_dir.display(),
                    current_user
                ),
            ]);
            println!(
                "Mounting Claude config from: {}",
                claude_config_dir.display()
            );
        }
    }

    let claude_json_paths = get_claude_json_paths();
    for (i, config_path) in claude_json_paths.iter().enumerate() {
        if config_path.exists() {
            let container_path = if config_path.file_name().unwrap() == ".claude.json" {
                format!("/home/{}/.claude.json", current_user)
            } else {
                format!("/home/{}/.claude/config_{}.json", current_user, i)
            };
            docker_run.args(&[
                "-v",
                &format!("{}:{}", config_path.display(), container_path),
            ]);
            println!(
                "Mounting Claude config from: {} -> {}",
                config_path.display(),
                container_path
            );
        }
    }

    // Mount .serena directory if it exists in current directory or home directory
    let serena_paths = [
        current_dir.join(".serena"),
        home::home_dir().unwrap_or_default().join(".serena"),
    ];

    for serena_path in serena_paths.iter() {
        if serena_path.exists() {
            let container_serena_path = format!("/home/{}/.serena", current_user);
            docker_run.args(&[
                "-v",
                &format!("{}:{}", serena_path.display(), container_serena_path),
            ]);
            println!(
                "Mounting Serena CMP config from: {} -> {}",
                serena_path.display(),
                container_serena_path
            );
            break; // Only mount the first one found
        }
    }

    docker_run.args(&["codesandbox-image", "/bin/bash"]);

    let run_output = docker_run
        .output()
        .context("Failed to run Docker container")?;

    if !run_output.status.success() {
        anyhow::bail!(
            "Failed to create container: {}",
            String::from_utf8_lossy(&run_output.stderr)
        );
    }

    attach_to_container(container_name, current_dir).await
}

pub async fn resume_container(container_name: &str) -> Result<()> {
    println!("Resuming container: {}", container_name);

    if !container_exists(container_name)? {
        anyhow::bail!("Container '{}' does not exist", container_name);
    }

    if !is_container_running(container_name)? {
        println!("Starting stopped container: {}", container_name);
        let start_output = Command::new("docker")
            .args(&["start", container_name])
            .output()
            .context("Failed to start container")?;

        if !start_output.status.success() {
            anyhow::bail!(
                "Failed to start container: {}",
                String::from_utf8_lossy(&start_output.stderr)
            );
        }
    } else {
        println!("Container is already running");
    }

    let current_dir = env::current_dir().context("Failed to get current directory")?;
    attach_to_container(container_name, &current_dir).await
}

async fn attach_to_container(container_name: &str, current_dir: &Path) -> Result<()> {
    println!("Attaching to container with Claude Code...");

    // Ensure the directory structure exists in the container
    let mkdir_status = Command::new("docker")
        .args(&[
            "exec",
            container_name,
            "mkdir",
            "-p",
            &current_dir.display().to_string(),
        ])
        .status()
        .context("Failed to create directory structure in container")?;

    if !mkdir_status.success() {
        println!("Warning: Failed to create directory structure in container");
    }

    let attach_status = Command::new("docker")
        .args(&[
            "exec",
            "-it",
            container_name,
            "/bin/bash",
            "-c",
            &format!(
                "cd {} && source ~/.bashrc && exec claude --dangerously-skip-permissions",
                current_dir.display()
            ),
        ])
        .status()
        .context("Failed to attach to container")?;

    if !attach_status.success() {
        println!("Failed to start Claude Code automatically.");
        println!(
            "You can manually attach with: docker exec -it {} /bin/bash",
            container_name
        );
    }

    Ok(())
}

fn create_dockerfile_content(user: &str) -> String {
    format!(
        r#"FROM ubuntu:22.04

# Avoid interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Update and install required packages
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    build-essential \
    python3 \
    python3-pip \
    sudo \
    ca-certificates \
    gnupg \
    lsb-release \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js v22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs

# Install Go
RUN wget https://go.dev/dl/go1.24.5.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go1.24.5.linux-amd64.tar.gz && \
    rm go1.24.5.linux-amd64.tar.gz

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    /root/.cargo/bin/rustup component add rustfmt clippy

# Create user with sudo privileges
RUN useradd -m -s /bin/bash {user} && \
    echo "{user} ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers
USER root
# Install Claude Code
RUN npm install -g @anthropic-ai/claude-code
# Switch to user
USER {user}
WORKDIR /home/{user}

# Install Rust for the user and ensure cargo is available
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    ~/.cargo/bin/rustup component add rustfmt clippy

# Add Go, Rust, and Cargo to PATH
RUN echo 'export PATH="/usr/local/go/bin:$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Add Claude Code to PATH for all sessions
RUN echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

# Set working directory to home
WORKDIR /home/{user}

# Keep container running
CMD ["/bin/bash"]
"#,
        user = user
    )
}
