package main

import (
	"log"
	"net/http"

	"github.com/go-chi/chi/v5"

	"github.com/example/code-sandbox/internal/handlers"
)

func main() {
	h, err := handlers.NewContainerHandler()
	if err != nil {
		log.Fatalf("failed to init container handler: %v", err)
	}

	r := chi.NewRouter()
	r.Get("/health", func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte("ok"))
	})
	r.Route("/api", func(r chi.Router) {
		r.Get("/containers", h.ListContainers)
		r.Post("/containers", h.CreateContainer)
	})

	http.ListenAndServe(":8080", r)
}
