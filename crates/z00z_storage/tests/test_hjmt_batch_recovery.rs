use tempfile::tempdir;
use z00z_core::assets::AssetLeaf;
use z00z_storage::fixture_support::settlement_corpus::{
    next_policy, split_ready_paths, HjmtEnvGuard,
};
use z00z_storage::settlement::{SettlementPath, SettlementStore, StoreItem, TerminalLeaf};

const INJ_STAGE_ENV: &str = "Z00Z_STORAGE_HJMT_INJ_STAGE";

fn item(path: SettlementPath, mark: u8) -> StoreItem {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id().into_bytes();
    core.r_pub = [mark; 32];
    core.owner_tag = [mark.wrapping_add(1); 32];
    core.c_amount = [mark.wrapping_add(2); 32];
    core.range_proof = vec![mark; 8];
    StoreItem::new(path, TerminalLeaf::from(core)).expect("store item")
}

fn path(def_mark: u8, serial: u32, term_mark: u8) -> SettlementPath {
    SettlementPath::new(
        z00z_storage::settlement::DefinitionId::new([def_mark; 32]),
        z00z_storage::settlement::SerialId::new(serial),
        z00z_storage::settlement::TerminalId::new([term_mark; 32]),
    )
}

#[test]
fn test_children_recovery_keeps_transition() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = HjmtEnvGuard::with_bits("1");
    let temp = tempdir()?;

    let mut store = SettlementStore::load(temp.path())?;
    let paths = split_ready_paths(&mut store, 41, 9);
    let anchor = paths[0];
    let split = store.split_proof(&anchor)?;
    let policy = next_policy(&store);
    let transition = store.policy_transition_proof(policy)?;
    let root = store.settlement_root()?;

    std::env::set_var(INJ_STAGE_ENV, "children");
    let err = store
        .put_settlement_item(item(path(42, 10, 201), 0x91))
        .expect_err("children-stage injection must fail");
    assert!(err
        .to_string()
        .contains("hjmt journal injection after ChildrenCommitted"));
    std::env::remove_var(INJ_STAGE_ENV);
    drop(store);

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(reloaded.settlement_root()?, root);
    reloaded.validate_split_proof(&split)?;
    reloaded.validate_policy_transition_proof(&transition, policy)?;
    assert!(reloaded.get_settlement_item(&path(42, 10, 201))?.is_none());

    Ok(())
}

#[test]
fn test_parent_recovery_keeps_root() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = HjmtEnvGuard::with_bits("1");
    let temp = tempdir()?;

    let seed_path = path(34, 10, 1);
    let pending_path = path(35, 11, 2);

    let mut store = SettlementStore::load(temp.path())?;
    let seed_root = store.put_settlement_item(item(seed_path, 0x51))?;

    std::env::set_var(INJ_STAGE_ENV, "parents");
    let err = store
        .put_settlement_item(item(pending_path, 0x52))
        .expect_err("parent-stage injection must fail");
    assert!(err
        .to_string()
        .contains("hjmt journal injection after ParentsCommitted"));
    std::env::remove_var(INJ_STAGE_ENV);
    drop(store);

    let recovered = SettlementStore::load(temp.path())?;
    let recovered_root = recovered.settlement_root()?;
    let recovery = recovered.recovery_state()?;
    assert_ne!(recovered_root, seed_root);
    assert_eq!(recovered_root, recovery.state_root);
    assert!(recovered.get_settlement_item(&seed_path)?.is_some());
    assert!(recovered.get_settlement_item(&pending_path)?.is_some());
    drop(recovered);

    let reloaded = SettlementStore::load(temp.path())?;
    assert_eq!(reloaded.settlement_root()?, recovered_root);
    assert!(reloaded.get_settlement_item(&seed_path)?.is_some());
    assert!(reloaded.get_settlement_item(&pending_path)?.is_some());

    Ok(())
}
