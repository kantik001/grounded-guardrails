package config

import (
	"os"
	"strconv"
)

// Config holds process configuration for the guardrails gRPC server.
type Config struct {
	// GRPCAddr is the listen address (default :50052).
	GRPCAddr string
}

// Load reads configuration from the environment.
func Load() Config {
	addr := os.Getenv("GRPC_ADDR")
	if addr == "" {
		port := os.Getenv("GRPC_PORT")
		if port == "" {
			port = "50052"
		}
		if _, err := strconv.Atoi(port); err == nil {
			addr = ":" + port
		} else {
			addr = port
		}
	}
	return Config{GRPCAddr: addr}
}
