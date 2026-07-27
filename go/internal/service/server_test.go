package service

import (
	"context"
	"net"
	"testing"
	"time"

	pb "github.com/kantik001/grounded-guardrails/go/gen/guardrails/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/health"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/test/bufconn"
)

const bufSize = 1024 * 1024

func startTestServer(t *testing.T) (pb.GuardrailsServiceClient, func()) {
	t.Helper()
	lis := bufconn.Listen(bufSize)
	s := grpc.NewServer()
	pb.RegisterGuardrailsServiceServer(s, New())
	hs := health.NewServer()
	healthpb.RegisterHealthServer(s, hs)
	hs.SetServingStatus("guardrails.v1.GuardrailsService", healthpb.HealthCheckResponse_SERVING)

	go func() {
		_ = s.Serve(lis)
	}()

	dialer := func(context.Context, string) (net.Conn, error) {
		return lis.Dial()
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	conn, err := grpc.DialContext(ctx, "bufnet",
		grpc.WithContextDialer(dialer),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		cancel()
		t.Fatalf("dial: %v", err)
	}
	cleanup := func() {
		cancel()
		_ = conn.Close()
		s.Stop()
		_ = lis.Close()
	}
	return pb.NewGuardrailsServiceClient(conn), cleanup
}

func TestVerifyText_PassAndFail(t *testing.T) {
	client, cleanup := startTestServer(t)
	defer cleanup()

	ctx := context.Background()
	ok, err := client.VerifyText(ctx, &pb.TextRequest{
		Text:    "Revenue was 14 млн per report.",
		Context: "Annual report lists 14,000,000.",
	})
	if err != nil {
		t.Fatal(err)
	}
	if !ok.GetPassed() {
		t.Fatalf("expected pass, violations=%v", ok.GetViolations())
	}

	bad, err := client.VerifyText(ctx, &pb.TextRequest{
		Text:    "Contact user@example.com for 15 млн",
		Context: "14,000,000 only",
	})
	if err != nil {
		t.Fatal(err)
	}
	if bad.GetPassed() {
		t.Fatal("expected fail")
	}
	if len(bad.GetViolations()) == 0 {
		t.Fatal("expected violations")
	}
}

func TestHealth(t *testing.T) {
	lis := bufconn.Listen(bufSize)
	s := grpc.NewServer()
	hs := health.NewServer()
	healthpb.RegisterHealthServer(s, hs)
	hs.SetServingStatus("guardrails.v1.GuardrailsService", healthpb.HealthCheckResponse_SERVING)
	go func() { _ = s.Serve(lis) }()
	defer s.Stop()

	ctx := context.Background()
	conn, err := grpc.DialContext(ctx, "bufnet",
		grpc.WithContextDialer(func(context.Context, string) (net.Conn, error) { return lis.Dial() }),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()

	resp, err := healthpb.NewHealthClient(conn).Check(ctx, &healthpb.HealthCheckRequest{
		Service: "guardrails.v1.GuardrailsService",
	})
	if err != nil {
		t.Fatal(err)
	}
	if resp.GetStatus() != healthpb.HealthCheckResponse_SERVING {
		t.Fatalf("status=%v", resp.GetStatus())
	}
}
