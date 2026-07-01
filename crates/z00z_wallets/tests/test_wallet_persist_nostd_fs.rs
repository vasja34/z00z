#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn test_wallet_persistence_not_use() {
    let redb_store = include_str!("../src/redb_store/mod.rs");

    assert!(
        !redb_store.contains("std::fs::remove_file"),
        "wallet persistence must not call std::fs::remove_file directly"
    );

    assert!(
        !redb_store.contains("std::fs::set_permissions"),
        "wallet persistence must not call std::fs::set_permissions directly"
    );
}
