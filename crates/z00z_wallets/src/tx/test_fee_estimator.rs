use super::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use z00z_core::{
    assets::{AssetClass, AssetWire},
    genesis::asset_std::asset_from_dev_class,
};
use z00z_utils::codec::{json, Codec, JsonCodec};
use z00z_utils::time::MockTimeProvider;

#[derive(Debug, Clone)]
struct TestRateSrc {
    state: Arc<Mutex<Vec<FeeEstimatorResult<u64>>>>,
}

impl TestRateSrc {
    fn with_seq(seq: Vec<FeeEstimatorResult<u64>>) -> Self {
        Self {
            state: Arc::new(Mutex::new(seq)),
        }
    }
}

impl FeeRateSource for TestRateSrc {
    fn get_fee_per_weight(&self) -> FeeEstimatorResult<u64> {
        let mut lock = self.state.lock().expect("lock");
        if lock.is_empty() {
            return Err(FeeEstimatorError::Network("empty source".to_string()));
        }
        lock.remove(0)
    }
}

#[test]
fn test_new_creates_estimator() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 100, 1);
    assert_eq!(estimator.min_fee, 100);
    assert_eq!(estimator.fee_per_weight, 1);
}

#[test]
fn test_estimate_size_estimate() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 100, 1);
    let result = estimator.estimate_by_size(500);
    assert!(result.is_ok());
    let estimate = result.unwrap();
    assert_eq!(estimate.medium, 500);
    assert_eq!(estimate.low, 400);
    assert_eq!(estimate.high, 750);
}

#[test]
fn test_estimate_uses_weight() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 1, 1);
    let tx = json!({
        "tx": {
            "inputs": [{"asset_id": "a"}],
            "outputs": [{"leaf": "b"}],
            "kernels": [{}]
        }
    });
    let bytes = JsonCodec.serialize(&tx).expect("serialize");
    let estimate = estimator.estimate(&bytes).expect("estimate");
    assert_eq!(estimate.medium, 64 + 96 + 900);
}

#[test]
fn test_estimate_respects_minimum_fee() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 100, 1);
    let result = estimator.estimate_by_size(50);
    assert!(result.is_ok());
    let estimate = result.unwrap();
    assert!(estimate.medium >= 100);
    assert!(estimate.low >= 100);
}

#[test]
fn test_get_fee_per_byte() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 100, 2);
    assert_eq!(estimator.get_fee_per_byte().unwrap(), 2);
}

#[test]
fn test_get_minimum_fee() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 150, 1);
    assert_eq!(estimator.get_minimum_fee().unwrap(), 150);
}

#[test]
fn test_update_rates_succeeds() {
    let mut estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(1), 100, 1);
    assert!(estimator.update_rates().is_ok());
}

#[test]
fn test_model_constants_frozen() {
    assert_eq!(FEE_WEIGHT_TAG, "fee-weight-v1");
    assert_eq!(BASE_TX_COST, 64);
    assert_eq!(PER_INPUT_COST, 96);
    assert_eq!(PER_OUTPUT_COST, 900);
    assert_eq!(PER_RANGE_BIT_COST, 1);
    assert_eq!(RANGE_BITS_PER_KERNEL, 120);
}

#[test]
fn test_weight_fixture() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(7), 10, 2);
    let estimate = estimator
        .estimate_by_weight(TxWeight {
            inputs: 2,
            outputs: 1,
            kernels: 1,
        })
        .expect("estimate");
    assert_eq!(estimate.medium, 64 + 2 * 96 + 900 + 120);
}

#[test]
fn test_low_load_vector() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(8), 1, 1);
    let estimate = estimator
        .estimate_by_weight(TxWeight {
            inputs: 0,
            outputs: 0,
            kernels: 0,
        })
        .expect("estimate");
    assert_eq!(estimate.medium, BASE_TX_COST);
}

#[test]
fn test_high_load_vector() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(9), 1, 1);
    let estimate = estimator
        .estimate_by_weight(TxWeight {
            inputs: 500,
            outputs: 400,
            kernels: 300,
        })
        .expect("estimate");
    assert!(estimate.medium > 0);
    assert!(estimate.high >= estimate.medium);
}

#[test]
fn test_mixed_load_vector() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(10), 50, 1);
    let estimate = estimator
        .estimate_by_weight(TxWeight {
            inputs: 2,
            outputs: 3,
            kernels: 1,
        })
        .expect("estimate");
    assert_eq!(estimate.medium, 64 + 2 * 96 + 3 * 900 + 120);
}

#[test]
fn test_size_overflow_fail() {
    let estimator = FeeEstimatorImpl::new(MockTimeProvider::from_unix_secs(10), 1, u64::MAX);
    let result = estimator.estimate_by_size(usize::MAX);
    assert!(result.is_err());
}

#[test]
fn test_calc_fee_overflow() {
    let result = calc_fee_units(GasCount {
        inputs: usize::MAX,
        outputs: usize::MAX,
        range_bits: usize::MAX,
    });
    assert!(result.is_err());
}

#[test]
fn test_fee_wires_matches_units() {
    let asset = asset_from_dev_class(AssetClass::Coin, 1, 100).expect("asset");
    let wire = AssetWire::from_asset(&asset);
    let fee = calculate_fee_for_wires(1, &[wire.clone()]).expect("fee");

    let range_bits = wire
        .range_proof
        .as_ref()
        .map(|proof| proof.len().saturating_mul(8))
        .unwrap_or(0);
    let expected = calc_fee_units(GasCount {
        inputs: 1,
        outputs: 1,
        range_bits,
    })
    .expect("expected");

    assert_eq!(fee, expected);
}

#[test]
fn test_rates_ttl_cache() {
    let time = MockTimeProvider::from_unix_secs(10);
    let src = TestRateSrc::with_seq(vec![Ok(9), Ok(11)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 3, src, 60);

    estimator.update_rates().expect("first");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);

    estimator.update_rates().expect("cached");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);
}

#[test]
fn test_rates_fallback_base() {
    let time = MockTimeProvider::from_unix_secs(20);
    let src = TestRateSrc::with_seq(vec![Err(FeeEstimatorError::Network("timeout".to_string()))]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 7, src, 0);

    estimator.update_rates().expect("fallback");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 7);
}

#[test]
fn test_rates_fallback_cache() {
    let time = MockTimeProvider::from_unix_secs(30);
    let src = TestRateSrc::with_seq(vec![
        Ok(13),
        Err(FeeEstimatorError::Network("source down".to_string())),
    ]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 5, src, 0);

    estimator.update_rates().expect("first");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 13);

    estimator.update_rates().expect("fallback cached");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 13);
}

#[test]
fn test_rates_ttl_refresh() {
    let time = MockTimeProvider::from_unix_secs(40);
    let src = TestRateSrc::with_seq(vec![Ok(9), Ok(11)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 3, src, 60);

    estimator.update_rates().expect("first");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);

    time.advance_by(Duration::from_secs(59));
    estimator.update_rates().expect("still cached");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 9);

    time.advance_by(Duration::from_secs(2));
    estimator.update_rates().expect("refreshed");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 11);
}

#[test]
fn test_rates_zero_ttl_requery() {
    let time = MockTimeProvider::from_unix_secs(50);
    let src = TestRateSrc::with_seq(vec![Ok(5), Ok(7)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 3, src, 0);

    estimator.update_rates().expect("first");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 5);

    estimator.update_rates().expect("second");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 7);
}

#[test]
fn test_rates_refresh_updates_estimate() {
    let time = MockTimeProvider::from_unix_secs(60);
    let src = TestRateSrc::with_seq(vec![Ok(2), Ok(4)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 1, src, 30);

    estimator.update_rates().expect("first");
    let initial = estimator.estimate_by_size(100).expect("initial estimate");
    assert_eq!(initial.medium, 200);

    time.advance_by(Duration::from_secs(31));
    estimator.update_rates().expect("refresh");
    let refreshed = estimator.estimate_by_size(100).expect("refreshed estimate");
    assert_eq!(refreshed.medium, 400);
    assert!(refreshed.medium > initial.medium);
}

#[test]
fn test_rates_stale_keep_cache() {
    let time = MockTimeProvider::from_unix_secs(70);
    let src = TestRateSrc::with_seq(vec![
        Ok(13),
        Err(FeeEstimatorError::Network("refresh timeout".to_string())),
    ]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time.clone(), 1, 5, src, 10);

    estimator.update_rates().expect("prime cache");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 13);

    time.advance_by(Duration::from_secs(11));
    estimator.update_rates().expect("stale fallback");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 13);

    let fee = estimator.estimate_by_size(10).expect("fee");
    assert_eq!(fee.medium, 130);
}

#[test]
fn test_rates_zero_use_cache() {
    let time = MockTimeProvider::from_unix_secs(80);
    let src = TestRateSrc::with_seq(vec![Ok(17), Ok(0)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 4, src, 0);

    estimator.update_rates().expect("prime cache");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 17);

    estimator.update_rates().expect("zero fallback");
    assert_eq!(estimator.get_fee_per_byte().expect("rate"), 17);
}

#[test]
fn test_rates_zero_no_fallback() {
    let time = MockTimeProvider::from_unix_secs(90);
    let src = TestRateSrc::with_seq(vec![Ok(0)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 0, src, 0);

    let err = estimator
        .update_rates()
        .expect_err("zero rate without safe fallback must fail closed");
    assert!(matches!(err, FeeEstimatorError::Config(_)));
}

#[test]
fn test_spike_rate_fails_safe() {
    let time = MockTimeProvider::from_unix_secs(100);
    let src = TestRateSrc::with_seq(vec![Ok(u64::MAX)]);
    let mut estimator = FeeEstimatorImpl::with_network_rate(time, 1, 1, src, 0);

    estimator.update_rates().expect("spike refresh");
    let err = estimator
        .estimate_by_size(2)
        .expect_err("spike overflow must fail closed");
    assert!(matches!(err, FeeEstimatorError::EstimationFailed(_)));
}
