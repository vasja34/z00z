use std::{collections::BTreeSet, path::Path};

use serde_json::Value;
use z00z_simulator::scenario_1::stage_6;
use z00z_simulator::StageResult;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{path_exists, read_file},
};

use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::scenario_1::support::scenario_support;
use z00z_simulator::scenario_1::support::stage_runner_support;

use crate::stage4_paths::assert_absent;
use scenario_support::{make_cfg, read_rpc_req_rows};

fn tx_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn pending_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_pending.json")
}

fn confirm_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_confirmed.json")
}

fn sel_file(out: &Path) -> std::path::PathBuf {
    out.join("transactions/wallets_selected_inputs.json")
}

fn pick_rows(out: &Path) -> Vec<Value> {
    JsonCodec
        .deserialize(&read_file(sel_file(out)).expect("read selected inputs"))
        .expect("decode selected inputs")
}

fn req_rows(out: &Path) -> Vec<Value> {
    read_rpc_req_rows(out)
}

fn sel_none(out: &Path) {
    if !path_exists(sel_file(out)).expect("selected inputs path") {
        return;
    }

    let rows = pick_rows(out);
    assert!(
        rows.is_empty(),
        "selected inputs must be absent or empty on failed selection"
    );
}

fn has_sel_err(msg: &str) -> bool {
    msg.contains("no spendable rows")
        || msg.contains("distinct serial_id requirement")
        || msg.contains("selected serial pool is empty")
}

fn has_wallet(log_id: &str, wallet_id: &str) -> bool {
    let prefix = &wallet_id[..10.min(wallet_id.len())];
    let suffix_start = wallet_id.len().saturating_sub(4);
    let suffix = &wallet_id[suffix_start..];
    log_id.contains(prefix) && log_id.contains(suffix)
}

fn req_methods(rows: &[Value], wallet_id: &str) -> Vec<String> {
    rows.iter()
        .filter(|row| {
            row["wallet_id"]
                .as_str()
                .map(|log_id| has_wallet(log_id, wallet_id))
                .unwrap_or(false)
        })
        .filter_map(|row| row["method"].as_str().map(str::to_string))
        .collect()
}

fn has_flow(methods: &[String], need: [&str; 3]) -> bool {
    let mut pos = 0usize;
    for item in methods {
        if item == need[pos] {
            pos += 1;
            if pos == need.len() {
                return true;
            }
        }
    }
    false
}

fn ok_out() -> std::path::PathBuf {
    let root = fixture_cache::ensure_shared_case("stage4_selection_ok_shared_v1", |base| {
        let (cfg_path, design_path, out) = scenario_support::make_cfg_in(base, |cfg| {
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
        });
        let _ctx = stage_runner_support::run_stage4_setup(&cfg_path, &design_path);
        assert!(tx_file(&out).exists(), "selection shared tx missing");
    });
    root.join("outputs/scenario_1")
}

#[test]
fn test_stage4_select_rows() {
    let out = ok_out();

    let rows = pick_rows(&out);
    assert_eq!(rows.len(), 3);
    let serials: BTreeSet<u32> = rows
        .iter()
        .map(|row| row["serial_id"].as_u64().expect("serial") as u32)
        .collect();
    assert_eq!(serials.len(), 3);
    assert!(rows.iter().all(|row| row["class"].as_str() == Some("Coin")));
    assert!(rows
        .iter()
        .all(|row| row["symbol"].as_str() == Some("Z00Z")));

    let wallet_id = rows[0]["wallet_id"].as_str().expect("wallet_id");
    let reqs = req_rows(&out);
    let methods = req_methods(&reqs, wallet_id);
    assert!(
        has_flow(
            &methods,
            [
                "wallet.session.unlock_wallet",
                "wallet.asset.list_assets",
                "wallet.session.lock_wallet",
            ],
        ),
        "missing sender unlock/list/lock flow in rpc log"
    );
}

#[test]
fn test_stage4_rejects_serial_pool() {
    let (cfg_path, design_path, out) = make_cfg(|cfg| {
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
        stage4.rpc.list_filter.min_balance = Some(u64::MAX);
    });

    let mut ctx = stage_runner_support::run_stage5_session(&cfg_path, &design_path);
    let stage6 = stage_runner_support::stage_by_id(&design_path, 6);
    match stage_6::run_tx_prepare(&mut ctx, &stage6) {
        StageResult::Fail(msg) => {
            assert!(has_sel_err(&msg), "unexpected error: {msg}");
        }
        other => panic!("tx_prepare stage must fail, got {other:?}"),
    }

    assert_absent(&tx_file(&out));
    sel_none(&out);
    assert_absent(&pending_file(&out));
    assert_absent(&confirm_file(&out));
}
