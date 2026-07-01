use criterion::{black_box, criterion_group, criterion_main, Criterion};
use z00z_crypto::hash_policy::{blake2b_hash, poseidon2_hash};

fn bench_blake(c: &mut Criterion) {
    let domain = b"z00z/wallet/bench";
    let part_a = vec![1u8; 64];
    let part_b = vec![2u8; 128];

    c.bench_function("blake2b_hash_2parts", |b| {
        b.iter(|| {
            blake2b_hash(
                black_box(domain),
                &[black_box(part_a.as_slice()), black_box(part_b.as_slice())],
            )
        })
    });
}

fn bench_poseidon(c: &mut Criterion) {
    let domain = b"z00z/consensus/bench";
    let part_a = vec![3u8; 64];
    let part_b = vec![4u8; 128];

    c.bench_function("poseidon2_hash_placeholder_2parts", |b| {
        b.iter(|| {
            poseidon2_hash(
                black_box(domain),
                &[black_box(part_a.as_slice()), black_box(part_b.as_slice())],
            )
        })
    });
}

criterion_group!(hash_benches, bench_blake, bench_poseidon);
criterion_main!(hash_benches);
