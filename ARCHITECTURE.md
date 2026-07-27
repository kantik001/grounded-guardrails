# Architecture — grounded-guardrails

## Why this exists

[Grounded LLM](https://github.com/kantik001/grounded-llm) verifies numeric claims **after** generation. That catches hallucinations, but you already paid for a full decode.

**grounded-guardrails** moves checks onto the **streaming / text path**: keep a bounded token window, run cheap rules (numeric continuity vs retrieved context, PII patterns), emit PASS / FLAG / BLOCK early.

## Placement in the ecosystem

```text
                    ┌─────────────────────┐
   client / agent ──┤  grounded-agent     │
                    └─────────┬───────────┘
                              │ retrieve
                    ┌─────────▼───────────┐
                    │  grounded-llm        │
                    │  Retriever :50051   │
                    └─────────┬───────────┘
                              │ verify (next integration)
                    ┌─────────▼───────────┐
                    │ grounded-guardrails │
                    │ gRPC :50052 (soon)  │
                    │ Rust hot path       │
                    └─────────────────────┘
```

## Crate layout

```text
rust/src/
  lib.rs
  buffer.rs    # TokenRingBuffer — O(1) push, iterator last_n
  pii.rs       # PiiDetector — LazyLock regex, mask helpers
  numeric.rs   # extract + tolerance verify (canonical algorithm)
```

### Numeric (canonical)

- Extracts grouped integers (`14,000,000`), decimals, percentages, `±` magnitudes
- Optional RU/EN multipliers (`млн` / `million` → ×1e6) so shorthand answers match report figures
- Default tolerance **0.01** (same absolute rule as Grounded LLM Go verify)
- Empty answer numbers → **pass** (parity with grounded-llm)

Go/Python must **call** this logic later — do not maintain a third fork.

### PII

- Types: email, phone (US/RU forms), SSN, 16-digit card
- Explicit non-matches: `version 1.2.3`, `order #123-456`

## Design constraints

1. One algorithm source of truth in Rust
2. Guardrails port **`:50052`** (Retriever owns `:50051`)
3. No CUDA until a serving-path profile demands it
