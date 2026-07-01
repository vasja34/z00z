use super::{
    build_rpc_file_logger, compute_wallet_file_id, path_exists, register_all_wallet_rpc_methods,
    AppRpcImpl, AppService, Arc, AssetRpcImpl, AtomicU64, BackupRpcImpl, ChainRpcImpl,
    ChainScanRpcImpl, KeyRpcImpl, LocalRpcTransport, LoggedRpcTransport, MockTimeProvider,
    NetworkRpcImpl, Ordering, Path, PersistWalletId, RpcDispatcher, RpcLoggingConfig, SimContext,
    StorageRpcImpl, SystemRngProvider, SystemTimeProvider, TimeProvider, TxRpcImpl, WalletRpcImpl,
    WalletService, WalletSource,
};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use z00z_wallets::rpc::types::wallet::WalletLifecycleEvent;

const WALLET_SOURCE_REOPEN_RETRIES: u32 = 200;
const WALLET_SOURCE_REOPEN_WAIT_MS: u64 = 50;

#[derive(Clone)]
struct SeqSecureRngProvider {
    seed: u64,
    counter: Arc<AtomicU64>,
}

impl SeqSecureRngProvider {
    fn new(seed: u64) -> Self {
        Self {
            seed,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    fn next_seed(&self) -> u64 {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        self.seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15)
    }
}

impl z00z_utils::rng::SecureRngProvider for SeqSecureRngProvider {
    type Rng = rand::rngs::StdRng;

    fn rng(&self) -> Self::Rng {
        use rand::SeedableRng;
        // Simulator-only seeded adapter: WalletService currently expects a
        // SecureRngProvider, but this implementation is still bounded to the
        // mock reproducibility lane and must never be treated as universal
        // production entropy.
        rand::rngs::StdRng::seed_from_u64(self.next_seed())
    }
}

// SIMULATOR-ONLY: DO NOT MOVE TO CORE.
pub(crate) fn build_logged_transport(
    ctx: &SimContext,
    wallets_dir: &Path,
    rpc_log_path: &Path,
) -> Result<
    (
        Arc<WalletService>,
        LoggedRpcTransport<LocalRpcTransport, SystemRngProvider>,
    ),
    String,
> {
    let wallet_svc = if ctx.config.simulation.use_mock_rng {
        let time = Arc::new(MockTimeProvider::default());
        let seed = ctx.config.simulation.mock_rng_seed.unwrap_or(0);
        // This audited stage-2 fixture keeps `None` as a deterministic
        // zero-seed fallback instead of advertising a repo-wide selector rule.
        // `None` inside the mock path is the explicit zero-seed reproducibility
        // fallback, not a request for production randomness.
        time.set_unix_millis(seed);
        let time_provider: Arc<dyn TimeProvider> = time;
        let entropy_rng = SeqSecureRngProvider::new(seed);
        Arc::new(WalletService::create_service_custom_output_directory(
            wallets_dir.to_path_buf(),
            time_provider,
            entropy_rng,
        ))
    } else {
        Arc::new(WalletService::with_output_dir(wallets_dir.to_path_buf()))
    };
    let transport = build_logged_transport_with_wallet(Arc::clone(&wallet_svc), rpc_log_path)?;

    Ok((wallet_svc, transport))
}

pub(crate) async fn lock_existing_wallet_sessions(ctx: &SimContext) -> Result<(), String> {
    let Some(wallet_svc) = &ctx.wallet_service else {
        return Ok(());
    };

    wallet_svc
        .on_lifecycle_event(WalletLifecycleEvent::Backgrounded)
        .await
        .map_err(|e| format!("background lock all wallets: {e}"))?;

    for actor in &ctx.actors {
        let wallet_id = PersistWalletId(actor.wallet_id.clone());
        wallet_svc
            .lock_wallet(&wallet_id)
            .await
            .map_err(|e| format!("lock_wallet({}): {e}", actor.name))?;
    }

    Ok(())
}

pub(crate) fn wallet_source_path(wallets_dir: &Path, wallet_id: &str) -> PathBuf {
    let wallet_hash = compute_wallet_file_id(wallet_id);
    let wallet_stem = hex::encode(&wallet_hash[..8]);
    wallets_dir.join(format!("wallet_{wallet_stem}.wlt"))
}

pub(crate) async fn reopen_wallet_sources(
    ctx: &SimContext,
    wallet_svc: &Arc<WalletService>,
    wallets_dir: &Path,
) -> Result<(), String> {
    for actor in &ctx.actors {
        let wlt_path = wallet_source_path(wallets_dir, &actor.wallet_id);
        let mut found = false;
        for attempt in 0..=WALLET_SOURCE_REOPEN_RETRIES {
            if path_exists(&wlt_path).map_err(|e| e.to_string())? {
                found = true;
                break;
            }
            if attempt < WALLET_SOURCE_REOPEN_RETRIES {
                sleep(Duration::from_millis(WALLET_SOURCE_REOPEN_WAIT_MS));
            }
        }
        if !found {
            let mut entries = Vec::new();
            if let Ok(dir_entries) = z00z_utils::io::read_dir(wallets_dir) {
                for entry in dir_entries {
                    if let Some(name) = entry.file_name().and_then(|value| value.to_str()) {
                        entries.push(name.to_string());
                    }
                }
                entries.sort();
            }
            return Err(format!(
                "wallet source missing for actor {}: {}; dir_entries=[{}]",
                actor.name,
                wlt_path.display(),
                entries.join(",")
            ));
        }
        wallet_svc
            .open_wallet_source(WalletSource::Path {
                path: wlt_path.to_string_lossy().to_string(),
            })
            .await
            .map_err(|e| format!("open_wallet_source({}): {e}", actor.name))?;
    }

    Ok(())
}

// SIMULATOR-ONLY: DO NOT MOVE TO CORE.
pub(crate) fn build_logged_transport_with_wallet(
    wallet_svc: Arc<WalletService>,
    rpc_log_path: &Path,
) -> Result<LoggedRpcTransport<LocalRpcTransport, SystemRngProvider>, String> {
    let app_svc = Arc::new(AppService::with_wallet_service(Arc::clone(&wallet_svc)));
    let app_rpc = Arc::new(AppRpcImpl::new(Arc::clone(&app_svc)));
    let wallet_rpc = Arc::new(WalletRpcImpl::new(Arc::clone(&wallet_svc)));
    let asset_rpc = Arc::new(AssetRpcImpl::with_wallet_service(Arc::clone(&wallet_svc)));
    let tx_rpc = Arc::new(TxRpcImpl::new(Arc::clone(&wallet_svc)));
    let backup_rpc = Arc::new(BackupRpcImpl::new(Arc::clone(&wallet_svc)));
    let key_rpc = Arc::new(KeyRpcImpl::new(Arc::clone(&wallet_svc)));
    let chain_rpc = Arc::new(ChainRpcImpl::new(Arc::clone(&app_svc)));
    let network_rpc = Arc::new(NetworkRpcImpl::with_app_service(Arc::clone(&app_svc)));
    let scan_rpc = Arc::new(ChainScanRpcImpl::new(Arc::clone(&app_svc)));
    let storage_rpc = Arc::new(StorageRpcImpl::new(Arc::clone(&wallet_svc)));

    let dispatcher = Arc::new(RpcDispatcher::new());
    register_all_wallet_rpc_methods(
        &dispatcher,
        app_rpc,
        wallet_rpc,
        asset_rpc,
        tx_rpc,
        backup_rpc,
        key_rpc,
        chain_rpc,
        network_rpc,
        scan_rpc,
        storage_rpc,
    )
    .map_err(|e| e.to_string())?;

    let mut logging_cfg = RpcLoggingConfig::from_default_wallet_yaml()
        .map_err(|e| format!("failed to load RPC logging config: {e}"))?;
    logging_cfg.output.path = rpc_log_path.to_string_lossy().to_string();
    logging_cfg.enabled = true;

    let logger = build_rpc_file_logger(&logging_cfg)
        .map_err(|e| format!("failed to initialize RPC log sink: {e}"))?;
    let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
    let rpc_rng = SystemRngProvider;

    let transport = LoggedRpcTransport::new(
        LocalRpcTransport::new(dispatcher),
        logging_cfg,
        logger,
        time_provider,
        rpc_rng,
    );

    Ok(transport)
}
