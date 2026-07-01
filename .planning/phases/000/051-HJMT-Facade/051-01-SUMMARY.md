---
phase: 051-HJMT-Facade
plan: 1
status: complete
completed_at: 2026-05-28
next_plan: 051-02
---

# Phase 051-01 Summary

## Completed Scope

`051-01` is complete for the storage facade and compatibility backend slice.
`z00z_storage::assets` now exposes one storage-owned `AssetTreeBackend`
semantic contract, with `AssetStore` remaining the caller-facing facade and the
current shared namespaced JMT implementation explicitly wrapped as
`CompatibilityBackend`.

The compatibility path reuses the existing semantic model, RedB reload path,
path index, checkpoint execution commit path, claim-source proof issuer, and
proof-envelope helpers. It does not create a forest backend, duplicate verifier,
duplicate checkpoint authority, duplicate path-index authority, or a parallel
semantic model.

## Files Changed

- `crates/z00z_storage/src/assets/store.rs`
- `crates/z00z_storage/src/assets/mod.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
- `crates/z00z_storage/tests/assets/test_assets.rs`
- `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
- `.planning/phases/051-HJMT-Facade/051-CONTEXT.md`

## Boundary Kept Intact

- Public `AssetStore` methods remain available and delegate through the
  compatibility backend.
- The public backend contract uses semantic storage types only:
  `AssetPath`, `StoreItem`, `StoreOp`, `AssetStateRoot`, `CheckRoot`,
  `ProofItem`, `ProofBlob`, `ProofScanOut`, and `ClaimSourceProof`.
- `TreeId`, namespace bytes, raw JMT keys, branch ordering, and backend-root
  authority stay out of the public backend facade.
- Stale suffixed claim-source proof wording was removed from active
  planning/docs terminology; the live documented and coded contract is
  `ClaimSourceProof`.

## Review Passes

- Pass 1: Found that public inherent `AssetStore` methods still owned direct
  implementation bodies instead of consistently routing through the
  compatibility backend. Fixed by moving implementation bodies into
  `compat_*` helpers and making public methods delegate through
  `CompatibilityBackend`.
- Pass 2: Found that `RootApi::asset_root` still bypassed the compatibility
  adapter. Fixed by routing `RootApi` through `CompatibilityBackend::root`.
- Pass 3: Rechecked the backend facade, compatibility adapter, public method
  delegation, and physical-layout leak guards. No significant issues remained.
- Pass 4: Rechecked the same source boundaries plus diff hygiene after the
  terminology sync. No significant issues remained.

Two consecutive clean review passes were achieved on passes 3 and 4.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final Rust tree after the last code change.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump test_asset_tree_backend -- --nocapture`
  passed, including both backend facade contract tests.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed across the workspace, including doc-tests.
- `cargo fmt --check` exited 0; rustfmt printed the repository's existing
  stable-toolchain warnings for nightly-only config keys.
- `git diff --check` passed after the final docs terminology update.
