use super::{
    allocate_object_id, commit_redb_write_txn_flush, encode_bincode,
    encode_encrypted_object_record, encode_object_id_be, encrypt_object_record,
    encrypt_secret_record, generate_16_bytes, generate_object_id, is_valid_mode, is_valid_tofu_lvl,
    object_id_from_be_bytes, object_id_to_be_bytes, read_stealth_meta, store_required_meta,
    store_wallet_save_seq, to_hex, try_lock_wallet_file, update_wallet_integrity, wallet_tmp_path,
    write_object_with_indexes, zstd_encode_to_writer, AccountPayload, AppPayload, AppPlatform, Arc,
    ChainPayload, Database, DerivationStatePayload, EncryptedObjectRecord, Hidden, KdfParams,
    KeysPayload, MasterKeyRecord, MnemonicLanguage, ModeAuditEntry, ObjectKindId, Path,
    PersistWalletId, ReadableTable, RngCoreExt, SafePassword, ScanStatePayload, SecretsKind,
    SecretsRecord, SecureRngProvider, SeedMainEntropyPayload, SeedMainMnemonicLanguage,
    SeedPhrase24, StealthMetaPayload, SystemRngProvider, SystemTimeProvider, TimeProvider,
    TofuPinsPayload, WalletDerivedKeys, WalletError, WalletIdentity, WalletIo,
    WalletRedbKeyManager, WalletResult, WalletRootPayload, WalletSession, Z00ZWalletIo,
    META_APP_OBJECT_ID, META_CHAIN_OBJECT_ID, META_DERIVATION_STATE_OBJECT_ID, META_KEYS_OBJECT_ID,
    META_SCAN_STATE_OBJECT_ID, META_STEALTH_META_OBJECT_ID, META_TABLE, META_TOFU_PINS_OBJECT_ID,
    OBJECTS_TABLE, PAYLOAD_VERSION_ACCOUNT, PAYLOAD_VERSION_APP, PAYLOAD_VERSION_CHAIN,
    PAYLOAD_VERSION_DERIVATION_STATE, PAYLOAD_VERSION_KEYS, PAYLOAD_VERSION_SCAN_STATE,
    PAYLOAD_VERSION_STEALTH_META, PAYLOAD_VERSION_TOFU_PINS, PAYLOAD_VERSION_WALLET_ROOT,
    SECRETS_MASTER_KEY, SECRETS_SEED_MAIN, SECRETS_TABLE, WLT_ZSTD_LEVEL,
};

#[path = "mutations_create_wallet.rs"]
mod create;
#[path = "initial_wallet_objects.rs"]
mod initial_objects;
#[path = "mutations_singletons.rs"]
mod upserts;

pub use self::create::create_wallet_store;
pub(crate) use self::create::{create_wlt_with_deps, store_object};
pub(crate) use self::upserts::upsert_scan_state_with_txn;
pub use self::upserts::{
    upsert_keys_payload, upsert_scan_state, upsert_stealth_meta, upsert_tofu_pins,
};
