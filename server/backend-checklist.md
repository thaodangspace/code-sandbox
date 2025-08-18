# Backend Checklist

Progress on Phase 1 backend tasks.

## Project Setup & Structure
- [x] Initialize project structure (`server` directory)
- [x] Set up Go backend project with chi router and `/health` endpoint
- [x] Organize container handlers into internal package

## Docker Container Management
- [x] Install and configure Docker SDK for Go
- [x] Implement Docker container creation API endpoint (`POST /api/containers`)
- [x] Implement Docker container management APIs (`GET /api/containers` for listing)
- [ ] Add error handling and logging for Docker operations

## WebSocket & Terminal Integration
- [ ] Implement WebSocket server in Go backend
- [ ] Connect terminal to Docker containers

## User Authentication & Session Management
- [ ] Design and implement user data models
- [ ] Implement user registration and login APIs
- [ ] Implement authentication middleware
