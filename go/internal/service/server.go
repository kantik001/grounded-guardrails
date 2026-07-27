package service

import (
	"context"
	"fmt"
	"io"
	"strings"
	"time"

	"github.com/kantik001/grounded-guardrails/go/internal/rules"
	pb "github.com/kantik001/grounded-guardrails/go/gen/guardrails/v1"
)

// Server implements guardrails.v1.GuardrailsService.
type Server struct {
	pb.UnimplementedGuardrailsServiceServer
}

// New returns a Guardrails gRPC service.
func New() *Server {
	return &Server{}
}

// VerifyText checks a complete answer against optional retrieval context.
func (s *Server) VerifyText(ctx context.Context, req *pb.TextRequest) (*pb.TextVerdict, error) {
	start := time.Now()
	if req == nil {
		return nil, fmt.Errorf("nil request")
	}
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	enabled := ruleSet(req.GetRules())
	var violations []string

	if enabled["pii_block"] && rules.ContainsPII(req.GetText()) {
		for _, m := range rules.DetectPII(req.GetText()) {
			violations = append(violations, rules.FormatPIIViolation(m))
		}
	}
	if enabled["numeric_verify"] {
		for _, n := range rules.UnmatchedNumerics(req.GetText(), req.GetContext(), rules.DefaultTolerance) {
			violations = append(violations, fmt.Sprintf("numeric_unmatched:%g", n))
		}
	}

	return &pb.TextVerdict{
		Passed:     len(violations) == 0,
		Violations: violations,
		LatencyMs:  float32(time.Since(start).Seconds() * 1000),
	}, nil
}

// VerifyStream accumulates decoded text deltas and returns a verdict per batch.
func (s *Server) VerifyStream(stream pb.GuardrailsService_VerifyStreamServer) error {
	var acc strings.Builder
	for {
		batch, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}
		start := time.Now()
		if delta := batch.GetTextDelta(); delta != "" {
			acc.WriteString(delta)
		}

		action := pb.Verdict_PASS
		reason := "ok"
		var matched []string

		text := acc.String()
		if text != "" && rules.ContainsPII(text) {
			action = pb.Verdict_BLOCK
			reason = "pii detected in stream"
			for _, m := range rules.DetectPII(text) {
				matched = append(matched, string(m.Type))
			}
		}

		// Token IDs alone cannot be checked without a tokenizer; surface as FLAG if only IDs arrive.
		if text == "" && len(batch.GetTokenIds()) > 0 {
			action = pb.Verdict_FLAG
			reason = "token_ids without text_delta; decode upstream or send text_delta"
			matched = append(matched, "missing_text_delta")
		}

		if err := stream.Send(&pb.Verdict{
			Action:       action,
			Reason:       reason,
			MatchedRules: matched,
			LatencyMs:    float32(time.Since(start).Seconds() * 1000),
		}); err != nil {
			return err
		}
	}
}

func ruleSet(rulesList []string) map[string]bool {
	out := map[string]bool{
		"numeric_verify": true,
		"pii_block":      true,
	}
	if len(rulesList) == 0 {
		return out
	}
	out = map[string]bool{}
	for _, r := range rulesList {
		out[strings.TrimSpace(r)] = true
	}
	return out
}
