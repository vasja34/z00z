# Services Refactor Specification

## Scope

- Target: crates/z00z_wallets/src/services
- Type: structural refactor plan only (no code changes in this document)
- Requirement coverage: every existing file under services receives an explicit placement decision

## Objectives

1. Reduce coupling between app orchestration, runtime adapters, and wallet service internals.
2. Make ownership boundaries explicit in folder layout (app/runtime/seed/wallet/*).
3. Isolate wallet subdomains (actions/session/store/types/paths/tests) for staged migration with predictable rewiring.

## Test Placement Policy

- Unit tests stay colocated with the runtime module they validate.
- Unit-test filenames must start with `test_`.
- Avoid flat test scattering inside `src/services`; converge under `services/app/tests` and `services/wallet/tests` in this target layout.
- Integration and end-to-end suites remain under `crates/z00z_wallets/tests`.

## Decision Method

- Input signal 1: top-level declarations extracted from each file (struct/trait/fn/type signatures).
- Input signal 2: actual intra-services dependency affinity (`use crate::services::...`) and cross-module links.
- Input signal 3: current boundary constraints from `services/mod.rs` and wallet-service composition files.
- For each file, placement decision is content-driven (domain behavior + dependency direction), not filename-only.

## High-Level Target Tree

```text
crates/z00z_wallets/src/services/
  app/
    tests/
      test_app_service_suite.rs
    app_chain_network.rs
    app_kernel.rs
    app_seed_password.rs
    app_service.rs
    app_service_impl.rs
    app_wallet_lifecycle.rs
  runtime/
    backup_service.rs
    chain_service.rs
    directory_service.rs
    key_service.rs
    network_service.rs
    session_service.rs
    storage_service.rs
    tx_service.rs
  seed/
    seed_phrase.rs
  wallet/
    actions/
      wallet_service_actions.rs
      wallet_service_actions_assets.rs
      wallet_service_actions_backup.rs
      wallet_service_actions_backup_rpc.rs
      wallet_service_actions_hardening.rs
      wallet_service_actions_reachability.rs
      wallet_service_actions_receive.rs
      wallet_service_actions_receiver.rs
      wallet_service_actions_rpc.rs
      wallet_service_actions_runtime.rs
      wallet_service_actions_tofu.rs
    paths/
      wallet_paths.rs
    session/
      wallet_service_session.rs
      wallet_service_session_build.rs
      wallet_service_session_construction.rs
      wallet_service_session_construction_helpers.rs
      wallet_service_session_construction_variants.rs
      wallet_service_session_derivation.rs
      wallet_service_session_derivation_recovery.rs
      wallet_service_session_guards.rs
      wallet_service_session_lifecycle.rs
      wallet_service_session_password.rs
      wallet_service_session_rotation.rs
      wallet_service_session_runtime.rs
      wallet_service_session_runtime_limits.rs
      wallet_service_session_seed_derivation.rs
      wallet_service_session_snapshot.rs
    store/
      wallet_service_store.rs
      wallet_service_store_create_unlock.rs
      wallet_service_store_create_unlock_open.rs
      wallet_service_store_load_restore.rs
      wallet_service_store_persistence_pack.rs
      wallet_service_store_persistence_pack_snapshot.rs
      wallet_service_store_support.rs
      wallet_service_store_transfer_import.rs
    tests/
      test_wallet_paths_suite.rs
      test_wallet_service_suite.rs
    types/
      wallet_service_types.rs
      wallet_service_types_core.rs
      wallet_service_types_reachability.rs
      wallet_service_types_state.rs
    wallet_service.rs
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
| crates/z00z_wallets/src/services/app_chain_network.rs | 1:impl AppService {;9:pub fn switch_to_onionet(&self) -> RuntimeSwitchChainResponse {;24:pub fn switch_to_tor(&self, enable: bool) -> RuntimeChainSettingsResponse { | none | MOVE (move-services-app) | crates/z00z_wallets/src/services/app/app_chain_network.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/app_kernel.rs | 4:pub struct AppService {;16:impl Default for AppService { | none | MOVE (move-services-app) | crates/z00z_wallets/src/services/app/app_kernel.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/app_seed_password.rs | 1:impl AppService { | none | MOVE (move-services-app) | crates/z00z_wallets/src/services/app/app_seed_password.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/app_service.rs | none | adapters:1,core:5,db:1,services:4, | MOVE (move-services-app) | crates/z00z_wallets/src/services/app/app_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/app_service_impl.rs | 1:impl AppService {;12:pub fn new() -> Self {;20:pub fn with_wallet_service(wallets: Arc<WalletService>) -> Self { | none | MOVE (move-services-app) | crates/z00z_wallets/src/services/app/app_service_impl.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/app_wallet_lifecycle.rs | 1:impl AppService { | none | MOVE (move-services-app) | crates/z00z_wallets/src/services/app/app_wallet_lifecycle.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/backup_service.rs | 4:pub struct BackupService;;6:impl Default for BackupService {;12:impl BackupService { | none | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/backup_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/chain_service.rs | 28:pub struct ChainService {;35:impl ChainService {;37:pub fn new() -> Self { | adapters:2,core:2, | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/chain_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/directory_service.rs | 5:pub struct DirectoryAuth {;12:impl DirectoryAuth {;14:pub fn new(api_key: String, endpoint: String) -> Self { | none | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/directory_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/key_service.rs | 4:pub struct KeyService;;6:impl Default for KeyService {;12:impl KeyService { | none | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/key_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/mod.rs | none | core:1, | KEEP | crates/z00z_wallets/src/services/mod.rs | Stable import path and minimal migration risk. | Folder remains broad; conceptual density is unchanged. |
| crates/z00z_wallets/src/services/network_service.rs | 4:pub struct NetworkService;;6:impl Default for NetworkService {;12:impl NetworkService { | none | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/network_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/seed_phrase.rs | none | core:1,{WalletError, WalletResult}:1, | MOVE (move-services-seed) | crates/z00z_wallets/src/services/seed/seed_phrase.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/session_service.rs | 20:impl SessionHandle {;78:impl std::fmt::Debug for WalletSessionManager {;86:impl WalletSessionManager { | adapters:1,db:1,{WalletError, WalletResult}:1, | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/session_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/storage_service.rs | 4:pub struct StorageService;;6:impl Default for StorageService {;12:impl StorageService { | none | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/storage_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/test_app_service_suite.rs | none | core:3,services:1, | MOVE (move-services-tests) | crates/z00z_wallets/src/services/app/tests/test_app_service_suite.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/test_wallet_paths_suite.rs | none | none | MOVE (move-services-tests) | crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/test_wallet_service_suite.rs | 27:impl MockSleeper {;33:impl Sleeper for MockSleeper {;48:impl WltStore for FailingWltStore { | core:2, | MOVE (move-services-tests) | crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/tx_service.rs | 17:pub struct TxService {;21:impl TxService {;27:pub fn new(wallets: Arc<WalletService>) -> Self { | none | MOVE (move-services-runtime) | crates/z00z_wallets/src/services/runtime/tx_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_paths.rs | none | WalletError:1,WalletResult:1,core:1,db:1, | MOVE (move-services-wallet-paths) | crates/z00z_wallets/src/services/wallet/paths/wallet_paths.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service.rs | none | adapters:1,core:4,services:1,{WalletError, WalletResult}:1, | MOVE (move-services-wallet) | crates/z00z_wallets/src/services/wallet/wallet_service.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions.rs | none | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_assets.rs | none | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_backup.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_backup_rpc.rs | none | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup_rpc.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_hardening.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_hardening.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs | 1:impl WalletService {;3:pub fn reachability(&self) -> WalletServiceReachability<'_> {;13:pub fn compact_storage(&self, params: &RuntimeCompactStorageParams) -> bool { | core:1, | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_receive.rs | 1:impl WalletService {;156:pub fn list_assets(;172:pub fn merge_assets( | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receiver.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_rpc.rs | 1:impl WalletService {;3:pub fn rotate_master_key(;19:pub fn list_addresses( | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs | 6:impl WalletService {;7:impl WalletService { | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_runtime.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_actions_tofu.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-actions) | crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_tofu.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session.rs | none | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_build.rs | 1:impl WalletService {;4:pub fn new() -> Self {;19:pub fn with_output_dir(output_dir: PathBuf) -> Self { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_build.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_construction.rs | 4:impl WalletService {;14:pub fn new() -> Self {;29:pub fn with_output_dir(output_dir: PathBuf) -> Self { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_construction_helpers.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction_helpers.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_construction_variants.rs | 1:impl WalletService {;3:pub fn with_dependencies(time_provider: Arc<dyn TimeProvider>) -> Self {;8:pub fn with_dependencies_and_rng_provider<P>( | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction_variants.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_derivation.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_derivation_recovery.rs | 3:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_guards.rs | 1:impl WalletService {;300:pub fn start_auto_lock_monitor(self: Arc<Self>) -> JoinHandle<()> { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_guards.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_lifecycle.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_lifecycle.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_password.rs | 1:impl WalletService {;2:pub fn with_dependencies(time_provider: Arc<dyn TimeProvider>) -> Self {;7:pub fn with_dependencies_and_rng_provider<P>( | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_password.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_rotation.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_runtime.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_runtime_limits.rs | 1:impl WalletService {;266:pub fn start_auto_lock_monitor(self: Arc<Self>) -> JoinHandle<()> { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime_limits.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_seed_derivation.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_seed_derivation.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_session_snapshot.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-session) | crates/z00z_wallets/src/services/wallet/session/wallet_service_session_snapshot.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store.rs | none | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_create_unlock.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_create_unlock_open.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs | 1:impl WalletService {;49:pub fn decode_export_seed_salt( | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_persistence_pack_snapshot.rs | 1:impl WalletService { | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_support.rs | 1:impl Default for WalletService {;31:impl WalletService {;119:impl WalletService { | none | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_store_transfer_import.rs | 1:impl WalletService { | core:2, | MOVE (move-services-wallet-store) | crates/z00z_wallets/src/services/wallet/store/wallet_service_store_transfer_import.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_types.rs | none | adapters:1,core:4,services:1,{WalletError, WalletResult}:1, | MOVE (move-services-wallet-types) | crates/z00z_wallets/src/services/wallet/types/wallet_service_types.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_types_core.rs | 8:pub(super) type AddressLabelList = Vec<(String, String)>;;9:type AddressLabelsStore = BTreeMap<PersistWalletId, AddressLabelList>;;10:pub(super) type WalletAddressDeriverHandle = Arc<RwLock<WalletAddressDeriverState>>; | none | MOVE (move-services-wallet-types) | crates/z00z_wallets/src/services/wallet/types/wallet_service_types_core.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_types_reachability.rs | 7:pub struct WalletServiceReachability<'a> {;11:impl<'a> WalletServiceReachability<'a> {;13:pub fn list_wallets(&self) -> Vec<PersistWalletInfo> { | none | MOVE (move-services-wallet-types) | crates/z00z_wallets/src/services/wallet/types/wallet_service_types_reachability.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
| crates/z00z_wallets/src/services/wallet_service_types_state.rs | 23:impl UnlockAttemptState {;44:impl RateLimitWindowState {;55:pub enum RateLimitPrecheck { | none | MOVE (move-services-wallet-types) | crates/z00z_wallets/src/services/wallet/types/wallet_service_types_state.rs | Higher cohesion around one runtime responsibility. | Requires staged path rewrites and compatibility re-exports. |
