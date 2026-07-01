use z00z_core::assets::Asset;

use crate::{
    key::ReceiverKeys,
    receiver::{ScanResult, StealthOutputScanner},
    WalletError,
};

/// Verify that a stealth asset is decryptable and commitment-valid for receiver keys.
pub fn check_stealth_own(asset: &Asset, keys: &ReceiverKeys) -> Result<(), WalletError> {
    let scanner = StealthOutputScanner::from_keys(keys);
    if let ScanResult::Mine { .. } = scanner.scan_leaf(asset) {
        return Ok(());
    }
    Err(WalletError::NotOwned)
}
