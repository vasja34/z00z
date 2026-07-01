use std::sync::{Mutex, OnceLock};

use tempfile::tempdir;
use z00z_storage::settlement::SettlementStore;

const BACKEND_ENV: &str = "Z00Z_SETTLEMENT_BACKEND_MODE";
const SERIALIZATION_MOD: &str = include_str!("../src/serialization/mod.rs");

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn assert_mode_rejects(raw: &str) {
    let temp = tempdir().expect("tempdir");
    std::env::set_var(BACKEND_ENV, raw);
    let err = match SettlementStore::load(temp.path()) {
        Ok(_) => panic!("unsupported mode must reject: {raw}"),
        Err(err) => err,
    };
    let text = err.to_string();
    assert!(
        text.contains("unsupported settlement backend mode"),
        "unexpected error for {raw}: {text}"
    );
    assert!(
        !text.contains(raw),
        "reject message must stay redacted for {raw}: {text}"
    );
}

#[test]
fn test_hjmt_default_mode() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let temp = tempdir()?;
    std::env::remove_var(BACKEND_ENV);
    let store = SettlementStore::load(temp.path())?;
    assert_eq!(store.backend_name(), "hjmt");
    Ok(())
}

#[test]
fn test_ok_explicit_hjmt() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let temp = tempdir()?;
    std::env::set_var(BACKEND_ENV, "hjmt");
    let store = SettlementStore::load(temp.path())?;
    assert_eq!(store.backend_name(), "hjmt");
    std::env::remove_var(BACKEND_ENV);
    Ok(())
}

#[test]
fn test_rejects_unknown_modes() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    for raw in ["unknown", "settlement-v2"] {
        assert_mode_rejects(raw);
    }
    std::env::remove_var(BACKEND_ENV);
}

#[test]
fn test_rejects_stale_alias_modes() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    for raw in ["compatibility", "forest", "dual-verify"] {
        assert_mode_rejects(raw);
    }
    std::env::remove_var(BACKEND_ENV);
}

#[test]
fn test_exports_live_builder() {
    assert!(
        !SERIALIZATION_MOD.contains("#[cfg(feature = \"test-params-fast\")]")
            && SERIALIZATION_MOD.contains("pub use self::build::build_artifact;"),
        "live serialization surface must re-export the settlement-native artifact builder"
    );
}
