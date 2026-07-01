use super::{
    decode_bincode, decode_bincode_bounded, decode_object_id_be, decode_object_record_bounded,
    decode_seed_plaintext_phrase24, decrypt_object_record, decrypt_secret_record,
    decrypt_secret_record_post_unlock, finalize_rotation_marker_on_db, generate_16_bytes,
    is_zstd_magic_bytes, meta, migrate_index_format_if_needed, read_wallet_meta_header, to_hex,
    try_lock_wallet_file, validate_seed_main_record, validate_seed_plaintext_unlock,
    verify_archived_wallet_copy, zstd_decode_bounded_to_writer, Arc, BufWriter, Database,
    MasterKeyRecord, OpenedWallet, Path, PersistWalletDiscovery, PersistWalletId, ReadableDatabase,
    ReadableTable, SafePassword, SecretsRecord, SystemRngProvider, SystemTimeProvider,
    TimeProvider, WalletDerivedKeys, WalletError, WalletIdentity, WalletIo, WalletRedbKeyManager,
    WalletResult, WalletSession, WltBacking, Write, Z00ZWalletIo, AAD_SECRET_VERSION,
    HKDF_INFO_VERSION, INDEX_FORMAT_VERSION_HMAC, MAX_WLT_DECOMPRESSED_BYTES,
    META_AAD_SECRET_VERSION, META_HKDF_INFO_VERSION, META_INDEX_FORMAT_VERSION,
    META_SCHEMA_VERSION, META_TABLE, META_WALLET_CHAIN, META_WALLET_ID, META_WALLET_INITIALIZED,
    META_WALLET_NETWORK, OBJECTS_TABLE, REDB_WALLET_SCHEMA_VERSION, SECRETS_MASTER_KEY,
    SECRETS_SEED_MAIN, SECRETS_TABLE, WALLET_META_INVALID,
};

#[path = "open_discovery.rs"]
mod discovery;
#[path = "open_wallet.rs"]
mod open_session;

pub use self::discovery::discover_wallet_store;
pub(crate) use self::discovery::discover_wlt_with_deps;
pub(crate) use self::open_session::{open_wlt_with_deps, validate_objects_on_open};
