---
phase: 052-HJMT-Backend
plan: 052-01
status: complete
completed_at: 2026-05-28
next_plan: 052-02
requirements:
  - PH52-BACKEND-MODE
  - PH52-BUCKET-POLICY
summary_artifact_for: .planning/phases/052-HJMT-Backend/052-01-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 052-01 Summary: Backend Mode And Fixed Bucket Policy

## Completed Scope

`052-01` is complete for backend-mode selection, the fail-closed forest
skeleton, and the fixed bucket metadata contract required before physical
forest commit work starts.

`AssetStore` now owns explicit backend-mode routing for compatibility, forest,
and dual-verify modes. Compatibility remains the default for normal store
construction and load paths. Forest and dual-verify modes are present only
behind the Phase 051 facade and return explicit unsupported-backend errors for
in-scope operations until later plans land real forest behavior.

The fixed bucket contract is also present: `BucketId`, `BucketPolicy`,
`BucketPolicyError`, and `BucketRootLeaf` define deterministic bucket
derivation, verifier-visible policy identity, stable byte encodings, and
fail-closed decode validation. Bucket metadata is available for future proof
verification, but no public put, delete, lookup, list, checkpoint, or normal
asset API accepts bucket ids as caller authority.

## Files Changed

- `crates/z00z_storage/src/assets/store.rs`
- `crates/z00z_storage/src/assets/mod.rs`
- `crates/z00z_storage/src/assets/types.rs`
- `crates/z00z_storage/src/assets/types_identity.rs`
- `crates/z00z_storage/src/assets/types_record.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_config.rs`
- `crates/z00z_storage/src/assets/store_internal/forest_policy.rs`
- `crates/z00z_storage/src/assets/store_internal/store_query.rs`
- `crates/z00z_storage/src/assets/store_internal/proof_help.rs`
- `crates/z00z_storage/tests/assets/test_backend_facade_contract.rs`
- `crates/z00z_storage/tests/test_phase051_guardrails.rs`
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`

## Boundary Kept Intact

- `AssetTreeBackend` remains the single caller-facing semantic backend seam.
- Compatibility remains the migration oracle and default execution path.
- Forest and dual-verify modes are configuration-gated and fail closed until
  concrete operations land in later Phase 052 plans.
- Public facade methods route through the selected backend mode instead of
  bypassing through compatibility helpers.
- Public asset APIs still use semantic types such as `AssetPath`, `StoreItem`,
  `StoreOp`, `AssetStateRoot`, `CheckRoot`, `ProofItem`, `ProofBlob`, and
  `ProofScanOut`.
- `TreeId`, namespace bytes, raw path keys, branch ordering, and backend-local
  layout authority remain inside storage internals.
- `BucketId`, `BucketPolicy`, and `BucketRootLeaf` are verifier metadata for
  later storage-owned proof recomputation, not caller-managed asset authority.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found a significant bypass: public inherent proof methods still called
  compatibility helpers directly. Fixed `proof_item` and `proof_blob` to route
  through `AssetTreeBackend`, matching the mode-aware query methods.
- Pass 1 also confirmed the earlier query-surface correction: public
  root/get/list/mutation/claim/proof-scan/claim-source methods route through
  the backend trait, with `compat_*` helpers retained as compatibility-only
  implementation bodies.
- Pass 2 rechecked facade dispatch, `compatibility_backend()` call sites,
  downstream bucket/type leakage, `claim_null_rec` call sites after the
  `Result<Option<_>>` change, and Plan 01 acceptance criteria. No significant
  issues remained.
- Pass 3 rechecked all public `AssetStore` methods, storage-internal physical
  layout helpers, downstream guardrails, future-term exports, and the final
  diff. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3 after the
Pass 1 fix.

## Validation

All Rust validation for this plan was run after the final Rust code change.

- `cargo fmt --all` completed. It printed only the repository's existing
  stable-toolchain rustfmt warnings for nightly-only config keys.
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed as
  the mandatory fail-fast gate.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_storage --test test_assets_suite --test test_phase051_guardrails`
  passed: `test_assets_suite` 25 passed and `test_phase051_guardrails` 8
  passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_simulator --test test_claim_tx_pipeline`
  passed: 24 passed.
- `cargo test --release --features test-fast --features wallet_debug_dump -p z00z_wallets --lib adapters::rpc::methods::asset_impl::asset_impl_tests::test_asset_add_valid_definition`
  passed after fixing the test helper's wallet-config environment isolation.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed for the workspace, including doc-tests.

## Result

`052-01` is complete. Phase 052 can advance to `052-02-PLAN.md` for the
private forest tree store and deterministic batch planner. The forest backend
is still intentionally unsupported for live operations until later plans
replace skeleton behavior with the physical forest implementation.
