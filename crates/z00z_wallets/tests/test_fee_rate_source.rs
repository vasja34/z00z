use std::sync::{Arc, Mutex};
use std::time::Duration;

use z00z_utils::time::MockTimeProvider;
use z00z_wallets::tx::fee_estimator::FeeRateSource;
use z00z_wallets::tx::{FeeEstimator, FeeEstimatorError, FeeEstimatorImpl, FeeEstimatorResult};

#[derive(Debug, Clone)]
struct SeqRateSource {
    state: Arc<Mutex<Vec<FeeEstimatorResult<u64>>>>,
}

impl SeqRateSource {
    fn with_seq(seq: Vec<FeeEstimatorResult<u64>>) -> Self {
        Self {
            state: Arc::new(Mutex::new(seq)),
        }
    }
}

impl FeeRateSource for SeqRateSource {
    fn get_fee_per_weight(&self) -> FeeEstimatorResult<u64> {
        let mut lock = self.state.lock().expect("lock");
        if lock.is_empty() {
            return Err(FeeEstimatorError::Network("empty fee source".to_string()));
        }
        lock.remove(0)
    }
}

#[test]
fn test_public_fee_ttl_refresh() {
    let time = MockTimeProvider::from_unix_secs(100);
    let src = SeqRateSource::with_seq(vec![Ok(4), Ok(7)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 2, src, 30);

    estimator.update_rates().expect("first refresh");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 4);

    time.advance_by(Duration::from_secs(31));
    estimator.update_rates().expect("ttl refresh");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 7);

    let fee = estimator.estimate_by_size(50).expect("fee");
    assert_eq!(fee.medium, 350);
}

#[test]
fn test_public_fee_stale_fallback() {
    let time = MockTimeProvider::from_unix_secs(200);
    let src = SeqRateSource::with_seq(vec![
        Ok(11),
        Err(FeeEstimatorError::Network("refresh timeout".to_string())),
    ]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 3, src, 10);

    estimator.update_rates().expect("prime cache");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 11);

    time.advance_by(Duration::from_secs(11));
    estimator.update_rates().expect("stale fallback");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 11);
}

#[test]
fn test_public_fee_zero_fallback() {
    let time = MockTimeProvider::from_unix_secs(300);
    let src = SeqRateSource::with_seq(vec![Ok(9), Ok(0)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 2, src, 0);

    estimator.update_rates().expect("prime cache");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);

    estimator.update_rates().expect("zero fallback");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);
}

#[test]
fn test_public_fee_spike_fails() {
    let time = MockTimeProvider::from_unix_secs(400);
    let src = SeqRateSource::with_seq(vec![Ok(u64::MAX)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 1, src, 0);

    estimator.update_rates().expect("accept spike rate");
    let err = estimator
        .estimate_by_size(2)
        .expect_err("overflowing spike must fail closed");
    assert!(matches!(err, FeeEstimatorError::EstimationFailed(_)));
}
