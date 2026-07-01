# Multi-Version and Renames

This file is the cleaned Phase 036 execution surface for version-suffixed symbols. It keeps only declaration-backed rows that still matter for real cleanup, hold, or follow-up decisions.

This is no longer a grouped-summary document. The old top summary table and the old remove table were removed because they covered only part of the inventory, duplicated the same decisions in two places, and forced manual reconciliation before any task could be created.

The `Raw Inventory Appendix` is now the only task-generation surface. Every implementation task must start from a raw row, not from summary prose.

## Raw-First Execution Model

Use the raw inventory directly:

- one non-test raw row = one production task candidate;
- one test-only raw row = one test review or test cleanup task candidate;
- `Current classification` tells you what the row is today;
- `Action now` tells you whether the current wave touches the row now or only later;
- `Future survivor target` tells you the simplified end-state name for the surviving lane after the execution-step condition closes;
- `Notes` tells you why the row exists and what must not be guessed.

Do not recreate a second planning layer outside the raw inventory. If a task cannot be created from a raw row plus the execution order below, the raw row must be fixed before implementation starts.

## Execution Order By Row Range

This is the execution order for the current Phase 036 planning surface. Execute rows in this order and do not reorder them without changing this file.

1. `Step 0 - Freeze keep-only and future-reserved rows`: non-test rows `1-3`, `5`, `7`, `20-23`, `25-26`, `31`, `40-42`, `57-60`.
    Implement now: no delete and no rename. If tracking artifacts are needed, create keep-only or hold-only tasks directly from the raw row. These rows are not cleanup blockers for the rest of the phase.
    Rows `27` and `61-63` are excluded here because Steps `6-7` handle them directly.
2. `Step 1 - Execute legacy wallet-KDF retirement`: non-test rows `8-17`, `24`, `28-30`.
    Implement now: keep survivor rows `9`, `11`, `13`, `15`; keep rows `8`, `10`, `12`, `14`, `16`, `17`, `24`, `28`, `29`, `30` as blocked compatibility or migration rows.
    Implement after compatibility retirement is proven, in this exact order: delete migration helpers `16-17`; delete compatibility rows `8`, `10`, `12`, `14`, `24`, `28-30`; rename survivor rows `9`, `11`, `13`, `15` to their future survivor targets.
3. `Step 2 - Execute legacy backup-KDF retirement`: non-test rows `43-45`.
    Implement now: keep survivor row `43`; keep rows `44-45` blocked as legacy compatibility lanes.
    Implement after legacy restore retirement is proven, in this exact order: delete rows `44-45`; rename survivor row `43` to its future survivor target.
4. `Step 3 - Execute legacy backup payload-import-export retirement`: non-test rows `32-38`, `50-56`.
    Implement now: keep survivor rows `34`, `35`, `52`, `53`; keep rows `32-33`, `36-38`, `50-51`, `54-56` blocked as compatibility rows.
    Implement after old backup-format retirement is proven, in this exact order: delete AAD compatibility rows `32-33`; delete decode compatibility rows `36-38`; delete verify compatibility rows `54-56`; delete legacy payload rows `50-51`; rename survivor rows `34`, `35`, `52`, `53` to their future survivor targets.
5. `Step 4 - Hold seed-container contract migration`: non-test rows `18-19`.
    Implement now: no delete and no rename. These rows stay live exactly as they are in this wave.
    Implement later: rename rows `18-19` only after an explicit persisted-format migration is approved.
6. `Step 5 - Hold receiver-card publication migration`: non-test row `39`.
    Implement now: no delete and no rename.
    Implement later: rename row `39` only after the published-record contract migration is approved.
7. `Step 6 - Hold claim-v2 protocol surface`: non-test row `27`.
    Implement now: no delete and no rename.
    Implement later: revisit only if a protocol-wide claim-v3 wave exists.
8. `Step 7 - Hold address-v2 future activation`: non-test rows `61-63`.
    Implement now: no cleanup task. These rows stay future-reserved only.
    Implement later: activate them only if the planned address migration becomes real.
9. `Step 8 - Review test-only rows after production steps`: test-only rows `1-28`.
    Implement now: do not execute test-only cleanup before its linked production surface is stable. Review test rows only after the production step they depend on has finished or has been explicitly held.
    Implement after each production step: keep compatibility-proof tests that still prove live behavior; rename or retire only the test rows that became pure suffix noise because the corresponding production row is already stable.

## Implementation Examples

Use these examples as the canonical task-translation patterns for production-current rows.
Legacy markers, future-reserved rows, migration helpers, and test residue stay in the raw inventory and execution order; they are not repeated here.

### Example 1: Current contract that stays in place

- Raw row: non-test row `31` (`DENYLIST_BLOOM_V1`)
- Classification: `production current`
- Action now: `keep`
- Future survivor target: `DENYLIST_BLOOM`
- Execution step: `Step 0`
- Task meaning: keep this current contract in place; it is not a delete candidate.

### Example 2: Current survivor that stays now and simplifies later

- Raw row: non-test row `43` (`default_v2`)
- Classification: `production current`
- Action now: `keep`
- Future survivor target: `default`
- Execution step: `Step 2`
- Task meaning: keep the row unchanged in the current wave; later, after `Step 2` conditions are satisfied, rename the surviving row to the unsuffixed target.

### Example 3: Current RPC metadata that stays in place

- Raw row: non-test row `57` (`TxStoreMetaV1`)
- Classification: `production current`
- Action now: `keep`
- Future survivor target: `TxStoreMeta`
- Execution step: `Step 0`
- Task meaning: keep this RPC metadata shape in place; do not create a delete task for it in this wave.

## Raw Inventory Appendix

This appendix is the raw working set behind the decision tables above. It keeps the original suffix-bearing signatures visible so nothing gets lost when the cleanup wave is executed.
Row IDs are stable audit IDs. If a row is removed after code-backed verification, its number is not reused.

### Non-Test Signatures

| # | Signature | Type | Path | Current classification | Action now | Future survivor target | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `HKDF_INFO_REDB_DATA_V2` | const | `crates/z00z_crypto/src/kdf_domains.rs` | future-reserved | keep | `HKDF_INFO_REDB_DATA` | exported domain marker; current wallet-open code does not select this directly |
| 2 | `HKDF_INFO_REDB_INDEX_V2` | const | `crates/z00z_crypto/src/kdf_domains.rs` | future-reserved | keep | `HKDF_INFO_REDB_INDEX` | exported domain marker; current wallet-open code does not select this directly |
| 3 | `HKDF_INFO_REDB_INTEGRITY_V2` | const | `crates/z00z_crypto/src/kdf_domains.rs` | future-reserved | keep | `HKDF_INFO_REDB_INTEGRITY` | exported domain marker; current wallet-open code does not select this directly |
| 5 | `RANGE_PROOF_BITS_V2` | const | `crates/z00z_crypto/src/crypto_constants.rs` | future-reserved | keep | `RANGE_PROOF_BITS` | live non-test range-proof backend accepts only `RANGE_PROOF_BITS`; keep only if a future proof-width migration is approved |
| 7 | `MAX_PROOF_SIZE_V2` | const | `crates/z00z_crypto/src/crypto_constants.rs` | future-reserved | keep | `MAX_PROOF_SIZE` | version-2 helper dispatch exists in validation code, but no non-test production proof lane currently selects this as the active contract |
| 8 | `HKDF_INFO_VER_V1` | const | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | compatibility lane | remove later | `HKDF_INFO` | old wallet metadata still opens through V1 |
| 9 | `HKDF_INFO_VER_V2` | const | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | production current | keep | `HKDF_INFO` | current target HKDF-info version |
| 10 | `AAD_SECRET_VER_V1` | const | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | compatibility lane | remove later | `AAD_SECRET` | old secret AAD fallback remains for migrations |
| 11 | `AAD_SECRET_VER_V2` | const | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | production current | keep | `AAD_SECRET` | current target secret AAD version |
| 12 | `VERSION_V1` | const | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | compatibility lane | remove later | `KdfParams` | old KDF record version still accepted and migrated |
| 13 | `VERSION_V2` | const | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | production current | keep | `KdfParams` | current selected KDF record version |
| 14 | `derive_wallet_keys_v1` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs` | compatibility lane | remove later | `derive_wallet_keys` | selected only for V1 wallet metadata |
| 15 | `derive_wallet_keys_v2` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs` | production current | keep | `derive_wallet_keys` | selected by current V2 wallet-key path |
| 16 | `migrate_kdf_v1_to_v2` | fn | `crates/z00z_wallets/src/core/key/key_manager_redb_wallet.rs` | migration helper | remove later | none | keep until legacy-open migration is retired |
| 17 | `migrate_kdf_v1_to_v2` | fn | `crates/z00z_wallets/src/core/key/key_manager_redb.rs` | migration helper | remove later | none | same helper, different crate path |
| 18 | `VERSION_V1` | const | `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` | production current | keep | `CipherSeedContainer` | current seed-container persisted version |
| 19 | `AAD_VERSION_V1` | const | `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` | production current | keep | `AAD_VERSION` | current and only supported seed-container AAD version |
| 20 | `OBJECT_PAYLOAD_HEADER_VERSION_V1` | const | `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs` | production current | keep | `OBJECT_PAYLOAD_HEADER_VERSION` | current encrypted object payload header version |
| 21 | `build_aad_v1` | fn | `crates/z00z_crypto/src/aead_aad.rs` | production current | keep | `build_aad` | current multipart AAD builder still funnels through this V1 framing helper |
| 22 | `TX_TIME_KEY_VERSION_V1` | const | `crates/z00z_wallets/src/db/index_codecs.rs` | production current | keep | `TX_TIME_KEY_VERSION` | current tx-time index key format |
| 23 | `SEMANTIC_KEY_VERSION_V1` | const | `crates/z00z_wallets/src/db/index_codecs.rs` | production current | keep | `SEMANTIC_KEY_VERSION` | current semantic index key format |
| 24 | `aad_secret_v1` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs` | compatibility lane | remove later | `aad_secret` | old secret AAD helper still used for migration or compatibility read |
| 25 | `META_WALLET_INTEGRITY_V1` | const | `crates/z00z_wallets/src/db/schema_keys.rs` | production current | keep | `META_WALLET_INTEGRITY` | current wallet integrity meta key |
| 26 | `META_WALLET_INTEGRITY_V1` | const | `crates/z00z_wallets/src/wasm/schema_keys.rs` | production current | keep | `META_WALLET_INTEGRITY` | same contract exposed through wasm |
| 27 | `claim_stmt_hash_v2` | fn | `crates/z00z_crypto/src/claim/v2.rs` | production current | keep | `claim_stmt_hash` | current hash helper for live claim-v2 signing surface; outer claim-v2 wire is live even though current non-test inner claim source proof version remains `ClaimProofVer::V1` |
| 28 | `redb_hkdf_info_data_v1` | fn | `crates/z00z_wallets/src/core/hashing.rs` | compatibility lane | remove later | `redb_hkdf_info_data` | legacy V1 HKDF info helper tied to V1 wallet keys |
| 29 | `redb_hkdf_info_index_v1` | fn | `crates/z00z_wallets/src/core/hashing.rs` | compatibility lane | remove later | `redb_hkdf_info_index` | legacy V1 HKDF info helper tied to V1 wallet keys |
| 30 | `redb_hkdf_info_integrity_v1` | fn | `crates/z00z_wallets/src/core/hashing.rs` | compatibility lane | remove later | `redb_hkdf_info_integrity` | legacy V1 HKDF info helper tied to V1 wallet keys |
| 31 | `DENYLIST_BLOOM_V1` | const | `crates/z00z_wallets/src/core/security/password.rs` | production current | keep | `DENYLIST_BLOOM` | current password denylist bloom payload |
| 32 | `build_aad_bytes_v1` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | compatibility lane | remove later | `build_aad_bytes` | legacy backup AAD framing accepted by importer/verifier |
| 33 | `build_aad_bytes_v2` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | compatibility lane | remove later | `build_aad_bytes` | intermediate backup AAD framing used for compatibility |
| 34 | `build_aad_bytes_v3` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | production current | keep | `build_aad_bytes` | current exporter path builds V3 AAD first |
| 35 | `decode_export_pack_v4` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | production current | keep | `decode_export_pack` | current backup importer prefers newest payload version |
| 36 | `decode_export_pack_v3` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | compatibility lane | remove later | `decode_export_pack` | older export-pack payload still supported |
| 37 | `decode_export_pack_v2` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | compatibility lane | remove later | `decode_export_pack` | older export-pack payload still supported |
| 38 | `decode_legacy_payload_v1` | fn | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | compatibility lane | remove later | `decode_legacy_payload` | legacy payload remains readable for compatibility import |
| 39 | `ReceiverCardRecordV1` | struct | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | production current | keep | `ReceiverCardRecord` | current receiver-card publication contract |
| 40 | `CLAIM_PROOF_V2` | const | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | production current | keep | `CLAIM_PROOF` | current canonical outer proof-type tag for claim tx; do not read this alone as proof that the inner `ClaimSourceProof` version lane is already V2 |
| 41 | `CLAIM_SCHEMA_V1` | const | `crates/z00z_wallets/src/core/claim/claim_receipt.rs` | production current | keep | `CLAIM_SCHEMA` | current claim receipt schema |
| 42 | `FEE_WGT_VER_V1` | const | `crates/z00z_wallets/src/core/tx/fee_estimator.rs` | production current | keep | `FEE_WGT_VER` | current fee weight model tag |
| 43 | `default_v2` | fn | `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs` | production current | keep | `default` | current backup KDF constructor; rename only after Step 2 compatibility retirement closes |
| 44 | `legacy_v1` | fn | `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs` | compatibility lane | remove later | none | explicit legacy backup KDF contract |
| 45 | `derive_key_legacy_v1` | fn | `crates/z00z_wallets/src/core/backup/wallet_backup.rs` | compatibility lane | remove later | `derive_key` | old backup key derivation lane only |
| 50 | `BackupPayloadV1` | struct | `crates/z00z_wallets/src/core/backup/backup_wire.rs` | compatibility lane | remove later | `BackupPayload` | legacy backup payload |
| 51 | `BackupPayloadV3` | struct | `crates/z00z_wallets/src/core/backup/backup_wire.rs` | compatibility lane | remove later | `BackupPayload` | intermediate backup payload |
| 52 | `BackupPayloadV4` | struct | `crates/z00z_wallets/src/core/backup/backup_wire.rs` | production current | keep | `BackupPayload` | current backup payload |
| 53 | `verify_export_pack_v4` | fn | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | production current | keep | `verify_export_pack` | current backup verification lane |
| 54 | `verify_export_pack_v3` | fn | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | compatibility lane | remove later | `verify_export_pack` | older export-pack verify lane |
| 55 | `verify_export_pack_v2` | fn | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | compatibility lane | remove later | `verify_export_pack` | older export-pack verify lane |
| 56 | `verify_legacy_payload_v1` | fn | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | compatibility lane | remove later | `verify_legacy_payload` | legacy payload verify lane |
| 57 | `TxStoreMetaV1` | struct | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` | production current | keep | `TxStoreMeta` | current RPC storage metadata |
| 58 | `export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | production current | keep | `export_public_material` | current RPC surface |
| 59 | `wallet.key.export_public_material_v2` | RPC string | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | production current | keep | `wallet.key.export_public_material` | current RPC method string |
| 60 | `export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs` | production current | keep | `export_public_material` | server-side implementation of the current RPC surface |
| 61 | `encode_single_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | future-reserved | keep | `encode_single` | not yet active, reserved for a later migration |
| 62 | `encode_dual_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | future-reserved | keep | `encode_dual` | not yet active, reserved for a later migration |
| 63 | `decode_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | future-reserved | keep | `decode` | not yet active, reserved for a later migration |

### Test-Only Signatures

| # | Signature | Type | Path | Current classification | Action now | Future survivor target | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `test_value_endian_v1` | fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue | keep or rename to `test_value_endian` | `test_value_endian` | test-only suffix noise |
| 2 | `test_offsets_v1` | fn | `crates/z00z_core/src/assets/leaf_tests.rs` | test residue | keep or rename to `test_offsets` | `test_offsets` | test-only suffix noise |
| 3 | `test_rejects_seed_genesis_v3` | fn | `crates/z00z_core/tests/genesis/test_config.rs` | test residue | keep | same | test-only versioned case |
| 4 | `test_rejects_seed_genesis_v2` | fn | `crates/z00z_core/tests/genesis/test_config.rs` | test residue | keep | same | test-only versioned case |
| 5 | `test_open_migrates_kdf_v1` | fn | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | test residue | keep | same | migration behavior test |
| 6 | `test_proof_size_validation_v1` | fn | `crates/z00z_crypto/src/types_tests.rs` | test residue | keep | same | test-only versioned case |
| 7 | `test_proof_size_validation_v2` | fn | `crates/z00z_crypto/src/types_tests.rs` | test residue | keep | same | test-only versioned case |
| 8 | `kdf_migration_noops_non_v1` | fn | `crates/z00z_wallets/src/core/key/key_manager_redb_tests.rs` | test residue | keep | same | negative migration test |
| 9 | `test_asset_cross_mixed_v2` | fn | `crates/z00z_core/tests/assets/test_integration_assets_test24.rs` | test residue | keep | same | test-only versioned case |
| 10 | `rewrite_v1` | fn | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` | test residue | keep | same | test-only versioned case |
| 11 | `test_chain_vector_v1` | fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | test residue | keep | same | test-only versioned case |
| 12 | `test_sender_fix_v1` | fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | test residue | keep | same | test-only versioned case |
| 13 | `test_receiver_fix_v1` | fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | test residue | keep | same | test-only versioned case |
| 14 | `test_api_parity_v1` | fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | test residue | keep | same | test-only versioned case |
| 15 | `test_domain_mismatch_v1` | fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | test residue | keep | same | test-only versioned case |
| 16 | `test_migration_vec_v1` | fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs` | test residue | keep | same | test-only versioned case |
| 17 | `test_rejects_params_untrusted_v2` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | test residue | keep | same | test-only versioned case |
| 18 | `test_rejects_params_untrusted_v3` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | test residue | keep | same | test-only versioned case |
| 19 | `test_rejects_params_untrusted_v4` | fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | test residue | keep | same | test-only versioned case |
| 20 | `test_rejects_wlt_open_v2` | fn | `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs` | test residue with misleading V2 noise | rename | `test_rejects_wlt_open_invalid_save_seq` | the test mutates `META_WALLET_SAVE_SEQ`, so the current name overstates version semantics |
| 21 | `test_golden_yaml_v1` | fn | `crates/z00z_wallets/src/core/stealth/facade_zkpack_tests.rs` | test residue | keep | same | test-only versioned case |
| 22 | `test_key_golden_v1` | fn | `crates/z00z_wallets/src/core/stealth/facade_kdf.rs` | test residue | keep | same | test-only versioned case |
| 23 | `test_nonce_golden_v1` | fn | `crates/z00z_wallets/src/core/stealth/facade_kdf.rs` | test residue | keep | same | test-only versioned case |
| 24 | `test_prove_claim_v1` | fn | `crates/z00z_crypto/src/claim/prover.rs` | test residue | keep | same | test-only versioned case |
| 25 | `test_weight_fixture_v1` | fn | `crates/z00z_wallets/src/core/tx/fee_estimator_tests.rs` | test residue | keep | same | test-only versioned case |
| 26 | `test_export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | test residue | keep | same | test-only versioned case |
| 27 | `test_encode_single_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | test residue | keep | same | test-only versioned case |
| 28 | `test_encode_dual_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | test residue | keep | same | test-only versioned case |

## Table-to-Task Translation Protocol

Use this file as the source of truth when turning Phase 036 inventory rows into execution tasks. Start from the raw row, then apply the execution step above if the row belongs to an ordered cleanup block.

### Raw Inventory Appendix -> Non-Test Signatures

This is the mandatory production execution register. Every production symbol that may be kept, renamed, or removed must be tracked row by row here.

| Column | Task meaning | Required translation rule |
| --- | --- | --- |
| `#` | raw signature row ID | Keep this as the per-signature subtask ID. Do not merge duplicate names from different paths. |
| `Signature` | exact symbol under review | Use as the production subtask title suffix. |
| `Type` | symbol kind | Record whether the subtask touches a `const`, `fn`, `struct`, `RPC string`, or other declared kind. |
| `Path` | exact file boundary | One raw row equals one path-specific execution item. Same signature in two files means two subtasks. |
| `Current classification` | present-state contract | Translate into task state such as `production current`, `compatibility lane`, `migration helper`, `future-reserved`, or `test residue` if ever misfiled. |
| `Action now` | current-wave action | `keep` means the current wave leaves the row in place; `remove later` means the row is a blocked cleanup candidate after its execution-step condition closes. |
| `Future survivor target` | survivor end-state label | This is the simplified name for the surviving lane after the execution-step condition closes. It is not automatically the rename target of the current row. |
| `Notes` | per-signature constraint | Copy into the subtask evidence/constraint field. |

Required rule for production execution:

1. Create one production subtask for every non-test raw row that belongs to the active execution step.
2. Never collapse multiple raw rows into one subtask only because the signature text is identical.
3. If `Current classification = production current` and `Action now = keep`, create a keep-proof subtask, not a rename subtask.
4. If `Current classification = compatibility lane` and `Action now = remove later`, create a blocked delete candidate subtask for the current row.
5. If `Current classification = migration helper` and `Action now = remove later`, create a blocked later-delete subtask. Do not rename the helper unless the raw row says so explicitly.
6. Use `Future survivor target` only to describe the simplified survivor end-state after the execution-step condition closes.
7. If the raw row is `future-reserved`, keep it in the inventory and mark it as no-action-in-this-wave.

Every non-test raw subtask must record:

- raw row ID;
- exact signature;
- exact type;
- exact path;
- current classification;
- action now;
- future survivor target or `none`;
- notes;
- linked execution step label or `none`.

### Raw Inventory Appendix -> Test-Only Signatures

This is the mandatory test execution register. It exists to prevent production cleanup from silently breaking or drifting test coverage.

| Column | Task meaning | Required translation rule |
| --- | --- | --- |
| `#` | raw test row ID | Keep as the per-test subtask ID. |
| `Signature` | exact test symbol | Use as the test subtask title suffix. |
| `Type` | symbol kind | Preserve exact kind, including `async fn` when present. |
| `Path` | exact file boundary | One row equals one test-only subtask. |
| `Current classification` | test status | Usually `test residue`; keep the literal value. |
| `Action now` | test action | Copy literally, usually `keep` or `keep or rename to ...`. |
| `Future survivor target` | future test rename target | Use only for test-track work; never let it drive production cleanup. |
| `Notes` | test rationale | Preserve the reason the row exists, such as versioned fixture coverage or migration proof. |

Required rule for test execution:

1. Keep test-only rows in a separate execution track from production rows.
2. Do not close an execution step if production renames have invalidated test-only rows and those rows were not reviewed.
3. If a test row exists to prove compatibility or migration behavior, it must stay open until the corresponding production compatibility lane is retired.
4. If a test row is only suffix noise and no longer proves a real contract, it may be scheduled as a later test-cleanup subtask, but only after production work is stable.

### Pitfall Guards

Do not violate these rules when creating tasks:

1. Do not recreate a summary table outside this file.
2. Do not try to match summary rows and raw rows by shared row number. The raw row ID is only a raw row ID.
3. Do not read `Future survivor target` as "rename this exact row now" unless the row is the surviving production lane and the execution-step condition is already satisfied.
4. Do not mix production cleanup tasks and test-only review tasks into one ticket.
5. Do not close an execution step while any raw row inside its declared row range is still unresolved.
6. Do not create tasks from prose examples alone; every task must cite the raw row ID it implements.

### Execution Step Completion Rules

Do not mark an execution step complete unless all of the following are true:

1. The execution order above still matches the raw rows it covers.
2. The step-specific hold, delete, or rename condition is satisfied or explicitly unchanged.
3. Every affected non-test raw row has its own resolved status: `kept`, `blocked`, `renamed`, `removed`, or `deferred with reason`.
4. Every affected test-only raw row has its own resolved status: `kept`, `renamed`, `deferred`, or `retired with proof`.
5. No signature-level change exists outside the raw inventory.

### Non-Negotiable Translation Rules

1. The raw inventory is the only task-generation surface.
2. Real execution must be driven at raw-row granularity.
3. Execution-step logic must come from the execution order by row range, not from recreated summary tables.
4. No signature change is complete until its own raw row is closed explicitly.
5. No execution step is complete until the raw rows prove that the intended cleanup or hold decision was actually implemented.
<!-- Table translation protocol ends here. -->
