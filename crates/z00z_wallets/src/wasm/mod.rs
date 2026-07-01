//! WASM-facing modules.
//!
//! These modules are safe to compile on `wasm32` and avoid native-only dependencies.

#![allow(missing_docs)]

// Re-export from db (source of truth)
pub use crate::db::schema_codecs::{
    decode_encrypted_object_record, decode_object_id_be, encode_encrypted_object_record,
    encode_object_id_be,
};
pub use crate::db::schema_keys::{
    IndexTable, META_APP_OBJECT_ID, META_CHAIN_OBJECT_ID, META_DERIVATION_STATE_OBJECT_ID,
    META_INDEX_FORMAT_VERSION, META_KEYS_OBJECT_ID, META_SCAN_STATE_OBJECT_ID, META_SCHEMA_VERSION,
    META_WALLET_CHAIN, META_WALLET_CREATED_AT, META_WALLET_ID, META_WALLET_INITIALIZED,
    META_WALLET_INTEGRITY, META_WALLET_KDF, META_WALLET_SAVE_SEQ, META_WALLET_UPDATED_AT,
    SECRETS_MASTER_KEY, SECRETS_SEED_MAIN, SECRETS_SEED_MAIN_REVEALED_AT,
};
pub use crate::db::wallet_store_crypto::{AeadEnvelope, SecretsKind, SecretsRecord};

pub mod object_types;
pub mod storage_traits;

#[cfg(target_arch = "wasm32")]
pub mod indexeddb_backend;

pub use object_types::{EncryptedObjectPayload, EncryptedObjectRecord};
pub use storage_traits::{
    WalletBlobBackend, WalletKvBackend, WalletKvTable, WalletKvTxn, WalletTxnMode,
};

#[cfg(target_arch = "wasm32")]
pub use indexeddb_backend::IndexedDbWalletBackend;
