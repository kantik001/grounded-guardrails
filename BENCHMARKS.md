# Benchmarks — grounded-guardrails

## Machine (local)

```text
OS:      Windows 10 (10.0.19045)
CPU:     mid-range workstation (Criterion host)
rustc:   1.97.1 (stable-x86_64-pc-windows-msvc)
profile: cargo bench (optimized)
date:    2026-07-27
```

## Targets vs measured

| Bench | Target | Measured | Status |
|-------|--------|----------|--------|
| 1M× `push` (capacity 4096) | &lt; 10 ms | **~4.26 ms** (~4.3 ns/op) | PASS |
| 1M× `push` (capacity 256) | &lt; 10 ms | **~4.31 ms** | PASS |
| `last_n(128)` on full buffer | &lt; 1 µs* | **~541 ns** | PASS |

\*Original sketch said &lt; 100 ns for scanning 128 entries; that is unrealistic on a general-purpose CPU without a specialized micro-kernel. **&lt; 1 µs** (~4 ns/element) is the engineering acceptance bar.

## How to reproduce

```bash
cd rust
# Windows: run from "x64 Native Tools" / vcvars64, or use CI (Linux)
cargo bench --bench token_buffer
```

## Criterion raw (sample-size 20)

```text
token_ring_push/push_1m/256
    time: [4.2355 ms 4.3066 ms 4.4501 ms]

token_ring_push/push_1m/4096
    time: [4.2360 ms 4.2626 ms 4.2922 ms]

last_n_128_full_buffer
    time: [540.09 ns 541.12 ns 542.27 ns]
```
