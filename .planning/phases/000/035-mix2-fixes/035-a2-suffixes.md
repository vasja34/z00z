# 035 Suffixes v1..vn Inventory And Cleanup Guidance

**Status:** Workspace-backed inventory and interpretation  
**Date:** 2026-04-11  
**Scope:** Rust signature definitions and workspace filenames carrying `_vN` or `Vn` suffixes

## 🎯 Objective

Collect a source-backed inventory of all `v1..vn` and `V1..Vn` suffix-bearing
surfaces found in:

1. Rust signature definitions in the workspace;
2. workspace filenames.

This note is broader and more mechanical than
`034-suffixes-V1-Vn.md`, but it is no longer inventory-only. It now combines:

1. the syntax-first suffix inventory for Phase 035;
2. a workspace-backed production-current vs compatibility interpretation table;
3. the merged versioning-cleanup guidance that was previously tracked as a
  separate Phase 035 note.

Use this file as the canonical Phase 035 source when deciding whether a
versioned surface is:

- the current production head;
- a still-shipped compatibility lane;
- a symbol-only review candidate;
- or stale inventory drift that should be corrected.

For Phase 035 execution, this file is the sole authority for suffix inventory,
production-head interpretation, cleanup guidance, and curated rename handoff.
`034-suffixes-V1-Vn.md` remains historical comparison context only and must
not be reused as an execution authority surface.

## 📌 Collection Rules

The inventory below is intentionally syntax-first.

1. Signature surfaces were collected from Rust definition lines only:
   `const`, `static`, `fn`, `struct`, `enum`, `type`, `trait`, and explicit RPC
   method signatures.
2. Filename surfaces were collected from non-hidden repo paths returned by the
    default `rg --files .` view.
3. Usage-only matches were intentionally excluded from the main inventory.
4. Local variables, temporary values, and grep noise such as `kdf_v1`,
   `container_v1`, `max_v2`, or `new_v4` are not treated as signature surfaces.
5. Comment-only or prose-only mentions are not treated as signature surfaces.
6. Hidden paths and worktree metadata such as `.git/`, `.github/`, `.planning/`,
   and `.temp/` are outside the filename inventory unless called out explicitly.
7. Every primary signature row must remain declaration-backed and path-backed.
  Repeated names such as `VERSION_V1` and `VERSION_V2` belong to their
  owning type or module path and must not be merged by bare symbol name alone.
8. Corrected rows that already moved to unsuffixed live code stay isolated in
  `Stale Or Corrected Inventory Rows` and are not part of the primary
  execution authority or default rename scope.

## ✅ Non-Test Signature Surfaces

- `crates/z00z_crypto/src/kdf_domains.rs`
  - `HKDF_INFO_REDB_DATA_V2` (`const`)
  - `HKDF_INFO_REDB_INDEX_V2` (`const`)
  - `HKDF_INFO_REDB_INTEGRITY_V2` (`const`)
- `crates/z00z_crypto/src/crypto_constants.rs`
  - `RANGE_PROOF_BITS_V1` (`const`)
  - `RANGE_PROOF_BITS_V2` (`const`)
  - `MAX_PROOF_SIZE_V1` (`const`)
  - `MAX_PROOF_SIZE_V2` (`const`)
- `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`
  - `HKDF_INFO_VER_V1` (`const`)
  - `HKDF_INFO_VER_V2` (`const`)
  - `AAD_SECRET_VER_V1` (`const`)
  - `AAD_SECRET_VER_V2` (`const`)
  - `VERSION_V1` (`const`)
  - `VERSION_V2` (`const`)
- `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs`
  - `derive_wallet_keys_v1` (`fn`)
  - `derive_wallet_keys_v2` (`fn`)
- `crates/z00z_wallets/src/core/key/key_manager_redb_wallet.rs`
  - `migrate_kdf_v1_to_v2` (`fn`)
- `crates/z00z_wallets/src/core/key/key_manager_redb.rs`
  - `migrate_kdf_v1_to_v2` (`fn`)
- `crates/z00z_wallets/src/core/key/seed_cipher_container.rs`
  - `VERSION_V1` (`const`)
  - `AAD_VERSION_V1` (`const`)
- `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs`
  - `OBJECT_PAYLOAD_HEADER_VERSION_V1` (`const`)
- `crates/z00z_crypto/src/aead_aad.rs`
  - `build_aad_v1` (`fn`)
- `crates/z00z_wallets/src/db/index_codecs.rs`
  - `TX_TIME_KEY_VERSION_V1` (`const`)
  - `SEMANTIC_KEY_VERSION_V1` (`const`)
- `crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs`
  - `aad_secret_v1` (`fn`)
- `crates/z00z_wallets/src/db/schema_keys.rs`
  - `META_WALLET_INTEGRITY_V1` (`const`)
- `crates/z00z_wallets/src/wasm/schema_keys.rs`
  - `META_WALLET_INTEGRITY_V1` (`const`)
- `crates/z00z_crypto/src/claim/v2.rs`
  - `claim_stmt_hash_v2` (`fn`)
- `crates/z00z_wallets/src/core/hashing.rs`
  - `redb_hkdf_info_data_v1` (`fn`)
  - `redb_hkdf_info_index_v1` (`fn`)
  - `redb_hkdf_info_integrity_v1` (`fn`)
- `crates/z00z_wallets/src/core/security/password.rs`
  - `DENYLIST_BLOOM_V1` (`const`)
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs`
  - `build_aad_bytes_v1` (`fn`)
  - `build_aad_bytes_v2` (`fn`)
  - `build_aad_bytes_v3` (`fn`)
  - `decode_export_pack_v4` (`fn`)
  - `decode_export_pack_v3` (`fn`)
  - `decode_export_pack_v2` (`fn`)
  - `decode_legacy_payload_v1` (`fn`)
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs`
  - `build_aad_bytes_v1` (`fn`)
  - `build_aad_bytes_v2` (`fn`)
  - `build_aad_bytes_v3` (`fn`)
- `crates/z00z_wallets/src/core/chain/receiver_card_record.rs`
  - `ReceiverCardRecordV1` (`struct`)
- `crates/z00z_wallets/src/core/tx/claim_wire_types.rs`
  - `CLAIM_PROOF_V2` (`const`)
- `crates/z00z_wallets/src/core/claim/claim_receipt.rs`
  - `CLAIM_SCHEMA_V1` (`const`)
- `crates/z00z_wallets/src/core/tx/fee_estimator.rs`
  - `FEE_WGT_VER_V1` (`const`)
- `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs`
  - `default_v2` (`fn`)
  - `legacy_v1` (`fn`)
- `crates/z00z_wallets/src/core/backup/backup_wire.rs`
  - `BackupEncryptionV1` (`struct`)
  - `BackupCompressionV1` (`struct`)
  - `BackupAssociatedDataV1` (`struct`)
  - `BackupContainerV1` (`struct`)
  - `BackupPayloadV1` (`struct`)
  - `BackupPayloadV3` (`struct`)
  - `BackupPayloadV4` (`struct`)
- `crates/z00z_wallets/src/core/backup/wallet_backup.rs`
  - `derive_key_legacy_v1` (`fn`)
- `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs`
  - `verify_export_pack_v4` (`fn`)
  - `verify_export_pack_v3` (`fn`)
  - `verify_export_pack_v2` (`fn`)
  - `verify_legacy_payload_v1` (`fn`)
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`
  - `TxStoreMetaV1` (`struct`)
- `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`
  - `export_public_material_v2` (`async fn`)
  - RPC method string surface: `wallet.key.export_public_material_v2`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs`
  - `export_public_material_v2` (`async fn`)
- `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`
  - `encode_single_v2` (`fn`)
  - `encode_dual_v2` (`fn`)
  - `decode_v2` (`fn`)

## 🧪 Test-Only Signature Surfaces

- `crates/z00z_core/src/assets/leaf_tests.rs`
  - `test_value_endian_v1` (`fn`)
  - `test_offsets_v1` (`fn`)
- `crates/z00z_core/tests/genesis/test_config.rs`
  - `test_rejects_seed_genesis_v3` (`fn`)
  - `test_rejects_seed_genesis_v2` (`fn`)
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
  - `test_open_migrates_kdf_v1` (`fn`)
- `crates/z00z_crypto/src/types_tests.rs`
  - `test_proof_size_validation_v1` (`fn`)
  - `test_proof_size_validation_v2` (`fn`)
- `crates/z00z_wallets/src/core/key/key_manager_redb_tests.rs`
  - `kdf_migration_noops_non_v1` (`fn`)
- `crates/z00z_core/tests/assets/test_integration_assets_test24.rs`
  - `test_asset_cross_mixed_v2` (`fn`)
- `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`
  - `rewrite_v1` (`fn`)
- `crates/z00z_wallets/tests/test_phase11_derivation.rs`
  - `test_chain_vector_v1` (`fn`)
  - `test_sender_fix_v1` (`fn`)
  - `test_receiver_fix_v1` (`fn`)
  - `test_api_parity_v1` (`fn`)
  - `test_domain_mismatch_v1` (`fn`)
  - `test_migration_vec_v1` (`fn`)
- `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs`
  - `test_rejects_params_untrusted_v2` (`fn`)
  - `test_rejects_params_untrusted_v3` (`fn`)
  - `test_rejects_params_untrusted_v4` (`fn`)
- `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs`
  - `test_rejects_wlt_open_v2` (`fn`)
- `crates/z00z_wallets/src/core/stealth/facade_zkpack_tests.rs`
  - `test_golden_yaml_v1` (`fn`)
- `crates/z00z_wallets/src/core/stealth/facade_kdf.rs`
  - `test_key_golden_v1` (`fn`)
  - `test_nonce_golden_v1` (`fn`)
- `crates/z00z_crypto/src/claim/prover.rs`
  - `test_prove_claim_v1` (`fn`)
- `crates/z00z_wallets/src/core/tx/fee_estimator_tests.rs`
  - `test_weight_fixture_v1` (`fn`)
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs`
  - `test_export_public_material_v2` (`async fn`)
- `crates/z00z_wallets/src/core/address/z00z_address/tests.rs`
  - `test_encode_single_v2` (`fn`)
  - `test_encode_dual_v2` (`fn`)

## Fixed Table

The table below gives a production-oriented classification for Rust-facing
production or API surfaces using the same status scale as
`034-suffixes-V1-Vn.md`:

- `production-current` = the surface is on the current non-test production path
- ==reserved-future== = the surface remains live compatibility,
  read/import/migration support, or another versioned lane that the current
  production path does not select by default

Test-only surfaces from the section above are excluded from this table because
they are not production contracts by themselves and inherit status from the
corresponding runtime surfaces.

| Surface | Type | Status | Short rationale | Paths | Phase 035 | Comments |
| --- | --- | --- | --- | --- | --- | --- |
| `HKDF_INFO_REDB_DATA_V2` | HKDF info constant | ==reserved-future== | exported V2 domain constant exists, but current wallet-open path selects via version gate rather than this constant directly | `crates/z00z_crypto/src/kdf_domains.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `HKDF_INFO_REDB_INDEX_V2` | HKDF info constant | ==reserved-future== | exported V2 domain constant exists, but current wallet-open path selects via version gate rather than this constant directly | `crates/z00z_crypto/src/kdf_domains.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `HKDF_INFO_REDB_INTEGRITY_V2` | HKDF info constant | ==reserved-future== | exported V2 domain constant exists, but current wallet-open path selects via version gate rather than this constant directly | `crates/z00z_crypto/src/kdf_domains.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `RANGE_PROOF_BITS_V1` | proof-width constant | `production-current` | current backend init and range-proof path enforce V1 width | `crates/z00z_crypto/src/crypto_constants.rs`; `backend_init.rs`; `backend_range_proofs.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `RANGE_PROOF_BITS_V2` | proof-width constant | ==reserved-future== | defined and exported, but current backend accepts only V1 bits | `crates/z00z_crypto/src/crypto_constants.rs`; `backend_range_proofs.rs`; `types.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `MAX_PROOF_SIZE_V1` | proof size limit constant | `production-current` | live single and batch verify paths enforce it | `crates/z00z_crypto/src/crypto_constants.rs`; `backend_range_proofs.rs`; `backend_batch.rs`; `types_validation.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `MAX_PROOF_SIZE_V2` | proof size limit constant | ==reserved-future== | generic V2 bound exists, but current backend path does not select it as the active runtime contract | `crates/z00z_crypto/src/crypto_constants.rs`; `types_validation.rs`; `types.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `HKDF_INFO_VER_V1` | version selector constant | ==reserved-future== | current code still opens or migrates old HKDF-info wallets through V1 | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`; `redb_wallet_store_migrations.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `HKDF_INFO_VER_V2` | version selector constant | `production-current` | current target HKDF-info version | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `AAD_SECRET_VER_V1` | version selector constant | ==reserved-future== | old secret AAD fallback still used for open or migration compatibility | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`; `redb_wallet_store_migrations.rs`; `redb_wallet_store_crypto_ops.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `AAD_SECRET_VER_V2` | version selector constant | `production-current` | current target secret AAD version | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `KdfParams::VERSION_V1` | version constant | ==reserved-future== | old KDF record version still accepted and migrated | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`; `crates/z00z_wallets/src/core/key/key_manager_redb.rs`; `redb_wallet_store_migrations.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `KdfParams::VERSION_V2` | version constant | `production-current` | current selected KDF record version | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`; `crates/z00z_wallets/src/core/key/key_manager_redb.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `derive_wallet_keys_v1` | helper fn | ==reserved-future== | selected only when wallet metadata says `HKDF_INFO_VER_V1` | `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs`; `redb_wallet_crypto.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `derive_wallet_keys_v2` | helper fn | `production-current` | selected by current V2 wallet-key path | `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs`; `redb_wallet_crypto.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `migrate_kdf_v1_to_v2` | migration helper fn | ==reserved-future== | current production keeps it only for legacy-open migration support | `crates/z00z_wallets/src/core/key/key_manager_redb.rs`; `key_manager_redb_wallet.rs`; `redb_wallet_store_migrations.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `CipherSeedContainer::VERSION_V1` | version constant | `production-current` | current seed-container persisted version | `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `AAD_VERSION_V1` | version constant | `production-current` | current and only supported seed-container AAD version | `crates/z00z_wallets/src/core/key/seed_cipher_container.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `OBJECT_PAYLOAD_HEADER_VERSION_V1` | version constant | `production-current` | current encrypted object payload header version | `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `build_aad_v1` | helper fn | `production-current` | current multipart AAD builders still funnel through this V1 framing helper | `crates/z00z_crypto/src/aead_aad.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `TX_TIME_KEY_VERSION_V1` | index-key version constant | `production-current` | current tx-time index key format | `crates/z00z_wallets/src/db/index_codecs.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `SEMANTIC_KEY_VERSION_V1` | index-key version constant | `production-current` | current semantic index key format | `crates/z00z_wallets/src/db/index_codecs.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `aad_secret_v1` | helper fn | ==reserved-future== | old secret AAD helper is still used for compatibility read or migration lanes | `crates/z00z_wallets/src/db/redb_wallet_crypto_aad.rs`; `redb_wallet_store_migrations.rs`; `redb_wallet_store_crypto_ops.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `META_WALLET_INTEGRITY_V1` | schema/meta key constant | `production-current` | current wallet integrity meta key | `crates/z00z_wallets/src/db/schema_keys.rs`; `crates/z00z_wallets/src/wasm/schema_keys.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `claim_stmt_hash_v2` | helper fn | `production-current` | current hash helper for the live claim-v2 signing surface | `crates/z00z_crypto/src/claim/v2.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `redb_hkdf_info_data_v1` | helper fn | ==reserved-future== | legacy V1 HKDF info helper remains only because `derive_wallet_keys_v1` still supports old wallets | `crates/z00z_wallets/src/core/hashing.rs`; `redb_wallet_crypto_kdf_helpers.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `redb_hkdf_info_index_v1` | helper fn | ==reserved-future== | legacy V1 HKDF info helper remains only because `derive_wallet_keys_v1` still supports old wallets | `crates/z00z_wallets/src/core/hashing.rs`; `redb_wallet_crypto_kdf_helpers.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `redb_hkdf_info_integrity_v1` | helper fn | ==reserved-future== | legacy V1 HKDF info helper remains only because `derive_wallet_keys_v1` still supports old wallets | `crates/z00z_wallets/src/core/hashing.rs`; `redb_wallet_crypto_kdf_helpers.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `DENYLIST_BLOOM_V1` | embedded data constant | `production-current` | current password denylist bloom payload | `crates/z00z_wallets/src/core/security/password.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `build_aad_bytes_v1` | helper fn | ==reserved-future== | legacy backup AAD framing is still accepted by current importer or verifier compatibility path | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs`; `backup_exporter_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `build_aad_bytes_v2` | helper fn | ==reserved-future== | intermediate backup AAD framing still exists for compatibility verification and import | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs`; `backup_exporter_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `build_aad_bytes_v3` | helper fn | `production-current` | current exporter path builds V3 AAD, and current importer or verifier tries it first | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs`; `backup_exporter_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `decode_export_pack_v4` | helper fn | `production-current` | current backup importer prefers newest payload version V4 | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `decode_export_pack_v3` | helper fn | ==reserved-future== | older export-pack payload still supported by current importer | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `decode_export_pack_v2` | helper fn | ==reserved-future== | older export-pack payload still supported by current importer | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `decode_legacy_payload_v1` | helper fn | ==reserved-future== | legacy payload remains readable for compatibility import | `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `ReceiverCardRecordV1` | struct name | `production-current` | current receiver-card publication contract | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `CLAIM_PROOF_V2` | field/tag constant | `production-current` | current canonical proof type tag for claim tx | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs`; `claim_tx.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `CLAIM_SCHEMA_V1` | schema/version constant | `production-current` | current claim receipt schema | `crates/z00z_wallets/src/core/claim/claim_receipt.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `FEE_WGT_VER_V1` | version tag constant | `production-current` | current fee weight model tag | `crates/z00z_wallets/src/core/tx/fee_estimator.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `default_v2` | constructor helper fn | `production-current` | canonical new backup KDF contract for fresh backups | `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs`; `wallet_backup.rs`; `backup_exporter_impl.rs` | ⛔️ | Only approved rename candidate; no later Phase 035 summary proves `default` landed. |
| `legacy_v1` | constructor helper fn | ==reserved-future== | explicit legacy backup KDF contract kept for compatibility import and restore | `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs`; `wallet_backup.rs`; `backup_importer_impl.rs`; `backup_exporter_impl.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `BackupEncryptionV1` | struct name | `production-current` | current backup encryption wire struct | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_exporter_impl.rs`; `backup_importer_impl.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `BackupCompressionV1` | struct name | `production-current` | current backup compression wire struct | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_exporter_impl.rs`; `backup_importer_impl.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `BackupAssociatedDataV1` | struct name | `production-current` | current backup AAD struct used by exporter, importer, and verify | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_exporter_impl.rs`; `backup_importer_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `BackupContainerV1` | struct name | `production-current` | current backup container type in exporter, importer, and verify | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_exporter_impl.rs`; `backup_importer_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `BackupPayloadV1` | struct name | ==reserved-future== | legacy backup payload still accepted by current importer or verify code | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_importer_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `BackupPayloadV3` | struct name | ==reserved-future== | older export-pack payload still supported by current importer or verify code | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_importer_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `BackupPayloadV4` | struct name | `production-current` | current newest backup payload | `crates/z00z_wallets/src/core/backup/backup_wire.rs`; `backup_exporter_impl.rs`; `backup_importer_impl.rs`; `backup_exporter_verify.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `derive_key_legacy_v1` | helper fn | ==reserved-future== | explicit legacy backup-key derivation remains only for compatibility restore lanes | `crates/z00z_wallets/src/core/backup/wallet_backup.rs`; `backup_importer_tests.rs`; `test_backup_kdf_contract.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `verify_export_pack_v4` | verify helper fn | `production-current` | verify path checks current V4 export pack | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `verify_export_pack_v3` | verify helper fn | ==reserved-future== | verify path still supports older V3 export pack | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `verify_export_pack_v2` | verify helper fn | ==reserved-future== | verify path still supports older V2 export pack | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `verify_legacy_payload_v1` | verify helper fn | ==reserved-future== | verify path still supports legacy V1 payloads | `crates/z00z_wallets/src/core/backup/backup_exporter_verify.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `TxStoreMetaV1` | struct name | `production-current` | current RPC tx storage metadata format | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `export_public_material_v2` | RPC method fn | `production-current` | current live RPC export contract; unsuffixed lane is not registered as a parallel public method | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`; `key_impl/server.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `wallet.key.export_public_material_v2` | RPC method string | `production-current` | explicit registered method string for the live export lane | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`; `wallet_dispatcher_wiring_register.rs` | ✅ | Disposition fixed: current live contract or frozen boundary; no additional Phase 035 rename required. |
| `encode_single_v2` | helper fn | ==reserved-future== | future enhancement helper; file marks it not yet active and keeps it under `#[allow(dead_code)]` | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`; `tests.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `encode_dual_v2` | helper fn | ==reserved-future== | future enhancement helper; file marks it not yet active and keeps it under `#[allow(dead_code)]` | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`; `tests.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |
| `decode_v2` | helper fn | ==reserved-future== | future enhancement helper; file marks it not yet active and keeps it under `#[allow(dead_code)]` | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`; `tests.rs` | ✅ | Disposition fixed: keep as compatibility or review lane; no active rename in closed Phase 035 waves. |

## 🔀 Cleanup Interpretation And Production-Head Guidance

This section is the canonical home for the version-family cleanup guidance.

### Intent

Remove version-suffixed names and stale compatibility clutter by selecting one
canonical production version per versioned family and deleting the rest only
after compatibility retirement is explicitly approved. The goal is to make the
code easier to read by retiring obsolete branches, dead forks, and suffix-only
noise once the real surviving production path is identified.

Version markers are not valuable by themselves. They become harmful when they
freeze old logic, preserve stale forks, and make transitional branches look
like permanent architecture.

### Guardrails

1. Do not decide by suffix alone.
2. `production-current` means the repository shows that this is the current
  write, runtime-default, exporter-default, or public-contract head.
3. `==reserved-future==` often means the code still ships a compatibility,
  import, migration, decode, verify, or feature-gated legacy lane.
4. A current-head decision is not the same thing as delete authorization.
5. For wallet, backup, and claim families, legacy lanes must survive until a
  separate compatibility-retirement decision says otherwise.
6. Single surviving `V1` symbols must be reviewed as naming or contract
  boundaries, not auto-collapsed mechanically.

### Production-Choice Reminder

Treat the following examples as version families that require an explicit
canonical-production decision, not as automatic keep-sets or delete-now sets:

- `HKDF_INFO_VER_V1` / `HKDF_INFO_VER_V2`
- `AAD_SECRET_VER_V1` / `AAD_SECRET_VER_V2`
- `ClaimV2Err`
- `AssetPackVersion::V1Basic` / `AssetPackVersion::V2Memo`
- `derive_wallet_keys_v1` / `derive_wallet_keys_v2`
- `aad_secret_v1`
- `redb_hkdf_info_data_v1`
- `redb_hkdf_info_index_v1`
- `redb_hkdf_info_integrity_v1`

Backup import and export version families must be judged by the same rule:
identify the one version that should remain the production contract, then
retire the rest only when compatibility policy allows it.

### Exact Cleanup Prompt

```text
Audit the repository for version-suffixed symbols, branches, constants, types,
helpers, migration shims, and compatibility forks. The objective is not to
preserve every still-readable lane. The objective is to identify one canonical
production version per family, keep that one, remove the others when
compatibility retirement is approved, and then remove versioning from the
survivor.

Required approach:

1. Build a repository-backed inventory of versioned symbols.
2. Group symbols by version family and concept.
3. For each family, determine the one canonical production version by checking:
  - which branch is the default runtime path today;
  - which version new state writes today;
  - which version export or emit paths produce today;
  - which public contract should survive after cleanup.
4. Classify every non-canonical version as one of:
  - removable old version;
  - removable compatibility ballast;
  - removable migration helper;
  - debug-only or test-only residue.
5. Remove non-canonical versions only after the audit proves that current
  compatibility promises no longer require them.
6. After only one real implementation remains, rename that survivor to the
  simplest non-versioned canonical name.
7. Update imports, exports, docs, tests, metadata constants, and dispatch logic
  together with the cleanup.
8. Remove stale branches, selectors, and forks that exist only to keep old
  versions alive.
9. Do not rename to a target that collides with an already-live canonical
  symbol. Choose the clearest collision-free canonical name.
10. The desired end state is one production path per concept, not a permanent
   multi-version compatibility matrix.

Deliverables:

- a production-choice table with repository evidence for the chosen survivor in
  each family;
- a remove table for every non-canonical version and helper;
- the concrete code cleanup patch;
- updated tests and docs;
- a short justification for every versioned symbol that remains, if any remain
  at all.

Examples of desired simplification intent after the canonical production head
is chosen and old lanes are removed:

- `AAD_SECRET_VER_V1` / `AAD_SECRET_VER_V2` -> `AAD_SECRET`
- `HKDF_INFO_VER_V1` / `HKDF_INFO_VER_V2` -> `HKDF_INFO`
- `ClaimV2Err` -> `ClaimErr`
- `V1Basic` -> `Basic`
- `V2Memo` -> `Memo`

Important: the key decision is not whether an old path is still technically
reachable. The key decision is which single version is the intended production
contract after cleanup. Preserve that one, remove the others when policy and
compatibility allow it, then collapse the surviving name.
```

### Multi-Version Families

In this table, `Current Production Head` is repo-backed. The retirement column
lists non-canonical lanes that should be considered for retirement only after a
manual compatibility review confirms that old wallets, backups, imports,
migrations, and gated legacy APIs no longer need them.

| Family | Current Production Head | Still-Shipped Non-Canonical Lanes | Cleanup Direction | Repository Evidence | Confidence | Phase 035 | Comments |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Wallet HKDF version family | `HKDF_INFO_VER_V2` plus `derive_wallet_keys_v2` | `HKDF_INFO_VER_V1`, `derive_wallet_keys_v1`, `redb_hkdf_info_data_v1`, `redb_hkdf_info_index_v1`, `redb_hkdf_info_integrity_v1`, and V1 open or migration branches | keep V2 as the canonical head; retire V1 lanes only after wallet-open and migration compatibility is explicitly dropped; only then consider collapsing the survivor to an unsuffixed name | new wallets derive with V2 in `redb_wallet_store_create.rs`; metadata persists V2 in `redb_wallet_store_meta.rs`; V1 still appears in `redb_wallet_store_open_session.rs` and `redb_wallet_store_migrations.rs` | High | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |
| Wallet secret AAD family | `AAD_SECRET_VER_V2` plus unsuffixed `aad_secret` | `AAD_SECRET_VER_V1`, `aad_secret_v1`, and V1 fallback decrypt or migration branches | keep unsuffixed `aad_secret` as the canonical path; do not remove V1 helper or selector branches until old-wallet compatibility is explicitly retired | V2 is written in `redb_wallet_store_meta.rs`; normal encryption uses `aad_secret`; V1 fallback remains live in `redb_wallet_store_crypto_ops.rs` and `redb_wallet_store_migrations.rs` | High | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |
| KDF record version family | `KdfParams::VERSION_V2` with `KDF_VERSION` already pointing at it | `KdfParams::VERSION_V1`, `migrate_kdf_v1_to_v2`, and V1 validation or import paths | keep V2 as the canonical KDF head; treat V1 retirement as a compatibility-policy change, not a syntax cleanup | `key_manager_redb.rs` defines `KDF_VERSION` as V2; `redb_wallet_crypto.rs` still validates V1 and V2; migration remains live in `key_manager_redb.rs` and `redb_wallet_store_migrations.rs` | High | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |
| Backup AAD framing family | `build_aad_bytes_v3` via unsuffixed `build_aad_bytes` | `build_aad_bytes_v1`, `build_aad_bytes_v2`, and old verify or import branches | keep the unsuffixed wrapper as the canonical API; do not retire old framing helpers until compatibility with old backups is explicitly dropped | `backup_exporter_impl.rs` routes the wrapper to V3; importer and verifier still accept V3, then V2, then V1 | High | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |
| Backup payload decode family | `decode_export_pack_v4` | `decode_export_pack_v3`, `decode_export_pack_v2`, and `decode_legacy_payload_v1` | treat V4 as the preferred head, but keep older decoders until backup compatibility policy changes; only then consider collapsing V4 to an unsuffixed name | `backup_importer_impl.rs` chooses V4 first, then falls back to V3, V2, and legacy V1 | High | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |
| Asset pack family | current scan support prefers `AssetPackVersion::V1Basic` | `AssetPackVersion::V2Memo` remains live in core version classification | do not remove `V2Memo` from the enum based only on scan support; first prove that no production wire, decode, or future-finalized path still depends on it | `z00z_core/src/assets/version.rs` still defines and classifies both variants; `stealth_scan_support.rs` only proves current scan behavior | Medium | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |
| Claim protocol family | current public head is the V2 surface exported through `claim/mod.rs` | legacy proof, statement, prover, and verifier modules still exist behind `legacy-claim-v1`, `test`, or `doctest` gates | treat V2 as the current public head; do not delete legacy modules or rename public V2 symbols until an explicit API-retirement decision is made | `claim/mod.rs` exports V2 unconditionally; legacy modules stay feature-gated | High | ✅ | Family review completed; boundary stays frozen until a later compatibility-policy change. |

### Single Surviving `V1` Review Candidates

These rows do not currently show a competing `V2` sibling in the repository,
but that does not make them automatic suffix-drop work. They should be read as
review candidates with explicit contract boundaries.

| Current Symbol | Current Role | Safe Action Boundary | Why Review Is Required | Confidence | Phase 035 | Comments |
| --- | --- | --- | --- | --- | --- | --- |
| `CipherSeedContainer::VERSION_V1` | current seed-container persisted version marker | only a symbol-level rename candidate | the stored or wire literal may still intentionally stay on version `1` even if the Rust name changes | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `AAD_VERSION_V1` | current seed-container AAD version marker | only a symbol-level rename candidate | part of deterministic AAD construction, so name cleanup must not silently mutate the contract | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `OBJECT_PAYLOAD_HEADER_VERSION_V1` | current encrypted object payload header marker | only a symbol-level rename candidate | encoded header byte is storage-facing | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `TX_TIME_KEY_VERSION_V1` | current tx-time index key marker | only a symbol-level rename candidate | index key bytes are persisted and may be relied on across reopen paths | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `SEMANTIC_KEY_VERSION_V1` | current semantic index key marker | only a symbol-level rename candidate | same persisted-index boundary as tx-time keys | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `META_WALLET_INTEGRITY_V1` | current wallet integrity meta key | only a symbol-level rename candidate | the persisted key string `wallet.integrity.v1` is live in schema, write, and validate paths | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `DENYLIST_BLOOM_V1` | current embedded password denylist blob marker | low-priority symbol review only | looks internal, but the repository does not prove that a rename is valuable or risk-free | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `ReceiverCardRecordV1` | current published receiver-card contract | do not rename without an explicit protocol decision | version is part of a published receiver-card contract and is validated explicitly | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |
| `build_aad_v1` | private helper under a public unsuffixed `build_aad` API | do not treat as a survivor rename | this is an internal helper-shape choice, not a canonical public name awaiting collapse | Medium | ✅ | Review lane established; no mechanical suffix collapse is authorized in closed Phase 035 work. |

### Curated Production-Head Cleanup Target

Phase 035 uses the following bounded cleanup target before any code rename wave
starts.

| Lane | Default cleanup target | Boundary | Phase 035 | Comments |
| --- | --- | --- | --- | --- |
| `production-current` | converge Rust-facing survivors toward unsuffixed canonical names on the default path | do not auto-promote persisted bytes, RPC strings, published protocol contracts, or filename-only rows into rename work | ✅ | Lane boundary fixed by the canonical suffix source; this row is already disposition-complete. |
| `==reserved-future==` | keep suffix-bearing rows as compatibility or retirement-review inputs only | no direct rename or delete unless a later policy decision explicitly drops the live support lane | ✅ | Lane boundary fixed by the canonical suffix source; this row is already disposition-complete. |
| single surviving `V1` review | keep rows in symbol-review mode only | no mechanical suffix collapse until the contract boundary is proven safe | ✅ | Lane boundary fixed by the canonical suffix source; this row is already disposition-complete. |

The current curated rename handoff is intentionally smaller than the full fixed
table. At this stage, only the following `production-current` Rust-facing rows
are approved for rename planning in the curated lane:

| Surface | Suggested unsuffixed target | Why it is in scope now | Paths | Phase 035 | Comments |
| --- | --- | --- | --- | --- | --- |
| `default_v2` | `default` | current fresh-backup KDF constructor while `legacy_v1` remains the explicit compatibility lane | `crates/z00z_wallets/src/core/backup/wallet_backup_kdf.rs`; `wallet_backup.rs`; `backup_exporter_impl.rs` | ⛔️ | Curated handoff candidate only; no later Phase 035 summary proves this rename landed. |

All other suffix-bearing rows remain outside the active rename handoff until
their contract boundary is narrowed by a later curated review.

Public claim-v2 helper symbols such as `claim_stmt_hash_v2` remain outside the
Plan 05 handoff because `claim/mod.rs` still re-exports them as live public V2
symbols, and the canonical cleanup guidance requires an explicit API-retirement
decision before renaming public V2 claim surfaces.

### Decision Notes

- `proc-cons` was not found as a runnable workspace command, script, or skill
  in the repository scan used for this consolidation, so the production-head
  choices above remain grounded in direct workspace evidence plus skeptical
  review rather than a separate proc-cons pass.
- This section is safe to use as the canonical production-head selection note.
  It is not a blanket delete authority for wallet, backup, or claim
  compatibility lanes.
- Confidence is `High` where the chosen head is proven by a current write path,
  default runtime branch, or first-choice importer or exporter branch.
- Confidence is `Medium` where the repository shows only one surviving `V1`
  surface and the recommendation is therefore limited to a case-by-case symbol
  review rather than repo-proven competing-version elimination.

### Stale Or Corrected Inventory Rows

| Row | Correction | Repository Evidence | Phase 035 | Comments |
| --- | --- | --- | --- | --- |
| `CheckpointStmtV1` | remove from the canonical suffix-authority surface and the Plan 05 suffix-lane handoff; live code already uses `CheckpointStmt`, while the surviving version marker is `CheckpointStatement::V1` rather than a suffixed struct name | `artifact_stmt.rs` defines `CheckpointStmt` and `CheckpointStatement::V1`; `artifact_proof_draft.rs` and `artifact_final.rs` import `CheckpointStmt` directly | ✅ | Correction is already captured; keep this row out of active rename execution. |
| `Argon2idParamsV1` | remove from the canonical suffix-authority surface and the Plan 05 suffix-lane handoff; live code already uses `Argon2idParams` | `seed_cipher_params.rs` defines `Argon2idParams`; no live `Argon2idParamsV1` symbol surfaced in the workspace scan | ✅ | Correction is already captured; keep this row out of active rename execution. |
| `ClaimStmtV2` / `ClaimAuthoritySigV2` / `ClaimSourceProof` | remove from the canonical suffix-authority surface and fixed-table status rows; live code exports the unsuffixed structs `ClaimStmt`, `ClaimAuthoritySig`, and `ClaimSourceProof` while only `claim_stmt_hash_v2` remains suffix-bearing | `claim/v2.rs` defines unsuffixed structs; `claim/mod.rs` re-exports the unsuffixed public surface | ✅ | Correction is already captured; keep this row out of active rename execution. |

These corrections apply to this canonical suffix-authority document and the
Plan 05 suffix-lane handoff only. Historical raw matrices or later phase-local
backlog slices may still carry legacy sightings until their own cleanup waves
reconcile them.

## 📁 Filename Surfaces

The non-hidden repo filenames carrying `vN` or `Vn` suffixes are:

- `crates/z00z_crypto/src/claim/v2.rs`
- `crates/z00z_crypto/tests/test_claim_v2_contract.rs`
- `crates/z00z_wallets/src/core/security/common-passwords-v1.txt`
- `crates/z00z_wallets/src/core/security/password_denylist_v1.bloom`
- `crates/z00z_wallets/src/core/tx/asset_selector_multi_v1.rs`
- `crates/z00z_wallets/src/core/tx/asset_selector_multi_v1_tests.rs`
- `docs/VADIM_Z00Z_Digital_Cash_V2.odt`

### Filename Hygiene Boundary

- The filename lane is separate from Rust signature ownership and cannot be used
  to infer symbol renames by itself.
- Only the non-hidden repository paths listed above are part of the filename
  lane for Phase 035.
- `docs/VADIM_Z00Z_Digital_Cash_V2.odt` stays inventory-only in this phase; it
  is not promoted into the curated rename lane.
- Hidden paths, planning artifacts, temporary files, and grep-only string hits
  remain outside the filename lane unless a later canonical source widens
  scope explicitly.

## ⚠️ Explicit Exclusions

This inventory does not elevate the following into primary signature surfaces:

1. usage-only tokens found by broad grep passes;
2. local variables and temporary values;
3. string fragments without definition ownership, except the explicit RPC method
   contract string `wallet.key.export_public_material_v2`;
4. comment-only labels such as `ZkPack_v1` when they are not backed by a Rust
   definition line;
5. UUID helpers such as `new_v4`, because they are not repository versioned
   API surfaces.

### Curated Rename Handoff Boundary

Before the suffix validation wave, only declaration-backed rows listed in
`Curated Production-Head Cleanup Target` may flow into
the Plan 05 suffix-lane handoff subsection inside `035-a6-renames.md` as active
rename candidates.

The following remain outside the Plan 05 suffix-lane handoff subsection in this
phase slice:

- `==reserved-future==` compatibility rows;
- single surviving `V1` review candidates;
- filename-only rows;
- corrected rows already quarantined under `Stale Or Corrected Inventory Rows`;
- explicit exclusions, including usage-only, comment-only, local, temporary,
  and hidden-path artifacts;
- public contract strings such as `wallet.key.export_public_material_v2`.

### Suffix Validation Wave - 2026-04-12

- A consistency sweep across `035-a2-suffixes.md`, `035-a6-renames.md`, and the
  appended suffix block in `035-TODO.md` rechecked the repeated declaration
  families, the active backup-wire `V1` rows, and the explicit RPC string
  exception against the current canonical wording.
- The validation sweep confirmed that repeated names such as `VERSION_V1` and
  `VERSION_V2` stay declaration-backed and do not collapse across unrelated
  types or modules.
- The validation sweep confirmed that filename-only rows, explicit exclusions,
  test-only rows, and corrected-row quarantine remain separate truths and do
  not widen the active rename handoff.
- The validation sweep confirmed that the curated rename lane still contains
  only the single declaration-backed `default_v2 -> default` candidate and that
  public contract strings such as `wallet.key.export_public_material_v2` remain
  outside rename execution authority.
- No undeclared suffix row or protected-row drift was found in the validated
  Plan 06 planning surface.

## 🔚 Bottom Line

Phase 035 now has one canonical document for suffix-bearing names and versioned
family interpretation:

1. syntax-first Rust and filename inventory;
2. workspace-backed `production-current` vs `==reserved-future==`
  classification;
3. merged production-head and cleanup guidance for versioned families.

Use this file to decide what is current, what is compatibility ballast, what is
only a symbol-review candidate, and what inventory entries have already drifted
from live code.

### Cleanup Readiness Decision

The suffix lane is now validated strongly enough to support later curated
rename planning that stays gated by this canonical suffix authority and its
bounded handoff.

This readiness decision is intentionally narrow:

- it proves the current planning lane is declaration-backed, scope-bounded, and
  anti-drift;
- it does not authorize blanket source-file renames, blanket deletion of
  compatibility rows, or semantic closure for unrelated wallet, storage,
  checkpoint, claim, sender, or stealth behavior.

## 🔗 TODO One-To-One Mapping

| 035-2 section | Task coverage | Mapping note | Phase 035 | Comments |
| --- | --- | --- | --- | --- |
| `Objective` | `035-08`; `035-09`; `035-11` | freezes this file as the canonical suffix source and inventory baseline | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Collection Rules` | `035-09`; `035-11`; `035-13` | preserves declaration-backed grouping and exclusion of grep noise | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Non-Test Signature Surfaces` | `035-09`; `035-10`; `035-13` | covers live signature ownership and production-head review | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Test-Only Signature Surfaces` | `035-09`; `035-13` | keeps test-only rows visible without promoting them into default rename scope | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Fixed Table` | `035-08`; `035-10`; `035-12`; `035-14` | drives `production-current` versus compatibility-lane cleanup decisions | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Cleanup Interpretation And Production-Head Guidance` | `035-08`; `035-10`; `035-12`; `035-13`; `035-14` | converts the merged cleanup guidance into execution order | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Intent` | `035-08`; `035-10` | fixes the default cleanup target and lane boundaries | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Guardrails` | `035-08`; `035-10`; `035-11`; `035-12` | blocks blanket deletion and inventory misuse | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Production-Choice Reminder` | `035-10`; `035-12`; `035-14` | keeps the unsuffixed default-path target explicit | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Exact Cleanup Prompt` | `035-10`; `035-12`; `035-13` | translates the precise cleanup wording into backlog checks | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Multi-Version Families` | `035-10`; `035-12`; `035-13` | routes family-level decisions into curated review rather than raw grep cleanup | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Single Surviving \`V1\` Review Candidates` | `035-10`; `035-12`; `035-13` | keeps surviving V1 rows in an explicit review lane | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Decision Notes` | `035-08`; `035-12`; `035-14` | preserves the canonical interpretation choices and closure wording | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Stale Or Corrected Inventory Rows` | `035-09`; `035-13` | prevents corrected rows from drifting back into rename scope | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Filename Surfaces` | `035-11`; `035-13` | preserves the dedicated filename lane | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Explicit Exclusions` | `035-11`; `035-13` | keeps hidden-path, local, and comment-only artifacts out of primary rows | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
| `Bottom Line` | `035-08`; `035-12`; `035-14` | closes the suffix lane on one canonical source and a curated rename handoff | ✅ | Section-to-task mapping is already present in this canonical suffix source. |
