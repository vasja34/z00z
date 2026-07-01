//! Wire types, verifier, and crypto helpers for claim transactions.
#![forbid(unsafe_code)]
//! MUST NOT import std::fs, serde_json, serde_yaml, or SystemTime.

pub use super::claim_auth::claim_auth_pk;
pub use super::claim_auth::require_claim_auth_simulator_anchor;
#[cfg(any(test, doctest, feature = "claim-auth-sign"))]
pub use super::claim_auth::sign_claim_auth;
use super::claim_auth::{decode_claim_auth, OWNER_ATTEST_CTX};
pub use super::claim_errors::{ClaimTxError, ClaimTxVerifyReport, ClaimVerifyResult};
pub use super::claim_tx_digest::{
    build_claim_tx_digest, compute_claim_scope_hash, derive_output_nonce,
};
pub use super::claim_tx_wire::{
    ClaimAuthWire, ClaimContextWire, ClaimInputWire, ClaimOutputWire, ClaimProofWire,
    ClaimScopeKey, ClaimTxPackage, ClaimTxWire, CLAIM_PKG,
};
use super::claim_tx_wire::{CLAIM_SOURCE_PROOF_TAG, CLAIM_TX_PACKAGE_TYPE, CLAIM_TX_TYPE};
use super::{
    tx_output::output_range_ctx_hash, tx_verifier::TX_PACKAGE_KIND,
    witness_gate::asset_wire_to_leaf,
};
use crate::{
    claim::derive_nullifier,
    key::{sign_identity, verify_identity, ReceiverKeys},
    receiver::ReceiverCard,
};
use std::collections::HashSet;
use z00z_core::{assets::AssetPkgWire, AssetClass};
use z00z_crypto::expert::encoding::ByteArray;
use z00z_crypto::{
    claim::{ClaimProofVer, ClaimSourceProof, ClaimStmt, CLAIM_ROOT_VERSION},
    validation::validate_scalar_nonzero,
    KernelSignature as Z00ZSchnorrSignature, Z00ZRistrettoPoint, Z00ZScalar,
};
use z00z_storage::settlement::{
    chk_blob_settlement_inclusion_carried, proof_blob_item, ProofItem, SettlementStateRoot,
};
use z00z_utils::codec::{Codec, JsonCodec};

const ZERO_ROOT: [u8; 32] = [0u8; 32];
include!("claim_tx_statement.rs");

/// Stateless verifier contract for claim package bytes.
pub trait ClaimTxVerifier {
    /// Verify raw claim package bytes and return structured result.
    fn verify(&self, raw_bytes: &[u8]) -> ClaimVerifyResult;
}

#[inline]
fn is_lower_hex(s: &str) -> bool {
    s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[inline]
fn is_even_hex(s: &str) -> bool {
    s.len().is_multiple_of(2) && is_lower_hex(s)
}

#[inline]
fn is_hex64(s: &str) -> bool {
    s.len() == 64 && is_lower_hex(s)
}

fn decode_hex32(value: &str, label: &str) -> Result<[u8; 32], ClaimTxError> {
    let bytes = hex::decode(value)
        .map_err(|_| ClaimTxError::StructureMalformed(format!("{label} invalid hex")))?;
    bytes
        .try_into()
        .map_err(|_| ClaimTxError::StructureMalformed(format!("{label} invalid len")))
}

fn sig_to_hex(sig: &Z00ZSchnorrSignature) -> String {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(sig.get_public_nonce().as_bytes());
    bytes[32..].copy_from_slice(sig.get_signature().as_bytes());
    hex::encode(bytes)
}

fn decode_sig_hex(value: &str, label: &str) -> Result<Z00ZSchnorrSignature, ClaimTxError> {
    if value.is_empty() || !is_even_hex(value) {
        return Err(ClaimTxError::OwnerAttestInvalidHex(format!(
            "{label} invalid hex"
        )));
    }

    let bytes = hex::decode(value)
        .map_err(|_| ClaimTxError::OwnerAttestInvalidHex(format!("{label} invalid hex decode")))?;
    let bytes: [u8; 64] = bytes.try_into().map_err(|_| {
        ClaimTxError::OwnerAttestInvalidHex(format!("{label} invalid signature length"))
    })?;

    let nonce =
        Z00ZRistrettoPoint::try_from_bytes(bytes[..32].try_into().map_err(|_| {
            ClaimTxError::OwnerAttestInvalid(format!("{label} invalid nonce bytes"))
        })?)
        .map_err(|_| ClaimTxError::OwnerAttestInvalid(format!("{label} invalid nonce bytes")))?;
    let scalar =
        Z00ZScalar::try_from_bytes(bytes[32..].try_into().map_err(|_| {
            ClaimTxError::OwnerAttestInvalid(format!("{label} invalid scalar bytes"))
        })?)
        .map_err(|_| ClaimTxError::OwnerAttestInvalid(format!("{label} invalid scalar bytes")))?;
    validate_scalar_nonzero(&scalar)
        .map_err(|_| ClaimTxError::OwnerAttestInvalid(format!("{label} zero scalar")))?;

    Ok(Z00ZSchnorrSignature::new(
        nonce.reveal().clone(),
        scalar.reveal().clone(),
    ))
}

/// Build the context-bound recipient attestation payload for one portable claim output.
pub fn build_owner_attest_msg(
    chain_id: u32,
    claim_id_hex: &str,
    recipient_wallet_id: &str,
    recipient_owner_hex: &str,
    claim_scope_hash_hex: &str,
    output_index: u32,
    leaf: &AssetPkgWire,
) -> Result<Vec<u8>, ClaimTxError> {
    let claim_id = decode_hex32(claim_id_hex, "claim_id_hex")?;
    let owner = decode_hex32(recipient_owner_hex, "recipient_owner_hex")?;
    let scope = decode_hex32(claim_scope_hash_hex, "claim_scope_hash_hex")?;
    let leaf_bytes = JsonCodec.serialize(leaf).map_err(|e| {
        ClaimTxError::StructureMalformed(format!("owner attestation leaf encode failed: {e}"))
    })?;

    let wallet = recipient_wallet_id.as_bytes();
    let mut msg = Vec::with_capacity(64 + wallet.len() + leaf_bytes.len());
    msg.extend_from_slice(b"z00z.claim.owner_attest.msg.v1");
    msg.extend_from_slice(&chain_id.to_le_bytes());
    msg.extend_from_slice(&(wallet.len() as u32).to_le_bytes());
    msg.extend_from_slice(wallet);
    msg.extend_from_slice(&claim_id);
    msg.extend_from_slice(&owner);
    msg.extend_from_slice(&scope);
    msg.extend_from_slice(&output_index.to_le_bytes());
    msg.extend_from_slice(&(leaf_bytes.len() as u32).to_le_bytes());
    msg.extend_from_slice(&leaf_bytes);
    Ok(msg)
}

/// Sign one portable claim output binding with the recipient identity key.
pub fn sign_owner_attest(
    keys: &ReceiverKeys,
    chain_id: u32,
    tx: &ClaimTxWire,
    output_index: u32,
    leaf: &AssetPkgWire,
) -> Result<String, String> {
    let claim_input = tx
        .inputs
        .first()
        .ok_or_else(|| "claim tx requires one input for owner attestation".to_string())?;
    let owner = decode_hex32(&tx.context.recipient_owner_hex, "recipient_owner_hex")
        .map_err(|e| e.to_string())?;
    if owner != keys.owner_handle {
        return Err("recipient_owner_hex does not match recipient keys owner_handle".to_string());
    }

    let msg = build_owner_attest_msg(
        chain_id,
        &claim_input.claim_id_hex,
        &tx.context.recipient_wallet_id,
        &tx.context.recipient_owner_hex,
        &tx.context.claim_scope_hash_hex,
        output_index,
        leaf,
    )
    .map_err(|e| e.to_string())?;
    let sig = sign_identity(keys.reveal_identity_sk(), &msg, OWNER_ATTEST_CTX)
        .map_err(|e| format!("owner attestation sign failed: {e}"))?;
    Ok(sig_to_hex(&sig))
}

#[derive(Debug, Default)]
/// Default stateless implementation of `ClaimTxVerifier`.
pub struct ClaimTxVerifierImpl;
// Root seam map for closure-gate audits; implementations live in claim_tx_verify.rs.
// fn verify_claim_proof(
// fn verify_claim_authority(
// fn verify_owner_attest(
// fn verify_digest(
include!("claim_tx_verify.rs");

fn err_class(err: &ClaimTxError) -> &'static str {
    match err {
        ClaimTxError::ProofTypeInvalid(_)
        | ClaimTxError::ProofBlobInvalidHex(_)
        | ClaimTxError::ProofBlobDecode(_)
        | ClaimTxError::ProofVerify(_)
        | ClaimTxError::SourceRootZero(_)
        | ClaimTxError::SourceRootVersion(_)
        | ClaimTxError::SourceProofVer(_)
        | ClaimTxError::SourceProofMismatch(_)
        | ClaimTxError::SourceLeafCount(_) => "claim_proof_invalid",
        ClaimTxError::AuthoritySigDecode(_) | ClaimTxError::AuthoritySigInvalid(_) => {
            "claim_authority_invalid"
        }
        ClaimTxError::FeeNonZero(_) => "claim_fee_invalid",
        ClaimTxError::NullifierInvalidHex(_) | ClaimTxError::NullifierMismatch(_) => {
            "claim_nullifier_invalid"
        }
        ClaimTxError::OutputAmountZero(_)
        | ClaimTxError::OutputAssetClassInvalid(_)
        | ClaimTxError::OutputNonceInvalidHex(_)
        | ClaimTxError::OutputNonceIsZero(_)
        | ClaimTxError::OutputAssetIdInvalidHex(_)
        | ClaimTxError::OutputOwnerBindingInvalidHex(_)
        | ClaimTxError::OutputOwnerBindingMismatch(_)
        | ClaimTxError::OutputNonceMismatch(_)
        | ClaimTxError::DuplicateNonce(_)
        | ClaimTxError::DuplicateAssetId(_)
        | ClaimTxError::LeafRequired(_)
        | ClaimTxError::LeafInvalid(_)
        | ClaimTxError::LeafMismatch(_)
        | ClaimTxError::RecipientCardRequired(_)
        | ClaimTxError::RecipientCardInvalid(_)
        | ClaimTxError::RecipientCardMismatch(_)
        | ClaimTxError::OwnerAttestRequired(_)
        | ClaimTxError::OwnerAttestInvalidHex(_)
        | ClaimTxError::OwnerAttestInvalid(_)
        | ClaimTxError::OutputsEmpty => "claim_output_invalid",
        _ => "claim_structure_invalid",
    }
}

impl ClaimTxVerifier for ClaimTxVerifierImpl {
    fn verify(&self, raw_bytes: &[u8]) -> ClaimVerifyResult {
        let codec = JsonCodec;
        let pkg = match codec.deserialize::<ClaimTxPackage>(raw_bytes) {
            Ok(v) => v,
            Err(e) => return Self::fail_raw("claim_malformed_json", e.to_string(), None),
        };
        let mut report = ClaimTxVerifyReport::default();

        if let Err(e) = Self::verify_structure(&pkg) {
            return Self::fail(e, report);
        }

        if let Err(e) = Self::verify_scope(&pkg) {
            return Self::fail(e, report);
        }

        if let Err(e) = Self::verify_tx_fields(&pkg.tx) {
            return Self::fail(e, report);
        }

        if let Err(e) = Self::verify_nullifier(&pkg.tx, pkg.chain_id) {
            return Self::fail(e, report);
        }
        report.nullifier_checked = true;

        let recipient_card = match Self::verify_recipient_card(&pkg.tx, pkg.version == CLAIM_PKG) {
            Ok(card) => card,
            Err(err) => return Self::fail(err, report),
        };
        report.card_checked = true;

        let leaves = match Self::verify_portable_leaf(&pkg.tx, pkg.version == CLAIM_PKG) {
            Ok(leaves) => leaves,
            Err(err) => return Self::fail(err, report),
        };
        report.leaf_checked = true;

        let stmt = match Self::build_claim_stmt(&pkg, &leaves) {
            Ok(stmt) => stmt,
            Err(err) => return Self::fail(err, report),
        };

        if let Err(err) = Self::verify_claim_proof(&pkg.tx, &stmt, leaves[0]) {
            return Self::fail(err, report);
        }
        report.proof_checked = true;

        if let Err(err) = Self::verify_claim_authority(&pkg.tx, &stmt) {
            return Self::fail(err, report);
        }
        report.authority_checked = true;

        let recipient_card = match recipient_card.as_ref() {
            Some(card) => card,
            None => {
                return Self::fail(
                    ClaimTxError::RecipientCardRequired(
                        "claim package requires tx.context.recipient_card_hex".to_string(),
                    ),
                    report,
                );
            }
        };

        if let Err(err) = Self::verify_owner_attest(&pkg.tx, pkg.chain_id, recipient_card, &leaves)
        {
            return Self::fail(err, report);
        }
        report.owner_attest_checked = true;

        if let Err(err) = Self::verify_digest(&pkg) {
            return Self::fail(err, report);
        }
        report.digest_checked = true;

        ClaimVerifyResult {
            valid: true,
            reject_class: String::new(),
            errors: vec![],
            report: Some(report),
        }
    }
}

#[cfg(test)]
#[path = "test_claim_tx.rs"]
mod test_claim_tx;
