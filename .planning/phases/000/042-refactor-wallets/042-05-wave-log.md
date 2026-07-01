# 042-05 Wave Log

## Wave 1 Kickoff

**Date:** 2026-05-05

**Objective:** Freeze the execution ledger, record the current live surface inventory, and lock the no-backward-compatibility migration policy before any source edits.

### Bootstrap Gate

**Command:** `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

**Result:** Passed.

**Observed output:**

- Multiple crate test suites passed, including `z00z_core`, `z00z_storage`, and `z00z_utils` release/profile checks.
- `z00z_wallets` compiled successfully and its tests passed.
- The only notable output was a small set of `missing_docs` warnings in `crates/z00z_wallets/src/stealth/output/output.rs`.
- Final bootstrap marker: `=== BOOTSTRAP COMPLETE ===`.

**Interpretation:** The mandatory fail-fast gate is green. The warnings are pre-existing or non-blocking for the current wave.

### Step 0 Inventory

**Inventory query set:**

- `rg "z00z_address|Z00ZSingleAddress|Z00ZDualAddress|Z00ZAddressFeatures" crates/z00z_wallets/src crates/z00z_wallets/tests`
- `rg "derive_dual_address|validate_address|label_address|list_addresses|PersistAddressInfo|RuntimeAddressFilter" crates/z00z_wallets/src crates/z00z_wallets/tests`

**Inventory result summary:**

- The first query returned a broad set of live matches, including public re-exports in `crates/z00z_wallets/src/lib.rs` and `crates/z00z_wallets/src/key/mod.rs`, legacy implementation files under `crates/z00z_wallets/src/address/z00z_address/**`, and live RPC/session callers.
- The second query returned live matches for `wallet.key.derive_dual_address`, `wallet.key.list_addresses`, `wallet.key.validate_address`, `wallet.key.label_address`, `RuntimeAddressFilter`, and `PersistAddressInfo` across RPC types, wiring, support helpers, and tests.
- The third query found `AddressDeriverState`, `.addr_cache`, and the old address-oriented persistence/state wiring in wallet snapshot and session code.

**Representative live surfaces found:**

- `crates/z00z_wallets/src/address/mod.rs`
- `crates/z00z_wallets/src/lib.rs`
- `crates/z00z_wallets/src/key/mod.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_derive.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_snapshot.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
- `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`
- `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`
- `crates/z00z_wallets/src/address/z00z_address/**`
- `crates/z00z_wallets/src/address/z00z_address.tar.gz`
- `crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`

### Bucket Split

**Legacy implementation to delete:** address codec and manager files, wallet-session address derivation/recovery/snapshot/rotation helpers, RPC server implementations, and the stale `z00z_address` archive subtree.

**Public facade / re-export to remove:** `lib.rs`, `key/mod.rs`, `address/mod.rs`, `wallet.rs`, `wallet_service.rs`, and `services/mod.rs` still expose legacy address symbols or transitively depend on them.

**Wallet-session caller to migrate:** session derivation, recovery, snapshot, and rotation code still use `AddressDeriverState`, address counters, and `.addr_cache` semantics.

**RPC caller to migrate:** `wallet.key.derive_key`, `wallet.key.derive_dual_address`, `wallet.key.list_addresses`, `wallet.key.validate_address`, and `wallet.key.label_address` remain wired.

**Tests to delete or rewrite:** address codec tests, RPC key tests, address-manager tests, snapshot/import/export tests, and any source-shape guards that still assume legacy address APIs.

**Historical docs / planning references:** `KEYS_EXPALNATION.md`, `WALLET-GUIDE.md`, `Z00Z-ADDRESS-GUIDE.md`, and `.planning` references need updates or archival handling.

### Migration Policy

**Decision:** epoch bump.

**Rationale:**

- The product explicitly accepts no backward compatibility for legacy address APIs.
- Legacy address-to-receiver label conversion is not reliably deterministic in the general case.
- Rejecting old snapshots/backups is cheaper and less ambiguous than inventing a partial one-shot converter.
- The policy keeps counter semantics and recovery determinism explicit instead of silently reshaping old state.

**Operational effect:** old wallet snapshots/backups will be rejected rather than translated; new stealth-only state becomes the only supported runtime format.

### Current Status

- Wave 1 control artifacts are in place.
- The controlling ledger now exists at `042-05-spec-coverage.md`.
- The execution log is now established at `042-05-wave-log.md`.
- The next slice is the receiver DTO and API-name migration from Step 1.

### Receiver-Card Display Field Rename

**Change:** Renamed the receiver-card display field from `address` to `owner_handle_display` in the response DTO and its direct construction sites.

**Validated by:**

- `cargo test -p z00z_wallets --test test_rpc_wiring_spec_a test_receiver_card_ok -- --nocapture`
- `cargo test -p z00z_wallets --lib test_get_receiver_card_ok -- --nocapture`

**Result:** Both tests passed. The receiver-card payload still serializes and validates correctly, and the output field no longer teaches an address-shaped display name.

### Receiver List and Label Aliases

**Change:** Added the receiver-oriented `wallet.key.list_receivers`, `wallet.key.validate_receiver_card`, and `wallet.key.label_receiver` aliases and wired them to the legacy list-addresses / validation / label implementations.

**Validated by:**

- Re-running the focused receiver-card integration and unit tests after the alias additions

**Result:** The crate still builds, the receiver-card path remains green, and the receiver-card validation alias now checks the compact card payload. The broader receiver-method cleanup is still open.

### Derive Surface De-Addressing

**Change:** Removed the legacy address string from live derive, deleted the public `wallet.key.derive_dual_address` RPC surface, and renamed the active derive route/DTO to `wallet.key.derive_receiver` / `RuntimeDeriveReceiverResponse`.

**Source edits:**

- `crates/z00z_wallets/src/adapters/rpc/types/key.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_derive.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs`
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
- `crates/z00z_wallets/tests/test_rpc_types_serialization.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/test_key_suite.rs`

**Contract result:**

- `RuntimeDeriveReceiverResponse` now returns `{ public_key, path }` only.
- `derive_receiver_impl(...)` no longer constructs `Z00ZSingleAddress` or encodes a bech32 address.
- `wallet.key.derive_receiver` is the active public derive route in the RPC trait, server forwarding layer, dispatcher wiring, and E2E tests.
- `wallet.key.derive_key` and `wallet.key.derive_dual_address` are no longer part of the active wallet RPC contract.

**Validated by:**

- `cargo test -p z00z_wallets --test test_rpc_key_derive_e2e -- --test-threads=1 --nocapture`
- `cargo test -p z00z_wallets --lib test_derive_key_ -- --nocapture`
- `cargo test -p z00z_wallets --lib test_runtime_derive_key_response -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_types_serialization test_key_types_roundtrip -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_wiring_spec_a -- --nocapture`

**Result:** The live derive RPC no longer emits a legacy address string, the dual-address endpoint is gone, and the active public derive contract is now receiver-native by route and DTO name.

**Still open:**

- The phase still needs a final decision on whether `{ public_key, path }` is the permanent receiver-material shape or only an intermediate receiver-native bridge.
- Broader wallet and planning docs outside the touched local files still need the full Wave 5 rewrite.

### Receiver Admin Vocabulary Cleanup

**Change:** Promoted `RuntimeReceiverFilter` to the primary receiver-list DTO and renamed the active receiver admin helper path from address-oriented helper names to receiver-oriented names.

**Source edits:**

- `crates/z00z_wallets/src/adapters/rpc/types/key.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_build.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/test_key_suite.rs`
- `crates/z00z_wallets/tests/test_rpc_types_serialization.rs`

**Contract result:**

- `RuntimeReceiverFilter` is now the primary list-receivers DTO and `RuntimeAddressFilter` is only a legacy alias.
- `apply_receiver_filter(...)`, `get_receiver_labels(...)`, and `upsert_receiver_label(...)` replace the old address-named helper surface in the active receiver admin path.
- `list_receivers_impl(...)` and `label_receiver_impl(...)` now call the receiver-named helpers directly.
- `wallet.key.label_receiver` no longer accepts the legacy request field name `address`; callers must send `receiver_id`.

**Validated by:**

- `cargo test -p z00z_wallets --lib test_list_receivers_returns_receiver_ids -- --nocapture`
- `cargo test -p z00z_wallets --lib test_label_receiver_by_receiver_id -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_types_serialization test_key_types_roundtrip -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_key_derive_e2e test_label_receiver_rejects_legacy_address_field -- --test-threads=1 --nocapture`

**Result:** The active receiver admin/list surface no longer teaches address-named filter/label helper vocabulary.

**Still open:**

- The backing wallet service now exposes receiver-native live helper names, but the underlying derivation/persistence engine still depends on `AddressManagerImpl`, `AddressDeriverState`, and `.addr_cache`; that deeper cleanup belongs to later Phase 042 waves.

### Live Session Vocabulary Cleanup

**Change:** Removed the live `derive_dual_address_for_path(...)` helper from the active derivation file and renamed the active session/service list-label state from address-owned names to receiver-native names.

**Source edits:**

- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_core.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_state.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_build.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_password.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction_variants.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_guards.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_transfer_import.rs`
- `crates/z00z_wallets/tests/test_phase30_split.rs`

**Contract result:**

- The active live session/service lane now uses `WalletReceiverDeriverState`, `WalletReceiverDeriverHandle`, `ReceiverUsageOracle`, `wallet_receiver_derivers`, `wallet_receiver_deriver_counters`, and `receiver_labels`.
- The live helper feeding receiver listings is now `list_cached_receivers(...)`, and its active callers in key RPC admin flows and asset owner checks were migrated.
- The active derivation file no longer exposes the dead `derive_dual_address_for_path(...)` helper.

**Validated by:**

- `cargo test -p z00z_wallets --test test_rpc_key_derive_e2e -- --test-threads=1 --nocapture`
- `cargo test -p z00z_wallets --lib test_list_receivers_returns_receiver_ids -- --nocapture`
- `cargo test -p z00z_wallets --test test_phase30_split -- --nocapture`

**Result:** The live session/service slice no longer teaches address-owned vocabulary for the active receiver list/label path, but the actual derivation engine still sits on `AddressManagerImpl` and persisted `AddressDeriverState`, which is why `z00z_address/` cannot yet be honestly deleted.

### Persisted Receiver-Deriver Contract Cleanup

**Change:** Renamed the persisted wallet derivation contract from address-owned names to receiver-native names and encoded the epoch bump directly in the live snapshot format.

**Source edits:**

- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_impl.rs`
- `crates/z00z_wallets/src/services/mod.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types.rs`
- `crates/z00z_wallets/src/services/wallet/wallet_service.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_core.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
- `crates/z00z_wallets/src/wallet/snapshot/test_snapshot_suite.rs`
- `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs`
- `crates/z00z_wallets/src/backup/import/test_backup_importer_suite.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_service_suite.rs`
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- `crates/z00z_wallets/tests/test_backup_restore_identity.rs`
- `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs`
- `crates/z00z_wallets/tests/test_backup_kdf_contract.rs`
- `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`

**Contract result:**

- `AddressDeriverState` is replaced by `ReceiverDeriverState` on the active persisted snapshot path.
- `WalletPersistenceState.address_deriver` / `WalletSnapshotArgs.address_deriver` are replaced by `receiver_deriver`.
- `WalletPersistenceState::VERSION` is now `5`, matching the epoch-bump policy for this no-backcompat migration.
- Active backup/import/export and wallet snapshot fixtures compile against the receiver-native persisted contract.

**Validated by:**

- `cargo test -p z00z_wallets --lib test_snapshot_version -- --nocapture`
- `cargo test -p z00z_wallets --no-run`
- `cargo test -p z00z_wallets --test test_backup_restore_identity -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_key_derive_e2e -- --test-threads=1 --nocapture`

**Result:** The persisted wallet snapshot contract is now receiver-native by type and field name, and the wallet crate compile gate is green after the backup/import/export fixture rewrite.

**Still open:**

- The live derivation engine still wraps `AddressManagerImpl`.
- `.addr_cache` persistence and the stale address-manager-backed runtime cache policy are still open Phase 042 work.

### Live Receiver Session-State Rename

**Change:** Renamed the active wallet session deriver helper and manager field from address-owned names to receiver-native names.

**Source edits:**

- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_state.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs`

**Contract result:**

- `create_address_deriver_state(...)` is now `create_receiver_deriver_state(...)` on the live path.
- `get_create_wallet_address_deriver(...)` is now `get_create_wallet_receiver_deriver(...)` on the live path.
- `WalletReceiverDeriverState.address_manager` is now `receiver_manager` on the live path.

**Validated by:**

- `cargo test -p z00z_wallets --no-run`
- `cargo test -p z00z_wallets --test test_rpc_key_derive_e2e -- --test-threads=1 --nocapture`

**Result:** The active wallet session derivation lane no longer teaches address-owned helper or field names, even though it still uses `AddressManagerImpl` internally.

### Unwired Duplicate Session File Cleanup

**Change:** Deleted the two duplicate wallet-session split files that still preserved address-era derivation logic after confirming they were not part of the active session module graph.

**Source edits:**

- Deleted `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_seed_derivation.rs`
- Deleted `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_snapshot.rs`
- Confirmed active session root wiring in `crates/z00z_wallets/src/services/wallet/session/wallet_service_session.rs`

**Contract result:**

- `wallet_service_session.rs` still includes only `wallet_service_session_construction.rs`, `wallet_service_session_derivation.rs`, and `wallet_service_session_runtime.rs`.
- The deleted split files were unwired residue, not part of the active compiled session root.
- The remaining live Step 2 blocker is no longer duplicate file residue; it is the active `AddressManagerImpl` and `.addr_cache` ownership chain in the receiver-native derivation and recovery files.

**Validated by:**

- `cargo test -p z00z_wallets --no-run`

**Result:** The wallet crate compile gate stayed green after deleting the duplicate files, so Phase 042 can treat them as closed residue rather than live implementation surface.

### Receiver Manager, Cache, and Public Leak Cleanup

**Change:** Rebound the active wallet runtime path to receiver-native manager/cache vocabulary, removed the crate-root and key-module legacy `Z00Z*` re-export leaks, and renamed the public address facade from `z00z_address` to `stealth_address`.

**Source edits:**

- `crates/z00z_wallets/src/address/mod.rs`
- `crates/z00z_wallets/src/key/mod.rs`
- `crates/z00z_wallets/src/lib.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types.rs`
- `crates/z00z_wallets/src/services/wallet/wallet_service.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_state.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_build.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_password.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction_variants.rs`
- `crates/z00z_wallets/src/wallet/entity/wallet_entity_core.rs`
- `crates/z00z_wallets/src/wallet/entity/wallet_entity_constructor.rs`

**Contract result:**

- The active wallet service path now imports `ReceiverManagerImpl` instead of raw `AddressManagerImpl`.
- The live runtime/cache lane now writes `receiver_cache` files through `receiver_cache_file_path(...)` instead of `.addr_cache` naming.
- Wallet entity and session-construction placeholders now use `receiver_manager` instead of `address_manager`.
- `key/mod.rs` and `lib.rs` no longer re-export `Z00ZAddressFeatures`, `Z00ZDualAddress`, or `Z00ZSingleAddress`.
- The public facade module name is now `stealth_address`, and the public alias `address_manager` is replaced by `receiver_manager`.

**Validated by:**

- `cargo test -p z00z_wallets --no-run`

**Result:** The active runtime/public surface no longer teaches raw `AddressManagerImpl`, `address_manager`, or `.addr_cache` vocabulary; the remaining blocker is the internal implementation family still hidden behind the receiver alias.

### Stealth Source-Tree Routing Cleanup

**Change:** Removed the last `z00z_address` routing name from the wallet source tree by renaming the backing directory to `stealth_address`, then renaming the internal helper-file prefixes away from `z00z_address_*`.

**Source edits:**

- Renamed `crates/z00z_wallets/src/address/z00z_address/` to `crates/z00z_wallets/src/address/stealth_address/`
- Renamed helper files `z00z_address_features.rs`, `z00z_address_validation.rs`, `z00z_address_codec.rs`, `z00z_address_normalize.rs`, `z00z_address_parts.rs`, and `test_z00z_address_suite.rs`
- Updated `crates/z00z_wallets/src/address/mod.rs`
- Updated `crates/z00z_wallets/src/address/stealth_address/mod.rs`
- Updated `crates/z00z_wallets/src/address/stealth_address/test_stealth_address_suite.rs`

**Contract result:**

- `crates/z00z_wallets/src/**/*.rs` no longer contains the `z00z_address` routing literal.
- The stealth facade now points at `#[path = "stealth_address/mod.rs"]`.
- The internal helper include surface no longer uses `z00z_address_*` filenames.
- The stale `z00z_address.tar.gz` archive sibling is absent from the current tree.

**Validated by:**

- `cargo test -p z00z_wallets --no-run`
- `rg "\bz00z_address\b" crates/z00z_wallets/src`
- `rg "z00z_address_" crates/z00z_wallets/src`

**Result:** The remaining Phase 042 residue is no longer a directory or helper-file routing problem. The honest next blocker is the still-live `Z00Z*` address type family and `AddressManagerImpl` implementation under the stealth-native directory structure.

### Stealth Type, Manager, and Receiver-List Continuation Cleanup

**Change:** Renamed the remaining live public stealth type family to `Stealth*`, renamed the concrete manager implementation to `ReceiverManagerImpl` / `AsyncReceiverManagerImpl`, removed dead list/filter DTO aliases, renamed `Z00ZAddressError` to `StealthAddressError`, and flipped the live manager inventory call from `list_addresses(...)` to `list_receivers(...)`.

**Source edits:**

- `crates/z00z_wallets/src/address/mod.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_builder.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_async.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_runtime_maintenance.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_runtime_derive.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_snapshot_io.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_trait_impl.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_trait.rs`
- `crates/z00z_wallets/src/address/manager/test_address_manager_suite.rs`
- `crates/z00z_wallets/src/address/stealth_address/stealth_address_validation.rs`
- `crates/z00z_wallets/src/address/stealth_address/z00z_single_address.rs`
- `crates/z00z_wallets/src/address/stealth_address/z00z_dual_address.rs`
- `crates/z00z_wallets/src/address/stealth_address/z00z_single_address_transport.rs`
- `crates/z00z_wallets/src/address/stealth_address/z00z_dual_address_transport.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/key.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs`

**Contract result:**

- `ReceiverManagerImpl` and `AsyncReceiverManagerImpl` are now the live concrete manager names on the compiled path.
- The public receive surface is now `ReceiverCard`, `PaymentRequest`, `ValidatedRequest`, `StealthOutputScanner`, and receiver-key material. No legacy stealth-address family remains in the wallet crate source tree.
- `RuntimeAddressFilter`, `PersistAddressInfo`, and `RuntimeListAddressesResponse` are removed from the active RPC type layer.
- The live manager inventory seam is now `list_receivers(...)` across trait impls, async wrapper, wallet session recovery/rotation callers, and manager tests.

**Validated by:**

- `cargo test -p z00z_wallets --no-run`

**Result:** The honest Phase 042 blocker is no longer the concrete manager impl name or the public `Z00Z*` type family. The remaining cleanup is concentrated in the `AddressManager` / `AsyncAddressManager` trait surface, address-era RPC response/comment residue, legacy helper struct names such as `Z00z*Serde`, and stale doctest/test references.

### Receiver Config Seam and Manager Contract Cleanup

**Change:** Finished the remaining receiver-native contract rename inside the wallet crate by renaming the public/internal manager contracts to `ReceiverManager`, `AsyncReceiverManager`, `ReceiverManagerConfig`, `ReceiverManagerError`, and `ReceiverManagerResult`, then cutting over the cache/rate-limit config seam to receiver-native keys.

**Source edits:**

- `crates/z00z_wallets/src/address/mod.rs`
- `crates/z00z_wallets/src/wallet/mod.rs`
- `crates/z00z_wallets/src/address/manager/mod.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_config.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_trait.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_async.rs`
- `crates/z00z_wallets/src/address/manager/address_manager_impl_snapshot.rs`
- `crates/z00z_wallets/src/address/manager/rate_limiter_bucket.rs`
- `crates/z00z_wallets/src/services/wallet/paths/wallet_paths.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/wallet_service.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs`
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
- `crates/z00z_wallets/benches/address_derivation.rs`
- `crates/z00z_wallets/benches/async_batch_threshold_bench.rs`

**Contract result:**

- The wallet crate no longer exposes `AddressManager*` / `AsyncAddressManager` contracts on the live path.
- The cache/rate-limit config seam now uses `wallet.receiver.cache_size`, `wallet.receiver.rate_limit.*`, `Z00Z_WALLET_RECEIVER_CACHE_SIZE`, `Z00Z_WALLET_RECEIVER_DERIVE_RATE_PER_SEC`, and `Z00Z_WALLET_RECEIVER_DERIVE_BURST`.
- The representative wallet-path tests were updated to the receiver-native seam.

**Validated by:**

- `cargo test -p z00z_wallets --lib test_receiver_rate_limit_env_ok -- --nocapture`
- `cargo test -p z00z_wallets --lib test_receiver_cache_size_yaml_valid -- --nocapture`
- `cargo check -p z00z_wallets --all-targets`

**Result:** The manager contract rename and config/env seam rename are green across lib, tests, benches, and helper targets.

### Stealth Filename, Helper Binary, and Residue Sweep Cleanup

**Change:** Renamed the remaining stealth source filenames, guide, and helper binary away from `z00z_address` or `addr-convert`, then refreshed the nearby wallet docs/tests to match the stealth-native surface.

**Source edits:**

- Renamed `crates/z00z_wallets/src/address/stealth_address/z00z_single_address.rs` to `stealth_single_address.rs`
- Renamed `crates/z00z_wallets/src/address/stealth_address/z00z_single_address_serde.rs` to `stealth_single_address_serde.rs`
- Renamed `crates/z00z_wallets/src/address/stealth_address/z00z_single_address_transport.rs` to `stealth_single_address_transport.rs`
- Renamed `crates/z00z_wallets/src/address/stealth_address/z00z_dual_address.rs` to `stealth_dual_address.rs`
- Renamed `crates/z00z_wallets/src/address/stealth_address/z00z_dual_address_serde.rs` to `stealth_dual_address_serde.rs`
- Renamed `crates/z00z_wallets/src/address/stealth_address/z00z_dual_address_transport.rs` to `stealth_dual_address_transport.rs`
- Renamed `crates/z00z_wallets/src/address/stealth_address/Z00Z-ADDRESS-GUIDE.md` to `STEALTH-ADDRESS-GUIDE.md`
- Renamed `crates/z00z_wallets/bin/z00z-wallet-addr-convert.rs` to `crates/z00z_wallets/bin/z00z-wallet-stealth-convert.rs`
- Updated `crates/z00z_wallets/src/address/stealth_address/mod.rs`
- Updated `crates/z00z_wallets/README.md`
- Updated `crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md`
- Updated `crates/z00z_wallets/src/key/manager/KEYS-DERIVATION.md`
- Updated `crates/z00z_wallets/tests/test_wallet_service_errors.rs`

**Validated by:**

- `cargo check -p z00z_wallets --all-targets`
- `rg "AddressManagerConfig|AddressManagerError|AddressManagerResult|AsyncAddressManager|AddressManagerImpl|AddressManager::|Z00ZSingleAddress|Z00ZDualAddress|Z00ZAddressFeatures|z00z_address\.rs|z00z-wallet-addr-convert|wallet\.address\.(cache_size|rate_limit)|Z00Z_WALLET_ADDRESS_CACHE_SIZE|Z00Z_WALLET_ADDR_DERIVE" crates/z00z_wallets`
- `file_search("crates/z00z_wallets/**/*z00z*address*")`
- `file_search("crates/z00z_wallets/**/*addr*convert*")`

**Result:** The targeted Phase 042 wallet-crate residue inventory is clean for source, tests, benches, docs, helper binary, and filenames.

### Asset RPC Contract Truth Pass

**Change:** Corrected the asset send/receive RPC responses so the live field name matches the payload semantics: `stealth_address` was replaced with `owner_handle` in both send and receive responses, server construction, stub defaults, and dispatcher roundtrip coverage.

**Source edits:**

- `crates/z00z_wallets/src/adapters/rpc/types/asset.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs`
- `crates/z00z_wallets/src/wallet/entity/wallet_entity_asset_api.rs`
- `crates/z00z_wallets/src/wallet/responses/stub_responses_asset.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs`
- `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs`

**Contract result:**

- `RuntimeSendAssetResponse` now exposes `owner_handle` rather than a misnamed stealth-address field.
- `RuntimeReceiveAssetResponse` now exposes `owner_handle` rather than a misnamed stealth-address field.
- The server implementation and stub responses both populate the same owner-handle contract.
- Dispatcher roundtrip coverage now asserts that the old JSON key is absent.

**Validated by:**

- `cargo test -p z00z_wallets --lib test_asset_send_ -- --test-threads=1`
- `cargo test -p z00z_wallets --lib test_asset_receive_ -- --test-threads=1`
- `cargo test -p z00z_wallets --test test_rpc_dispatcher_roundtrip -- --test-threads=1`

**Result:** The renamed asset RPC contract is green in focused send/receive tests and in dispatcher roundtrip validation. The only output from these runs was the same pre-existing `missing_docs` warnings in `crates/z00z_wallets/src/stealth/output/output.rs`.

### Current Closeout State

**Status:** Source cleanup and focused validation are green; the remaining open work is doc/planning reconciliation and final closeout packaging.

**Still open:**

- `042-05-PLAN.md` still embeds the copied source spec verbatim and should remain untouched in that block.
- `042-05-SUMMARY.md` is now the required closeout artifact and should carry the final evidence map.
- Broader docs/planning sync remains the last visible closeout gap outside the validated source tree.
