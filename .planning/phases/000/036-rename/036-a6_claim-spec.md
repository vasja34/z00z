# 036 A6 Claim Rename Spec

This file records the exact claim-lane replacements requested for Phase 036.
It removes `ClaimRootVer`, renames the claim module away from `v2`, and updates
every live `claim-v2` human-facing string and test reference to the new claim
contract wording.

## Rename Table

| # | File | Line(s) | Old | New / Action | Notes | Comments |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `crates/z00z_crypto/src/claim/v2.rs` | file | `v2.rs` | `claim_contract.rs` | Rename the claim implementation module file. |  |
| 2 | `crates/z00z_crypto/tests/test_claim_v2_contract.rs` | file | `test_claim_v2_contract.rs` | `test_claim_contract.rs` | Rename the test-only file to match the new contract name. |  |
| 3 | `crates/z00z_crypto/src/claim/mod.rs` | 1-4 | `mod v2;` / `v2::{...}` | `mod claim_contract;` / `claim_contract::{...}` | Update the module declaration and re-export path after the file rename. |  |
| 4 | `crates/z00z_crypto/src/lib.rs` | 114 | `ClaimRootVer` | remove from export list; export `CLAIM_ROOT_VERSION` instead | Public facade must stop exporting the deleted type. |  |
| 5 | `crates/z00z_crypto/src/claim/v2.rs` | 17-29 | `ClaimRootVer` type, `V1`, `V2`, `new`, `as_u8` | delete; replace with `pub const CLAIM_ROOT_VERSION: u8 = 2` | Remove the dead wrapper type entirely. |  |
| 6 | `crates/z00z_crypto/src/claim/v2.rs` | 60-80 | `claim-v2 ...` error strings and `BadRootVer` / `RootVerMix` | `claim contract ...`; `BadRootVersion` / `RootVersionMismatch` | Keep the claim-proof / tx-version errors as they are unless their own names are touched separately. |  |
| 7 | `crates/z00z_crypto/src/claim/v2.rs` | 91-321 | `root_ver` field / getter / setter sites | `root_version` | Rename the root-version field and its accessor on `ClaimStmt`, `ClaimSourceRoot`, and `ClaimSourceProof`. |  |
| 8 | `crates/z00z_crypto/src/claim/v2.rs` | 146-201 | `ClaimRootVer::V1/::V2` checks | `CLAIM_ROOT_VERSION` for valid cases; `3u8` (or another non-2 nonzero byte) for mismatch fixtures | Replace the enum-based root-version handling with a raw-byte constant and explicit mismatch fixtures. |  |
| 9 | `crates/z00z_crypto/src/claim/v2.rs` | 355 | `"claim_v2"` | `"claim_contract"` | Deliberate domain-separator migration; this is compatibility-breaking and must be treated as part of the claim contract rename. |  |
| 10 | `crates/z00z_wallets/src/core/tx/claim_auth.rs` | 39, 69 | `claim_v2` in comments | `claim contract` | Human-facing comment cleanup only. |  |
| 11 | `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs` | 15, 33 | `ClaimRootVer::V1.as_u8()` / `claim_v2` in comment | `CLAIM_ROOT_VERSION` / `claim contract` | Helper and comment must follow the new claim root version surface. |  |
| 12 | `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` | 93, 105, 163-165 | `claim_v2`, `ClaimRootVer::V1`, `SourceRootVer` | `claim contract`, `CLAIM_ROOT_VERSION`, `SourceRootVersion` | Update the proof verifier and its error path. |  |
| 13 | `crates/z00z_wallets/src/core/tx/claim_errors.rs` | 26 | `SourceRootVer` | `SourceRootVersion` | Keep wallet error naming consistent with the new root-version spelling. |  |
| 14 | `crates/z00z_wallets/src/core/tx/claim_tx.rs` | 192 | `SourceRootVer(_)` | `SourceRootVersion(_)` | Update the verifier error-class match arm. |  |
| 15 | `crates/z00z_storage/src/assets/store_internal/store_query.rs` | 1, 194, 206 | `ClaimRootVer` import/usages | `CLAIM_ROOT_VERSION` / `root_version` | Update storage-side claim-source root construction and checks. |  |
| 16 | `crates/z00z_storage/src/checkpoint/build.rs` | 22, 344 | `ClaimRootVer::V1` | `CLAIM_ROOT_VERSION` | Storage checkpoint builder should use the single live claim root version constant. |  |
| 17 | `crates/z00z_storage/src/assets/types_identity.rs` | 1, 121, 127, 132 | `ClaimRootVer` import/field/ctor/getter | `CLAIM_ROOT_VERSION` / `root_version` | Storage-side identity wrapper should stop carrying the dead type. |  |
| 18 | `crates/z00z_crypto/tests/test_claim_v2_contract.rs` | 1, 16, 51, 67, 70, 80, 82, 88, 170, 177, 184-192 | file name, `claim-v2` expectations, `ClaimRootVer::V1/::V2` | `test_claim_contract.rs`, `claim contract`, `CLAIM_ROOT_VERSION` / `3u8` in mismatch fixtures | Update the test to the new contract name and root-version constant. |  |
| 19 | `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` | 4, 229-230, 301, 326, 343, 349, 371 | `ClaimRootVer::V1/::V2` and `root_ver()` expectations | `CLAIM_ROOT_VERSION` / `root_version` | Simulator tests must follow the same root-version simplification. |  |
| 20 | `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` | 6, 333, 467, 592, 639 | `ClaimRootVer::V1/::V2` and `ClaimSourceProof::new(...)` | `CLAIM_ROOT_VERSION` / `root_version` | Wallet claim tests need the same root-version migration. |  |
| 21 | `crates/z00z_storage/tests/test_claim_source_proof.rs` | 1, 34, 35, 48, 50, 55, 63, 80, 88, 96, 99, 113 | `ClaimRootVer::V1` and `root_ver()` checks | `CLAIM_ROOT_VERSION` / `root_version` | Storage proof tests must be updated to the new root-version naming. |  |

## Summary

- File renames: 2
- Module/export updates: 2
- Core claim API replacements: 6
- Wallet/storage call-site updates: 6
- Test updates: 5

## Coverage Proof

Live `crates/**/*.rs` hits for `claim-v2` / `claim_v2` map to the rows above as follows:

| Live hit group | Covered row(s) | Notes |
| --- | --- | --- |
| `claim-v2` error literals in `crates/z00z_crypto/src/claim/v2.rs` | 6 | Covers all human-facing error strings in the claim module. |
| `claim_v2` domain label in `crates/z00z_crypto/src/claim/v2.rs` | 9 | Covers the hash domain separator migration. |
| `claim_v2` comments in `crates/z00z_wallets/src/core/tx/claim_auth.rs` | 10 | Covers both auth comments. |
| `claim_v2` comment and root-version helper in `crates/z00z_wallets/src/core/tx/claim_tx_helpers.rs` | 11 | Covers the helper and its comment. |
| `claim_v2` string and version checks in `crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs` | 12-14 | Covers the live proof-verifier string and its root-version error path. |
| `claim-v2` expectations in `crates/z00z_crypto/tests/test_claim_contract.rs` | 18 | Covers the test file rename plus all quoted `claim-v2` expectations. |
| `ClaimRootVer::V1/::V2` call sites in storage, simulator, wallets, and tests | 15-21 | Covers every live root-version caller that still anchors the claim contract to the deleted type. |

Archive and planning artifacts under `.planning/**` and `logs/**` still contain historical `claim_v2` mentions. They are intentionally not part of this rename wave unless you want a separate documentation cleanup pass.
