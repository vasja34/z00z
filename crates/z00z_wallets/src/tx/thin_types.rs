use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Thin snapshot schema version.
pub const THIN_SNAPSHOT_VERSION: u16 = 1;
/// Thin transaction wrapper schema version.
pub const THIN_TX_PACKAGE_VERSION: u16 = 1;

/// Canonical input/path identity kept in thin helper references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinAssetPathRef {
    /// Canonical `asset_id_hex` input identity.
    pub asset_id_hex: String,
    /// Canonical `serial_id` paired with the input asset id.
    pub serial_id: u32,
}

/// Signed helper snapshot validity context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinSnapshotContext {
    /// Chain identifier for every entry in the snapshot.
    pub chain_id: String,
    /// Explicit compatibility generation for stale-drift rejection.
    pub compatibility_generation: u64,
    /// Canonical checkpoint-facing root context.
    pub prev_root_hex: String,
    /// Optional checkpoint id when one exists outside the pre-state root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_id_hex: Option<String>,
    /// Snapshot issue time in Unix milliseconds.
    pub issued_at_ms: u64,
    /// Snapshot expiry time in Unix milliseconds.
    pub expires_at_ms: u64,
}

/// Helper-side signed-index entry for one canonical tx candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinIndexEntry {
    /// Stable helper-scoped entry id.
    pub entry_id_hex: String,
    /// Canonical `TxPackage.tx_digest_hex`.
    pub tx_hash_hex: String,
    /// Canonical `TxPackage.kind`.
    pub package_kind: String,
    /// Canonical `TxPackage.package_type`.
    pub package_type: String,
    /// Chain id copied from the bound package.
    pub chain_id: String,
    /// Canonical `prev_root_hex` from the spend proof.
    pub prev_root_hex: String,
    /// Digest over the bound public proof payload.
    pub proof_digest_hex: String,
    /// Canonical input/path identity family this entry expands against.
    pub input_refs: Vec<ThinAssetPathRef>,
    /// Canonical thick package bytes the helper rehydrates.
    pub tx_bytes: Vec<u8>,
}

/// Signed helper snapshot over one or more thin index entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinSnapshot {
    /// Snapshot wire version.
    pub snapshot_version: u16,
    /// Signer identity public key encoded as lowercase hex.
    pub signer_identity_hex: String,
    /// Digest over the unsigned snapshot body.
    pub snapshot_digest_hex: String,
    /// Signature over the unsigned snapshot body encoded as lowercase hex.
    pub signature_hex: String,
    /// Explicit checkpoint/generation validity context.
    pub context: ThinSnapshotContext,
    /// Signed helper entries.
    pub entries: Vec<ThinIndexEntry>,
}

/// Digest-pinned snapshot handle used by the wallet when constructing thin requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinSnapshotPin {
    /// Exact snapshot digest pinned by the wallet.
    pub snapshot_digest_hex: String,
    /// Chain id carried by the pinned snapshot.
    pub chain_id: String,
    /// Explicit compatibility generation carried by the pinned snapshot.
    pub compatibility_generation: u64,
    /// Canonical checkpoint-facing root bound by the pinned snapshot.
    pub prev_root_hex: String,
    /// Optional checkpoint id carried by the pinned snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_id_hex: Option<String>,
    /// Snapshot expiry copied from the signed snapshot.
    pub expires_at_ms: u64,
}

/// Thin wallet transport wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThinWalletTxPackage {
    /// Thin wrapper schema version.
    pub package_version: u16,
    /// Canonical chain id copied from the thick package.
    pub chain_id: String,
    /// Canonical `TxPackage.kind`.
    pub package_kind: String,
    /// Canonical `TxPackage.package_type`.
    pub package_type: String,
    /// Canonical package digest bound by the wallet.
    pub tx_hash_hex: String,
    /// Exact helper snapshot digest pinned by the wallet.
    pub snapshot_digest_hex: String,
    /// Expected compatibility generation.
    pub compatibility_generation: u64,
    /// Expected checkpoint-facing root context.
    pub prev_root_hex: String,
    /// Optional expected checkpoint id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_id_hex: Option<String>,
    /// Helper entry id to expand.
    pub snapshot_entry_id_hex: String,
    /// Expected canonical input/path family.
    pub input_refs: Vec<ThinAssetPathRef>,
    /// Digest over the unsigned thin wrapper fields.
    pub metadata_hash_hex: String,
}

/// Typed thin helper/index failure classes.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ThinIndexError {
    /// Thin wrapper version is unsupported.
    #[error("unsupported thin tx package version: {0}")]
    UnsupportedThinVersion(u16),
    /// Snapshot version is unsupported.
    #[error("unsupported thin snapshot version: {0}")]
    UnsupportedSnapshotVersion(u16),
    /// One required hex field is malformed.
    #[error("{field} must be lowercase {expected_len}-byte hex")]
    InvalidHex {
        /// Field name that failed canonical lowercase-hex validation.
        field: &'static str,
        /// Expected raw byte length before hex encoding.
        expected_len: usize,
    },
    /// Snapshot metadata is internally inconsistent.
    #[error("thin snapshot context mismatch for {field}: expected {expected}, got {actual}")]
    SnapshotContextMismatch {
        /// Snapshot context field that drifted.
        field: &'static str,
        /// Canonical expected value.
        expected: String,
        /// Observed conflicting value.
        actual: String,
    },
    /// Snapshot signature failed verification.
    #[error("thin snapshot signature is invalid")]
    InvalidSnapshotSignature,
    /// Snapshot digest does not match the unsigned body.
    #[error("thin snapshot digest does not match the signed body")]
    InvalidSnapshotDigest,
    /// Thin wrapper metadata digest does not match the unsigned body.
    #[error("thin wallet tx metadata hash does not match the wrapper body")]
    InvalidMetadataHash,
    /// Snapshot exists structurally but is stale for current policy time.
    #[error("thin snapshot expired at {expires_at_ms} before {now_ms}")]
    SnapshotExpired {
        /// Snapshot expiry timestamp in Unix milliseconds.
        expires_at_ms: u64,
        /// Verification time in Unix milliseconds.
        now_ms: u64,
    },
    /// Snapshot lookup failed.
    #[error("thin snapshot {0} is missing")]
    SnapshotMissing(String),
    /// Equivocation/conflict detected for one context key.
    #[error("thin snapshot conflict for {context_key}: {existing_digest} vs {new_digest}")]
    SnapshotConflict {
        /// Canonical context key where equivocation was detected.
        context_key: String,
        /// Already-published snapshot digest for this context.
        existing_digest: String,
        /// Newly proposed conflicting snapshot digest.
        new_digest: String,
    },
    /// Wrapper generation does not match the pinned snapshot generation.
    #[error("thin snapshot generation mismatch: expected {expected}, got {actual}")]
    SnapshotGenerationMismatch {
        /// Canonical expected compatibility generation.
        expected: u64,
        /// Observed conflicting compatibility generation.
        actual: u64,
    },
    /// Entry id lookup failed inside an otherwise present snapshot.
    #[error("thin snapshot entry {0} is missing")]
    EntryMissing(String),
    /// Duplicate or conflicting entry ids exist inside one snapshot.
    #[error("thin snapshot entry conflict for {0}")]
    EntryConflict(String),
    /// Canonical input/path family drifted between wrapper and entry.
    #[error("thin wallet tx input refs do not match the signed helper entry")]
    InputRefMismatch,
    /// Canonical package digest drifted.
    #[error("thin wallet tx digest mismatch: expected {expected}, got {actual}")]
    PackageDigestMismatch {
        /// Canonical expected package digest.
        expected: String,
        /// Observed conflicting package digest.
        actual: String,
    },
    /// Canonical package kind drifted.
    #[error("thin wallet tx kind mismatch: expected {expected}, got {actual}")]
    PackageKindMismatch {
        /// Canonical expected package kind.
        expected: String,
        /// Observed conflicting package kind.
        actual: String,
    },
    /// Canonical package type drifted.
    #[error("thin wallet tx type mismatch: expected {expected}, got {actual}")]
    PackageTypeMismatch {
        /// Canonical expected package type.
        expected: String,
        /// Observed conflicting package type.
        actual: String,
    },
    /// Canonical chain id drifted.
    #[error("thin wallet tx chain mismatch: expected {expected}, got {actual}")]
    PackageChainMismatch {
        /// Canonical expected chain id.
        expected: String,
        /// Observed conflicting chain id.
        actual: String,
    },
    /// Canonical root context drifted.
    #[error("thin wallet tx prev_root mismatch: expected {expected}, got {actual}")]
    PackageRootMismatch {
        /// Canonical expected previous root hex.
        expected: String,
        /// Observed conflicting previous root hex.
        actual: String,
    },
    /// The helper entry failed canonical package verification.
    #[error("thin helper package verification failed: {0}")]
    PackageVerificationFailed(String),
    /// One required signed snapshot field is empty or malformed.
    #[error("thin snapshot structure is invalid: {0}")]
    InvalidSnapshotShape(String),
}
