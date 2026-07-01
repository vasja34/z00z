// Threat T-5 anchor: TxVerifierImpl::verify performs pre-broadcast checks only; it is not the canonical admission gate.
//! Transaction package verifier for pre-broadcast checks.

use super::fee_estimator::{calc_fee_units, GasCount};
use super::spend_verification::verify_tx_public_spend_contract;
pub use super::tx_digest::build_tx_package_digest;
pub use super::tx_verifier_errors::{TxVerifierError, TxVerifierResult};
use super::tx_wire::decode_tx_input_asset_id;
pub use super::tx_wire::{
    TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire, TxPackage, TxProofWire, TxWire,
};
pub(crate) use super::tx_wire::{REGULAR_TX_PACKAGE_TYPE, REGULAR_TX_TYPE, TX_PACKAGE_KIND};
use std::collections::HashSet;
use z00z_core::Asset;
use z00z_utils::codec::{Codec, JsonCodec};

include!("tx_verifier_decode.rs");

/// Verification result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationResult {
    /// Whether the transaction package is valid.
    pub valid: bool,
    /// Human-readable validation errors.
    pub errors: Vec<String>,
}

/// Transaction verifier trait.
pub trait TxVerifier {
    /// Verify a serialized TxPackage payload.
    fn verify(&self, tx_bytes: &[u8]) -> TxVerifierResult<VerificationResult>;

    /// Verify signatures inside a serialized TxPackage payload.
    fn verify_signatures(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool>;

    /// Verify range proofs inside a serialized TxPackage payload.
    fn verify_range_proofs(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool>;

    /// Verify package-level balance assumptions.
    fn verify_balance(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool>;

    /// Verify package structure and mandatory fields.
    fn verify_structure(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool>;
}

/// Default transaction verifier implementation.
#[derive(Debug)]
pub struct TxVerifierImpl;

impl TxVerifierImpl {
    /// Create a new transaction verifier.
    pub fn new() -> Self {
        Self
    }

    fn decode_package(&self, tx_bytes: &[u8]) -> TxVerifierResult<TxPackage> {
        if tx_bytes.is_empty() {
            return Err(TxVerifierError::InvalidStructure(
                "empty transaction package payload".to_string(),
            ));
        }

        let codec = JsonCodec;
        codec.deserialize::<TxPackage>(tx_bytes).map_err(|err| {
            TxVerifierError::InvalidStructure(format!("decode tx package failed: {err}"))
        })
    }

    fn decode_assets(&self, tx_bytes: &[u8]) -> TxVerifierResult<Vec<Asset>> {
        decode_assets_from_package(self.decode_package(tx_bytes)?)
    }

    fn verify_digest(&self, pkg: &TxPackage) -> TxVerifierResult<bool> {
        let expected = build_tx_package_digest(
            &pkg.kind,
            &pkg.package_type,
            pkg.version,
            pkg.chain_id,
            &pkg.chain_type,
            &pkg.chain_name,
            &pkg.tx,
        )
        .map_err(|err| TxVerifierError::InvalidStructure(format!("digest build failed: {err}")))?;
        if pkg.tx_digest_hex != expected {
            return Err(TxVerifierError::InvalidStructure(
                "tx_digest_hex does not match payload".to_string(),
            ));
        }
        Ok(true)
    }

    fn fee_sum(&self, outputs: &[TxOutputWire]) -> TxVerifierResult<u64> {
        fee_sum_from_outputs(outputs)
    }
}

impl Default for TxVerifierImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify the full regular transaction package, including the current public spend contract.
pub fn verify_full_tx_package(tx_bytes: &[u8]) -> TxVerifierResult<VerificationResult> {
    let verifier = TxVerifierImpl::new();
    let report = verifier.verify(tx_bytes)?;
    if !report.valid {
        return Ok(report);
    }

    let pkg = verifier.decode_package(tx_bytes)?;
    match verify_package_public_spend_contract(&pkg) {
        Ok(()) => Ok(report),
        Err(err) => Ok(VerificationResult {
            valid: false,
            errors: vec![err.to_string()],
        }),
    }
}

/// Verify only the public spend contract for a decoded regular transaction package.
pub fn verify_package_public_spend_contract(pkg: &TxPackage) -> TxVerifierResult<()> {
    verify_tx_public_spend_contract(
        pkg.chain_id,
        pkg.version,
        &pkg.chain_type,
        &pkg.chain_name,
        &pkg.tx,
    )
    .map_err(|err| {
        TxVerifierError::VerificationFailed(format!("public spend contract failed: {err}"))
    })
}

impl TxVerifier for TxVerifierImpl {
    fn verify(&self, tx_bytes: &[u8]) -> TxVerifierResult<VerificationResult> {
        if let Err(err) = self.verify_structure(tx_bytes) {
            return Ok(VerificationResult {
                valid: false,
                errors: vec![err.to_string()],
            });
        }
        if let Err(err) = self.verify_balance(tx_bytes) {
            return Ok(VerificationResult {
                valid: false,
                errors: vec![err.to_string()],
            });
        }
        if let Err(err) = self.verify_digest(&self.decode_package(tx_bytes)?) {
            return Ok(VerificationResult {
                valid: false,
                errors: vec![err.to_string()],
            });
        }
        if let Err(err) = self.verify_signatures(tx_bytes) {
            return Ok(VerificationResult {
                valid: false,
                errors: vec![err.to_string()],
            });
        }
        if let Err(err) = self.verify_range_proofs(tx_bytes) {
            return Ok(VerificationResult {
                valid: false,
                errors: vec![err.to_string()],
            });
        }

        Ok(VerificationResult {
            valid: true,
            errors: Vec::new(),
        })
    }

    fn verify_signatures(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool> {
        let assets = self.decode_assets(tx_bytes)?;
        for asset in assets {
            if asset.owner_signature.is_some() {
                asset.verify_owner_signature().map_err(|err| {
                    TxVerifierError::InvalidSignature(format!(
                        "owner signature check failed: {err}"
                    ))
                })?;
            }
        }
        Ok(true)
    }

    fn verify_range_proofs(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool> {
        let assets = self.decode_assets(tx_bytes)?;
        for asset in assets {
            match asset.definition.class {
                z00z_core::assets::AssetClass::Coin | z00z_core::assets::AssetClass::Token => {
                    asset.verify_range_proof().map_err(|err| {
                        TxVerifierError::InvalidRangeProof(format!(
                            "range proof check failed: {err}"
                        ))
                    })?;
                }
                z00z_core::assets::AssetClass::Nft | z00z_core::assets::AssetClass::Void => {
                    if asset.range_proof.is_some() {
                        asset.verify_range_proof().map_err(|err| {
                            TxVerifierError::InvalidRangeProof(format!(
                                "range proof check failed: {err}"
                            ))
                        })?;
                    }
                }
            }
        }
        Ok(true)
    }

    fn verify_balance(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool> {
        let pkg = self.decode_package(tx_bytes)?;
        if pkg.tx.inputs.is_empty() {
            return Err(TxVerifierError::VerificationFailed(
                "transaction must contain at least one input".to_string(),
            ));
        }
        if pkg.tx.outputs.is_empty() {
            return Err(TxVerifierError::VerificationFailed(
                "transaction must contain at least one output".to_string(),
            ));
        }

        let mut input_keys = HashSet::new();
        for input in &pkg.tx.inputs {
            let input_key = decode_tx_input_asset_id(&input.asset_id_hex)
                .map_err(|err| TxVerifierError::VerificationFailed(err.to_string()))?;
            if !input_keys.insert(hex::encode(input_key)) {
                return Err(TxVerifierError::VerificationFailed(
                    "duplicate tx input state_key".to_string(),
                ));
            }
        }

        let mut range_bits = 0usize;
        let mut fee_n = 0usize;
        let mut seen_nonce = HashSet::new();
        let mut seen_out_keys = HashSet::new();
        for output in &pkg.tx.outputs {
            let asset = output.asset_wire.clone().to_asset().map_err(|err| {
                TxVerifierError::InvalidStructure(format!("asset decode failed: {err}"))
            })?;
            let out_key = hex::encode(asset.asset_id());
            if !seen_out_keys.insert(out_key.clone()) {
                return Err(TxVerifierError::VerificationFailed(
                    "duplicate tx output state_key".to_string(),
                ));
            }
            if input_keys.contains(out_key.as_str()) {
                return Err(TxVerifierError::VerificationFailed(
                    "tx output state_key must not overlap consumed input".to_string(),
                ));
            }

            let out_bits = asset
                .range_proof
                .as_ref()
                .map(|proof| proof.len().saturating_mul(8))
                .unwrap_or(0);
            range_bits = range_bits.checked_add(out_bits).ok_or_else(|| {
                TxVerifierError::VerificationFailed("range bits overflow".to_string())
            })?;

            if asset.nonce == [0u8; 32] {
                return Err(TxVerifierError::VerificationFailed(
                    "transaction output nonce must be non-zero".to_string(),
                ));
            }
            if !seen_nonce.insert(asset.nonce) {
                return Err(TxVerifierError::VerificationFailed(
                    "duplicate transaction output nonce".to_string(),
                ));
            }

            match asset.definition.class {
                z00z_core::assets::AssetClass::Coin | z00z_core::assets::AssetClass::Token => {
                    if asset.amount == 0 {
                        return Err(TxVerifierError::VerificationFailed(
                            "coin/token output amount must be positive".to_string(),
                        ));
                    }
                }
                z00z_core::assets::AssetClass::Nft | z00z_core::assets::AssetClass::Void => {
                    if asset.amount != 0 {
                        return Err(TxVerifierError::VerificationFailed(
                            "nft/void output amount must be zero".to_string(),
                        ));
                    }
                }
            }

            if output.role == TxOutRole::Fee {
                fee_n = fee_n.checked_add(1).ok_or_else(|| {
                    TxVerifierError::VerificationFailed("fee output count overflow".to_string())
                })?;
            }
        }

        let fee_sum = self.fee_sum(&pkg.tx.outputs)?;
        if pkg.tx.fee == 0 {
            if fee_n != 0 {
                return Err(TxVerifierError::VerificationFailed(
                    "fee outputs are forbidden when declared fee is zero".to_string(),
                ));
            }
        } else if fee_n == 0 {
            return Err(TxVerifierError::VerificationFailed(
                "fee outputs are required when declared fee is positive".to_string(),
            ));
        }
        if pkg.tx.fee != fee_sum {
            return Err(TxVerifierError::VerificationFailed(
                "declared fee must equal sum of fee outputs".to_string(),
            ));
        }

        let expect_fee = calc_fee_units(GasCount {
            inputs: pkg.tx.inputs.len(),
            outputs: pkg.tx.outputs.len(),
            range_bits,
        })
        .map_err(|err| TxVerifierError::VerificationFailed(err.to_string()))?;
        if pkg.tx.fee != expect_fee {
            return Err(TxVerifierError::VerificationFailed(
                "declared fee mismatch".to_string(),
            ));
        }

        // Package-level verification stays local and honest here: inputs are
        // reference-only, so full value conservation depends on tx-proof
        // validation or a resolved pre-state path. Membership remains a
        // separate checkpoint failure mode and is not inferred from TxInputWire.

        Ok(true)
    }

    fn verify_structure(&self, tx_bytes: &[u8]) -> TxVerifierResult<bool> {
        let pkg = self.decode_package(tx_bytes)?;

        if pkg.kind != TX_PACKAGE_KIND {
            return Err(TxVerifierError::InvalidStructure(
                "unsupported tx package kind".to_string(),
            ));
        }
        if pkg.package_type != REGULAR_TX_PACKAGE_TYPE {
            return Err(TxVerifierError::InvalidStructure(
                "unsupported tx package subtype".to_string(),
            ));
        }
        if pkg.version == 0 {
            return Err(TxVerifierError::InvalidStructure(
                "tx package version must be non-zero".to_string(),
            ));
        }
        if pkg.chain_id == 0 || pkg.chain_type.trim().is_empty() || pkg.chain_name.trim().is_empty()
        {
            return Err(TxVerifierError::InvalidStructure(
                "tx package chain metadata is incomplete".to_string(),
            ));
        }
        if pkg.tx.outputs.is_empty() {
            return Err(TxVerifierError::InvalidStructure(
                "tx package outputs are empty".to_string(),
            ));
        }
        if pkg.tx.tx_type != REGULAR_TX_TYPE {
            return Err(TxVerifierError::InvalidStructure(
                "tx payload type must be regular_tx".to_string(),
            ));
        }
        if pkg.tx_digest_hex.len() != 64 {
            return Err(TxVerifierError::InvalidStructure(
                "tx digest must be 32-byte hex".to_string(),
            ));
        }
        if !pkg
            .tx_digest_hex
            .as_bytes()
            .iter()
            .all(u8::is_ascii_hexdigit)
        {
            return Err(TxVerifierError::InvalidStructure(
                "tx digest must contain only hex characters".to_string(),
            ));
        }
        if pkg.status.is_empty() {
            return Err(TxVerifierError::InvalidStructure(
                "tx package status is empty".to_string(),
            ));
        }
        Ok(true)
    }
}

#[cfg(test)]
#[path = "test_tx_verifier.rs"]
mod tests;
