# DB Refactor Specification

## Scope

- Target: crates/z00z_wallets/src/db
- Type: structural refactor plan only (no code changes in this document)
- Requirement coverage: every existing file under db receives an explicit placement decision

## Objectives

1. Reduce coupling between Redb store internals, codec surfaces, and wallet I/O entrypoints.
2. Make runtime ownership boundaries explicit in folder layout (backends/codecs/crypto/redb/io).
3. Isolate migration and test surfaces to reduce incidental cross-module churn during staged rewiring.

## Test Placement Policy

- Unit tests stay colocated with the runtime module they validate.
- Unit-test filenames must start with `test_`.
- Avoid mixed root-level and nested test scattering inside `src/db`; converge under `db/redb/tests` in this target layout.
- Integration and end-to-end suites remain under `crates/z00z_wallets/tests`.

## Decision Method

- Input signal 1: top-level declarations extracted from each file (struct/trait/fn/type signatures).
- Input signal 2: actual intra-db dependency affinity (`use crate::db::...`) and cross-core links.
- Input signal 3: current boundary constraints from `db/mod.rs` and Redb store composition files.
- For each file, placement decision is content-driven (domain behavior + dependency direction), not filename-only.

## High-Level Target Tree

```text
crates/z00z_wallets/src/db/
  backends/
    storage_backend.rs
    wallet_store.rs
  codecs/
    index_codecs.rs
    index_codecs_body.rs
    index_codecs_tx_time.rs
    schema_codecs.rs
    schema_keys.rs
  crypto/
    redb_wallet_crypto.rs
    redb_wallet_crypto_aad.rs
    redb_wallet_crypto_kdf_helpers.rs
    redb_wallet_crypto_models.rs
  io/
    wallet_io.rs
    wallet_validate.rs
  redb/
    migrations/
      redb_wallet_store_migrations.rs
      redb_wallet_store_migrations_tables.rs
    schema/
      redb-schema.yaml
    store/
      redb_wallet_store.rs
      redb_wallet_store_backup.rs
      redb_wallet_store_codecs.rs
      redb_wallet_store_create.rs
      redb_wallet_store_crypto_ops.rs
      redb_wallet_store_crypto_ops_seed.rs
      redb_wallet_store_debug.rs
      redb_wallet_store_debug_export.rs
      redb_wallet_store_debug_types.rs
      redb_wallet_store_discovery.rs
      redb_wallet_store_initial_objects.rs
      redb_wallet_store_meta.rs
      redb_wallet_store_mutations.rs
      redb_wallet_store_objects.rs
      redb_wallet_store_objects_test_support.rs
      redb_wallet_store_open.rs
      redb_wallet_store_open_session.rs
      redb_wallet_store_queries.rs
      redb_wallet_store_session.rs
      redb_wallet_store_tables.rs
      redb_wallet_store_upserts.rs
    tests/
      redb_wallet_store.rs
      test_index_codecs_suite.rs
      test_redb_wallet_crypto_suite.rs
      test_storage_backend_suite.rs
  index_codecs.rs.backup
  mod.rs
```

## Pros and Cons Taxonomy Used Per File

- keep
  - Pros: zero API path churn, lower migration risk.
  - Cons: preserves current cognitive load and mixed folder density.
- move-<domain>
  - Pros: increases cohesion of one runtime responsibility and simplifies ownership boundaries.
  - Cons: requires mod.rs rewiring and staged import-path migration.

## Exhaustive File Placement Ledger

| File | Content Signals | Core Dependencies | Decision | Target Path | Pros | Cons |
|---|---|---|---|---|---|---|
| crates/z00z_wallets/src/db/index_codecs.rs | 53:pub struct IndexKeyBytes(pub(crate) Vec<u8>);;55:impl IndexKeyBytes {;56:pub fn new(table: IndexTable, key: Vec<u8>) -> WalletResult<Self> { | {WalletError, WalletResult}:1, | MOVE (move-db-codecs) | crates/z00z_wallets/src/db/codecs/index_codecs.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/index_codecs.rs.backup | 57:pub fn encode_index_semantic_kv(domain: &str, field: &str, value: &[u8]) -> WalletResult<Vec<u8>> {;98:pub struct IndexKeyBytes(pub(crate) Vec<u8>);;100:impl IndexKeyBytes { | {WalletError, WalletResult}:1, | MOVE (move-redb-store) | crates/z00z_wallets/src/db/index_codecs.rs.backup | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/index_codecs_body.rs | 5:pub fn encode_index_key(;43:pub enum IndexKeyMode {;47:pub fn encode_index_key_mode( | core:1, | MOVE (move-db-codecs) | crates/z00z_wallets/src/db/codecs/index_codecs_body.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/index_codecs_tx_time.rs | 4:pub fn encode_tx_time_index_key(timestamp_ms: u64, object_id: u128) -> Vec<u8> {;14:pub fn decode_tx_time_index_key(key: &[u8]) -> WalletResult<(u64, u128)> { | none | MOVE (move-db-codecs) | crates/z00z_wallets/src/db/codecs/index_codecs_tx_time.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/mod.rs | none | none | KEEP | crates/z00z_wallets/src/db/mod.rs | Stable import path and minimal migration risk. | Folder remains broad; conceptual density is unchanged. |
| crates/z00z_wallets/src/db/redb-schema.yaml | none | none | MOVE (move-redb-schema) | crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_crypto.rs | 114:pub enum KdfAlgo {;123:pub struct KdfParams {;138:impl KdfParams { | core:1, | MOVE (move-db-crypto) | crates/z00z_wallets/src/db/crypto/redb_wallet_crypto.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs | 4:pub fn aad_master_key(wallet_id: &[u8]) -> Vec<u8> {;13:pub fn aad_secret(wallet_id: &[u8], secret_name: &str) -> Vec<u8> {;27:pub fn aad_object(wallet_id: &[u8], object_id: u128, payload_version: u16) -> [u8; 34] { | none | MOVE (move-db-crypto) | crates/z00z_wallets/src/db/crypto/redb_wallet_crypto_aad.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs | none | none | MOVE (move-db-crypto) | crates/z00z_wallets/src/db/crypto/redb_wallet_crypto_kdf_helpers.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_crypto_models.rs | 5:pub struct AeadEnvelope {;10:impl AeadEnvelope {;12:pub fn algo_id(&self) -> Result<u8, CryptoError> { | none | MOVE (move-db-crypto) | crates/z00z_wallets/src/db/crypto/redb_wallet_crypto_models.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store.rs | none | adapters:2,core:3,index_codecs:1,wallet_store:1,wasm:1,{WalletError, WalletResult}:1, | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_backup.rs | 8:pub fn write_wallet_snapshot<R: SecureRngProvider>(;77:pub fn read_wallet_snapshot(session: &WalletSession) -> WalletResult<SecretBytes> { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_backup.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs | 270:pub(crate) fn generate_object_id(rng: &mut impl rand::RngCore) -> u128 {;278:rng: &mut impl rand::RngCore,;297:pub(crate) fn generate_16_bytes(rng: &mut impl rand::RngCore) -> [u8; 16] { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_codecs.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_create.rs | 16:rng: &mut impl rand::RngCore,;74:pub fn create_wallet_store<R: SecureRngProvider + Clone>( | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_create.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs | 9:_rng: &mut impl rand::RngCore,;31:rng: &mut impl rand::RngCore,;47:rng: &mut impl rand::RngCore, | core:1, | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_crypto_ops.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops_seed.rs | 68:rng: &mut impl rand::RngCore,;100:pub fn reveal_seed_phrase_once<R: SecureRngProvider>(;145:pub fn reveal_seed_phrase<R: SecureRngProvider>( | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_crypto_ops_seed.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_debug.rs | none | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_debug.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs | 20:pub fn debug_export_wallet( | core:1,index_codecs:1, | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_debug_export.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_debug_types.rs | 9:pub struct DebugTableRow {;16:pub struct DebugMetaEntry {;24:pub struct DebugSecretEntry { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_debug_types.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_discovery.rs | 3:pub fn discover_wallet_store(path: &Path) -> WalletResult<PersistWalletDiscovery> { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_discovery.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_initial_objects.rs | 4:rng: &mut impl rand::RngCore, | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_initial_objects.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_meta.rs | none | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_meta.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_migrations.rs | none | none | MOVE (move-redb-migrations) | crates/z00z_wallets/src/db/redb/migrations/redb_wallet_store_migrations.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_migrations_tables.rs | none | none | MOVE (move-redb-migrations) | crates/z00z_wallets/src/db/redb/migrations/redb_wallet_store_migrations_tables.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_mutations.rs | none | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_mutations.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_objects.rs | 16:pub fn write_object<R: SecureRngProvider>( | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_objects.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_objects_test_support.rs | none | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_objects_test_support.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_open.rs | none | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_open.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_open_session.rs | 9:impl TmpfsWorkGuard {;23:impl Drop for TmpfsWorkGuard { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_open_session.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_queries.rs | 9:pub fn read_object_by_id(;37:pub fn read_stealth_meta(session: &WalletSession) -> WalletResult<Option<StealthMetaPayload>> {;74:pub fn read_scan_state(session: &WalletSession) -> WalletResult<Option<ScanStatePayload>> { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_queries.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_session.rs | 20:impl std::fmt::Debug for WalletFileLockInner {;29:impl Drop for WalletFileLockInner {;130:pub struct OpenedWallet { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_session.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_tables.rs | 56:pub enum ObjectKindId {;81:pub struct IndexUpdate {;87:impl IndexUpdate { | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_tables.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/redb_wallet_store_upserts.rs | 3:pub fn upsert_scan_state<R: SecureRngProvider>(;59:pub fn upsert_stealth_meta<R: SecureRngProvider>(;136:pub fn upsert_tofu_pins<R: SecureRngProvider>( | none | MOVE (move-redb-store) | crates/z00z_wallets/src/db/redb/store/redb_wallet_store_upserts.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/schema_codecs.rs | 10:pub fn encode_object_id_be(object_id: u128) -> [u8; 16] {;15:pub fn decode_object_id_be(bytes: &[u8]) -> WalletResult<u128> {;25:pub fn encode_encrypted_object_record<T: Serialize>(record: &T) -> WalletResult<Vec<u8>> { | {WalletError, WalletResult}:1, | MOVE (move-db-codecs) | crates/z00z_wallets/src/db/codecs/schema_codecs.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/schema_keys.rs | 73:pub enum IndexTable { | none | MOVE (move-db-codecs) | crates/z00z_wallets/src/db/codecs/schema_keys.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/storage_backend.rs | 34:impl RedbWalletKvBackend {;60:impl RedbWalletKvTxn {;306:impl WalletKvTxn for RedbWalletKvTxn { | wallet_store:1,wasm:1,{WalletError, WalletResult}:1, | MOVE (move-db-backends) | crates/z00z_wallets/src/db/backends/storage_backend.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/test_index_codecs_suite.rs | none | none | MOVE (move-redb-tests) | crates/z00z_wallets/src/db/redb/tests/test_index_codecs_suite.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/test_redb_wallet_crypto_suite.rs | none | none | MOVE (move-redb-tests) | crates/z00z_wallets/src/db/redb/tests/test_redb_wallet_crypto_suite.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/test_storage_backend_suite.rs | none | none | MOVE (move-redb-tests) | crates/z00z_wallets/src/db/redb/tests/test_storage_backend_suite.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/tests/redb_wallet_store.rs | 35:impl crate::db::wallet_store::WalletIo for DenyPermissionsIo {;163:impl crate::db::wallet_store::WalletIo for TrackingWalletIo {;210:impl Drop for FailpointGuard { | none | MOVE (move-redb-tests) | crates/z00z_wallets/src/db/redb/tests/redb_wallet_store.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/wallet_io.rs | none | {WalletError, WalletResult}:1, | MOVE (move-db-io) | crates/z00z_wallets/src/db/io/wallet_io.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/wallet_store.rs | 42:pub struct WalletIdentity {;54:impl WalletIo for Z00ZWalletIo {;137:impl RedbWalletStore { | WalletError:1,adapters:2,core:1,wallet_io:1,{WalletError, WalletResult}:1, | MOVE (move-db-backends) | crates/z00z_wallets/src/db/backends/wallet_store.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/db/wallet_validate.rs | 136:pub fn validate_wallet_file_codes(path: &Path) -> WalletResult<Vec<String>> {;163:impl TempPathGuard {;172:impl Drop for TempPathGuard { | core:1,wallet_store:1,{WalletError, WalletResult}:1, | MOVE (move-db-io) | crates/z00z_wallets/src/db/io/wallet_validate.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
