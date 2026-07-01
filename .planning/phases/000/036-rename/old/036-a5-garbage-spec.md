# Garbage Filter Audit

## 🔑 Classification Rules

- `InProduction = TRUE`: active current path or active compatibility/migration path proven by live callers.
- `InProduction = FALSE`: debug-only, test-only, or dead support path not used by the main processing flow.
- `Comments = GARBAGE`: safe candidate for removal review because it is test-only or a stale no-op helper.

| **Path**                                                     | **Name**                      | **Kind** | **VariationOf**                    | **AllVariants**                                              | **Role**                                            | **InProduction** | **Evidence**                                                 | **Comments**                                    |
| ------------------------------------------------------------ | ----------------------------- | -------- | ---------------------------------- | ------------------------------------------------------------ | --------------------------------------------------- | ---------------- | ------------------------------------------------------------ | ----------------------------------------------- |
| crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs | LegacyProofBlob               | struct   | proof blob wire                    | ProofBlob, ProofBlobV0, LegacyProofBlob                      | Whitebox legacy fixture blob                        | FALSE            | Exists only in test/whitebox module                          | GARBAGE                                         |
| crates/z00z_storage/src/checkpoint/ids.rs                    | ArtWire (top-level shell)     | struct   | checkpoint artifact statement wire | LegacyArtWire, ArtWire, CheckpointStmtV1                     | Test-only top-level artifact shell used in ID tests | FALSE            | Present only as the top-level test shell in ids.rs; the surviving local replacement is UnsupportedVersionArtWire | GARBAGE                                         |
| crates/z00z_wallets/src/core/backup/wallet_backup.rs         | derive_key_legacy_v1          | fn       | wallet backup key derivation       | derive_key, derive_key_legacy_v1, derive_key_with_kdf        | Public convenience wrapper for legacy KDF           | FALSE            | Only direct hits were test calls; live importer uses derive_key_with_kdf() instead | Support-only; removal possible after API review |
| crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs | debug_export_wallet           | fn       | debug wallet dump pipeline         | debug_export_wallet, verify_debug_wallets, enrich_debug_dump_with_assets | Wallet debug dump exporter                          | FALSE            | Re-exported only behind wallet_debug_tools; consumed by feature-gated simulator debug-dump and post-claim export paths | DEBUG-ONLY                                      |
| crates/z00z_simulator/src/scenario_1/runner_verify.rs        | verify_debug_wallets          | fn       | debug wallet dump pipeline         | debug_export_wallet, verify_debug_wallets, enrich_debug_dump_with_assets | Scenario-side debug dump verifier                   | FALSE            | Called only from simulator verification flow                 | DEBUG-ONLY                                      |
| crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs | enrich_debug_dump_with_assets | fn       | debug wallet dump pipeline         | debug_export_wallet, verify_debug_wallets, enrich_debug_dump_with_assets | Adds asset snapshots to debug dumps                 | FALSE            | Local simulator helper only                                  | DEBUG-ONLY                                      |
| crates/z00z_wallets/src/core/tx/state_checkpoint.rs          | _keep_checkpoint_draft        | fn       | checkpoint draft keepalive helper  | _keep_checkpoint_draft                                       | No-op keeper stub                                   | FALSE            | No direct live callers; function body is empty               | GARBAGE                                         |

## ⚠️ Filtering Notes

- Do not remove version-suffixed types only because the suffix looks old. In this repository many `V0` and `V1` names are still the active compatibility boundary.
- The strongest keep-set is in `z00z_storage` and `z00z_wallets`, where legacy decode and migration ladders are intentionally live.
- The cleanest garbage candidates found in this pass are `LegacyProofBlob`, the top-level `ArtWire` shell in `crates/z00z_storage/src/checkpoint/ids.rs`, and `_keep_checkpoint_draft`.
- Simulator debug dump helpers are working code, but they are not part of the production processing path.

