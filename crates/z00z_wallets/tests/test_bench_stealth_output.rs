#[cfg(feature = "test-params-fast")]
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::{benchmark_stealth_output, SenderWallet},
};

#[cfg(feature = "test-params-fast")]
#[test]
fn test_bench_stealth_output() {
    let keys =
        ReceiverKeys::from_receiver_secret(ReceiverSecret::generate().expect("receiver secret"))
            .expect("receiver keys");
    let card = keys.export_receiver_card().expect("receiver card");
    let mut sender = SenderWallet::new([9u8; 32]);

    let iters = 2_048usize;
    let elapsed = benchmark_stealth_output(&card, &mut sender, iters).expect("bench");
    let avg_us = (elapsed.as_micros() as f64) / (iters as f64);

    println!("bench_stealth_output total={elapsed:?} avg_us={avg_us:.3}");
    assert!(
        avg_us < 5_000.0,
        "avg output time exceeds 5ms: {avg_us:.3}us"
    );
}
