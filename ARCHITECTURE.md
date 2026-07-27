# Architecture — grounded-guardrails

## Why

Post-hoc verify in Grounded LLM catches bad numbers **after** decode. This service exposes the same class of checks over **gRPC** so agents and future vLLM callbacks can fail fast.

## Ports

| Service | Port |
|---------|------|
| grounded-llm Retriever | `:50051` |
| **grounded-guardrails** | **`:50052`** |

## Layout

```text
proto/guardrails.proto     # GuardrailsService contract
rust/                      # Reference hot-path algorithms + benches
go/
  cmd/server/              # gRPC process
  internal/rules/          # PII + numeric (parity with Rust semantics)
  internal/service/        # VerifyText / VerifyStream
  gen/guardrails/v1/       # generated stubs
```

## RPC

- **`VerifyText`** — unary; default rules `numeric_verify` + `pii_block`
- **`VerifyStream`** — client stream of `TokenBatch`; prefers `text_delta` (decoded chunk). Token IDs alone → `FLAG` (`missing_text_delta`) until a tokenizer is wired

## Algorithm ownership

- **Rust** = reference implementation (benches, future FFI/cdylib)
- **Go** = gRPC host with **semantic parity** tests (same tolerance 0.01, RU `млн`, PII masks)

Do not invent a third numeric algorithm in grounded-llm — call this service (or later link the Rust lib).

## Integration sketch

```text
                    ┌─────────────────────┐
   client / agent ──┤  grounded-llm        │
                    │  Retriever :50051   │
                    │  Go server :8080    │
                    └─────────┬───────────┘
                              │ GUARDRAILS_MODE=remote|hybrid
                              │ VerifyText
                    ┌─────────▼───────────┐
                    │ grounded-guardrails │
                    │ gRPC :50052         │
                    │ Rust ref + Go host  │
                    └─────────────────────┘
```

Wire docs: [grounded-llm docs/en/GUARDRAILS.md](https://github.com/kantik001/grounded-llm/blob/feat/guardrails-remote-verify/docs/en/GUARDRAILS.md).
