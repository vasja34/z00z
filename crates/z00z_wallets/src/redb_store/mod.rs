#![cfg(not(target_arch = "wasm32"))]
#![allow(missing_docs)]
#![cfg_attr(test, allow(clippy::clone_on_copy))]

use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::db::wallet_store_crypto::{
    aad_object, AeadEnvelope, KdfParams, MasterKeyRecord, SecretsKind, SecretsRecord,
    WalletDerivedKeys, AAD_SECRET_VERSION, HKDF_INFO_VERSION, REDB_WALLET_SCHEMA_VERSION,
};
use crate::key::{mnemonic, MnemonicLanguage, SeedPhrase24, SeedWords};
use once_cell::sync::Lazy;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use z00z_crypto::expert::encoding::{to_hex, SafePassword};
use z00z_crypto::Hidden;
use z00z_utils::codec::{BincodeCodec, Codec};
use z00z_utils::compression::{zstd_decode_bounded_to_writer, zstd_encode_to_writer};
use z00z_utils::rng::{RngCoreExt, SecureRngProvider, SystemRngProvider};
use z00z_utils::time::{SystemTimeProvider, TimeProvider};

#[cfg(test)]
use std::cell::Cell;

use crate::db::index_codecs::{validate_index_update, IndexKeyBytes, IndexValueBytes};
use crate::db::schema_codecs::{
    decode_encrypted_object_record, decode_object_id_be, encode_encrypted_object_record,
    encode_object_id_be,
};
use crate::db::schema_keys::{
    META_AAD_SECRET_VERSION, META_APP_OBJECT_ID, META_CHAIN_OBJECT_ID,
    META_DERIVATION_STATE_OBJECT_ID, META_HKDF_INFO_VERSION, META_INDEX_FORMAT_VERSION,
    META_KEYS_OBJECT_ID, META_ROTATION_IN_PROGRESS, META_SCAN_STATE_OBJECT_ID, META_SCHEMA_VERSION,
    META_STEALTH_META_OBJECT_ID, META_TOFU_PINS_OBJECT_ID, META_WALLET_CHAIN,
    META_WALLET_CREATED_AT, META_WALLET_ID, META_WALLET_INITIALIZED, META_WALLET_INTEGRITY,
    META_WALLET_KDF, META_WALLET_NETWORK, META_WALLET_PROFILE_OBJECT_ID, META_WALLET_SAVE_SEQ,
    META_WALLET_UPDATED_AT, SECRETS_MASTER_KEY, SECRETS_SEED_MAIN, SECRETS_SEED_MAIN_REVEALED_AT,
};
use crate::db::wallet_store::{WalletIdentity, WalletIo, Z00ZWalletIo};
use crate::rpc::types::common::PersistWalletId;
use crate::rpc::types::wallet::PersistWalletDiscovery;
pub use crate::wasm::{EncryptedObjectPayload, EncryptedObjectRecord, IndexTable};
use crate::{key::WalletRedbKeyManager, security::SecretBytes};
use crate::{WalletError, WalletResult};

#[path = "record_codecs.rs"]
mod codecs;
mod crypto_ops;
mod debug;
mod meta;
mod migrations;
mod mutations;
#[path = "object_writes.rs"]
mod objects;
mod open;
mod owned_assets;
mod owned_objects;
mod profile;
#[path = "object_queries.rs"]
mod queries;
mod session;
mod tables;
pub use self::crypto_ops::{
    reveal_seed_phrase, reveal_seed_phrase_once, verify_password_for_session,
};
#[cfg(feature = "wallet_debug_tools")]
#[rustfmt::skip]
pub(crate) use self::debug::{debug_export_wallet};
#[allow(unused_imports)]
pub(crate) use self::meta::REQUIRED_META_POINTER_KEYS_OPEN;
use self::meta::{
    bump_wallet_write_meta, read_wallet_meta_header, store_required_meta, store_wallet_save_seq,
    wallet_tmp_path, WALLET_META_INVALID,
};
pub(crate) use self::mutations::create_wlt_with_deps;
use self::mutations::store_object;
pub use self::mutations::{
    create_wallet_store, upsert_keys_payload, upsert_scan_state, upsert_stealth_meta,
    upsert_tofu_pins,
};
pub use self::objects::write_object;
pub use self::open::discover_wallet_store;
pub(crate) use self::open::{discover_wlt_with_deps, open_wlt_with_deps};
pub(crate) use self::owned_assets::{
    wallet_asset_store, AssetFilter, AssetPage, AssetPersistContext, PutAssetOutcome,
    WalletAssetStore,
};
pub use self::owned_objects::{
    object_inventory_store, ObjectInventoryFilter, ObjectInventoryPage, ObjectInventoryStore,
    PutOwnedObjectOutcome,
};
pub use self::profile::{read_wallet_profile, write_wallet_profile};
pub use self::queries::{
    read_keys_payload, read_object_by_id, read_scan_state, read_stealth_meta, read_tofu_pins,
};
pub(crate) use self::session::is_lock_held_local;
#[cfg(test)]
pub(crate) use self::session::set_rotate_master_fp_commit;
pub use self::session::{open_wallet_store, OpenedWallet, WalletSession};
pub use self::tables::{
    AccountPayload, AppPayload, AppPlatform, AssetSeenRef, BackupManifestPayload, ChainPayload,
    ConfirmRef, DerivationStatePayload, IndexUpdate, KeyRefPayload, KeysPayload, ModeAuditEntry,
    ObjectKindId, ObjectSeenRef, OwnedAssetPayload, OwnedAssetPolicy, OwnedAssetSource,
    OwnedAssetStatus, OwnedNonAssetPayload, OwnedObjectFamily, OwnedObjectPayload,
    OwnedObjectPolicy, OwnedObjectSource, OwnedRightPayload, OwnedRightStatus, OwnedVoucherPayload,
    OwnedVoucherStatus, ReceiveRef, ScanRef, ScanStatePayload, StealthMetaPayload, TofuPinRecord,
    TofuPinsPayload, WalletInventoryPayload, WalletObjectStatus, WalletOwnedObject,
    WalletPolicyAvailability, WalletProfilePayload, WalletRootPayload, WalletTxEventPayload,
    WalletTxEventType, WalletTxPayload, WalletTxRole, PAYLOAD_VERSION_ACCOUNT, PAYLOAD_VERSION_APP,
    PAYLOAD_VERSION_BACKUP_MANIFEST, PAYLOAD_VERSION_CHAIN, PAYLOAD_VERSION_DERIVATION_STATE,
    PAYLOAD_VERSION_KEYS, PAYLOAD_VERSION_OWNED_ASSET, PAYLOAD_VERSION_OWNED_RIGHT,
    PAYLOAD_VERSION_OWNED_VOUCHER, PAYLOAD_VERSION_SCAN_STATE, PAYLOAD_VERSION_STEALTH_META,
    PAYLOAD_VERSION_TOFU_PINS, PAYLOAD_VERSION_WALLET_PROFILE, PAYLOAD_VERSION_WALLET_ROOT,
    PAYLOAD_VERSION_WALLET_TX, PAYLOAD_VERSION_WALLET_TX_EVENT,
};
use self::{
    codecs::{
        allocate_object_id, decode_bincode, decode_bincode_bounded, decode_object_record_bounded,
        decode_seed_plaintext_phrase24, encode_bincode, generate_16_bytes, generate_object_id,
        object_id_from_be_bytes, object_id_to_be_bytes, unwrap_object_payload_with_header,
        validate_seed_main_record, validate_seed_plaintext_unlock, wrap_object_payload_with_header,
        MAX_COMPRESSED_BYTES, OBJECT_PAYLOAD_HEADER_LEN, WALLET_OBJECT_PAYLOAD_INVALID,
        WALLET_SECRET_INVALID,
    },
    crypto_ops::{
        commit_redb_write_txn_flush, decrypt_object_record, decrypt_secret_record,
        decrypt_secret_record_post_unlock, encrypt_object_record, encrypt_secret_record,
        update_wallet_integrity,
    },
    migrations::{is_zstd_magic_bytes, migrate_index_format_if_needed},
    objects::{write_object_with_index_key, write_object_with_indexes},
    session::{finalize_rotation_marker_on_db, try_lock_wallet_file, verify_archived_wallet_copy},
    tables::{
        IndexManifestEntry, SeedMainEntropyPayload, SeedMainMnemonicLanguage, ValidatedIndexUpdate,
        WltBacking, INDEX_MANIFEST_TABLE, META_TABLE, OBJECTS_TABLE, SECRETS_TABLE,
    },
};

// Zstd frame magic number (little-endian): 0xFD2FB528
const WLT_ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

// Zstd compression level for whole-wallet compression (Phase 9).
// Level 3 provides good compression ratio with reasonable speed.
const WLT_ZSTD_LEVEL: i32 = 3;

// Whole-wallet size cap (DoS bounds).
// This is the maximum allowed size of the decompressed RedB file when loading a zstd `.wlt`.
const MAX_WLT_DECOMPRESSED_BYTES: usize = 128 * 1024 * 1024;

// Index key encoding format version marker.
// v2: HMAC-SHA256 (current)
const INDEX_FORMAT_VERSION_HMAC: u32 = 2;

#[cfg(test)]
thread_local! {
    static CREATE_WLT_FAILPOINT_DB: Cell<bool> = const { Cell::new(false) };
    static CREATE_WLT_FP_META: Cell<bool> = const { Cell::new(false) };
    static CREATE_WLT_FP_SECRETS: Cell<bool> = const { Cell::new(false) };
    static CREATE_WLT_FP_COMMIT: Cell<bool> = const { Cell::new(false) };

    static CREATE_WLT_COMMIT_CT: Cell<u32> = const { Cell::new(0) };
}

#[cfg(test)]
fn set_create_wlt_failpoint_db(enabled: bool) {
    CREATE_WLT_FAILPOINT_DB.with(|flag| flag.set(enabled));
}

#[cfg(test)]
fn create_take_wlt_failpoint_db() -> bool {
    CREATE_WLT_FAILPOINT_DB.with(|flag| {
        let enabled = flag.get();
        if enabled {
            flag.set(false);
        }
        enabled
    })
}

#[cfg(test)]
fn set_create_wlt_fp_meta(enabled: bool) {
    CREATE_WLT_FP_META.with(|flag| flag.set(enabled));
}

#[cfg(test)]
fn take_create_wlt_fp_meta() -> bool {
    CREATE_WLT_FP_META.with(|flag| {
        let enabled = flag.get();
        if enabled {
            flag.set(false);
        }
        enabled
    })
}

#[cfg(test)]
fn set_create_wlt_fp_secrets(enabled: bool) {
    CREATE_WLT_FP_SECRETS.with(|flag| flag.set(enabled));
}

#[cfg(test)]
fn take_create_wlt_fp_secrets() -> bool {
    CREATE_WLT_FP_SECRETS.with(|flag| {
        let enabled = flag.get();
        if enabled {
            flag.set(false);
        }
        enabled
    })
}

#[cfg(test)]
fn set_create_wlt_fp_commit(enabled: bool) {
    CREATE_WLT_FP_COMMIT.with(|flag| flag.set(enabled));
}

#[cfg(test)]
fn take_create_wlt_fp_commit() -> bool {
    CREATE_WLT_FP_COMMIT.with(|flag| {
        let enabled = flag.get();
        if enabled {
            flag.set(false);
        }
        enabled
    })
}

#[cfg(test)]
fn reset_create_wlt_commit_ct() {
    CREATE_WLT_COMMIT_CT.with(|ct| ct.set(0));
}

#[cfg(test)]
fn inc_create_wlt_commit_ct() {
    CREATE_WLT_COMMIT_CT.with(|ct| ct.set(ct.get().saturating_add(1)));
}

#[cfg(test)]
fn get_create_wlt_commit_ct() -> u32 {
    CREATE_WLT_COMMIT_CT.with(|ct| ct.get())
}

fn is_supported_payload_version(kind_id: u8, payload_version: u16) -> bool {
    match kind_id {
        x if x == ObjectKindId::WalletRoot as u8 => payload_version == PAYLOAD_VERSION_WALLET_ROOT,
        x if x == ObjectKindId::Account as u8 => payload_version == PAYLOAD_VERSION_ACCOUNT,
        x if x == ObjectKindId::DerivationState as u8 => {
            payload_version == PAYLOAD_VERSION_DERIVATION_STATE
        }
        x if x == ObjectKindId::ScanState as u8 => payload_version == PAYLOAD_VERSION_SCAN_STATE,
        x if x == ObjectKindId::App as u8 => payload_version == PAYLOAD_VERSION_APP,
        x if x == ObjectKindId::Chain as u8 => payload_version == PAYLOAD_VERSION_CHAIN,
        x if x == ObjectKindId::Keys as u8 => payload_version == PAYLOAD_VERSION_KEYS,
        x if x == ObjectKindId::StealthMeta as u8 => {
            payload_version == PAYLOAD_VERSION_STEALTH_META
        }
        x if x == ObjectKindId::TofuPins as u8 => payload_version == PAYLOAD_VERSION_TOFU_PINS,
        x if x == ObjectKindId::WalletProfile as u8 => {
            payload_version == PAYLOAD_VERSION_WALLET_PROFILE
        }
        x if x == ObjectKindId::OwnedAsset as u8 => payload_version == PAYLOAD_VERSION_OWNED_ASSET,
        x if x == ObjectKindId::OwnedVoucher as u8 => {
            payload_version == PAYLOAD_VERSION_OWNED_VOUCHER
        }
        x if x == ObjectKindId::OwnedRight as u8 => payload_version == PAYLOAD_VERSION_OWNED_RIGHT,
        x if x == ObjectKindId::WalletTx as u8 => payload_version == PAYLOAD_VERSION_WALLET_TX,
        x if x == ObjectKindId::WalletTxEvent as u8 => {
            payload_version == PAYLOAD_VERSION_WALLET_TX_EVENT
        }
        x if x == ObjectKindId::BackupManifest as u8 => {
            payload_version == PAYLOAD_VERSION_BACKUP_MANIFEST
        }
        _ => false,
    }
}

fn is_valid_mode(mode: &str) -> bool {
    mode == "stealth_ecdh"
}

fn is_valid_tofu_lvl(level: u8) -> bool {
    level <= 3
}

#[cfg(test)]
#[path = "test_store_suite.rs"]
mod tests;
