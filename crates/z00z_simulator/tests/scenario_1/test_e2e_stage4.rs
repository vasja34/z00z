#![cfg(all(
    not(target_arch = "wasm32"),
    feature = "test-params-fast",
    feature = "wallet_debug_tools"
))]

//! Acceptance coverage:
//! - Scenario 8: simulator Stage-3 claim leaves and Stage-4 tx outputs
//!   must stay valid under wallet-side canonical consumers.
//!   Round-trip fields fixed by this test: `asset_id`, `serial_id`, `amount`,
//!   `r_pub`, `owner_tag`, `asset_secret`, `blinding`.
//! - Scenario 9: owned and foreign receive verdicts must survive `.wlt`
//!   reload with stable claimed state.
//! - Scenario 10: duplicate delivery must keep stable receive verdicts
//!   and must not inflate claimed state.
//!   This protects both replay safety and duplicate inbox handling.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use z00z_crypto::expert::encoding::from_hex;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_simulator::scenario_1::claim_pkg_consumer::load_claim_leaves;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_file, save_json},
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{
        receiver_scan_leaf, ReceiveNext, ReceiveReject, ReceiveStatus, ScanResult,
        StealthOutputScanner,
    },
    rpc::types::{common::PersistWalletId, wallet::WalletSource},
    services::WalletService,
    tx::{asset_wire_to_leaf, resolve_input_secret, wire_decrypt_leaf, TxOutRole, TxPackage},
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

const ALICE_PASS: &str = "Alice_Pass_Z00Z_42!";
const BOB_PASS: &str = "Bob_Pass_Z00Z_43!";

struct RunCase {
    out: PathBuf,
}

struct WalletContext {
    svc: Arc<WalletService>,
    id: PersistWalletId,
    keys: ReceiverKeys,
}

struct WirePick {
    leaf: z00z_core::assets::AssetLeaf,
    asset: z00z_core::Asset,
}

type MineRow = (
    [u8; 32],
    u32,
    u64,
    [u8; 32],
    [u8; 32],
    Option<[u8; 32]>,
    Option<[u8; 32]>,
);

fn after_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_state_after.json")
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn claim_file(out: &Path) -> PathBuf {
    out.join("claim/tx_claim_pkg.json")
}

fn scenario_cfg(cfg: &mut z00z_simulator::config::ScenarioCfg) {
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 3;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 3;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 3;
    stage4.transaction.outputs.bob_outputs_count = 3;
    stage4.transaction.mode = "fraction".to_string();
    stage4.transaction.fraction = Some(0.5);
    stage4.transaction.amount = None;
}

fn load_json_value(path: &Path) -> serde_json::Value {
    JsonCodec
        .deserialize(&read_file(path).expect("read json"))
        .expect("decode json")
}

fn load_tx_package(out: &Path) -> TxPackage {
    JsonCodec
        .deserialize(&read_file(tx_file(out)).expect("read tx pkg"))
        .expect("decode tx")
}

fn rewrite_wallet_paths(out: &Path) {
    let map_path = out.join("wallets/wlt_map.md");
    let map_text = std::fs::read_to_string(&map_path).expect("read wlt_map");
    let mut lines = Vec::new();
    for line in map_text.lines() {
        let cols: Vec<_> = line.split('|').map(str::trim).collect();
        if cols.len() == 3 && cols[0] != "name" && !cols[0].starts_with("-----") {
            let wallet_name = Path::new(cols[2])
                .file_name()
                .and_then(|item| item.to_str())
                .expect("wallet filename");
            let new_path = out.join("wallets").join(wallet_name);
            lines.push(format!(
                "{} | {} | {}",
                cols[0],
                cols[1],
                new_path.display()
            ));
        } else {
            lines.push(line.to_string());
        }
    }
    std::fs::write(&map_path, lines.join("\n")).expect("rewrite wlt_map");

    let after_path = after_file(out);
    if after_path.exists() {
        let mut dump = load_json_value(&after_path);
        for row in dump["wallets"].as_array_mut().expect("wallet rows") {
            let wallet_name = Path::new(
                row["wlt_path"]
                    .as_str()
                    .expect("wallet path string in after dump"),
            )
            .file_name()
            .and_then(|item| item.to_str())
            .expect("wallet filename in after dump");
            row["wlt_path"] = serde_json::Value::String(
                out.join("wallets")
                    .join(wallet_name)
                    .to_string_lossy()
                    .to_string(),
            );
        }
        save_json(&after_path, &dump).expect("rewrite wallets_state_after");
    }
}

fn copy_file(src: &Path, dst: &Path) {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).expect("create destination parent");
    }
    std::fs::copy(src, dst).expect("copy file");
}

fn clone_runtime_case(src_out: &Path, include_tx_pkg: bool) -> RunCase {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let base = temp.keep();
    let out = base.join("outputs/scenario_1");
    fixture_cache::copy_tree(&src_out.join("wallets"), &out.join("wallets"));
    fixture_cache::copy_tree(&src_out.join("claim"), &out.join("claim"));
    let after_src = src_out.join("transactions/wallets_state_after.json");
    if after_src.exists() {
        copy_file(
            &after_src,
            &out.join("transactions/wallets_state_after.json"),
        );
    }
    if include_tx_pkg {
        copy_file(
            &src_out.join("transactions/tx_alice_to_bob_pkg.json"),
            &out.join("transactions/tx_alice_to_bob_pkg.json"),
        );
    }
    rewrite_wallet_paths(&out);
    RunCase { out }
}

fn stage3_baseline_out() -> &'static PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        let root = fixture_cache::ensure_case("e2e_stage4_stage3_v3", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, scenario_cfg);
            let _ctx =
                stage_runner_support::run_stage_setup(&cfg_path, &design_path, &[1_u32, 2, 3]);
            assert!(claim_file(&out).exists(), "stage3 claim bundle must exist");
        });
        root.join("outputs/scenario_1")
    })
}

fn stage3_case() -> RunCase {
    clone_runtime_case(stage3_baseline_out(), false)
}

fn stage6_baseline_out() -> &'static PathBuf {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    OUT.get_or_init(|| {
        let root = fixture_cache::ensure_case("e2e_stage4_stage6_v3", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, scenario_cfg);
            let _ctx = stage_runner_support::run_stage_setup(
                &cfg_path,
                &design_path,
                &[1_u32, 2, 3, 4, 5, 6],
            );
            assert!(tx_file(&out).exists(), "stage6 tx package must exist");
        });
        root.join("outputs/scenario_1")
    })
}

fn run_ok() -> RunCase {
    clone_runtime_case(stage6_baseline_out(), true)
}

fn find_wallet_id(out: &Path, actor: &str) -> String {
    let after = after_file(out);
    if after.exists() {
        let dump = load_json_value(&after);
        if let Some(id) = dump["wallets"]
            .as_array()
            .expect("wallets rows")
            .iter()
            .find(|row| row["actor"].as_str() == Some(actor))
            .and_then(|row| row["wallet_id"].as_str())
        {
            return id.to_string();
        }
    }

    find_wlt_row(out, actor)
        .map(|(wallet_id, _)| wallet_id)
        .expect("wallet id")
}

fn find_wlt_row(out: &Path, actor: &str) -> Option<(String, String)> {
    let map = std::fs::read_to_string(out.join("wallets/wlt_map.md")).ok()?;
    map.lines()
        .skip(3)
        .filter_map(|line| {
            let cols: Vec<_> = line.split('|').map(str::trim).collect();
            if cols.len() != 3 {
                return None;
            }
            Some((
                cols[0].to_string(),
                format!("wallet_{}", cols[1]),
                cols[2].to_string(),
            ))
        })
        .find(|(name, _, _)| name == actor)
        .map(|(_, wallet_id, wlt_path)| (wallet_id, wlt_path))
}

fn find_wallet_path(out: &Path, actor: &str) -> PathBuf {
    let after = after_file(out);
    if after.exists() {
        let dump = load_json_value(&after);
        if let Some(path) = dump["wallets"]
            .as_array()
            .expect("wallets rows")
            .iter()
            .find(|row| row["actor"].as_str() == Some(actor))
            .and_then(|row| row["wlt_path"].as_str())
        {
            return PathBuf::from(path);
        }
    }

    PathBuf::from(
        find_wlt_row(out, actor)
            .map(|(_, wlt_path)| wlt_path)
            .expect("wlt path"),
    )
}

fn find_secret_hex(out: &Path, actor: &str) -> String {
    let secrets_file = out.join("wallets/private/wlt_secrets_debug.md");
    let text = std::fs::read_to_string(&secrets_file).expect("read secret debug table");

    text.lines()
        .filter(|line| line.contains('|'))
        .filter_map(|line| {
            let cols: Vec<_> = line.split('|').map(str::trim).collect();
            if cols.len() < 5 {
                return None;
            }
            if cols[0] == "name" || cols[0].starts_with("-----") {
                return None;
            }
            Some((cols[0], cols[4]))
        })
        .find(|(name, _)| name.eq_ignore_ascii_case(actor))
        .map(|(_, secret_hex)| secret_hex.to_string())
        .expect("receiver secret hex")
}

fn open_bob_wallet(out: &Path) -> WalletContext {
    open_actor(out, "bob", BOB_PASS)
}

fn open_actor(out: &Path, actor: &str, pass: &str) -> WalletContext {
    let id = PersistWalletId(find_wallet_id(out, actor));
    let path = find_wallet_path(out, actor);
    let dir = path.parent().expect("wallet dir").to_path_buf();
    let svc = Arc::new(WalletService::with_output_dir(dir));
    let keys = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            svc.open_wallet_source(WalletSource::Path {
                path: path.to_string_lossy().to_string(),
            })
            .await
            .expect("open wallet source");
            svc.unlock_wallet_in_memory(&id, &SafePassword::from(pass))
                .await
                .expect("unlock wallet");
            svc.receiver_keys(&id).await.expect("receiver keys")
        });

    WalletContext { svc, id, keys }
}

#[test]
fn test_stage3_alice_remain_spendable() {
    let case = stage3_case();
    let out = &case.out;
    let alice = open_actor(out, "alice", ALICE_PASS);
    let claimed = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            alice
                .svc
                .list_claimed_assets(&alice.id)
                .await
                .expect("claimed")
        });
    let coin_rows: Vec<_> = claimed
        .into_iter()
        .filter(|item| item.definition.class == z00z_core::AssetClass::Coin)
        .filter(|item| item.definition.symbol == "Z00Z")
        .collect();
    let stealth_cnt = coin_rows
        .iter()
        .filter(|item| {
            item.r_pub.is_some()
                && item.owner_tag.is_some()
                && item.enc_pack.is_some()
                && item.tag16.is_some()
        })
        .count();
    let recv_hex = find_secret_hex(out, "alice");
    let recv_raw = from_hex(&recv_hex).expect("decode receiver secret");
    let recv_sec = ReceiverSecret::from_bytes(
        recv_raw
            .as_slice()
            .try_into()
            .expect("receiver secret size"),
    )
    .expect("receiver secret")
    .as_bytes()
    .to_owned();
    let dec_cnt = coin_rows
        .iter()
        .filter(|item| {
            resolve_input_secret(recv_sec, &z00z_core::AssetWire::from_asset(item)).is_ok()
        })
        .count();
    let mine_cnt = coin_rows
        .iter()
        .filter(|item| {
            let wire = z00z_core::AssetWire::from_asset(item);
            let leaf = decrypt_claim_leaf(&wire);
            receiver_scan_leaf(&alice.keys, &leaf)
                .expect("scan")
                .is_some()
        })
        .count();

    assert!(
        !coin_rows.is_empty(),
        "alice must keep claimed Coin/Z00Z rows after stage3"
    );
    assert_eq!(
        stealth_cnt,
        coin_rows.len(),
        "all Coin/Z00Z rows must keep stealth fields after stage3"
    );
    assert_eq!(
        dec_cnt,
        coin_rows.len(),
        "all Coin/Z00Z rows must stay decryptable for alice after stage3"
    );
    assert_eq!(
        mine_cnt,
        coin_rows.len(),
        "all Coin/Z00Z rows must scan as mine"
    );

    lock_one(&alice);
}

fn lock_one(wallet: &WalletContext) {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            wallet
                .svc
                .lock_wallet(&wallet.id)
                .await
                .expect("lock wallet")
        });
}

fn decrypt_claim_leaf(wire: &z00z_core::AssetWire) -> z00z_core::assets::AssetLeaf {
    wire_decrypt_leaf(wire).expect("claim leaf").into()
}

fn scan_asset(
    wallet: &WalletContext,
    asset: &z00z_core::Asset,
) -> Result<ScanResult, ReceiveReject> {
    Ok(StealthOutputScanner::from_keys(&wallet.keys).scan_leaf(asset))
}

fn recv_put(wallet: &WalletContext, asset: z00z_core::Asset) -> bool {
    tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            wallet
                .svc
                .recv_route(&wallet.id, asset, ReceiveNext::PersistClaim)
                .await
                .expect("persist claim")
        })
}

fn claim_ids(wallet: &WalletContext) -> Vec<[u8; 32]> {
    let mut out = tokio::runtime::Runtime::new()
        .expect("tokio runtime")
        .block_on(async {
            wallet
                .svc
                .list_claimed_assets(&wallet.id)
                .await
                .expect("claimed")
        })
        .into_iter()
        .map(|row| row.asset_id())
        .collect::<Vec<_>>();
    out.sort_unstable();
    out
}

fn pick_s3(wallet: &WalletContext, out: &Path) -> WirePick {
    let rows = load_claim_leaves(&claim_file(out)).expect("load claim leaves");

    for row in rows {
        let leaf = decrypt_claim_leaf(&row);
        if receiver_scan_leaf(&wallet.keys, &leaf)
            .expect("claim scan")
            .is_some()
        {
            return WirePick {
                leaf,
                asset: row.to_asset().expect("claim asset"),
            };
        }
    }

    panic!("bob stage3 leaf missing");
}

fn pick_s4(wallet: &WalletContext, out: &Path, want_mine: bool) -> WirePick {
    let pkg = load_tx_package(out);

    for row in pkg.tx.outputs {
        if want_mine && !matches!(row.role, TxOutRole::Recipient) {
            continue;
        }

        let wire = row.asset_wire.to_wire().expect("stage4 wire");
        let leaf = wire_decrypt_leaf(&wire).expect("stage4 leaf");
        let is_mine = receiver_scan_leaf(&wallet.keys, &leaf)
            .expect("stage4 scan")
            .is_some();
        if is_mine == want_mine {
            return WirePick {
                leaf: leaf.into(),
                asset: wire.to_asset().expect("stage4 asset"),
            };
        }
    }

    panic!("stage4 wire missing");
}

fn bad_asset(asset: &z00z_core::Asset) -> z00z_core::Asset {
    let mut bad = asset.clone();
    let pack = bad.enc_pack.as_mut().expect("enc pack");
    pack.ciphertext[0] ^= 1;
    bad
}

fn mine_out(scan: &ScanResult) -> &z00z_wallets::receiver::WalletStealthOutput {
    let ScanResult::Mine { wallet_output } = scan else {
        panic!("expected Mine, got {scan:?}");
    };
    wallet_output
}

fn mine_row(out: &z00z_wallets::receiver::WalletStealthOutput) -> MineRow {
    (
        out.asset_id,
        out.serial_id,
        out.amount,
        out.r_pub,
        out.owner_tag,
        out.asset_secret.into(),
        out.blinding.into(),
    )
}

#[test]
fn test_stage34_ok() {
    let case = run_ok();
    let bob = open_bob_wallet(&case.out);

    let s3 = pick_s3(&bob, &case.out);
    s3.asset.verify_complete().expect("stage3 verify_complete");
    let s3_pack = receiver_scan_leaf(&bob.keys, &s3.leaf)
        .expect("stage3 leaf scan")
        .expect("stage3 owned pack");
    let s3_run = scan_asset(&bob, &s3.asset).expect("stage3 scan_asset");
    let s3_out = mine_out(&s3_run);

    assert_eq!(s3_run.recv_report().status, ReceiveStatus::Detected);
    assert_eq!(s3_out.asset_id, s3.asset.asset_id());
    assert_eq!(s3_out.serial_id, s3.leaf.serial_id);
    assert_eq!(s3_out.amount, s3_pack.value);
    assert_eq!(s3_out.r_pub, s3.leaf.r_pub);
    assert_eq!(s3_out.owner_tag, s3.leaf.owner_tag);
    assert_eq!(s3_out.asset_secret, Some(s3_pack.s_out));
    assert_eq!(s3_out.blinding, Some(s3_pack.blinding));

    let s4 = pick_s4(&bob, &case.out, true);
    s4.asset.verify_complete().expect("stage4 verify_complete");
    let s4_pack = receiver_scan_leaf(&bob.keys, &s4.leaf)
        .expect("stage4 leaf scan")
        .expect("stage4 owned pack");
    let s4_run = scan_asset(&bob, &s4.asset).expect("stage4 scan_asset");
    let s4_out = mine_out(&s4_run);

    assert_eq!(s4_run.recv_report().status, ReceiveStatus::Detected);
    assert_eq!(s4_out.asset_id, s4.asset.asset_id());
    assert_eq!(s4_out.serial_id, s4.leaf.serial_id);
    assert_eq!(s4_out.amount, s4_pack.value);
    assert_eq!(s4_out.r_pub, s4.leaf.r_pub);
    assert_eq!(s4_out.owner_tag, s4.leaf.owner_tag);
    assert_eq!(s4_out.asset_secret, Some(s4_pack.s_out));
    assert_eq!(s4_out.blinding, Some(s4_pack.blinding));

    let bad = bad_asset(&s4.asset);
    let bad_run = scan_asset(&bob, &bad).expect("bad scan_asset");
    let bad_leaf = asset_wire_to_leaf(&z00z_core::AssetWire::from_asset(&bad)).expect("bad leaf");
    let bad_rep =
        z00z_wallets::receiver::receiver_scan_report(&bob.keys, &bad_leaf).expect("bad report");

    assert_eq!(bad_rep.status, ReceiveStatus::InvalidProof);
    assert_eq!(bad_run.recv_report().status, ReceiveStatus::InvalidProof);
    assert_eq!(bad_rep.status, bad_run.recv_report().status);
    assert!(matches!(bad_run, ScanResult::MaybeMine { .. }));

    lock_one(&bob);
}

#[test]
fn test_reload_ok() {
    let case = run_ok();
    let before = {
        let bob = open_bob_wallet(&case.out);
        let own = pick_s4(&bob, &case.out, true);
        let own_run = scan_asset(&bob, &own.asset).expect("owned recv");
        let own_row = mine_row(mine_out(&own_run));
        let foreign = pick_s4(&bob, &case.out, false);
        let foreign_run = scan_asset(&bob, &foreign.asset).expect("foreign recv");
        let ids = claim_ids(&bob);
        let id = bob.id.clone();
        let out_dir = case.out.clone();

        assert_eq!(foreign_run.recv_report().status, ReceiveStatus::NotMine);
        let row = (own_row, ids, out_dir, id);
        lock_one(&bob);
        row
    };

    let (own_a, ids_a, out_dir, id) = before;
    let bob = open_bob_wallet(&out_dir);
    assert_eq!(bob.id, id);

    let own = pick_s4(&bob, &out_dir, true);
    let own_run = scan_asset(&bob, &own.asset).expect("owned recv after reload");
    let own_b = mine_row(mine_out(&own_run));
    let foreign = pick_s4(&bob, &out_dir, false);
    let foreign_run = scan_asset(&bob, &foreign.asset).expect("foreign recv after reload");
    let ids_b = claim_ids(&bob);

    assert_eq!(own_b, own_a);
    assert_eq!(foreign_run.recv_report().status, ReceiveStatus::NotMine);
    assert_eq!(ids_b, ids_a);

    lock_one(&bob);
}

#[test]
fn test_replay_ok() {
    let case = run_ok();
    let before = {
        let bob = open_bob_wallet(&case.out);
        let own = pick_s4(&bob, &case.out, true);
        let foreign = pick_s4(&bob, &case.out, false);

        let own_a = scan_asset(&bob, &own.asset).expect("own a");
        let own_b = scan_asset(&bob, &own.asset).expect("own b");
        let own_row = mine_row(mine_out(&own_a));
        let foreign_a = scan_asset(&bob, &foreign.asset).expect("foreign a");
        let foreign_b = scan_asset(&bob, &foreign.asset).expect("foreign b");
        let ids_a = claim_ids(&bob);
        let add_a = recv_put(&bob, own.asset.clone());
        let add_b = recv_put(&bob, own.asset.clone());
        let ids_b = claim_ids(&bob);
        let id = bob.id.clone();
        let out_dir = case.out.clone();

        assert_eq!(own_a, own_b);
        assert_eq!(foreign_a, foreign_b);
        assert_eq!(foreign_a.recv_report().status, ReceiveStatus::NotMine);
        assert!(!add_b, "second persist must be deduped");
        assert_eq!(ids_b.len(), ids_a.len() + usize::from(add_a));

        let row = (
            own_row,
            own_a.recv_report().status,
            foreign_a.recv_report().status,
            ids_b,
            out_dir,
            id,
        );
        lock_one(&bob);
        row
    };

    let (own_a, own_st, foreign_st, ids_a, out_dir, id) = before;
    let bob = open_bob_wallet(&out_dir);
    assert_eq!(bob.id, id);

    let own = pick_s4(&bob, &out_dir, true);
    let foreign = pick_s4(&bob, &out_dir, false);
    let own_b = scan_asset(&bob, &own.asset).expect("own reload");
    let foreign_b = scan_asset(&bob, &foreign.asset).expect("foreign reload");
    let own_row = mine_row(mine_out(&own_b));
    let ids_b = claim_ids(&bob);

    assert_eq!(own_row, own_a);
    assert_eq!(own_b.recv_report().status, own_st);
    assert_eq!(foreign_b.recv_report().status, foreign_st);
    assert_eq!(ids_b, ids_a);

    lock_one(&bob);
}
