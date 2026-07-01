//! Transaction assembly.

use std::collections::HashSet;

use thiserror::Error;
use z00z_core::{assets::AssetWire, Asset};
use z00z_crypto::{create_commitment, Z00ZCommitment, Z00ZScalar};
use z00z_utils::codec::{Codec, JsonCodec};

use super::{
    balance::{verify_tx_balance, verify_tx_balance_meta, TxBalErr},
    fee_estimator::{FeeEstimate, FeeEstimator},
    tx_verifier::{
        build_tx_package_digest, verify_full_tx_package, TxAuthWire, TxContextWire, TxInputWire,
        TxOutRole, TxOutputWire, TxPackage, TxProofWire, TxVerifier, TxVerifierImpl, TxWire,
        REGULAR_TX_PACKAGE_TYPE, REGULAR_TX_TYPE, TX_PACKAGE_KIND,
    },
    tx_wire::decode_tx_input_asset_id,
};

/// Transaction assembler errors.
#[derive(Debug, Error)]
pub enum TxAssemblerError {
    /// Assembly failed.
    #[error("assembly failed: {0}")]
    AssemblyFailed(String),

    /// Not enough inputs.
    #[error("insufficient inputs: required {required}, available {available}")]
    InsufficientInputs {
        /// Total input amount required to build the transaction.
        required: u64,
        /// Total input amount available for assembly.
        available: u64,
    },

    /// Commitment is invalid.
    #[error("invalid commitment: {0}")]
    InvalidCommitment(String),

    /// Cryptographic error.
    #[error("cryptographic error: {0}")]
    Crypto(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AsmInputWire {
    asset_id_hex: String,
    serial_id: u32,
    value: u64,
    opening_hex: String,
    commitment_hex: String,
}

impl AsmInputWire {
    fn input_ref(&self) -> TxAssemblerResult<TxInputWire> {
        let asset_id = decode_input_asset_id(&self.asset_id_hex)?;
        Ok(TxInputWire {
            asset_id_hex: hex::encode(asset_id),
            serial_id: self.serial_id,
        })
    }

    fn opening(&self) -> TxAssemblerResult<Z00ZScalar> {
        decode_scalar_hex(&self.opening_hex, "opening_hex")
    }

    fn commitment(&self) -> TxAssemblerResult<Z00ZCommitment> {
        let opening = self.opening()?;
        let expected = create_commitment(self.value, &opening)
            .map_err(|err| TxAssemblerError::Crypto(format!("resolved input commitment: {err}")))?;
        let commitment = decode_commitment_hex(&self.commitment_hex, "commitment_hex")?;

        if commitment.as_bytes() != expected.as_bytes() {
            return Err(TxAssemblerError::InvalidCommitment(
                "resolved input commitment does not match value and opening".to_string(),
            ));
        }

        Ok(commitment)
    }
}

fn decode_input_asset_id(value: &str) -> TxAssemblerResult<[u8; 32]> {
    decode_tx_input_asset_id(value).map_err(|err| TxAssemblerError::AssemblyFailed(err.to_string()))
}

fn decode_scalar_hex(value: &str, field: &'static str) -> TxAssemblerResult<Z00ZScalar> {
    let bytes = decode_hex32(value, field)?;
    Z00ZScalar::try_from_bytes(bytes).map_err(|_| {
        TxAssemblerError::AssemblyFailed(format!("{field} must be 32-byte lowercase hex"))
    })
}

fn decode_commitment_hex(value: &str, field: &'static str) -> TxAssemblerResult<Z00ZCommitment> {
    let bytes = decode_hex32(value, field)?;
    z00z_crypto::Commitment::from_bytes(&bytes)
        .map(|commitment| commitment.as_commitment().clone())
        .map_err(|_| {
            TxAssemblerError::AssemblyFailed(format!("{field} must be 32-byte lowercase hex"))
        })
}

fn decode_hex32(value: &str, field: &'static str) -> TxAssemblerResult<[u8; 32]> {
    let bytes = hex::decode(value)
        .map_err(|err| TxAssemblerError::AssemblyFailed(format!("{field} decode failed: {err}")))?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        TxAssemblerError::AssemblyFailed(format!("{field} must be 32-byte lowercase hex"))
    })?;
    if hex::encode(bytes) != value {
        return Err(TxAssemblerError::AssemblyFailed(format!(
            "{field} must be 32-byte lowercase hex"
        )));
    }
    Ok(bytes)
}

fn decode_asm_input(bytes: &[u8]) -> TxAssemblerResult<AsmInputWire> {
    JsonCodec.deserialize(bytes).map_err(|err| {
        TxAssemblerError::AssemblyFailed(format!("resolved input decode failed: {err}"))
    })
}

pub(crate) fn encode_asm_input_wire(
    asset_id: [u8; 32],
    serial_id: u32,
    value: u64,
    opening: [u8; 32],
    commitment: [u8; 32],
) -> TxAssemblerResult<Vec<u8>> {
    JsonCodec
        .serialize(&AsmInputWire {
            asset_id_hex: hex::encode(asset_id),
            serial_id,
            value,
            opening_hex: hex::encode(opening),
            commitment_hex: hex::encode(commitment),
        })
        .map_err(|err| {
            TxAssemblerError::AssemblyFailed(format!("resolved input encode failed: {err}"))
        })
}

fn decode_tx_output(bytes: &[u8]) -> TxAssemblerResult<TxOutputWire> {
    JsonCodec
        .deserialize(bytes)
        .map_err(|err| TxAssemblerError::AssemblyFailed(format!("tx output decode failed: {err}")))
}

fn lane_is_empty(bytes: &[u8]) -> bool {
    bytes.is_empty()
}

/// Transaction assembler result type.
pub type TxAssemblerResult<T> = std::result::Result<T, TxAssemblerError>;

/// Transaction assembly parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxAssemblyParams {
    /// Serialized wallet-local resolved input DTOs.
    pub inputs_bytes: Vec<Vec<u8>>,
    /// Serialized canonical output wires.
    pub tx_outputs_bytes: Vec<Vec<u8>>,
    /// Fee in native units.
    pub fee: u64,
    /// Canonical chain identifier for the assembled package.
    pub chain_id: u32,
    /// Canonical chain type for the assembled package.
    pub chain_type: String,
    /// Canonical chain name for the assembled package.
    pub chain_name: String,
}

/// Transaction assembler trait.
pub trait TxAssembler {
    /// Assemble a serialized canonical regular transaction package.
    fn assemble(&self, params: TxAssemblyParams) -> TxAssemblerResult<Vec<u8>>;

    /// Sum resolved input values.
    fn sum_inputs(&self, inputs_bytes: &[Vec<u8>]) -> TxAssemblerResult<u64>;

    /// Sum visible output values.
    fn sum_tx_outputs(&self, tx_outputs_bytes: &[Vec<u8>]) -> TxAssemblerResult<u64>;

    /// Verify package balance through the public package verifier.
    fn verify_balance(&self, tx_bytes: &[u8]) -> TxAssemblerResult<()>;
}

/// Default `TxAssembler` implementation for canonical regular packages.
#[derive(Debug)]
pub struct TxAssemblerImpl;

impl TxAssemblerImpl {
    /// Create a new assembler.
    pub fn new() -> Self {
        Self
    }

    fn decode_inputs(&self, inputs_bytes: &[Vec<u8>]) -> TxAssemblerResult<Vec<AsmInputWire>> {
        if inputs_bytes.is_empty() {
            return Err(TxAssemblerError::AssemblyFailed(
                "empty resolved input lane".to_string(),
            ));
        }

        let mut inputs = Vec::with_capacity(inputs_bytes.len());
        for bytes in inputs_bytes {
            if lane_is_empty(bytes) {
                return Err(TxAssemblerError::AssemblyFailed(
                    "empty resolved input lane".to_string(),
                ));
            }
            inputs.push(decode_asm_input(bytes)?);
        }
        Ok(inputs)
    }

    fn decode_outputs(&self, tx_outputs_bytes: &[Vec<u8>]) -> TxAssemblerResult<Vec<TxOutputWire>> {
        if tx_outputs_bytes.is_empty() {
            return Err(TxAssemblerError::AssemblyFailed(
                "empty output lane".to_string(),
            ));
        }

        let mut outputs = Vec::with_capacity(tx_outputs_bytes.len());
        for bytes in tx_outputs_bytes {
            if lane_is_empty(bytes) {
                return Err(TxAssemblerError::AssemblyFailed(
                    "empty output lane".to_string(),
                ));
            }
            outputs.push(decode_tx_output(bytes)?);
        }
        Ok(outputs)
    }

    fn build_inputs(&self, inputs: &[AsmInputWire]) -> TxAssemblerResult<Vec<TxInputWire>> {
        let mut seen = HashSet::new();
        let mut out = Vec::with_capacity(inputs.len());
        for input in inputs {
            let tx_input = input.input_ref()?;
            let key = format!("{}:{}", tx_input.asset_id_hex, tx_input.serial_id);
            if !seen.insert(key) {
                return Err(TxAssemblerError::AssemblyFailed(
                    "duplicate tx input ref".to_string(),
                ));
            }
            out.push(tx_input);
        }
        Ok(out)
    }

    fn build_output_assets(
        &self,
        outputs: &[TxOutputWire],
    ) -> TxAssemblerResult<(Vec<Asset>, usize, u64)> {
        let mut seen_keys = HashSet::new();
        let mut seen_nonces = HashSet::new();
        let mut fee_count = 0usize;
        let mut fee_sum = 0u64;
        let mut output_assets = Vec::with_capacity(outputs.len());

        for output in outputs {
            let asset = output.asset_wire.clone().to_asset().map_err(|err| {
                TxAssemblerError::AssemblyFailed(format!("output decode failed: {err}"))
            })?;

            let out_key = hex::encode(asset.asset_id());
            if !seen_keys.insert(out_key) {
                return Err(TxAssemblerError::AssemblyFailed(
                    "duplicate tx output state_key".to_string(),
                ));
            }
            if !seen_nonces.insert(asset.nonce) {
                return Err(TxAssemblerError::AssemblyFailed(
                    "duplicate transaction output nonce".to_string(),
                ));
            }

            if output.role == TxOutRole::Fee {
                fee_count = fee_count.checked_add(1).ok_or_else(|| {
                    TxAssemblerError::AssemblyFailed("fee output count overflow".to_string())
                })?;
                if asset.definition.class != z00z_core::assets::AssetClass::Coin {
                    return Err(TxAssemblerError::AssemblyFailed(
                        "fee outputs must use coin class".to_string(),
                    ));
                }
                fee_sum = fee_sum.checked_add(asset.amount).ok_or_else(|| {
                    TxAssemblerError::AssemblyFailed("fee output amount overflow".to_string())
                })?;
            }

            output_assets.push(asset);
        }

        Ok((output_assets, fee_count, fee_sum))
    }

    fn verify_resolved_balance(
        &self,
        inputs: &[AsmInputWire],
        outputs: &[Asset],
    ) -> TxAssemblerResult<()> {
        let mut input_commitments = Vec::with_capacity(inputs.len());
        let mut output_commitments = Vec::with_capacity(outputs.len());

        for input in inputs {
            input_commitments.push(input.commitment()?);
        }
        for output in outputs {
            output_commitments.push(output.commitment.clone());
        }

        if verify_tx_balance(&input_commitments, &output_commitments) {
            return Ok(());
        }

        Err(TxAssemblerError::InvalidCommitment(
            "resolved input and output commitments are not balanced".to_string(),
        ))
    }

    /// Delegate commitment-balance check to the balance module.
    pub fn check_commitment_balance(
        &self,
        inputs: &[Z00ZCommitment],
        outputs: &[Z00ZCommitment],
    ) -> TxAssemblerResult<()> {
        if verify_tx_balance(inputs, outputs) {
            return Ok(());
        }

        Err(TxAssemblerError::InvalidCommitment(
            "commitment sums are not balanced".to_string(),
        ))
    }

    /// Delegate commitment-balance check while treating the extra commitment as metadata.
    pub fn check_commitment_balance_meta(
        &self,
        inputs: &[Z00ZCommitment],
        outputs: &[Z00ZCommitment],
        meta: &Z00ZCommitment,
    ) -> TxAssemblerResult<()> {
        match verify_tx_balance_meta(inputs, outputs, meta) {
            Ok(()) => Ok(()),
            Err(TxBalErr::MetaMismatch) => Err(TxAssemblerError::InvalidCommitment(
                "metadata commitment must stay zero-valued".to_string(),
            )),
            Err(TxBalErr::CommitMismatch) => Err(TxAssemblerError::InvalidCommitment(
                "commitment sums are not balanced".to_string(),
            )),
        }
    }

    /// Delegate fee calculation to a fee estimator.
    pub fn estimate_fee<E: FeeEstimator>(
        &self,
        estimator: &E,
        tx_bytes: &[u8],
    ) -> TxAssemblerResult<FeeEstimate> {
        estimator
            .estimate(tx_bytes)
            .map_err(|err| TxAssemblerError::AssemblyFailed(format!("fee estimate failed: {err}")))
    }

    /// Calculate canonical declared fee for tx wire construction.
    pub fn calculate_fee_for_wires(
        &self,
        inputs: usize,
        outputs: &[AssetWire],
    ) -> TxAssemblerResult<u64> {
        super::fee_estimator::calculate_fee_for_wires(inputs, outputs)
            .map_err(|err| TxAssemblerError::AssemblyFailed(format!("fee calc failed: {err}")))
    }

    /// Delegate tx package verification to verifier pipeline.
    pub fn verify_with<V: TxVerifier>(
        &self,
        verifier: &V,
        tx_bytes: &[u8],
    ) -> TxAssemblerResult<()> {
        let result = verifier.verify(tx_bytes).map_err(|err| {
            TxAssemblerError::AssemblyFailed(format!("verification pipeline failed: {err}"))
        })?;
        if result.valid {
            return Ok(());
        }

        Err(TxAssemblerError::AssemblyFailed(format!(
            "verification failed: {}",
            result.errors.join("; ")
        )))
    }
}

impl Default for TxAssemblerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl TxAssembler for TxAssemblerImpl {
    fn assemble(&self, params: TxAssemblyParams) -> TxAssemblerResult<Vec<u8>> {
        if params.chain_id == 0
            || params.chain_type.trim().is_empty()
            || params.chain_name.trim().is_empty()
        {
            return Err(TxAssemblerError::AssemblyFailed(
                "tx package chain metadata is incomplete".to_string(),
            ));
        }

        let resolved_inputs = self.decode_inputs(&params.inputs_bytes)?;
        let tx_outputs = self.decode_outputs(&params.tx_outputs_bytes)?;

        let input_total = self.sum_inputs(&params.inputs_bytes)?;
        let output_total = self.sum_tx_outputs(&params.tx_outputs_bytes)?;
        if input_total != output_total {
            return Err(TxAssemblerError::InvalidCommitment(
                "resolved input value total does not equal output value total".to_string(),
            ));
        }

        let input_refs = self.build_inputs(&resolved_inputs)?;
        let (output_assets, fee_count, fee_sum) = self.build_output_assets(&tx_outputs)?;
        self.verify_resolved_balance(&resolved_inputs, &output_assets)?;

        if params.fee == 0 {
            if fee_count != 0 {
                return Err(TxAssemblerError::AssemblyFailed(
                    "fee outputs are forbidden when declared fee is zero".to_string(),
                ));
            }
        } else if fee_count == 0 {
            return Err(TxAssemblerError::AssemblyFailed(
                "fee outputs are required when declared fee is positive".to_string(),
            ));
        }
        if params.fee != fee_sum {
            return Err(TxAssemblerError::AssemblyFailed(
                "declared fee must equal sum of fee outputs".to_string(),
            ));
        }

        let tx = TxWire {
            tx_type: REGULAR_TX_TYPE.to_string(),
            inputs: input_refs,
            outputs: tx_outputs,
            fee: params.fee,
            nonce: 0,
            context: TxContextWire::default(),
            proof: TxProofWire::default(),
            auth: TxAuthWire::default(),
        };

        let tx_digest = build_tx_package_digest(
            TX_PACKAGE_KIND,
            REGULAR_TX_PACKAGE_TYPE,
            1,
            params.chain_id,
            &params.chain_type,
            &params.chain_name,
            &tx,
        )
        .map_err(|err| TxAssemblerError::AssemblyFailed(format!("digest build failed: {err}")))?;

        let package = TxPackage {
            kind: TX_PACKAGE_KIND.to_string(),
            package_type: REGULAR_TX_PACKAGE_TYPE.to_string(),
            version: 1,
            chain_id: params.chain_id,
            chain_type: params.chain_type,
            chain_name: params.chain_name,
            tx,
            tx_digest_hex: tx_digest,
            status: "prepared".to_string(),
        };

        let bytes = JsonCodec.serialize(&package).map_err(|err| {
            TxAssemblerError::AssemblyFailed(format!("package serialize failed: {err}"))
        })?;

        let report = TxVerifierImpl::new().verify(&bytes).map_err(|err| {
            TxAssemblerError::AssemblyFailed(format!("local verification failed: {err}"))
        })?;
        if !report.valid {
            return Err(TxAssemblerError::AssemblyFailed(format!(
                "local verification failed: {}",
                report.errors.join("; ")
            )));
        }

        Ok(bytes)
    }

    fn sum_inputs(&self, inputs_bytes: &[Vec<u8>]) -> TxAssemblerResult<u64> {
        let inputs = self.decode_inputs(inputs_bytes)?;
        let mut total = 0u64;
        for input in inputs {
            total = total.checked_add(input.value).ok_or_else(|| {
                TxAssemblerError::AssemblyFailed("resolved input value overflow".to_string())
            })?;
            let _ = input.commitment()?;
        }
        Ok(total)
    }

    fn sum_tx_outputs(&self, tx_outputs_bytes: &[Vec<u8>]) -> TxAssemblerResult<u64> {
        let outputs = self.decode_outputs(tx_outputs_bytes)?;
        let mut total = 0u64;
        for output in outputs {
            let asset = output.asset_wire.clone().to_asset().map_err(|err| {
                TxAssemblerError::AssemblyFailed(format!("output decode failed: {err}"))
            })?;
            total = total.checked_add(asset.amount).ok_or_else(|| {
                TxAssemblerError::AssemblyFailed("output value overflow".to_string())
            })?;
        }
        Ok(total)
    }

    fn verify_balance(&self, tx_bytes: &[u8]) -> TxAssemblerResult<()> {
        let report = verify_full_tx_package(tx_bytes).map_err(|err| {
            TxAssemblerError::AssemblyFailed(format!("full package verification failed: {err}"))
        })?;

        if report.valid {
            return Ok(());
        }

        Err(TxAssemblerError::AssemblyFailed(format!(
            "full package verification failed: {}",
            report.errors.join("; ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{
        fee_estimator::{FeeEstimate, FeeEstimatorResult},
        tx_verifier::{TxVerifierResult, VerificationResult},
    };
    use z00z_core::assets::AssetPkgWire;
    use z00z_core::{assets::AssetClass, genesis::asset_std::asset_from_dev_class};
    use z00z_utils::codec::{json, Codec, JsonCodec};
    use z00z_utils::rng::SystemRngProvider;

    fn scalar(seed: u64) -> z00z_crypto::Z00ZScalar {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        z00z_crypto::Z00ZScalar::try_from_bytes(bytes).expect("valid scalar")
    }

    fn resolved_input_bytes(
        asset_id: [u8; 32],
        serial_id: u32,
        value: u64,
        opening_seed: u64,
    ) -> Vec<u8> {
        let opening = scalar(opening_seed);
        let commitment = z00z_crypto::create_commitment(value, &opening).expect("commitment");

        JsonCodec
            .serialize(&json!({
            "asset_id_hex": hex::encode(asset_id),
            "serial_id": serial_id,
            "value": value,
            "opening_hex": hex::encode(opening.to_bytes()),
            "commitment_hex": hex::encode(commitment.as_bytes()),
            }))
            .expect("resolved input bytes")
    }

    fn output_wire_bytes(asset: &z00z_core::Asset, role: TxOutRole) -> Vec<u8> {
        let wire = TxOutputWire {
            role,
            asset_wire: AssetPkgWire::from_asset(asset),
        };
        JsonCodec.serialize(&wire).expect("output wire bytes")
    }

    fn valid_assembly_case() -> (TxAssemblyParams, u64) {
        let assembler = TxAssemblerImpl::new();
        let recipient_amount = 800u64;
        let recipient_blind = scalar(31);
        let fee_blind = scalar(47);
        let input_blind = &recipient_blind + &fee_blind;

        let recipient_template = asset_from_dev_class(AssetClass::Coin, 7, recipient_amount)
            .expect("recipient template");
        let definition = recipient_template.definition.clone();

        let mut rng = SystemRngProvider.rng();
        let recipient = z00z_core::Asset::new(
            definition.clone(),
            7,
            recipient_amount,
            &recipient_blind,
            [0x21; 32],
            &mut rng,
        )
        .expect("recipient asset");

        let fee_probe =
            z00z_core::Asset::new(definition.clone(), 9, 1, &fee_blind, [0x22; 32], &mut rng)
                .expect("fee probe");

        let fee = assembler
            .calculate_fee_for_wires(
                1,
                &[
                    z00z_core::assets::AssetWire::from_asset(&recipient),
                    z00z_core::assets::AssetWire::from_asset(&fee_probe),
                ],
            )
            .expect("fee");

        let fee_asset = z00z_core::Asset::new(definition, 9, fee, &fee_blind, [0x22; 32], &mut rng)
            .expect("fee asset");

        let input_total = recipient_amount + fee;
        let input_commitment =
            z00z_crypto::create_commitment(input_total, &input_blind).expect("input commitment");
        let output_commitment = &recipient.commitment + &fee_asset.commitment;
        assert_eq!(
            input_commitment.as_bytes(),
            output_commitment.as_bytes(),
            "fixture commitments must balance before wire encoding"
        );

        let params = TxAssemblyParams {
            inputs_bytes: vec![resolved_input_bytes([0x11; 32], 1, input_total, 31 + 47)],
            tx_outputs_bytes: vec![
                output_wire_bytes(&recipient, TxOutRole::Recipient),
                output_wire_bytes(&fee_asset, TxOutRole::Fee),
            ],
            fee,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: "z00z-devnet-1".to_string(),
        };

        (params, input_total)
    }

    struct TestFeeEstimator;

    impl FeeEstimator for TestFeeEstimator {
        fn estimate(&self, _tx_bytes: &[u8]) -> FeeEstimatorResult<FeeEstimate> {
            Ok(FeeEstimate {
                low: 1,
                medium: 2,
                high: 3,
            })
        }

        fn estimate_by_size(&self, _size_bytes: usize) -> FeeEstimatorResult<FeeEstimate> {
            self.estimate(&[])
        }

        fn get_fee_per_byte(&self) -> FeeEstimatorResult<u64> {
            Ok(1)
        }

        fn get_minimum_fee(&self) -> FeeEstimatorResult<u64> {
            Ok(1)
        }

        fn update_rates(&mut self) -> FeeEstimatorResult<()> {
            Ok(())
        }
    }

    struct TestVerifier;

    impl TxVerifier for TestVerifier {
        fn verify(&self, _tx_bytes: &[u8]) -> TxVerifierResult<VerificationResult> {
            Ok(VerificationResult {
                valid: true,
                errors: Vec::new(),
            })
        }

        fn verify_signatures(&self, _tx_bytes: &[u8]) -> TxVerifierResult<bool> {
            Ok(true)
        }

        fn verify_range_proofs(&self, _tx_bytes: &[u8]) -> TxVerifierResult<bool> {
            Ok(true)
        }

        fn verify_balance(&self, _tx_bytes: &[u8]) -> TxVerifierResult<bool> {
            Ok(true)
        }

        fn verify_structure(&self, _tx_bytes: &[u8]) -> TxVerifierResult<bool> {
            Ok(true)
        }
    }

    #[test]
    fn test_new_creates_assembler() {
        let assembler = TxAssemblerImpl::new();
        assert!(format!("{:?}", assembler).contains("TxAssemblerImpl"));
    }

    #[test]
    fn test_assemble_returns_error() {
        let assembler = TxAssemblerImpl::new();
        let params = TxAssemblyParams {
            inputs_bytes: vec![],
            tx_outputs_bytes: vec![],
            fee: 0,
            chain_id: 3,
            chain_type: "devnet".to_string(),
            chain_name: "z00z-devnet-1".to_string(),
        };
        assert!(assembler.assemble(params).is_err());
    }

    #[test]
    fn test_assemble_rejects_chain_meta() {
        let assembler = TxAssemblerImpl::new();
        let mut params = valid_assembly_case().0;
        params.chain_type = "   ".to_string();
        params.chain_name = "\t".to_string();

        assert!(assembler.assemble(params).is_err());
    }

    #[test]
    fn test_assemble_builds_package() {
        let assembler = TxAssemblerImpl::new();
        let (params, input_total) = valid_assembly_case();
        let bytes = assembler
            .assemble(params.clone())
            .expect("assembled package");
        let package: TxPackage = JsonCodec.deserialize(&bytes).expect("package");

        assert_eq!(package.kind, TX_PACKAGE_KIND);
        assert_eq!(package.package_type, REGULAR_TX_PACKAGE_TYPE);
        assert_eq!(package.tx.tx_type, REGULAR_TX_TYPE);
        assert_eq!(package.status, "prepared");
        assert_eq!(package.tx.inputs.len(), 1);
        assert_eq!(package.tx.outputs.len(), 2);
        assert_eq!(package.tx.fee, params.fee);
        assert!(TxVerifierImpl::new().verify(&bytes).expect("verify").valid);
        assert_eq!(
            input_total,
            assembler.sum_inputs(&params.inputs_bytes).expect("sum")
        );
        assert_eq!(
            input_total,
            assembler
                .sum_tx_outputs(&params.tx_outputs_bytes)
                .expect("sum")
        );
    }

    #[test]
    fn test_check_commitment_balance_ok() {
        let assembler = TxAssemblerImpl::new();
        let blind_in = scalar(11);
        let blind_out = scalar(7);
        let blind_change = &blind_in - &blind_out;

        let input = z00z_crypto::create_commitment(1000, &blind_in).expect("input commitment");
        let pay = z00z_crypto::create_commitment(800, &blind_out).expect("pay commitment");
        let change = z00z_crypto::create_commitment(200, &blind_change).expect("change commitment");
        let outputs = &pay + &change;

        let result = assembler.check_commitment_balance(&[input], &[outputs]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_commitment_meta_ok() {
        let assembler = TxAssemblerImpl::new();

        let in_a_v = 70u64;
        let in_b_v = 30u64;
        let out_a_v = 60u64;
        let out_b_v = 35u64;
        let fee_v = 5u64;

        let in_a_r = scalar(11);
        let in_b_r = scalar(13);
        let out_a_r = scalar(17);
        let out_b_r = scalar(19);
        let fee_r = &(&(&in_a_r + &in_b_r) - &out_a_r) - &out_b_r;

        let in_a = z00z_crypto::create_commitment(in_a_v, &in_a_r).expect("in a");
        let in_b = z00z_crypto::create_commitment(in_b_v, &in_b_r).expect("in b");
        let out_a = z00z_crypto::create_commitment(out_a_v, &out_a_r).expect("out a");
        let out_b = z00z_crypto::create_commitment(out_b_v, &out_b_r).expect("out b");
        let fee = z00z_crypto::create_commitment(fee_v, &fee_r).expect("fee");
        let meta = &in_a - &in_a;

        let result =
            assembler.check_commitment_balance_meta(&[in_a, in_b], &[out_a, out_b, fee], &meta);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_commitment_meta_error() {
        let assembler = TxAssemblerImpl::new();

        let in_c = z00z_crypto::create_commitment(10, &scalar(31)).expect("in");
        let out_c = z00z_crypto::create_commitment(10, &scalar(31)).expect("out");
        let bad_meta = z00z_crypto::create_commitment(1, &scalar(47)).expect("meta");

        let result = assembler.check_commitment_balance_meta(&[in_c], &[out_c], &bad_meta);
        assert!(matches!(
            result,
            Err(TxAssemblerError::InvalidCommitment(_))
        ));
    }

    #[test]
    fn test_estimate_fee_delegates() {
        let assembler = TxAssemblerImpl::new();
        let est = TestFeeEstimator;
        let fee = assembler.estimate_fee(&est, b"{}").expect("fee estimate");
        assert_eq!(fee.medium, 2);
    }

    #[test]
    fn test_verify_with_delegates() {
        let assembler = TxAssemblerImpl::new();
        let verifier = TestVerifier;
        let result = assembler.verify_with(&verifier, b"{}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sum_inputs_resolved_wire() {
        let assembler = TxAssemblerImpl::new();
        let inputs = vec![resolved_input_bytes([0x44; 32], 7, 1_000, 91)];

        assert_eq!(assembler.sum_inputs(&inputs).expect("sum"), 1_000);
    }

    #[test]
    fn test_inputs_rejects_public_wire() {
        let assembler = TxAssemblerImpl::new();
        let wire = TxInputWire {
            asset_id_hex: hex::encode([0x11; 32]),
            serial_id: 1,
        };
        let bytes = vec![JsonCodec.serialize(&wire).expect("public wire")];

        assert!(assembler.sum_inputs(&bytes).is_err());
    }

    #[test]
    fn test_sum_uses_visible_vals() {
        let assembler = TxAssemblerImpl::new();
        let (params, input_total) = valid_assembly_case();

        assert_eq!(
            assembler
                .sum_tx_outputs(&params.tx_outputs_bytes)
                .expect("sum"),
            input_total
        );
    }

    #[test]
    fn test_assemble_rejects_plain_mismatch() {
        let assembler = TxAssemblerImpl::new();
        let (mut params, _) = valid_assembly_case();
        let mut output: TxOutputWire = JsonCodec
            .deserialize(&params.tx_outputs_bytes[0])
            .expect("recipient output");
        let asset = output
            .asset_wire
            .clone()
            .to_asset()
            .expect("recipient asset");
        let mut rng = SystemRngProvider.rng();
        let replacement = z00z_core::Asset::new(
            asset.definition.clone(),
            asset.serial_id,
            asset.amount.saturating_add(1),
            &scalar(31),
            asset.nonce,
            &mut rng,
        )
        .expect("replacement asset");
        output.asset_wire = AssetPkgWire::from_asset(&replacement);
        params.tx_outputs_bytes[0] = JsonCodec.serialize(&output).expect("updated output");

        let err = assembler
            .assemble(params)
            .expect_err("plaintext value mismatch must reject");
        assert!(
            err.to_string()
                .contains("resolved input value total does not equal output value total"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_assemble_rejects_commit_mismatch() {
        let assembler = TxAssemblerImpl::new();
        let (mut params, _) = valid_assembly_case();
        let mut output: TxOutputWire = JsonCodec
            .deserialize(&params.tx_outputs_bytes[0])
            .expect("recipient output");
        let asset = output
            .asset_wire
            .clone()
            .to_asset()
            .expect("recipient asset");
        let mut rng = SystemRngProvider.rng();
        let replacement = z00z_core::Asset::new(
            asset.definition.clone(),
            asset.serial_id,
            asset.amount,
            &scalar(99),
            asset.nonce,
            &mut rng,
        )
        .expect("replacement asset");
        output.asset_wire = AssetPkgWire::from_asset(&replacement);
        params.tx_outputs_bytes[0] = JsonCodec.serialize(&output).expect("updated output");

        let err = assembler
            .assemble(params)
            .expect_err("commitment mismatch must reject");
        assert!(
            err.to_string()
                .contains("resolved input and output commitments are not balanced"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_verify_balance_routes_full() {
        let assembler = TxAssemblerImpl::new();
        let bytes = assembler
            .assemble(valid_assembly_case().0)
            .expect("assembled package");

        let err = assembler
            .verify_balance(&bytes)
            .expect_err("missing spend proof");
        assert!(
            err.to_string().contains("full package verification failed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_calc_fee_wires_ok() {
        let assembler = TxAssemblerImpl::new();
        let asset = asset_from_dev_class(AssetClass::Coin, 1, 100).expect("asset");
        let wire = z00z_core::assets::AssetWire::from_asset(&asset);
        let fee = assembler.calculate_fee_for_wires(1, &[wire]).expect("fee");
        assert!(fee > 0);
    }
}
