# codesandbox (Code Sandbox)

A Rust CLI tool that creates isolated Ubuntu Docker containers with a development agent pre-installed (Claude by default).
This lets you run AI assistants in a disposable sandbox where their actions are
confined to the container.

## Why sandbox an AI agent?

Running an agent inside an isolated container provides several benefits:

-   Protects your host machine by keeping the agent's file system changes and
    processes separate from your environment
-   Ensures a clean, reproducible workspace with all dependencies installed
    from scratch
-   Makes it easy to experiment with untrusted code or dependencies and then
    discard the container when finished

## Features

-   Creates a new Ubuntu Docker container with a chosen agent (Claude, Gemini, Codex, Qwen) installed
-   Mounts current directory to `/workspace` in the container
-   Automatically copies your `.claude` configuration
-   Starts the selected agent in the container
-   Generates contextual container names to avoid conflicts (`csb-{agent}-{dir}-{branch}-{yymmddhhmm}`)
-   Cleans up all containers for a directory with `codesandbox --cleanup`

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

1. Create a new Ubuntu container with a unique name (e.g., `csb-claude-project-main-2401011230`)
2. Mount the current directory to `/workspace` in the container
3. Copy your `.claude` config from `~/.claude` (if it exists)
4. Install and start the selected agent in the container

To use a different agent, specify the `--agent` flag. For example, to start Qwen:

```
codesandbox --agent qwen
```

To mount an additional directory read-only inside the container, use:

```
codesandbox --add_dir /path/to/other/repo
```

## Connecting to the Container

After the container is created, you can connect to it using:

```bash
docker exec -it <container-name> /bin/bash
```

The container name will be displayed when `codesandbox` runs.

## Listing Existing Containers

List all sandbox containers created from the current directory and optionally attach to one:

```bash
codesandbox ls
```

You will be shown a numbered list of containers. Enter a number to attach or press Enter to cancel.

## Container Contents

-   **Base**: Ubuntu 22.04
-   **Tools**: curl, wget, git, build-essential, python3, nodejs, npm
-   **User**: `ubuntu` with sudo privileges
-   **Agent**: Claude Code pre-installed (other agents can be started if available)
-   **Working Directory**: `/workspace` (your mounted folder)

## Configuration

The tool automatically detects and mounts your Claude configuration from:

-   `~/.claude` (standard location)
-   `$XDG_CONFIG_HOME/claude` (XDG standard)

## Cleanup

To remove all containers created from the current directory:

```bash
codesandbox --cleanup
```

To remove the built image:

```bash
docker rmi codesandbox-image
```

## Troubleshooting

-   **Docker not found**: Ensure Docker is installed and running
-   **Permission denied**: Make sure your user is in the `docker` group
-   **Agent fails to start**: You can manually start it with `docker exec -it <container> <agent>`
