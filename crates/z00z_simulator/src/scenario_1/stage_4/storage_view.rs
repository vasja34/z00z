use std::path::{Path, PathBuf};

use z00z_core::{genesis::GenesisRightRecord, rights::RightClassConfig};
use z00z_storage::{
    checkpoint::{
        audit::CheckpointAudit, CheckpointArtifact, CheckpointDraft, CheckpointExecInput,
        CheckpointFsStore, CheckpointLink, CheckpointStore,
    },
    settlement::{
        CheckRoot, DefinitionId, RightClass, RightLeaf, SerialId, SettlementListReq,
        SettlementLookup, SettlementPath, SettlementStore, StoreItem, StoreOp, TerminalId,
    },
    snapshot::{PrepFsStore, PrepSnapshot, PrepSnapshotId, PrepSnapshotStore},
};
use z00z_utils::codec::json;
use z00z_utils::io::path_exists;

use super::storage_view_patch::{save_ledger_path, save_summary};

const CLAIM_POST_DIR: &str = "claim_post";
const PRE_TX_DIR: &str = "pre_tx";
const POST_TX_DIR: &str = "post_tx";
const REDB_FILE: &str = "settlement_state.redb";
pub(super) const LEDGER_PATH_FILE: &str = "ledger_path.json";
const LEDGER_DRAFT_KEYS: &[&str] = &["checkpoint_id_hex"];
const STORAGE_VIEW_PAGE_SIZE: usize = if cfg!(test) { 8 } else { 256 };

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorageViewRoots {
    pub(crate) state_root_hex: String,
}

fn settlement_check_root(store: &SettlementStore) -> Result<CheckRoot, String> {
    store
        .settlement_root()
        .map(CheckRoot::from)
        .map_err(|e| e.to_string())
}

pub(crate) fn export_claim_post_view(
    out: &Path,
    store: &SettlementStore,
) -> Result<PathBuf, String> {
    let view_root = storage_root(out, CLAIM_POST_DIR);
    let want_root = settlement_check_root(store)?;
    let persisted = materialize_live_store(&view_root, store)?;
    let got_root = settlement_check_root(&persisted)?;
    if got_root != want_root {
        return Err(format!(
            "claim_post view root drifted persisted={} rebuilt={}",
            hex::encode(got_root.as_bytes()),
            hex::encode(want_root.as_bytes())
        ));
    }
    let roots = describe_store_roots(&persisted)?;
    save_summary(
        &view_root,
        json!({
            "view": CLAIM_POST_DIR,
            "source_check_root_hex": hex::encode(want_root.as_bytes()),
            "state_root_hex": roots.state_root_hex,
            "view_check_root_hex": hex::encode(got_root.as_bytes()),
            "root_match": got_root == want_root,
            "status": "ok"
        }),
    )?;
    save_ledger_path(
        out,
        LEDGER_DRAFT_KEYS,
        json!({
            "claim_root_hex": hex::encode(want_root.as_bytes()),
        }),
    )?;
    Ok(view_root)
}

pub(crate) fn export_pre_tx_view(
    out: &Path,
    snapshot_id: PrepSnapshotId,
    snapshot: &PrepSnapshot,
) -> Result<PathBuf, String> {
    let view_root = storage_root(out, PRE_TX_DIR);
    let persisted = sync_snapshot_store(&view_root, snapshot)?;
    let got_root = settlement_check_root(&persisted)?;
    let roots = describe_store_roots(&persisted)?;
    save_snapshot(&view_root, snapshot_id, snapshot)?;
    save_summary(
        &view_root,
        json!({
            "view": PRE_TX_DIR,
            "snapshot_id_hex": hex::encode(snapshot_id.as_bytes()),
            "source_check_root_hex": hex::encode(snapshot.prev_root.as_bytes()),
            "state_root_hex": roots.state_root_hex,
            "view_check_root_hex": hex::encode(got_root.as_bytes()),
            "root_match": got_root == snapshot.prev_root,
            "status": "ok"
        }),
    )?;
    save_ledger_path(
        out,
        LEDGER_DRAFT_KEYS,
        json!({
            "prep_root_hex": hex::encode(snapshot.prev_root.as_bytes()),
            "snapshot_id_hex": hex::encode(snapshot_id.as_bytes()),
        }),
    )?;
    Ok(view_root)
}

pub(crate) fn export_post_tx_view(
    out: &Path,
    snapshot_id: PrepSnapshotId,
    snapshot: &PrepSnapshot,
    exec: &CheckpointExecInput,
    draft: &CheckpointDraft,
) -> Result<PathBuf, String> {
    let view_root = storage_root(out, POST_TX_DIR);
    let persisted = sync_post_store(&view_root, snapshot, exec, draft)?;
    let got_root = settlement_check_root(&persisted)?;
    let roots = describe_store_roots(&persisted)?;
    save_snapshot(&view_root, snapshot_id, snapshot)?;
    let mut checkpoint_store = CheckpointFsStore::new(&view_root);
    let exec_id = checkpoint_store
        .save_exec_input(exec)
        .map_err(|e| format!("storage view exec save failed: {e}"))?;
    let draft_id = checkpoint_store
        .save_draft(draft)
        .map_err(|e| format!("storage view draft save failed: {e}"))?;
    save_summary(
        &view_root,
        json!({
            "view": POST_TX_DIR,
            "snapshot_id_hex": hex::encode(snapshot_id.as_bytes()),
            "exec_input_id_hex": hex::encode(exec_id.as_bytes()),
            "draft_id_hex": hex::encode(draft_id.as_bytes()),
            "prev_root_hex": hex::encode(draft.prev_root().as_bytes()),
            "new_root_hex": hex::encode(draft.new_root().as_bytes()),
            "state_root_hex": roots.state_root_hex,
            "view_check_root_hex": hex::encode(got_root.as_bytes()),
            "matches_prev_root": got_root == draft.prev_root(),
            "matches_new_root": got_root == draft.new_root(),
            "status": "draft_ok"
        }),
    )?;
    save_ledger_path(
        out,
        LEDGER_DRAFT_KEYS,
        json!({
            "prep_root_hex": hex::encode(draft.prev_root().as_bytes()),
            "post_apply_root_hex": hex::encode(draft.new_root().as_bytes()),
            "snapshot_id_hex": hex::encode(snapshot_id.as_bytes()),
            "exec_input_id_hex": hex::encode(exec_id.as_bytes()),
            "draft_id_hex": hex::encode(draft_id.as_bytes()),
        }),
    )?;
    Ok(view_root)
}

pub(crate) fn export_post_tx_final_view(
    out: &Path,
    artifact: &CheckpointArtifact,
    link: &CheckpointLink,
    audit: &CheckpointAudit,
) -> Result<PathBuf, String> {
    // Any prior-final files exported here stay noncanonical and do not
    // weaken the canonical draft/final checkpoint class boundary.
    let view_root = storage_root(out, POST_TX_DIR);
    let mut checkpoint_store = CheckpointFsStore::new(&view_root);
    let checkpoint_id = checkpoint_store
        .export_noncanonical_final_bundle(artifact, link, audit)
        .map_err(|e| format!("storage view export save failed: {e}"))?;
    save_summary(
        &view_root,
        json!({
            "view": POST_TX_DIR,
            "checkpoint_id_hex": hex::encode(checkpoint_id.as_bytes()),
            "final_lane": "noncanonical_export",
            "status": "finalized"
        }),
    )?;
    save_ledger_path(
        out,
        &[],
        json!({
            "checkpoint_id_hex": hex::encode(checkpoint_id.as_bytes()),
        }),
    )?;
    Ok(view_root)
}

pub(crate) fn publish_genesis_rights(
    store: &mut SettlementStore,
    rights: &[GenesisRightRecord],
) -> Result<(), String> {
    for record in rights {
        let item = genesis_right_item(record)?;
        store.put_settlement_item(item).map_err(|e| {
            format!(
                "claim publish right insert failed for {}#{}: {e}",
                record.right_id, record.right_index
            )
        })?;
    }
    Ok(())
}

pub(super) fn storage_root(out: &Path, label: &str) -> PathBuf {
    out.join("storage").join(label)
}

fn genesis_right_item(record: &GenesisRightRecord) -> Result<StoreItem, String> {
    let terminal_id = TerminalId::new(record.leaf.terminal_id);
    let path = SettlementPath::new(
        DefinitionId::new(record.definition_id),
        SerialId::new(record.serial_id),
        terminal_id,
    );
    let leaf = RightLeaf {
        version: record.leaf.version,
        terminal_id,
        right_class: storage_right_class(record.leaf.right_class),
        issuer_scope: record.leaf.issuer_scope,
        provider_scope: record.leaf.provider_scope,
        holder_commitment: record.leaf.holder_commitment,
        control_commitment: record.leaf.control_commitment,
        beneficiary_commitment: record.leaf.beneficiary_commitment,
        payload_commitment: record.leaf.payload_commitment,
        valid_from: record.leaf.valid_from,
        valid_until: record.leaf.valid_until,
        challenge_from: record.leaf.challenge_from,
        challenge_until: record.leaf.challenge_until,
        use_nonce: record.leaf.use_nonce,
        revocation_policy_id: record.leaf.revocation_policy_id,
        transition_policy_id: record.leaf.transition_policy_id,
        challenge_policy_id: record.leaf.challenge_policy_id,
        disclosure_policy_id: record.leaf.disclosure_policy_id,
        retention_policy_id: record.leaf.retention_policy_id,
    };
    StoreItem::new(path, leaf).map_err(|e| {
        format!(
            "claim publish right store item failed for {}#{}: {e}",
            record.right_id, record.right_index
        )
    })
}

const fn storage_right_class(class: RightClassConfig) -> RightClass {
    match class {
        RightClassConfig::MachineCapability => RightClass::MachineCapability,
        RightClassConfig::DataAccess => RightClass::DataAccess,
        RightClassConfig::ServiceEntitlement => RightClass::ServiceEntitlement,
        RightClassConfig::ValidatorMandate => RightClass::ValidatorMandate,
        RightClassConfig::OneTimeUse => RightClass::OneTimeUse,
    }
}

fn materialize_live_store(
    view_root: &Path,
    source: &SettlementStore,
) -> Result<SettlementStore, String> {
    let (persisted, has_state) = load_view_store(view_root)?;
    if has_state {
        return Ok(persisted);
    }
    let mut after = None;
    let mut ops = Vec::new();
    loop {
        let page = source
            .list_settlement(SettlementListReq::all(STORAGE_VIEW_PAGE_SIZE).with_after(after))
            .map_err(|e| format!("storage view list failed: {e}"))?;
        ops.extend(
            page.items()
                .iter()
                .cloned()
                .map(|item| StoreOp::Put(Box::new(item))),
        );
        after = page.next();
        if after.is_none() {
            break;
        }
    }
    let persisted = apply_ops_only(persisted, ops)?;
    Ok(persisted)
}

pub(crate) fn sync_snapshot_store(
    view_root: &Path,
    snapshot: &PrepSnapshot,
) -> Result<SettlementStore, String> {
    let (persisted, has_state) = load_view_store(view_root)?;
    if has_state {
        return check_store_root(persisted, "pre_tx storage", snapshot.prev_root);
    }
    let ops = snapshot_put_ops(snapshot)?;
    apply_ops_only(persisted, ops)
}

pub(crate) fn sync_post_store(
    view_root: &Path,
    snapshot: &PrepSnapshot,
    exec: &CheckpointExecInput,
    draft: &CheckpointDraft,
) -> Result<SettlementStore, String> {
    // These rehydrate/load flows prove the bundle guards around persisted
    // checkpoint ids, links, and bound roots, but they are not yet a dedicated
    // spent-row rehydrate-then-replay theorem.
    let (persisted, has_state) = load_view_store(view_root)?;
    let persisted = init_post_store(persisted, has_state, snapshot, draft)?;
    let now = settlement_check_root(&persisted)?;
    if now == draft.new_root() {
        return Ok(persisted);
    }
    let ops = exec_apply_ops(&persisted, exec)?;
    apply_ops_only(persisted, ops)
}

pub(crate) fn describe_store_roots(store: &SettlementStore) -> Result<StorageViewRoots, String> {
    let state_root = settlement_check_root(store)?;

    Ok(StorageViewRoots {
        state_root_hex: hex::encode(state_root.as_bytes()),
    })
}

fn load_view_store(view_root: &Path) -> Result<(SettlementStore, bool), String> {
    let has_state = has_redb(view_root)?;
    let persisted = SettlementStore::load(view_root).map_err(|e| e.to_string())?;
    Ok((persisted, has_state))
}

fn init_post_store(
    persisted: SettlementStore,
    has_state: bool,
    snapshot: &PrepSnapshot,
    draft: &CheckpointDraft,
) -> Result<SettlementStore, String> {
    if has_state {
        let got = settlement_check_root(&persisted)?;
        if got == draft.new_root() {
            return Ok(persisted);
        }
        if got == snapshot.prev_root {
            return Ok(persisted);
        }
        return Err(format!(
            "post_tx storage root mismatch: want_pre={} want_post={} got={}",
            hex::encode(snapshot.prev_root.as_bytes()),
            hex::encode(draft.new_root().as_bytes()),
            hex::encode(got.as_bytes())
        ));
    }

    let ops = snapshot_put_ops(snapshot)?;
    apply_ops_only(persisted, ops)
}

fn check_store_root(
    persisted: SettlementStore,
    label: &str,
    want_root: z00z_storage::settlement::CheckRoot,
) -> Result<SettlementStore, String> {
    let got = settlement_check_root(&persisted)?;
    if got != want_root {
        return root_mix(label, want_root, got);
    }
    Ok(persisted)
}

fn apply_ops_only(
    mut persisted: SettlementStore,
    ops: Vec<StoreOp>,
) -> Result<SettlementStore, String> {
    persisted
        .apply_settlement_ops(ops)
        .map_err(|e| e.to_string())?;
    Ok(persisted)
}

fn snapshot_put_ops(snapshot: &PrepSnapshot) -> Result<Vec<StoreOp>, String> {
    snapshot
        .entries
        .iter()
        .map(|entry| {
            Ok(StoreOp::Put(Box::new(
                StoreItem::new(entry.path(), entry.leaf().clone())
                    .map_err(|e| format!("snapshot store item build failed: {e}"))?,
            )))
        })
        .collect()
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use z00z_storage::settlement::{SettlementListReq, StoreItem};
    use z00z_utils::codec::Value;
    use z00z_utils::io::load_json;

    use super::*;

    fn field_bytes(index: u32, tag: u8) -> [u8; 32] {
        let mut bytes = [tag; 32];
        bytes[..4].copy_from_slice(&index.to_le_bytes());
        bytes
    }

    fn right_item(index: u32) -> StoreItem {
        let terminal_bytes = field_bytes(index, 0xA1);
        let terminal_id = TerminalId::new(terminal_bytes);
        let path = SettlementPath::new(
            DefinitionId::new(terminal_bytes),
            SerialId::new(index + 1),
            terminal_id,
        );
        let leaf = RightLeaf {
            version: 1,
            terminal_id,
            right_class: RightClass::MachineCapability,
            issuer_scope: field_bytes(index, 0xB1),
            provider_scope: field_bytes(index, 0xB2),
            holder_commitment: field_bytes(index, 0xB3),
            control_commitment: field_bytes(index, 0xBC),
            beneficiary_commitment: field_bytes(index, 0xB4),
            payload_commitment: field_bytes(index, 0xB5),
            valid_from: 10,
            valid_until: 20,
            challenge_from: 11,
            challenge_until: 19,
            use_nonce: field_bytes(index, 0xB6),
            revocation_policy_id: field_bytes(index, 0xB7),
            transition_policy_id: field_bytes(index, 0xB8),
            challenge_policy_id: field_bytes(index, 0xB9),
            disclosure_policy_id: field_bytes(index, 0xBA),
            retention_policy_id: field_bytes(index, 0xBB),
        };
        StoreItem::new(path, leaf).expect("right store item")
    }

    #[test]
    fn claim_post_view_paginates_store() {
        let temp = TempDir::new().expect("temp dir");
        let mut store = SettlementStore::new();
        for index in 0..(STORAGE_VIEW_PAGE_SIZE as u32 + 1) {
            store
                .put_settlement_item(right_item(index))
                .expect("insert settlement item");
        }

        let first_page = store
            .list_settlement(SettlementListReq::all(STORAGE_VIEW_PAGE_SIZE))
            .expect("list first page");
        assert_eq!(first_page.items().len(), STORAGE_VIEW_PAGE_SIZE);
        assert!(first_page.next().is_some());

        let view_root =
            export_claim_post_view(temp.path(), &store).expect("export claim_post view");
        let summary: Value = load_json(view_root.join("summary.json")).expect("claim_post summary");

        assert_eq!(summary["status"].as_str(), Some("ok"));
        assert_eq!(summary["root_match"].as_bool(), Some(true));
        assert_eq!(
            summary["source_check_root_hex"].as_str(),
            summary["view_check_root_hex"].as_str()
        );
    }

    #[test]
    fn claim_post_rejects_stale_store() {
        let temp = TempDir::new().expect("temp dir");
        let mut store = SettlementStore::new();
        for index in 0..(STORAGE_VIEW_PAGE_SIZE as u32 + 1) {
            store
                .put_settlement_item(right_item(index))
                .expect("insert settlement item");
        }

        export_claim_post_view(temp.path(), &store).expect("initial export claim_post view");
        store
            .put_settlement_item(right_item(999))
            .expect("mutate live store after export");

        let err = export_claim_post_view(temp.path(), &store)
            .expect_err("stale persisted claim_post must fail closed");

        assert!(err.contains("claim_post view root drifted"));
    }
}

fn exec_apply_ops(
    store: &SettlementStore,
    exec: &CheckpointExecInput,
) -> Result<Vec<StoreOp>, String> {
    let mut ops = Vec::new();
    for tx in exec.txs() {
        for input in tx.input_refs() {
            let item = store
                .lookup_settlement(SettlementLookup::Terminal(input.terminal_id()))
                .map_err(|e| format!("post_tx resolve failed: {e}"))?
                .ok_or_else(|| {
                    format!(
                        "post_tx input missing {}",
                        hex::encode(input.terminal_id().into_bytes())
                    )
                })?;
            ops.push(StoreOp::Delete(item.path()));
        }
        for output in tx.outputs() {
            let leaf = output.leaf().clone();
            let path = SettlementPath::new(
                output.definition_id(),
                SerialId::new(leaf.serial_id),
                TerminalId::new(leaf.asset_id),
            );
            let item = StoreItem::new(path, leaf)
                .map_err(|e| format!("post_tx output item build failed: {e}"))?;
            ops.push(StoreOp::Put(Box::new(item)));
        }
    }
    Ok(ops)
}

fn save_snapshot(
    view_root: &Path,
    snapshot_id: PrepSnapshotId,
    snapshot: &PrepSnapshot,
) -> Result<(), String> {
    let mut store = PrepFsStore::new(view_root);
    let saved = store.save_snapshot(snapshot).map_err(|e| e.to_string())?;
    if saved != snapshot_id {
        return Err("storage view snapshot id drift".to_string());
    }
    Ok(())
}

fn has_redb(view_root: &Path) -> Result<bool, String> {
    path_exists(view_root.join(REDB_FILE)).map_err(|e| e.to_string())
}

fn root_mix(
    label: &str,
    want: z00z_storage::settlement::CheckRoot,
    got: z00z_storage::settlement::CheckRoot,
) -> Result<SettlementStore, String> {
    Err(format!(
        "{label} root mismatch: want={} got={}",
        hex::encode(want.as_bytes()),
        hex::encode(got.as_bytes())
    ))
}
