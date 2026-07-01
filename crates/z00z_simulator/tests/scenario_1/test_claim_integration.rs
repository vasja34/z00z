use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

#[cfg(feature = "wallet_debug_tools")]
use std::collections::HashSet;
use tempfile::TempDir;
#[cfg(not(feature = "wallet_debug_tools"))]
use z00z_core::{assets::Asset, AssetDefinitionRegistry, ChainType};
#[cfg(feature = "wallet_debug_tools")]
use z00z_core::{
    assets::{Asset, AssetClass},
    AssetDefinitionRegistry, ChainType,
};
#[cfg(feature = "wallet_debug_tools")]
use z00z_crypto::{
    verify_range_proof, KernelSignature, AGGREGATION_FACTOR, MIN_VALUE_PROMISE, RANGE_PROOF_BITS,
};
use z00z_simulator::{
    config::ScenarioCfg,
    design::DesignDoc,
    scenario_1::{stage_1, stage_2, stage_3},
    DesignStage, SimContext, StageResult,
};
use z00z_utils::{
    codec::{Codec, YamlCodec},
    io::{load_bincode, write_file},
    logger::NoopLogger,
    metrics::NoopMetrics,
    time::{SystemTimeProvider, TimeProvider},
};

use z00z_simulator::scenario_1::{
    support::claim_shared_cases, support::fixture_cache, support::stage_runner_support,
};

#[derive(serde::Deserialize)]
struct ClaimRow {
    class: String,
    amount: u64,
}

#[derive(serde::Deserialize)]
struct ClaimGenesisEvent {
    scenario_id: u32,
    stage: u32,
    distributed: usize,
    input_assets: usize,
    actor_claims: Vec<serde_json::Value>,
}

#[cfg(feature = "wallet_debug_tools")]
#[derive(serde::Deserialize)]
struct WalletDebugDump {
    imported_assets_full: Vec<Asset>,
}

static S3_OUT: OnceLock<PathBuf> = OnceLock::new();

fn stage3_out() -> &'static PathBuf {
    S3_OUT.get_or_init(claim_shared_cases::default_stage3_out)
}

#[cfg(feature = "wallet_debug_tools")]
fn load_actor_assets(out: &Path, name: &str) -> Vec<Asset> {
    let path = out.join(format!("claim/export_wallet_debug_{name}.json"));
    let dump: WalletDebugDump =
        z00z_utils::io::load_json_bounded(&path, 64 * 1024 * 1024).expect("wallet debug dump");
    dump.imported_assets_full
}

fn mk_cfg_with_consume_in(base: &Path, consume: bool) -> (PathBuf, PathBuf, PathBuf) {
    let out = base.join("outputs/scenario_1");
    let mut cfg = ScenarioCfg::from_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_config.yaml"),
    )
    .expect("load scenario config");

    cfg.stage1_genesis
        .get_or_insert_with(Default::default)
        .genesis_config = z00z_core::config_paths::devnet_genesis_path()
        .to_string_lossy()
        .to_string();
    cfg.outputs.dir = out.to_string_lossy().to_string();

    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(consume);
    }

    let cfg_path = base.join("scenario_config.yaml");
    let cfg_bytes = YamlCodec.serialize(&cfg).expect("serialize cfg");
    write_file(&cfg_path, &cfg_bytes).expect("write cfg");

    let design_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scenario_1/scenario_design.yaml");
    (cfg_path, design_path, out)
}

fn mk_cfg_with_consume(consume: bool) -> (PathBuf, PathBuf, PathBuf) {
    let temp = TempDir::new().expect("temp dir");
    let base = temp.keep();
    mk_cfg_with_consume_in(&base, consume)
}

fn stage_by_id(design_path: &Path, stage_id: u32) -> DesignStage {
    let doc = DesignDoc::from_file(design_path).expect("load design");
    doc.stages
        .iter()
        .find(|item| item.stage == stage_id)
        .cloned()
        .expect("stage exists")
}

fn mk_ctx(cfg_path: &Path) -> SimContext {
    let cfg = ScenarioCfg::from_file(cfg_path).expect("load scenario cfg");
    let chain = cfg
        .chain
        .parse::<ChainType>()
        .unwrap_or_else(|err| panic!("invalid simulator chain '{}': {err}", cfg.chain));
    let out_dir = PathBuf::from(cfg.outputs.dir.clone());
    let reg = AssetDefinitionRegistry::new(
        Arc::new(NoopLogger),
        Arc::new(NoopMetrics),
        Arc::new(SystemTimeProvider),
    );

    SimContext {
        config: cfg,
        chain_type: chain,
        registry: reg,
        assets: Vec::new(),
        genesis_rights: Vec::new(),
        actors: Vec::new(),
        leaves: Vec::new(),
        block_height: 0,
        outputs_dir: out_dir,
        logger: Arc::new(NoopLogger),
        wallet_service: None,
    }
}

#[cfg(feature = "wallet_debug_tools")]
fn load_gen_assets(out: &Path) -> Vec<Asset> {
    let mut all = Vec::new();
    for path in list_gen_bins(out) {
        let mut part: Vec<Asset> = load_bincode(&path).expect("load genesis assets");
        all.append(&mut part);
    }
    all
}

fn list_gen_bins(out: &Path) -> Vec<PathBuf> {
    let base = out.join("genesis");
    let mut files = Vec::new();
    let entries = std::fs::read_dir(&base).expect("read genesis dir");

    for entry in entries {
        let path = entry.expect("read dir entry").path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|item| item.to_str()) else {
            continue;
        };
        if name.starts_with("genesis_") && name.ends_with(".bin") {
            files.push(path);
        }
    }

    files.sort();
    assert!(!files.is_empty(), "no genesis_*.bin files found");
    files
}

fn run_stage12(cfg_path: &Path, design_path: &Path) -> SimContext {
    let _lock = stage_runner_support::acquire_process_lock();
    let mut ctx = mk_ctx(cfg_path);
    let stage1 = stage_by_id(design_path, 1);
    let stage2 = stage_by_id(design_path, 2);

    let stage1_res = stage_1::run(&mut ctx, &stage1);
    assert!(matches!(stage1_res, StageResult::Ok), "stage 1 failed");
    let stage2_res = stage_2::run(&mut ctx, &stage2);
    assert!(matches!(stage2_res, StageResult::Ok), "stage 2 failed");
    ctx
}

fn consume_case() -> &'static PathBuf {
    static CASE: OnceLock<PathBuf> = OnceLock::new();
    CASE.get_or_init(claim_shared_cases::consume_stage3_out)
}

fn class_split_case() -> &'static PathBuf {
    static CASE: OnceLock<PathBuf> = OnceLock::new();
    CASE.get_or_init(claim_shared_cases::class_split_stage3_out)
}

#[test]
fn test_stage3_bins_post_consume() {
    let out = consume_case();

    for path in list_gen_bins(out) {
        let file = path
            .file_name()
            .and_then(|item| item.to_str())
            .expect("bin file name");
        assert!(path.exists(), "{file} not found after run");
        let assets: Vec<Asset> = load_bincode(&path).expect("load bincode assets");
        assert!(
            assets.is_empty(),
            "{file} must be empty after consume_bins=true"
        );
    }
}

#[test]
fn test_stage3_double_claim_rejected() {
    let (cfg_path, design_path, out) = mk_cfg_with_consume(true);
    fixture_cache::copy_tree(consume_case(), &out);

    for path in list_gen_bins(&out) {
        let file = path
            .file_name()
            .and_then(|item| item.to_str())
            .expect("bin file name");
        let assets: Vec<Asset> = load_bincode(&path).expect("load bincode assets");
        assert!(
            assets.is_empty(),
            "{file} must be empty before second stage3"
        );
    }

    let _lock = stage_runner_support::acquire_process_lock();
    let mut ctx = mk_ctx(&cfg_path);
    let stage2 = stage_by_id(&design_path, 2);
    let stage3 = stage_by_id(&design_path, 3);

    let stage2_res = stage_2::run(&mut ctx, &stage2);
    assert!(matches!(stage2_res, StageResult::Ok), "stage 2 prep failed");

    let claim_res = stage_3::run_claim_genesis(&mut ctx, &stage3);
    assert!(
        matches!(claim_res, StageResult::Fail(_)),
        "second Stage 3 must be rejected"
    );
}

#[test]
fn test_stage3_class_split_correct() {
    let out = class_split_case();

    let alice_path = out.join("claim/claim_rows_alice.json");
    assert!(alice_path.exists(), "alice claim file missing");
    let alice_rows: Vec<ClaimRow> = z00z_utils::io::load_json(&alice_path).expect("alice rows");
    assert!(!alice_rows.is_empty(), "alice must have assets");
    assert!(
        alice_rows.iter().all(|row| row.class == "Coin"),
        "alice must get only Coin"
    );

    let bob_path = out.join("claim/claim_rows_bob.json");
    assert!(bob_path.exists(), "bob claim file missing");
    let bob_rows: Vec<ClaimRow> = z00z_utils::io::load_json(&bob_path).expect("bob rows");
    assert!(!bob_rows.is_empty(), "bob must have assets");
    assert!(
        bob_rows.iter().all(|row| row.class == "Nft"),
        "bob must get only Nft"
    );

    let charlie_path = out.join("claim/claim_rows_charlie.json");
    assert!(charlie_path.exists(), "charlie claim file missing");
    let charlie_rows: Vec<ClaimRow> =
        z00z_utils::io::load_json(&charlie_path).expect("charlie rows");
    assert!(!charlie_rows.is_empty(), "charlie must have assets");
    assert!(
        charlie_rows
            .iter()
            .all(|row| row.class == "Token" || row.class == "Void"),
        "charlie must get only Token or Void"
    );
}

#[test]
fn test_stage3_claim_event_valid() {
    let path = stage3_out().join("events/claim_genesis.event.json");
    assert!(path.exists(), "claim_genesis.event.json not found");

    let evt: ClaimGenesisEvent = z00z_utils::io::load_json(&path).expect("claim event");
    assert_eq!(evt.scenario_id, 1, "scenario_id must be 1");
    assert_eq!(evt.stage, 3, "stage must be 3");
    assert_eq!(
        evt.distributed, evt.input_assets,
        "distributed must equal input_assets (no assets lost)"
    );
    assert_eq!(evt.actor_claims.len(), 3, "exactly 3 actor claims expected");
}

#[test]
fn test_stage3_snapshot_counts_consistent() {
    let path = stage3_out().join("stage_3_snapshot.json");
    assert!(path.exists(), "stage_3_snapshot.json not found");

    let snap: serde_json::Value = z00z_utils::io::load_json(&path).expect("snapshot");
    let dist = snap["distributed_assets_count"]
        .as_u64()
        .expect("distributed_assets_count missing") as usize;
    let claims = snap["actor_claims"]
        .as_array()
        .expect("actor_claims must be array");
    let actor_sum: usize = claims
        .iter()
        .map(|item| item["assets_count"].as_u64().expect("assets_count missing") as usize)
        .sum();

    assert_eq!(
        actor_sum, dist,
        "actor_claims.sum ({actor_sum}) != distributed_assets_count ({dist})"
    );
    assert_eq!(claims.len(), 3, "must have exactly 3 actor claim entries");
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_stage3_claimed_nonces_unique() {
    let mut seen: HashSet<[u8; 32]> = HashSet::new();
    for name in ["alice", "bob", "charlie"] {
        let items = load_actor_assets(stage3_out(), name);
        assert!(!items.is_empty(), "{name}: no claimed assets found");

        for item in &items {
            let nonce: [u8; 32] = item.nonce;
            assert!(
                seen.insert(nonce),
                "nonce collision for {name} at serial {}",
                item.serial_id
            );
        }
    }
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_stage3_non_amounts_positive() {
    for name in ["alice", "bob", "charlie"] {
        let items = load_actor_assets(stage3_out(), name);
        let mut non_void_count = 0usize;

        for item in &items {
            if item.definition.class == AssetClass::Void {
                continue;
            }

            non_void_count += 1;
            assert!(
                item.amount > 0,
                "{name}: zero amount in {:?} asset id={:?}",
                item.definition.class,
                item.definition.id
            );
        }

        assert!(
            non_void_count > 0,
            "{name}: expected at least one non-void asset"
        );
    }
}

#[test]
fn test_stage3_snapshot_json_balance() {
    let snap: serde_json::Value =
        z00z_utils::io::load_json(stage3_out().join("stage_3_snapshot.json")).expect("snapshot");
    let claims = snap["actor_claims"]
        .as_array()
        .expect("actor_claims must be array");

    for claim in claims {
        let name = claim["name"]
            .as_str()
            .expect("actor name missing")
            .to_lowercase();
        let snap_total = claim["total_amount"]
            .as_u64()
            .expect("total_amount field missing");

        let rows: Vec<ClaimRow> =
            z00z_utils::io::load_json(stage3_out().join(format!("claim/claim_rows_{name}.json")))
                .expect("claimed rows");
        let json_total: u64 = rows.iter().map(|row| row.amount).sum();
        assert_eq!(
            snap_total, json_total,
            "{name}: snapshot total_amount ({snap_total}) != JSON sum ({json_total})"
        );
    }
}

#[test]
fn test_stage3_import_no_rejections() {
    let snap: serde_json::Value =
        z00z_utils::io::load_json(stage3_out().join("stage_3_snapshot.json")).expect("snapshot");
    let claims = snap["actor_claims"]
        .as_array()
        .expect("actor_claims must be array");
    let stats = snap["wallet_import_stats"]
        .as_array()
        .expect("wallet_import_stats field missing");

    assert_eq!(
        stats.len(),
        3,
        "must have 3 import stat entries (one per actor)"
    );

    for stat in stats {
        let actor = stat["actor"].as_str().unwrap_or("unknown");
        let rejected = stat["rejected"].as_u64().expect("rejected field missing");
        assert_eq!(rejected, 0, "actor {actor} has {rejected} rejected imports");

        let inserted = stat["inserted"].as_u64().unwrap_or(0);
        let already_exists = stat["already_exists"].as_u64().unwrap_or(0);
        let assets_count = claims
            .iter()
            .find(|row| {
                row["name"]
                    .as_str()
                    .map(|n| n.eq_ignore_ascii_case(actor))
                    .unwrap_or(false)
            })
            .and_then(|row| row["assets_count"].as_u64())
            .expect("assets_count for actor missing");
        assert_eq!(
            inserted + already_exists,
            assets_count,
            "actor {actor}: inserted + already_exists must equal assets_count"
        );
    }
}

#[test]
fn test_stage3_resume_artifacts_written() {
    let _lock = stage_runner_support::acquire_process_lock();
    let (cfg_path, design_path, out) = mk_cfg_with_consume(false);
    let mut ctx = run_stage12(&cfg_path, &design_path);
    let stage3 = stage_by_id(&design_path, 3);

    let state = stage_3::ClaimStateFile {
        run_id: "uniform_all|mock:42|consume=false".to_string(),
        mode: "uniform_all".to_string(),
        rng_kind: "mock:42".to_string(),
        step: stage_3::ClaimStep::ArtifactsWritten,
        started_at_unix: z00z_utils::time::SystemTimeProvider.compat_unix_timestamp(),
        claimed_rows: Vec::new(),
    };
    let claim_state_path = out.join("genesis/claim_state.json");
    z00z_utils::io::save_json(&claim_state_path, &state).expect("write claim_state");

    let partial_rows = vec![serde_json::json!({
        "asset_id": "00",
        "symbol": "Z00Z",
        "class": "Coin",
        "serial_id": 1,
        "amount": 1
    })];
    let partial_path = out.join("claim/claim_rows_alice.json");
    z00z_utils::io::save_json(&partial_path, &partial_rows).expect("write partial claimed rows");

    let result = stage_3::run_claim_genesis(&mut ctx, &stage3);
    match result {
        StageResult::Ok => {
            let s1: serde_json::Value =
                z00z_utils::io::load_json(out.join("stage_1_snapshot.json"))
                    .expect("stage 1 snapshot");
            let s3: serde_json::Value =
                z00z_utils::io::load_json(out.join("stage_3_snapshot.json"))
                    .expect("stage 3 snapshot");
            let s1_count = s1["assets_count"]
                .as_u64()
                .expect("s1.assets_count missing");
            let s3_count = s3["distributed_assets_count"]
                .as_u64()
                .expect("s3.distributed_assets_count missing");
            assert_eq!(
                s1_count, s3_count,
                "resume must not duplicate assets: s1={s1_count} s3={s3_count}"
            );
        }
        StageResult::Fail(err) => {
            let msg = err.to_ascii_lowercase();
            assert!(
                !msg.contains("panic") && !msg.contains("unwrap"),
                "error must be descriptive and non-panic: {err}"
            );
        }
        StageResult::Warn(warn) => {
            let msg = warn.to_ascii_lowercase();
            assert!(
                !msg.contains("panic") && !msg.contains("unwrap"),
                "warning must be descriptive and non-panic: {warn}"
            );
        }
    }
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_neg_tampered_proof_rejected() {
    let items = load_actor_assets(stage3_out(), "alice");
    assert!(!items.is_empty(), "alice has no claimed assets");

    let base = items
        .iter()
        .find(|asset| asset.range_proof.is_some())
        .expect("no asset with range proof")
        .clone();
    assert!(
        base.verify_complete().is_ok(),
        "baseline asset must be cryptographically valid"
    );

    let mut item = base;

    if let Some(ref mut proof) = item.range_proof {
        for byte in proof.iter_mut().take(16) {
            *byte = 0x00;
        }
    }

    let proof = item
        .range_proof
        .as_ref()
        .expect("range proof must exist for tamper test");
    let result = verify_range_proof(
        proof,
        &item.commitment,
        RANGE_PROOF_BITS,
        AGGREGATION_FACTOR,
        MIN_VALUE_PROMISE,
    );
    assert!(result.is_err(), "tampered range proof must be rejected");

    let empty: Vec<u8> = Vec::new();
    assert!(
        verify_range_proof(
            &empty,
            &item.commitment,
            RANGE_PROOF_BITS,
            AGGREGATION_FACTOR,
            MIN_VALUE_PROMISE,
        )
        .is_err(),
        "empty range proof must be rejected"
    );
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_neg_tampered_sig_rejected() {
    let items = load_gen_assets(stage3_out());
    assert!(!items.is_empty(), "genesis has no assets");

    let mut item = items
        .iter()
        .find(|asset| asset.owner_signature.is_some())
        .expect("no asset with owner_signature")
        .clone();
    assert!(
        item.verify_complete().is_ok(),
        "baseline asset with signature must be valid"
    );

    let forged: KernelSignature = items
        .iter()
        .filter_map(|asset| asset.owner_signature.clone())
        .find(|sig| match item.owner_signature.as_ref() {
            Some(orig) => orig != sig,
            None => true,
        })
        .expect("need a different owner_signature for tamper");
    item.owner_signature = Some(forged);

    assert!(
        item.verify_complete().is_err(),
        "tampered owner signature must fail verify_complete"
    );
}

#[cfg(feature = "wallet_debug_tools")]
#[test]
fn test_neg_tampered_commitment_rejected() {
    let items = load_actor_assets(stage3_out(), "alice");
    assert!(!items.is_empty(), "alice has no claimed assets");

    let base = items
        .iter()
        .find(|asset| asset.range_proof.is_some())
        .expect("no asset with range proof")
        .clone();
    assert!(
        base.verify_complete().is_ok(),
        "baseline asset must be valid before commitment tamper"
    );

    let mut has_tamper_check = false;

    if let Ok(tampered) = z00z_crypto::Commitment::from_bytes(&[0xDE; 32]) {
        let mut item = base.clone();
        item.commitment = tampered.as_commitment().clone();
        has_tamper_check = true;
        assert!(
            item.verify_complete().is_err(),
            "tampered commitment must fail verify_complete"
        );
    }

    if let Ok(tampered) = z00z_crypto::Commitment::from_bytes(&[0x00; 32]) {
        let mut item = base.clone();
        item.commitment = tampered.as_commitment().clone();
        has_tamper_check = true;
        assert!(
            item.verify_complete().is_err(),
            "zero-point commitment must fail verify_complete"
        );
    }

    if let Some(other) = items.iter().find(|asset| asset.serial_id != base.serial_id) {
        let mut item = base.clone();
        item.commitment = other.commitment.clone();
        has_tamper_check = true;
        assert!(
            item.verify_complete().is_err(),
            "swapped valid commitment must fail verify_complete"
        );
    }

    assert!(
        has_tamper_check,
        "at least one commitment tamper check must execute"
    );
}
