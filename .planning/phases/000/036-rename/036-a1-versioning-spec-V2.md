# 036-a1-versioning-spec-V2

This file is a working Phase 036 verification register for version-bearing
symbols, local names, and contract-bearing body literals.

Repository authority remains the closed planning chain rooted at
`036-a1-versioning-spec.md` and continued by `036-TODO-2.md` plus
`036-CONTEXT.md`. If this verification sheet disagrees with that authority
chain, the authority chain wins.

## 🎯 Policy

1. There is exactly one production lane unless the code proves that multiple
   lanes are simultaneously required by a live wire, storage, cryptographic, or
   public RPC contract.
2. This file covers every repo-visible version-bearing declaration, every local
   variable name from the raw scan, and every body literal that is itself a real
   contract marker. Pure prose, `expect(...)` text, and log chatter are not
   treated as signatures and are intentionally excluded.
3. `keep` means the version token is still required by a real contract or by an
   explicit version-scenario test.
4. `rename` means only the Rust/test/local symbol name is noise; if the encoded
   literal is still a contract, the literal stays unchanged and only the symbol
   loses the suffix/prefix.
5. `remove` means the symbol is dead or dormant and should be deleted in the
   current wave.
6. `remove after migration proof` means the symbol is a compatibility import
   lane for persisted bytes and cannot be deleted truthfully until the import
   window is explicitly retired.
7. File names are not the rename target in this wave. Only symbols, signatures,
   local variable names, and contract-bearing body markers are in scope.

Checklist semantics used in the table below:

- `✅ Done` means the current workspace already matches the row's present-tense
   Phase 036 expectation in this verification register;
- `❌ Not` means the current workspace still diverges from the row's
   verification target here;
- this checklist is a visibility aid only and does not override the canonical
   authority chain in `036-a1-versioning-spec.md`, `036-TODO-2.md`, and
   `036-CONTEXT.md`.

## 📋 Single Authoritative Table

| ID | Checklist | Kind | Current item | Path | Action | Canonical target after cleanup | Why |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ✅ Done | decl | `test_lz4_legacy_rejected` | `crates/z00z_utils/src/compression/test_compression.rs` | keep | same | explicit legacy rejection scenario |
| 2 | ✅ Done | decl | `legacy_tests` | `crates/z00z_utils/src/io/fs.rs` | keep | same | test module explicitly scoped to legacy paths |
| 3 | ✅ Done | decl | `legacy_made_rows` | `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs` | keep | same | simulator compatibility helper; legacy is the behavior, not noise |
| 4 | ✅ Done | decl | `test_version_v1_is_supported` | `crates/z00z_storage/src/serialization/artifact.rs` | keep | same | explicit V1 support test should stay version-explicit |
| 5 | ✅ Done | decl | `HKDF_INFO_WALLET_KEY` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live KDF domain contract value is versioned |
| 6 | ✅ Done | decl | `HKDF_INFO_WALLET_ENCRYPTION` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live KDF domain contract value is versioned |
| 7 | ✅ Done | decl | `HKDF_INFO_TX_SIGNING` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live KDF domain contract value is versioned |
| 8 | ✅ Done | decl | `HKDF_INFO_BACKUP` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live KDF domain contract value is versioned |
| 9 | ✅ Done | decl | `HKDF_INFO_ONIONNET_SESSION` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live KDF domain contract value is versioned |
| 10 | ✅ Done | decl | `HKDF_INFO_ASSET_BLINDING` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live KDF domain contract value is versioned |
| 11 | ✅ Done | decl | `HKDF_INFO_REDB_DATA` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live redb derivation domain currently uses `.v2` literal |
| 12 | ✅ Done | decl | `HKDF_INFO_REDB_INDEX` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live redb derivation domain currently uses `.v2` literal |
| 13 | ✅ Done | decl | `HKDF_INFO_REDB_INTEGRITY` | `crates/z00z_crypto/src/kdf_domains.rs` | keep | same | live redb derivation domain currently uses `.v2` literal |
| 15 | ✅ Done | decl | `ASSET_PACK_DOMAIN` | `crates/z00z_crypto/src/aead_transport.rs` | keep | same | body literal is live crypto contract |
| 16 | ✅ Done | decl | `ProofBlobV0` | `crates/z00z_storage/src/assets/proof.rs` | remove after migration proof | none | persisted decode fallback |
| 17 | ✅ Done | decl | `test_legacy_opaque_proof_rejects_on_seal` | `crates/z00z_storage/tests/test_checkpoint_store_api.rs` | keep | same | explicit legacy rejection scenario |
| 18 | ✅ Done | decl | `try_hmac_helpers_match_legacy_outputs` | `crates/z00z_crypto/tests/test_fail_closed.rs` | keep | same | explicit legacy compatibility comparison |
| 19 | ✅ Done | decl | `owner_message_bytes` | `crates/z00z_core/src/assets/asset_ownership.rs` | keep | same | parameter `legacy` models real compatibility branch |
| 20 | ✅ Done | decl | `to_owner_message_legacy` | `crates/z00z_core/src/assets/asset_ownership.rs` | keep | same | explicit compatibility helper still exercised |
| 21 | ✅ Done | decl | `test_legacy_owner_signature_still_verifies` | `crates/z00z_core/src/assets/test_asset_suite.rs` | keep | same | explicit legacy behavior test |
| 23 | ✅ Done | decl | `KDF_CONS_SALT` | `crates/z00z_crypto/src/kdf.rs` | keep | same | live KDF salt contract |
| 24 | ✅ Done | decl | `KDF_WLT_SALT` | `crates/z00z_crypto/src/kdf.rs` | keep | same | live KDF salt contract |
| 25 | ✅ Done | decl | `KDF_WLT_VAR_SALT` | `crates/z00z_crypto/src/kdf.rs` | keep | same | live KDF salt contract |
| 26 | ✅ Done | decl | `test_rejects_seed_genesis_v3` | `crates/z00z_core/tests/genesis/test_config.rs` | keep | same | explicit version-subject rejection test |
| 27 | ✅ Done | decl | `test_rejects_seed_genesis_v2` | `crates/z00z_core/tests/genesis/test_config.rs` | keep | same | explicit version-subject rejection test |
| 28 | ✅ Done | decl | `ClaimNullRecV0` | `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs` | remove after migration proof | none | persisted row compatibility import lane |
| 29 | ✅ Done | decl | `legacy_hash` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | helper models legacy hash path explicitly |
| 30 | ✅ Done | decl | `ClaimNullRecV0` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | remove after migration proof | none | fixture mirrors persisted compatibility row |
| 31 | ✅ Done | decl | `test_redb_loads_legacy_claim_null_rows` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit compatibility import proof |
| 32 | ✅ Done | decl | `test_redb_rejects_legacy_checkpoint_link_bundle` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit legacy rejection test |
| 33 | ✅ Done | decl | `test_proof_blob_decode_legacy_v0_upgrades_root_bind` | `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | keep | same | required until `ProofBlobV0` retirement proof exists |
| 34 | ✅ Done | decl | `test_legacy_stage6_wrapper_rejects` | `crates/z00z_storage/tests/test_checkpoint_codec.rs` | keep | same | explicit legacy rejection test |
| 35 | ✅ Done | decl | `V1` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | `ClaimRootVer::V1` is a live wire discriminant |
| 36 | ✅ Done | decl | `V2` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | `ClaimRootVer::V2` is an explicit reserved discriminant |
| 37 | ✅ Done | decl | `V1` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | `ClaimProofVer::V1` is a live wire discriminant |
| 38 | ✅ Done | decl | `V2` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | `ClaimProofVer::V2` is an explicit reserved discriminant |
| 40 | ✅ Done | decl | `legacy_stage6_json` | `crates/z00z_storage/tests/checkpoint/test_fixtures.rs` | keep | same | explicit legacy fixture |
| 41 | ✅ Done | decl | `test_legacy_link_bytes_upgrade_to_bound_link` | `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` | keep | same | explicit compatibility upgrade test |
| 42 | ✅ Done | decl | `test_legacy_artifact_rejects_link_binding` | `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` | keep | same | explicit legacy rejection test |
| 43 | ✅ Done | decl | `legacy_hash` | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | keep | same | compatibility validation helper |
| 44 | ✅ Done | decl | `test_legacy_opaque_finalize_rejects_live_surface` | `crates/z00z_storage/tests/test_checkpoint_finalization.rs` | keep | same | explicit legacy rejection test |
| 45 | ✅ Done | decl | `test_legacy_opaque_bytes_stay_legacy` | `crates/z00z_storage/tests/test_checkpoint_finalization.rs` | keep | same | explicit legacy scenario |
| 46 | ✅ Done | decl | `DOMAIN` | `crates/z00z_core/tests/assets/test_fixtures.rs` | keep | same | test domain constant intentionally tracks `.v1` |
| 48 | ✅ Done | decl | `is_legacy_opaque` | `crates/z00z_storage/src/checkpoint/artifact_types.rs` | keep | same | explicit compatibility predicate |
| 49 | ✅ Done | decl | `decode_legacy_artifact_bin` | `crates/z00z_storage/src/checkpoint/codec.rs` | keep | same | compatibility import lane |
| 50 | ✅ Done | decl | `decode_legacy_artifact_json` | `crates/z00z_storage/src/checkpoint/codec.rs` | keep | same | compatibility import lane |
| 51 | ✅ Done | decl | `new_legacy` | `crates/z00z_storage/src/checkpoint/artifact_final.rs` | keep | same | explicit legacy artifact constructor |
| 52 | ✅ Done | decl | `check_legacy_sys` | `crates/z00z_storage/src/checkpoint/artifact_final.rs` | keep | same | explicit compatibility validator |
| 53 | ✅ Done | decl | `test_adv_serial_relabel_v2_is_not_mine` | `crates/z00z_wallets/tests/test_adversarial.rs` | keep | same | explicit adversarial V2 scenario should stay version-explicit |
| 55 | ✅ Done | decl | `test_import_rejects_legacy_backup_contract` | `crates/z00z_wallets/tests/test_backup_kdf_contract.rs` | keep | same | explicit rejection test |
| 56 | ✅ Done | decl | `test_import_rejects_legacy_backup_payload_versions` | `crates/z00z_wallets/tests/test_backup_kdf_contract.rs` | keep | same | explicit rejection test |
| 57 | ✅ Done | decl | `DRAFT_ID_LABEL` | `crates/z00z_storage/src/checkpoint/ids.rs` | keep | same | live checkpoint ID label contract |
| 58 | ✅ Done | decl | `CHECKPOINT_ID_LABEL` | `crates/z00z_storage/src/checkpoint/ids.rs` | keep | same | live checkpoint ID label contract |
| 59 | ✅ Done | decl | `EXEC_ID_LABEL` | `crates/z00z_storage/src/checkpoint/ids.rs` | keep | same | live checkpoint ID label contract |
| 60 | ✅ Done | decl | `wallet_key_export_public_material_v2_is_canonical_live_contract` | `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs` | keep | same | test mirrors live public RPC contract |
| 62 | ✅ Done | decl | `test_receiver_card_record_v1_is_canonical_live_contract` | `crates/z00z_wallets/tests/test_receiver_card_record.rs` | keep | same | explicit receiver-card V1 contract test should stay version-explicit |
| 64 | ✅ Done | decl | `tx_package_digest_rejects_legacy_boundary_collision` | `crates/z00z_wallets/tests/test_tx_digest_framing.rs` | keep | same | explicit legacy rejection scenario |
| 66 | ✅ Done | decl | `test_open_rejects_legacy_wallet_kdf` | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` | keep | same | explicit rejection scenario |
| 67 | ✅ Done | decl | `test_open_rejects_v1_kdf` | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | keep | same | explicit V1 reject scenario |
| 68 | ✅ Done | decl | `ENC_AAD` | `crates/z00z_wallets/tests/test_receiver_secret_validation.rs` | keep | same | literal is a live encryption domain |
| 75 | ✅ Done | decl | `migrate_legacy_wallet_files` | `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs` | keep | same | explicit migration function; legacy is the actual behavior |
| 76 | ✅ Done | decl | `META_WALLET_INTEGRITY` | `crates/z00z_wallets/src/wasm/schema_keys.rs` | keep | same | symbol already flattened; value remains contract |
| 77 | ✅ Done | decl | `test_legacy_compact_rejected` | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | keep | same | explicit rejection scenario |
| 78 | ✅ Done | decl | `legacy_v1_bytes` | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | keep | same | explicit legacy backup fixture helper should stay version-explicit |
| 79 | ✅ Done | decl | `legacy_v1_restore_fails` | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | keep | same | explicit rejection scenario |
| 80 | ✅ Done | decl | `test_wallet_id_accepts_legacy_id` | `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs` | keep | same | explicit compatibility scenario |
| 81 | ✅ Done | decl | `test_wallet_id_password_accepts_legacy_id` | `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs` | keep | same | explicit compatibility scenario |
| 85 | ✅ Done | decl | `is_legacy` | `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs` | keep | same | explicit compatibility predicate |
| 86 | ✅ Done | decl | `AAD_MASTER_KEY_LABEL` | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | keep | same | value is live persisted/AAD contract |
| 87 | ✅ Done | decl | `AAD_SECRET_PREFIX` | `crates/z00z_wallets/src/db/redb_wallet_crypto.rs` | keep | same | value is live persisted/AAD contract |
| 88 | ✅ Done | decl | `test_rejects_legacy_kdf_version` | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | keep | same | explicit rejection scenario |
| 89 | ✅ Done | decl | `test_rejects_params_untrusted_v3` | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | keep | same | explicit version-subject rejection test |
| 90 | ✅ Done | decl | `test_rejects_params_untrusted_v4` | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | keep | same | explicit version-subject rejection test |
| 91 | ✅ Done | decl | `reject_legacy_kdf_params_on_wrap` | `crates/z00z_wallets/src/core/key/key_manager_redb_tests.rs` | keep | same | explicit rejection scenario |
| 92 | ✅ Done | decl | `SPEND_AUTH_CTX` | `crates/z00z_wallets/src/core/tx/prover.rs` | keep | same | live signing domain contract |
| 93 | ✅ Done | decl | `CLAIM_TX_TYPE` | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | keep | same | live transport tag contract |
| 94 | ✅ Done | decl | `CLAIM_PROOF_V2` | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | keep | same | outer proof-type lane is still explicitly V2 in production transport |
| 95 | ✅ Done | decl | `export_public_material_v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | keep | same | live public RPC contract name |
| 96 | ✅ Done | decl | `test_legacy_proof_stub` | `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` | keep | same | explicit legacy stub scenario |
| 97 | ✅ Done | decl | `test_legacy_sig_stub` | `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` | keep | same | explicit legacy stub scenario |
| 98 | ✅ Done | decl | `test_import_legacy_v1_backup_is_rejected` | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | keep | same | explicit version-subject rejection test |
| 99 | ✅ Done | decl | `test_import_rejects_legacy_payload_versions` | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | keep | same | explicit rejection scenario |
| 100 | ✅ Done | decl | `test_import_v4_roundtrip_preserves_chain` | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | keep | same | explicit version-subject positive scenario |
| 101 | ✅ Done | decl | `build_legacy_v1_bytes` | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | keep | same | explicit legacy fixture builder should stay version-explicit |
| 102 | ✅ Done | decl | `OWNER_ATTEST_CTX` | `crates/z00z_wallets/src/core/tx/claim_auth.rs` | keep | same | live signing domain contract |
| 103 | ✅ Done | decl | `CLAIM_CTX` | `crates/z00z_wallets/src/core/claim/claim_receipt.rs` | keep | same | live receipt contract |
| 104 | ✅ Done | decl | `CACHE_SNAPSHOT_HMAC_LABEL` | `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl_snapshot.rs` | keep | same | live persisted integrity label uses `v3` |
| 105 | ✅ Done | decl | `test_rejects_legacy_snapshot_versions` | `crates/z00z_wallets/src/core/wallet/snapshot_tests.rs` | keep | same | explicit rejection scenario |
| 106 | ✅ Done | decl | `TX_ID_DOMAIN` | `crates/z00z_wallets/src/core/tx/tx_id.rs` | keep | same | live hash-domain contract |
| 108 | ✅ Done | decl | `test_rejects_wlt_open_v2` | `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs` | rename | `test_rejects_wlt_open_invalid_save_seq` | the test corrupts `META_WALLET_SAVE_SEQ`, not a wallet version field, so the current name overstates version semantics |
| 109 | ✅ Done | decl | `AAD_DOMAIN` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support_tail.rs` | keep | same | live AEAD/RPC contract literal |
| 110 | ✅ Done | decl | `PAY_REF_DOMAIN` | `crates/z00z_wallets/src/core/tx/pay_ref.rs` | keep | same | live hash-domain contract |
| 111 | ✅ Done | decl | `REQUEST_SIGN_CTX` | `crates/z00z_wallets/src/core/address/stealth_request.rs` | keep | same | live signing domain contract |
| 112 | ✅ Done | decl | `export_public_material_v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs` | keep | same | implementation mirrors live RPC contract |
| 113 | ✅ Done | decl | `test_snapshot_v3_verify_ok` | `crates/z00z_wallets/src/core/address/address_manager/tests.rs` | keep | same | explicit version-subject scenario |
| 114 | ✅ Done | decl | `encode_single_v2` | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | keep | same | reserved address-v2 helper is intentionally explicit and not active cleanup scope |
| 115 | ✅ Done | decl | `encode_dual_v2` | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | keep | same | reserved address-v2 helper is intentionally explicit and not active cleanup scope |
| 116 | ✅ Done | decl | `decode_v2` | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | keep | same | reserved address-v2 helper is intentionally explicit and not active cleanup scope |
| 117 | ✅ Done | decl | `export_public_material_v2_stub` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | keep | same | explicit v2 test stub boundary should stay version-explicit |
| 118 | ✅ Done | decl | `test_export_public_material_v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | keep | same | explicit public contract scenario |
| 119 | ✅ Done | decl | `test_export_public_material_v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | keep | same | explicit public contract scenario |
| 120 | ✅ Done | decl | `AAD_DOMAIN` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | keep | same | mirrors live contract literal |
| 121 | ✅ Done | decl | `ENC_AAD` | `crates/z00z_wallets/src/core/key/stealth_keys.rs` | keep | same | live encryption domain contract |
| 124 | ✅ Done | decl | `test_decode_v2_single` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | remove | none | delete with dormant helper |
| 125 | ✅ Done | decl | `test_decode_v2_dual` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | remove | none | delete with dormant helper |
| 126 | ✅ Done | decl | `test_decode_v2_invalid_type` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | remove | none | delete with dormant helper |
| 127 | ✅ Done | decl | `test_decode_v2_length_mismatch` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | remove | none | delete with dormant helper |
| 128 | ✅ Done | decl | `test_decode_v2_rejects_wrong_version` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | remove | none | delete with dormant helper |
| 129 | ✅ Done | decl | `test_v2_type_discriminates_correctly` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | remove | none | delete with dormant helper |
| 130 | ✅ Done | decl | `FEE_WGT_VER` | `crates/z00z_wallets/src/core/tx/fee_estimator.rs` | keep | same | symbol already flattened; literal remains contract |
| 131 | ✅ Done | decl | `detected_pack_to_bytes_rejects_oversized_v2_memo` | `crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs` | keep | same | explicit V2 memo oversize rejection test should stay version-explicit |
| 132 | ✅ Done | decl | `asset_send_rejects_legacy` | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` | keep | same | explicit legacy rejection scenario |
| 133 | ✅ Done | local | `legacy` | `crates/z00z_utils/src/compression/test_compression.rs` | keep | same | local models legacy payload explicitly |
| 134 | ✅ Done | local | `key_v1` | `crates/z00z_crypto/src/kdf_tests.rs` | keep | same | local compares explicit v1 domain output |
| 135 | ✅ Done | local | `key_v2` | `crates/z00z_crypto/src/kdf_tests.rs` | keep | same | local compares explicit v2 domain output |
| 136 | ✅ Done | local | `max_v1` | `crates/z00z_crypto/src/types_tests.rs` | keep | same | explicit comparative bound variable |
| 137 | ✅ Done | local | `max_v2` | `crates/z00z_crypto/src/types_tests.rs` | keep | same | explicit comparative bound variable |
| 138 | ✅ Done | local | `v1_actual_max` | `crates/z00z_crypto/src/types_tests.rs` | keep | same | explicit comparative bound variable |
| 139 | ✅ Done | local | `legacy_wallet` | `crates/z00z_crypto/tests/test_hash_policy.rs` | keep | same | local models legacy wallet-domain set |
| 140 | ✅ Done | local | `legacy_backup` | `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs` | keep | same | local names real legacy backup file path |
| 141 | ✅ Done | local | `legacy` | `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | keep | same | local models legacy proof blob |
| 142 | ✅ Done | local | `legacy_bytes` | `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | keep | same | local models encoded legacy proof bytes |
| 143 | ✅ Done | local | `legacy` | `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` | keep | same | local models legacy link wire |
| 144 | ✅ Done | local | `legacy_art` | `crates/z00z_storage/tests/test_checkpoint_link_injective.rs` | keep | same | local models legacy artifact |
| 145 | ✅ Done | local | `snapshot_v5` | `crates/z00z_core/src/assets/registry_tests.rs` | keep | same | explicit version-subject fixture |
| 146 | ✅ Done | local | `legacy` | `crates/z00z_storage/src/checkpoint/link.rs` | keep | same | local decode variable for compatibility lane |
| 147 | ✅ Done | local | `legacy` | `crates/z00z_storage/src/checkpoint/codec.rs` | keep | same | local decode variable for compatibility lane |
| 148 | ✅ Done | local | `legacy` | `crates/z00z_storage/src/assets/store_internal/redb_backend_helpers.rs` | keep | same | explicit legacy text/value parse path |
| 149 | ✅ Done | local | `legacy_key` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | compatibility fixture variable |
| 150 | ✅ Done | local | `legacy_row` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | compatibility fixture variable |
| 151 | ✅ Done | local | `legacy_exec_id` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit legacy hash/result fixture |
| 152 | ✅ Done | local | `legacy_check_bytes` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit compatibility fixture |
| 153 | ✅ Done | local | `legacy_draft_id` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit compatibility fixture |
| 154 | ✅ Done | local | `legacy_check_id` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit compatibility fixture |
| 155 | ✅ Done | local | `legacy_link` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit compatibility fixture |
| 156 | ✅ Done | local | `legacy_link_bytes` | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | keep | same | explicit compatibility fixture |
| 157 | ✅ Done | local | `legacy_message` | `crates/z00z_core/src/assets/asset_ownership.rs` | keep | same | local models compatibility message |
| 158 | ✅ Done | local | `legacy_exec_id` | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | keep | same | compatibility validation variable |
| 159 | ✅ Done | local | `legacy_draft_id` | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | keep | same | compatibility validation variable |
| 160 | ✅ Done | local | `legacy_check_id` | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | keep | same | compatibility validation variable |
| 161 | ✅ Done | local | `is_legacy_era` | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs` | keep | same | explicit compatibility branch flag |
| 162 | ✅ Done | local | `legacy_stage4` | `crates/z00z_simulator/tests/test_stage4_source_shape.rs` | keep | same | explicit historical-shape fixture |
| 163 | ✅ Done | local | `legacy_stage6` | `crates/z00z_simulator/tests/test_stage4_source_shape.rs` | keep | same | explicit historical-shape fixture |
| 164 | ✅ Done | local | `legacy_report` | `crates/z00z_simulator/src/scenario_1/stage_3_runtime.rs` | keep | same | real legacy report file path variable |
| 165 | ✅ Done | local | `v1_json` | `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs` | keep | same | explicit version-subject fixture |
| 166 | ✅ Done | local | `legacy` | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` | keep | same | explicit compatibility fixture |
| 167 | ✅ Done | local | `aad_v1` | `crates/z00z_wallets/src/core/key/seed_cipher_metadata_tests.rs` | keep | same | explicit v1/v2 comparison variable |
| 168 | ✅ Done | local | `aad_v2` | `crates/z00z_wallets/src/core/key/seed_cipher_metadata_tests.rs` | keep | same | explicit v1/v2 comparison variable |
| 169 | ✅ Done | local | `legacy_kdf` | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | keep | same | explicit compatibility fixture |
| 170 | ✅ Done | local | `legacy_record` | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | keep | same | explicit compatibility fixture |
| 171 | ✅ Done | local | `legacy_kdf_blob` | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | keep | same | explicit compatibility fixture |
| 172 | ✅ Done | local | `legacy_record_blob` | `crates/z00z_wallets/tests/test_redb_wlt_open.rs` | keep | same | explicit compatibility fixture |
| 173 | ✅ Done | local | `legacy` | `crates/z00z_wallets/tests/test_claim_state_compat.rs` | keep | same | explicit compatibility fixture |
| 174 | ✅ Done | local | `legacy_bytes` | `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs` | keep | same | explicit compatibility counter variable |
| 175 | ✅ Done | local | `legacy_tmp_path` | `crates/z00z_wallets/src/db/redb_wallet_store_create.rs` | keep | same | real legacy temp-path variable |
| 176 | ✅ Done | local | `legacy_kdf` | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` | keep | same | explicit compatibility fixture |
| 177 | ✅ Done | local | `legacy_record` | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs` | keep | same | explicit compatibility fixture |
| 178 | ✅ Done | local | `card_v0` | `crates/z00z_wallets/tests/test_stealth_request.rs` | keep | same | explicit v0/v1 comparison variable |
| 179 | ✅ Done | local | `card_v1` | `crates/z00z_wallets/tests/test_stealth_request.rs` | keep | same | explicit v0/v1 comparison variable |
| 180 | ✅ Done | local | `legacy_bech32` | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` | keep | same | explicit compatibility fixture |
| 181 | ✅ Done | local | `key_v0` | `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` | keep | same | explicit v0/v1 comparison variable |
| 182 | ✅ Done | local | `key_v1` | `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` | keep | same | explicit v0/v1 comparison variable |
| 183 | ✅ Done | local | `hash_v0` | `crates/z00z_wallets/src/core/key/stealth_keys_tests.rs` | keep | same | explicit v0/v1 comparison variable |
| 184 | ✅ Done | local | `card_v0` | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | keep | same | explicit v0/v1 comparison variable |
| 185 | ✅ Done | local | `card_v1` | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | keep | same | explicit v0/v1 comparison variable |
| 186 | ✅ Done | local | `outputs_v0` | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | keep | same | explicit v0/v1 comparison variable |
| 187 | ✅ Done | local | `outputs_v1` | `crates/z00z_wallets/tests/test_stealth_scanner_flow.rs` | keep | same | explicit v0/v1 comparison variable |
| 188 | ✅ Done | local | `legacy_include` | `crates/z00z_wallets/tests/test_phase30_split.rs` | keep | same | explicit old-include assembly fixture |
| 189 | ✅ Done | local | `legacy` | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs` | keep | same | explicit compatibility fixture |
| 190 | ✅ Done | local | `legacy` | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` | keep | same | explicit compatibility fixture |
| 193 | ✅ Done | body | `wallet.key.export_public_material_v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_requests.rs`, `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs`, `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs` | keep | same | live public RPC contract |
| 194 | ✅ Done | body | `z00z.wallet.key.export_public_material.v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support_tail.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | keep | same | live AEAD AAD contract |
| 195 | ✅ Done | body | `z00z-pub-material-v1:` | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support_tail.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | keep | same | encrypted export payload prefix is part of live envelope contract |
| 196 | ✅ Done | body | `proof_root_bind_v1` | `crates/z00z_storage/src/assets/proof.rs`, `crates/z00z_storage/tests/test_checkpoint_root_binding.rs` | keep | same | live proof binding contract label |
| 197 | ✅ Done | body | `z00z.claim.digest.v1` | `crates/z00z_wallets/src/core/tx/claim_helpers.rs` | keep | same | live claim digest domain contract |
| 198 | ✅ Done | body | `z00z.claim.scope.v1` | `crates/z00z_wallets/src/core/tx/claim_helpers.rs` | keep | same | live claim scope hash contract |
| 199 | ✅ Done | body | `z00z.output.nonce.v1` | `crates/z00z_wallets/src/core/tx/claim_helpers.rs` | keep | same | live output nonce derivation contract |
| 200 | ✅ Done | body | `z00z.claim.output_leaf.v1` | `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` | keep | same | live output-leaf hash contract |
| 201 | ✅ Done | body | `z00z.claim.owner_bind.v1` | `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` | keep | same | live owner-bind hash contract |
| 202 | ✅ Done | body | `z00z.tx.pkg.digest.v2.` | `crates/z00z_wallets/src/core/tx/tx_digest.rs` | keep | same | live tx package digest contract |
| 203 | ✅ Done | body | `data_key.v2` | `crates/z00z_wallets/src/core/hashing.rs` | keep | same | live redb HKDF info contract |
| 204 | ✅ Done | body | `index_key.v2` | `crates/z00z_wallets/src/core/hashing.rs` | keep | same | live redb HKDF info contract |
| 205 | ✅ Done | body | `integrity_key.v2` | `crates/z00z_wallets/src/core/hashing.rs` | keep | same | live redb HKDF info contract |
| 206 | ✅ Done | body | `z00z.wallet.receiver_secret.v1` | `crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs`, `crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` | keep | same | live receiver-secret derivation contract |
| 207 | ✅ Done | body | `z00z.wallet.receiver_secret.retry.v1` | `crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs`, `crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` | keep | same | live retry derivation contract |
| 208 | ✅ Done | body | `claim_tx_v1` | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`, `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs` | keep | same | live transport tag |
| 209 | ✅ Done | body | `claim_source` | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`, `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`, `crates/z00z_simulator/src/claim_pkg_consumer.rs` | keep | same | live source-proof transport tag |
| 210 | ✅ Done | body | `claim-v2 decode failed` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 211 | ✅ Done | body | `claim-v2 root version must be non-zero` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 212 | ✅ Done | body | `claim-v2 proof version must be non-zero` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 213 | ✅ Done | body | `claim-v2 tx version must be non-zero` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 214 | ✅ Done | body | `claim-v2 output leaf list must be non-empty` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 215 | ✅ Done | body | `claim-v2 proof bytes must be non-empty` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 216 | ✅ Done | body | `claim-v2 authority signature is invalid` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 217 | ✅ Done | body | `claim-v2 root version mismatch` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 218 | ✅ Done | body | `claim-v2 proof version mismatch` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 219 | ✅ Done | body | `claim-v2 source root mismatch` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 220 | ✅ Done | body | `claim-v2 backend signing failed: {0}` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | path-specific claim-v2 validation error |
| 221 | ✅ Done | body | `claim_v2` | `crates/z00z_crypto/src/claim/v2.rs` | keep | same | hasher label participates in live claim statement hash contract |
| 223 | ✅ Done | body | `Z00Z/REQv1` | `crates/z00z_crypto/src/domains.rs`, `crates/z00z_wallets/src/core/address/stealth_request.rs` | keep | same | live request-signing domain contract |
| 224 | ✅ Done | body | `Z00Z/SPEND_AUTH_V1` | `crates/z00z_wallets/src/core/tx/prover.rs` | keep | same | live spend-authorization signing domain contract |
| 225 | ✅ Done | body | `genesis_claim_v1` | `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`, `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` | keep | same | live Scenario-1 proof-type transport tag |
| 226 | ✅ Done | body | `z00z_coin_devnet_v1` | `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_core/src/assets/gas.rs`, `crates/z00z_core/src/genesis/genesis_config_devnet.yaml`, `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml`, `crates/z00z_core/src/assets/assets_config.yaml`, `crates/z00z_core/examples/genesis/genesis_config_devnet_test.yaml`, `crates/z00z_wallets/tests/test_tests_genesis_config.yaml` | keep | same | live devnet native-asset domain name contract |
| 227 | ✅ Done | body | `z00z_coin_testnet_v1` | `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_core/src/genesis/genesis_config_testnet.yaml` | keep | same | live testnet native-asset domain name contract |
| 228 | ✅ Done | body | `z00z_coin_mainnet_v1` | `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_core/src/genesis/genesis_config_mainnet.yaml` | keep | same | live mainnet native-asset domain name contract |

## ✅ Execution Order

1. Remove dead non-production surfaces first: rows `14`, `22`, `39`, `114-129`, `191-192`.
2. Rename pure symbol noise next, without changing live encoded literals: rows `4`, `47`, `53-54`, `62-65`, `69-74`, `78`, `82-84`, `94`, `101`, `107`, `117`, `131`, `210-222`, `229`.
3. Keep explicit version-subject tests and live contract literals intact; only rename helper identifiers when the scenario stays version-specific but the helper name itself adds stale `legacy` noise.
4. Keep live contract literals and public RPC names untouched unless there is an explicit protocol/storage/RPC migration plan.
5. Compatibility decode/import rows `16`, `28`, and `30` can only move to `remove` after code-backed migration proof closes the persisted import window.
