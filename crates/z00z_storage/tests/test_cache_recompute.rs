use std::sync::{Mutex, OnceLock};

use tempfile::tempdir;
use z00z_core::assets::{AssetLeaf, AssetPackPlain};
use z00z_crypto::ZkPackEncrypted;
use z00z_storage::settlement::{
    DefinitionId, SettlementLeafFamily, SettlementPath, SettlementStore, StoreItem, TerminalId,
    TerminalLeaf,
};

const HJMT_INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    bucket_bits: Option<String>,
    redb_root: Option<String>,
    inj_stage: Option<String>,
}

impl EnvGuard {
    fn with_bucket_bits(bits: &str) -> Self {
        let guard = Self {
            bucket_bits: std::env::var("Z00Z_SETTLEMENT_BUCKET_BITS").ok(),
            redb_root: std::env::var("Z00Z_STORAGE_REDB_ROOT").ok(),
            inj_stage: std::env::var(HJMT_INJ_STAGE_ENV).ok(),
        };
        std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        std::env::remove_var(HJMT_INJ_STAGE_ENV);
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(bits) = &self.bucket_bits {
            std::env::set_var("Z00Z_SETTLEMENT_BUCKET_BITS", bits);
        } else {
            std::env::remove_var("Z00Z_SETTLEMENT_BUCKET_BITS");
        }
        if let Some(root) = &self.redb_root {
            std::env::set_var("Z00Z_STORAGE_REDB_ROOT", root);
        } else {
            std::env::remove_var("Z00Z_STORAGE_REDB_ROOT");
        }
        if let Some(stage) = &self.inj_stage {
            std::env::set_var(HJMT_INJ_STAGE_ENV, stage);
        } else {
            std::env::remove_var(HJMT_INJ_STAGE_ENV);
        }
    }
}

fn path(definition: u8, serial: u32, asset: u8) -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new([definition; 32]),
        z00z_storage::settlement::SerialId::new(serial),
        TerminalId::new([asset; 32]),
    )
}

fn leaf(path: SettlementPath, value: u64) -> TerminalLeaf {
    let payload = AssetPackPlain {
        value,
        blinding: [3u8; 32],
        s_out: [4u8; 32],
    }
    .to_bytes();

    AssetLeaf {
        asset_id: path.terminal_id().into_bytes(),
        serial_id: path.serial_id.get(),
        r_pub: [1u8; 32],
        owner_tag: [2u8; 32],
        c_amount: [5u8; 32],
        enc_pack: ZkPackEncrypted {
            version: 1,
            ciphertext: payload,
            tag: [0u8; 16],
        },
        range_proof: vec![9u8; 4],
        tag16: 11,
    }
    .into()
}

fn item(path: SettlementPath, value: u64) -> StoreItem {
    StoreItem::new(path, leaf(path, value)).expect("store item")
}

fn put(store: &mut SettlementStore, path: SettlementPath, value: u64) {
    store
        .put_settlement_item(item(path, value))
        .expect("put settlement item");
}

#[cfg(debug_assertions)]
#[test]
fn test_cache_detects_corruption() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let target = path(51, 1, 1);
    put(&mut store, target, 5_101);
    let _proof = store
        .settlement_proof_blob(&target)
        .expect("settlement proof blob");
    let _absence = store
        .settlement_nonexistence_proof_blob(&path(51, 1, 9), SettlementLeafFamily::Terminal)
        .expect("absence proof");

    assert!(store.corrupt_forest_cache_for_test());
    let err = store
        .verify_forest_cache()
        .expect_err("corrupted cache must fail closed");
    assert!(err.to_string().contains("forest cache"));
}

#[cfg(debug_assertions)]
#[test]
fn test_cache_detects_journal_drift() {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let mut store = SettlementStore::new();
    let target = path(52, 1, 1);
    put(&mut store, target, 5_201);
    let _proof = store
        .settlement_proof_blob(&target)
        .expect("settlement proof blob");

    assert!(store.corrupt_journal_key_for_test());
    let err = store
        .verify_forest_cache()
        .expect_err("corrupted journal key must fail closed");
    assert!(err.to_string().contains("forest cache journal"));
}

#[test]
fn test_cache_rollback_clears_pending() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("1");
    let temp = tempdir()?;
    let seed = path(61, 1, 1);
    let pending = path(61, 1, 9);

    let mut seeded = SettlementStore::load(temp.path())?;
    put(&mut seeded, seed, 6_101);
    let baseline_root = seeded.settlement_root()?;
    drop(seeded);

    let mut interrupted = SettlementStore::load(temp.path())?;
    let absence_before =
        interrupted.settlement_nonexistence_proof_blob(&pending, SettlementLeafFamily::Terminal)?;
    std::env::set_var(HJMT_INJ_STAGE_ENV, "children");
    let err = interrupted
        .put_settlement_item(item(pending, 6_202))
        .expect_err("children-stage injection must fail");
    std::env::remove_var(HJMT_INJ_STAGE_ENV);
    assert!(err
        .to_string()
        .contains("hjmt journal injection after ChildrenCommitted"));
    assert_eq!(interrupted.settlement_root()?, baseline_root);
    assert!(interrupted.get_settlement_item(&pending)?.is_none());
    let absence_after =
        interrupted.settlement_nonexistence_proof_blob(&pending, SettlementLeafFamily::Terminal)?;
    assert_eq!(absence_after, absence_before);
    interrupted
        .verify_forest_cache()
        .expect("verify interrupted forest cache");
    drop(interrupted);

    let reloaded = SettlementStore::load(temp.path()).expect("reload after rollback");
    assert_eq!(
        reloaded.settlement_root().expect("reloaded root"),
        baseline_root
    );
    assert!(reloaded
        .get_settlement_item(&pending)
        .expect("reloaded pending item")
        .is_none());
    reloaded
        .verify_forest_cache()
        .expect("verify reloaded forest cache");
    Ok(())
}

#[test]
fn test_reload_keeps_cache_parity() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let _env = EnvGuard::with_bucket_bits("2");
    let temp = tempdir()?;
    let target = path(71, 1, 1);

    let mut seeded = SettlementStore::load(temp.path())?;
    put(&mut seeded, target, 7_101);
    put(&mut seeded, path(71, 2, 2), 7_102);
    put(&mut seeded, path(72, 1, 3), 7_103);
    let baseline = seeded.settlement_proof_blob(&target)?;
    let baseline_root = seeded.settlement_root()?;
    drop(seeded);

    let reloaded = SettlementStore::load(temp.path())?;
    let metrics = reloaded.forest_cache_metrics();
    assert!(metrics.subtree_root.entries > 0);
    assert!(metrics.parent_leaf.entries > 0);
    assert!(metrics.terminal_leaf.entries > 0);
    assert!(metrics.path_index.entries > 0);
    assert_eq!(reloaded.settlement_root()?, baseline_root);
    let hot = reloaded.settlement_proof_blob(&target)?;
    reloaded.validate_settlement_proof_blob(&hot)?;
    assert_eq!(hot, baseline);
    reloaded.verify_forest_cache()?;
    Ok(())
}
