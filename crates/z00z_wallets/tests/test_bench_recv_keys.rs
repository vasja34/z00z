#[cfg(feature = "test-params-fast")]
use z00z_wallets::key::benchmark_recv_keys;

#[cfg(feature = "test-params-fast")]
#[test]
fn test_bench_recv_keys() {
    let iters = 2_048usize;
    let elapsed = benchmark_recv_keys(iters).expect("bench");
    let avg_us = (elapsed.as_micros() as f64) / (iters as f64);

    println!("bench_recv_keys total={elapsed:?} avg_us={avg_us:.3}");
    assert!(
        avg_us < 1_000.0,
        "avg derivation time exceeds 1ms: {avg_us:.3}us"
    );
}
