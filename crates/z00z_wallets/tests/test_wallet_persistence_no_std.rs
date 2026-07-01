#![cfg(not(target_arch = "wasm32"))]

fn wallet_store_sources() -> [(&'static str, &'static str); 8] {
    [
        ("redb_store.rs", include_str!("../src/redb_store/mod.rs")),
        (
            "redb_store_codecs.rs",
            include_str!("../src/redb_store/record_codecs.rs"),
        ),
        (
            "redb_store_crypto_ops.rs",
            include_str!("../src/redb_store/crypto_ops.rs"),
        ),
        (
            "redb_store_migrations.rs",
            include_str!("../src/redb_store/migrations.rs"),
        ),
        (
            "redb_store_objects.rs",
            include_str!("../src/redb_store/object_writes.rs"),
        ),
        (
            "redb_store_queries.rs",
            include_str!("../src/redb_store/object_queries.rs"),
        ),
        (
            "redb_store_session.rs",
            include_str!("../src/redb_store/session.rs"),
        ),
        (
            "redb_store_tables.rs",
            include_str!("../src/redb_store/tables.rs"),
        ),
    ]
}

#[test]
fn test_no_use_forbidden_std() {
    for (source_name, source) in wallet_store_sources() {
        assert!(
            !source.contains("std::fs::remove_file"),
            "wallet persistence must not call std::fs::remove_file directly in {source_name}"
        );

        assert!(
            !source.contains("std::fs::set_permissions"),
            "wallet persistence must not call std::fs::set_permissions directly in {source_name}"
        );
    }
}

#[test]
fn test_integrity_uses_codec() {
    let (source_name, source) = wallet_store_sources()
        .into_iter()
        .find(|(_, source)| source.contains("fn update_wallet_integrity"))
        .expect("update_wallet_integrity must exist in one wallet-store source file");

    let start = source
        .find("fn update_wallet_integrity")
        .expect("update_wallet_integrity must exist");
    let end = source[start..]
        .find("fn decrypt_object_record")
        .map(|i| start + i)
        .unwrap_or(source.len());
    let slice = &source[start..end];

    assert!(
        slice.contains("BincodeCodec"),
        "integrity encoding must use z00z_utils::codec::BincodeCodec in {source_name}"
    );
    assert!(
        !slice.contains("serde_json"),
        "integrity encoding must not use serde_json in {source_name}"
    );
    assert!(
        !slice.contains("serde_yaml"),
        "integrity encoding must not use serde_yaml in {source_name}"
    );
    assert!(
        !slice.contains("bincode::"),
        "integrity encoding must not use bincode directly in {source_name}"
    );
}

#[test]
fn test_wallet_store_semantic_modules() {
    let redb_store = include_str!("../src/redb_store/mod.rs");

    for seam_reexport in [
        "pub use self::crypto_ops::",
        "pub use self::objects::write_object;",
        "pub use self::queries::",
        "pub use self::session::",
        "pub use self::tables::",
    ] {
        assert!(
            redb_store.contains(seam_reexport),
            "redb wallet store must preserve semantic seam re-export {seam_reexport}"
        );
    }
}

#[test]
fn test_store_root_excludes_types() {
    let redb_store = include_str!("../src/redb_store/mod.rs");

    for root_type in [
        "pub struct WalletSession",
        "pub struct ScanStatePayload",
        "pub struct StealthMetaPayload",
        "pub struct TofuPinsPayload",
    ] {
        assert!(
            !redb_store.contains(root_type),
            "boundary type {root_type} must be owned by a focused wallet-store module, not the root facade"
        );
    }
}
