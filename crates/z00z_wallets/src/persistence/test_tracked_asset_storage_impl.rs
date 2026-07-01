use super::*;
use z00z_core::Asset;

fn create_test_asset(_tx_hash: &str, _output_index: u32) -> Asset {
    use std::sync::Arc;
    let mut asset = z00z_core::genesis::asset_std::asset_from_dev_cfg("z00z", 0, 1000)
        .expect("valid std asset");
    let mut definition = (*asset.definition).clone();
    definition.id = [0u8; 32];
    asset.definition = Arc::new(definition);

    let mut nonce = [0u8; 32];
    nonce[0] = _output_index as u8;
    asset.nonce = nonce;
    asset
}

#[test]
fn test_asset_storage_put_get() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset = create_test_asset("tx123", 0);
    let asset_id = asset.asset_id();
    store.put(asset.clone()).unwrap();

    let retrieved = store.get(&asset_id).unwrap();
    assert_eq!(retrieved.asset_id(), asset_id);
    assert_eq!(retrieved.amount(), asset.amount());
}

#[test]
fn test_asset_storage_duplicate_put() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset = create_test_asset("tx123", 0);
    store.put(asset.clone()).unwrap();

    let result = store.put(asset);
    assert!(matches!(result, Err(AssetStorageError::AlreadyExists(_))));
}

#[test]
fn test_asset_storage_list_unspent() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset1 = create_test_asset("tx1", 1);
    let asset2 = create_test_asset("tx2", 2);

    store.put(asset1).unwrap();
    store.put(asset2).unwrap();

    let unspent = store.list_unspent().unwrap();
    assert_eq!(unspent.len(), 2);
}

#[test]
fn test_asset_storage_mark_spent() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset = create_test_asset("tx123", 0);
    let asset_id = asset.asset_id();
    store.put(asset).unwrap();

    store.mark_spent(&asset_id, 1001).unwrap();

    let unspent = store.list_unspent().unwrap();
    assert_eq!(unspent.len(), 0);

    let spent = store.list_spent().unwrap();
    assert_eq!(spent.len(), 1);

    let retrieved = store.get(&asset_id).unwrap();
    assert_eq!(retrieved.asset_id(), asset_id);
}

#[test]
fn test_asset_storage_spent_twice() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset = create_test_asset("tx123", 0);
    let asset_id = asset.asset_id();
    store.put(asset).unwrap();

    store.mark_spent(&asset_id, 1001).unwrap();

    let result = store.mark_spent(&asset_id, 1002);
    assert!(matches!(result, Err(AssetStorageError::AlreadySpent(_))));
}

#[test]
fn test_asset_storage_get_balance() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset1 = create_test_asset("tx1", 3);
    let asset2 = create_test_asset("tx2", 4);

    store.put(asset1).unwrap();
    store.put(asset2).unwrap();

    let balance = store.get_balance().unwrap();
    assert_eq!(balance, 2000);
}

#[test]
fn test_asset_storage_remove() {
    let mut store = AssetStorageImpl::new(":memory:").unwrap();

    let asset = create_test_asset("tx123", 0);
    let asset_id = asset.asset_id();
    store.put(asset).unwrap();

    store.remove(&asset_id).unwrap();

    let result = store.get(&asset_id);
    assert!(matches!(result, Err(AssetStorageError::NotFound(_))));
}
