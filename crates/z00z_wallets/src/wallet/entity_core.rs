/// Z00Z wallet - central wallet entity.
///
/// Aggregates all components according to the wallet architecture specification.
/// Total: 16 generic parameters (Secrets, Persistence, Spending, UX).
///
/// # Architecture
///
/// This struct follows the composition-over-inheritance pattern, injecting
/// all dependencies as trait implementations. This enables:
/// - Testability (mock implementations)
/// - Flexibility (swap implementations)
/// - Modularity (components are independent)
///
/// # Generic Parameters
///
/// - **Secrets & Keys (3):** `Sec`, `K`, `Addr`
/// - **Persistence (4):** `WStorage`, `Assets`, `Txs`, `Receipts`
/// - **Spending Pipeline (6):** `Sel`, `Fee`, `Asm`, `Sig`, `Prover`, `Ver`
/// - **UX Support (3):** `BackupExp`, `BackupImp`, `Pol`
///
/// # Phase 1 Limitations
///
/// Phase 1 intentionally keeps behavior minimal; methods will be introduced
/// incrementally as checklist items are implemented.
///
/// # Examples
///
/// ```rust,ignore
/// use z00z_wallets::wallet::Z00ZWallet;
/// use z00z_wallets::wallet::WalletId;
///
/// // Create wallet with all injected dependencies
/// let wallet = Z00ZWallet::new(
///     wallet_id,
///     secret_store,
///     key_manager,
///     // ... (20+ more parameters)
/// );
/// ```
pub struct Z00ZWallet<
    // Secrets & Keys
    Sec,
    K,
    Addr,
    // Persistence
    WStorage,
    Assets,
    Txs,
    Receipts,
    // Spending Pipeline
    Sel,
    Fee,
    Asm,
    Sig,
    Prover,
    Ver,
    // UX Support
    BackupExp,
    BackupImp,
    Pol,
> {
    /// Wallet kernel (always-present core state).
    pub(crate) kernel: WalletKernel,

    /// Secret storage.
    pub secret_store: Sec,
    /// Key manager.
    pub key_manager: K,
    /// Receiver manager.
    pub receiver_manager: Addr,

    /// Wallet metadata store.
    pub wallet_storage: WStorage,
    /// Asset storage.
    pub asset_storage: Assets,
    /// Transaction storage.
    pub tx_storage: Txs,
    /// Receipts storage.
    pub receipt_storage: Receipts,

    /// Asset selection for transaction building.
    pub asset_selector: Sel,
    /// Fee estimation.
    pub fee_estimator: Fee,
    /// Transaction assembler.
    pub tx_assembler: Asm,
    /// Signer.
    pub signer: Sig,
    /// Range proof generator (Bulletproofs+).
    pub prover: Prover,
    /// Local verifier.
    pub local_verifier: Ver,

    /// Backup exporter.
    pub backup_exporter: BackupExp,
    /// Backup importer.
    pub backup_importer: BackupImp,
    /// Policy engine.
    pub policy: Pol,

    /// Wallet state.
    pub state: Arc<RwLock<WalletState>>,
}

impl<
        Sec,
        K,
        Addr,
        WStorage,
        Assets,
        Txs,
        Receipts,
        Sel,
        Fee,
        Asm,
        Sig,
        Prover,
        Ver,
        BackupExp,
        BackupImp,
        Pol,
    >
    Z00ZWallet<
        Sec,
        K,
        Addr,
        WStorage,
        Assets,
        Txs,
        Receipts,
        Sel,
        Fee,
        Asm,
        Sig,
        Prover,
        Ver,
        BackupExp,
        BackupImp,
        Pol,
    >
{
    /// Get the stable wallet identifier.
    pub fn wallet_id(&self) -> &WalletId {
        self.kernel.wallet_id()
    }
}