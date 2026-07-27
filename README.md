# grounded-guardrails

[![CI](https://github.com/kantik001/grounded-guardrails/actions/workflows/ci.yml/badge.svg)](https://github.com/kantik001/grounded-guardrails/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](rust/Cargo.toml)

**Token-level verification primitives** for grounded LLM inference. Part of the [Grounded](https://github.com/kantik001/grounded-llm) verifiable AI ecosystem.

> Block bad numbers and PII at token *N*, not after the full answer is generated.

| | |
|---|---|
| **Hot path** | Fixed-capacity ring buffer — no allocation after init |
| **Port (planned gRPC)** | `:50052` (Retriever stays on `:50051`) |
| **Role in stack** | Rust = source of truth; Go/Python call in (no divergent reimplementations) |

## Status (v0.1.0)

Shipped:

- `TokenRingBuffer` — streaming token window for online checks
- Unit + property tests, Criterion benches
- CI (test, clippy, fmt)

Next:

- PII regex detector + numeric extract/verify
- Go gRPC `GuardrailsService` on `:50052`
- Wire into [grounded-llm](https://github.com/kantik001/grounded-llm) verify path

## Quick start

```bash
cd rust
cargo test
cargo clippy -- -D warnings
cargo bench -p grounded-guardrails --bench token_buffer
```

```rust
use grounded_guardrails::TokenRingBuffer;

let mut buf = TokenRingBuffer::with_default_capacity(); // 4096
buf.push(/* token_id */ 42, /* stream position */ 0);
for (token_id, position) in buf.last_n(128) {
    // run sliding-window rules on recent tokens
    let _ = (token_id, position);
}
```

## Ecosystem

| Repo | Role |
|------|------|
| [grounded-llm](https://github.com/kantik001/grounded-llm) | Cited RAG + Spec v1 + gRPC Retriever `:50051` |
| [mcp-gateway](https://github.com/kantik001/mcp-gateway) | HTTP bridge to MCP tools |
| [grounded-agent](https://github.com/kantik001/grounded-agent) | ReAct orchestrator |
| **grounded-guardrails** | Token-level verify (this repo) |

## Docs

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [BENCHMARKS.md](BENCHMARKS.md)

## License

MIT
