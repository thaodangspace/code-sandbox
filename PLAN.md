# Project Plan: Code Sandbox Application

This document outlines the project plan for developing a web-based code sandbox application, incorporating Go for the backend and React for the frontend, based on the conducted research and technical analysis.

## Project Scope

The primary goal of the `code-sandbox` application is to provide a web-accessible platform where users can:

-   Start and manage isolated Docker containers.
-   Access a web-based terminal (Xterm.js) to interact with the container.
-   Utilize pre-installed AI coding agent CLIs (Claude Code, Cursor CLI, Gemini CLI) within the container.
-   Manage multiple development sessions using Git worktrees.
-   Bypass permissions within the container for AI agents while maintaining host security.

### Application Flow

The typical user interaction flow within the application will be as follows:

1.  **User Selects Repository**: The user will initiate a new session by selecting a Git repository (e.g., by providing a URL or choosing from a list of previously configured repositories).
2.  **User Selects AI Agent**: The user will then choose which AI coding agent they want to use for the session (e.g., Claude Code, Gemini CLI, Cursor CLI). This selection will also determine the initial command to be executed after the container is created.
3.  **User Selects Branch**: The user will specify the Git branch they wish to work on. This branch will be cloned into the container after its creation.
4.  **Container Creation**: The backend will create a new Linux Docker container, configured with the necessary tools and security measures (gVisor, seccomp, cgroups).
5.  **Code Mapping and Git Worktree**: The selected Git repository and branch will be cloned into the container. A new Git worktree will be created within the container for the user's session, providing an isolated working environment.
6.  **Agent Command Execution**: The backend will execute the initial command associated with the selected AI agent within the container.
7.  **Terminal Display**: A web-based terminal (Xterm.js) will be displayed to the user, showing the output of the agent's command and allowing the user to interact directly with the containerized environment.

## Key Modules and Development Breakdown

The project can be broken down into the following key modules:

1.  **Frontend Web Application (React)**:

    -   **User Interface**: Develop a responsive and intuitive UI, cloning the design principles of ChatGPT Codex (focusing on mobile-first UI), for managing containers, sessions, and files.
    -   **Web Terminal Integration**: Integrate Xterm.js (using a React wrapper like `react-xtermjs` or `xterm-for-react`) for interactive terminal access to containers.
    -   **Code Change Visualization**: Implement a UI for visualizing code changes (e.g., diff view) within the container.
    -   **Session Management UI**: Provide controls for creating, switching, and deleting Git worktree sessions.
    -   **Authentication/Authorization UI**: User login and session management.
    -   ** Tech Stack**
        -   React
        -   Tailwind CSS
        -   Shadcn UI
        -   Xterm.js
        -   React Router
        -   React Query
        -   Jotai

2.  **Backend API Server (Go)**:

    -   **User Management**: Handle user registration, login, and session management.
    -   **Docker Orchestration**: Implement APIs to create, start, stop, and remove Docker containers for user sessions. This will involve using Go's official Docker SDK to interact with the Docker daemon.
    -   **WebSocket Server**: Set up a WebSocket server (using Go's `net/websocket` package or a third-party library) to relay terminal input/output between the frontend Xterm.js and the Docker containers (using `docker exec` or `docker attach`).
    -   **AI Agent CLI Wrapper**: Develop Go wrappers to execute Claude Code, Cursor CLI, and Gemini CLI commands within the containers and capture their output. This will involve executing shell commands from Go and parsing their output.
    -   **Git Worktree Management**: Implement APIs to manage Git worktrees (add, list, remove) by executing `git worktree` commands from Go.
    -   **Security Enforcement**: Integrate gVisor runtime, apply custom seccomp profiles, and enforce cgroup resource limits for containers.
    -   ** Tech Stack**
        -   Go
        -   Go-chi
        -   Docker SDK
        -   WebSocket
        -   Git
        -   gVisor

3.  **Container Images**:
    -   Create optimized Docker images for the sandbox environment, pre-installing necessary tools (Python, Node.js, Git, AI agent CLIs, Jupyter).
    -   Configure gVisor as the default runtime for these images.

## High-Level Timeline (Estimated)

This is a high-level estimate and may vary based on team size and unforeseen challenges.

-   **Phase 1: Core Infrastructure (2-4 weeks)**

    -   Set up basic Docker container creation and management from the Go backend.
    -   Implement basic Xterm.js and WebSocket integration for a functional web terminal.
    -   Develop user authentication and session management.

-   **Phase 2: AI Agent Integration & File Management (3-5 weeks)**

    -   Integrate Claude Code, Cursor CLI, and Gemini CLI execution and output capture via Go wrappers.
    -   Implement file browsing and basic editing within the web UI.
    -   Refine container images with all necessary tools and configurations.

-   **Phase 3: Git Worktree & Advanced Features (3-5 weeks)**

    -   Implement Git worktree management APIs and integrate with the frontend UI.
    -   Enhance security measures (custom seccomp profiles, more granular cgroup limits).
    -   Implement persistent storage solutions for user data and Git repositories.

-   **Phase 4: Testing, Optimization & Deployment (2-3 weeks)**
    -   Comprehensive testing (unit, integration, security, performance).
    -   Performance optimization of backend and frontend.
    -   Deployment setup and documentation.

## Future Enhancements

-   **Collaborative Features**: Real-time collaboration on code and terminal sessions.
-   **Pre-built Templates**: Provide templates for different project types and programming languages.
-   **Plugin System**: Allow users to install custom tools or extensions.
-   **Advanced IDE Features**: Debugging tools, syntax highlighting for more languages, code completion.
-   **Monitoring and Logging**: Comprehensive monitoring of container resources and agent activities.

This roadmap provides a structured approach to developing the `code-sandbox` application, ensuring all key requirements are addressed systematically.
