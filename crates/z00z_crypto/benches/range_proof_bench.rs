use criterion::{black_box, criterion_group, criterion_main, Criterion};
use z00z_crypto::{
    verify_asset_output_proofs_batch, AssetOutputProof, AssetRangeProof, Commitment, Z00ZScalar,
};

fn bench_single_proof(c: &mut Criterion) {
    let value = 1000u64;
    let blind = Z00ZScalar::one();

    c.bench_function("range_proof_single_gen", |b| {
        b.iter(|| {
            let _ = AssetRangeProof::new(black_box(value), black_box(&blind));
        })
    });

    let proof = AssetRangeProof::new(value, &blind).expect("proof");
    let commitment = Commitment::new_with_blinding(value, &blind).expect("commitment");

    c.bench_function("range_proof_single_verify", |b| {
        b.iter(|| {
            let _ = proof.verify(black_box(&commitment));
        })
    });
}

fn bench_batch_verify(c: &mut Criterion) {
    let mut outputs = Vec::new();
    for value in 1u64..=16u64 {
        outputs.push(AssetOutputProof::new(value).expect("output"));
    }

    c.bench_function("range_proof_batch_verify_16", |b| {
        b.iter(|| {
            let _ = verify_asset_output_proofs_batch(black_box(&outputs));
        })
    });
}

criterion_group!(range_benches, bench_single_proof, bench_batch_verify);
criterion_main!(range_benches);
