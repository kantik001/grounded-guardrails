# Benchmarks — grounded-guardrails

## Machine (local)

```text
OS:      Windows 10 (10.0.19045)
rustc:   1.97.1 (stable-x86_64-pc-windows-msvc)
profile: cargo bench (optimized)
date:    2026-07-27
```

## Token ring buffer

| Bench | Target | Measured | Status |
|-------|--------|----------|--------|
| 1M× `push` (capacity 4096) | &lt; 10 ms | **~4.26 ms** | PASS |
| `last_n(128)` on full buffer | &lt; 1 µs | **~541 ns** | PASS |

## PII + numeric

| Bench | Target | Measured | Status |
|-------|--------|----------|--------|
| `contains_pii` on 1KB | &lt; 50 µs | **~98 ns** (early exit on first hit) | PASS |
| `detect` on 1KB (all matches) | — | **~128 µs** | info |
| `extract_numerics` on 1KB | &lt; 100 µs | **~34 µs** | PASS |
| verify 100 numbers vs context | &lt; 10 µs | **~4.2 µs** | PASS |

## Reproduce

```bash
cd rust
cargo bench --bench token_buffer
cargo bench --bench pii_numeric
```

## Criterion raw (`pii_numeric`, sample-size 20)

```text
contains_pii_1kb              [97.857 ns 98.444 ns 99.101 ns]
detect_pii_1kb                [127.62 µs 128.49 µs 129.36 µs]
extract_numerics_1kb          [33.246 µs 33.623 µs 34.094 µs]
verify_100_numbers            [4.1901 µs 4.2405 µs 4.2877 µs]
verify_answer_against_context [1.2171 µs 1.2289 µs 1.2415 µs]
```
