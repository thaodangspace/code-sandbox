package handlers

import (
	"encoding/json"
	"net/http"

	"github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/client"
)

// ContainerHandler handles Docker container operations.
type ContainerHandler struct {
	cli *client.Client
}

// NewContainerHandler creates a ContainerHandler with a Docker client.
func NewContainerHandler() (*ContainerHandler, error) {
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		return nil, err
	}
	return &ContainerHandler{cli: cli}, nil
}

type containerRequest struct {
	Image string   `json:"image"`
	Cmd   []string `json:"cmd"`
}

// CreateContainer creates a new Docker container.
func (h *ContainerHandler) CreateContainer(w http.ResponseWriter, r *http.Request) {
	var req containerRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if req.Image == "" {
		req.Image = "busybox"
	}

	resp, err := h.cli.ContainerCreate(r.Context(), &container.Config{Image: req.Image, Cmd: req.Cmd, Tty: true}, nil, nil, nil, "")
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(map[string]string{"id": resp.ID})
}

// ListContainers returns the list of Docker containers.
func (h *ContainerHandler) ListContainers(w http.ResponseWriter, r *http.Request) {
	containers, err := h.cli.ContainerList(r.Context(), types.ContainerListOptions{})
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	json.NewEncoder(w).Encode(containers)
}
