use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use z00z_storage::settlement::{
    chk_blob_settlement_inclusion, DefinitionRootLeaf, SerialRootLeaf, SettlementListReq,
    SettlementPath, SettlementStateRoot, SettlementStore, TerminalLeaf as StoreLeaf,
};
use z00z_wallets::key::ReceiverKeys;
use z00z_wallets::receiver::{receiver_scan_leaf, receiver_scan_report};

use crate::SimActor;

const POST_TX_DIR: &str = "post_tx";

/// Holds the committed-state proof inputs needed to validate one post-tx leaf.
#[derive(Clone, Debug)]
pub struct JmtScanCandidate {
    pub root: SettlementStateRoot,
    pub path: SettlementPath,
    pub def_leaf: DefinitionRootLeaf,
    pub ser_leaf: SerialRootLeaf,
    pub leaf: StoreLeaf,
    pub proof_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedCandidates {
    pub(crate) root: SettlementStateRoot,
    pub(crate) skipped_non_asset_count: usize,
    pub(crate) candidates: Vec<JmtScanCandidate>,
}

/// Records the wallet scan result for one committed post-tx leaf.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JmtScanRow {
    pub asset_id_hex: String,
    pub serial_id: u32,
    pub proof_validated: bool,
    pub receive_status: String,
    pub receive_next: String,
    pub receive_reject: Option<String>,
    pub owner_detected: bool,
    pub amount: Option<u64>,
    pub scan_path: String,
}

/// Summarizes the full committed-state scan for the post-tx artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JmtScanArtifact {
    pub actor: String,
    pub store_root_hex: String,
    pub scan_path: String,
    pub proof_step: String,
    pub distinction: String,
    pub candidate_count: usize,
    pub skipped_non_asset_count: usize,
    pub proof_validated_count: usize,
    pub detected_count: usize,
    pub total_detected_amount: u64,
    pub rows: Vec<JmtScanRow>,
    pub status: String,
}

/// Load committed post-tx leaves from storage so scan results stay reproducible.
pub fn load_post_tx_candidates(out: &Path) -> Result<Vec<JmtScanCandidate>, String> {
    Ok(load_post_tx_candidate_set(out)?.candidates)
}

pub(crate) fn load_post_tx_candidate_set(out: &Path) -> Result<LoadedCandidates, String> {
    let store_root = post_tx_store_root(out);
    let store = SettlementStore::load(&store_root).map_err(|e| {
        format!(
            "jmt scan: post_tx store load failed at {}: {e}",
            store_root.display()
        )
    })?;
    let settlement_root = store
        .settlement_root()
        .map_err(|e| format!("jmt scan: post_tx root load failed: {e}"))?;
    let (candidates, skipped_non_asset_count) = collect_post_tx_candidates(&store)?;
    Ok(LoadedCandidates {
        root: settlement_root,
        skipped_non_asset_count,
        candidates,
    })
}

fn collect_post_tx_candidates(
    store: &SettlementStore,
) -> Result<(Vec<JmtScanCandidate>, usize), String> {
    let mut after = None;
    let mut rows = Vec::new();
    let mut skipped_non_asset_count = 0usize;

    loop {
        let page = store
            .list_settlement(SettlementListReq::all(256).with_after(after))
            .map_err(|e| format!("jmt scan: post_tx list failed: {e}"))?;

        for item in page.items() {
            let Ok(leaf) = item.terminal_leaf() else {
                skipped_non_asset_count = skipped_non_asset_count.saturating_add(1);
                continue;
            };
            let path = item.path();
            let leaf = leaf.clone();
            let blob = store
                .settlement_proof_blob(&path)
                .map_err(|e| format!("jmt scan: proof build failed: {e}"))?;
            rows.push(JmtScanCandidate {
                root: blob.item().settlement_root(),
                path,
                def_leaf: blob.item().def_leaf(),
                ser_leaf: blob.item().ser_leaf(),
                leaf,
                proof_bytes: blob
                    .encode()
                    .map_err(|e| format!("jmt scan: proof encode failed: {e}"))?,
            });
        }

        after = page.next();
        if after.is_none() {
            break;
        }
    }

    Ok((rows, skipped_non_asset_count))
}

/// Validate the proof first, then ask the wallet scanner whether the leaf is mine.
pub fn scan_candidate(
    actor: &SimActor,
    candidate: &JmtScanCandidate,
) -> Result<JmtScanRow, String> {
    scan_candidate_keys(&actor.keys, candidate)
}

/// Validate the proof first, then ask the wallet scanner whether the leaf is mine.
pub fn scan_candidate_keys(
    keys: &ReceiverKeys,
    candidate: &JmtScanCandidate,
) -> Result<JmtScanRow, String> {
    verify_candidate(candidate)?;
    let report = receiver_scan_report(keys, &candidate.leaf)
        .map_err(|e| format!("jmt scan: receiver_scan_report failed: {e}"))?;
    let pack = receiver_scan_leaf(keys, &candidate.leaf)
        .map_err(|e| format!("jmt scan: receiver_scan_leaf failed: {e}"))?;
    let owner_detected = pack.is_some();

    Ok(JmtScanRow {
        asset_id_hex: hex::encode(candidate.path.terminal_id().as_bytes()),
        serial_id: candidate.path.serial_id.get(),
        proof_validated: true,
        receive_status: report.status.rpc_code().to_string(),
        receive_next: format!("{:?}", report.next),
        receive_reject: report.reject.map(|item| format!("{:?}", item)),
        owner_detected,
        amount: pack.map(|item| item.value),
        scan_path: "committed_post_tx_jmt".to_string(),
    })
}

/// Aggregate candidate scans into the post-tx JMT report artifact.
pub fn scan_post_tx_actor(out: &Path, actor: &SimActor) -> Result<JmtScanArtifact, String> {
    scan_post_tx_keys(out, &actor.name, &actor.keys)
}

/// Aggregate candidate scans into the post-tx JMT report artifact.
pub fn scan_post_tx_keys(
    out: &Path,
    actor_name: &str,
    keys: &ReceiverKeys,
) -> Result<JmtScanArtifact, String> {
    let loaded = load_post_tx_candidate_set(out)?;
    let candidates = loaded.candidates;
    let store_root_hex = hex::encode(loaded.root.as_bytes());
    let mut rows = Vec::with_capacity(candidates.len());
    let mut total_detected_amount = 0u64;
    let mut detected_count = 0usize;

    for candidate in &candidates {
        let row = scan_candidate_keys(keys, candidate)?;
        if row.owner_detected {
            detected_count += 1;
            total_detected_amount = total_detected_amount.saturating_add(row.amount.unwrap_or(0));
        }
        rows.push(row);
    }

    Ok(JmtScanArtifact {
        actor: actor_name.to_string(),
        store_root_hex,
        scan_path: "jmt_scan".to_string(),
        proof_step: "proof_blob+chk_blob_settlement before ownership detection".to_string(),
        distinction: "This artifact proves committed-state JMT inclusion first; it is not equivalent to detached Stage 5 leaf scan.".to_string(),
        candidate_count: rows.len(),
        skipped_non_asset_count: loaded.skipped_non_asset_count,
        proof_validated_count: rows.len(),
        detected_count,
        total_detected_amount,
        rows,
        status: "ok".to_string(),
    })
}

/// Reject incomplete proof blobs before the leaf is treated as scan-valid.
pub fn verify_candidate(candidate: &JmtScanCandidate) -> Result<(), String> {
    if candidate.proof_bytes.is_empty() {
        return Err(
            "jmt scan: committed-state proof bytes are required before ownership detection"
                .to_string(),
        );
    }

    chk_blob_settlement_inclusion(
        &candidate.proof_bytes,
        candidate.root,
        &candidate.path,
        candidate.def_leaf,
        candidate.ser_leaf,
        &candidate.leaf,
    )
    .map(|_| ())
    .map_err(|e| format!("jmt scan: committed-state proof validation failed: {e}"))
}

fn post_tx_store_root(out: &Path) -> PathBuf {
    out.join("storage").join(POST_TX_DIR)
}

#[cfg(test)]
mod tests {
    use z00z_storage::settlement::{
        DefinitionId, RightClass, RightLeaf, SerialId, SettlementPath, StoreItem, TerminalId,
        TerminalLeaf,
    };

    use super::*;

    fn bytes(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn right_leaf(mark: u8) -> RightLeaf {
        RightLeaf {
            version: 1,
            terminal_id: TerminalId::new(bytes(mark)),
            right_class: RightClass::MachineCapability,
            issuer_scope: bytes(mark.wrapping_add(1)),
            provider_scope: bytes(mark.wrapping_add(2)),
            holder_commitment: bytes(mark.wrapping_add(3)),
            control_commitment: bytes(mark.wrapping_add(4)),
            beneficiary_commitment: bytes(mark.wrapping_add(5)),
            payload_commitment: bytes(mark.wrapping_add(6)),
            valid_from: 10,
            valid_until: 20,
            challenge_from: 12,
            challenge_until: 18,
            use_nonce: bytes(mark.wrapping_add(6)),
            revocation_policy_id: bytes(mark.wrapping_add(7)),
            transition_policy_id: bytes(mark.wrapping_add(8)),
            challenge_policy_id: bytes(mark.wrapping_add(9)),
            disclosure_policy_id: bytes(mark.wrapping_add(10)),
            retention_policy_id: bytes(mark.wrapping_add(11)),
        }
    }

    #[test]
    fn collect_post_tx_skips_rights() {
        let mut store = SettlementStore::new();

        let asset_path = SettlementPath::new(
            DefinitionId::new(bytes(0x11)),
            SerialId::new(7),
            TerminalId::new(bytes(0x21)),
        );
        let mut asset_leaf = TerminalLeaf::dummy_for_scan(7);
        asset_leaf.asset_id = asset_path.terminal_id().into_bytes();
        asset_leaf.serial_id = asset_path.serial_id.get();
        store
            .put_settlement_item(StoreItem::new(asset_path, asset_leaf).expect("asset item"))
            .expect("put asset item");

        let right_path = SettlementPath::new(
            DefinitionId::new(bytes(0x12)),
            SerialId::new(8),
            TerminalId::new(bytes(0x31)),
        );
        store
            .put_settlement_item(StoreItem::new(right_path, right_leaf(0x31)).expect("right item"))
            .expect("put right item");

        let (rows, skipped_non_asset_count) =
            collect_post_tx_candidates(&store).expect("collect candidates");

        assert_eq!(rows.len(), 1);
        assert_eq!(skipped_non_asset_count, 1);
        assert_eq!(rows[0].path, asset_path);
    }
}
