# z00z_core Rename Plan

Scope: `crates/z00z_core/src/**/*.rs` and `crates/z00z_core/tests/**/*.rs`
Excluded: `crates/z00z_crypto/tari/**`

This document is a Markdown rename plan only. It does not apply code changes.

## Core Symbol Renames

| # | Kind | Old Name | New Name | File | Line | Why | Follow-up |
| --- | --- | --- | --- | --- | ---: | --- | --- |
| 1 | fn | `h2s_zk` | `hash_to_scalar_zk` | `crates/z00z_core/src/hash.rs` | 815 | `h2s` is cryptic; the replacement spells out the operation clearly. | Update `pub use` re-exports in `hash.rs` and `lib.rs`, plus all call sites. |
| 2 | fn | `h2scalar_zk` | `hash_to_scalar_domain` | `crates/z00z_core/src/hash.rs` | 918 | Raw domain bytes plus scalar derivation is clearer with an explicit name. | Update the deprecated wrapper note, re-exports, and all call sites. |
| 3 | fn | `h2scalar_zk` | `hash_to_scalar_domain` | `crates/z00z_core/src/kdf.rs` | 564 | Same public API shape, same clearer replacement. | Update the crate re-export in `lib.rs` and all call sites. |
| 4 | fn | `derive_ctx` | `derive_domain_hash` | `crates/z00z_core/src/hash.rs` | 1098 | `ctx` is vague; `domain_hash` matches the length-prefixed domain/data derivation. | Update `pub use` in `lib.rs`, docs, and all call sites. |
| 5 | fn | `derive_k_dh` | `derive_dh_key` | `crates/z00z_core/src/ecdh.rs` | 239 | `k_dh` is audit-hostile shorthand; `dh_key` keeps the cryptographic meaning clear. | Update `ecdh_stealth.rs`, docs, tests, and any public re-exports. |
| 6 | enum | `EcdhErr` | `EcdhError` | `crates/z00z_core/src/ecdh_stealth.rs` | 22 | `Err` is an unnecessary abbreviation for a public error type. | Update imports, type references, and re-exports. |
| 7 | enum | `ValidationErr` | `ValidationError` | `crates/z00z_core/src/validation.rs` | 44 | `Err` is an unnecessary abbreviation for a public error enum. | Update imports, type references, and re-exports. |

## Test File Prefix Fixes

All test-only files in `tests/` and any `_tests.rs` / `_test.rs` files under `src/` must start with `test_`.

### High-confidence file renames

| Old File | New File | Why | Notes |
| --- | --- | --- | --- |
| `crates/z00z_core/src/assets/wire_tests.rs` | `crates/z00z_core/src/assets/test_wire.rs` | Test-only file suffix must become `test_` prefix. | Update the matching module declaration in `crates/z00z_core/src/assets/mod.rs`. |
| `crates/z00z_core/tests/assets/assets_tests.rs` | `crates/z00z_core/tests/assets/test_assets.rs` | Test-only integration file must start with `test_`. | If the file is listed from a module file, update that `mod` declaration as well. |
| `crates/z00z_core/tests/assets/metadata_limits_test.rs` | `crates/z00z_core/tests/assets/test_metadata_limits.rs` | Test-only integration file must start with `test_`. | Same note as above. |
| `crates/z00z_core/tests/assets/property_based_test.rs` | `crates/z00z_core/tests/assets/test_property_based.rs` | Test-only integration file must start with `test_`. | Same note as above. |
| `crates/z00z_core/tests/genesis/genesis_tests.rs` | `crates/z00z_core/tests/genesis/test_genesis.rs` | Test-only integration file must start with `test_`. | Update the matching module declaration in `crates/z00z_core/tests/genesis/mod.rs`. |

### Batch rule for the remaining integration files

Apply the same `test_` prefix rule to every other non-prefixed test-only file under:

- `crates/z00z_core/tests/assets/`
- `crates/z00z_core/tests/genesis/`

If a renamed file is pulled in via a `mod` declaration, update the corresponding `mod` line in the parent `mod.rs` file.

## Test Function Prefix Audit

Every `#[test]` function that does not already start with `test_` must be renamed to `test_<current_name>`.

### Source test files that need a prefix pass

| File | Required action | Notes |
|---|---|---|
| `crates/z00z_core/src/range_proofs.rs` | Prefix all non-`test_` test functions | Includes the proof, batch, and tamper tests. |
| `crates/z00z_core/src/commitments.rs` | Prefix all non-`test_` test functions | Includes the blind/opening/commitment tests. |
| `crates/z00z_core/src/ecdh.rs` | Prefix all non-`test_` test functions | Includes determinism and identity-rejection tests. |
| `crates/z00z_core/src/ecdh_stealth.rs` | Prefix all non-`test_` test functions | Keep already-prefixed tests unchanged. |
| `crates/z00z_core/src/aead.rs` | Prefix all non-`test_` test functions | Large test module; apply the prefix uniformly. |
| `crates/z00z_core/src/claim/verifier.rs` | Prefix all non-`test_` test functions | Includes all `verify_*` tests. |
| `crates/z00z_core/src/claim/proof.rs` | Prefix all non-`test_` test functions | Includes proof and signature roundtrip tests. |
| `crates/z00z_core/src/claim/statement.rs` | Prefix all non-`test_` test functions | Includes roundtrip, ordering, and hash-change tests. |
| `crates/z00z_core/src/claim/prover.rs` | Prefix all non-`test_` test functions | Includes claim proof and signature tests. |
| `crates/z00z_core/src/validation.rs` | Prefix all non-`test_` test functions | Includes point/scalar validation tests. |
| `crates/z00z_core/src/domains.rs` | Prefix all non-`test_` test functions | Includes domain uniqueness and domain-count tests. |
| `crates/z00z_core/src/hash.rs` | Prefix all non-`test_` test functions | Includes the hash, HMAC, and benchmark tests. |
| `crates/z00z_core/src/secret.rs` | Prefix all non-`test_` test functions | Includes secret-bytes and zeroization tests. |
| `crates/z00z_core/src/backend_tari.rs` | Prefix all non-`test_` test functions | Large backend test module; apply uniformly. |

### Representative examples from the source tree

| Current Name | Proposed Name | File |
|---|---|---|
| `proof_valid_value` | `test_proof_valid_value` | `crates/z00z_core/src/range_proofs.rs` |
| `blind_random_unique` | `test_blind_random_unique` | `crates/z00z_core/src/commitments.rs` |
| `validate_rejects_identity` | `test_validate_rejects_identity` | `crates/z00z_core/src/ecdh.rs` |
| `compat_sender_matches` | `test_compat_sender_matches` | `crates/z00z_core/src/ecdh_stealth.rs` |
| `domain_separation_changes_output` | `test_domain_separation_changes_output` | `crates/z00z_core/src/hash.rs` |
| `reject_oversized_point` | `test_reject_oversized_point` | `crates/z00z_core/src/validation.rs` |
| `test_secret_bytes_basic` | keep as-is | `crates/z00z_core/src/secret.rs` |
| `test_create_commitment_deterministic` | keep as-is | `crates/z00z_core/src/backend_tari.rs` |

## Module Re-export Follow-ups

After renaming the core public API functions and types, update these facades so the old names do not remain visible:

- `crates/z00z_core/src/lib.rs`
- `crates/z00z_core/src/hash.rs`
- `crates/z00z_core/src/kdf.rs`
- `crates/z00z_core/src/ecdh.rs`
- `crates/z00z_core/src/ecdh_stealth.rs`
- `crates/z00z_core/src/validation.rs`

## Suggested Apply Order

1. Rename the public API symbols first.
2. Update `pub use` re-exports and module facades.
3. Rename test-only files to satisfy the `test_` prefix rule.
4. Prefix all non-`test_` test functions.
5. Run the full verification gate and fix any fallout from the symbol renames.

---



# z00z_crypto Rename Plan

Scope: `crates/z00z_crypto/src/**/*.rs`
Excluded: `crates/z00z_crypto/tari/**`

This document is a Markdown rename plan only. It does not apply code changes.

## Summary

The highest-confidence rename candidates in this scope are public API names that are either cryptic abbreviations or ambiguous error/type suffixes. The test suite in this crate also contains many `#[test]` functions that do not start with `test_`; those are handled as a batch audit below.

## Rename Table

| #    | Kind | Current Name    | Proposed Name           | File                                     | Line | Why                                                          | Follow-up                                                    |
| ---- | ---- | --------------- | ----------------------- | ---------------------------------------- | ---- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| 1    | fn   | `h2s_zk`        | `hash_to_scalar_zk`     | `crates/z00z_crypto/src/hash.rs`         | 815  | `h2s` is cryptic; the new name spells out the operation while keeping the ZK context. | Update `pub use` re-exports in `hash.rs` and `lib.rs`, plus all call sites. |
| 2    | fn   | `h2scalar_zk`   | `hash_to_scalar_domain` | `crates/z00z_crypto/src/hash.rs`         | 918  | The wrapper takes raw domain bytes; `hash_to_scalar_domain` is explicit and still compact. | Update the deprecated wrapper note, re-exports, and all call sites. |
| 3    | fn   | `h2scalar_zk`   | `hash_to_scalar_domain` | `crates/z00z_crypto/src/kdf.rs`          | 564  | Same name, same meaning: explicit hash-to-scalar derivation from domain-tagged input. | Update the crate re-export in `lib.rs` and all call sites.   |
| 4    | fn   | `derive_ctx`    | `derive_domain_hash`    | `crates/z00z_crypto/src/hash.rs`         | 1098 | `ctx` is too vague; `domain_hash` matches the length-prefixed domain/data derivation. | Update the `pub use` in `lib.rs`, docs, and all call sites.  |
| 5    | fn   | `derive_k_dh`   | `derive_dh_key`         | `crates/z00z_crypto/src/ecdh.rs`         | 239  | `k_dh` is audit-hostile shorthand; `dh_key` keeps the cryptographic meaning clear. | Update `ecdh_stealth.rs`, docs, tests, and any public re-exports. |
| 6    | enum | `EcdhErr`       | `EcdhError`             | `crates/z00z_crypto/src/ecdh_stealth.rs` | 22   | `Err` is an unnecessary abbreviation for a public error type. | Update imports, type references, and re-exports.             |
| 7    | enum | `ValidationErr` | `ValidationError`       | `crates/z00z_crypto/src/validation.rs`   | 44   | `Err` is unnecessary in a public error enum and reads less cleanly than `Error`. | Update imports, type references, and re-exports.             |

## Batch Audit: `#[test]` Functions Without `test_` Prefix

All `#[test]` functions in the following files that do not already start with `test_` should be renamed to `test_<current_name>`.

| File                                        | Action                                | Notes                                                        |
| ------------------------------------------- | ------------------------------------- | ------------------------------------------------------------ |
| `crates/z00z_crypto/src/range_proofs.rs`    | Prefix all non-`test_` test functions | Rename every `#[test] fn` in this file to the `test_` form.  |
| `crates/z00z_crypto/src/commitments.rs`     | Prefix all non-`test_` test functions | Includes the `blind_*`, `opening_*`, and `commitment_*` cases. |
| `crates/z00z_crypto/src/ecdh.rs`            | Prefix all non-`test_` test functions | Includes the deterministic and identity-rejection tests.     |
| `crates/z00z_crypto/src/ecdh_stealth.rs`    | Prefix all non-`test_` test functions | Keep already-prefixed tests unchanged.                       |
| `crates/z00z_crypto/src/aead.rs`            | Prefix all non-`test_` test functions | Large test module; apply the prefix uniformly.               |
| `crates/z00z_crypto/src/claim/verifier.rs`  | Prefix all non-`test_` test functions | All `verify_*` tests need the prefix.                        |
| `crates/z00z_crypto/src/claim/proof.rs`     | Prefix all non-`test_` test functions | Includes the proof and signature roundtrip tests.            |
| `crates/z00z_crypto/src/claim/statement.rs` | Prefix all non-`test_` test functions | Includes roundtrip, ordering, and hash-change tests.         |
| `crates/z00z_crypto/src/claim/prover.rs`    | Prefix all non-`test_` test functions | Includes claim proof and signature tests.                    |
| `crates/z00z_crypto/src/validation.rs`      | Prefix all non-`test_` test functions | Includes point/scalar validation tests.                      |
| `crates/z00z_crypto/src/domains.rs`         | Prefix all non-`test_` test functions | Includes domain uniqueness and domain-count tests.           |
| `crates/z00z_crypto/src/hash.rs`            | Prefix all non-`test_` test functions | Includes the hash determinism, HMAC, and benchmark tests.    |

## Implementation Notes

- Public symbol renames require updating `pub use` re-exports in `crates/z00z_crypto/src/lib.rs` and any module-level re-exports that expose the old names.
- The compatibility wrapper in `ecdh_stealth.rs` should be updated after `derive_dh_key` lands so the compatibility layer keeps matching the canonical API.
- After renaming tests, keep the test bodies unchanged; this is a naming-only pass.

## Suggested Apply Order

1. Rename the public API symbols first.
2. Update re-exports in `lib.rs` and affected module facades.
3. Rename all non-`test_` test functions in the batch audit files.
4. Run the full verification gate and fix any fallout from the symbol renames.

---



# z00z_simulator Rename Plan

Scope: `crates/z00z_simulator/**/*.rs`
This document is a Markdown rename plan only. It does not apply code changes.

## Summary

The simulator crate is already mostly clean. I found a small set of high-confidence public API names with one clear issue: the `leafs` plural should be `leaves`, and several package helpers use `pkg` where the longer form reads better on the public surface. I did not find any test-only files or `#[test]` functions that violate the `test_` prefix rule.

## Rename Table

| #    | Kind | Current Name        | Proposed Name           | File                                              | Line | Why                                                          | Follow-up                                                    |
| ---- | ---- | ------------------- | ----------------------- | ------------------------------------------------- | ---: | ------------------------------------------------------------ | ------------------------------------------------------------ |
| 1    | fn   | `wrap_claim_pkgs`   | `wrap_claim_packages`   | `crates/z00z_simulator/src/claim_pkg_consumer.rs` |   42 | `pkg` is readable but the full noun is clearer for a public helper that serializes multiple claim packages. | Update the call sites in the consumer module and any external imports. |
| 2    | fn   | `verify_claim_pkgs` | `verify_claim_packages` | `crates/z00z_simulator/src/claim_pkg_consumer.rs` |  111 | Same package naming cleanup; this helper is part of the public consumer API. | Update callers in the consumer module and tests.             |
| 3    | fn   | `load_claim_pkgs`   | `load_claim_packages`   | `crates/z00z_simulator/src/claim_pkg_consumer.rs` |  126 | The longer form is easier to scan and matches the file content it loads. | Update the consumer call sites and any test imports.         |
| 4    | fn   | `claim_leafs`       | `claim_leaves`          | `crates/z00z_simulator/src/claim_pkg_consumer.rs` |  141 | `leafs` is an incorrect plural; `leaves` is the standard form and removes a small but real readability defect. | Update `load_claim_leafs` and any callers.                   |
| 5    | fn   | `load_claim_leafs`  | `load_claim_leaves`     | `crates/z00z_simulator/src/claim_pkg_consumer.rs` |  158 | This keeps the loader name aligned with the corrected plural. | Update the internal `claim_leafs(&packages)` call.           |

## Test Prefix Audit

All discovered test-only files in `crates/z00z_simulator/tests/` already start with `test_`, including support files under `tests/support/`. Inline `#[test]` functions sampled in `src/claim_pkg_consumer.rs` and `src/scenario_1/stage_6.rs` also already use the `test_` prefix, so no file-name or test-function prefix rename is required.

| File                                                         | Status    | Notes                                           |
| ------------------------------------------------------------ | --------- | ----------------------------------------------- |
| `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`      | compliant | Already uses the required `test_` prefix.       |
| `crates/z00z_simulator/tests/test_stage4_claim_gate.rs`      | compliant | Already uses the required `test_` prefix.       |
| `crates/z00z_simulator/tests/support/test_stage4_support.rs` | compliant | Support file already uses the required prefix.  |
| `crates/z00z_simulator/src/claim_pkg_consumer.rs`            | compliant | Inline tests are already prefixed with `test_`. |
| `crates/z00z_simulator/src/scenario_1/stage_6.rs`            | compliant | Inline helper test names already use `test_`.   |

## Module Follow-ups

- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
- `crates/z00z_simulator/src/claim_pkg_store.rs`

If the package helper names are renamed later, update the module-local call sites and any re-exports that surface them.

## Suggested Apply Order

1. Rename the package helper functions first.
2. Rename `claim_leafs` and `load_claim_leafs` to the corrected plurals.
3. Update the local call sites and any imports in tests or examples.
4. Re-run the simulator audit to confirm there are no remaining high-confidence candidates.

---



# z00z_storage Rename Plan

Scope: `crates/z00z_storage/**/*.rs`
Excluded: `crates/z00z_crypto/tari/**`

This document is a Markdown rename plan only. It does not apply code changes.

## Summary

The storage crate has a small set of high-confidence public API names that should be spelled out more clearly, plus a much larger set of test-only files that must be normalized to the `test_` prefix rule. Integration test files under `crates/z00z_storage/tests/` also need the `test_` prefix pass on their `#[test]` functions after the filename rename.

## Core Symbol Renames

| #    | Kind       | Old Name                   | New Name                         | File                                               | Line | Violation | Follow-up                                                    |
| ---- | ---------- | -------------------------- | -------------------------------- | -------------------------------------------------- | ---: | --------- | ------------------------------------------------------------ |
| 1    | type alias | `CheckResult`              | `CheckpointResult`               | `crates/z00z_storage/src/error.rs`                 |    4 | R1        | Update `lib.rs`, `checkpoint/*`, and all call sites that import the alias. |
| 2    | type alias | `SerResult`                | `SerializationResult`            | `crates/z00z_storage/src/error.rs`                 |    7 | R1        | Update `lib.rs`, `serialization/*`, and all call sites that import the alias. |
| 3    | fn         | `def_key`                  | `definition_key`                 | `crates/z00z_storage/src/assets/keys.rs`           |   10 | R1        | Update `assets/store.rs`, `assets/store_internal/*`, and all key helpers that call it. |
| 4    | fn         | `ser_key`                  | `serial_key`                     | `crates/z00z_storage/src/assets/keys.rs`           |   15 | R1        | Update `assets/store.rs`, `assets/store_internal/*`, and all key helpers that call it. |
| 5    | fn         | `derive_art_id`            | `derive_checkpoint_id`           | `crates/z00z_storage/src/checkpoint/ids.rs`        |  145 | R1        | Update `checkpoint/mod.rs`, `checkpoint/store.rs`, `assets/store_internal/redb_backend.rs`, and tests. |
| 6    | fn         | `derive_art_id_from_draft` | `reject_draft_for_checkpoint_id` | `crates/z00z_storage/src/checkpoint/ids.rs`        |  152 | R1        | Update the test module in `checkpoint/ids.rs` and any exported re-exports. |
| 7    | trait      | `TxProofChk`               | `TxProofVerifier`                | `crates/z00z_storage/src/checkpoint/build.rs`      |  203 | R1        | Update all call sites in `checkpoint/build.rs` and any implementors in tests. |
| 8    | trait      | `SpentIdx`                 | `SpentIndex`                     | `crates/z00z_storage/src/checkpoint/build.rs`      |  208 | R1        | Update all call sites in `checkpoint/build.rs` and any implementors in tests. |
| 9    | trait      | `MemberIdx`                | `MemberIndex`                    | `crates/z00z_storage/src/checkpoint/build.rs`      |  214 | R1        | Update all call sites in `checkpoint/build.rs` and any implementors in tests. |
| 10   | enum       | `TxProofErr`               | `TxProofError`                   | `crates/z00z_storage/src/checkpoint/build.rs`      |  138 | R1        | Update `checkpoint/mod.rs`, `checkpoint/build.rs`, and all proof verifier call sites. |
| 11   | enum       | `SpentIdxErr`              | `SpentIndexError`                | `crates/z00z_storage/src/checkpoint/build.rs`      |  147 | R1        | Update `checkpoint/mod.rs`, `checkpoint/build.rs`, and all spent-index call sites. |
| 12   | enum       | `StateErr`                 | `StateError`                     | `crates/z00z_storage/src/checkpoint/build.rs`      |  153 | R1        | Update `checkpoint/mod.rs`, `checkpoint/build.rs`, and all state resolution call sites. |
| 13   | struct     | `CheckpointVer`            | `CheckpointVersion`              | `crates/z00z_storage/src/checkpoint/artifact.rs`   |   18 | R1        | Update `checkpoint/mod.rs`, `checkpoint/codec.rs`, `checkpoint/store.rs`, and all examples/tests. |
| 14   | struct     | `CheckpointProofSys`       | `CheckpointProofSystem`          | `crates/z00z_storage/src/checkpoint/artifact.rs`   |  184 | R1        | Update `checkpoint/mod.rs`, `checkpoint/artifact.rs`, `checkpoint/codec.rs`, `checkpoint/store.rs`, and tests. |
| 15   | struct     | `CheckpointAuditVer`       | `CheckpointAuditVersion`         | `crates/z00z_storage/src/checkpoint/audit.rs`      |   24 | R1        | Update `checkpoint/mod.rs`, `checkpoint/audit.rs`, `checkpoint/codec.rs`, and tests. |
| 16   | struct     | `CheckpointLinkVer`        | `CheckpointLinkVersion`          | `crates/z00z_storage/src/checkpoint/link.rs`       |   18 | R1        | Update `checkpoint/mod.rs`, `checkpoint/link.rs`, `checkpoint/codec.rs`, and tests. |
| 17   | struct     | `CheckpointExecVer`        | `CheckpointExecVersion`          | `crates/z00z_storage/src/checkpoint/exec_input.rs` |   19 | R1        | Update `checkpoint/mod.rs`, `checkpoint/exec_input.rs`, `checkpoint/codec.rs`, and tests. |

## Test-Only File Renames

| #    | Kind | Old Name                         | New Name                              | File                                                         | Line | Violation | Follow-up                                                    |
| ---- | ---- | -------------------------------- | ------------------------------------- | ------------------------------------------------------------ | ---: | --------- | ------------------------------------------------------------ |
| 1    | file | `model_tests.rs`                 | `test_model.rs`                       | `crates/z00z_storage/src/assets/model_tests.rs`              |    — | R2        | Update `src/assets/mod.rs` to point at the renamed test module. |
| 2    | file | `whitebox_crud.rs`               | `test_whitebox_crud.rs`               | `crates/z00z_storage/src/assets/store_internal/whitebox_crud.rs` |    — | R2        | Update the `#[cfg(test)]` path in `src/assets/store.rs`.     |
| 3    | file | `whitebox_help.rs`               | `test_whitebox_help.rs`               | `crates/z00z_storage/src/assets/store_internal/whitebox_help.rs` |    — | R2        | Update the `#[cfg(test)]` path in `src/assets/store.rs`.     |
| 4    | file | `whitebox_paths.rs`              | `test_whitebox_paths.rs`              | `crates/z00z_storage/src/assets/store_internal/whitebox_paths.rs` |    — | R2        | Update the `#[cfg(test)]` path in `src/assets/store.rs`.     |
| 5    | file | `whitebox_proofs.rs`             | `test_whitebox_proofs.rs`             | `crates/z00z_storage/src/assets/store_internal/whitebox_proofs.rs` |    — | R2        | Update the `#[cfg(test)]` path in `src/assets/store.rs`.     |
| 6    | file | `whitebox_state.rs`              | `test_whitebox_state.rs`              | `crates/z00z_storage/src/assets/store_internal/whitebox_state.rs` |    — | R2        | Update the `#[cfg(test)]` path in `src/assets/store.rs`.     |
| 7    | file | `assets_suite.rs`                | `test_assets_suite.rs`                | `crates/z00z_storage/tests/assets_suite.rs`                  |    — | R2        | Update the suite `mod assets;` declaration to the renamed assets test module. |
| 8    | file | `mod.rs`                         | `test_assets.rs`                      | `crates/z00z_storage/tests/assets/mod.rs`                    |    — | R2        | Update the nested `mod store_api;` path in the renamed file and the suite module path. |
| 9    | file | `store_api.rs`                   | `test_store_api.rs`                   | `crates/z00z_storage/tests/assets/store_api.rs`              |    — | R2        | Update the parent assets test module to point at the renamed file. |
| 10   | file | `checkpoint_codec.rs`            | `test_checkpoint_codec.rs`            | `crates/z00z_storage/tests/checkpoint_codec.rs`              |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 11   | file | `checkpoint_draft_build.rs`      | `test_checkpoint_draft_build.rs`      | `crates/z00z_storage/tests/checkpoint_draft_build.rs`        |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 12   | file | `checkpoint_draft_final.rs`      | `test_checkpoint_draft_final.rs`      | `crates/z00z_storage/tests/checkpoint_draft_final.rs`        |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 13   | file | `checkpoint_finalization.rs`     | `test_checkpoint_finalization.rs`     | `crates/z00z_storage/tests/checkpoint_finalization.rs`       |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 14   | file | `checkpoint_ids.rs`              | `test_checkpoint_ids.rs`              | `crates/z00z_storage/tests/checkpoint_ids.rs`                |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 15   | file | `checkpoint_leaf_hash.rs`        | `test_checkpoint_leaf_hash.rs`        | `crates/z00z_storage/tests/checkpoint_leaf_hash.rs`          |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 16   | file | `checkpoint_link_injective.rs`   | `test_checkpoint_link_injective.rs`   | `crates/z00z_storage/tests/checkpoint_link_injective.rs`     |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 17   | file | `checkpoint_replay_inputs.rs`    | `test_checkpoint_replay_inputs.rs`    | `crates/z00z_storage/tests/checkpoint_replay_inputs.rs`      |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 18   | file | `checkpoint_root_binding.rs`     | `test_checkpoint_root_binding.rs`     | `crates/z00z_storage/tests/checkpoint_root_binding.rs`       |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 19   | file | `checkpoint_store_api.rs`        | `test_checkpoint_store_api.rs`        | `crates/z00z_storage/tests/checkpoint_store_api.rs`          |    — | R2        | Update the `#[path = "checkpoint/test_fixtures.rs"] mod fixtures;` declaration after the rename. |
| 20   | file | `fixtures.rs`                    | `test_fixtures.rs`                    | `crates/z00z_storage/tests/checkpoint/fixtures.rs`           |    — | R2        | Update `tests/checkpoint_store_api.rs` to point at the renamed fixture module. |
| 21   | file | `redb_mutation.rs`               | `test_redb_mutation.rs`               | `crates/z00z_storage/tests/redb_mutation.rs`                 |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 22   | file | `redb_rehydrate.rs`              | `test_redb_rehydrate.rs`              | `crates/z00z_storage/tests/redb_rehydrate.rs`                |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 23   | file | `search_api.rs`                  | `test_search_api.rs`                  | `crates/z00z_storage/tests/search_api.rs`                    |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 24   | file | `serialization_determinism.rs`   | `test_serialization_determinism.rs`   | `crates/z00z_storage/tests/serialization_determinism.rs`     |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 25   | file | `serialization_restore.rs`       | `test_serialization_restore.rs`       | `crates/z00z_storage/tests/serialization_restore.rs`         |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 26   | file | `serialization_roundtrip.rs`     | `test_serialization_roundtrip.rs`     | `crates/z00z_storage/tests/serialization_roundtrip.rs`       |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 27   | file | `serialization_visualization.rs` | `test_serialization_visualization.rs` | `crates/z00z_storage/tests/serialization_visualization.rs`   |    — | R2        | Rename the integration test file and then prefix any remaining non-`test_` test functions inside it. |
| 28   | file | `snapshot_suite.rs`              | `test_snapshot_suite.rs`              | `crates/z00z_storage/tests/snapshot_suite.rs`                |    — | R2        | Update the suite `mod snapshot;` declaration to the renamed snapshot test module. |
| 29   | file | `mod.rs`                         | `test_snapshot.rs`                    | `crates/z00z_storage/tests/snapshot/mod.rs`                  |    — | R2        | Update the nested snapshot child module paths inside the renamed file and the suite module path. |
| 30   | file | `fix.rs`                         | `test_fix.rs`                         | `crates/z00z_storage/tests/snapshot/fix.rs`                  |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 31   | file | `ids.rs`                         | `test_ids.rs`                         | `crates/z00z_storage/tests/snapshot/ids.rs`                  |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 32   | file | `leaf_hash.rs`                   | `test_leaf_hash.rs`                   | `crates/z00z_storage/tests/snapshot/leaf_hash.rs`            |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 33   | file | `ordering.rs`                    | `test_ordering.rs`                    | `crates/z00z_storage/tests/snapshot/ordering.rs`             |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 34   | file | `path_bind.rs`                   | `test_path_bind.rs`                   | `crates/z00z_storage/tests/snapshot/path_bind.rs`            |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 35   | file | `persist.rs`                     | `test_persist.rs`                     | `crates/z00z_storage/tests/snapshot/persist.rs`              |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 36   | file | `replay_bound.rs`                | `test_replay_bound.rs`                | `crates/z00z_storage/tests/snapshot/replay_bound.rs`         |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 37   | file | `root_bind.rs`                   | `test_root_bind.rs`                   | `crates/z00z_storage/tests/snapshot/root_bind.rs`            |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 38   | file | `versions.rs`                    | `test_versions.rs`                    | `crates/z00z_storage/tests/snapshot/versions.rs`             |    — | R2        | Update the parent snapshot test module to point at the renamed file. |
| 39   | file | `wit_decode.rs`                  | `test_wit_decode.rs`                  | `crates/z00z_storage/tests/snapshot/wit_decode.rs`           |    — | R2        | Update the parent snapshot test module to point at the renamed file. |

## Test Function Prefix Audit

All `#[test]` functions in the renamed integration files above should be prefixed with `test_` as part of the same migration. The source-level test modules below still contain non-prefixed test functions and need a direct rename pass.

| File                                                       | Required action                       | Notes                                                        |
| ---------------------------------------------------------- | ------------------------------------- | ------------------------------------------------------------ |
| `crates/z00z_storage/src/checkpoint/artifact.rs`           | Prefix all non-`test_` test functions | Includes version, proof-system, and finalization tests.      |
| `crates/z00z_storage/src/checkpoint/audit.rs`              | Prefix all non-`test_` test functions | Includes audit-version and constructor tests.                |
| `crates/z00z_storage/src/checkpoint/codec.rs`              | Prefix all non-`test_` test functions | Includes codec roundtrip and malformed-payload tests.        |
| `crates/z00z_storage/src/checkpoint/exec_input.rs`         | Prefix all non-`test_` test functions | Includes execution-input version and validation tests.       |
| `crates/z00z_storage/src/checkpoint/ids.rs`                | Prefix all non-`test_` test functions | Includes draft/id derivation and rejection tests.            |
| `crates/z00z_storage/src/checkpoint/link.rs`               | Prefix all non-`test_` test functions | Includes link-version and constructor tests.                 |
| `crates/z00z_storage/src/checkpoint/store.rs`              | Prefix all non-`test_` test functions | Includes load/save and key-check tests.                      |
| `crates/z00z_storage/src/assets/model_tests.rs`            | Prefix all non-`test_` test functions | Test-only module should be renamed and the functions should be prefixed together. |
| `crates/z00z_storage/src/assets/store_internal/tree_id.rs` | Prefix all non-`test_` test functions | Includes namespace uniqueness checks.                        |
| `crates/z00z_storage/src/serialization/artifact.rs`        | Prefix all non-`test_` test functions | Includes artifact version and contract tests.                |

## Suggested Apply Order

1. Rename the public API symbols first.
2. Rename the test-only files under `src/` and `tests/`.
3. Update the affected module path declarations in the parent files.
4. Prefix the remaining non-`test_` test functions in the source test modules.
5. Re-run the storage audit and the full verification gate.

## Summary Counts

- Files scanned: 81
- Symbols inventoried: 55 high-confidence items
- Rename proposals: 55  (breakdown: R1=17, R2=38, R3=0, R4=0, R5=0)
- INFO notes: 0
- Files to rename: 38

----



# z00z_utils Rename Plan

Scope: `crates/z00z_utils/src/**/*.rs` and `crates/z00z_utils/tests/**/*.rs`

This document is a Markdown rename plan only. It does not apply code changes.

## Test File Prefix Fixes

All test-only files in `tests/` and any `tests.rs` files under `src/` must start with `test_`.

### High-confidence file renames

| Old File                                              | New File                                                   | Why                                                 | Notes                                                        |
| ----------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------ |
| `crates/z00z_utils/src/compression/tests.rs`          | `crates/z00z_utils/src/compression/test_compression.rs`    | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/compression.rs` at line 268. |
| `crates/z00z_utils/src/codec/tests.rs`                | `crates/z00z_utils/src/codec/test_codec.rs`                | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/codec/mod.rs` at line 22. |
| `crates/z00z_utils/src/config/tests.rs`               | `crates/z00z_utils/src/config/test_config.rs`              | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/config/mod.rs` at line 53. |
| `crates/z00z_utils/src/logger/tests.rs`               | `crates/z00z_utils/src/logger/test_logger.rs`              | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/logger/mod.rs` at line 37. |
| `crates/z00z_utils/src/metrics/tests.rs`              | `crates/z00z_utils/src/metrics/test_metrics.rs`            | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/metrics/mod.rs` at line 27. |
| `crates/z00z_utils/src/rng/tests.rs`                  | `crates/z00z_utils/src/rng/test_rng.rs`                    | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/rng/mod.rs` at line 33. |
| `crates/z00z_utils/src/time/tests.rs`                 | `crates/z00z_utils/src/time/test_time.rs`                  | Test-only file name must start with `test_`.        | Update the matching `mod` declaration in `crates/z00z_utils/src/time/mod.rs` at line 49. |
| `crates/z00z_utils/tests/io_integration.rs`           | `crates/z00z_utils/tests/test_io_integration.rs`           | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |
| `crates/z00z_utils/tests/time_policy_micros.rs`       | `crates/z00z_utils/tests/test_time_policy_micros.rs`       | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |
| `crates/z00z_utils/tests/thread_safety.rs`            | `crates/z00z_utils/tests/test_thread_safety.rs`            | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |
| `crates/z00z_utils/tests/codec_integration.rs`        | `crates/z00z_utils/tests/test_codec_integration.rs`        | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |
| `crates/z00z_utils/tests/logger_integration.rs`       | `crates/z00z_utils/tests/test_logger_integration.rs`       | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |
| `crates/z00z_utils/tests/os_hardening_integration.rs` | `crates/z00z_utils/tests/test_os_hardening_integration.rs` | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |
| `crates/z00z_utils/tests/config_integration.rs`       | `crates/z00z_utils/tests/test_config_integration.rs`       | Test-only integration file must start with `test_`. | No parent `mod` declaration is used here.                    |

## Test Function Prefix Audit

Every `#[test]` function that does not already start with `test_` must be renamed to `test_<current_name>`.

### Source test functions that need a prefix pass

| File                                                   | Current Name                     | Proposed Name                         | Why                                         |
| ------------------------------------------------------ | -------------------------------- | ------------------------------------- | ------------------------------------------- |
| `crates/z00z_utils/src/os_hardening.rs`                | `api_contract_apply_best_effort` | `test_api_contract_apply_best_effort` | Test function name must start with `test_`. |
| `crates/z00z_utils/src/os_hardening.rs`                | `api_contract_lock_bytes`        | `test_api_contract_lock_bytes`        | Test function name must start with `test_`. |
| `crates/z00z_utils/src/os_hardening.rs`                | `lock_bytes_lifetime`            | `test_lock_bytes_lifetime`            | Test function name must start with `test_`. |
| `crates/z00z_utils/src/os_hardening.rs`                | `lock_bytes_zero_on_drop`        | `test_lock_bytes_zero_on_drop`        | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/file_logger.rs`          | `file_mode`                      | `test_file_mode`                      | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/file_logger.rs`          | `msg_sanitize`                   | `test_msg_sanitize`                   | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/file_logger.rs`          | `symlink_reject`                 | `test_symlink_reject`                 | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/rotating_file_logger.rs` | `rotates_and_keeps_files`        | `test_rotates_and_keeps_files`        | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/rotating_file_logger.rs` | `log_file_mode`                  | `test_log_file_mode`                  | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/rotating_file_logger.rs` | `log_msg_sanitize`               | `test_log_msg_sanitize`               | Test function name must start with `test_`. |
| `crates/z00z_utils/src/logger/rotating_file_logger.rs` | `log_symlink_reject`             | `test_log_symlink_reject`             | Test function name must start with `test_`. |

### Representative compliant files

These files already use `test_`-prefixed test names in the inspected blocks and do not need function renames:

- `crates/z00z_utils/src/compression/tests.rs`
- `crates/z00z_utils/src/codec/tests.rs`
- `crates/z00z_utils/src/config/tests.rs`
- `crates/z00z_utils/src/logger/tests.rs`
- `crates/z00z_utils/src/metrics/tests.rs`
- `crates/z00z_utils/src/rng/tests.rs`
- `crates/z00z_utils/src/time/tests.rs`

## Suggested Apply Order

1. Rename the test-only files under `src/` and `tests/`.
2. Update the `mod tests;` declarations in the parent modules that now need `mod test_*;`.
3. Rename the non-prefixed `#[test]` functions in `os_hardening.rs` and the logger implementations.
4. Re-run the rename audit to confirm no test-only names are left behind.

----



# z00z_wallets Rename Plan

Scope: `crates/z00z_wallets/src/**/*.rs` and `crates/z00z_wallets/tests/**/*.rs`

This document is a Markdown rename plan only. It does not apply code changes.

## Test File Prefix Fixes

All test-only files in `tests/` and any `tests.rs` / `*_tests.rs` files under `src/` must start with `test_`.

### High-confidence file renames

| #    | Kind | Old Name                   | File                                                         | Line | New Name                       | Violation      | Rationale                                                    |
| ---- | ---- | -------------------------- | ------------------------------------------------------------ | ---: | ------------------------------ | -------------- | ------------------------------------------------------------ |
| 1    | file | `tests.rs`                 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs` |    — | `test_key_impl.rs`             | R2 file prefix | Test-only file name must start with `test_`; rename keeps the suite local to `key_impl`. |
| 2    | file | `output_tests.rs`          | `crates/z00z_wallets/src/core/stealth/output_tests.rs`       |    — | `test_output.rs`               | R2 file prefix | Test-only file name must start with `test_`; `output` is the canonical feature name. |
| 3    | file | `output_tests_extra.rs`    | `crates/z00z_wallets/src/core/stealth/output_tests_extra.rs` |    — | `test_output_extra.rs`         | R2 file prefix | Test-only file name must start with `test_`; keep the secondary suite under the same topic. |
| 4    | file | `nullifier_store_tests.rs` | `crates/z00z_wallets/src/core/claim/nullifier_store_tests.rs` |    — | `test_nullifier_store.rs`      | R2 file prefix | Test-only file name must start with `test_`; the name should describe the store under test. |
| 5    | file | `claim_tx_tests.rs`        | `crates/z00z_wallets/src/core/tx/claim_tx_tests.rs`          |    — | `test_claim_tx.rs`             | R2 file prefix | Test-only file name must start with `test_`; this suite targets claim transaction verification. |
| 6    | file | `stealth_request_tests.rs` | `crates/z00z_wallets/src/core/address/stealth_request_tests.rs` |    — | `test_stealth_request.rs`      | R2 file prefix | Test-only file name must start with `test_`; keep the request suite aligned with the runtime module. |
| 7    | file | `stealth_card_tests.rs`    | `crates/z00z_wallets/src/core/address/stealth_card_tests.rs` |    — | `test_stealth_card.rs`         | R2 file prefix | Test-only file name must start with `test_`; keep the card suite aligned with the runtime module. |
| 8    | file | `tests.rs`                 | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |    — | `test_stealth_scanner.rs`      | R2 file prefix | Test-only file name must start with `test_`; this keeps the scanner suite easy to discover. |
| 9    | file | `proof_blob_fix.rs`        | `crates/z00z_wallets/tests/proof_blob_fix.rs`                |    — | `test_proof_blob_fix.rs`       | R2 file prefix | Integration test file under `tests/` must start with `test_`. |
| 10   | file | `stealth_scan_support.rs`  | `crates/z00z_wallets/tests/stealth_scan_support.rs`          |    — | `test_stealth_scan_support.rs` | R2 file prefix | Integration test file under `tests/` must start with `test_`. |

### Parent module follow-ups

| #    | Kind     | Old Name                   | File                                                       | Line | New Name                  | Violation      | Rationale                                                    |
| ---- | -------- | -------------------------- | ---------------------------------------------------------- | ---: | ------------------------- | -------------- | ------------------------------------------------------------ |
| 11   | mod decl | `tests.rs`                 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs` |   21 | `test_key_impl.rs`        | R2 file prefix | Update the `#[path]` declaration so it points at `test_key_impl.rs`. |
| 12   | mod decl | `output_tests.rs`          | `crates/z00z_wallets/src/core/stealth/output.rs`           |  520 | `test_output.rs`          | R2 file prefix | Update the `#[path]` declaration so it points at `test_output.rs`. |
| 13   | mod decl | `output_tests_extra.rs`    | `crates/z00z_wallets/src/core/stealth/output_tests.rs`     |  545 | `test_output_extra.rs`    | R2 file prefix | Update the nested `#[path]` declaration after the first rename lands. |
| 14   | mod decl | `nullifier_store_tests.rs` | `crates/z00z_wallets/src/core/claim/nullifier_store.rs`    |  424 | `test_nullifier_store.rs` | R2 file prefix | Update the `#[path]` declaration so it points at `test_nullifier_store.rs`. |
| 15   | mod decl | `claim_tx_tests.rs`        | `crates/z00z_wallets/src/core/tx/claim_tx.rs`              | 1196 | `test_claim_tx.rs`        | R2 file prefix | Update the `#[path]` declaration and the sibling `mod` name together. |
| 16   | mod decl | `stealth_request_tests.rs` | `crates/z00z_wallets/src/core/address/stealth_request.rs`  |  578 | `test_stealth_request.rs` | R2 file prefix | Update the `#[path]` declaration so it points at `test_stealth_request.rs`. |
| 17   | mod decl | `stealth_card_tests.rs`    | `crates/z00z_wallets/src/core/address/stealth_card.rs`     |  483 | `test_stealth_card.rs`    | R2 file prefix | Update the `#[path]` declaration so it points at `test_stealth_card.rs`. |
| 18   | mod decl | `stealth_scanner/tests.rs` | `crates/z00z_wallets/src/core/address/stealth_scanner.rs`  |  390 | `test_stealth_scanner.rs` | R2 file prefix | Update the `#[path]` declaration so it points at `test_stealth_scanner.rs`. |

## Concrete Test Function Prefix Audit

Every `#[test]`, `#[tokio::test]`, and `#[async_std::test]` function that does not already start with `test_` must be renamed to `test_<current_name>`.

### High-confidence symbol renames

| #    | Kind    | Old Name                       | File                                                         | Line | New Name                            | Violation        | Rationale                                                    |
| ---- | ------- | ------------------------------ | ------------------------------------------------------------ | ---: | ----------------------------------- | ---------------- | ------------------------------------------------------------ |
| 19   | test fn | `def_id_as_asset_fails`        | `crates/z00z_wallets/src/core/tx/claim_tx_tests.rs`          |  252 | `test_def_id_as_asset_fails`        | R2 missing test_ | Test name is already precise; only the required prefix is missing. |
| 20   | test fn | `scan_falls_back`              | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |   18 | `test_scan_falls_back`              | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 21   | test fn | `scan_rejects_partial`         | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |   55 | `test_scan_rejects_partial`         | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 22   | test fn | `scan_ignores_cache_false_hit` | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |   67 | `test_scan_ignores_cache_false_hit` | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 23   | test fn | `recv_status_map`              | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  104 | `test_recv_status_map`              | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 24   | test fn | `recv_status_code_map`         | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  135 | `test_recv_status_code_map`         | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 25   | test fn | `recv_report_map`              | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  145 | `test_recv_report_map`              | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 26   | test fn | `recv_reject_map`              | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  188 | `test_recv_reject_map`              | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 27   | test fn | `scan_skips_bad_ctx`           | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  218 | `test_scan_skips_bad_ctx`           | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 28   | test fn | `scan_tag_stale_req_ctx`       | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  261 | `test_scan_tag_stale_req_ctx`       | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 29   | test fn | `scan_range_resume_ok`         | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  307 | `test_scan_range_resume_ok`         | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |
| 30   | test fn | `scan_range_bad_cursor`        | `crates/z00z_wallets/src/core/address/stealth_scanner/tests.rs` |  341 | `test_scan_range_bad_cursor`        | R2 missing test_ | Prefix missing; the current name is otherwise clear.         |

## Remaining Suites Requiring a Prefix Pass

The scan also found additional non-`test_` test functions in larger suites. Keep the same `test_` prefix rule and rename them uniformly in these files:

- `crates/z00z_wallets/src/services/wallet_paths.rs`
- `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/network_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`
- `crates/z00z_wallets/tests/test_stealth_reference_bridge.rs`
- `crates/z00z_wallets/tests/test_rpc_persistence_spec_e3.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`
- `crates/z00z_wallets/tests/test_claim_state_compat.rs`
- `crates/z00z_wallets/tests/test_tx_tamper.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_configured_path.rs`
- `crates/z00z_wallets/tests/test_create_wallet_crypto_e2e.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_request_id.rs`
- `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs`
- `crates/z00z_wallets/tests/test_key_manager.rs`
- `crates/z00z_wallets/tests/test_rpc_scenarios_minimal.rs`
- `crates/z00z_wallets/tests/test_claim_import_reason_codes.rs`
- `crates/z00z_wallets/tests/test_tx_wrong_root.rs`
- `crates/z00z_wallets/tests/test_claim_core_safety.rs`
- `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
- `crates/z00z_wallets/tests/test_phase24_gate.rs`
- `crates/z00z_wallets/tests/test_addr_rate_limit_integration.rs`
- `crates/z00z_wallets/tests/test_app_service_create_wallet.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_file_sink.rs`
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
- `crates/z00z_wallets/tests/test_claim_state_core.rs`
- `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs`
- `crates/z00z_wallets/tests/test_phase12_dual_keys.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_replay_audit.rs`
- `crates/z00z_wallets/tests/test_phase15_regress.rs`
- `crates/z00z_wallets/tests/test_tx_roundtrip.rs`
- `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs`
- `crates/z00z_wallets/tests/test_wlt_validator.rs`
- `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs`
- `crates/z00z_wallets/tests/test_tx_store_integration.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_e2e_print.rs`

## Suggested Apply Order

1. Rename the test-only files under `src/` and `tests/`.
2. Update the `#[path]` declarations in the parent modules that now need `test_*` file names.
3. Prefix the concrete non-`test_` `#[test]` / `#[tokio::test]` functions listed above.
4. Run the rename audit again for the remaining suites listed in the batch section.

## Summary

- Files scanned: targeted `z00z_wallets` source and integration Rust files
- Symbols inventoried: 30 direct rename targets plus batch-listed suites
- Rename proposals: 30 direct rows
- INFO notes: 0
- Files to rename: 10



---



