# 061-08 Summary

## Outcome

`061-08` is closed. The key tree is now physically flat under
`crates/z00z_wallets/src/key/`, the key-doc corpus moved out of `src/` into
`crates/z00z_wallets/docs/`, the include-heavy facades still resolve, and
the path-sensitive tests and planning artifacts now point at the live canonical
paths.

## File Moves

### BIP

- `src/key/bip/mod.rs` -> `src/key/bip.rs`
- `src/key/bip/bip32.rs` -> `src/key/bip32.rs`
- `src/key/bip/bip32_constants.rs` -> `src/key/bip32_constants.rs`
- `src/key/bip/bip32_key_deriver.rs` -> `src/key/bip32_key_deriver.rs`
- `src/key/bip/bip32_manager.rs` -> `src/key/bip32_manager.rs`
- `src/key/bip/bip32_path.rs` -> `src/key/bip32_path.rs`
- `src/key/bip/bip32_path_builder.rs` -> `src/key/bip32_path_builder.rs`
- `src/key/bip/bip32_path_builder_helpers.rs` ->
  `src/key/bip32_path_builder_helpers.rs`
- `src/key/bip/bip32_path_errors.rs` -> `src/key/bip32_path_errors.rs`
- `src/key/bip/bip32_path_serde.rs` -> `src/key/bip32_path_serde.rs`
- `src/key/bip/bip32_path_validator.rs` -> `src/key/bip32_path_validator.rs`
- `src/key/bip/bip32_path_value.rs` -> `src/key/bip32_path_value.rs`
- `src/key/bip/bip32_ristretto_bridge.rs` ->
  `src/key/bip32_ristretto_bridge.rs`
- `src/key/bip/test_bip32_manager.inc.rs` ->
  `src/key/test_bip32_manager.inc.rs`
- `src/key/bip/test_bip32_manager_entropy.inc.rs` ->
  `src/key/test_bip32_manager_entropy.inc.rs`

### Manager

- `src/key/manager/mod.rs` -> `src/key/manager.rs`
- `src/key/manager/key_cache.rs` -> `src/key/key_cache.rs`
- `src/key/manager/key_manager.rs` -> `src/key/key_manager.rs`
- `src/key/manager/key_manager_impl.rs` -> `src/key/key_manager_impl.rs`
- `src/key/manager/key_manager_impl_cache.rs` ->
  `src/key/key_manager_impl_cache.rs`
- `src/key/manager/key_manager_impl_cache_validation.rs` ->
  `src/key/key_manager_impl_cache_validation.rs`
- `src/key/manager/key_manager_impl_gap.rs` ->
  `src/key/key_manager_impl_gap.rs`
- `src/key/manager/key_manager_impl_state.rs` ->
  `src/key/key_manager_impl_state.rs`
- `src/key/manager/key_manager_impl_system.rs` ->
  `src/key/key_manager_impl_system.rs`
- `src/key/manager/key_manager_impl_trait.rs` ->
  `src/key/key_manager_impl_trait.rs`
- `src/key/manager/key_manager_redb.rs` -> `src/key/key_manager_redb.rs`
- `src/key/manager/key_manager_redb_wallet.rs` ->
  `src/key/key_manager_redb_wallet.rs`
- `src/key/manager/key_state.rs` -> `src/key/key_state.rs`
- `src/key/manager/test_key_manager_impl_suite.rs` ->
  `src/key/test_key_manager_impl_suite.rs`
- `src/key/manager/test_key_manager_password_suite.rs` ->
  `src/key/test_key_manager_password_suite.rs`
- `src/key/manager/test_key_manager_redb_suite.rs` ->
  `src/key/test_key_manager_redb_suite.rs`

### Receiver

- `src/key/receiver/mod.rs` -> `src/key/receiver.rs`
- `src/key/receiver/stealth_keys.rs` -> `src/key/stealth_keys.rs`
- `src/key/receiver/stealth_keys_identity.rs` ->
  `src/key/stealth_keys_identity.rs`
- `src/key/receiver/stealth_keys_receiver.rs` ->
  `src/key/stealth_keys_receiver.rs`
- `src/key/receiver/stealth_keys_secret.rs` ->
  `src/key/stealth_keys_secret.rs`
- `src/key/receiver/test_stealth_keys_suite.rs` ->
  `src/key/test_stealth_keys_suite.rs`

### Seed

- `src/key/seed/mod.rs` -> `src/key/seed.rs`
- `src/key/seed/seed_backup_format.rs` -> `src/key/seed_backup_format.rs`
- `src/key/seed/seed_backup_format_errors.rs` ->
  `src/key/seed_backup_format_errors.rs`
- `src/key/seed/seed_backup_format_phrase.rs` ->
  `src/key/seed_backup_format_phrase.rs`
- `src/key/seed/seed_cipher.rs` -> `src/key/seed_cipher.rs`
- `src/key/seed/seed_cipher_container.rs` ->
  `src/key/seed_cipher_container.rs`
- `src/key/seed/seed_cipher_container_crypto.rs` ->
  `src/key/seed_cipher_container_crypto.rs`
- `src/key/seed/seed_cipher_ids.rs` -> `src/key/seed_cipher_ids.rs`
- `src/key/seed/seed_cipher_params.rs` -> `src/key/seed_cipher_params.rs`
- `src/key/seed/seed_cipher_persistence.rs` ->
  `src/key/seed_cipher_persistence.rs`
- `src/key/seed/seed_cipher_types.rs` -> `src/key/seed_cipher_types.rs`
- `src/key/seed/seed_entropy.rs` -> `src/key/seed_entropy.rs`
- `src/key/seed/seed_mnemonic.rs` -> `src/key/seed_mnemonic.rs`
- `src/key/seed/test_seed_backup_format_basic.rs` ->
  `src/key/test_seed_backup_format_basic.rs`
- `src/key/seed/test_seed_backup_format_language.rs` ->
  `src/key/test_seed_backup_format_language.rs`
- `src/key/seed/test_seed_backup_format_suite.rs` ->
  `src/key/test_seed_backup_format_suite.rs`
- `src/key/seed/test_seed_cipher_basic_suite.rs` ->
  `src/key/test_seed_cipher_basic_suite.rs`
- `src/key/seed/test_seed_cipher_metadata_suite.rs` ->
  `src/key/test_seed_cipher_metadata_suite.rs`
- `src/key/seed/test_seed_cipher_reencrypt_suite.rs` ->
  `src/key/test_seed_cipher_reencrypt_suite.rs`

### Docs

- `src/key/manager/KEYS-DERIVATION.md` ->
  `docs/KEYS-DERIVATION.md`
- `src/key/bip/docs/KEYS-Bip44-UserGuide.md` ->
  `docs/KEYS-Bip44-UserGuide.md`
- `src/key/bip/docs/KEYS-GUIDE.md` -> `docs/KEYS-GUIDE.md`
- `src/key/bip/docs/KEYS_EXPALNATION.md` -> `docs/KEYS_EXPALNATION.md`
- `src/key/bip/docs/bip44_derivation.md` -> `docs/bip44_derivation.md`

## Rewiring

- `src/key/bip.rs` now explicitly roots the BIP facade at `bip32.rs`.
- `src/key/manager.rs` now explicitly roots the manager facade at
  `key_manager.rs` and `key_manager_redb.rs`.
- `src/key/receiver.rs` now explicitly roots the receiver facade at
  `stealth_keys.rs`.
- The key include graph remained intact because moved `include!()` fragments and
  `.inc.rs` inputs stayed in the same flat directory as their owning facade
  files.
- The path-sensitive tests now inspect `src/key/key_manager.rs`,
  `src/key/seed.rs`, and `src/key/bip32.rs`.
- `tests/test_rename_guards.rs` now guards the flat key file homes and the
  moved `docs/*` key-doc corpus, and it now includes a canonical key-doc anchor
  check.
- `061-CONTEXT.md` and `061-08-PLAN.md` now describe the live flat key/doc
  paths instead of the pre-flattening nested tree.

## Validation

### Mandatory Gate

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- Result: green; tail ended with `skip z00z_wallets examples: no example targets`
  and `=== BOOTSTRAP COMPLETE ===`

### Release Checks

- `cargo check --release -p z00z_wallets --all-targets --all-features`
- `cargo test --release -p z00z_wallets --all-targets --all-features`
- Result: green

### Slice-Specific Structural Checks

- `rg -n "src/key/(bip/|manager/|receiver/|seed/)|KEYS-DERIVATION|KEYS-Bip44-UserGuide|KEYS-GUIDE|KEYS_EXPALNATION|bip44_derivation|include!" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_wallets/docs`
- `find crates/z00z_wallets/src/key -type f -name '*.rs' | sed 's#^crates/z00z_wallets/src/key/##' | awk -F/ 'NF > 1 {print}'`
- Result: no nested `src/key/*/*` Rust files remained; no stale nested key-path
  residue remained outside the intentional negative guards in
  `tests/test_rename_guards.rs`

## Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
fallback review loop was executed manually.

### Pass 1

- `git diff --check -- ...061-08 touched files...`
- `find crates/z00z_wallets/src/key -mindepth 2 -type f | sort`
- `rg -n "src/key/(bip/|manager/|receiver/|seed/)|src/key/manager/key_manager.rs|src/key/bip/bip32.rs|src/key/seed/mod.rs|src/key/receiver/mod.rs|src/key/bip/mod.rs|src/key/manager/mod.rs|src/key/manager/KEYS-DERIVATION.md|src/key/bip/docs/" ...061-08 live code/docs/planning files...`
- `find crates/z00z_wallets/docs -maxdepth 1 -type f | sort`
- Result: clean

### Pass 2

- `git diff --check -- ...061-08 touched files...`
- `rg -n "crates/z00z_wallets/docs/KEYS-Bip44-UserGuide.md|\\[Z00Z BIP-44 User Guide\\]\\(\\./KEYS-Bip44-UserGuide.md\\)|crates/z00z_wallets/src/key/key_manager.rs|crates/z00z_wallets/src/key/bip32.rs" crates/z00z_wallets/docs/KEYS_EXPALNATION.md crates/z00z_wallets/docs/bip44_derivation.md crates/z00z_wallets/tests/test_rename_guards.rs`
- `rg -n "\\.\\./src/key/manager/key_manager.rs|\\.\\./src/key/seed/mod.rs|\\.\\./src/key/bip/bip32.rs|src/key/manager/key_manager.rs|src/key/bip/bip32.rs" crates/z00z_wallets/tests/test_key_manager.rs crates/z00z_wallets/tests/test_seed_salt_policy.rs crates/z00z_wallets/tests/test_bip44.rs crates/z00z_wallets/docs/KEYS_EXPALNATION.md`
- `find crates/z00z_wallets/src/key -maxdepth 1 -type f | sort | rg "(bip32|key_manager|stealth_keys|seed|test_bip32_manager|test_key_manager|test_stealth_keys)"`
- Result: clean

### Pass 3

- `git diff --check -- ...061-08 touched files...`
- `rg -n "#\\[path = \\\"(bip32|key_manager|key_manager_redb|stealth_keys)\\.rs\\\"\\]" crates/z00z_wallets/src/key/bip.rs crates/z00z_wallets/src/key/manager.rs crates/z00z_wallets/src/key/receiver.rs`
- `cargo test --release -p z00z_wallets --test test_rename_guards --test test_key_manager --test test_seed_salt_policy --test test_bip44`
- Result: clean

Passes 2 and 3 were consecutive clean runs.

## Next Lane

- `061-09-PLAN.md` becomes the active lane.
