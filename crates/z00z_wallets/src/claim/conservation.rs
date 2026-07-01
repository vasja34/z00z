//! Claim conservation invariants.

use z00z_core::Asset;
use z00z_crypto::Z00ZCommitment;

/// Runtime conservation verification errors.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConservationError {
    /// Amount sums differ between input and imported assets.
    #[error("amount conservation mismatch: input={input}, imported={imported}")]
    AmountMismatch {
        /// Sum from source input assets.
        input: u128,
        /// Sum from imported assets.
        imported: u128,
    },
    /// Amount sum overflow was detected.
    #[error("amount conservation overflow")]
    AmountOverflow,
    /// Commitment sums differ between input and imported assets.
    #[error("commitment conservation mismatch")]
    CommitmentMismatch,
}

/// Verify amount and commitment conservation for claim flow.
pub fn verify_claim_conservation(
    input: &[Asset],
    imported: &[Asset],
) -> Result<(), ConservationError> {
    let input_sum = sum_amount(input)?;
    let imported_sum = sum_amount(imported)?;
    if input_sum != imported_sum {
        return Err(ConservationError::AmountMismatch {
            input: input_sum,
            imported: imported_sum,
        });
    }

    let input_commit = sum_commit(input);
    let imported_commit = sum_commit(imported);
    if input_commit != imported_commit {
        return Err(ConservationError::CommitmentMismatch);
    }

    Ok(())
}

fn sum_amount(items: &[Asset]) -> Result<u128, ConservationError> {
    let mut total = 0u128;
    for item in items {
        total = total
            .checked_add(u128::from(item.amount))
            .ok_or(ConservationError::AmountOverflow)?;
    }
    Ok(total)
}

fn sum_commit(items: &[Asset]) -> Option<Z00ZCommitment> {
    let mut it = items.iter();
    let first = it.next()?.commitment.clone();
    Some(it.fold(first, |acc, item| &acc + &item.commitment))
}

#[cfg(test)]
mod tests {
    use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass};

    use super::{verify_claim_conservation, ConservationError};

    fn mk_assets() -> Vec<z00z_core::Asset> {
        vec![
            asset_from_dev_class(AssetClass::Coin, 1, 10).expect("asset 1"),
            asset_from_dev_class(AssetClass::Coin, 2, 20).expect("asset 2"),
        ]
    }

    #[test]
    fn test_verify_conservation_ok() {
        let input_assets = mk_assets();
        let imported_assets = input_assets.clone();
        assert!(verify_claim_conservation(&input_assets, &imported_assets).is_ok());
    }

    #[test]
    fn test_verify_conservation_amount_fail() {
        let input_assets = mk_assets();
        let mut imported_assets = input_assets.clone();
        imported_assets[0].amount = imported_assets[0].amount.checked_add(1).expect("bump");

        let result = verify_claim_conservation(&input_assets, &imported_assets);
        assert!(matches!(
            result,
            Err(ConservationError::AmountMismatch {
                input: 30,
                imported: 31
            })
        ));
    }

    #[test]
    fn test_verify_conservation_commit_fail() {
        let input_assets = mk_assets();
        let mut imported_assets = input_assets.clone();
        let replacement = asset_from_dev_class(
            AssetClass::Coin,
            imported_assets[0].serial_id.saturating_add(1),
            imported_assets[0].amount,
        )
        .expect("replacement");
        imported_assets[0].commitment = replacement.commitment;

        let result = verify_claim_conservation(&input_assets, &imported_assets);
        assert!(matches!(result, Err(ConservationError::CommitmentMismatch)));
    }
}
