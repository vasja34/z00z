/// App-level service boundary.
///
/// The app owns wallets and orchestrates cross-wallet concerns.
pub struct AppService {
    core_app: Z00ZApp<
        AppKernel,
        NetworkService,
        Arc<ChainService>,
        Arc<dyn TimeProvider>,
        SystemRngProvider,
    >,
    wallets: Arc<WalletService>,
    chain_service: Arc<ChainService>,
}

impl Default for AppService {
    fn default() -> Self {
        Self::new()
    }
}
