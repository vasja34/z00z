use super::*;
use crate::backup::{decode_tx_history_rows, WalletTxHistoryEntryKind};
use crate::chain::ReceiverCardRecord;
use crate::db::test_owned_objects::{
    test_mandate_lock_payload, test_owned_right_payload, test_owned_voucher_payload,
};
use crate::db::OwnedObjectPayload;
use crate::receiver::StealthOutputScanner;
use crate::rpc::types::tx::{
    RuntimePaginationParams, RuntimeTxHistoryFilter, RuntimeTxHistorySort, RuntimeTxLifecycle,
};
use crate::rpc::types::wallet::PersistWalletSettings;
use crate::stealth::{build_seeded_output_bundle, SenderWallet};
use crate::wallet::policy::PolicyRules;
use crate::wallet::stub_defaults::StubDefault;
use crate::wallet::AutoLockPolicy;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tempfile::TempDir;
use z00z_core::assets::AssetClass;
use z00z_crypto::expert::encoding::SafePassword;
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_utils::config::{ConfigSource, EnvConfig};
use z00z_utils::rng::{SecureRngProvider, SystemRngProvider};
use z00z_utils::time::MockTimeProvider;

use super::tx_rpc_server::run_with_retry;

const TEST_SEED_PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TEST_WALLET_PASSWORD: &str = "StrongPassw0rd!";
const BASE_TIME_SECS: u64 = 1_703_260_800;
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn ms(secs: u64) -> u64 {
    secs.saturating_mul(1_000)
}

fn mock_time_with_offset(offset_secs: u64) -> Arc<MockTimeProvider> {
    Arc::new(MockTimeProvider::from_unix_secs(
        BASE_TIME_SECS + offset_secs,
    ))
}

fn tx_history_rows(ctx: &TestSessionCtx) -> Vec<crate::backup::WalletTxHistoryJsonlEntry> {
    let history_path = ctx.service.wallet_history_jsonl_path(&ctx.wallet_id);
    let bytes = z00z_utils::io::read_file(&history_path).expect("read tx-history JSONL");
    decode_tx_history_rows(&bytes).expect("decode tx-history rows")
}

fn tx_history_kinds(ctx: &TestSessionCtx) -> Vec<WalletTxHistoryEntryKind> {
    tx_history_rows(ctx)
        .into_iter()
        .map(|row| row.entry_kind)
        .collect()
}

fn env_value(key: &str) -> Option<String> {
    EnvConfig.get(key).ok().flatten()
}

struct TestEnvVarGuard {
    key: &'static str,
    previous: Option<String>,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl Drop for TestEnvVarGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn set_test_env_var(key: &'static str, value: &str) -> TestEnvVarGuard {
    let guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock");
    let previous = env_value(key);
    std::env::set_var(key, value);
    TestEnvVarGuard {
        key,
        previous,
        _guard: guard,
    }
}

struct WalletConfigEnvRestore {
    prev_path: Option<String>,
    prev_network: Option<String>,
    prev_chain: Option<String>,
}

impl WalletConfigEnvRestore {
    fn capture() -> Self {
        Self {
            prev_path: env_value("Z00Z_WALLET_CONFIG_PATH"),
            prev_network: env_value("Z00Z_WALLET_NETWORK"),
            prev_chain: env_value("Z00Z_WALLET_CHAIN"),
        }
    }
}

impl Drop for WalletConfigEnvRestore {
    fn drop(&mut self) {
        match &self.prev_path {
            Some(value) => std::env::set_var("Z00Z_WALLET_CONFIG_PATH", value),
            None => std::env::remove_var("Z00Z_WALLET_CONFIG_PATH"),
        }
        match &self.prev_network {
            Some(value) => std::env::set_var("Z00Z_WALLET_NETWORK", value),
            None => std::env::remove_var("Z00Z_WALLET_NETWORK"),
        }
        match &self.prev_chain {
            Some(value) => std::env::set_var("Z00Z_WALLET_CHAIN", value),
            None => std::env::remove_var("Z00Z_WALLET_CHAIN"),
        }
    }
}

fn assert_session_guard_error(err: ErrorObjectOwned) {
    assert!(
        matches!(err.code(), -32402 | -32403 | -32003),
        "expected session guard error, got {}: {}",
        err.code(),
        err.message()
    );
}

#[derive(Debug)]
struct RecordingWalletTxAdmitter {
    requests: Arc<Mutex<Vec<tx_rpc_admission::WalletTxAdmissionRequest>>>,
}

impl WalletTxAdmitter for RecordingWalletTxAdmitter {
    fn admit(
        &self,
        request: tx_rpc_admission::WalletTxAdmissionRequest,
    ) -> Result<tx_rpc_admission::WalletTxAdmissionReceipt, tx_rpc_admission::AdmissionError> {
        self.requests
            .lock()
            .expect("recording admitter lock")
            .push(request.clone());
        SimulatedWalletTxAdmitter.admit(request)
    }

    fn confirm(
        &self,
        receipt: &tx_rpc_admission::WalletTxAdmissionReceipt,
    ) -> Result<RuntimeConfirmationReceipt, tx_rpc_admission::AdmissionError> {
        SimulatedWalletTxAdmitter.confirm(receipt)
    }
}

fn rpc_with_recording_admitter(
    service: Arc<WalletService>,
    time_provider: Arc<MockTimeProvider>,
    requests: Arc<Mutex<Vec<tx_rpc_admission::WalletTxAdmissionRequest>>>,
) -> TxRpcImpl {
    TxRpcImpl {
        service,
        time_provider,
        tx_admitter: Arc::new(RecordingWalletTxAdmitter { requests }),
        tx_send_rate_limits: Arc::new(RwLock::new(Vec::new())),
        tx_build_rate_limits: Arc::new(RwLock::new(Vec::new())),
        idempotency_cache: Arc::new(RwLock::new(Vec::new())),
        pending_txs: Arc::new(RwLock::new(Vec::new())),
        pending_tx_bytes: Arc::new(RwLock::new(Vec::new())),
        confirmation_evidence: Arc::new(RwLock::new(Vec::new())),
        thin_index: Arc::new(RwLock::new(crate::tx::ThinIndexStore::new())),
        thin_cache: Arc::new(RwLock::new(crate::tx::ThinSnapshotCache::new())),
        tx_store: None,
    }
}

struct TestSessionCtx {
    service: Arc<WalletService>,
    session: SessionToken,
    wallet_id: PersistWalletId,
    _dir: TempDir,
    _wallet_config_lock: crate::rpc::logging::WalletConfigEnvLock,
    _wallet_config_restore: WalletConfigEnvRestore,
}

#[derive(Debug, Clone)]
struct SeqTxTestRng {
    seed: u64,
    counter: Arc<AtomicU64>,
}

impl SeqTxTestRng {
    fn new(seed: u64) -> Self {
        Self {
            seed,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl SecureRngProvider for SeqTxTestRng {
    type Rng = StdRng;

    fn rng(&self) -> Self::Rng {
        let next = self.counter.fetch_add(1, Ordering::Relaxed);
        StdRng::seed_from_u64(self.seed ^ next.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }
}

async fn setup_session(time: Arc<MockTimeProvider>) -> TestSessionCtx {
    setup_session_with_rng(time, SystemRngProvider).await
}

async fn setup_session_with_rng<P>(time: Arc<MockTimeProvider>, rng_provider: P) -> TestSessionCtx
where
    P: SecureRngProvider + Send + Sync + 'static,
{
    let wallet_config_lock =
        crate::rpc::logging::RpcLoggingConfig::__lock_wallet_config_env_async().await;
    let wallet_config_restore = WalletConfigEnvRestore::capture();
    std::env::remove_var("Z00Z_WALLET_CONFIG_PATH");
    std::env::remove_var("Z00Z_WALLET_NETWORK");
    std::env::remove_var("Z00Z_WALLET_CHAIN");

    let dir = tempfile::tempdir().expect("tempdir");
    let service = Arc::new(WalletService::create_service_custom_output_directory(
        dir.path().to_path_buf(),
        time.clone(),
        rng_provider,
    ));

    let password = SafePassword::from(TEST_WALLET_PASSWORD);
    let wallet_id = service
        .create_wallet_in_memory("w1", password.clone(), TEST_SEED_PHRASE_24)
        .await
        .expect("create_wallet_in_memory must succeed");

    let session = service
        .unlock_wallet_in_memory(&wallet_id, &password)
        .await
        .expect("unlock_wallet_in_memory must succeed");

    TestSessionCtx {
        service,
        session,
        wallet_id,
        _dir: dir,
        _wallet_config_lock: wallet_config_lock,
        _wallet_config_restore: wallet_config_restore,
    }
}

async fn seed_spendable_stealth_coin(ctx: &TestSessionCtx, amount: u64, serial_id: u32) {
    for offset in 0..3u32 {
        let current_serial = serial_id.saturating_add(offset);
        let mut asset = z00z_core::genesis::asset_std::asset_from_dev_class(
            AssetClass::Coin,
            current_serial,
            amount,
        )
        .expect("valid std asset");
        let recv_keys = ctx
            .service
            .receiver_keys(&ctx.wallet_id)
            .await
            .expect("receiver keys");
        let card = recv_keys.export_receiver_card().expect("receiver card");
        let tx_seed = [41u8; 32];
        let tx_digest = [current_serial as u8; 32];
        let mut sender_wallet = SenderWallet::new(tx_seed);
        let output = build_seeded_output_bundle(
            ctx.wallet_id.0.clone(),
            crate::tx::TxOutRole::Recipient,
            AssetClass::Coin,
            &card,
            None,
            &mut sender_wallet,
            &tx_digest,
            asset.serial_id,
            [current_serial as u8; 32],
            asset.amount,
            &asset.definition.id,
            asset.serial_id,
        )
        .expect("stealth output");

        asset.commitment = z00z_crypto::Commitment::from_bytes(&output.leaf.c_amount)
            .expect("commitment")
            .0;
        asset.owner_pub = None;
        asset.owner_signature = None;
        asset.r_pub = Some(output.leaf.r_pub);
        asset.owner_tag = Some(output.leaf.owner_tag);
        asset.enc_pack = Some(output.leaf.enc_pack);
        asset.tag16 = Some(output.leaf.tag16);
        asset.leaf_ad_id = Some(output.leaf.asset_id);
        asset.range_proof = Some(output.leaf.range_proof);

        ctx.service
            .put_claimed_asset(&ctx.wallet_id, asset)
            .await
            .expect("seed put_claimed_asset must succeed");
    }
}

async fn mk_recv_card_compact(ctx: &TestSessionCtx) -> String {
    let recv_keys = ctx
        .service
        .receiver_keys(&ctx.wallet_id)
        .await
        .expect("receiver keys");
    let card = recv_keys.export_receiver_card().expect("receiver card");
    ReceiverCardRecord::new(&card, card.canonical_encoding(), 0)
        .expect("receiver record")
        .to_compact()
        .expect("receiver compact")
}

mod test_tx_impl_suite;
