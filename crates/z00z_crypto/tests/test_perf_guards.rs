use std::time::Instant;

use z00z_crypto::{verify_asset_output_proofs_batch, AssetOutputProof};

const DEFAULT_MAX_BATCH_VERIFY_MS: u128 = 1_000;
const PERF_THRESHOLD_ENV: &str = "Z00Z_CRYPTO_BATCH_VERIFY_MAX_MS";
const PERF_WARMUP_RUNS: usize = 1;
const PERF_SAMPLE_RUNS: usize = 3;

fn max_batch_verify_ms() -> u128 {
    std::env::var(PERF_THRESHOLD_ENV)
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_BATCH_VERIFY_MS)
}

#[test]
fn test_batch_verification_performance() {
    let mut outputs = Vec::new();
    for _ in 0..10 {
        outputs.push(AssetOutputProof::new(100u64).expect("output"));
    }

    // Keep release-mode performance authority in Criterion benches. This test
    // runs under the default test profile, so it guards against catastrophic
    // regressions while tolerating one-time backend warmup noise.
    for _ in 0..PERF_WARMUP_RUNS {
        let result = verify_asset_output_proofs_batch(&outputs).expect("batch verify warmup");
        assert!(result, "batch verification warmup result must be true");
    }

    let mut samples = Vec::with_capacity(PERF_SAMPLE_RUNS);
    for _ in 0..PERF_SAMPLE_RUNS {
        let start = Instant::now();
        let result = verify_asset_output_proofs_batch(&outputs).expect("batch verify");
        let elapsed = start.elapsed();
        assert!(result, "batch verification result must be true");
        samples.push(elapsed.as_millis());
    }

    samples.sort_unstable();
    let median_ms = samples[samples.len() / 2];
    let max_batch_verify_ms = max_batch_verify_ms();
    assert!(
        median_ms < max_batch_verify_ms,
        "batch verification median too slow: {}ms (limit: {}ms, samples: {:?}, override: {})",
        median_ms,
        max_batch_verify_ms,
        samples,
        PERF_THRESHOLD_ENV
    );
}
