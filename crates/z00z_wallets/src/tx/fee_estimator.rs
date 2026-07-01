//! Fee estimation.

use thiserror::Error;
use z00z_core::assets::AssetWire;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::time::TimeProvider;

/// Fee estimation errors.
#[derive(Debug, Error)]
pub enum FeeEstimatorError {
    /// Estimation failed.
    #[error("estimation failed: {0}")]
    EstimationFailed(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Transaction is invalid for estimation.
    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
}

/// Fee estimator result type.
pub type FeeEstimatorResult<T> = std::result::Result<T, FeeEstimatorError>;

/// Transaction weight snapshot for fee estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxWeight {
    /// Number of input entries.
    pub inputs: usize,
    /// Number of output entries.
    pub outputs: usize,
    /// Number of kernels/signatures.
    pub kernels: usize,
}

/// Fee estimation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeEstimate {
    /// Low priority fee.
    pub low: u64,
    /// Medium priority fee.
    pub medium: u64,
    /// High priority fee.
    pub high: u64,
}

/// Fee estimator trait.
///
/// Uses opaque `tx_bytes` for now, until canonical `z00z_core::tx` types are stabilized.
///
/// Current Phase 062 contract:
/// - network-rate refresh runs through the `FeeRateSource` seam
/// - callers may use static rates or local deterministic simulated-live sources
/// - stale, zero, and overflow cases must fail closed
pub trait FeeEstimator {
    /// Estimate fee for a serialized transaction.
    fn estimate(&self, tx_bytes: &[u8]) -> FeeEstimatorResult<FeeEstimate>;

    /// Estimate fee by size (bytes).
    fn estimate_by_size(&self, size_bytes: usize) -> FeeEstimatorResult<FeeEstimate>;

    /// Get recommended fee per byte.
    fn get_fee_per_byte(&self) -> FeeEstimatorResult<u64>;

    /// Get minimum fee.
    fn get_minimum_fee(&self) -> FeeEstimatorResult<u64>;

    /// Update network fee rates.
    fn update_rates(&mut self) -> FeeEstimatorResult<()>;
}

/// Network fee source seam used by live and simulated-live estimators.
pub trait FeeRateSource {
    /// Return current network fee rate per weight unit.
    fn get_fee_per_weight(&self) -> FeeEstimatorResult<u64>;
}

#[derive(Debug)]
enum RateMode<S: FeeRateSource> {
    Static,
    Network {
        source: S,
        cache_ttl_secs: u64,
        cached_rate: Option<(u64, u64)>,
    },
}

/// Default FeeEstimator implementation.
///
/// Estimates transaction fees based on size and network conditions.
#[derive(Debug)]
pub struct FeeEstimatorImpl<T: TimeProvider, S: FeeRateSource = NoopFeeRateSource> {
    time_provider: T,
    min_fee: u64,
    fee_per_weight: u64,
    rate_mode: RateMode<S>,
}

/// Default no-network source used by static-rate mode.
#[derive(Debug, Clone, Copy)]
pub struct NoopFeeRateSource;

impl FeeRateSource for NoopFeeRateSource {
    fn get_fee_per_weight(&self) -> FeeEstimatorResult<u64> {
        Err(FeeEstimatorError::Network(
            "network fee source is not configured".to_string(),
        ))
    }
}

impl<T: TimeProvider> FeeEstimatorImpl<T, NoopFeeRateSource> {
    /// Create new fee estimator.
    ///
    /// # Arguments
    /// - `time_provider` - Time provider for cache timestamps
    /// - `min_fee` - Minimum fee to prevent stuck transactions
    /// - `fee_per_weight` - Base fee rate per tx weight unit
    pub fn new(time_provider: T, min_fee: u64, fee_per_weight: u64) -> Self {
        Self::with_static_rate(time_provider, min_fee, fee_per_weight)
    }

    /// Create estimator in Phase 1 static-rate mode.
    pub fn with_static_rate(time_provider: T, min_fee: u64, fee_per_weight: u64) -> Self {
        Self {
            time_provider,
            min_fee,
            fee_per_weight,
            rate_mode: RateMode::Static,
        }
    }
}

impl<T: TimeProvider, S: FeeRateSource> FeeEstimatorImpl<T, S> {
    /// Create estimator in Phase 2 network-rate mode with TTL cache.
    pub fn with_network_rate(
        time_provider: T,
        min_fee: u64,
        initial_fee_per_weight: u64,
        source: S,
        cache_ttl_secs: u64,
    ) -> Self {
        Self {
            time_provider,
            min_fee,
            fee_per_weight: initial_fee_per_weight,
            rate_mode: RateMode::Network {
                source,
                cache_ttl_secs,
                cached_rate: None,
            },
        }
    }

    /// Estimate by precomputed transaction weight.
    pub fn estimate_by_weight(&self, tx_weight: TxWeight) -> FeeEstimatorResult<FeeEstimate> {
        let range_bits = (tx_weight.kernels as u64)
            .checked_mul(RANGE_BITS_PER_KERNEL)
            .ok_or_else(|| {
                FeeEstimatorError::EstimationFailed("range bits overflow".to_string())
            })?;
        let medium = calc_fee_units(GasCount {
            inputs: tx_weight.inputs,
            outputs: tx_weight.outputs,
            range_bits: range_bits as usize,
        })?
        .max(self.min_fee);
        let low = ((medium * 8) / 10).max(self.min_fee);
        Ok(FeeEstimate {
            low,
            medium,
            high: (medium * 15) / 10,
        })
    }
}

impl<T: TimeProvider, S: FeeRateSource> FeeEstimator for FeeEstimatorImpl<T, S> {
    fn estimate(&self, tx_bytes: &[u8]) -> FeeEstimatorResult<FeeEstimate> {
        let weight = compute_tx_weight(tx_bytes)?;
        self.estimate_by_size(weight as usize)
    }

    fn estimate_by_size(&self, size_bytes: usize) -> FeeEstimatorResult<FeeEstimate> {
        let base_fee = (size_bytes as u64)
            .checked_mul(self.fee_per_weight)
            .ok_or_else(|| FeeEstimatorError::EstimationFailed("size fee overflow".to_string()))?;
        let base_fee = base_fee.max(self.min_fee);
        let low_fee = ((base_fee * 8) / 10).max(self.min_fee);

        Ok(FeeEstimate {
            low: low_fee,               // 0.8x with min fee floor
            medium: base_fee,           // 1.0x
            high: (base_fee * 15) / 10, // 1.5x
        })
    }

    fn get_fee_per_byte(&self) -> FeeEstimatorResult<u64> {
        Ok(self.fee_per_weight)
    }

    fn get_minimum_fee(&self) -> FeeEstimatorResult<u64> {
        Ok(self.min_fee)
    }

    fn update_rates(&mut self) -> FeeEstimatorResult<()> {
        let now = self.time_provider.compat_unix_timestamp();
        match &mut self.rate_mode {
            RateMode::Static => Ok(()),
            RateMode::Network {
                source,
                cache_ttl_secs,
                cached_rate,
            } => {
                if let Some((rate, cached_at)) = *cached_rate {
                    if now.saturating_sub(cached_at) < *cache_ttl_secs {
                        self.fee_per_weight = rate;
                        return Ok(());
                    }
                }

                match source.get_fee_per_weight().and_then(validate_live_fee_rate) {
                    Ok(fresh_rate) => {
                        self.fee_per_weight = fresh_rate;
                        *cached_rate = Some((fresh_rate, now));
                    }
                    Err(_) => {
                        let fallback = resolve_fallback_rate(*cached_rate, self.fee_per_weight)?;
                        self.fee_per_weight = fallback;
                        *cached_rate = Some((fallback, now));
                    }
                }
                Ok(())
            }
        }
    }
}

fn validate_live_fee_rate(rate: u64) -> FeeEstimatorResult<u64> {
    if rate == 0 {
        return Err(FeeEstimatorError::Network(
            "network fee source returned zero rate".to_string(),
        ));
    }
    Ok(rate)
}

fn resolve_fallback_rate(
    cached_rate: Option<(u64, u64)>,
    current_rate: u64,
) -> FeeEstimatorResult<u64> {
    let fallback = cached_rate.map(|(rate, _)| rate).unwrap_or(current_rate);
    if fallback == 0 {
        return Err(FeeEstimatorError::Config(
            "fee rate fallback cannot be zero".to_string(),
        ));
    }
    Ok(fallback)
}

/// Canonical fee-weight model version tag.
pub const FEE_WEIGHT_TAG: &str = "fee-weight-v1";
/// Base gas cost term.
pub const BASE_TX_COST: u64 = 64;
/// Per-input gas cost term.
pub const PER_INPUT_COST: u64 = 96;
/// Per-output gas cost term.
pub const PER_OUTPUT_COST: u64 = 900;
/// Per-range-proof-bit gas cost term.
pub const PER_RANGE_BIT_COST: u64 = 1;
/// Backward-compatibility conversion from `kernels` to range-proof bits.
pub const RANGE_BITS_PER_KERNEL: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Canonical counters used for deterministic gas/fee computation.
pub struct GasCount {
    /// Number of transaction inputs.
    pub inputs: usize,
    /// Number of transaction outputs.
    pub outputs: usize,
    /// Total number of range-proof bits.
    pub range_bits: usize,
}

/// Calculate canonical fee units from tx wire counts.
pub fn calculate_fee_for_wires(inputs: usize, outputs: &[AssetWire]) -> FeeEstimatorResult<u64> {
    let mut range_bits = 0usize;
    for output in outputs {
        let out_bits = output
            .range_proof
            .as_ref()
            .map(|proof| proof.len().saturating_mul(8))
            .unwrap_or(0);
        range_bits = range_bits.checked_add(out_bits).ok_or_else(|| {
            FeeEstimatorError::EstimationFailed("range bits overflow".to_string())
        })?;
    }

    calc_fee_units(GasCount {
        inputs,
        outputs: outputs.len(),
        range_bits,
    })
}

pub(crate) fn calc_fee_units(gas: GasCount) -> FeeEstimatorResult<u64> {
    let in_cost = (gas.inputs as u64)
        .checked_mul(PER_INPUT_COST)
        .ok_or_else(|| FeeEstimatorError::EstimationFailed("input cost overflow".to_string()))?;
    let out_cost = (gas.outputs as u64)
        .checked_mul(PER_OUTPUT_COST)
        .ok_or_else(|| FeeEstimatorError::EstimationFailed("output cost overflow".to_string()))?;
    let bit_cost = (gas.range_bits as u64)
        .checked_mul(PER_RANGE_BIT_COST)
        .ok_or_else(|| FeeEstimatorError::EstimationFailed("range cost overflow".to_string()))?;

    BASE_TX_COST
        .checked_add(in_cost)
        .and_then(|sum| sum.checked_add(out_cost))
        .and_then(|sum| sum.checked_add(bit_cost))
        .ok_or_else(|| FeeEstimatorError::EstimationFailed("gas units overflow".to_string()))
}

fn count_array_field(v: &z00z_utils::codec::Value, name: &str) -> usize {
    v.get(name)
        .and_then(z00z_utils::codec::Value::as_array)
        .map_or(0, Vec::len)
}

fn compute_tx_weight(tx_bytes: &[u8]) -> FeeEstimatorResult<u64> {
    let codec = JsonCodec;
    let parsed = codec
        .deserialize::<z00z_utils::codec::Value>(tx_bytes)
        .map_err(|err| {
            FeeEstimatorError::InvalidTransaction(format!("invalid tx wire payload: {err}"))
        })?;

    let tx_node = parsed.get("tx").unwrap_or(&parsed);
    let range_bits = tx_node
        .get("outputs")
        .and_then(z00z_utils::codec::Value::as_array)
        .map(|outputs| outputs.iter().map(calc_out_bits).sum())
        .unwrap_or(0usize);

    let gas = GasCount {
        inputs: count_array_field(tx_node, "inputs"),
        outputs: count_array_field(tx_node, "outputs"),
        range_bits,
    };
    calc_fee_units(gas)
}

fn calc_out_bits(out: &z00z_utils::codec::Value) -> usize {
    out.get("leaf")
        .and_then(|leaf| leaf.get("range_proof"))
        .map(calc_val_bits)
        .unwrap_or(0)
}

fn calc_val_bits(val: &z00z_utils::codec::Value) -> usize {
    if let Some(arr) = val.as_array() {
        return arr.len().saturating_mul(8);
    }
    if let Some(text) = val.as_str() {
        return text.len().saturating_mul(4);
    }
    0
}

#[cfg(test)]
#[path = "test_fee_estimator.rs"]
mod tests;
