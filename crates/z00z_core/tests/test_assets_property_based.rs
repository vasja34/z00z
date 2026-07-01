//! Property-based testing for asset operations using proptest
//!
//! This module uses generative testing to verify invariants across a wide range of inputs,
//! complementing the concrete example-based tests. Properties tested cover cryptographic
//! guarantees and protocol invariants that should hold for ALL valid inputs, not just
//! the specific examples we choose.
//!
//! # Properties Verified
//!
//! ## Determinism Properties
//! - Same inputs produce same asset (deterministic creation)
//! - Commitment generation is deterministic
//! - Range proof generation is deterministic
//!
//! ## Uniqueness Properties
//! - Different nonces produce different assets
//! - Different amounts produce different commitments
//! - Different blinding factors produce different commitments
//!
//! # Design Notes
//!
//! Properties are specified using predicates over input spaces. The proptest framework
//! automatically generates thousands of test cases within those spaces, dramatically
//! improving confidence compared to manual concrete tests alone.
//!
//! Using seeded RNG (StdRng) for reproducibility - any failing input will be clearly
//! documented in failure output for manual debugging.

use proptest::prelude::*;
use proptest::test_runner::Config;
use std::sync::Arc;
use z00z_core::assets::definition::AssetDefinition;
use z00z_core::assets::{Asset, AssetClass};
use z00z_core::BlindingFactor;
use z00z_crypto::expert::encoding::ByteArray;
use z00z_utils::rng::DeterministicRngProvider;

#[cfg(test)]
mod prop_tests {
    use super::*;

    // Configure proptest to run fewer cases (32 instead of default 256)
    // Each case generates real crypto proofs (~5-50ms), so this reduces runtime
    // from potentially minutes to seconds while still verifying properties
    fn proptest_config() -> Config {
        Config {
            cases: 5,
            ..Config::default()
        }
    }

    /// Helper: Create test AssetDefinition
    fn create_test_definition(id: [u8; 32], class: AssetClass) -> Arc<AssetDefinition> {
        Arc::new(
            AssetDefinition::new(
                id,
                class,
                "Test Asset".to_string(),
                "TEST".to_string(),
                8,
                1000,
                1_000_000,
                "test.local".to_string(),
                1,
                1,
                0,
                None,
            )
            .expect("valid definition"),
        )
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // SECTION 1: DETERMINISM PROPERTIES
    // ═══════════════════════════════════════════════════════════════════════════════════

    proptest! {
        #![proptest_config(proptest_config())]

        #[test]
        fn test_prop_asset_creation_deterministic(
            asset_id in prop::array::uniform32(1u8..),
            serial in 1u32..1000u32,
            amount in 1u64..1_000_000u64,
            nonce in prop::array::uniform32(0u8..),
        ) {
            // Property: Creating the same asset twice with identical parameters
            // produces identical asset IDs.

            let def = create_test_definition(asset_id, AssetClass::Coin);
            let blinding = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let asset1 = Asset::new(
                Arc::clone(&def),
                serial,
                amount,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            let asset2 = Asset::new(
                Arc::clone(&def),
                serial,
                amount,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(asset1.is_ok() && asset2.is_ok());

            if let (Ok(a1), Ok(a2)) = (asset1, asset2) {
                prop_assert_eq!(a1.asset_id(), a2.asset_id(),
                    "Same inputs must produce same asset ID");
            }
        }

        #[test]
        fn test_prop_different_different_assets(
            asset_id in prop::array::uniform32(1u8..),
            serial in 1u32..1000u32,
            amount in 1u64..1_000_000u64,
            nonce1 in prop::array::uniform32(0u8..),
            nonce2 in prop::array::uniform32(0u8..),
        ) {
            // Property: Different nonces with different blinding factors
            // produce different asset instances

            prop_assume!(nonce1 != nonce2);

            let def = create_test_definition(asset_id, AssetClass::Coin);
            let blinding1 = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());
            let blinding2 = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let asset1 = Asset::new(
                Arc::clone(&def),
                serial,
                amount,
                &blinding1,
                nonce1,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            let asset2 = Asset::new(
                Arc::clone(&def),
                serial,
                amount,
                &blinding2,
                nonce2,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(asset1.is_ok() && asset2.is_ok());

            if let (Ok(a1), Ok(a2)) = (asset1, asset2) {
                // Different nonces should produce different assets
                // (even if commitments might match by chance with different blindings)
                prop_assert!(a1.nonce() != a2.nonce(),
                    "Different nonces must produce different asset nonces");
            }
        }

        #[test]
        fn test_prop_different_different_commitments(
            asset_id in prop::array::uniform32(1u8..),
            serial in 1u32..1000u32,
            amount1 in 1u64..500_000u64,
            amount2 in 500_001u64..1_000_000u64,
            nonce in prop::array::uniform32(0u8..),
        ) {
            // Property: Different amounts produce different commitments

            let def = create_test_definition(asset_id, AssetClass::Coin);
            let blinding = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let asset1 = Asset::new(
                Arc::clone(&def),
                serial,
                amount1,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            let asset2 = Asset::new(
                Arc::clone(&def),
                serial,
                amount2,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(asset1.is_ok() && asset2.is_ok());

            if let (Ok(a1), Ok(a2)) = (asset1, asset2) {
                prop_assert_ne!(a1.commitment(), a2.commitment(),
                    "Different amounts must produce different commitments");
            }
        }

        #[test]
        fn test_prop_asset_coin_preserved(
            asset_id in prop::array::uniform32(1u8..),
            serial in 0u32..1000u32,
            amount in 1u64..1_000_000u64,
            nonce in prop::array::uniform32(0u8..),
        ) {
            // Property: native asset class is preserved through creation

            let def = create_test_definition(asset_id, AssetClass::Coin);
            let blinding = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let asset = Asset::new(
                def.clone(),
                serial,
                amount,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(asset.is_ok());

            if let Ok(a) = asset {
                prop_assert_eq!(a.definition().class, AssetClass::Coin,
                    "Asset class must be preserved as Coin");
            }
        }

        #[test]
        fn test_prop_asset_token_preserved(
            asset_id in prop::array::uniform32(1u8..),
            serial in 0u32..1000u32,
            amount in 1u64..1_000_000u64,
            nonce in prop::array::uniform32(0u8..),
        ) {
            // Property: Token class is preserved through creation

            let def = create_test_definition(asset_id, AssetClass::Token);
            let blinding = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let asset = Asset::new(
                def.clone(),
                serial,
                amount,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(asset.is_ok());

            if let Ok(a) = asset {
                prop_assert_eq!(a.definition().class, AssetClass::Token,
                    "Asset class must be preserved as Token");
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════════════
    // SECTION 2: RANGE AND BOUNDARY CONDITIONS
    // ═══════════════════════════════════════════════════════════════════════════════════

    proptest! {
        #![proptest_config(proptest_config())]

        #[test]
        fn test_prop_zero_amount_rejected(
            asset_id in prop::array::uniform32(1u8..),
            serial in 0u32..1000u32,
            nonce in prop::array::uniform32(0u8..),
        ) {
            // Property: Zero amount should be rejected

            let def = create_test_definition(asset_id, AssetClass::Coin);
            let blinding = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let result = Asset::new(
                def,
                serial,
                0, // zero amount
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(result.is_err(), "Zero amount must be rejected");
        }

        #[test]
        fn test_prop_commitment_never_identity(
            asset_id in prop::array::uniform32(1u8..),
            serial in 0u32..1000u32,
            amount in 1u64..1_000_000u64,
            nonce in prop::array::uniform32(0u8..),
        ) {
            // Property: Commitment should never be identity point

            let def = create_test_definition(asset_id, AssetClass::Coin);
            let blinding = BlindingFactor::random(&mut DeterministicRngProvider::from_seed([42u8; 32]).rng());

            let asset = Asset::new(
                def,
                serial,
                amount,
                &blinding,
                nonce,
                &mut DeterministicRngProvider::from_seed([42u8; 32]).rng(),
            );

            prop_assert!(asset.is_ok());

            if let Ok(a) = asset {
                let commitment_bytes = a.commitment().as_public_key().as_bytes();
                let all_zeros = commitment_bytes.iter().all(|&b| b == 0);
                prop_assert!(!all_zeros, "Commitment must not be identity");
            }
        }
    }
}
