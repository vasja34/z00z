# Phase 036 A1 Embedded Versioning Inventory V3

## Scope

This file is the declaration-backed Rust signature inventory for embedded
version markers matching `vN` case-insensitively, where the marker may appear
as a prefix, body fragment, or suffix inside the symbol name.

Included scope:

- Rust declarations under `crates/**/*.rs`
- Production declarations
- Test, fixture, and helper declarations
- Exact path-specific duplicate signatures when multiple declarations exist

Excluded scope:

- `crates/z00z_crypto/tari/**`
- Usage-only call sites
- Local variables
- String literals and other non-signature rows

## Collection Method

1. Use [036-a1-versioning-spec.md](./036-a1-versioning-spec.md) as the current
   authoritative tracked register.
2. Keep only declaration-backed rows from that register.
3. Re-scan the live Rust codebase for declaration symbols containing `vN`.
4. Record live declarations that are not yet present in
   [036-a1-versioning-spec.md](./036-a1-versioning-spec.md) as `live-only delta`.

## Consolidated Declaration Inventory

| # | ID | Coverage | Symbol | Kind | Path | Source row | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | P06 | tracked production | `ClaimRootVer::V1` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | non-test 6 | tracked in `036-a1` |
| 2 | P07 | tracked production | `ClaimRootVer::V2` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | non-test 7 | tracked in `036-a1` |
| 3 | P08 | tracked production | `ClaimProofVer::V1` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | non-test 8 | tracked in `036-a1` |
| 4 | P09 | tracked production | `ClaimProofVer::V2` | assoc const | `crates/z00z_crypto/src/claim/v2.rs` | non-test 9 | tracked in `036-a1` |
| 5 | P24 | tracked production | `ProofBlobV0` | struct | `crates/z00z_storage/src/assets/proof.rs` | non-test 24 | tracked in `036-a1` |
| 6 | P25 | tracked production | `ClaimNullRecV0` | struct | `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs` | non-test 25 | tracked in `036-a1` |
| 7 | P26 | tracked production | `v2` | mod | `crates/z00z_crypto/src/claim/mod.rs` | non-test 26 | tracked in `036-a1` |
| 8 | P49 | tracked production | `encode_single_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | non-test 49 | tracked in `036-a1` |
| 9 | P50 | tracked production | `encode_dual_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | non-test 50 | tracked in `036-a1` |
| 10 | P51 | tracked production | `decode_v2` | fn | `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs` | non-test 51 | tracked in `036-a1` |
| 11 | P53 | tracked production | `CLAIM_PROOF_V2` | const | `crates/z00z_wallets/src/core/tx/claim_wire_types.rs` | non-test 53 | tracked in `036-a1` |
| 12 | P54 | tracked production | `export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | non-test 54 | tracked in `036-a1` |
| 13 | P55 | tracked production | `export_public_material_v2` | async fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs` | non-test 55 | tracked in `036-a1` |
| 14 | T13 | tracked test/helper | `legacy_v1_bytes` | fn | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | local/test 13 | tracked in `036-a1` |
| 15 | T17 | tracked test/helper | `detected_pack_to_bytes_rejects_oversized_v2_memo` | test fn | `crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs` | local/test 17 | tracked in `036-a1` |
| 16 | T50 | tracked test/helper | `test_proof_blob_decode_legacy_v0_upgrades_root_bind` | test fn | `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | local/test 50 | tracked in `036-a1` |
| 17 | T51 | tracked test/helper | `test_version_v1_is_supported` | test fn | `crates/z00z_storage/src/serialization/artifact.rs` | local/test 51 | tracked in `036-a1` |
| 18 | T52 | tracked test/helper | `ClaimNullRecV0` | test struct | `crates/z00z_storage/tests/test_redb_rehydrate.rs` | local/test 52 | tracked in `036-a1` |
| 19 | T53 | tracked test/helper | `export_public_material_v2_stub` | test mod | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs` | local/test 53 | tracked in `036-a1` |
| 20 | T54 | tracked test/helper | `test_snapshot_v3_verify_ok` | test fn | `crates/z00z_wallets/src/core/address/address_manager/tests.rs` | local/test 54 | tracked in `036-a1` |
| 21 | T61 | tracked test/helper | `test_import_legacy_v1_backup_is_rejected` | test fn | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | local/test 61 | tracked in `036-a1` |
| 22 | T62 | tracked test/helper | `test_import_v4_roundtrip_preserves_chain` | test fn | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | local/test 62 | tracked in `036-a1` |
| 23 | T63 | tracked test/helper | `build_legacy_v1_bytes` | test fn | `crates/z00z_wallets/src/core/backup/backup_importer_tests.rs` | local/test 63 | tracked in `036-a1` |
| 24 | T70 | tracked test/helper | `legacy_v1_restore_fails` | test fn | `crates/z00z_wallets/src/services/wallet_service_tests.rs` | local/test 70 | tracked in `036-a1` |
| 25 | T71 | tracked test/helper | `test_adv_serial_relabel_v2_is_not_mine` | test fn | `crates/z00z_wallets/tests/test_adversarial.rs` | local/test 71 | tracked in `036-a1` |
| 26 | T77 | tracked test/helper | `test_receiver_card_record_v1_is_canonical_live_contract` | test fn | `crates/z00z_wallets/tests/test_receiver_card_record.rs` | local/test 77 | tracked in `036-a1` |
| 27 | T82 | tracked test/helper | `wallet_key_export_public_material_v2_is_canonical_live_contract` | test fn | `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs` | local/test 82 | tracked in `036-a1` |
| 28 | D01 | live-only delta | `test_asset_cross_mixed_v2` | test fn | `crates/z00z_core/tests/assets/test_integration_assets_test24.rs:156` | none | present in code, not tracked in `036-a1` |
| 29 | D02 | live-only delta | `test_prove_claim_v1` | test fn | `crates/z00z_crypto/src/claim/prover.rs:67` | none | present in code, not tracked in `036-a1` |
| 30 | D03 | live-only delta | `compat_hash_v1` | fn | `crates/z00z_storage/src/assets/store_internal/redb_backend_validate.rs:171` | none | present in code, not tracked in `036-a1` |
| 31 | D04 | live-only delta | `test_export_public_material_v2` | test fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs:253` | none | nested under `export_public_material_v2_stub`; separate declaration not tracked in `036-a1` |
| 32 | D05 | live-only delta | `test_export_public_material_v2` | test fn | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_key_impl.rs:272` | none | outer test declaration not tracked in `036-a1` |
| 33 | D06 | live-only delta | `test_encode_single_v2` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs:2096` | none | present in code, not tracked in `036-a1` |
| 34 | D07 | live-only delta | `test_encode_dual_v2` | test fn | `crates/z00z_wallets/src/core/address/z00z_address/tests.rs:2109` | none | present in code, not tracked in `036-a1` |
| 35 | D08 | live-only delta | `test_weight_fixture_v1` | test fn | `crates/z00z_wallets/src/core/tx/fee_estimator_tests.rs:133` | none | present in code, not tracked in `036-a1` |
| 36 | D09 | live-only delta | `test_golden_yaml_v1` | test fn | `crates/z00z_wallets/src/core/stealth/facade_zkpack_tests.rs:356` | none | present in code, not tracked in `036-a1` |
| 37 | D10 | live-only delta | `test_key_golden_v1` | test fn | `crates/z00z_wallets/src/core/stealth/facade_kdf.rs:104` | none | present in code, not tracked in `036-a1` |
| 38 | D11 | live-only delta | `test_nonce_golden_v1` | test fn | `crates/z00z_wallets/src/core/stealth/facade_kdf.rs:158` | none | present in code, not tracked in `036-a1` |
| 39 | D12 | live-only delta | `rewrite_v1` | fn | `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs:64` | none | present in code, not tracked in `036-a1` |
| 40 | D13 | live-only delta | `test_chain_vector_v1` | test fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs:130` | none | present in code, not tracked in `036-a1` |
| 41 | D14 | live-only delta | `test_sender_fix_v1` | test fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs:150` | none | present in code, not tracked in `036-a1` |
| 42 | D15 | live-only delta | `test_receiver_fix_v1` | test fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs:167` | none | present in code, not tracked in `036-a1` |
| 43 | D16 | live-only delta | `test_api_parity_v1` | test fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs:182` | none | present in code, not tracked in `036-a1` |
| 44 | D17 | live-only delta | `test_domain_mismatch_v1` | test fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs:203` | none | present in code, not tracked in `036-a1` |
| 45 | D18 | live-only delta | `test_migration_vec_v1` | test fn | `crates/z00z_wallets/tests/test_phase11_derivation.rs:225` | none | present in code, not tracked in `036-a1` |
| 46 | D19 | live-only delta | `test_asset_cross_mixed_v2` | test fn | `crates/z00z_core/tests/assets/test_integration_assets_test24.rs:156` | none | present in code, not tracked in `036-a1` |
| 47 | D20 | live-only delta | `test_rejects_seed_genesis_v3` | test fn | `crates/z00z_core/tests/genesis/test_config.rs:75` | none | present in code, not tracked in `036-a1` |
| 48 | D21 | live-only delta | `test_rejects_seed_genesis_v2` | test fn | `crates/z00z_core/tests/genesis/test_config.rs:105` | none | present in code, not tracked in `036-a1` |
| 49 | D22 | live-only delta | `CompatArtWireV1` | struct | `crates/z00z_storage/src/checkpoint/codec.rs:16` | none | present in code, not tracked in `036-a1` |
| 50 | D23 | live-only delta | `decode_artifact_compat_v1_bin` | fn | `crates/z00z_storage/src/checkpoint/codec.rs:29` | none | present in code, not tracked in `036-a1` |
| 51 | D24 | live-only delta | `decode_artifact_compat_v1_json` | fn | `crates/z00z_storage/src/checkpoint/codec.rs:46` | none | present in code, not tracked in `036-a1` |
| 52 | D25 | live-only delta | `CompatCheckpointLinkV1` | struct | `crates/z00z_storage/src/checkpoint/link.rs:16` | none | present in code, not tracked in `036-a1` |
| 53 | D26 | live-only delta | `test_rejects_params_untrusted_v3` | test fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs:184` | none | present in code, not tracked in `036-a1` |
| 54 | D27 | live-only delta | `test_rejects_params_untrusted_v4` | test fn | `crates/z00z_wallets/src/db/redb_wallet_crypto_tests.rs:193` | none | present in code, not tracked in `036-a1` |

## Delta Notes

- `D01-D27` are the live declaration rows that were found in Rust source but are
  not yet represented in [036-a1-versioning-spec.md](./036-a1-versioning-spec.md).
- `D04-D05` intentionally remain separate because both declarations exist in the
  same file with different scopes.
- `D22-D25` are the additional checkpoint compatibility helpers and decode
   surfaces found in the full codebase rescan.
- `D03` and `D12` were verified as declarations, not usage-only matches.

## Recommended Follow-Up

1. Merge `D01-D27` into the authoritative `036-a1` register or explicitly mark
   them out of scope.
2. Keep future extraction declaration-backed and continue excluding locals,
   literals, and usage-only matches.
3. If Phase 036 execution uses this V3 file directly, prefer `Source row` when
   the row already exists in `036-a1` and prefer the explicit `:line` location
   for `live-only delta` rows.

## Follow-On Closure Note

The live follow-on cleanup for the V3 inventory is now closed in the workspace.
The remaining scoped declarations were neutralized or renamed to version-neutral
symbols, and the zero-versioning residual scan across the scoped non-Tari files
returned no matches.

The final cleaned set was anchored by the Wave 7 and Wave 8 closure summaries:

- `036-17-SUMMARY.md` records the V3 inventory follow-on cleanup, including the
  storage, crypto, and wallet declaration renames and the clean residual scan
  proof.
- `036-18-SUMMARY.md` records the a3 follow-on rename sweep, including the
  storage, core, simulator, wallet, support, and bench helper renames and the
  clean residual scan proof.

The authoritative scan used for closure was the scoped `rg -n` residual sweep
over the live `crates/**` declaration files listed by `036-17-PLAN.md` and
`036-18-PLAN.md`, with `crates/z00z_crypto/tari/**` excluded from the proof.
