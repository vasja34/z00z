# 036-a2 Legacy Removing Spec

## Goal

Delete remaining Rust code containing `legacy` outside Tari. This is a
deletion-first migration slice, not a preservation exercise.

User override, 2026-04-17:

- delete `legacy` code instead of renaming it;
- do not create intentional non-deletion states inside this phase;
- any remaining production `legacy` owner is a blocker or open defect, not an
  approved end state.

Status update, 2026-04-18:

- the authoritative Rust substring scan
  `rg -n "legacy|Legacy" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`
  is now zero after the `036-15` closure and the bootstrap revalidation rerun;
- the last simulator Rust residue in `crates/z00z_simulator/tests/test_claim_emit.rs`
  was deleted by removing `LEGACY_PROOF_STUB` and `LEGACY_SIG_STUB`, so that
  command remains zero on the live root `crates/` tree;
- `.temp/**` mirror trees are not part of this phase scope even if a broader
  workspace grep surfaces third-party or scratch `legacy` matches there;
- unless a fresh scan finds new Rust residue, the detailed tables below are
  retained as delete-first audit history rather than as a live blocker list.

## Scope

This spec governs the post-036 full legacy-deletion planning wave.

In scope:

- Rust code under `crates/`, excluding `crates/z00z_crypto/tari/`;
- production symbol owners that still contain `legacy` in their identifier;
- fixture and test residue that must be deleted after the owning production
  deletion decision lands;
- Cargo feature and compatibility gates whose names still contain `legacy`.

Out of scope:

- Tari vendor code;
- blind text replacement across comments, docs, logs, or generated assets;
- protocol or persisted-value changes without an explicit migration step;
- speculative cleanup that is not backed by repository evidence.

## Repository-Backed Baseline

The current planning baseline is the quick-task inventory at:

- `.planning/quick/260417-pvi-implement-036-a1-versioning-spec-v2-md-r/LEGACY-INVENTORY.md`

That inventory established:

- `358` raw `legacy` substring matches in Rust code outside Tari;
- `.planning/phases/036-rename/036-a1-versioning-spec-V2.md` contains `85`
  rows whose tracked owner, local symbol, or tracked boundary still contains
  `legacy`;
- remaining production owners exist in storage, wallet backup, wallet KDF,
  wallet DB, simulator helpers, RPC/file cleanup helpers, and Cargo feature
  gates;
- the current workspace scan found declaration-level `legacy` symbols in `src`
  and `tests`, and found no declaration-level `legacy` symbols in
  `examples/**/*.rs` or `benches/**/*.rs`;
- remaining `legacy` names are all delete targets; some rows are blocked only
  because explicit prerequisites still need to be removed first.

## Deletion Authority Override

All planning and execution under this spec must interpret `legacy` rows as
delete targets.

A row marked `blocked until prerequisite proof` is still a delete target; the
label only records the proof boundary that must clear before deletion can
land.

- forward-only migration is not a valid terminal outcome for this phase.
- intentional non-deletion state is not a valid terminal outcome for this phase.
- any copied mixed-action wording in inventory rows is stale shorthand and
  must be executed as deletion unless a blocker is recorded explicitly.

## Detailed Legacy Removal Table

This table is the authoritative deletion roadmap for this migration slice.

Historical note:

- this row inventory is preserved to show what the continuation audited,
  deleted, or blocked while the slice was live;
- after the `036-15` closure refresh, do not treat these rows as current Rust
  hits without rerunning the authoritative substring scan.

Interpretation rules:

- every `036-a1-versioning-spec-V2.md` row that still carries a `legacy`
  symbol becomes a removal target here, even if Phase 036 previously marked
  that row `keep | same`;
- `Planned end state` describes the intended result of the full removal
  slice, not immediate authorization to edit production code in one pass;
- `Delete gate` names the proof boundary that must be satisfied before the
  symbol can actually disappear.

### Src Scope Removal Targets

| Bucket | Path | Symbols scheduled for removal | Evidence source | Planned end state | Delete gate |
| --- | --- | --- | --- | --- | --- |
| src | `crates/z00z_utils/src/compression/test_compression.rs` | `test_lz4_legacy_rejected`, local `legacy` | V2 rows `1`, `133` | delete explicit legacy test names | after compression reject-path tests survive under legacy-free names |
| src | `crates/z00z_utils/src/io/fs.rs` | `legacy_tests` | V2 row `2` | delete the legacy-named test module | after fs legacy-path coverage survives under a legacy-free module name |
| src | `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs` | `legacy_made_rows` | V2 row `3` | delete the legacy-named helper | after Stage 6 historical row-shape compatibility is deleted with proof |
| src | `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs` | local `legacy_report` | V2 row `164` | delete the legacy-named report variable and lane | after the old claim wallet import report lane is deleted behind legacy-free naming |
| src | `crates/z00z_core/src/assets/asset_ownership.rs` | `owner_message_bytes` legacy parameter, `to_owner_message_legacy`, local `legacy_message` | V2 rows `19`, `20`, `157` | delete legacy-named owner-message path | after owner-signature compatibility route is deleted or explicitly blocked |
| src | `crates/z00z_core/src/assets/test_asset_suite.rs` | `test_legacy_owner_signature_still_verifies` | V2 row `21` | delete the legacy-named compatibility test | after owner-message compatibility proof is closed |
| src | `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | `test_proof_blob_decode_legacy_v0_upgrades_root_bind`, `LegacyProofBlobWire`, locals `legacy`, `legacy_bytes` | V2 rows `33`, `141`, `142`; workspace scan miss | delete legacy proof-blob decode fixture surface | after `ProofBlobV0` compatibility is deleted or explicitly rejected |
| src | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | `legacy_hash`, locals `legacy_exec_id`, `legacy_draft_id`, `legacy_check_id`, `is_legacy_era` | V2 rows `43`, `158`-`161` | delete legacy-era validation naming | after persisted checkpoint-era metadata no longer accepts legacy hash lineage |
| src | `crates/z00z_storage/src/assets/store_internal/redb_backend_helpers.rs` | local `legacy` | V2 row `148` | delete legacy decode variable | after claim-nullifier V0 rehydrate lane is deleted |
| src | `crates/z00z_storage/src/checkpoint/artifact_types.rs` | `LEGACY_OPAQUE`, `is_legacy_opaque` | V2 row `48`; workspace scan miss | delete legacy opaque proof-system naming | after checkpoint opaque compatibility contract fate is decided |
| src | `crates/z00z_storage/src/checkpoint/codec.rs` | `LegacyArtWire`, `decode_legacy_artifact_bin`, `decode_legacy_artifact_json`, local `legacy` | V2 rows `49`, `50`, `147`; workspace scan miss | delete legacy artifact decode lane | after old artifact blob imports are deleted or explicitly rejected |
| src | `crates/z00z_storage/src/checkpoint/artifact_final.rs` | `new_legacy`, `check_legacy_sys` | V2 rows `51`, `52` | delete legacy artifact constructor/validator surface | after legacy opaque artifact creation is no longer needed |
| src | `crates/z00z_storage/src/checkpoint/link.rs` | `LegacyCheckpointLink`, local `legacy` | V2 row `146`; workspace scan miss | delete legacy link decode surface | after historical checkpoint-link payloads are deleted or explicitly rejected |
| src | `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs` | `migrate_legacy_wallet_files`, local `legacy_backup` | V2 rows `75`, `140` | delete legacy wallet-file migration surface | after plaintext filename migration and `.legacy` rollback path are closed |
| src | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | `legacy_v1_bytes`, `legacy_v1_restore_fails` | V2 rows `78`, `79` | delete backup v1 reject fixtures with legacy naming | after backup-v1 compatibility proof is closed |
| src | `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs` | `test_wallet_id_accepts_legacy_id`, `test_wallet_id_password_accepts_legacy_id` | V2 rows `80`, `81` | delete legacy-id compatibility tests | after RPC wallet-id compatibility path is deleted |
| src | `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs` | `is_legacy`, local `legacy_bytes` | V2 rows `85`, `174` | delete legacy file-detection helper naming | after unsuffixed and legacy-bytes detection lanes are deleted |
| src | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` | `asset_send_rejects_legacy`, local `legacy` | V2 rows `132`, `190` | delete the explicit legacy reject fixture | after asset send compatibility rejection coverage is preserved under legacy-free names |
| src | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | `test_legacy_compact_rejected`, local `legacy` | V2 rows `77`, `166` | delete the explicit legacy compact-card fixture | after receiver-card reject coverage remains intact |
| src | `crates/z00z_wallets/src/core/key/key_manager_redb_tests.rs` | `reject_legacy_kdf_params_on_wrap` | V2 row `91` | delete the explicit legacy KDF reject test | after KDF-v1 rejection proof remains under legacy-free naming |
| src | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | `test_rejects_legacy_kdf_version`, local `legacy` | V2 rows `88`, `189` | delete the explicit legacy KDF fixtures | after wallet KDF v1 rejection remains covered |
| src | `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` | `LEGACY_PROOF_STUB`, `LEGACY_SIG_STUB`, `test_legacy_proof_stub`, `test_legacy_sig_stub` | V2 rows `96`, `97`; workspace scan miss | delete legacy claim stub naming | after claim proof/signature stub compatibility story is closed |
| src | `crates/z00z_wallets/src/core/tx/tx_verifier_tests.rs` | `legacy_digest`, locals `legacy_a`, `legacy_b` | workspace scan miss | delete legacy-named boundary-collision helpers and locals | after digest-framing regression proof is preserved under legacy-free test names |
| src | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | `test_import_legacy_v1_backup_is_rejected`, `test_import_rejects_legacy_payload_versions`, `build_legacy_v1_bytes` | V2 rows `98`, `99`, `101` | delete backup legacy fixture builders and tests | after backup import rejection coverage stays intact |
| src | `crates/z00z_wallets/src/core/wallet/snapshot_tests.rs` | `test_rejects_legacy_snapshot_versions` | V2 row `105` | delete the snapshot legacy reject fixture | after snapshot version rejection proof is preserved |
| src | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | `legacy_bech32` | V2 row `180` | delete legacy Bech32 fixture naming | after historical address fixture coverage is preserved under legacy-free names |
| src | `crates/z00z_wallets/src/db/redb_wallet_store_create.rs` | `legacy_tmp_path` | V2 row `175` | delete the legacy-named temp-path helper variable | after old temp file cleanup semantics remain under legacy-free naming |
| src | `crates/z00z_wallets/src/db/redb_wallet_store.rs` | `INDEX_FORMAT_VERSION_LEGACY` | workspace scan miss | delete legacy index-format constant name | after index format v1 is migrated or rejected explicitly |
| src | `crates/z00z_wallets/src/core/key/seed_cipher_params.rs` | `Argon2idParams::LEGACY` | workspace scan miss | delete legacy Argon2 profile constant name | after v1 seed containers are migrated or rejected |
| src | `crates/z00z_wallets/src/core/backup/wallet_backup_container.rs` | `LegacyBackupContainer` | workspace scan miss | delete legacy backup container type | after legacy backup payload container is no longer accepted |
| src | `crates/z00z_wallets/src/core/backup/backup_wire.rs` | `LEGACY_EXPORT_PACK_FORMAT_VERSION` | workspace scan miss | delete legacy export pack constant name | after backup export pack v2 compatibility is deleted |

### Tests Scope Removal Targets

| Bucket | Path | Symbols scheduled for removal | Evidence source | Planned end state | Delete gate |
| --- | --- | --- | --- | --- | --- |
| tests | `crates/z00z_storage/tests/test_checkpoint_store_api.rs` | `test_legacy_opaque_proof_rejects_on_seal` | V2 row `17` | delete the explicit legacy reject test | after checkpoint legacy opaque seal path is deleted |
| tests | `crates/z00z_crypto/tests/test_fail_closed.rs` | `try_hmac_helpers_match_legacy_outputs` | V2 row `18` | delete the explicit legacy comparison test | after legacy output-compat expectation is deleted |
| tests | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | `LegacyArtWire`, `legacy_hash`, `LegacyLinkWire`, `test_redb_loads_legacy_claim_null_rows`, `test_redb_rejects_legacy_checkpoint_link_bundle`, locals `legacy_key`, `legacy_row`, `legacy_exec_id`, `legacy_check_bytes`, `legacy_draft_id`, `legacy_check_id`, `legacy_link`, `legacy_link_bytes` | V2 rows `29`, `31`, `32`, `149`-`156`; workspace scan misses | delete historical redb rehydrate fixture surface | after persisted-row and checkpoint-link compatibility are deleted |
| tests | `crates/z00z_storage/tests/test_checkpoint_codec.rs` | `test_legacy_stage6_wrapper_rejects` | V2 row `34` | delete the explicit legacy wrapper reject test | after stage6 wrapper compatibility is deleted |
| tests | `crates/z00z_storage/tests/checkpoint/test_fixtures.rs` | `LegacyMadeEnt`, `LegacyStage6`, `legacy_stage6_json` | V2 row `40`; workspace scan misses | delete legacy stage6 fixture types | after stage6 legacy JSON wrapper is no longer a supported input |
| tests | `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` | `LegacyArtWire`, `LegacyLinkWire`, `test_legacy_link_bytes_upgrade_to_bound_link`, `test_legacy_artifact_rejects_link_binding`, locals `legacy`, `legacy_art` | V2 rows `41`, `42`, `143`, `144`; workspace scan misses | delete legacy checkpoint-link injective fixtures | after link-byte upgrade lane is deleted |
| tests | `crates/z00z_storage/tests/test_checkpoint_finalization.rs` | `LegacyArtWire`, `test_legacy_opaque_finalize_rejects_live_surface`, `test_legacy_opaque_bytes_stay_legacy` | V2 rows `44`, `45`; workspace scan miss | delete legacy opaque finalization fixtures | after live-surface rejection boundary is deleted |
| tests | `crates/z00z_wallets/tests/test_backup_kdf_contract.rs` | `LegacyMd`, `LegacyEnc`, `LegacyComp`, `legacy_bytes`, `test_import_rejects_legacy_backup_contract`, `test_import_rejects_legacy_backup_payload_versions` | V2 rows `55`, `56`; workspace scan misses | delete legacy backup contract fixtures | after backup contract v1/v2 rejection lanes are closed |
| tests | `crates/z00z_wallets/tests/test_tx_digest_framing.rs` | `legacy_digest`, `tx_package_digest_rejects_legacy_boundary_collision` | V2 row `64`; workspace scan miss | delete legacy-named digest framing helpers and tests | after tx package digest collision proof is preserved under legacy-free naming |
| tests | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` | `test_open_rejects_legacy_wallet_kdf`, locals `legacy_kdf`, `legacy_record` | V2 row `66`, `176`, `177` | deleted; the duplicate reject fixture is gone, and the surviving wallet split guard now lives in `crates/z00z_wallets/tests/test_phase30_split.rs::test_wallet_source_split` | closed by lower-level key-manager rejection proof plus `test_redb_wlt_open.rs::test_open_rejects_v1_kdf` |
| tests | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | `test_open_rejects_v1_kdf`, locals `legacy_kdf`, `legacy_record`, `legacy_kdf_blob`, `legacy_record_blob` | V2 row `67`, `169`-`172` | delete redb open legacy KDF fixtures | after redb wallet-open V1 KDF reject path is deleted |
| tests | `crates/z00z_crypto/tests/test_public_surface.rs` | `test_public_surface_gates_legacy_claim_and_custom_zkpack` | workspace scan miss | delete the feature-gate legacy surface audit | after public legacy claim feature gate is deleted |
| tests | `crates/z00z_crypto/tests/test_hash_policy.rs` | local `legacy_wallet` | V2 row `139` | delete the legacy-named wallet-domain local | after legacy wallet-domain comparison is deleted |
| tests | `crates/z00z_wallets/tests/test_claim_state_compat.rs` | local `legacy` | V2 row `173` | delete legacy claim-state fixture locals | after claim-state compat fixture is deleted |
| tests | `crates/z00z_wallets/tests/test_phase30_split.rs` | local `legacy_include` | V2 row `188` | delete old include-assembly fixture locals with legacy naming | after split regression proof remains under legacy-free naming |
| tests | `crates/z00z_simulator/tests/test_stage4_source_shape.rs` | locals `legacy_stage4`, `legacy_stage6` | V2 rows `162`, `163` | delete legacy-named historical-shape fixture locals | after source-shape regression proof is preserved |
| tests | `crates/z00z_simulator/tests/test_claim_emit.rs` | `LEGACY_PROOF_STUB`, `LEGACY_SIG_STUB` | workspace scan miss | delete legacy simulator claim stubs | after simulator emit path no longer references legacy placeholders |

### Empty Buckets Verified By Workspace Scan

| Bucket | Result | Evidence source | Meaning for backlog |
| --- | --- | --- | --- |
| `examples/**/*.rs` | no declaration-level `legacy` symbols found | workspace scan | no example-file delete work is currently authorized |
| `benches/**/*.rs` | no declaration-level `legacy` symbols found | workspace scan | no bench-file delete work is currently authorized |

## Wave 0 Delete-Order Freeze For Src Scope Targets

This table is the row-exact Wave 0 freeze for every production owner listed in
`Src Scope Removal Targets`. Each row has exactly one explicit delete
disposition: `delete in Wave 2`, `delete in Wave 3`, or `blocked until
prerequisite proof`. The final column records either the exact blocker that
still prevents deletion or the repository-backed reason the row is already safe
for its assigned delete wave.

| Path | Symbols scheduled for removal | Wave 0 delete disposition | Protected contract or blocker note |
| --- | --- | --- | --- |
| `crates/z00z_utils/src/compression/test_compression.rs` | `test_lz4_legacy_rejected`, local `legacy` | delete in Wave 3 | Reject-path fixture naming only; no persisted, public, or transport contract is carried by the `legacy` identifier. |
| `crates/z00z_utils/src/io/fs.rs` | `legacy_tests` | delete in Wave 3 | Src-local test module name only; no runtime file-system contract depends on this identifier. |
| `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs` | `legacy_made_rows` | delete in Wave 2 | Helper name only; outward result is still `Vec<MadeEnt>` and no encoded value, schema marker, or transport literal contains `legacy`. |
| `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs` | local `legacy_report` | delete in Wave 2 | Local variable name only; the filesystem lane it points to is already the legacy-free literal `claim_wallet_import_report.json`. |
| `crates/z00z_core/src/assets/asset_ownership.rs` | `owner_message_bytes` legacy parameter, `to_owner_message_legacy`, local `legacy_message` | blocked until prerequisite proof | `verify_owner_signature()` still accepts the pre-framing owner-signature message bytes by verifying both `to_owner_message()` and `to_owner_message_legacy()` for older signed assets. |
| `crates/z00z_core/src/assets/test_asset_suite.rs` | `test_legacy_owner_signature_still_verifies` | delete in Wave 3 | Test name only; it proves the owner-signature compatibility lane owned by `asset_ownership.rs`. |
| `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | `test_proof_blob_decode_legacy_v0_upgrades_root_bind`, `LegacyProofBlobWire`, locals `legacy`, `legacy_bytes` | delete in Wave 3 | White-box fixture names only; they model the `ProofBlobV0` upgrade story but do not themselves define a live persisted surface. |
| `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | `legacy_hash`, locals `legacy_exec_id`, `legacy_draft_id`, `legacy_check_id`, `is_legacy_era` | delete in Wave 2 | Persisted checkpoint metadata still accepts the old SHA-256-derived exec, draft, and checkpoint id era alongside the current derived ids when validating RedB rows. |
| `crates/z00z_storage/src/assets/store_internal/redb_backend_helpers.rs` | local `legacy` | delete in Wave 2 | Local decode variable only; the claim-nullifier rehydrate contract is carried by the decoded row bytes, not by the identifier name. |
| `crates/z00z_storage/src/checkpoint/artifact_types.rs` | `LEGACY_OPAQUE`, `is_legacy_opaque` | blocked until prerequisite proof | Persisted checkpoint artifact rows still treat proof-system value `1` as the legacy opaque lane, and `OPAQUE` remains an alias of that value. |
| `crates/z00z_storage/src/checkpoint/codec.rs` | `LegacyArtWire`, `decode_legacy_artifact_bin`, `decode_legacy_artifact_json`, local `legacy` | delete in Wave 2 | Binary and JSON checkpoint artifact decode still fall back to the older artifact wire shape and rebuild a canonical `CheckpointArtifact::new_legacy(...)` from those bytes. |
| `crates/z00z_storage/src/checkpoint/artifact_final.rs` | `new_legacy`, `check_legacy_sys` | blocked until prerequisite proof | Canonical checkpoint artifact construction still has an explicit legacy opaque constructor and validator for artifacts that carry no snapshot/exec ids and use proof-system `LEGACY_OPAQUE`. |
| `crates/z00z_storage/src/checkpoint/link.rs` | `LegacyCheckpointLink`, local `legacy` | delete in Wave 2 | Checkpoint-link decode still accepts older link payloads without bind fields and upgrades them into canonical `CheckpointLink` values. |
| `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs` | `migrate_legacy_wallet_files`, local `legacy_backup` | blocked until prerequisite proof | Wallet persistence still recognizes older timestamp-based `z00z_wallet_*.json` and `z00z_wallet_*.bin` snapshots and preserves a reversible `*.legacy` backup suffix during migration. |
| `crates/z00z_wallets/src/services/wallet_service_tests.rs` | `legacy_v1_bytes`, `legacy_v1_restore_fails` | delete in Wave 3 | Test fixtures only; they prove backup-v1 rejection behavior owned by the wallet backup import path. |
| `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs` | `test_wallet_id_accepts_legacy_id`, `test_wallet_id_password_accepts_legacy_id` | delete in Wave 3 | RPC compatibility test names only; the live wallet-id surface is owned elsewhere. |
| `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs` | `is_legacy`, local `legacy_bytes` | blocked until prerequisite proof | Storage RPC stats, compaction, and export still treat files ending in `.legacy` as compatibility leftovers, counting and optionally deleting them when `force` or `include_deleted` rules apply. |
| `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` | `asset_send_rejects_legacy`, local `legacy` | delete in Wave 3 | Reject-fixture names only; they exercise compatibility rejection behavior but do not define a transport or persisted lane. |
| `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | `test_legacy_compact_rejected`, local `legacy` | delete in Wave 3 | Src-local reject fixture names only; no receiver-card encoded value carries `legacy` in production. |
| `crates/z00z_wallets/src/core/key/key_manager_redb_tests.rs` | `reject_legacy_kdf_params_on_wrap` | delete in Wave 3 | Test name only; it proves persisted KDF rejection owned by the seed/KDF compatibility layer. |
| `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | `test_rejects_legacy_kdf_version`, local `legacy` | delete in Wave 3 | Test-only KDF fixtures; no live DB or transport contract depends on these names. |
| `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` | `LEGACY_PROOF_STUB`, `LEGACY_SIG_STUB`, `test_legacy_proof_stub`, `test_legacy_sig_stub` | delete in Wave 3 | Claim stub fixture names only; they do not define a public proof or signature encoding contract. |
| `crates/z00z_wallets/src/core/tx/tx_verifier_tests.rs` | `legacy_digest`, locals `legacy_a`, `legacy_b` | delete in Wave 3 | Digest-framing regression helper names only; the verifier contract is carried by the hash framing logic, not by these identifiers. |
| `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | `test_import_legacy_v1_backup_is_rejected`, `test_import_rejects_legacy_payload_versions`, `build_legacy_v1_bytes` | delete in Wave 3 | Backup importer reject-fixture names only; they prove importer behavior but are not themselves the transport lane. |
| `crates/z00z_wallets/src/core/wallet/snapshot_tests.rs` | `test_rejects_legacy_snapshot_versions` | delete in Wave 3 | Snapshot reject test name only; no persisted snapshot version tag uses this identifier. |
| `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | `legacy_bech32` | delete in Wave 3 | Historical address fixture variable only; no public bech32 prefix or wire literal is named by this symbol. |
| `crates/z00z_wallets/src/db/redb_wallet_store_create.rs` | `legacy_tmp_path` | delete in Wave 2 | Local temp-path variable only; the file-handling behavior is unchanged when the legacy-named helper disappears because no persisted marker or public output string uses this identifier. |
| `crates/z00z_wallets/src/db/redb_wallet_store.rs` | `INDEX_FORMAT_VERSION_LEGACY` | blocked until prerequisite proof | Persisted RedB wallet index rows still recognize format version `1` as the old keyed Blake2b index-key encoding lane. |
| `crates/z00z_wallets/src/core/key/seed_cipher_params.rs` | `Argon2idParams::LEGACY` | blocked until prerequisite proof | Persisted encrypted seed containers still accept the 64 MiB Argon2id parameter profile as the backward-compatibility KDF lane. |
| `crates/z00z_wallets/src/core/backup/wallet_backup_container.rs` | `LegacyBackupContainer` | blocked until prerequisite proof | Backup import still has an older container byte layout with explicit version, salt, nonce, metadata, ciphertext, and checksum fields that must be decoded before deletion is allowed. |
| `crates/z00z_wallets/src/core/backup/backup_wire.rs` | `LEGACY_EXPORT_PACK_FORMAT_VERSION` | blocked until prerequisite proof | Backup metadata still recognizes export-pack format version `2` as a legacy transport lane alongside the current version. |

## Deletion Blockers

The following groups are deletion blockers until explicit proof is written and
validated:

| Group | Representative owners | Why this group is frozen by default |
| --- | --- | --- |
| Checkpoint artifact compatibility | `LegacyArtWire`, `decode_legacy_artifact_bin`, `decode_legacy_artifact_json` | decodes older artifact representations into live checkpoint models |
| Checkpoint link compatibility | `LegacyCheckpointLink` | preserves backward decode of historical link payloads |
| Wallet backup transport | `LegacyBackupContainer`, `LEGACY_EXPORT_PACK_FORMAT_VERSION`, `BackupKdfField::Legacy` | older backup payloads remain readable or rejectable under explicit rules |
| Wallet KDF compatibility | `Argon2idParams::LEGACY` | older persisted seed-container parameters remain accepted or migrated |
| Wallet DB compatibility | `INDEX_FORMAT_VERSION_LEGACY`, `legacy_bip44` | persisted DB/index or address-mode compatibility markers |
| Public compatibility feature gates | `legacy-claim-v1` | public or semi-public compatibility switches cannot be deleted blindly |

Row-exact delete dispositions for the current src-scope owners are frozen in
`Wave 0 Delete-Order Freeze For Src Scope Targets`. When this row-exact table
is more specific than a grouped blocker label, execution must follow the row-
exact disposition and blocker note.

## Wave 1 Deletion Decisions For Production Groups

Wave 1 decides deletion order per blocked production group. The meanings are
strict:

- `delete in Wave 2`: production code may be deleted in the current execution
  slice once the proof bundle passes;
- `delete in Wave 3`: residue/tests may be deleted only after the owning
  production delete is complete;
- `blocked until prerequisite proof`: deletion is mandatory but cannot land
  until the listed prerequisite is removed or disproven.

| Group | Rows covered | Fate | Proof bundle before code edits | Current execution authorization |
| --- | --- | --- | --- | --- |
| Owner-signature compatibility fallback | `crates/z00z_core/src/assets/asset_ownership.rs` | blocked until prerequisite proof | `crates/z00z_core/src/assets/test_asset_suite.rs::test_legacy_owner_signature_still_verifies`; `crates/z00z_core/tests/assets/asset_signature_domain.rs`; data path `Asset::verify_owner_signature()` proving both canonical and pre-framing owner-message bytes. Deletion is blocked until a repo-backed asset re-sign or accept/reject migration plan exists. | `036-13` is not authorized to touch this lane; `036-14` may only record the blocker. |
| Checkpoint legacy-id compatibility | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | delete in Wave 2 | `crates/z00z_storage/tests/test_redb_rehydrate.rs::test_redb_loads_legacy_claim_null_rows`; `crates/z00z_storage/tests/test_redb_rehydrate.rs::test_redb_rejects_legacy_checkpoint_link_bundle`; data path persisted exec, draft, and checkpoint metadata acceptance in `redb_backend_validate.rs`. Old SHA-256-derived bytes must stay unchanged unless the lane is removed in the same proof-backed delete. | `036-13` did not satisfy delete-first authority here. It left the accepted SHA-256-derived metadata era in place and did not delete the live lane. This row remains open and must stay truthfully legacy-tracked until actual deletion or an explicit blocker decision lands. |
| Checkpoint artifact decode compatibility | `crates/z00z_storage/src/checkpoint/codec.rs` | delete in Wave 2 | `crates/z00z_storage/tests/test_checkpoint_store_api.rs::test_legacy_opaque_proof_rejects_on_seal`; `crates/z00z_storage/tests/test_checkpoint_finalization.rs::test_legacy_opaque_finalize_rejects_live_surface`; `crates/z00z_storage/tests/test_checkpoint_finalization.rs::test_legacy_opaque_bytes_stay_legacy`; data path binary/JSON decode fallback and canonical opaque artifact rebuild through `CheckpointArtifact::new_legacy(...)`. | `036-13` did not satisfy delete-first authority here. It left the legacy decode fallback in place and did not delete the live lane. This row remains open and must stay truthfully legacy-tracked until the decode lane is actually removed or explicitly kept blocked. |
| Checkpoint artifact compatibility | `crates/z00z_storage/src/checkpoint/artifact_types.rs`; `crates/z00z_storage/src/checkpoint/artifact_final.rs` | blocked until prerequisite proof | `crates/z00z_storage/tests/test_checkpoint_store_api.rs::test_legacy_opaque_proof_rejects_on_seal`; `crates/z00z_storage/tests/test_checkpoint_finalization.rs::test_legacy_opaque_finalize_rejects_live_surface`; `crates/z00z_storage/tests/test_checkpoint_finalization.rs::test_legacy_opaque_bytes_stay_legacy`; data path proof-system value `1`, canonical opaque constructor, and finalization checks. | `artifact_types.rs` and `artifact_final.rs` stay blocked for follow-on planning because `LEGACY_OPAQUE`, `is_legacy_opaque`, `new_legacy`, `check_legacy_sys`, and `CheckpointStatement::LegacyOpaque` still span broader proof-system and test surfaces than the current safe subset. |
| Checkpoint link compatibility | `crates/z00z_storage/src/checkpoint/link.rs` | delete in Wave 2 | `crates/z00z_storage/tests/test_checkpoint_link_injective.rs::test_legacy_link_bytes_upgrade_to_bound_link`; `crates/z00z_storage/tests/test_redb_rehydrate.rs::test_redb_rejects_legacy_checkpoint_link_bundle`; data path bound-link upgrade inside link decode. Older unbound link payload bytes must stay readable unless the lane is removed in the same proof-backed delete. | `036-13` did not satisfy delete-first authority here. It left the same older-payload upgrade path in place and did not delete the live lane. This row remains open and must stay truthfully legacy-tracked until the upgrade lane is actually removed or explicitly kept blocked. |

Wave 3 storage residue status after `036-14.A`:

- The earlier blocker-only wording for this subsection was superseded by the
  2026-04-18 Wave 3 truth-repair and Wave 4 closure pass.
- The authoritative Rust closure command for this continuation is
  `rg -n "legacy|Legacy" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`.
  The older bounded pattern `\blegacy\b|Legacy` is invalid for closure because
  it misses underscore-linked identifiers.
- After the final cleanup and validation reruns, that authoritative substring
  scan returned no Rust matches outside Tari, so this continuation no longer
  carries an open Rust-code blocker inventory for the storage residue subset.

Wave 3 wallet, crypto, simulator, and utility residue status after `036-14.B/C`:

- `036-14.B/C` is now closed on real deletion instead of rename-only masking.
- The final Wave 3 cleanup removed the remaining helper-only and test-only Rust
  `legacy|Legacy` residue from the wallet, crypto, simulator, and utility
  surfaces, including the last substring hits in
  `crates/z00z_crypto/tests/test_hash_policy.rs`,
  `crates/z00z_crypto/tests/test_fail_closed.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_6_utils/exec_input_builder.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs`,
  `crates/z00z_simulator/tests/test_stage4_source_shape.rs`,
  `crates/z00z_utils/src/compression/test_compression.rs`,
  `crates/z00z_wallets/tests/test_redb_wlt_open.rs`,
  `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`,
  `crates/z00z_wallets/tests/test_phase30_split.rs`, and
  `crates/z00z_wallets/tests/test_receiver_card_record.rs`.
- After that patch, the authoritative substring scan returned no Rust matches
  outside Tari, so this continuation no longer carries an open Rust-code
  blocker inventory for the wallet, crypto, simulator, or utility residue
  subset.

| Wallet filesystem `.legacy` lane | `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs`; `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs` | blocked until prerequisite proof | Data path `WalletService::migrate_legacy_wallet_files()`, `StorageRpcImpl::compact_storage()`, `StorageRpcImpl::get_storage_stats()`, and `StorageRpcImpl::export_storage()`; current regression anchors `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl_tests.rs::test_compact_storage_force`, `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl_tests.rs::test_get_storage_stats_basic`, and `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl_tests.rs::test_export_storage_json`. Dedicated `.legacy` fixture coverage and operator-facing migration policy are still missing, so deletion is blocked. | Not authorized in the current `036-13` subset; `036-14` may only record the blocker and wait for a delete plan. |
| Wallet persisted-format compatibility markers | `crates/z00z_wallets/src/db/redb_wallet_store.rs`; `crates/z00z_wallets/src/core/key/seed_cipher_params.rs`; `crates/z00z_wallets/src/core/backup/wallet_backup_container.rs`; `crates/z00z_wallets/src/core/backup/backup_wire.rs` | blocked until prerequisite proof | `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs::index_format_migration_clears_indexes`; `crates/z00z_wallets/src/core/key/seed_cipher_metadata_tests.rs::test_all_preset_params_valid`; `crates/z00z_wallets/src/core/key/key_manager_redb_tests.rs::reject_legacy_kdf_params_on_wrap`; `crates/z00z_wallets/src/core/backup/wallet_backup_tests.rs::backup_container_roundtrip`; `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs::test_import_rejects_legacy_payload_versions`; data path `META_INDEX_FORMAT_VERSION`, seed-container metadata, backup container bytes, and export-pack version metadata. | This cluster needs follow-on planning because the DB, KDF, and backup lanes are independent wallet subsystems. |

## Delete-Only Candidate Class

An identifier may be frozen as `delete in Wave 2` only when all of the
following are true:

1. it does not define a persisted value, stored schema marker, wire value, or
   feature-gate string;
2. it does not represent a compatibility struct or decode/import lane;
3. its outward behavior is deleted rather than preserved in any non-deletion lane;
4. the deletion can be proven through local tests without any migration step.

## Removal Wave Model

### Wave 0: Inventory Freeze

- freeze every remaining production owner into one of three delete states:
  - delete in Wave 2;
  - delete in Wave 3;
  - blocked until prerequisite proof.

### Wave 1: Deletion Decisions

- for each blocked production owner, choose one explicit deletion outcome:
  - delete in Wave 2;
  - delete in Wave 3;
  - remain blocked with a named prerequisite.

### Wave 2: Production Deletion

- perform only the production deletions authorized by Wave 1;
- keep all encoded values and transport literals stable unless the
  deletion decision explicitly removes that contract.

### Wave 3: Fixture, Test, And Residue Deletion

- delete fixtures only after their owning production deletions are complete;
- keep explicit reject-coverage tests only as temporary blockers while an
  owning production row remains blocked.

### Wave 4: Validation Closure

- prove no unintended production `legacy` owners remain outside explicit blocker
  tracking or deletion.

## Proof Requirements Before Any Delete

Before a production `legacy` symbol is deleted or blocked, the phase must prove:

- what persisted or public contract the symbol currently protects;
- why that contract can now be deleted or why deletion is still blocked;
- which tests demonstrate no live compatibility regression;
- whether any specification, README, or operator-facing documentation must be
  updated in the same wave.

## Validation Requirements

The migration backlog must preserve or add targeted validation for:

- storage checkpoint decode paths;
- wallet backup import and export behavior;
- seed/KDF parameter acceptance and rejection behavior;
- wallet DB/index format handling;
- simulator and RPC flows that still refer to historical artifact lanes;
- fixture-backed reject tests that intentionally model older inputs.

## Non-Negotiable Rules

- Do not replace deletion with non-deletion cleanup.
- Do not infer safety from test-only naming if a production owner still exists.
- Do not change encoded values just because the Rust symbol name changes.
- Do not modify Tari.
- Do not reopen closed Phase 036 versioning work under the label of legacy
  removal; this is a new migration slice with its own proof burden.
