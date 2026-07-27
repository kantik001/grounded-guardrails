# Changelog

## [0.1.0] - 2026-07-27

### Added

- `TokenRingBuffer` — fixed-capacity streaming token window (zero growth after init)
- `PiiDetector` — email / phone / SSN / credit-card detection + masking
- `extract_numerics`, `verify_numeric`, `verify_answer_against_context` — canonical numeric verify (tolerance 0.01, RU/EN million shorthand)
- Criterion benches for buffer, PII, and numeric paths
- GitHub Actions CI (fmt, clippy, test)
