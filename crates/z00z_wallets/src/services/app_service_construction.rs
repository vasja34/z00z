impl AppService {
    fn normalize_mnemonic(input: &str) -> String {
        input
            .split_whitespace()
            .map(|w| w.trim().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Create a new `AppService` with default wallet and chain services.
    pub fn new() -> Self {
        let time_provider: Arc<dyn TimeProvider> = Arc::new(SystemTimeProvider);
        let wallets = Arc::new(WalletService::with_dependencies(Arc::clone(&time_provider)));

        Self::with_dependencies(time_provider, wallets)
    }

    /// Create `AppService` backed by an existing wallet service.
    pub fn with_wallet_service(wallets: Arc<WalletService>) -> Self {
        Self::with_dependencies_and_services(
            Arc::new(SystemTimeProvider),
            wallets,
            Arc::new(ChainService::new()),
        )
    }

    /// Create `AppService` with injected dependencies.
    pub fn with_dependencies(
        time_provider: Arc<dyn TimeProvider>,
        wallets: Arc<WalletService>,
    ) -> Self {
        Self::with_dependencies_and_services(time_provider, wallets, Arc::new(ChainService::new()))
    }

    /// Create `AppService` with injected dependencies and services.
    pub fn with_dependencies_and_services(
        time_provider: Arc<dyn TimeProvider>,
        wallets: Arc<WalletService>,
        chain_service: Arc<ChainService>,
    ) -> Self {
        let network_service = NetworkService::new();
        Self {
            core_app: Z00ZApp::new(
                AppKernel::new(),
                network_service,
                Arc::clone(&chain_service),
                time_provider,
                SystemRngProvider,
            ),
            wallets,
            chain_service,
        }
    }
}
