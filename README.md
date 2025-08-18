# codesandbox (Code Sandbox)

A Rust CLI tool that creates isolated Ubuntu Docker containers with Claude Code pre-installed for development work.

## Features

-   Creates a new Ubuntu Docker container with Claude Code installed
-   Mounts current directory to `/workspace` in the container
-   Automatically copies your `.claude` configuration
-   Starts Claude Code in the container
-   Generates contextual container names to avoid conflicts (`csb-{dir}-{branch}-{yymmddhhmm}`)

## Prerequisites

-   Docker installed and running
-   Rust toolchain (for building from source)

## Installation

### Build from source

```bash
git clone <this-repo>
cd code-sandbox-script
cargo build --release
sudo cp target/release/codesandbox /usr/local/bin/
```

### Install via Cargo

```bash
cargo install --path .
```

## Usage

Navigate to any folder and run:

```bash
codesandbox
```

This will:

1. Create a new Ubuntu container with a unique name (e.g., `csb-project-main-2401011230`)
2. Mount the current directory to `/workspace` in the container
3. Copy your `.claude` config from `~/.claude` (if it exists)
4. Install and start Claude Code in the container

## Connecting to the Container

After the container is created, you can connect to it using:

```bash
docker exec -it <container-name> /bin/bash
```

The container name will be displayed when `codesandbox` runs.

## Container Contents

-   **Base**: Ubuntu 22.04
-   **Tools**: curl, wget, git, build-essential, python3, nodejs, npm
-   **User**: `ubuntu` with sudo privileges
-   **Claude Code**: Pre-installed and available in PATH
-   **Working Directory**: `/workspace` (your mounted folder)

## Configuration

The tool automatically detects and mounts your Claude configuration from:

-   `~/.claude` (standard location)
-   `$XDG_CONFIG_HOME/claude` (XDG standard)

## Cleanup

To remove containers when done:

```bash
docker rm -f <container-name>
```

To remove the built image:

```bash
docker rmi codesandbox-image
```

## Troubleshooting

-   **Docker not found**: Ensure Docker is installed and running
-   **Permission denied**: Make sure your user is in the `docker` group
-   **Claude Code fails to start**: You can manually start it with `docker exec -it <container> claude`
