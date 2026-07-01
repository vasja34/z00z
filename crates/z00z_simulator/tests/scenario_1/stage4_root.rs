use std::path::Path;

use z00z_simulator::config::ScenarioCfg;

pub fn set_s4_root(cfg: &mut ScenarioCfg, root: &Path) {
    let stage4 = cfg.stage4_tx_prepare.as_mut().expect("stage4 cfg");
    let root = root.to_path_buf();
    stage4.paths.outputs_dir = root.to_string_lossy().to_string();
    stage4.paths.logs_dir = root.join("logs").to_string_lossy().to_string();
    stage4.paths.transactions_dir = root.join("transactions").to_string_lossy().to_string();
    stage4.paths.wallets_dir = root.join("wallets").to_string_lossy().to_string();
    stage4.paths.tx_pkg_file = root
        .join("transactions/tx_alice_to_bob_pkg.json")
        .to_string_lossy()
        .to_string();
    stage4.paths.snapshot_file = root
        .join("stage_4_snapshot.json")
        .to_string_lossy()
        .to_string();
    stage4.paths.logger_file = root.join("logs/logger.json").to_string_lossy().to_string();
    stage4.paths.rpc_logger_file = root
        .join("logs/rpc_logger.json")
        .to_string_lossy()
        .to_string();
    stage4.paths.alice_keys_file = root
        .join("keys/alice_keys.json")
        .to_string_lossy()
        .to_string();
    stage4.paths.bob_keys_file = root
        .join("keys/bob_keys.json")
        .to_string_lossy()
        .to_string();
    stage4.paths.wallets_state_before_file = Some(
        root.join("transactions/wallets_state_before.json")
            .to_string_lossy()
            .to_string(),
    );
    stage4.paths.wallets_state_after_file = Some(
        root.join("transactions/wallets_state_after.json")
            .to_string_lossy()
            .to_string(),
    );
    stage4.paths.wallets_state_diff_file = Some(
        root.join("transactions/wallets_state_diff.json")
            .to_string_lossy()
            .to_string(),
    );
    stage4.paths.wallets_state_report_md_file = Some(
        root.join("transactions/wallets_state_report.md")
            .to_string_lossy()
            .to_string(),
    );
    stage4.paths.wallets_state_report_xlsx_file = Some(
        root.join("transactions/wallets_state_report.xlsx")
            .to_string_lossy()
            .to_string(),
    );
}
