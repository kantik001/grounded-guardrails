package main

import (
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/kantik001/grounded-guardrails/go/internal/config"
	"github.com/kantik001/grounded-guardrails/go/internal/service"
	pb "github.com/kantik001/grounded-guardrails/go/gen/guardrails/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/health"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/reflection"
)

func main() {
	cfg := config.Load()
	lis, err := net.Listen("tcp", cfg.GRPCAddr)
	if err != nil {
		log.Fatalf("listen %s: %v", cfg.GRPCAddr, err)
	}

	grpcServer := grpc.NewServer()
	pb.RegisterGuardrailsServiceServer(grpcServer, service.New())

	healthServer := health.NewServer()
	healthpb.RegisterHealthServer(grpcServer, healthServer)
	healthServer.SetServingStatus("guardrails.v1.GuardrailsService", healthpb.HealthCheckResponse_SERVING)
	healthServer.SetServingStatus("", healthpb.HealthCheckResponse_SERVING)

	reflection.Register(grpcServer)

	go func() {
		log.Printf("grounded-guardrails gRPC listening on %s", cfg.GRPCAddr)
		if err := grpcServer.Serve(lis); err != nil {
			log.Fatalf("serve: %v", err)
		}
	}()

	ch := make(chan os.Signal, 1)
	signal.Notify(ch, syscall.SIGINT, syscall.SIGTERM)
	<-ch
	log.Printf("shutting down")
	grpcServer.GracefulStop()
}
