use criterion::{Criterion, black_box, criterion_group, criterion_main};
use grounded_guardrails::{
    DEFAULT_TOLERANCE, PiiDetector, extract_numerics, verify_answer_against_context,
};

fn sample_1kb() -> String {
    let unit = "Contact ops@example.com or +1 (555) 123-4567. Revenue was 14,000,000 (tolerance ±0.01). Version 1.2.3 and order #123-456 are not PII. ";
    let mut s = String::new();
    while s.len() < 1024 {
        s.push_str(unit);
    }
    s.truncate(1024);
    s
}

fn bench_pii(c: &mut Criterion) {
    let text = sample_1kb();
    let detector = PiiDetector::new();
    // Warm regex LazyLock.
    let _ = detector.contains_pii(&text);

    c.bench_function("contains_pii_1kb", |b| {
        b.iter(|| detector.contains_pii(black_box(&text)));
    });

    c.bench_function("detect_pii_1kb", |b| {
        b.iter(|| detector.detect(black_box(&text)));
    });
}

fn bench_numeric(c: &mut Criterion) {
    let text = sample_1kb();
    let _ = extract_numerics(&text);

    c.bench_function("extract_numerics_1kb", |b| {
        b.iter(|| extract_numerics(black_box(&text)));
    });

    let context: Vec<_> = (0..100).map(|i| format!("{i}.0")).collect();
    let context_joined = context.join(" ");
    let ctx_vals = extract_numerics(&context_joined);

    c.bench_function("verify_100_numbers", |b| {
        b.iter(|| {
            let mut ok = true;
            for i in 0..100 {
                ok &= grounded_guardrails::verify_numeric(
                    black_box(i as f64),
                    black_box(&ctx_vals),
                    DEFAULT_TOLERANCE,
                );
            }
            black_box(ok)
        });
    });

    c.bench_function("verify_answer_against_context", |b| {
        b.iter(|| {
            verify_answer_against_context(
                black_box("Revenue 14,000,000"),
                black_box("Report shows 14,000,000"),
                DEFAULT_TOLERANCE,
            )
        });
    });
}

criterion_group!(benches, bench_pii, bench_numeric);
criterion_main!(benches);
