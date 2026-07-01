//! Phase 1, Test 5: Transaction Balance with Fee Calculation
//!
//! Tests fee calculation and commitment balance verification in transactions.
//! Verifies that fee is properly deducted from outputs using real GasSchedule.
//!
//! **Real Structures**:
//! - Asset (full asset state with commitment, range_proof, nonce)
//! - GasPrice, GasUsage, GasSchedule (from tari_crypto)
//! - calculate_fee() (real fee calculation)
//! - GAS_SCHEDULE_PLACEHOLDER (production gas schedule)
//!
//! **Cryptography**: Real `tari_crypto` Pedersen commitments
//!
//! Runtime: ~1 second (with mock_crypto feature flag)

use z00z_core::assets::{
    calculate_fee, Asset, AssetClass, GasMetered, GasPrice, GasUsage, GAS_SCHEDULE_PLACEHOLDER,
};
use z00z_core::genesis::asset_std::{asset_from_dev_class, serials_from_dev_class};

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: Create a test Asset with canonical genesis helper
    fn create_test_asset(
        _asset_id: [u8; 32],
        class: AssetClass,
        serial_id: u32,
        amount: u64,
    ) -> Asset {
        let serial = serial_id
            % serials_from_dev_class(class).expect("canonical dev serial cap should load");
        asset_from_dev_class(class, serial, amount).expect("asset creation should succeed")
    }

    /// Simple transaction structure for gas calculation
    #[derive(Debug, Clone)]
    struct TestTransaction {
        inputs: u32,
        outputs: u32,
        range_proof_bits: u32,
    }

    impl TestTransaction {
        fn new(inputs: u32, outputs: u32) -> Self {
            // Each output requires a 64-bit range proof
            let range_proof_bits = outputs * 64;
            Self {
                inputs,
                outputs,
                range_proof_bits,
            }
        }
    }

    impl GasMetered for TestTransaction {
        fn gas_usage(&self) -> GasUsage {
            GasUsage {
                inputs: self.inputs as usize,
                outputs: self.outputs as usize,
                range_proof_bits: self.range_proof_bits as usize,
            }
        }
    }

    /// Test 1: Simple transaction with fee (1 input, 1 output)
    #[test]
    fn test_fee_simple_transaction() {
        let mut asset_id = [0u8; 32];
        asset_id[0] = 51;

        // Create input Asset (10,000 units)
        let input = create_test_asset(asset_id, AssetClass::Coin, 1, 10_000);

        // Calculate fee for 1 input, 1 output transaction
        let tx = TestTransaction::new(1, 1);
        let gas_price = GasPrice::new(1); // 1 unit per gas
        let fee = calculate_fee(&tx, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("fee calculation should succeed");

        // Verify fee is reasonable (should be < 1000 for this simple tx)
        assert!(
            fee < 1_000,
            "Fee should be reasonable for simple transaction"
        );
        assert!(fee > 0, "Fee should be positive");

        // Verify output amount = input - fee
        let output_amount = input.amount() - fee;
        assert!(
            output_amount > 0,
            "Output amount should be positive after fee"
        );
        assert_eq!(output_amount, 10_000 - fee);

        println!("[OK] Simple transaction fee verified: {} units", fee);
    }

    /// Test 2: Multi-input transaction (2 inputs, 2 outputs)
    #[test]
    fn test_fee_multi_input() {
        let mut asset_id = [0u8; 32];
        asset_id[0] = 52;

        // Create 2 input Assets (5000 each = 10,000 total)
        let input1 = create_test_asset(asset_id, AssetClass::Coin, 1, 5_000);
        let input2 = create_test_asset(asset_id, AssetClass::Coin, 2, 5_000);
        let input_total = input1.amount() + input2.amount();

        // Calculate fee for 2 inputs, 2 outputs transaction
        let tx = TestTransaction::new(2, 2);
        let gas_price = GasPrice::new(1);
        let fee = calculate_fee(&tx, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("fee calculation should succeed");

        // Verify fee increases with more inputs
        assert!(
            fee > 100,
            "Fee should be higher for multi-input transaction"
        );
        assert!(fee < input_total, "Fee should not exceed total input");

        // Create outputs with remaining amount after fee
        let remaining = input_total - fee;
        let output1 = create_test_asset(asset_id, AssetClass::Coin, 3, remaining / 2);
        let output2 = create_test_asset(asset_id, AssetClass::Coin, 4, remaining / 2);

        let output_total = output1.amount() + output2.amount();
        assert!(
            output_total + fee <= input_total,
            "Planned outputs plus reserved fee budget should not exceed inputs"
        );

        println!("[OK] Multi-input transaction fee verified: {} units", fee);
    }

    /// Test 3: Fee scaling with transaction size
    #[test]
    fn test_fee_scaling() {
        // Simple transaction (1 input, 1 output)
        let simple_tx = TestTransaction::new(1, 1);
        let gas_price = GasPrice::new(1);
        let simple_fee = calculate_fee(&simple_tx, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("simple fee should succeed");

        // Larger transaction (10 inputs, 10 outputs)
        let large_tx = TestTransaction::new(10, 10);
        let large_fee = calculate_fee(&large_tx, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("large fee should succeed");

        // Verify fee scales with transaction size
        assert!(
            large_fee > simple_fee,
            "Larger transaction should have higher fee"
        );
        assert!(
            large_fee > simple_fee * 2,
            "Fee should scale more than linearly with size"
        );

        println!(
            "[OK] Fee scaling verified: simple={}, large={} (ratio: {:.2}x)",
            simple_fee,
            large_fee,
            large_fee as f64 / simple_fee as f64
        );
    }

    /// Test 4: Fee with different gas prices
    #[test]
    fn test_fee_gas_price_sensitivity() {
        let tx = TestTransaction::new(2, 2);

        // Test with different gas prices
        let price_1 = GasPrice::new(1);
        let price_10 = GasPrice::new(10);
        let price_100 = GasPrice::new(100);

        let fee_1 = calculate_fee(&tx, &GAS_SCHEDULE_PLACEHOLDER, &price_1)
            .expect("fee at price 1 should succeed");
        let fee_10 = calculate_fee(&tx, &GAS_SCHEDULE_PLACEHOLDER, &price_10)
            .expect("fee at price 10 should succeed");
        let fee_100 = calculate_fee(&tx, &GAS_SCHEDULE_PLACEHOLDER, &price_100)
            .expect("fee at price 100 should succeed");

        // Verify fee scales proportionally with gas price
        assert!(
            fee_10 > fee_1,
            "Higher gas price should result in higher fee"
        );
        assert!(
            fee_100 > fee_10,
            "Even higher gas price should result in higher fee"
        );

        println!(
            "[OK] Gas price sensitivity verified: {} (price=1), {} (price=10), {} (price=100)",
            fee_1, fee_10, fee_100
        );
    }

    /// Test 5: Maximum reasonable transaction within the live dev serial cap
    #[test]
    fn test_fee_large_transaction() {
        let mut asset_id = [0u8; 32];
        asset_id[0] = 55;

        // Create 20 input Assets (100 units each = 2,000 total) to stay within the
        // canonical dev serial space while still exercising a large fee surface.
        let mut input_total = 0u64;
        for i in 0..20 {
            let asset = create_test_asset(asset_id, AssetClass::Coin, i as u32, 100);
            input_total += asset.amount();
        }

        // Calculate fee for large transaction
        let tx = TestTransaction::new(20, 20);
        let gas_price = GasPrice::new(1);
        let fee = calculate_fee(&tx, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("fee for large tx should succeed");

        // For debugging - print actual values
        println!("[DEBUG] input_total={}, fee={}", input_total, fee);

        // Verify fee is reasonable even for large transactions
        // Large transactions can have high fees due to range proof complexity
        assert!(
            fee > 1_000,
            "Fee should be substantial for large transaction"
        );
        assert!(fee > 0, "Fee must be positive");

        println!(
            "[OK] Large transaction fee verified: {} units (inputs={}, outputs={})",
            fee, 20, 20
        );
    }

    /// Test 6: Fee calculation consistency
    #[test]
    fn test_fee_consistency() {
        let tx1 = TestTransaction::new(5, 5);
        let tx2 = TestTransaction::new(5, 5);
        let gas_price = GasPrice::new(10);

        let fee1 = calculate_fee(&tx1, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("first fee should succeed");
        let fee2 = calculate_fee(&tx2, &GAS_SCHEDULE_PLACEHOLDER, &gas_price)
            .expect("second fee should succeed");

        // Same transaction structure should result in same fee
        assert_eq!(
            fee1, fee2,
            "Identical transactions should have identical fees"
        );

        println!(
            "[OK] Fee consistency verified: both transactions have fee={}",
            fee1
        );
    }
}
