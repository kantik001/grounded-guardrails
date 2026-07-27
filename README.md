# grounded-guardrails

[![CI](https://github.com/kantik001/grounded-guardrails/actions/workflows/ci.yml/badge.svg)](https://github.com/kantik001/grounded-guardrails/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](rust/Cargo.toml)

**Token-level verification primitives** for grounded LLM inference. Part of the [Grounded](https://github.com/kantik001/grounded-llm) verifiable AI ecosystem.

> Block bad numbers and PII at token *N*, not after the full answer is generated.

| | |
|---|---|
| **Hot path** | Fixed-capacity ring buffer — no allocation after init |
| **Rules** | Numeric verify (Δ ≤ 0.01) + regex PII detect/mask |
| **Port (planned gRPC)** | `:50052` (Retriever stays on `:50051`) |
| **Role in stack** | Rust = source of truth; Go/Python call in |

## Status (v0.1.0)

Shipped:

- `TokenRingBuffer` — streaming token window
- `PiiDetector` — email / phone / SSN / card + masking
- `extract_numerics` / `verify_answer_against_context` — canonical numeric algorithm
- Unit + property tests, Criterion benches, CI

Next:

- Go gRPC `GuardrailsService` on `:50052`
- Wire into [grounded-llm](https://github.com/kantik001/grounded-llm) verify path

## Quick start

```bash
cd rust
cargo test
cargo clippy -- -D warnings
cargo bench --bench token_buffer
cargo bench --bench pii_numeric
```

```rust
use grounded_guardrails::{
    DEFAULT_TOLERANCE, PiiDetector, TokenRingBuffer, verify_answer_against_context,
};

let mut buf = TokenRingBuffer::with_default_capacity();
buf.push(42, 0);

let detector = PiiDetector::new();
assert!(!detector.contains_pii("version 1.2.3"));

assert!(verify_answer_against_context(
    "Revenue 14 млн",
    "Report: 14,000,000",
    DEFAULT_TOLERANCE,
));
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
