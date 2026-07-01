# 061-10 Summary

## Outcome

`061-10` is closed. The remaining domains, service-test, and egui cleanup is
now physically flat on the live wallet tree, the last non-Rust wallet assets
that still belonged under `src/` moved into the canonical flat
`crates/z00z_wallets/docs/` home, the stale egui typo and placeholder paths
are retired, and the final one-level wallet tree contract is now proven on the
validated current tree.

The closeout also needed two validation-unblock fixes on the same tree:

- `test_rpc_reunlock_verify` now takes the wallet-config env lock and clears
  config env overrides before execution, which removed a real release-test
  failure in `z00z_wallets`.
- `z00z_simulator` fixture-cache unit tests now serialize through a local test
  mutex, which removed a parallel-only flake in `cargo test --release` without
  changing production logic.

## File Moves

### Domains

- `src/domains/definitions/test_mod.rs` -> `src/domains/test_definitions.rs`
- `src/domains/hashing/test_mod.rs` -> `src/domains/test_hashing.rs`

### Services

- `src/services/wallet/tests/test_wallet_paths_suite.rs` ->
  `src/services/test_wallet_paths_suite.rs`

### Egui

- `src/egui_views/wallet_tab_stacking.rs` ->
  `src/egui_views/wallet_tab_staking.rs`
- `src/egui_views/app_settings_tab_2.rs` -> retired after live-reference proof

### Docs And Assets

- `src/domains/domains_snapshot.txt` -> `docs/domains_snapshot.txt`
- `src/egui_views/egui_views.tar.gz` -> `docs/egui_views.tar.gz`

## Rewiring

- `src/domains/definitions.rs` now roots its flat test file at
  `test_definitions.rs`.
- `src/domains/hashing.rs` now roots its flat test file at `test_hashing.rs`.
- `src/services/wallet_paths.rs` now includes
  `test_wallet_paths_suite.rs` from the flat services root.
- `src/egui_views/tab_registry.rs` now points at `wallet_tab_staking` as the
  only live module name.
- `src/domains/test_definitions.rs` now reads
  `../../docs/domains_snapshot.txt`.
- `crates/z00z_storage/tests/test_live_guardrails.rs` now points at the live
  tx proof file `../../z00z_wallets/src/tx/claim_tx_verifier_impl_proof.rs`.
- `src/redb_store/test_redb_store.rs` and the wallet docs now stay anchored to
  the flat `crates/z00z_wallets/docs/*` authority.

## Validation

### Mandatory Gate

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Result: green with explicit `STATUS:0` and trailing
  `=== BOOTSTRAP COMPLETE ===`

### Release Checks

- `cargo fmt --all --check`
- `cargo check --release -p z00z_wallets --all-targets --all-features`
- `cargo test --release -p z00z_wallets --all-targets --all-features`
- `cargo test --release -p z00z_simulator --lib`
- `cargo test --release`
- Result: green

### Targeted Guardrails

- `cargo test --release -p z00z_wallets --test test_rename_guards`
- `cargo test --release -p z00z_storage --test test_live_guardrails`
- Result: green

### Structural Checks

- `find crates/z00z_wallets/src -mindepth 3 -type f -name '*.rs'`
- `find crates/z00z_wallets/src -type f ! -name '*.rs'`
- `rg -No "#\\[path *= *\\\"[^\\\"]+\\\"\\]" crates/z00z_wallets/src -g "*.rs" | sed -E 's/.*\\\"([^\\\"]+)\\\".*/\\1/' | sort | uniq -cd`
- `rg -n "wallet_tab_stacking|app_settings_tab_2|claim_helpers|claim_tx_helpers|db::redb_wallet_crypto|crate::db::redb_wallet_crypto|pub mod redb_wallet_crypto" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_wallets/src/tx -g '*.rs'`
- Result: no nested Rust residue remained under `crates/z00z_wallets/src`, no
  non-Rust assets remained under `src`, no duplicate `#[path]` targets
  remained, and the only stale-name hits were the intentional negative guards
  in `tests/test_rename_guards.rs`

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
fallback review loop was executed manually against the same closeout scope.

### Pass 1

- `find crates/z00z_wallets/src -mindepth 3 -type f -name '*.rs'`
- `find crates/z00z_wallets/src -type f ! -name '*.rs'`
- `rg -No "#\\[path *= *\\\"[^\\\"]+\\\"\\]" crates/z00z_wallets/src -g "*.rs" | sed -E 's/.*\\\"([^\\\"]+)\\\".*/\\1/' | sort | uniq -cd`
- `rg -n "wallet_tab_stacking|app_settings_tab_2|claim_helpers|claim_tx_helpers|db::redb_wallet_crypto|crate::db::redb_wallet_crypto|pub mod redb_wallet_crypto" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_wallets/src/tx -g '*.rs'`
- Result: clean; only intentional negative-guard hits remained

### Pass 2

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo fmt --all --check`
- `cargo test --release -p z00z_simulator --lib`
- Result: clean

### Pass 3

- `cargo test --release`
- `cargo test --release -p z00z_wallets --test test_rename_guards`
- `cargo test --release -p z00z_storage --test test_live_guardrails`
- Result: clean

Passes 2 and 3 were consecutive clean runs.

## Closeout

- `061-10-SUMMARY.md` now records the final domains/service-test/egui cleanup
  truth on the live tree.
- Phase 061 is complete on the existing
  `.planning/phases/061-Wallet-Refactoring/` directory only.
