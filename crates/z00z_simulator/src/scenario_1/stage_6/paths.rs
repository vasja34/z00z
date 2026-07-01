use super::{Codec, SimContext, Stage4ResolvedPaths, Stage4TxPrepareCfg};

// SIMULATOR-ONLY: DO NOT MOVE TO CORE.
pub(crate) fn resolve_stage4_paths(
    ctx: &SimContext,
    cfg: &Stage4TxPrepareCfg,
) -> Stage4ResolvedPaths {
    let runtime_out = ctx.outputs_dir.clone();
    let cfg_out = std::path::PathBuf::from(&cfg.paths.outputs_dir);
    let outputs_dir = remap_out(&runtime_out, &cfg_out, &cfg.paths.outputs_dir);
    let logs_dir = remap_out(&runtime_out, &cfg_out, &cfg.paths.logs_dir);
    let transactions_dir = remap_out(&runtime_out, &cfg_out, &cfg.paths.transactions_dir);
    let wallets_dir = remap_out(&runtime_out, &cfg_out, &cfg.paths.wallets_dir);
    let tx_pkg_file = remap_out(&runtime_out, &cfg_out, &cfg.paths.tx_pkg_file);
    let snapshot_file = remap_out(&runtime_out, &cfg_out, &cfg.paths.snapshot_file);
    let logger_file = remap_out(&runtime_out, &cfg_out, &cfg.paths.logger_file);
    let rpc_logger_file = remap_out(&runtime_out, &cfg_out, &cfg.paths.rpc_logger_file);
    let alice_keys_file = remap_out(&runtime_out, &cfg_out, &cfg.paths.alice_keys_file);
    let bob_keys_file = remap_out(&runtime_out, &cfg_out, &cfg.paths.bob_keys_file);

    let tx_anchor = transactions_dir.clone();

    Stage4ResolvedPaths {
        outputs_dir,
        logs_dir,
        transactions_dir,
        wallets_dir,
        tx_pkg_file,
        snapshot_file,
        logger_file,
        rpc_logger_file,
        alice_keys_file,
        bob_keys_file,
        wallets_state_before_file: cfg
            .paths
            .wallets_state_before_file
            .as_ref()
            .map(|v| remap_tx(&runtime_out, &cfg_out, &tx_anchor, v)),
        wallets_state_after_file: cfg
            .paths
            .wallets_state_after_file
            .as_ref()
            .map(|v| remap_tx(&runtime_out, &cfg_out, &tx_anchor, v)),
        wallets_state_diff_file: cfg
            .paths
            .wallets_state_diff_file
            .as_ref()
            .map(|v| remap_tx(&runtime_out, &cfg_out, &tx_anchor, v)),
        wallets_state_report_md_file: cfg
            .paths
            .wallets_state_report_md_file
            .as_ref()
            .map(|v| remap_tx(&runtime_out, &cfg_out, &tx_anchor, v)),
        wallets_state_report_xlsx_file: cfg
            .paths
            .wallets_state_report_xlsx_file
            .as_ref()
            .map(|v| remap_tx(&runtime_out, &cfg_out, &tx_anchor, v)),
    }
}

fn remap_out(
    runtime_out: &std::path::Path,
    cfg_out: &std::path::Path,
    raw: &str,
) -> std::path::PathBuf {
    let configured = std::path::PathBuf::from(raw);
    if configured.is_absolute() {
        return configured;
    }
    if configured.starts_with(cfg_out) {
        if let Ok(suffix) = configured.strip_prefix(cfg_out) {
            return runtime_out.join(suffix);
        }
    }
    runtime_out.join(configured)
}

fn remap_tx(
    runtime_out: &std::path::Path,
    cfg_out: &std::path::Path,
    tx_anchor: &std::path::Path,
    raw: &str,
) -> std::path::PathBuf {
    let configured = std::path::PathBuf::from(raw);
    if configured.is_absolute() {
        return configured;
    }
    if configured.starts_with(cfg_out) {
        if let Ok(suffix) = configured.strip_prefix(cfg_out) {
            return runtime_out.join(suffix);
        }
    }
    tx_anchor.join(configured)
}

// SIMULATOR-ONLY: DO NOT MOVE TO CORE.
pub(crate) fn find_actor<'a>(
    ctx: &'a SimContext,
    actor_name: &str,
) -> Result<&'a crate::SimActor, String> {
    let name = actor_name.to_ascii_lowercase();
    ctx.actors
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(&name))
        .ok_or_else(|| format!("stage4: actor not found in context: {actor_name}"))
}
