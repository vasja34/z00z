---
phase: 030-refactor-long-files
artifact: rust-length-stat
created: 2026-04-01
updated: 2026-04-03
scope: repo-wide
language: rust
exclusions:
  - test directories and test-named Rust files
  - Tari vendor tree
source_skills:
  - skill-selector
  - rust-refactoring
  - doublecheck
---

# Rust Length Statistics

## Continuation Closeout Snapshot

📌 Final continuation verification was rerun on `2026-04-03` after the full Plan `030-24` residue burn-down.

📌 The extended Phase 030 closeout is now summary-backed by `030-24-SUMMARY.md` and `030-25-SUMMARY.md`, and the canonical `full_verify --max-safe-run` rerun closed green on `2026-04-03`.

| Metric | Value |
| --- | ---: |
| Current `TOTAL_GT400` | 0 |
| Remaining non-test Rust files above 400 lines | 0 |

📌 The repo-wide baseline captured below remains useful as the `2026-04-01` starting snapshot, but it is no longer the live continuation end state.

## Scope

📌 This report measures all Rust source files in the Z00Z project with two
explicit exclusions: test files and the Tari vendor tree.

📌 The inventory was built from repo-wide `*.rs` paths, then filtered out:
`/tari/`, `/test/`, `/tests/`, `tests.rs`, `test_*.rs`, `*_test.rs`, and
`*_tests.rs`.

📌 Test exclusion is therefore convention-based rather than semantic AST
classification. This matches the user's request closely enough for a planning
report, but it should be read as "all non-test files by repo naming
convention."

## Routing Decision

📌 Skill chain used for this artifact:

1. `rust-refactoring`
2. `doublecheck`

📌 `rust-refactoring` won because the task is Rust-only, repo-wide, and centered
on oversized modules and refactor pressure rather than general documentation or
mixed-language analysis.

## Best-Practice Baseline

📌 The comparison baseline used here is intentionally conservative and matches
both the local Phase 030 refactor guidance and the repo's refactor skills.

| Threshold | Meaning |
| --- | --- |
| 150 lines | Still comfortable for a focused Rust module |
| 250 lines | Upper end of the "good" range |
| 300 lines | Preferred top-level target in current Z00Z Phase 030 work |
| 400 lines | Strong refactor-needed threshold |
| 900 lines | Extreme size, split again before closing a wave |
| 1000 lines | Beyond the repo's allowed ceiling for resulting modules |

📌 For the user question "how much longer than best practices," the main delta
column is measured against the `300`-line preferred target. A second delta is
shown against the `400`-line strong refactor threshold.

## Global Summary

📌 Clean non-test, non-vendor Rust inventory size: `639` files.

| Metric | Value |
| --- | ---: |
| Total files | 639 |
| Total lines | 160487 |
| Average lines per file | 251.15 |
| Median lines per file | 131 |
| Smallest file | 1 |
| Largest file | 6363 |
| Top 5% sample size | 32 files |

## Threshold Counts

📌 Oversize pressure across the current non-test Rust tree:

| Threshold | Files above threshold | Share of inventory |
| --- | ---: | ---: |
| >150 lines | 286 | 44.76% |
| >250 lines | 197 | 30.83% |
| >300 lines | 174 | 27.23% |
| >400 lines | 108 | 16.90% |
| >900 lines | 28 | 4.38% |
| >1000 lines | 24 | 3.76% |

📌 The main signal is that more than one quarter of the non-test Rust files are
already beyond the current Phase 030 preferred `300`-line target, and `24`
files are even beyond the `1000`-line hard ceiling.

## Top 5 Percent Longest Files

📌 The table below lists the longest `32` files, which is the top `5%` of the
clean inventory.

📌 The top `5%` set is rounded up with `ceil(639 * 0.05) = 32` files.

| Rank | Lines | +300 | +400 | +1000 | Severity | File |
| ---: | ---: | ---: | ---: | ---: | --- | --- |
| 1 | 6363 | 6063 | 5963 | 5363 | hard-stop | `crates/z00z_wallets/src/db/redb_wallet_store.rs` |
| 2 | 4039 | 3739 | 3639 | 3039 | hard-stop | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs` |
| 3 | 2170 | 1870 | 1770 | 1170 | hard-stop | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs` |
| 4 | 2168 | 1868 | 1768 | 1168 | hard-stop | `crates/z00z_wallets/src/core/key/key_manager_impl.rs` |
| 5 | 2059 | 1759 | 1659 | 1059 | hard-stop | `crates/z00z_wallets/src/core/key/seed_cipher.rs` |
| 6 | 1997 | 1697 | 1597 | 997 | extreme | `crates/z00z_core/src/assets/assets.rs` |
| 7 | 1799 | 1499 | 1399 | 799 | extreme | `crates/z00z_wallets/src/core/key/bip32.rs` |
| 8 | 1722 | 1422 | 1322 | 722 | extreme | `crates/z00z_wallets/src/services/wallet_service_store.rs` |
| 9 | 1673 | 1373 | 1273 | 673 | extreme | `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs` |
| 10 | 1640 | 1340 | 1240 | 640 | extreme | `crates/z00z_wallets/src/services/wallet_service_session.rs` |
| 11 | 1548 | 1248 | 1148 | 548 | extreme | `crates/z00z_wallets/src/services/wallet_service_actions.rs` |
| 12 | 1456 | 1156 | 1056 | 456 | extreme | `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl.rs` |
| 13 | 1311 | 1011 | 911 | 311 | extreme | `crates/z00z_wallets/src/core/tx/state_update.rs` |
| 14 | 1300 | 1000 | 900 | 300 | extreme | `crates/z00z_core/src/assets/registry.rs` |
| 15 | 1173 | 873 | 773 | 173 | extreme | `crates/z00z_wallets/src/core/key/seed_backup_format.rs` |
| 16 | 1168 | 868 | 768 | 168 | extreme | `crates/z00z_crypto/src/aead.rs` |
| 17 | 1163 | 863 | 763 | 163 | extreme | `crates/z00z_crypto/src/kdf.rs` |
| 18 | 1129 | 829 | 729 | 129 | extreme | `crates/z00z_wallets/src/core/wallet/wallet_entity.rs` |
| 19 | 1101 | 801 | 701 | 101 | extreme | `crates/z00z_storage/src/assets/store.rs` |
| 20 | 1047 | 747 | 647 | 47 | extreme | `crates/z00z_crypto/src/types.rs` |
| 21 | 1047 | 747 | 647 | 47 | extreme | `crates/z00z_crypto/src/hash.rs` |
| 22 | 1030 | 730 | 630 | 30 | extreme | `crates/z00z_wallets/src/core/tx/tx_verifier.rs` |
| 23 | 1020 | 720 | 620 | 20 | extreme | `crates/z00z_storage/src/checkpoint/artifact.rs` |
| 24 | 1008 | 708 | 608 | 8 | extreme | `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs` |
| 25 | 979 | 679 | 579 | -21 | critical | `crates/z00z_crypto/src/lib.rs` |
| 26 | 963 | 663 | 563 | -37 | critical | `crates/z00z_wallets/src/core/tx/claim_tx.rs` |
| 27 | 921 | 621 | 521 | -79 | critical | `crates/z00z_wallets/src/core/tx/asset_selector.rs` |
| 28 | 921 | 621 | 521 | -79 | critical | `crates/z00z_core/bin/assets/assets_generation_cli.rs` |
| 29 | 899 | 599 | 499 | -101 | critical | `crates/z00z_wallets/src/core/key/stealth_keys.rs` |
| 30 | 890 | 590 | 490 | -110 | critical | `crates/z00z_simulator/src/scenario_1/stage_3.rs` |
| 31 | 883 | 583 | 483 | -117 | critical | `crates/z00z_core/src/assets/definition.rs` |
| 32 | 880 | 580 | 480 | -120 | critical | `crates/z00z_wallets/src/core/key/bip32_path.rs` |

## Interpretation

📌 The largest concentration of oversize files is inside `z00z_wallets`, with
the heaviest pressure in wallet persistence, RPC transport, key management, and
service orchestration.

📌 The single biggest outlier is
`crates/z00z_wallets/src/db/redb_wallet_store.rs` at `6363` lines. Relative to
the current good-design target, it is `6063` lines too large, or about
`21.21x` the `300`-line target.

📌 The second outlier,
`crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`, is `3739` lines
over the preferred target and `3039` lines over the hard `1000` limit.

📌 Even the bottom edge of the top `5%` remains very high: rank `32` is still
`880` lines, which is `580` lines above the preferred target and `480` above
the strong refactor threshold.

## Practical Conclusion For Phase 030

📌 If the project follows the current Z00Z refactor standard strictly, the
highest-value split candidates remain concentrated in wallet and core asset
surfaces, and the long tail above `900` lines is still materially larger than a
healthy Rust module shape.

📌 In a good Rust design, the preferred steady-state is not "everything below
exactly one number," but this dataset shows a clear structural smell: too many
files are far beyond even the repo's generous warning band, not just slightly
above it.

📌 The main action point is therefore not micro-trimming files from `320` to
`280` lines. The biggest payoff is continued responsibility-based splitting of
the top outliers above `900` and especially above `1000` lines.
