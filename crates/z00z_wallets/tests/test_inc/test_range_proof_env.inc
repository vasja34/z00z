use std::{
    ffi::OsString,
    sync::{Mutex, OnceLock},
};

pub const ALLOW_DEBUG_RANGE_PROOF: &str = "Z00Z_ALLOW_DEBUG_RANGE_PROOF";

pub fn range_proof_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct RangeProofEnvGuard {
    _guard: std::sync::MutexGuard<'static, ()>,
    prev: Option<OsString>,
}

impl RangeProofEnvGuard {
    pub fn new() -> Self {
        let guard = range_proof_lock().lock().unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var_os(ALLOW_DEBUG_RANGE_PROOF);
        std::env::set_var(ALLOW_DEBUG_RANGE_PROOF, "1");
        Self {
            _guard: guard,
            prev,
        }
    }
}

impl Default for RangeProofEnvGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RangeProofEnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.prev.take() {
            std::env::set_var(ALLOW_DEBUG_RANGE_PROOF, value);
        } else {
            std::env::remove_var(ALLOW_DEBUG_RANGE_PROOF);
        }
    }
}
