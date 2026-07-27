# Changelog

## [0.1.0] - 2026-07-27

### Added

- `TokenRingBuffer` — fixed-capacity streaming token window (zero growth after init)
- `PiiDetector` — email / phone / SSN / credit-card detection + masking (Rust)
- `extract_numerics`, `verify_numeric`, `verify_answer_against_context` — canonical numeric verify (Rust)
- Criterion benches for buffer, PII, and numeric paths
- **Go gRPC** `GuardrailsService` on `:50052` (`VerifyText`, `VerifyStream`)
- Proto [`proto/guardrails.proto`](proto/guardrails.proto) + health + reflection
- Go rules parity package (`internal/rules`) + bufconn tests
- Dockerfile / Compose / Makefile
- GitHub Actions CI (Rust + Go)
