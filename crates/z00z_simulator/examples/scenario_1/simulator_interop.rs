use std::{
    fs,
    path::{Path, PathBuf},
    process,
    sync::Arc,
    sync::OnceLock,
};

use thiserror::Error;
use z00z_simulator::scenario_1::support::fixture_cache;
use z00z_simulator::{config::ScenarioCfg, scenario_1::runner};
use z00z_utils::{
    io::create_dir_all,
    time::{SystemTimeProvider, TimeProvider},
};
use z00z_wallets::{key::ReceiverKeys, rpc::types::common::PersistWalletId, WalletService};

#[path = "simulator_interop_support.inc"]
mod simulator_interop_support;

use simulator_interop_support::{
    ensure_stage4_ok, load_cfg, load_rows, open_bob_wallet, print_foreign, print_row, scan_asset,
    write_cfg,
};

const BOB_PASS: &str = "Bob_Pass_Z00Z_43!";

#[derive(Debug, Error)]
pub enum DemoError {
    #[error("io failure: {0}")]
    Io(String),
    #[error("decode failure: {0}")]
    Decode(String),
    #[error("scenario failure: {0}")]
    Scenario(String),
    #[error("wallet failure: {0}")]
    Wallet(String),
    #[error("missing simulator artifact: {0}")]
    Missing(String),
}

struct RunCase {
    out: PathBuf,
}

struct WalletContext {
    svc: Arc<WalletService>,
    id: PersistWalletId,
    keys: ReceiverKeys,
}

struct WirePick {
    label: &'static str,
    leaf: z00z_core::assets::AssetLeaf,
    asset: z00z_core::Asset,
}

pub fn run_demo() -> Result<(), DemoError> {
    println!(
        "note: this example is separate from the pure simulator scenario because it proves that wallet public receive paths can consume simulator artifacts without any semantic rewrite."
    );

    let run = run_case()?;
    let bob = open_bob_wallet(&run.out)?;

    let (claim, owned, foreign) = load_rows(&bob, &run.out)?;

    let claim_scan = scan_asset(&bob, &claim.asset)?;
    let owned_scan = scan_asset(&bob, &owned.asset)?;
    let foreign_scan = scan_asset(&bob, &foreign.asset)?;

    print_row(&claim, &claim_scan)?;
    print_row(&owned, &owned_scan)?;

    print_foreign(&foreign, &foreign_scan)?;

    lock_one(&bob)?;
    Ok(())
}

#[cfg(not(test))]
fn main() -> Result<(), DemoError> {
    run_demo()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn work_base(name: &str) -> Result<PathBuf, DemoError> {
    let now = SystemTimeProvider.compat_unix_timestamp_micros();
    let path = repo_root()
        .join("target/phase7")
        .join(format!("{name}_{}_{}", process::id(), now));
    create_dir_all(&path).map_err(|e| DemoError::Io(e.to_string()))?;
    Ok(path)
}

fn after_file(out: &Path) -> PathBuf {
    out.join("transactions/wallets_state_after.json")
}

fn tx_file(out: &Path) -> PathBuf {
    out.join("transactions/tx_alice_to_bob_pkg.json")
}

fn claim_file(out: &Path) -> PathBuf {
    out.join("claim/tx_claim_pkg.json")
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), DemoError> {
    fs::create_dir_all(dst).map_err(|e| DemoError::Io(e.to_string()))?;
    for entry in fs::read_dir(src).map_err(|e| DemoError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| DemoError::Io(e.to_string()))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let ty = entry
            .file_type()
            .map_err(|e| DemoError::Io(e.to_string()))?;
        if ty.is_dir() {
            copy_tree(&src_path, &dst_path)?;
        } else if ty.is_file() {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).map_err(|e| DemoError::Io(e.to_string()))?;
            }
            fs::copy(&src_path, &dst_path).map_err(|e| DemoError::Io(e.to_string()))?;
        }
    }
    Ok(())
}

fn shared_stage4_out() -> Result<PathBuf, DemoError> {
    static OUT: OnceLock<PathBuf> = OnceLock::new();
    Ok(OUT
        .get_or_init(|| {
            let root =
                fixture_cache::ensure_shared_case("simulator_interop_stage4_shared_v1", |base| {
                    let out = base.join("outputs/scenario_1");
                    let mut cfg = load_cfg(&out).expect("load interop cfg");
                    tune_cfg(&mut cfg);

                    let cfg_path = base.join("scenario_config.yaml");
                    write_cfg(&cfg_path, &cfg).expect("write interop cfg");

                    let design_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("src/scenario_1/scenario_design.yaml");
                    let run = runner::run_with_paths(&cfg_path, &design_path)
                        .expect("shared interop scenario run");
                    ensure_stage4_ok(&run).expect("shared interop stage4");
                });
            root.join("outputs/scenario_1")
        })
        .clone())
}

fn run_case() -> Result<RunCase, DemoError> {
    let base = work_base("sim_interop")?;
    let out = base.join("outputs/scenario_1");
    let shared = shared_stage4_out()?;
    copy_tree(&shared, &out)?;

    Ok(RunCase { out })
}

fn tune_cfg(cfg: &mut ScenarioCfg) {
    if let Some(stage3) = cfg.stage3_claim.as_mut() {
        stage3.consume_bins = Some(false);
    }

    if let Some(stage4) = cfg.stage4_tx_prepare.as_mut() {
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
}

fn lock_one(wallet: &WalletContext) -> Result<(), DemoError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| DemoError::Wallet(e.to_string()))?;
    rt.block_on(async {
        wallet
            .svc
            .lock_wallet(&wallet.id)
            .await
            .map_err(|e| DemoError::Wallet(e.to_string()))
    })
}
