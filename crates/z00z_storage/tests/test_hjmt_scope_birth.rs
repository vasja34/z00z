use std::sync::{Mutex, OnceLock};

use tempfile::tempdir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::{expert::encoding::to_hex, ZkPackEncrypted};
use z00z_storage::{
    checkpoint::{CheckpointExecOut, CheckpointExecTx, CheckpointInRef},
    settlement::{
        DefinitionId, RightClass, RightLeaf, ScopeFlow, ScopeLeafKind, ScopeOpKind, SerialId,
        SettlementExecHandoff, SettlementLeaf, SettlementPath, SettlementRouteCtx, SettlementStore,
        SettlementStoreError, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    },
};
use z00z_utils::io::{read_to_string, save_json};

const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    backend_mode: Option<String>,
}

impl EnvGuard {
    fn live() -> Self {
        let guard = Self {
            backend_mode: std::env::var(BACKEND_ENV).ok(),
        };
        std::env::set_var(BACKEND_ENV, "hjmt");
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(mode) = &self.backend_mode {
            std::env::set_var(BACKEND_ENV, mode);
        } else {
            std::env::remove_var(BACKEND_ENV);
        }
    }
}

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(definition)),
        SerialId::new(serial),
        TerminalId::new(bytes(asset)),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3u8; 32],
        s_out: [4u8; 32],
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1u8; 32],
        owner_tag: [2u8; 32],
        c_amount: [5u8; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn term_item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("terminal store item")
}

fn right_leaf(path: SettlementPath, mark: u8) -> RightLeaf {
    RightLeaf {
        version: 1,
        terminal_id: path.terminal_id(),
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
        use_nonce: bytes(mark.wrapping_add(7)),
        revocation_policy_id: bytes(mark.wrapping_add(8)),
        transition_policy_id: bytes(mark.wrapping_add(9)),
        challenge_policy_id: bytes(mark.wrapping_add(10)),
        disclosure_policy_id: bytes(mark.wrapping_add(11)),
        retention_policy_id: bytes(mark.wrapping_add(12)),
    }
}

fn right_item(path: SettlementPath, mark: u8) -> StoreItem {
    StoreItem::new(path, right_leaf(path, mark)).expect("right store item")
}

fn route_ctx(batch: u8, shard_id: u32, routing_generation: u64, digest: u8) -> SettlementRouteCtx {
    SettlementRouteCtx::new(bytes(batch), shard_id, routing_generation, bytes(digest))
}

fn exec_tx(input: SettlementPath, outputs: &[StoreItem], proof: &[u8]) -> CheckpointExecTx {
    let outputs = outputs
        .iter()
        .map(|item| {
            CheckpointExecOut::new(
                item.path().definition_id,
                item.terminal_leaf().expect("terminal output").clone(),
            )
            .expect("exec out")
        })
        .collect();

    CheckpointExecTx::new(
        vec![CheckpointInRef::new(
            input.terminal_id().into_bytes(),
            input.serial_id,
        )],
        outputs,
        proof.to_vec(),
    )
    .expect("exec tx")
}

fn flow_item(flow: &ScopeFlow, path: SettlementPath) -> &z00z_storage::settlement::ScopeFlowItem {
    let terminal_id = to_hex(&path.terminal_id().into_bytes());
    flow.items
        .iter()
        .find(|item| item.terminal_id == terminal_id)
        .unwrap_or_else(|| panic!("missing flow row for terminal {}", terminal_id))
}

fn assert_exec_mismatch(err: SettlementStoreError) {
    match err {
        SettlementStoreError::Backend(detail) => {
            assert!(detail.contains("checkpoint exec does not match store ops"));
        }
        other => panic!("expected checkpoint mismatch, got {other:?}"),
    }
}

fn env_guard() -> std::sync::MutexGuard<'static, ()> {
    env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[test]
fn test_records_birth_reload() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_guard();
    let _env = EnvGuard::live();
    let root = tempdir()?;
    let mut store = SettlementStore::load(root.path())?;

    let spent_path = path(1, 1, 1);
    let existing_path = path(9, 9, 9);
    store.put_settlement_item(term_item(spent_path, 10))?;
    store.put_settlement_item(term_item(existing_path, 20))?;

    let new_scope_path = path(2, 2, 2);
    let sibling_scope_path = path(2, 2, 3);
    let mixed_path = path(9, 9, 10);
    let new_scope_item = term_item(new_scope_path, 30);
    let sibling_scope_item = term_item(sibling_scope_path, 35);
    let mixed_item = term_item(mixed_path, 40);
    let ops = vec![
        StoreOp::Delete(spent_path),
        StoreOp::Put(Box::new(new_scope_item.clone())),
        StoreOp::Put(Box::new(sibling_scope_item.clone())),
        StoreOp::Put(Box::new(mixed_item.clone())),
    ];
    let txs = vec![exec_tx(
        spent_path,
        &[
            new_scope_item.clone(),
            sibling_scope_item.clone(),
            mixed_item.clone(),
        ],
        b"scope-birth-1",
    )];

    let flow = store.apply_exec_handoff(SettlementExecHandoff::new(
        route_ctx(0x11, 3, 7, 0x44),
        ops,
        txs,
    ))?;
    let new_row = flow_item(&flow, new_scope_path);
    let sibling_row = flow_item(&flow, sibling_scope_path);
    let mixed_row = flow_item(&flow, mixed_path);

    assert_eq!(flow.shard_id, 3);
    assert_eq!(flow.routing_generation, 7);
    assert_eq!(new_row.op_kind, ScopeOpKind::Put);
    assert_eq!(new_row.leaf_family, ScopeLeafKind::Terminal);
    assert!(new_row.first_seen.definition);
    assert!(new_row.first_seen.serial);
    assert!(new_row.first_seen.object);
    assert_eq!(sibling_row.leaf_family, ScopeLeafKind::Terminal);
    assert!(!sibling_row.first_seen.definition);
    assert!(!sibling_row.first_seen.serial);
    assert!(sibling_row.first_seen.object);
    assert_eq!(mixed_row.leaf_family, ScopeLeafKind::Terminal);
    assert!(!mixed_row.first_seen.definition);
    assert!(!mixed_row.first_seen.serial);
    assert!(mixed_row.first_seen.object);
    assert_ne!(flow.root_flow.prev_root, flow.root_flow.post_root);

    let scope_path = root.path().join("scope_flow.json");
    save_json(&scope_path, &flow)?;
    let json = read_to_string(&scope_path)?;
    for field in [
        "\"batch_id\"",
        "\"tx_id\"",
        "\"shard_id\"",
        "\"routing_generation\"",
        "\"route_table_digest\"",
        "\"definition_id\"",
        "\"serial_id\"",
        "\"leaf_family\"",
        "\"first_seen\"",
        "\"post_root\"",
    ] {
        assert!(json.contains(field), "scope_flow.json missing {field}");
    }
    assert!(!json.contains("HjmtTreeId"));

    let post_root = store.settlement_root()?;
    drop(store);

    let mut reloaded = SettlementStore::load(root.path())?;
    assert_eq!(reloaded.settlement_root()?, post_root);
    let recovery = reloaded.recovery_state()?;
    assert_eq!(recovery.route, Some(route_ctx(0x11, 3, 7, 0x44)));
    assert_eq!(
        reloaded
            .get_settlement_item(&new_scope_path)?
            .expect("new scope item")
            .path(),
        new_scope_path
    );

    let follow_up_path = path(2, 2, 4);
    let follow_up_item = term_item(follow_up_path, 50);
    let replay_flow = reloaded.apply_exec_handoff(SettlementExecHandoff::new(
        route_ctx(0x12, 3, 7, 0x44),
        vec![
            StoreOp::Delete(mixed_path),
            StoreOp::Put(Box::new(follow_up_item.clone())),
        ],
        vec![exec_tx(mixed_path, &[follow_up_item], b"scope-birth-2")],
    ))?;
    let follow_row = flow_item(&replay_flow, follow_up_path);
    assert_eq!(follow_row.leaf_family, ScopeLeafKind::Terminal);
    assert!(!follow_row.first_seen.definition);
    assert!(!follow_row.first_seen.serial);
    assert!(follow_row.first_seen.object);

    Ok(())
}

#[test]
fn test_records_first_right() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_guard();
    let _env = EnvGuard::live();
    let mut store = SettlementStore::new();
    let base_path = path(7, 7, 1);
    store.put_settlement_item(term_item(base_path, 70))?;

    let right_path = path(7, 7, 2);
    let right_item = right_item(right_path, 81);
    let flow = store.apply_exec_handoff(SettlementExecHandoff::new(
        route_ctx(0x21, 7, 13, 0x55),
        vec![StoreOp::Put(Box::new(right_item.clone()))],
        Vec::new(),
    ))?;
    let row = flow_item(&flow, right_path);

    assert_eq!(row.op_kind, ScopeOpKind::Put);
    assert_eq!(row.leaf_family, ScopeLeafKind::Right);
    assert!(!row.first_seen.definition);
    assert!(!row.first_seen.serial);
    assert!(row.first_seen.object);
    assert!(matches!(
        store
            .get_settlement_item(&right_path)?
            .expect("right item present")
            .leaf(),
        SettlementLeaf::Right(_)
    ));

    Ok(())
}

#[test]
fn test_rejects_duplicate_terminal() {
    let _guard = env_guard();
    let _env = EnvGuard::live();
    let mut store = SettlementStore::new();
    let terminal_id = TerminalId::new(bytes(91));
    let left_path =
        SettlementPath::new(DefinitionId::new(bytes(31)), SerialId::new(1), terminal_id);
    let right_path =
        SettlementPath::new(DefinitionId::new(bytes(32)), SerialId::new(2), terminal_id);
    let err = store
        .apply_exec_handoff(SettlementExecHandoff::new(
            route_ctx(0x31, 2, 5, 0x66),
            vec![
                StoreOp::Put(Box::new(right_item(left_path, 91))),
                StoreOp::Put(Box::new(right_item(right_path, 92))),
            ],
            Vec::new(),
        ))
        .expect_err("duplicate terminal id must reject");

    assert!(matches!(err, SettlementStoreError::PathTerminalMix));
}

#[test]
fn test_handoff_rejects_mixed_families() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_guard();
    let _env = EnvGuard::live();
    let mut store = SettlementStore::new();
    let spent_path = path(8, 8, 8);
    store.put_settlement_item(term_item(spent_path, 11))?;
    let term_path = path(8, 8, 9);
    let term_item = term_item(term_path, 22);
    let right_path = path(8, 8, 10);
    let err = store
        .apply_exec_handoff(SettlementExecHandoff::new(
            route_ctx(0x33, 2, 5, 0x66),
            vec![
                StoreOp::Delete(spent_path),
                StoreOp::Put(Box::new(term_item.clone())),
                StoreOp::Put(Box::new(right_item(right_path, 91))),
            ],
            vec![exec_tx(spent_path, &[term_item], b"mixed-families")],
        ))
        .expect_err("mixed checkpoint and right ops must reject");

    match err {
        SettlementStoreError::Backend(detail) => {
            assert!(detail.contains("cannot mix terminal and non-terminal"));
        }
        other => panic!("expected mixed-family backend error, got {other:?}"),
    }

    Ok(())
}

#[test]
fn test_handoff_rejects_orphan_exec() {
    let _guard = env_guard();
    let _env = EnvGuard::live();
    let mut store = SettlementStore::new();
    let path = path(8, 8, 8);
    let item = term_item(path, 11);
    let err = store
        .apply_exec_handoff(SettlementExecHandoff::new(
            route_ctx(0x34, 2, 5, 0x66),
            Vec::new(),
            vec![exec_tx(path, &[item], b"orphan-exec")],
        ))
        .expect_err("orphan exec rows must reject");

    match err {
        SettlementStoreError::Backend(detail) => {
            assert!(detail.contains("checkpoint exec rows require terminal settlement ops"));
        }
        other => panic!("expected orphan-exec backend error, got {other:?}"),
    }
}

#[test]
fn test_rejects_path_leaf_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_guard();
    let _env = EnvGuard::live();
    let mut path_store = SettlementStore::new();
    let spent_path = path(4, 4, 4);
    path_store.put_settlement_item(term_item(spent_path, 11))?;
    let target_path = path(5, 5, 5);
    let good_item = term_item(target_path, 22);
    let wrong_path_item = term_item(path(6, 5, 5), 22);
    let path_err = path_store
        .apply_exec_handoff(SettlementExecHandoff::new(
            route_ctx(0x41, 4, 9, 0x77),
            vec![
                StoreOp::Delete(spent_path),
                StoreOp::Put(Box::new(good_item.clone())),
            ],
            vec![exec_tx(spent_path, &[wrong_path_item], b"path-mismatch")],
        ))
        .expect_err("wrong definition must reject");
    assert_exec_mismatch(path_err);
    assert!(path_store.get_settlement_item(&spent_path)?.is_some());

    let mut leaf_store = SettlementStore::new();
    leaf_store.put_settlement_item(term_item(spent_path, 11))?;
    let leaf_err = leaf_store
        .apply_exec_handoff(SettlementExecHandoff::new(
            route_ctx(0x42, 4, 9, 0x77),
            vec![
                StoreOp::Delete(spent_path),
                StoreOp::Put(Box::new(good_item.clone())),
            ],
            vec![exec_tx(
                spent_path,
                &[term_item(target_path, 23)],
                b"leaf-mismatch",
            )],
        ))
        .expect_err("wrong leaf hash must reject");
    assert_exec_mismatch(leaf_err);
    assert!(leaf_store.get_settlement_item(&spent_path)?.is_some());

    Ok(())
}
