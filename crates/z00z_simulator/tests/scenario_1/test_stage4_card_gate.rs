use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use std::path::{Path, PathBuf};

use serde::Serialize;
use z00z_simulator::{scenario_1::stage_6, StageResult};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{create_dir_all, path_exists, read_to_string, write_file},
};
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::{encode_card_compact, ReceiverCard},
};

#[derive(Serialize)]
struct KeyFile {
    owner_handle: String,
    view_pk: String,
    identity_pk: String,
    card_compact: Option<String>,
}

fn mk_card(seed: u8) -> ReceiverCard {
    let secret = ReceiverSecret::from_bytes([seed; 32]).expect("receiver secret");
    ReceiverKeys::from_receiver_secret(secret)
        .expect("receiver keys")
        .export_receiver_card()
        .expect("receiver card")
}

fn mk_key(seed: u8, compact: Option<String>) -> KeyFile {
    let card = mk_card(seed);
    KeyFile {
        owner_handle: hex::encode(card.owner_handle),
        view_pk: hex::encode(card.view_pk),
        identity_pk: hex::encode(card.identity_pk),
        card_compact: compact,
    }
}

fn bad_compact(seed: u8) -> String {
    let mut card = mk_card(seed);
    card.signature[0] ^= 1;
    encode_card_compact(&card)
}

fn write_key(path: &Path, key: &KeyFile) {
    let bytes = JsonCodec.serialize(key).expect("key json");
    write_file(path, &bytes).expect("write key file");
}

fn key_fixture(base: &Path, name: &str, key: &KeyFile) -> PathBuf {
    let dir = base.join("keys_fixture");
    create_dir_all(&dir).expect("fixture dir");
    let path = dir.join(format!("{name}.json"));
    write_key(&path, key);
    path
}

fn tune_stage4(cfg: &mut z00z_simulator::config::ScenarioCfg) {
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_min = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_target = 4;
    stage4
        .transaction
        .input_assets_selection
        .distinct_serial_ids_max = 10;
    stage4.transaction.outputs.bob_outputs_count = 4;
    stage4.transaction.fraction = Some(0.1);
}

struct OutCase {
    out: PathBuf,
}

struct FailCase {
    out: PathBuf,
    msg: String,
}

fn ok_case() -> &'static OutCase {
    static CASE: std::sync::OnceLock<OutCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_card_ok_v1", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, tune_stage4);
            let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
            assert!(
                path_exists(out.join("transactions/tx_alice_to_bob_pkg.json")).expect("tx pkg")
            );
            assert!(path_exists(out.join("transactions/checkpoint_prep.json")).expect("prep pkg"));
            assert!(
                path_exists(out.join("transactions/wallets_pending.json")).expect("pending rows")
            );
            assert!(path_exists(out.join("transactions/wallets_confirmed.json"))
                .expect("confirmed rows"));
        });
        OutCase {
            out: root.join("outputs/scenario_1"),
        }
    })
}

fn load_fail_case(root: &Path) -> FailCase {
    FailCase {
        out: root.join("outputs/scenario_1"),
        msg: read_to_string(root.join("fail_msg.txt")).expect("read card fail msg"),
    }
}

fn card_missing_case() -> &'static FailCase {
    static CASE: std::sync::OnceLock<FailCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_card_missing_v1", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |cfg| {
                tune_stage4(cfg);
                let alice = key_fixture(base, "alice_missing", &mk_key(7, None));
                let bob = key_fixture(
                    base,
                    "bob_ok",
                    &mk_key(8, Some(encode_card_compact(&mk_card(8)))),
                );
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.paths.alice_keys_file = alice.to_string_lossy().to_string();
                stage4.paths.bob_keys_file = bob.to_string_lossy().to_string();
            });
            let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
            let stage = stage_runner_support::stage_by_id(&design_path, 6);
            let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("expected StageResult::Fail, got {other:?}"),
            };
            write_file(base.join("fail_msg.txt"), msg.as_bytes()).expect("write card fail msg");
            assert!(
                !path_exists(out.join("transactions/tx_alice_to_bob_pkg.json"))
                    .expect("tx missing")
            );
        });
        load_fail_case(&root)
    })
}

fn card_verify_fail_case() -> &'static FailCase {
    static CASE: std::sync::OnceLock<FailCase> = std::sync::OnceLock::new();
    CASE.get_or_init(|| {
        let root = fixture_cache::ensure_case("stage4_card_verify_fail_v1", |base| {
            let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |cfg| {
                tune_stage4(cfg);
                let alice = key_fixture(base, "alice_bad", &mk_key(9, Some(bad_compact(9))));
                let bob = key_fixture(
                    base,
                    "bob_ok",
                    &mk_key(10, Some(encode_card_compact(&mk_card(10)))),
                );
                let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
                stage4.paths.alice_keys_file = alice.to_string_lossy().to_string();
                stage4.paths.bob_keys_file = bob.to_string_lossy().to_string();
            });
            let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
            let stage = stage_runner_support::stage_by_id(&design_path, 6);
            let msg = match stage_6::run_tx_prepare(&mut ctx, &stage) {
                StageResult::Fail(msg) => msg,
                other => panic!("expected StageResult::Fail, got {other:?}"),
            };
            write_file(base.join("fail_msg.txt"), msg.as_bytes()).expect("write card fail msg");
            assert!(
                !path_exists(out.join("transactions/tx_alice_to_bob_pkg.json"))
                    .expect("tx missing")
            );
        });
        load_fail_case(&root)
    })
}

#[test]
fn test_stage4_card_ok() {
    let out = &ok_case().out;

    assert!(path_exists(out.join("transactions/tx_alice_to_bob_pkg.json")).expect("tx pkg"));
    assert!(path_exists(out.join("transactions/checkpoint_prep.json")).expect("prep pkg"));
    assert!(path_exists(out.join("transactions/wallets_pending.json")).expect("pending rows"));
    assert!(path_exists(out.join("transactions/wallets_confirmed.json")).expect("confirmed rows"));

    let rows = log_rows(out);
    assert!(rows.iter().any(|row| {
        row["step"] == "S4-3"
            && row["event"] == "load_card_compact"
            && row["detail"] == "alice_keys.json + bob_keys.json via card_compact verify"
    }));
}

#[test]
fn test_stage4_card_missing() {
    let case = card_missing_case();
    assert!(case.msg.contains("stage4: missing card_compact"));
    assert!(
        !path_exists(case.out.join("transactions/tx_alice_to_bob_pkg.json")).expect("tx missing")
    );
}

#[test]
fn test_stage4_card_verify_fail() {
    let case = card_verify_fail_case();
    assert!(case.msg.contains("stage4: receiver card verify failed"));
    assert!(
        !path_exists(case.out.join("transactions/tx_alice_to_bob_pkg.json")).expect("tx missing")
    );
}

fn log_rows(out: &Path) -> Vec<serde_json::Value> {
    read_to_string(out.join("logs/logger.json"))
        .expect("read stage log")
        .lines()
        .filter_map(|line| {
            JsonCodec
                .deserialize::<serde_json::Value>(line.as_bytes())
                .ok()
        })
        .collect()
}
