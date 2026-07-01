use std::collections::HashSet;

use thiserror::Error;
use z00z_core::assets::AssetClass;
use z00z_crypto::Z00ZCommitment;
use z00z_storage::settlement::{ProofScanOut, SettlementStateRoot};

use super::{spend_proof_backend::SpendProofStmt, tx_wire::TxOutputWire};

/// One validated storage leaf paired with its transaction output evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct AssetClassAuditEntry {
    /// Sanitized storage proof summary for the leaf.
    pub scan: ProofScanOut,
    /// Transaction output wire that should match the scanned leaf.
    pub output: TxOutputWire,
}

/// Explicit target for one operator-invoked asset-class audit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetClassAuditTarget {
    /// Compare the recomputed total against a caller-provided commitment.
    ExpectedTotalCommitment {
        /// Expected total commitment for the selected asset class.
        expected_total: Z00ZCommitment,
    },
    /// Compare against a checkpoint-resolved expected commitment.
    CheckpointEquation {
        /// Expected total commitment resolved from the checkpoint equation.
        expected_total: Z00ZCommitment,
        /// Operator-visible checkpoint identifier used to resolve the target.
        checkpoint_id: String,
    },
    /// Compare against the commitment delta between issued and burned totals.
    IssuanceBurnDeltaTarget {
        /// Issued total commitment.
        issued_total: Z00ZCommitment,
        /// Burned total commitment.
        burned_total: Z00ZCommitment,
    },
}

impl AssetClassAuditTarget {
    /// Resolve this target into the concrete commitment used by the audit.
    pub fn expected_total(&self) -> Z00ZCommitment {
        match self {
            Self::ExpectedTotalCommitment { expected_total } => expected_total.clone(),
            Self::CheckpointEquation { expected_total, .. } => expected_total.clone(),
            Self::IssuanceBurnDeltaTarget {
                issued_total,
                burned_total,
            } => issued_total - burned_total,
        }
    }
}

/// Status for a typed asset-class audit outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetClassAuditStatus {
    /// The selected asset class matched the resolved target commitment.
    Pass,
    /// The audit rejected the evidence and preserved typed diagnostics.
    FailClosed,
}

/// Typed fail-closed class for asset-class audit diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetClassAuditMismatchClass {
    /// Required audit evidence was absent.
    MissingEvidence,
    /// Semantic root, backend root, or root-bind evidence did not match.
    RootMismatch,
    /// Storage leaf fields did not match transaction output evidence.
    LeafMismatch,
    /// A leaf belonged to a different asset class than the selected audit class.
    AssetClassMismatch,
    /// An output commitment did not match the scanned leaf commitment.
    CommitmentMismatch,
    /// The recomputed total did not match the resolved audit target.
    TargetMismatch,
    /// The audit evidence contained the same leaf identity more than once.
    DuplicateEntry,
    /// Hash-bound evidence failed validation.
    HashMismatch,
    /// Serialized evidence failed validation.
    SerializationMismatch,
    /// Spend-proof public leaves did not match the audited leaves.
    SpendProofMismatch,
}

/// Explicit diagnostic report for one asset-class total audit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetClassAuditReport {
    /// Asset class that was audited.
    pub asset_class: AssetClass,
    /// Semantic root that the storage proofs were checked against.
    pub semantic_root: SettlementStateRoot,
    /// Diagnostic proof-local backend root from validated storage proof scans.
    ///
    /// This field is recorded only with `semantic_root` and `root_bind`; it is
    /// not a state authority root and must not replace `SettlementStateRoot`.
    pub backend_root: [u8; 32],
    /// Root-bind value returned by the validated storage proof scans.
    pub root_bind: [u8; 32],
    /// Number of validated leaves included in the total.
    pub leaf_count: usize,
    /// Sum of the matched output commitments.
    pub total_commitment: Z00ZCommitment,
    /// Target that was resolved before comparing the recomputed total.
    pub target: AssetClassAuditTarget,
    /// Optional typed fail-closed class associated with this report.
    pub mismatch_class: Option<AssetClassAuditMismatchClass>,
}

/// Typed result object for a manual asset-class audit attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetClassAuditOutcome {
    /// Pass or fail-closed status.
    pub status: AssetClassAuditStatus,
    /// Report payload for the selected asset class and target.
    pub report: AssetClassAuditReport,
    /// Top-level mismatch class when the audit failed closed.
    pub mismatch_class: Option<AssetClassAuditMismatchClass>,
    /// Entry index for entry-specific mismatch classes.
    pub entry_index: Option<usize>,
}

/// Fail-closed reasons for the operator-invoked asset-class audit path.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssetClassAuditErr {
    /// Required evidence was not provided to the audit helper.
    #[error("missing evidence: {0}")]
    MissingEvidence(&'static str),
    /// The scanned leaf is not anchored to the selected semantic root.
    #[error("membership mismatch at entry {entry_index}")]
    Membership {
        /// Index of the entry that failed the root check.
        entry_index: usize,
    },
    /// The backend root or root-bind does not match the validated scan.
    #[error("backend-root binding mismatch at entry {entry_index}")]
    RootBind {
        /// Index of the entry that failed the root-bind check.
        entry_index: usize,
    },
    /// The leaf fields do not match the transaction output evidence.
    #[error("leaf mismatch at entry {entry_index}")]
    Leaf {
        /// Index of the entry that failed the leaf check.
        entry_index: usize,
    },
    /// The spend proof output leaves do not match the scanned leaves.
    #[error("spend-proof mismatch at entry {entry_index}")]
    SpendProof {
        /// Index of the entry that failed the spend-proof check.
        entry_index: usize,
    },
    /// The hash-bound storage leaf evidence did not match the scanned leaf.
    #[error("asset leaf hash mismatch at entry {entry_index}")]
    Hash {
        /// Index of the entry that failed the leaf-hash check.
        entry_index: usize,
    },
    /// The matched commitments do not add up to the expected total.
    #[error("commitment mismatch at entry {entry_index}")]
    Commitment {
        /// Index of the entry associated with the commitment mismatch.
        entry_index: usize,
    },
    /// The output is not of the selected asset class.
    #[error("asset-class audit mismatch at entry {entry_index}")]
    AssetClass {
        /// Index of the entry that failed the asset-class check.
        entry_index: usize,
    },
    /// A duplicate entry appeared in the operator-provided audit evidence.
    #[error("duplicate audit entry at entry {entry_index}")]
    DuplicateEntry {
        /// Index of the duplicated entry.
        entry_index: usize,
    },
    /// The recomputed total did not match the resolved audit target.
    #[error("asset-class target mismatch at entry {entry_index}")]
    Target {
        /// Index of the entry associated with the target mismatch.
        entry_index: usize,
    },
}

impl AssetClassAuditErr {
    /// Return the typed fail-closed mismatch class for this error.
    pub fn mismatch_class(&self) -> AssetClassAuditMismatchClass {
        match self {
            Self::MissingEvidence(_) => AssetClassAuditMismatchClass::MissingEvidence,
            Self::Membership { .. } | Self::RootBind { .. } => {
                AssetClassAuditMismatchClass::RootMismatch
            }
            Self::Leaf { .. } => AssetClassAuditMismatchClass::LeafMismatch,
            Self::SpendProof { .. } => AssetClassAuditMismatchClass::SpendProofMismatch,
            Self::Hash { .. } => AssetClassAuditMismatchClass::HashMismatch,
            Self::Commitment { .. } => AssetClassAuditMismatchClass::CommitmentMismatch,
            Self::AssetClass { .. } => AssetClassAuditMismatchClass::AssetClassMismatch,
            Self::DuplicateEntry { .. } => AssetClassAuditMismatchClass::DuplicateEntry,
            Self::Target { .. } => AssetClassAuditMismatchClass::TargetMismatch,
        }
    }

    /// Return the entry index when the mismatch is entry-specific.
    pub fn entry_index(&self) -> Option<usize> {
        match self {
            Self::MissingEvidence(_) => None,
            Self::Membership { entry_index }
            | Self::RootBind { entry_index }
            | Self::Leaf { entry_index }
            | Self::SpendProof { entry_index }
            | Self::Hash { entry_index }
            | Self::Commitment { entry_index }
            | Self::AssetClass { entry_index }
            | Self::DuplicateEntry { entry_index }
            | Self::Target { entry_index } => Some(*entry_index),
        }
    }
}

/// Compute an explicit asset-class Pedersen total diagnostic over validated leaves.
///
/// This helper is intentionally diagnostic only. It is not part of canonical tx
/// admission and it does not replace membership verification, spend proof
/// verification, or the normal transaction balance gates.
pub fn audit_asset_class_total(
    asset_class: AssetClass,
    semantic_root: SettlementStateRoot,
    entries: &[AssetClassAuditEntry],
    spend_stmt: Option<&SpendProofStmt>,
    expected_total: Option<&Z00ZCommitment>,
) -> Result<AssetClassAuditReport, AssetClassAuditErr> {
    let expected_total = expected_total.ok_or(AssetClassAuditErr::MissingEvidence(
        "expected total commitment",
    ))?;
    audit_asset_class_with_target(
        asset_class,
        semantic_root,
        entries,
        spend_stmt,
        AssetClassAuditTarget::ExpectedTotalCommitment {
            expected_total: expected_total.clone(),
        },
    )
}

/// Compute a typed asset-class audit outcome over validated leaves.
///
/// This is the status-preserving API for operator diagnostics. It keeps the
/// manual audit out-of-band from canonical transaction admission while exposing
/// the target, status, mismatch class, and entry index together.
pub fn audit_asset_class_outcome(
    asset_class: AssetClass,
    semantic_root: SettlementStateRoot,
    entries: &[AssetClassAuditEntry],
    spend_stmt: Option<&SpendProofStmt>,
    target: AssetClassAuditTarget,
) -> AssetClassAuditOutcome {
    match audit_asset_class_with_target(
        asset_class,
        semantic_root,
        entries,
        spend_stmt,
        target.clone(),
    ) {
        Ok(report) => AssetClassAuditOutcome {
            status: AssetClassAuditStatus::Pass,
            report,
            mismatch_class: None,
            entry_index: None,
        },
        Err(err) => {
            let mismatch_class = err.mismatch_class();
            AssetClassAuditOutcome {
                status: AssetClassAuditStatus::FailClosed,
                report: failure_report(asset_class, semantic_root, entries, target, mismatch_class),
                mismatch_class: Some(mismatch_class),
                entry_index: err.entry_index(),
            }
        }
    }
}

fn audit_asset_class_with_target(
    asset_class: AssetClass,
    semantic_root: SettlementStateRoot,
    entries: &[AssetClassAuditEntry],
    spend_stmt: Option<&SpendProofStmt>,
    target: AssetClassAuditTarget,
) -> Result<AssetClassAuditReport, AssetClassAuditErr> {
    if entries.is_empty() {
        return Err(AssetClassAuditErr::MissingEvidence("asset entries"));
    }

    let spend_stmt = spend_stmt.ok_or(AssetClassAuditErr::MissingEvidence("spend proof"))?;
    let expected_total = target.expected_total();

    if spend_stmt.output_leaves.len() != entries.len() {
        return Err(AssetClassAuditErr::SpendProof { entry_index: 0 });
    }

    let first = &entries[0];
    let backend_root = first.scan.backend_root();
    let root_bind = first.scan.root_bind();
    let mut total_commitment = first.output.asset_wire.commitment.clone();
    let mut seen_entries = HashSet::with_capacity(entries.len());

    validate_entry(
        0,
        asset_class,
        semantic_root,
        backend_root,
        root_bind,
        first,
        spend_stmt,
    )?;
    insert_entry_key(&mut seen_entries, 0, first)?;

    for (entry_index, entry) in entries.iter().enumerate().skip(1) {
        validate_entry(
            entry_index,
            asset_class,
            semantic_root,
            backend_root,
            root_bind,
            entry,
            spend_stmt,
        )?;
        insert_entry_key(&mut seen_entries, entry_index, entry)?;
        total_commitment = &total_commitment + &entry.output.asset_wire.commitment;
    }

    if total_commitment.as_bytes() != expected_total.as_bytes() {
        return Err(AssetClassAuditErr::Target {
            entry_index: entries.len().saturating_sub(1),
        });
    }

    Ok(AssetClassAuditReport {
        asset_class,
        semantic_root,
        backend_root,
        root_bind,
        leaf_count: entries.len(),
        total_commitment,
        target,
        mismatch_class: None,
    })
}

fn insert_entry_key(
    seen_entries: &mut HashSet<([u8; 32], u32)>,
    entry_index: usize,
    entry: &AssetClassAuditEntry,
) -> Result<(), AssetClassAuditErr> {
    let leaf = entry
        .scan
        .terminal_leaf()
        .map_err(|_| AssetClassAuditErr::Leaf { entry_index })?;
    let key = (leaf.asset_id, leaf.serial_id);
    if !seen_entries.insert(key) {
        return Err(AssetClassAuditErr::DuplicateEntry { entry_index });
    }
    Ok(())
}

fn failure_report(
    asset_class: AssetClass,
    semantic_root: SettlementStateRoot,
    entries: &[AssetClassAuditEntry],
    target: AssetClassAuditTarget,
    mismatch_class: AssetClassAuditMismatchClass,
) -> AssetClassAuditReport {
    let (backend_root, root_bind, total_commitment) = entries.first().map_or_else(
        || ([0u8; 32], [0u8; 32], Z00ZCommitment::default()),
        |entry| {
            let total = entries
                .iter()
                .skip(1)
                .fold(entry.output.asset_wire.commitment.clone(), |acc, item| {
                    &acc + &item.output.asset_wire.commitment
                });
            (entry.scan.backend_root(), entry.scan.root_bind(), total)
        },
    );

    AssetClassAuditReport {
        asset_class,
        semantic_root,
        backend_root,
        root_bind,
        leaf_count: entries.len(),
        total_commitment,
        target,
        mismatch_class: Some(mismatch_class),
    }
}

fn validate_entry(
    entry_index: usize,
    asset_class: AssetClass,
    semantic_root: SettlementStateRoot,
    backend_root: [u8; 32],
    root_bind: [u8; 32],
    entry: &AssetClassAuditEntry,
    spend_stmt: &SpendProofStmt,
) -> Result<(), AssetClassAuditErr> {
    if entry.scan.settlement_root() != semantic_root {
        return Err(AssetClassAuditErr::Membership { entry_index });
    }

    // Backend-root equality is only a diagnostic consistency check. The
    // semantic root and explicit root-bind verification remain the authority.
    if entry.scan.backend_root() != backend_root || entry.scan.root_bind() != root_bind {
        return Err(AssetClassAuditErr::RootBind { entry_index });
    }

    entry
        .scan
        .check_root_bind()
        .map_err(|_| AssetClassAuditErr::RootBind { entry_index })?;
    entry
        .scan
        .check_leaf_hash()
        .map_err(|_| AssetClassAuditErr::Hash { entry_index })?;

    if entry.output.asset_wire.definition.class != asset_class {
        return Err(AssetClassAuditErr::AssetClass { entry_index });
    }

    let scan_leaf = entry
        .scan
        .terminal_leaf()
        .map_err(|_| AssetClassAuditErr::Leaf { entry_index })?;

    let leaf_ad_id = entry
        .output
        .asset_wire
        .leaf_ad_id
        .ok_or(AssetClassAuditErr::MissingEvidence("output leaf_ad_id"))?;
    let output_r_pub = entry
        .output
        .asset_wire
        .r_pub
        .ok_or(AssetClassAuditErr::MissingEvidence("output r_pub"))?;
    let output_owner_tag = entry
        .output
        .asset_wire
        .owner_tag
        .ok_or(AssetClassAuditErr::MissingEvidence("output owner_tag"))?;

    if scan_leaf.asset_id != leaf_ad_id
        || scan_leaf.serial_id != entry.output.asset_wire.serial_id
        || scan_leaf.r_pub != output_r_pub
        || scan_leaf.owner_tag != output_owner_tag
    {
        return Err(AssetClassAuditErr::Leaf { entry_index });
    }

    if scan_leaf.c_amount != entry.output.asset_wire.commitment.as_bytes() {
        return Err(AssetClassAuditErr::Commitment { entry_index });
    }

    let stmt_leaf = spend_stmt
        .output_leaves
        .get(entry_index)
        .ok_or(AssetClassAuditErr::SpendProof { entry_index })?;
    if stmt_leaf.asset_id != scan_leaf.asset_id
        || stmt_leaf.serial_id != scan_leaf.serial_id
        || stmt_leaf.r_pub != scan_leaf.r_pub
        || stmt_leaf.owner_tag != scan_leaf.owner_tag
        || stmt_leaf.c_amount != scan_leaf.c_amount
    {
        return Err(AssetClassAuditErr::SpendProof { entry_index });
    }

    Ok(())
}
