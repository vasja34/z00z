//! Asset selection for transaction building.

use rand::Rng;
use std::cmp::Reverse;
use thiserror::Error;
use z00z_core::assets::Asset;
use z00z_utils::rng::SecureRngProvider;

/// Asset selection strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    /// Minimize the number of inputs.
    MinInputs,
    /// Minimize estimated fees.
    MinFees,
    /// Maximize privacy (randomized selection).
    MaxPrivacy,
}

/// Asset selector errors.
#[derive(Debug, Error)]
pub enum AssetSelectorError {
    /// Insufficient funds.
    #[error("insufficient funds: need {needed}, have {available}")]
    InsufficientFunds {
        /// Amount required to satisfy the selection.
        needed: u64,
        /// Total amount available for selection.
        available: u64,
    },

    /// No spendable assets are available.
    #[error("no assets available")]
    NoAssets,

    /// Selection failed.
    #[error("selection failed: {0}")]
    SelectionFailed(String),

    /// Amount arithmetic overflow.
    #[error("amount overflow")]
    AmountOverflow,

    /// Amount arithmetic underflow.
    #[error("amount underflow")]
    AmountUnderflow,
}

/// Asset selector result type.
pub type AssetSelectorResult<T> = std::result::Result<T, AssetSelectorError>;

/// Selected assets for a transaction.
#[derive(Debug, Clone)]
pub struct AssetSelection {
    /// Selected input assets.
    pub inputs: Vec<Asset>,
    /// Total selected amount.
    pub total_amount: u64,
    /// Calculated change amount.
    pub change_amount: u64,
}

/// Asset selector trait.
pub trait AssetSelector {
    /// Select assets for a transaction.
    fn select(
        &self,
        available: &[Asset],
        target_amount: u64,
        fee: u64,
        strategy: SelectionStrategy,
    ) -> AssetSelectorResult<AssetSelection>;

    /// Calculate change amount.
    fn calculate_change(&self, inputs: &[Asset], amount: u64, fee: u64)
        -> AssetSelectorResult<u64>;
}

/// Default AssetSelector implementation.
///
/// Selects assets for transaction inputs using various strategies.
#[derive(Debug)]
pub struct AssetSelectorImpl<R: SecureRngProvider> {
    rng_provider: R,
}

impl<R: SecureRngProvider> AssetSelectorImpl<R> {
    /// Create new asset selector.
    ///
    /// # Arguments
    /// - `rng_provider` - RNG provider for MaxPrivacy strategy
    pub fn new(rng_provider: R) -> Self {
        Self { rng_provider }
    }
}

impl<R: SecureRngProvider> AssetSelector for AssetSelectorImpl<R> {
    fn select(
        &self,
        available: &[Asset],
        target_amount: u64,
        fee: u64,
        strategy: SelectionStrategy,
    ) -> AssetSelectorResult<AssetSelection> {
        if available.is_empty() {
            return Err(AssetSelectorError::NoAssets);
        }

        let needed = target_amount
            .checked_add(fee)
            .ok_or(AssetSelectorError::AmountOverflow)?;
        if needed == 0 {
            return Ok(AssetSelection {
                inputs: Vec::new(),
                total_amount: 0,
                change_amount: 0,
            });
        }

        let mut candidates: Vec<Asset> = available.to_vec();
        match strategy {
            SelectionStrategy::MinInputs => {
                candidates.sort_by_key(|asset| Reverse(asset.amount));
            }
            SelectionStrategy::MinFees => {
                // Phase 1 approximation: treat fee as fixed and prefer fewer inputs.
                candidates.sort_by_key(|asset| Reverse(asset.amount));
            }
            SelectionStrategy::MaxPrivacy => {
                // Phase 1: randomized selection order using injected RNG.
                let mut rng = self.rng_provider.rng();
                for i in (1..candidates.len()).rev() {
                    let j = rng.gen_range(0..=i);
                    candidates.swap(i, j);
                }
            }
        }

        let mut inputs = Vec::new();
        let mut total_amount = 0u64;
        for asset in candidates.into_iter() {
            total_amount = total_amount
                .checked_add(asset.amount)
                .ok_or(AssetSelectorError::AmountOverflow)?;
            inputs.push(asset);
            if total_amount >= needed {
                break;
            }
        }

        if total_amount < needed {
            return Err(AssetSelectorError::InsufficientFunds {
                needed,
                available: total_amount,
            });
        }

        let change_amount = self.calculate_change(&inputs, target_amount, fee)?;

        Ok(AssetSelection {
            inputs,
            total_amount,
            change_amount,
        })
    }

    fn calculate_change(
        &self,
        inputs: &[Asset],
        amount: u64,
        fee: u64,
    ) -> AssetSelectorResult<u64> {
        let total_inputs = inputs.iter().try_fold(0u64, |acc, a| {
            acc.checked_add(a.amount)
                .ok_or(AssetSelectorError::AmountOverflow)
        })?;

        let total_needed = amount
            .checked_add(fee)
            .ok_or(AssetSelectorError::AmountOverflow)?;

        total_inputs
            .checked_sub(total_needed)
            .ok_or(AssetSelectorError::AmountUnderflow)
    }
}

#[cfg(test)]
#[path = "test_asset_selector.rs"]
mod tests;

#[allow(missing_docs)]
/// Canonical multi-asset selector helpers.
#[path = "asset_selector_multi.rs"]
mod multi;

pub use self::multi::{
    build_selection_fixture, check_batch, check_statement, derive_output_id, is_low_link,
    link_score, InRef, MultiErr, MultiStmt, OutRef, SelCase, SelFix,
};
