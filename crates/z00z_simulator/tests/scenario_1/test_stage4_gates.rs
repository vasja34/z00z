use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde_json::Value;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{read_file, read_to_string},
};
use z00z_wallets::tx::TxPackage;

use z00z_simulator::scenario_1::stage_6::shared_cases;

struct RunCase {
    out: PathBuf,
}

static RUN_CASE: OnceLock<RunCase> = OnceLock::new();
const TX_PREP_STAGE: u32 = 6;

fn log_file(out: &Path) -> PathBuf {
    out.join("logs/logger.json")
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn sel_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_selected_inputs.json")
}

fn run_ok() -> &'static RunCase {
    RUN_CASE.get_or_init(|| {
        let out = shared_cases::default_stage6_out();
        assert!(
            tx_file(&out).exists(),
            "stage4 gates cache must contain tx package"
        );
        assert!(
            log_file(&out).exists(),
            "stage4 gates cache must contain logger"
        );
        assert!(
            sel_file(&out).exists(),
            "stage4 gates cache must contain selected inputs"
        );
        RunCase { out }
    })
}

fn load_tx_package(out: &Path) -> TxPackage {
    JsonCodec
        .deserialize(&read_file(tx_file(out)).expect("read tx"))
        .expect("decode tx")
}

fn load_logs(out: &Path) -> Vec<Value> {
    read_to_string(log_file(out))
        .expect("read logger")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("decode log row"))
        .collect()
}

fn input_sum(out: &Path) -> u64 {
    let rows: Vec<Value> = JsonCodec
        .deserialize(&read_file(sel_file(out)).expect("read selected"))
        .expect("decode selected");
    rows.iter()
        .map(|row| row["amount"].as_u64().expect("amount"))
        .sum()
}

fn step_ok(rows: &[Value], step: &str, event: &str, detail: &str) -> bool {
    rows.iter().any(|row| {
        row["stage"].as_u64() == Some(u64::from(TX_PREP_STAGE))
            && row["step"].as_str() == Some(step)
            && row["event"].as_str() == Some(event)
            && row["status"].as_str() == Some("ok")
            && row["detail"]
                .as_str()
                .is_some_and(|txt| txt.contains(detail))
    })
}

fn step_pos(rows: &[Value], step: &str) -> usize {
    rows.iter()
        .position(|row| {
            row["stage"].as_u64() == Some(u64::from(TX_PREP_STAGE))
                && row["step"].as_str() == Some(step)
        })
        .expect("step present")
}

#[test]
fn test_stage4_plain_gate_ok() {
    let out = &run_ok().out;
    let pkg = load_tx_package(out);
    let in_sum = input_sum(out);
    let out_sum: u64 = pkg.tx.outputs.iter().map(|out| out.asset_wire.amount).sum();
    let fee_sum: u64 = pkg
        .tx
        .outputs
        .iter()
        .filter(|out| matches!(out.role, z00z_wallets::tx::TxOutRole::Fee))
        .map(|out| out.asset_wire.amount)
        .sum();

    assert!(pkg.tx.fee > 0, "canonical run must produce non-zero fee");
    assert_eq!(pkg.tx.fee, fee_sum, "declared fee must equal Fee outputs");
    assert_eq!(
        in_sum, out_sum,
        "plaintext balance must include fee outputs inside total outputs"
    );
}

#[test]
fn test_stage4_commit_gate_ok() {
    let out = &run_ok().out;
    let rows = load_logs(out);

    assert!(
        step_ok(
            &rows,
            "S4-9",
            "balance_gate",
            "plaintext + commitment fee-inclusive balance verified"
        ),
        "logger must record balance gate success"
    );
}

#[test]
fn test_stage4_witness_gate_ok() {
    let out = &run_ok().out;
    let rows = load_logs(out);
    let bal_pos = step_pos(&rows, "S4-9");
    let wit_pos = step_pos(&rows, "S4-10");
    let pkg_pos = step_pos(&rows, "S4-6");

    assert!(
        step_ok(
            &rows,
            "S4-10",
            "spend_witness_gate",
            "spend witness gate passed"
        ),
        "logger must record spend witness gate success"
    );
    assert!(
        bal_pos < wit_pos,
        "spend witness gate must run after balance gate"
    );
    assert!(
        wit_pos < pkg_pos,
        "spend witness gate must run before tx package write"
    );
}

#[test]
fn test_stage4_verifier_order_ok() {
    let out = &run_ok().out;
    let rows = load_logs(out);

    let self_pos = step_pos(&rows, "S4-5");
    let bal_pos = step_pos(&rows, "S4-9");
    let wit_pos = step_pos(&rows, "S4-10");
    let pkg_pos = step_pos(&rows, "S4-6");
    let done_pos = step_pos(&rows, "S4-13");

    assert!(
        step_ok(&rows, "S4-5", "self_decrypt", "verified_outputs="),
        "logger must record self-decrypt success"
    );
    assert!(
        step_ok(&rows, "S4-6", "write_tx_pkg", "tx_alice_to_bob_pkg.json"),
        "logger must record tx package write after verifier gates"
    );
    assert!(
        step_ok(&rows, "S4-13", "stage_complete", "all gates passed"),
        "logger must record stage completion after the verifier chain"
    );
    assert!(
        self_pos < bal_pos,
        "self-decrypt must run before the balance gate"
    );
    assert!(
        bal_pos < wit_pos,
        "balance gate must run before the spend witness gate"
    );
    assert!(
        wit_pos < pkg_pos,
        "spend witness gate must run before tx package write"
    );
    assert!(
        pkg_pos < done_pos,
        "tx package write must finish before stage completion"
    );
}
