use std::time::Instant;

use z00z_crypto::hash::poseidon2_hash;

#[test]
#[ignore]
fn test_perf_poseidon2() {
    const ITERS: u64 = 10_000;
    let input = [0x42u8; 32];

    let start = Instant::now();
    for _ in 0..ITERS {
        let _ = poseidon2_hash(b"z00z.consensus.receiver_id.v1", &[&input]);
    }

    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as u64 / ITERS;
    println!("Poseidon2 avg: {} us", avg_us);
    assert!(avg_us < 100, "Poseidon2 must be < 100 us, got {}", avg_us);
}
