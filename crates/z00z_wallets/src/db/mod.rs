//! Wallet persistence schemas and database backends.
//!
//! Native-only for now because `.wlt` is implemented via `redb`.

#![cfg(not(target_arch = "wasm32"))]

#[path = "../redb_store/mod.rs"]
pub mod redb_store;
pub mod wallet_store_crypto;

pub use self::redb_store::{
    create_wallet_store, discover_wallet_store, open_wallet_store, verify_password_for_session,
    write_wallet_profile, BackupManifestPayload, KeysPayload, ObjectSeenRef, OpenedWallet,
    OwnedAssetPayload, OwnedNonAssetPayload, OwnedObjectFamily, OwnedObjectPayload,
    OwnedObjectPolicy, OwnedObjectSource, OwnedRightPayload, OwnedRightStatus, OwnedVoucherPayload,
    OwnedVoucherStatus, ScanStatePayload, StealthMetaPayload, TofuPinRecord, TofuPinsPayload,
    WalletInventoryPayload, WalletObjectStatus, WalletOwnedObject, WalletPolicyAvailability,
    WalletProfilePayload, WalletSession,
};
pub(crate) use self::redb_store::{
    create_wlt_with_deps, discover_wlt_with_deps, is_lock_held_local, open_wlt_with_deps,
    read_wallet_profile, reveal_seed_phrase,
};
pub use self::redb_store::{
    object_inventory_store, ObjectInventoryFilter, ObjectInventoryPage, ObjectInventoryStore,
    PutOwnedObjectOutcome,
};
pub(crate) use self::redb_store::{
    read_keys_payload, read_scan_state, read_stealth_meta, read_tofu_pins, upsert_keys_payload,
    upsert_scan_state, upsert_stealth_meta, upsert_tofu_pins,
};
pub(crate) use self::redb_store::{
    wallet_asset_store, AssetFilter, AssetPage, AssetPersistContext, PutAssetOutcome,
    WalletAssetStore,
};
pub use wallet_store::WalletIdentity;
pub(crate) use wallet_store::{RedbWalletStore, WltStore, Z00ZWalletIo};
pub use wallet_validate::validate_wallet_file_codes;

pub(crate) mod index_codecs;
#[cfg(test)]
pub(crate) mod redb_kv_backend;
pub mod schema_codecs;
pub mod schema_keys;
#[cfg(test)]
pub(crate) mod test_owned_objects;
pub(crate) mod wallet_io;
pub(crate) mod wallet_store;
pub(crate) mod wallet_validate;
