# 061-09 Summary

## Outcome

`061-09` is closed. The tx, claim, stealth, wallet, backup, and chain leaf
trees are now physically flat under one-level domain files, the wallet guide
moved out of `src/` into `crates/z00z_wallets/docs/`, the helper rename
pair now matches the TODO semantics (`claim_tx_hashing` vs
`claim_tx_statement`), and the path-sensitive tests and guardrails point at the
live canonical homes.

## File Moves

### Backup

- `src/backup/backup_importer_impl/mod.rs` -> `src/backup/backup_importer_impl.rs`
- `src/backup/backup_importer_impl/test_mod.rs` ->
  `src/backup/test_backup_importer_impl.rs`
- `src/backup/crypto/wallet_backup_kdf.rs` ->
  `src/backup/wallet_backup_kdf.rs`
- `src/backup/export/backup_exporter_verify.rs` ->
  `src/backup/backup_exporter_verify.rs`
- `src/backup/export/test_backup_exporter_suite.rs` ->
  `src/backup/test_backup_exporter_suite.rs`
- `src/backup/wallet_backup/mod.rs` -> `src/backup/wallet_backup.rs`
- `src/backup/wallet_backup/test_mod.rs` ->
  `src/backup/test_wallet_backup.rs`

### Chain

- `src/chain/broadcast/broadcast_impl.rs` -> `src/chain/broadcast_impl.rs`

### Claim

- `src/claim/nullifier_store/test_mod.rs` ->
  `src/claim/test_nullifier_store.rs`
- `src/claim/registry/claim_registry.rs` -> `src/claim/claim_registry.rs`

### Stealth

- `src/stealth/crypto/ecdh.rs` -> `src/stealth/crypto_ecdh.rs`
- `src/stealth/crypto/ecdh_validation.rs` ->
  `src/stealth/crypto_ecdh_validation.rs`
- `src/stealth/crypto/encoding.rs` -> `src/stealth/crypto_encoding.rs`
- `src/stealth/crypto/ephemeral.rs` -> `src/stealth/crypto_ephemeral.rs`
- `src/stealth/crypto/mod.rs` -> `src/stealth/crypto.rs`
- `src/stealth/output/mod.rs` -> `src/stealth/output.rs`
- `src/stealth/output/output_build.rs` -> `src/stealth/output_build.rs`
- `src/stealth/output/tests/test_extra.rs` ->
  `src/stealth/test_output_extra.rs`
- `src/stealth/output/tests/test_mod.rs` -> `src/stealth/test_output.rs`
- `src/stealth/zkpack/mod.rs` -> `src/stealth/zkpack.rs`
- `src/stealth/zkpack/test_mod.rs` -> `src/stealth/test_zkpack.rs`

### Tx

- `src/tx/asset_selector/mod.rs` -> `src/tx/asset_selector.rs`
- `src/tx/asset_selector/multi.rs` -> `src/tx/asset_selector_multi.rs`
- `src/tx/asset_selector/multi/test_mod.rs` ->
  `src/tx/test_asset_selector_multi.rs`
- `src/tx/asset_selector/test_mod.rs` -> `src/tx/test_asset_selector.rs`
- `src/tx/claim_helpers.rs` -> `src/tx/claim_tx_hashing.rs`
- `src/tx/claim/claim_tx_helpers.rs` -> `src/tx/claim_tx_statement.rs`
- `src/tx/claim/claim_tx_verifier_impl.rs` ->
  `src/tx/claim_tx_verifier_impl.rs`
- `src/tx/claim/claim_tx_verifier_impl_proof.rs` ->
  `src/tx/claim_tx_verifier_impl_proof.rs`
- `src/tx/claim_tx/mod.rs` -> `src/tx/claim_tx.rs`
- `src/tx/claim_tx/test_claim_tx.rs` -> `src/tx/test_claim_tx.rs`
- `src/tx/fee_estimator/mod.rs` -> `src/tx/fee_estimator.rs`
- `src/tx/fee_estimator/test_mod.rs` -> `src/tx/test_fee_estimator.rs`
- `src/tx/state_update/mod.rs` -> `src/tx/state_update.rs`
- `src/tx/state_update/test_mod.rs` -> `src/tx/test_state_update.rs`
- `src/tx/tx_verifier/mod.rs` -> `src/tx/tx_verifier.rs`
- `src/tx/tx_verifier/test_mod.rs` -> `src/tx/test_tx_verifier.rs`
- `src/tx/verify/tx_verifier_helpers.rs` -> `src/tx/tx_verifier_helpers.rs`

### Wallet

- `src/wallet/entity/wallet_entity.rs` -> `src/wallet/wallet_entity.rs`
- `src/wallet/entity/wallet_entity_asset_api.rs` ->
  `src/wallet/wallet_entity_asset_api.rs`
- `src/wallet/entity/wallet_entity_constructor.rs` ->
  `src/wallet/wallet_entity_constructor.rs`
- `src/wallet/entity/wallet_entity_core.rs` ->
  `src/wallet/wallet_entity_core.rs`
- `src/wallet/entity/wallet_entity_wallet_api.rs` ->
  `src/wallet/wallet_entity_wallet_api.rs`
- `src/wallet/errors/errors_impls.rs` -> `src/wallet/errors_impls.rs`
- `src/wallet/errors/errors_types.rs` -> `src/wallet/errors_types.rs`
- `src/wallet/errors/test_errors_suite.rs` -> `src/wallet/test_errors_suite.rs`
- `src/wallet/persistence/persistence_types.rs` ->
  `src/wallet/persistence_types.rs`
- `src/wallet/responses/stub_responses_asset.rs` ->
  `src/wallet/stub_responses_asset.rs`
- `src/wallet/responses/stub_responses_backup.rs` ->
  `src/wallet/stub_responses_backup.rs`
- `src/wallet/responses/stub_responses_tx.rs` ->
  `src/wallet/stub_responses_tx.rs`
- `src/wallet/responses/stub_responses_wallet.rs` ->
  `src/wallet/stub_responses_wallet.rs`

### Docs

- `src/wallet/WALLET-GUIDE.md` -> `docs/WALLET-GUIDE.md`

## Rewiring

- `src/tx/mod.rs` now roots the helper split at `claim_tx_hashing`.
- `src/tx/claim_tx.rs` now re-exports hashing helpers from
  `claim_tx_hashing`, includes `claim_tx_statement.rs` plus
  `claim_tx_verifier_impl.rs`, and keeps its tests on `test_claim_tx.rs`.
- `src/tx/asset_selector.rs`, `src/tx/asset_selector_multi.rs`,
  `src/tx/fee_estimator.rs`, `src/tx/state_update.rs`, and
  `src/tx/tx_verifier.rs` now point at flat tests or include fragments in the
  same directory.
- `src/claim/nullifier_store.rs` and `src/claim/registry.rs` now point at the
  flat claim files.
- `src/stealth/crypto.rs`, `src/stealth/output.rs`,
  `src/stealth/test_output.rs`, and `src/stealth/zkpack.rs` now root the flat
  stealth files with explicit `#[path = "..."]` anchors.
- `src/backup/backup_exporter_impl.rs`, `src/backup/backup_importer_impl.rs`,
  and `src/backup/wallet_backup.rs` now include or mount the flat backup
  helpers and tests.
- `src/wallet/wallet.rs`, `src/wallet/errors.rs`,
  `src/wallet/persistence.rs`, and `src/wallet/stub_responses.rs` now include
  the flat wallet files directly from the wallet root.
- `src/redb_store/test_redb_store.rs` now loads
  `../../docs/WALLET-GUIDE.md`, and the path-sensitive integration tests
  now inspect the live flat tx, stealth, and wallet file homes.
- `tests/test_rename_guards.rs` now guards the full 061-09 flat-file set,
  absence of the old nested paths, and the critical `include!` or `#[path]`
  rewiring strings that keep one canonical path per behavior.

## Validation

### Mandatory Gate

- First run: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Failure: `src/tx/asset_selector_multi.rs` still used the stale
  `multi/test_mod.rs` test path.
- Fix: update the local `#[path]` target to `test_asset_selector_multi.rs`.
- Recheck: `cargo test --release -p z00z_wallets --lib --tests --no-run`
- Rerun: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Result: green; tail ended with `skip z00z_wallets examples: no example
  targets` and `=== BOOTSTRAP COMPLETE ===`

### Release Checks

- `cargo check --release -p z00z_wallets --all-targets --all-features`
- `cargo test --release -p z00z_wallets --all-targets --all-features`
- Result: green

### Slice-Specific Structural Checks

- `rg -n "claim_tx|claim_helpers|claim_tx_helpers|WALLET-GUIDE|backup/" crates/z00z_wallets/src crates/z00z_wallets/tests`
- `rg -n "claim_helpers|claim_tx_helpers" crates/z00z_wallets/src/tx -g "*.rs"`
- `find crates/z00z_wallets/src/tx crates/z00z_wallets/src/claim crates/z00z_wallets/src/wallet crates/z00z_wallets/src/backup crates/z00z_wallets/src/stealth crates/z00z_wallets/src/chain -type f -mindepth 2`
- Result: no nested Rust residue remained in the 061-09 domains, and no stale
  helper-path hits remained in live code outside the intentional negative
  guards in `tests/test_rename_guards.rs`

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
fallback review loop was executed manually against the same task scope.

### Pass 1

- `git diff --check -- ...061-09 touched files...`
- `find crates/z00z_wallets/src/tx crates/z00z_wallets/src/claim crates/z00z_wallets/src/wallet crates/z00z_wallets/src/backup crates/z00z_wallets/src/stealth crates/z00z_wallets/src/chain -type f -mindepth 2 | sort`
- `rg -n "...old 061-09 nested paths..." .planning/phases/061-Wallet-Refactoring crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md`
- Result: clean; only intentional negative-guard or historical-plan hits
  remained

### Pass 2

- `cargo test --release -p z00z_wallets --test test_rename_guards --test test_view_key_contract --test test_s5_closure_gate --test test_s5_sender_examples --test test_wallet_split`
- `rg -n "...061-09 canonical flat anchors..." crates/z00z_wallets/src crates/z00z_wallets/tests`
- `git diff --check -- ...061-09 touched files...`
- Result: clean

### Pass 3

- `cargo test --release -p z00z_wallets --test test_rename_guards`
- `rg -n "...stale include/path rewiring strings..." crates/z00z_wallets/src crates/z00z_wallets/tests`
- `git diff --check -- ...061-09 touched files...`
- Result: clean

Passes 2 and 3 were consecutive clean runs.

## Next Lane

- `061-10-PLAN.md` becomes the active lane.
