use cbor::{decode_cbor, Cbor};
use criterion::{criterion_group, criterion_main, Criterion};
use ezcbor::*;
use rand::rngs::ThreadRng;

fn my_benchmark(c: &mut Criterion) {

    let mut group = c.benchmark_group("All benchmarks");
    
    
    let mut large_vec = Vec::new();
    let rng = rand::thread_rng();
    for i in 0..1_000_000 {
        large_vec.push(i);
    }

    group.bench_function("Seriallize large Vec", |b| b.iter(|| {
        let bytes = large_vec.to_cbor_bytes();
    }));
    let bytes = large_vec.to_cbor_bytes();
    group.bench_function("Deseriallize bytes to large Vec", |b| b.iter(|| {
        let decoded_vec: Vec<i32> = decode_cbor(&bytes).unwrap();
    }));


}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);