# 042-core-row-manifest

Row-by-row execution tracker for the core structural refactor.
Each row = one `git mv` batch + import fix + `cargo check`.

Legend: ✅ done | ❌ pending

---

| # | Group | Files Moved | Destination | Status |
|---|-------|-------------|-------------|--------|
| 01 | move-foundation | `core/domains.rs`, `core/hashing.rs`, `core/domains_snapshot.txt`, `core/test_domains_suite.rs`, `core/test_hashing_suite.rs` | `core/foundation/` | ✅ |
| 02 | move-network-kernel | `core/network/mod.rs` → `core/network/kernel.rs` (new mod.rs re-exports) | `core/network/` | ✅ |
| 03 | move-key-bip | `key/bip32.rs`, `bip32_constants.rs`, `bip32_key_deriver.rs`, `bip32_manager.rs`, `bip32_path.rs`, `bip32_path_builder.rs`, `bip32_path_builder_helpers.rs`, `bip32_path_errors.rs`, `bip32_path_serde.rs`, `bip32_path_validator.rs`, `bip32_path_value.rs`, `bip32_ristretto_bridge.rs`, `bip44_derivation.md`, `test_bip32_manager.inc.rs`, `test_bip32_manager_entropy.inc.rs` | `key/bip/` | ✅ |
| 04 | move-key-manager | `key/key_manager.rs`, `key_manager_impl.rs`, `key_manager_impl_cache.rs`, `key_manager_impl_cache_validation.rs`, `key_manager_impl_gap.rs`, `key_manager_impl_state.rs`, `key_manager_impl_system.rs`, `key_manager_impl_trait.rs`, `key_manager_redb.rs`, `key_manager_redb_wallet.rs`, `key_cache.rs`, `key_state.rs`, `KEYS-DERIVATION.md`, `test_key_manager_impl_suite.rs`, `test_key_manager_password_suite.rs`, `test_key_manager_redb_suite.rs` | `key/manager/` | ✅ |
| 05 | move-key-seed | `key/seed.rs`, `seed_backup_format.rs`, `seed_backup_format_errors.rs`, `seed_backup_format_phrase.rs`, `seed_backup_format_tests_basic.rs`→`test_seed_backup_format_basic.rs`, `seed_backup_format_tests_language.rs`→`test_seed_backup_format_language.rs`, `seed_cipher.rs`, `seed_cipher_container.rs`, `seed_cipher_container_crypto.rs`, `seed_cipher_ids.rs`, `seed_cipher_params.rs`, `seed_cipher_persistence.rs`, `seed_cipher_types.rs`, `seed_entropy.rs`, `seed_mnemonic.rs`, `test_seed_backup_format_suite.rs`, `test_seed_cipher_basic_suite.rs`, `test_seed_cipher_metadata_suite.rs`, `test_seed_cipher_reencrypt_suite.rs` | `key/seed/` | ✅ |
| 06 | move-key-receiver | `key/stealth_keys.rs`, `stealth_keys_identity.rs`, `stealth_keys_receiver.rs`, `stealth_keys_secret.rs`, `test_stealth_keys_suite.rs` | `key/receiver/` | ✅ |
| 07 | move-doc-key | `key/KEYS-Bip44-UserGuide.md`, `key/KEYS-GUIDE.md` | `key/docs/` | ✅ |
| 08 | move-persistence-root | `core/storage/mod.rs` → `core/persistence/mod.rs` | `core/persistence/` | ✅ |
| 09 | move-persistence-assets | `storage/asset_storage.rs`, `asset_storage_impl.rs`, `test_asset_storage_impl_suite.rs` | `persistence/assets/` | ✅ |
| 10 | move-persistence-receipts | `storage/receipt_storage.rs`, `receipt_storage_impl.rs` | `persistence/receipts/` | ❌ |
| 11 | move-persistence-scans | `storage/scan_storage.rs`, `scan_storage_impl.rs` | `persistence/scans/` | ❌ |
| 12 | move-persistence-tx | `storage/tx_storage.rs`, `tx_storage_impl.rs` | `persistence/tx/` | ❌ |
| 13 | move-persistence-wallets | `storage/wallet_storage.rs`, `wallet_storage_impl.rs` | `persistence/wallets/` | ✅ |
| 14 | move-claim-registry | `storage/claim_registry.rs` | `claim/registry/` | ❌ |
| 15 | move-security-vault | `storage/file_key_store.rs`, `secret_store.rs`, `secret_store_impl.rs` | `security/vault/` | ❌ |
| 16 | move-stealth-crypto | `stealth/ecdh.rs`, `ecdh_validation.rs`, `encoding.rs`, `ephemeral.rs` | `stealth/crypto/` | ❌ |
| 17 | move-stealth-facade | `stealth/facade_ecdh.rs`, `facade_kdf.rs`, `facade_zkpack.rs` | `stealth/facade/` | ❌ |
| 18 | move-stealth-output | `stealth/output.rs`, `output_build.rs`, `output_validator.rs`, `owner_tag.rs`, `tag.rs`, `test_facade_zkpack_suite.rs`, `test_output.rs`, `test_output_extra.rs` | `stealth/output/` | ❌ |
| 19 | move-tx-assembly | `tx/tx_assembler.rs` | `tx/assembly/` | ❌ |
| 20 | move-tx-claim | `tx/claim_auth.rs`, `claim_errors.rs`, `claim_helpers.rs`, `claim_tx.rs`, `claim_tx_helpers.rs`, `claim_tx_verifier_impl.rs`, `claim_tx_verifier_impl_proof.rs`, `claim_wire_types.rs`, `test_claim_tx.rs` | `tx/claim/` | ❌ |
| 21 | move-tx-fees | `tx/fee_estimator.rs`, `test_fee_estimator_suite.rs` | `tx/fees/` | ❌ |
| 22 | move-tx-ids | `tx/pay_ref.rs`, `tx_id.rs` | `tx/ids/` | ❌ |
| 23 | move-tx-output | `tx/builder.rs`, `output_flow.rs` | `tx/output/` | ❌ |
| 24 | move-tx-proof | `tx/prover.rs`, `signer.rs`, `spend_proof_backend.rs` | `tx/proof/` | ❌ |
| 25 | move-tx-selection | `tx/asset_selector.rs`, `asset_selector_multi.rs`, `test_asset_selector_suite.rs`, `test_asset_selector_multi_suite.rs` | `tx/selection/` | ❌ |
| 26 | move-tx-spend | `tx/spending.rs`, `spend_rules.rs`, `spend_verification.rs`, `witness_gate.rs` | `tx/spend/` | ❌ |
| 27 | move-tx-state | `tx/lifecycle.rs`, `state_checkpoint.rs`, `state_errors.rs`, `state_resolved_input.rs`, `state_traits.rs`, `state_update.rs`, `state_witness.rs`, `test_state_update_suite.rs` | `tx/state/` | ❌ |
| 28 | move-tx-verify | `tx/tx_digest.rs`, `tx_errors.rs`, `tx_verifier.rs`, `tx_verifier_helpers.rs`, `tx_wire_types.rs`, `test_tx_verifier_suite.rs` | `tx/verify/` | ❌ |
| 29 | move-wallet-entity | `wallet/wallet_entity.rs`, `wallet_entity_asset_api.rs`, `wallet_entity_backup_api.rs`, `wallet_entity_constructor.rs`, `wallet_entity_core.rs`, `wallet_entity_key_api.rs`, `wallet_entity_tx_api.rs`, `wallet_entity_wallet_api.rs` | `wallet/entity/` | ❌ |
| 30 | move-wallet-errors | `wallet/errors.rs`, `errors_impls.rs`, `errors_types.rs`, `test_errors_suite.rs` | `wallet/errors/` | ❌ |
| 31 | move-wallet-snapshot | `wallet/snapshot.rs`, `snapshot_impl.rs`, `snapshot_types.rs`, `test_snapshot_suite.rs` | `wallet/snapshot/` | ❌ |
| 32 | move-wallet-stub | `wallet/stub_responses.rs`, `stub_responses_asset.rs`, `stub_responses_backup.rs`, `stub_responses_tx.rs`, `stub_responses_wallet.rs` | `wallet/stub/` | ❌ |
| 33 | move-doc-wallet | `wallet/WALLET-GUIDE.md` | `wallet/docs/` | ❌ |
