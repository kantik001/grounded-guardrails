# Architecture — grounded-guardrails

## Why this exists

[Grounded LLM](https://github.com/kantik001/grounded-llm) already verifies numeric claims **after** generation. That catches hallucinations, but wastefully: you pay for a full decode first.

**grounded-guardrails** moves checks onto the **streaming token path**: keep a bounded window of recent token IDs, run cheap rules (numeric continuity, PII patterns once text is recoverable), and emit PASS / FLAG / BLOCK early.

## Placement in the ecosystem

```text
                    ┌─────────────────────┐
   client / agent ──┤  grounded-agent     │
                    └─────────┬───────────┘
                              │ retrieve
                    ┌─────────▼───────────┐
                    │  grounded-llm        │
                    │  Retriever :50051   │
                    │  (citations, RAG)   │
                    └─────────┬───────────┘
                              │ verify (next)
                    ┌─────────▼───────────┐
                    │ grounded-guardrails │
                    │ gRPC :50052 (soon)  │
                    │ Rust hot path       │
                    └─────────────────────┘
```

## Crate layout (current)

```text
rust/
  src/
    lib.rs        # crate root
    buffer.rs     # TokenRingBuffer — fixed capacity, O(1) push, iterator last_n
  benches/
    token_buffer.rs
```

### TokenRingBuffer

- Backing storage: `Box<[u32]>` + `Box<[usize]>` sized once in `new`
- `push`: write at `head`, advance modulo capacity; overwrite when full
- `last_n`: index arithmetic only — **no `Vec` allocation**
- `Send + Sync`: safe to own per session behind a lock or shard map

## Design constraints

1. **One algorithm source of truth** — numeric/PII logic lives in Rust; Go/Python are clients.
2. **Do not steal Retriever’s port** — guardrails listen on **`:50052`**.
3. **No CUDA until profiled** — CPU μs-scale rules first; GPU only if a serving-path bench demands it.

## Roadmap modules

| Module | Purpose |
|--------|---------|
| `pii` | Compiled regex PII detect + mask |
| `numeric` | Extract + tolerance verify vs retrieved context |
| `go/` + `proto/` | `GuardrailsService` (`VerifyStream`, `VerifyText`) |
