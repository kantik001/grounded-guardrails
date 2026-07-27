use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use grounded_guardrails::TokenRingBuffer;

fn bench_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_ring_push");
    for capacity in [256usize, 4096] {
        group.bench_with_input(
            BenchmarkId::new("push_1m", capacity),
            &capacity,
            |b, &capacity| {
                b.iter(|| {
                    let mut buf = TokenRingBuffer::new(capacity);
                    for i in 0..1_000_000u32 {
                        buf.push(black_box(i), black_box(i as usize));
                    }
                    black_box(buf.len())
                });
            },
        );
    }
    group.finish();
}

fn bench_last_n(c: &mut Criterion) {
    let mut buf = TokenRingBuffer::new(4096);
    for i in 0..4096u32 {
        buf.push(i, i as usize);
    }

    c.bench_function("last_n_128_full_buffer", |b| {
        b.iter(|| {
            let mut sum = 0u64;
            for (tok, pos) in buf.last_n(black_box(128)) {
                sum = sum.wrapping_add(u64::from(tok)).wrapping_add(pos as u64);
            }
            black_box(sum)
        });
    });
}

criterion_group!(benches, bench_push, bench_last_n);
criterion_main!(benches);
