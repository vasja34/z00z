//! Canonical claim statement contract and authority-signature helpers.

use thiserror::Error;
use z00z_utils::codec::{BincodeCodec, Codec};

use crate::{
    domains::ClaimStmtDomain, expert::keys::RistrettoSecretKey, hash::DomainHasher256,
    sign_kernel_signature, verify_kernel_signature, KernelSignature, Z00ZRistrettoPoint,
    Z00ZScalar,
};

pub const CLAIM_ROOT_VERSION: u8 = 2;

const CLAIM_TAG: &[u8; 4] = b"CLM2";
const HASH_LEN: usize = 32;
const BASE_LEN: usize = 18 + (HASH_LEN * 9);

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ClaimProofVer(u8);

impl ClaimProofVer {
    pub const V1: Self = Self(1);
    pub const V2: Self = Self(2);

    pub fn new(value: u8) -> Result<Self, ClaimError> {
        if value == 0 {
            return Err(ClaimError::BadProofVer);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ClaimError {
    #[error("claim contract decode failed")]
    DecodeFail,
    #[error("claim contract root version must be non-zero")]
    BadRootVersion,
    #[error("claim contract proof version must be non-zero")]
    BadProofVer,
    #[error("claim contract tx version must be non-zero")]
    BadTxVer,
    #[error("claim contract output leaf list must be non-empty")]
    NoOutputs,
    #[error("claim contract proof bytes must be non-empty")]
    ProofEmpty,
    #[error("claim contract authority signature is invalid")]
    SigInvalid,
    #[error("claim contract root version mismatch")]
    RootVersionMismatch,
    #[error("claim contract proof version mismatch")]
    ProofVerMix,
    #[error("claim contract source root mismatch")]
    SourceRootMix,
    #[error("claim contract backend signing failed: {0}")]
    SignBackend(String),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimStmt {
    /// This canonical statement-signing surface is already real; the unresolved
    /// fix set is the live anchored authority lifecycle above it, not the
    /// statement binding itself. Preserve current statement binding and move the remaining
    /// genesis-membership work to authoritative membership proofs instead of helper continuity.
    pub chain_id: u32,
    pub root_version: u8,
    pub proof_ver: ClaimProofVer,
    /// Fill from `crate::protocol::stealth_bind::range_ctx_hash` once claim and tx paths migrate.
    pub tx_ver: u32,
    pub range_ctx_hash: [u8; 32],
    pub claim_id: [u8; 32],
    pub claim_source_asset_id: [u8; 32],
    pub claim_source_commitment: [u8; 32],
    pub source_root: [u8; 32],
    pub claim_scope_hash: [u8; 32],
    pub recipient_binding: [u8; 32],
    pub nullifier: [u8; 32],
    pub owner_bind_digest: [u8; 32],
    pub output_leaf_hashes: Vec<[u8; 32]>,
}

impl ClaimStmt {
    pub fn to_bytes(&self) -> Result<Vec<u8>, ClaimError> {
        self.chk_shape()?;

        let mut out = Vec::with_capacity(BASE_LEN + (self.output_leaf_hashes.len() * HASH_LEN));
        out.extend_from_slice(CLAIM_TAG);
        out.extend_from_slice(&self.chain_id.to_le_bytes());
        out.push(self.root_version);
        out.push(self.proof_ver.as_u8());
        out.extend_from_slice(&self.tx_ver.to_le_bytes());
        push_arr(&mut out, &self.range_ctx_hash);
        push_arr(&mut out, &self.claim_id);
        push_arr(&mut out, &self.claim_source_asset_id);
        push_arr(&mut out, &self.claim_source_commitment);
        push_arr(&mut out, &self.source_root);
        push_arr(&mut out, &self.claim_scope_hash);
        push_arr(&mut out, &self.recipient_binding);
        push_arr(&mut out, &self.nullifier);
        push_arr(&mut out, &self.owner_bind_digest);
        out.extend_from_slice(&(self.output_leaf_hashes.len() as u32).to_le_bytes());
        for leaf_hash in &self.output_leaf_hashes {
            push_arr(&mut out, leaf_hash);
        }

        Ok(out)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ClaimError> {
        if bytes.len() < BASE_LEN {
            return Err(ClaimError::DecodeFail);
        }

        let mut idx = 0usize;
        let tag = read_tag(bytes, &mut idx)?;
        if &tag != CLAIM_TAG {
            return Err(ClaimError::DecodeFail);
        }

        let chain_id = read_u32(bytes, &mut idx)?;
        let root_version = read_u8(bytes, &mut idx)?;
        if root_version == 0 {
            return Err(ClaimError::BadRootVersion);
        }
        let proof_ver = ClaimProofVer::new(read_u8(bytes, &mut idx)?)?;
        let tx_ver = read_u32(bytes, &mut idx)?;
        let range_ctx_hash = read_arr(bytes, &mut idx)?;
        let claim_id = read_arr(bytes, &mut idx)?;
        let claim_source_asset_id = read_arr(bytes, &mut idx)?;
        let claim_source_commitment = read_arr(bytes, &mut idx)?;
        let source_root = read_arr(bytes, &mut idx)?;
        let claim_scope_hash = read_arr(bytes, &mut idx)?;
        let recipient_binding = read_arr(bytes, &mut idx)?;
        let nullifier = read_arr(bytes, &mut idx)?;
        let owner_bind_digest = read_arr(bytes, &mut idx)?;
        let leaf_count = read_u32(bytes, &mut idx)? as usize;
        let output_leaf_hashes = read_hashes(bytes, &mut idx, leaf_count)?;

        if idx != bytes.len() {
            return Err(ClaimError::DecodeFail);
        }

        let stmt = Self {
            chain_id,
            root_version,
            proof_ver,
            tx_ver,
            range_ctx_hash,
            claim_id,
            claim_source_asset_id,
            claim_source_commitment,
            source_root,
            claim_scope_hash,
            recipient_binding,
            nullifier,
            owner_bind_digest,
            output_leaf_hashes,
        };
        stmt.chk_shape()?;
        Ok(stmt)
    }

    pub fn chk_source(&self, proof: &ClaimSourceProof) -> Result<(), ClaimError> {
        self.chk_shape()?;
        if self.root_version != proof.root_version() {
            return Err(ClaimError::RootVersionMismatch);
        }
        if self.proof_ver != proof.proof_ver() {
            return Err(ClaimError::ProofVerMix);
        }
        if self.source_root != proof.source_root() {
            return Err(ClaimError::SourceRootMix);
        }
        Ok(())
    }

    fn chk_shape(&self) -> Result<(), ClaimError> {
        if self.root_version == 0 {
            return Err(ClaimError::BadRootVersion);
        }
        if self.proof_ver.as_u8() == 0 {
            return Err(ClaimError::BadProofVer);
        }
        if self.tx_ver == 0 {
            return Err(ClaimError::BadTxVer);
        }
        if self.output_leaf_hashes.is_empty() {
            return Err(ClaimError::NoOutputs);
        }
        Ok(())
    }

    #[must_use]
    pub const fn source_root(&self) -> [u8; 32] {
        self.source_root
    }

    #[must_use]
    pub const fn claim_scope_hash(&self) -> [u8; 32] {
        self.claim_scope_hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimAuthoritySig {
    /// This signature primitive is already live on the canonical statement-
    /// signing surface, but it still needs a live anchored authority lifecycle
    /// instead of simulator-fixed authority roots.
    auth_pk: Z00ZRistrettoPoint,
    auth_sig: KernelSignature,
}

impl ClaimAuthoritySig {
    pub fn sign<R>(
        stmt: &ClaimStmt,
        auth_sk: &RistrettoSecretKey,
        rng: &mut R,
    ) -> Result<Self, ClaimError>
    where
        R: rand::CryptoRng + rand::RngCore,
    {
        let auth_scalar = Z00ZScalar::from_ristretto_secret_key(auth_sk.clone());
        let stmt_hash = claim_stmt_hash(stmt)?;
        let auth_sig = sign_kernel_signature(&auth_scalar, stmt_hash, rng)
            .map_err(|err| ClaimError::SignBackend(err.to_string()))?;
        let auth_pk = Z00ZRistrettoPoint::from_secret_key(&auth_scalar);

        Ok(Self { auth_pk, auth_sig })
    }

    pub fn verify(&self, stmt: &ClaimStmt) -> Result<(), ClaimError> {
        let stmt_hash = claim_stmt_hash(stmt)?;
        if verify_kernel_signature(&self.auth_sig, &self.auth_pk, stmt_hash) {
            return Ok(());
        }
        Err(ClaimError::SigInvalid)
    }

    pub fn verify_with_pk(
        &self,
        stmt: &ClaimStmt,
        auth_pk: &Z00ZRistrettoPoint,
    ) -> Result<(), ClaimError> {
        if &self.auth_pk != auth_pk {
            return Err(ClaimError::SigInvalid);
        }
        let stmt_hash = claim_stmt_hash(stmt)?;
        if verify_kernel_signature(&self.auth_sig, auth_pk, stmt_hash) {
            return Ok(());
        }
        Err(ClaimError::SigInvalid)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ClaimError> {
        BincodeCodec
            .serialize(self)
            .map_err(|_| ClaimError::DecodeFail)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ClaimError> {
        BincodeCodec
            .deserialize(bytes)
            .map_err(|_| ClaimError::DecodeFail)
    }

    #[must_use]
    pub fn auth_pk(&self) -> &Z00ZRistrettoPoint {
        &self.auth_pk
    }

    #[must_use]
    pub fn auth_sig(&self) -> &KernelSignature {
        &self.auth_sig
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClaimSourceProof {
    root_version: u8,
    source_root: [u8; 32],
    proof_ver: ClaimProofVer,
    proof_blob: Vec<u8>,
}

impl ClaimSourceProof {
    pub fn new(
        root_version: u8,
        source_root: [u8; 32],
        proof_ver: ClaimProofVer,
        proof_blob: Vec<u8>,
    ) -> Result<Self, ClaimError> {
        let proof = Self {
            root_version,
            source_root,
            proof_ver,
            proof_blob,
        };
        proof.chk_shape()?;
        Ok(proof)
    }

    fn chk_shape(&self) -> Result<(), ClaimError> {
        if self.root_version == 0 {
            return Err(ClaimError::BadRootVersion);
        }
        if self.proof_ver.as_u8() == 0 {
            return Err(ClaimError::BadProofVer);
        }
        if self.proof_blob.is_empty() {
            return Err(ClaimError::ProofEmpty);
        }
        Ok(())
    }

    #[must_use]
    pub const fn root_version(&self) -> u8 {
        self.root_version
    }

    #[must_use]
    pub const fn source_root(&self) -> [u8; 32] {
        self.source_root
    }

    #[must_use]
    pub const fn proof_ver(&self) -> ClaimProofVer {
        self.proof_ver
    }

    #[must_use]
    pub fn proof_blob(&self) -> &[u8] {
        &self.proof_blob
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ClaimError> {
        self.chk_shape()?;
        BincodeCodec
            .serialize(self)
            .map_err(|_| ClaimError::DecodeFail)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ClaimError> {
        let proof = BincodeCodec
            .deserialize::<Self>(bytes)
            .map_err(|_| ClaimError::DecodeFail)?;
        proof.chk_shape()?;
        Ok(proof)
    }
}

pub fn claim_stmt_hash(stmt: &ClaimStmt) -> Result<[u8; 32], ClaimError> {
    let bytes = stmt.to_bytes()?;
    let hash = DomainHasher256::<ClaimStmtDomain>::new_with_label("claim_contract")
        .chain(&bytes)
        .finalize();
    let mut out = [0u8; HASH_LEN];
    out.copy_from_slice(hash.as_ref());
    Ok(out)
}

fn push_arr(out: &mut Vec<u8>, arr: &[u8; HASH_LEN]) {
    out.extend_from_slice(arr);
}

fn read_tag(bytes: &[u8], idx: &mut usize) -> Result<[u8; 4], ClaimError> {
    let end = idx.checked_add(4).ok_or(ClaimError::DecodeFail)?;
    let slice = bytes.get(*idx..end).ok_or(ClaimError::DecodeFail)?;
    let mut out = [0u8; 4];
    out.copy_from_slice(slice);
    *idx = end;
    Ok(out)
}

fn read_u8(bytes: &[u8], idx: &mut usize) -> Result<u8, ClaimError> {
    let end = idx.checked_add(1).ok_or(ClaimError::DecodeFail)?;
    let value = *bytes.get(*idx).ok_or(ClaimError::DecodeFail)?;
    *idx = end;
    Ok(value)
}

fn read_u32(bytes: &[u8], idx: &mut usize) -> Result<u32, ClaimError> {
    let end = idx.checked_add(4).ok_or(ClaimError::DecodeFail)?;
    let slice = bytes.get(*idx..end).ok_or(ClaimError::DecodeFail)?;
    let mut out = [0u8; 4];
    out.copy_from_slice(slice);
    *idx = end;
    Ok(u32::from_le_bytes(out))
}

fn read_arr(bytes: &[u8], idx: &mut usize) -> Result<[u8; HASH_LEN], ClaimError> {
    let end = idx.checked_add(HASH_LEN).ok_or(ClaimError::DecodeFail)?;
    let slice = bytes.get(*idx..end).ok_or(ClaimError::DecodeFail)?;
    let mut out = [0u8; HASH_LEN];
    out.copy_from_slice(slice);
    *idx = end;
    Ok(out)
}

fn read_hashes(
    bytes: &[u8],
    idx: &mut usize,
    count: usize,
) -> Result<Vec<[u8; HASH_LEN]>, ClaimError> {
    let remain = bytes
        .len()
        .checked_sub(*idx)
        .ok_or(ClaimError::DecodeFail)?;
    let need = count.checked_mul(HASH_LEN).ok_or(ClaimError::DecodeFail)?;
    if remain != need {
        return Err(ClaimError::DecodeFail);
    }

    let mut hashes = Vec::with_capacity(count);
    for _ in 0..count {
        hashes.push(read_arr(bytes, idx)?);
    }
    Ok(hashes)
}
