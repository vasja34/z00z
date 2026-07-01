use super::wallet_service_reachability::ReachabilityWallet;
use super::wallet_service_state::{
    BackupCreateRateLimitState, RateLimitWindowState, UnlockAttemptState,
    WalletPasswordVerifierState, WalletReceiverDeriverState,
};
use super::{
    Arc, Asset, AutoLockPolicy, BTreeMap, Bip44Path, Duration, Future, PathBuf,
    PersistAuditLogEntry, PersistBackupSettings, PersistWalletId, PersistWalletSettings, Pin,
    ReceiverDeriverState, RngCoreExt, RwLock, SecureRngProvider, TimeProvider, WalletIdentity,
    WalletResult, WalletSessionManager, WalletState, WltStore,
};

pub(super) type ReceiverLabelList = Vec<(String, String)>;
type ReceiverLabelsStore = BTreeMap<PersistWalletId, ReceiverLabelList>;
pub(super) type WalletReceiverDeriverHandle = Arc<RwLock<WalletReceiverDeriverState>>;
/// Async callback used by `WalletService` to check whether a derived receiver was already used.
pub type ReceiverUsageOracle = Arc<
    dyn Fn(Bip44Path, [u8; 32]) -> Pin<Box<dyn Future<Output = WalletResult<bool>> + Send>>
        + Send
        + Sync,
>;

pub(crate) trait WalletEntropy: Send + Sync {
    fn fill_bytes(&self, dest: &mut [u8]);
}

/// Async sleep abstraction used by wallet service rate-limit and retry flows.
pub trait Sleeper: Send + Sync {
    /// Sleep for the requested duration.
    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TokioSleeper;

impl Sleeper for TokioSleeper {
    fn sleep<'a>(&'a self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(tokio::time::sleep(duration))
    }
}

#[derive(Debug, Clone)]
pub(super) struct WalletEntropyFromRngProvider<P> {
    provider: P,
}

impl<P> WalletEntropyFromRngProvider<P> {
    pub(super) fn new(provider: P) -> Self {
        Self { provider }
    }
}

impl<P> WalletEntropy for WalletEntropyFromRngProvider<P>
where
    P: SecureRngProvider + Send + Sync,
{
    fn fill_bytes(&self, dest: &mut [u8]) {
        let mut rng = self.provider.rng();
        rng.fill_bytes_ext(dest);
    }
}

/// Wallet service boundary for persisted wallet state and wallet-local policy.
///
/// This service owns the live wallet-facing authority lanes used by the app
/// and RPC layers, including `.wlt` persistence, JSONL sidecars, receive
/// status projection, and restart-safe policy state.
pub struct WalletService {
    /// Time provider abstraction (ONE SOURCE OF TRUTH)
    pub(super) time_provider: Arc<dyn TimeProvider>,

    pub(super) sleeper: Arc<dyn Sleeper>,

    /// RNG boundary for business logic.
    ///
    /// Phase 1: used for wallet id entropy, password verifier salts, and session tokens.
    pub(super) entropy: Arc<dyn WalletEntropy>,
    /// Auto-lock policy configuration
    pub(super) auto_lock_policy: AutoLockPolicy,
    /// Receiver derivation cache size captured at service startup.
    pub(super) receiver_cache_size: usize,
    /// Receiver derivation rate limit captured at service startup.
    pub(super) receiver_derive_rate_limit:
        Option<crate::services::wallet_runtime_config::ReceiverDeriveRateLimit>,
    /// In-memory wallet states (for auto-lock tracking)
    /// BTreeMap for O(log n) lookups.
    pub(super) wallet_states: Arc<RwLock<BTreeMap<PersistWalletId, WalletState>>>,

    /// In-memory wallet settings.
    /// BTreeMap for O(log n) lookups.
    pub(super) wallet_settings: Arc<RwLock<BTreeMap<PersistWalletId, PersistWalletSettings>>>,

    /// In-memory unlock attempt tracking for Phase 1.
    ///
    /// Used to enforce rate limiting and exponential backoff for `wallet.unlock`
    /// without introducing persistent infrastructure.
    /// BTreeMap for O(log n) lookups.
    pub(super) unlock_attempts: Arc<RwLock<BTreeMap<PersistWalletId, UnlockAttemptState>>>,

    /// In-memory rate limit tracking for `wallet.show_seed_phrase` (Phase 1).
    ///
    /// This is intentionally ephemeral and per-process.
    /// BTreeMap for O(log n) lookups.
    pub(super) show_seed_phrase_limits:
        Arc<RwLock<BTreeMap<PersistWalletId, RateLimitWindowState>>>,

    /// In-memory rate limit tracking for `wallet.key.rotate_master_key`.
    ///
    /// This is intentionally ephemeral and per-process.
    /// BTreeMap for O(log n) lookups.
    pub(super) rotate_master_key_limits:
        Arc<RwLock<BTreeMap<PersistWalletId, RateLimitWindowState>>>,

    /// In-memory rate limit tracking for `key.derive` (Phase 1).
    ///
    /// This is intentionally ephemeral and per-process.
    pub(super) key_derive_limits: Arc<RwLock<BTreeMap<PersistWalletId, RateLimitWindowState>>>,

    /// In-memory rate limit tracking for `wallet.backup.create_backup` (Phase 1).
    ///
    /// This is intentionally ephemeral and per-process.
    pub(super) backup_create_limits:
        Arc<RwLock<BTreeMap<PersistWalletId, BackupCreateRateLimitState>>>,

    /// In-memory backup settings per wallet (Phase 1).
    pub(super) backup_settings: Arc<RwLock<BTreeMap<PersistWalletId, PersistBackupSettings>>>,

    /// In-memory receiver labels store (Phase 1).
    ///
    /// Stored per wallet id.
    pub(super) receiver_labels: Arc<RwLock<ReceiverLabelsStore>>,

    /// In-memory audit log store for key-related operations (Phase 1).
    ///
    /// This is process-local and intended only for RPC testing.
    pub(super) key_audit_logs: Arc<RwLock<Vec<PersistAuditLogEntry>>>,

    /// In-memory address derivation state (Phase 1).
    ///
    /// Phase 1 implementation:
    /// - ephemeral (in-memory only)
    /// - BTreeMap for O(log n) lookups
    /// - uses existing KeyManager + ReceiverManager implementations
    ///
    /// Performance:
    /// - map lock is only held for get/insert
    /// - each wallet's derivation state has its own lock to reduce cross-wallet contention
    pub(super) wallet_receiver_derivers:
        Arc<RwLock<BTreeMap<PersistWalletId, WalletReceiverDeriverHandle>>>,

    /// Persisted derivation counters (Phase 2).
    ///
    /// Stored in wallet profile payloads and restored on wallet load.
    /// Kept separately from `wallet_receiver_derivers` because deriver state is created lazily
    /// and is dropped on lock.
    pub(super) wallet_receiver_deriver_counters:
        Arc<RwLock<BTreeMap<PersistWalletId, ReceiverDeriverState>>>,

    /// Persisted wallet identity (network + chain) per wallet.
    ///
    /// This is used to validate unlock/open against the identity stored in the `.wlt` meta,
    /// without depending on process-level environment configuration.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) wallet_identities: Arc<RwLock<BTreeMap<PersistWalletId, WalletIdentity>>>,

    /// In-memory password verifier store (Phase 1).
    ///
    /// This provides password confirmation for sensitive RPC operations without
    /// introducing persistent infrastructure.
    /// BTreeMap for O(log n) lookups.
    pub(super) wallet_password_verifiers:
        Arc<RwLock<BTreeMap<PersistWalletId, WalletPasswordVerifierState>>>,

    /// Persisted wallet-owned seed salts keyed by wallet id.
    pub(super) wallet_seed_salts: Arc<RwLock<BTreeMap<PersistWalletId, [u8; 16]>>>,

    /// In-memory wallet sessions (Phase 1).
    ///
    /// Phase 1 contract: unlock/open creates a single in-memory owner for key-bearing state.
    /// The session is keyed by wallet id and returns a session token to the caller.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) wallet_sessions: WalletSessionManager,

    /// Output directory for wallet persistence.
    ///
    /// Wallets are persisted as `.wlt` files:
    /// - `{output_dir}/wallet_{wallet_id_hash}.wlt`
    pub(super) output_dir: PathBuf,

    /// `.wlt` persistence boundary.
    ///
    /// Keeps RedB-specific details out of the service layer.
    pub(super) wlt_store: Arc<dyn WltStore>,

    /// In-memory wallet names mapping (Phase 1.5).
    ///
    /// Maps WalletId to user-provided wallet name.
    /// BTreeMap for O(log n) lookups.
    pub(super) wallet_names: Arc<RwLock<BTreeMap<PersistWalletId, String>>>,

    /// In-memory claimed assets mirrored from wallet-native owned-asset records.
    pub(super) wallet_claimed_assets: Arc<RwLock<BTreeMap<PersistWalletId, Vec<Asset>>>>,

    /// Last wallet-local receive outcome per wallet.
    ///
    /// This augments the public scan-status DTO without introducing a second
    /// persistence authority for receive progress or assets.
    pub(super) last_receive_scan_outcomes:
        Arc<RwLock<BTreeMap<PersistWalletId, crate::rpc::types::chain::RuntimeReceiveScanOutcome>>>,

    /// Validation-only core wallet instance used to prove RPC -> Service -> Core reachability.
    ///
    /// This is intentionally a minimal validation-only wallet with unit `()`
    /// components. It is reached via dedicated structural reachability probes
    /// and does not own persisted wallet, receive, or tx-history authority.
    pub(super) reachability_wallet: ReachabilityWallet,
}
