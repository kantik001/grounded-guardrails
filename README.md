# grounded-guardrails

[![CI](https://github.com/kantik001/grounded-guardrails/actions/workflows/ci.yml/badge.svg)](https://github.com/kantik001/grounded-guardrails/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](rust/Cargo.toml)
[![Go](https://img.shields.io/badge/Go-1.25-00ADD8?logo=go)](go/go.mod)

**Token-level verification** for grounded LLM inference. Part of the [Grounded](https://github.com/kantik001/grounded-llm) ecosystem.

> Block bad numbers and PII early — not after a full hallucinated answer.

| | |
|---|---|
| **Rust core** | Ring buffer + reference PII/numeric algorithms + benches |
| **Go gRPC** | `GuardrailsService` on **`:50052`** (Retriever stays `:50051`) |
| **Rules** | `numeric_verify` (Δ ≤ 0.01), `pii_block` |

## Quick start — gRPC

```bash
cd go
go test ./...
go run ./cmd/server
# listens on :50052
```

```bash
# requires grpcurl
grpcurl -plaintext localhost:50052 list
grpcurl -plaintext localhost:50052 grpc.health.v1.Health/Check

grpcurl -plaintext -d '{
  "text": "Revenue was 14 млн.",
  "context": "Report: 14,000,000"
}' localhost:50052 guardrails.v1.GuardrailsService/VerifyText
```

Docker:

```bash
docker compose up --build -d
```

## Quick start — Rust

```bash
cd rust
cargo test
cargo bench --bench pii_numeric
```

## Status

Shipped:

- Rust: `TokenRingBuffer`, `PiiDetector`, canonical numeric extract/verify
- Go: gRPC `VerifyText` + `VerifyStream`, health, reflection
- Proto: [`proto/guardrails.proto`](proto/guardrails.proto)
- CI for Rust + Go

Next:

- Wire into [grounded-llm](https://github.com/kantik001/grounded-llm) verify path — **done** (`GUARDRAILS_MODE=remote`)
- Serving-path adapter — **done:** [grounded-vllm](https://github.com/kantik001/grounded-vllm)
- Optional Rust cdylib FFI so Go host calls Rust without a Go port
- Upstream: land [vLLM docs PR #50051](https://github.com/vllm-project/vllm/pull/50051)

## Ecosystem

| Repo | Role |
|------|------|
| [grounded-llm](https://github.com/kantik001/grounded-llm) | Cited RAG + Retriever `:50051` |
| [grounded-bench](https://github.com/kantik001/grounded-bench) | Offline NVR / CP / HR / RR benchmark |
| [grounded-vllm](https://github.com/kantik001/grounded-vllm) | vLLM serving-path verify proxy |
| [mcp-gateway](https://github.com/kantik001/mcp-gateway) | MCP HTTP bridge |
| [grounded-agent](https://github.com/kantik001/grounded-agent) | ReAct orchestrator |
| **grounded-guardrails** | Verify service `:50052` |

## Docs

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [BENCHMARKS.md](BENCHMARKS.md)
- [CHANGELOG.md](CHANGELOG.md)

## License

MIT
