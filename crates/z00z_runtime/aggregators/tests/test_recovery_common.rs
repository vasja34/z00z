#![allow(dead_code)]

use std::sync::{Mutex, OnceLock};

use sha2::{Digest, Sha256};
use tempfile::tempdir;

use z00z_aggregators::{
    AggregatorId, BatchId, BatchRoute, PlanDigest, RecoveryBoundary, RejectClass, RouteRangeRule,
    SecondaryState, ShardExecState, ShardExecTicket, ShardId, ShardPlacement, ShardPlacementTable,
    ShardRecoveryRecord, ShardRouteTable,
};
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::{
    checkpoint::{CheckpointDraftId, CheckpointExecOut, CheckpointExecTx, CheckpointInRef},
    settlement::{
        CheckpointPublicationV1, DefinitionId, PolicySetCommitmentV1, PublicationModeTagV1,
        RootGenerationTagV1, SerialId, SettlementExecHandoff, SettlementPath,
        SettlementRecoveryState, SettlementRouteCtx, SettlementStateRoot, SettlementStore,
        ShardRootLeafV1, StoreItem, StoreOp, TerminalId, TerminalLeaf,
    },
};

const REGEN_CMD: &str = "Z00Z_REGEN_DUMP=1 cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage test_manifest_matches_contract -- --exact --nocapture";
const TEST_CMD: &str = "cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage test_manifest_matches_contract -- --exact --nocapture";
const EVIDENCE_PTR: &str = "crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs::test_manifest_matches_contract";
const STORAGE_INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";

fn storage_injection_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FailoverManifest {
    pub version: u32,
    pub regen_command: String,
    pub test_command: String,
    pub evidence_pointer: String,
    pub cases: Vec<FailoverCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FailoverCase {
    pub fixture_id: String,
    pub fixture_class: String,
    pub kind: String,
    pub expected_verdict: String,
    pub expected_reject_class: Option<String>,
    pub expected_detail: Option<String>,
    pub shard_id: u16,
    pub routing_generation: u64,
    pub requester_aggregator_id: u16,
    pub primary_aggregator_id: u16,
    pub secondary_ids: Vec<u16>,
    pub journal_lineage_hex: String,
    pub state_root_hex: String,
    pub root_generation: u8,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_public_root_hexes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carried_forward_leaf_hex: Option<String>,
}

pub struct CarryForwardPublicationCase {
    pub prior: CheckpointPublicationV1,
    pub later: CheckpointPublicationV1,
    pub carried_forward_leaf: ShardRootLeafV1,
    pub later_recovery: SettlementRecoveryState,
}

pub struct DurableCommitPublicationCase {
    pub prior: CheckpointPublicationV1,
    pub later: CheckpointPublicationV1,
    pub later_recovery: SettlementRecoveryState,
}

pub struct RouteMigrationPublicationCase {
    pub prior: CheckpointPublicationV1,
    pub migration_candidate: CheckpointPublicationV1,
    pub record: ShardRecoveryRecord,
    pub live_table: ShardPlacementTable,
    pub recovery: SettlementRecoveryState,
}

pub fn batch_id(label: &str) -> BatchId {
    let digest: [u8; 32] = Sha256::digest(label.as_bytes()).into();
    BatchId::new(CheckpointDraftId::new(digest))
}

pub fn route(shard_id: u16, routing_generation: u64) -> BatchRoute {
    BatchRoute {
        shard_id: z00z_aggregators::ShardId::new(shard_id),
        routing_generation,
    }
}

pub fn placement_table(
    route: BatchRoute,
    primary: AggregatorId,
    secondary: Vec<SecondaryState>,
    journal_lineage: [u8; 32],
) -> ShardPlacementTable {
    let mut table = ShardPlacementTable::default();
    table.insert(ShardPlacement::new(
        route,
        primary,
        secondary,
        journal_lineage,
    ));
    table
}

pub fn recovery_record(
    label: &str,
    route: BatchRoute,
    primary: AggregatorId,
    secondary: Vec<SecondaryState>,
    recovery: SettlementRecoveryState,
) -> ShardRecoveryRecord {
    let placement = ShardPlacement::new(route, primary, secondary, recovery.journal_lineage);
    let ticket = ShardExecTicket {
        batch_id: batch_id(label),
        placement: placement.view(),
        state: ShardExecState::Routed,
    };
    let boundary = RecoveryBoundary;
    let publication = boundary.mark_handed_off(ticket.batch_id);
    boundary
        .capture(&ticket, &publication, recovery)
        .expect("recovery record")
}

pub fn bind_recovery_route(
    recovery: SettlementRecoveryState,
    batch_id: BatchId,
    route: BatchRoute,
    route_table_digest: [u8; 32],
) -> SettlementRecoveryState {
    recovery.with_route(SettlementRouteCtx::new(
        batch_id.into_bytes(),
        route.shard_id.as_u32(),
        route.routing_generation,
        route_table_digest,
    ))
}

pub fn live_recovery_state(
    seed: u8,
) -> Result<SettlementRecoveryState, Box<dyn std::error::Error>> {
    let _guard = storage_injection_lock()
        .lock()
        .expect("storage injection lock");
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    store.apply_settlement_ops(vec![StoreOp::Put(Box::new(item(
        path(seed),
        9_000 + u64::from(seed),
    )))])?;
    Ok(store.recovery_state()?)
}

pub fn route_bound_recovery_state(
    seed: u8,
    batch_id: BatchId,
    route: BatchRoute,
    route_table_digest: [u8; 32],
) -> Result<SettlementRecoveryState, Box<dyn std::error::Error>> {
    let _guard = storage_injection_lock()
        .lock()
        .expect("storage injection lock");
    let temp = tempdir()?;
    let mut store = SettlementStore::load(temp.path())?;
    let spent_path = path(seed);
    let output_path = path(seed.wrapping_add(0x20));
    let output = item(output_path, 9_100 + u64::from(seed));
    store.apply_settlement_ops(vec![StoreOp::Put(Box::new(item(
        spent_path,
        9_000 + u64::from(seed),
    )))])?;
    store.apply_exec_handoff(SettlementExecHandoff::new(
        SettlementRouteCtx::new(
            batch_id.into_bytes(),
            route.shard_id.as_u32(),
            route.routing_generation,
            route_table_digest,
        ),
        vec![
            StoreOp::Delete(spent_path),
            StoreOp::Put(Box::new(output.clone())),
        ],
        vec![exec_handoff_tx(
            spent_path,
            &[output],
            b"route-bound-durable-recovery",
        )],
    ))?;
    Ok(store.recovery_state()?)
}

fn path(seed: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([seed; 32]),
        SerialId::new(u32::from(seed) + 1),
        TerminalId::new([seed.wrapping_add(1); 32]),
    )
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("settlement item")
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

fn exec_handoff_tx(input: SettlementPath, outputs: &[StoreItem], proof: &[u8]) -> CheckpointExecTx {
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

fn publication_route_table(
    routing_generation: u64,
    activation_checkpoint: u64,
    previous_generation_digest: Option<PlanDigest>,
) -> ShardRouteTable {
    ShardRouteTable {
        routing_generation,
        shard_set: vec![ShardId::new(3), ShardId::new(4)],
        rules: vec![
            RouteRangeRule::new([0x00; 32], [0x7f; 32], ShardId::new(3)),
            RouteRangeRule::new([0x80; 32], [0xff; 32], ShardId::new(4)),
        ],
        previous_generation_digest,
        activation_checkpoint,
    }
}

fn publication_policy_set_digest(recovery: &SettlementRecoveryState) -> [u8; 32] {
    PolicySetCommitmentV1::singleton_live(
        u64::from(recovery.bucket_policy_generation),
        recovery.bucket_policy_id,
        recovery.version,
    )
    .digest()
    .expect("policy-set digest")
}

fn publication_leaf(
    shard_id: u32,
    recovery: &SettlementRecoveryState,
    routing_generation: u64,
    route_table_digest: [u8; 32],
    journal_checkpoint: u64,
    shard_epoch: u64,
    local_sequence: u64,
) -> ShardRootLeafV1 {
    ShardRootLeafV1::new(
        shard_id,
        recovery.state_root.into_bytes(),
        shard_epoch,
        routing_generation,
        route_table_digest,
        publication_policy_set_digest(recovery),
        journal_checkpoint,
        local_sequence,
        0,
    )
}

fn publication(
    publication_checkpoint: u64,
    route_table_digest: [u8; 32],
    prior_public_root: SettlementStateRoot,
    shard_leaves: Vec<ShardRootLeafV1>,
) -> Result<CheckpointPublicationV1, Box<dyn std::error::Error>> {
    let publication = CheckpointPublicationV1::new(
        RootGenerationTagV1::RootGeneration1,
        PublicationModeTagV1::CheckpointWindow,
        publication_checkpoint,
        route_table_digest,
        prior_public_root,
        shard_leaves,
    );
    publication.check_contract_v1()?;
    Ok(publication)
}

fn public_root_hex(
    publication: &CheckpointPublicationV1,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(hex::encode(publication.public_root_v1()?.into_bytes()))
}

fn leaf_hex(leaf: &ShardRootLeafV1) -> Result<String, Box<dyn std::error::Error>> {
    Ok(hex::encode(leaf.canonical_bytes()?))
}

pub fn carry_forward_publication_case(
) -> Result<CarryForwardPublicationCase, Box<dyn std::error::Error>> {
    let prior_recovery = live_recovery_state(0x71)?;
    let later_recovery = live_recovery_state(0x72)?;
    let route_table = publication_route_table(9, 21, None);
    let route_digest = route_table.digest().into_bytes();
    let prior_anchor = SettlementStateRoot::settlement_v1([0x51; 32]);

    let carried_forward_leaf = publication_leaf(3, &prior_recovery, 9, route_digest, 21, 5, 9);
    let prior_advanced_leaf = publication_leaf(4, &prior_recovery, 9, route_digest, 21, 5, 10);
    let prior = publication(
        30,
        route_digest,
        prior_anchor,
        vec![carried_forward_leaf, prior_advanced_leaf],
    )?;
    let later_advanced_leaf = publication_leaf(4, &later_recovery, 9, route_digest, 22, 6, 0);
    let later = publication(
        31,
        route_digest,
        prior.public_root_v1()?,
        vec![carried_forward_leaf, later_advanced_leaf],
    )?;

    Ok(CarryForwardPublicationCase {
        prior,
        later,
        carried_forward_leaf,
        later_recovery,
    })
}

pub fn durable_commit_publication_case(
) -> Result<DurableCommitPublicationCase, Box<dyn std::error::Error>> {
    let _guard = storage_injection_lock()
        .lock()
        .expect("storage injection lock");
    let temp = tempdir()?;

    let spent_path = path(0x40);
    let scope_path = path(0x41);
    let sibling_path = path(0x42);
    let route = SettlementRouteCtx::new([0x56; 32], 6, 14, [0x79; 32]);

    let mut seed = SettlementStore::load(temp.path())?;
    seed.put_settlement_item(item(spent_path, 4_001))?;
    let prior_recovery = seed.recovery_state()?;
    drop(seed);

    let scope_item = item(scope_path, 4_102);
    let sibling_item = item(sibling_path, 4_103);
    let ops = vec![
        StoreOp::Delete(spent_path),
        StoreOp::Put(Box::new(scope_item.clone())),
        StoreOp::Put(Box::new(sibling_item.clone())),
    ];
    let txs = vec![exec_handoff_tx(
        spent_path,
        &[scope_item, sibling_item],
        b"crash-after-durable-journal-advance",
    )];

    std::env::set_var(STORAGE_INJ_STAGE_ENV, "parents");
    let mut store = SettlementStore::load(temp.path())?;
    let err = store
        .apply_exec_handoff(SettlementExecHandoff::new(route, ops, txs))
        .expect_err("parent-stage crash injection must fail after durable journal advance");
    assert!(
        err.to_string()
            .contains("hjmt journal injection after ParentsCommitted"),
        "{err}"
    );
    std::env::remove_var(STORAGE_INJ_STAGE_ENV);
    drop(store);

    let recovered = SettlementStore::load(temp.path())?;
    let later_recovery = recovered.recovery_state()?;
    drop(recovered);

    let route_table = publication_route_table(14, 42, None);
    let route_digest = route_table.digest().into_bytes();
    let prior_anchor = SettlementStateRoot::settlement_v1([0x52; 32]);
    let prior = publication(
        40,
        route_digest,
        prior_anchor,
        vec![
            publication_leaf(3, &prior_recovery, 14, route_digest, 40, 7, 1),
            publication_leaf(4, &prior_recovery, 14, route_digest, 40, 7, 2),
        ],
    )?;
    let later = publication(
        41,
        route_digest,
        prior.public_root_v1()?,
        vec![
            publication_leaf(3, &later_recovery, 14, route_digest, 41, 8, 1),
            publication_leaf(4, &later_recovery, 14, route_digest, 41, 8, 2),
        ],
    )?;

    Ok(DurableCommitPublicationCase {
        prior,
        later,
        later_recovery,
    })
}

pub fn route_migration_publication_case(
) -> Result<RouteMigrationPublicationCase, Box<dyn std::error::Error>> {
    let old_route = route(5, 12);
    let new_route = route(5, 13);
    let primary = AggregatorId::new(21);
    let secondary = SecondaryState::ready(AggregatorId::new(22));
    let old_table = publication_route_table(12, 50, None);
    let new_table = publication_route_table(13, 61, Some(old_table.digest()));
    let old_digest = old_table.digest().into_bytes();
    let new_digest = new_table.digest().into_bytes();
    let recovery = route_bound_recovery_state(
        0x91,
        batch_id("route-migration-drift"),
        old_route,
        old_digest,
    )?;
    let record = recovery_record(
        "route-migration-drift",
        old_route,
        primary,
        vec![secondary],
        recovery.clone(),
    );
    let live_table = placement_table(
        new_route,
        primary,
        vec![secondary],
        recovery.journal_lineage,
    );
    let prior = publication(
        50,
        old_digest,
        SettlementStateRoot::settlement_v1([0x53; 32]),
        vec![
            publication_leaf(3, &recovery, 12, old_digest, 50, 9, 1),
            publication_leaf(4, &recovery, 12, old_digest, 50, 9, 2),
        ],
    )?;
    let migration_candidate = publication(
        61,
        new_digest,
        prior.public_root_v1()?,
        vec![
            publication_leaf(3, &recovery, 13, new_digest, 61, 10, 1),
            publication_leaf(4, &recovery, 13, new_digest, 61, 10, 2),
        ],
    )?;

    Ok(RouteMigrationPublicationCase {
        prior,
        migration_candidate,
        record,
        live_table,
        recovery,
    })
}

pub fn live_failover_manifest() -> Result<FailoverManifest, Box<dyn std::error::Error>> {
    let accept_route = route(3, 9);
    let accept_primary = AggregatorId::new(7);
    let accept_secondary = SecondaryState::ready(AggregatorId::new(8));

    let reject_route = route(5, 12);
    let reject_primary = AggregatorId::new(21);
    let reject_secondary = SecondaryState::ready(AggregatorId::new(22));
    let reject_requester = reject_secondary.aggregator_id;
    let accept = route_bound_recovery_state(0x71, batch_id("FOV-001"), accept_route, [0x31; 32])?;
    let reject = route_bound_recovery_state(
        0x81,
        batch_id("route-migration-drift"),
        reject_route,
        [0x41; 32],
    )?;
    let carry_forward = carry_forward_publication_case()?;
    let durable_commit = durable_commit_publication_case()?;
    let migration = route_migration_publication_case()?;

    Ok(FailoverManifest {
        version: 1,
        regen_command: REGEN_CMD.to_string(),
        test_command: TEST_CMD.to_string(),
        evidence_pointer: EVIDENCE_PTR.to_string(),
        cases: vec![
            manifest_case(
                "FOV-001",
                "Failover fixture",
                "same-lineage takeover",
                "accept",
                None,
                None,
                accept_route,
                accept_primary,
                accept_secondary.aggregator_id,
                vec![accept_secondary.aggregator_id],
                &accept,
            ),
            manifest_case(
                "FOV-T-001",
                "Failover fixture",
                "wrong lineage",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("wrong lineage"),
                reject_route,
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case(
                "FOV-T-001",
                "Failover fixture",
                "wrong generation",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("wrong generation"),
                reject_route,
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case(
                "FOV-T-001",
                "Failover fixture",
                "stale local root",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("stale local root"),
                reject_route,
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case(
                "FOV-T-001",
                "Failover fixture",
                "stale restart",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("stale restart"),
                reject_route,
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case(
                "FOV-T-002",
                "Failover fixture",
                "secondary aggregator down",
                "recovery_reject",
                Some(RejectClass::DeferredRetry),
                Some("secondary aggregator down"),
                reject_route,
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case(
                "FOV-T-002",
                "Failover fixture",
                "split-brain",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("split-brain"),
                reject_route,
                reject_primary,
                reject_primary,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case(
                "FOV-T-002",
                "Route migration fixture",
                "route migration during crash",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("wrong generation"),
                reject_route,
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &reject,
            ),
            manifest_case_with_publication(
                "FOV-G-002",
                "Carry-forward fixture",
                "failed-shard carry-forward publication accepts",
                "accept",
                None,
                None,
                accept_route,
                accept_primary,
                accept_secondary.aggregator_id,
                vec![accept_secondary.aggregator_id],
                &carry_forward.later_recovery,
                vec![public_root_hex(&carry_forward.later)?],
                Some(leaf_hex(&carry_forward.carried_forward_leaf)?),
            ),
            manifest_case_with_publication(
                "FOV-G-003",
                "Crash fixture",
                "crash after durable commit before publication recovers lawfully",
                "accept",
                None,
                None,
                route(6, 14),
                accept_primary,
                accept_secondary.aggregator_id,
                vec![accept_secondary.aggregator_id],
                &durable_commit.later_recovery,
                vec![
                    public_root_hex(&durable_commit.prior)?,
                    public_root_hex(&durable_commit.later)?,
                ],
                None,
            ),
            manifest_case_with_publication(
                "FOV-G-004",
                "Route migration fixture",
                "crash during route migration resolves lawfully",
                "recovery_reject",
                Some(RejectClass::PolicyReject),
                Some("wrong generation"),
                route(5, 12),
                reject_primary,
                reject_requester,
                vec![reject_secondary.aggregator_id],
                &migration.recovery,
                vec![public_root_hex(&migration.prior)?],
                None,
            ),
        ],
    })
}

fn manifest_case(
    fixture_id: &str,
    fixture_class: &str,
    kind: &str,
    expected_verdict: &str,
    expected_reject_class: Option<RejectClass>,
    expected_detail: Option<&str>,
    route: BatchRoute,
    primary: AggregatorId,
    requester: AggregatorId,
    secondary_ids: Vec<AggregatorId>,
    recovery: &SettlementRecoveryState,
) -> FailoverCase {
    FailoverCase {
        fixture_id: fixture_id.to_string(),
        fixture_class: fixture_class.to_string(),
        kind: kind.to_string(),
        expected_verdict: expected_verdict.to_string(),
        expected_reject_class: expected_reject_class.map(|class| format!("{class:?}")),
        expected_detail: expected_detail.map(str::to_string),
        shard_id: route.shard_id.as_u16(),
        routing_generation: route.routing_generation,
        requester_aggregator_id: requester.as_u16(),
        primary_aggregator_id: primary.as_u16(),
        secondary_ids: secondary_ids
            .into_iter()
            .map(|secondary| secondary.as_u16())
            .collect(),
        journal_lineage_hex: hex::encode(recovery.journal_lineage),
        state_root_hex: hex::encode(recovery.state_root.into_bytes()),
        root_generation: recovery.root_generation,
        expected_public_root_hexes: Vec::new(),
        carried_forward_leaf_hex: None,
    }
}

fn manifest_case_with_publication(
    fixture_id: &str,
    fixture_class: &str,
    kind: &str,
    expected_verdict: &str,
    expected_reject_class: Option<RejectClass>,
    expected_detail: Option<&str>,
    route: BatchRoute,
    primary: AggregatorId,
    requester: AggregatorId,
    secondary_ids: Vec<AggregatorId>,
    recovery: &SettlementRecoveryState,
    expected_public_root_hexes: Vec<String>,
    carried_forward_leaf_hex: Option<String>,
) -> FailoverCase {
    let mut case = manifest_case(
        fixture_id,
        fixture_class,
        kind,
        expected_verdict,
        expected_reject_class,
        expected_detail,
        route,
        primary,
        requester,
        secondary_ids,
        recovery,
    );
    case.expected_public_root_hexes = expected_public_root_hexes;
    case.carried_forward_leaf_hex = carried_forward_leaf_hex;
    case
}
