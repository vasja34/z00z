use std::sync::{Mutex, OnceLock};

pub fn wallet_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct WalletEnvGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl WalletEnvGuard {
    pub fn new(network: &str, chain: &str) -> Self {
        let guard = wallet_env_lock().lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("Z00Z_WALLET_NETWORK", network);
        std::env::set_var("Z00Z_WALLET_CHAIN", chain);
        Self { _guard: guard }
    }
}

impl Drop for WalletEnvGuard {
    fn drop(&mut self) {
        std::env::remove_var("Z00Z_WALLET_NETWORK");
        std::env::remove_var("Z00Z_WALLET_CHAIN");
    }
}
