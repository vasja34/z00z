use criterion::{black_box, criterion_group, criterion_main, Criterion};

use z00z_wallets::domains::hashing::verify_index_mac;

fn bench_verify_index_mac(c: &mut Criterion) {
    let a = [0u8; 32];
    let mut b = [0u8; 32];
    b[0] = 1;

    c.bench_function("verify_index_mac/equal", |bencher| {
        bencher.iter(|| black_box(verify_index_mac(black_box(&a), black_box(&a))))
    });

    c.bench_function("verify_index_mac/different", |bencher| {
        bencher.iter(|| black_box(verify_index_mac(black_box(&a), black_box(&b))))
    });
}

criterion_group!(benches, bench_verify_index_mac);
criterion_main!(benches);
