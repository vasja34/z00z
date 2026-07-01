---
phase: 054-Refactor-Crates
plan: 054-01
status: complete
completed_at: 2026-06-08
next_plan: 054-02
requirements-completed: [PH54-01]
---

# 054-01 Summary

## Outcome

Phase `054-01` is complete.

The storage crate now has an explicit low-level backend seam under the live
settlement facade, and the public proof or semantic surface is guarded before
any deeper adapter extraction. `SettlementStore`, `SettlementTreeBackend`,
`SettlementStateRoot`, and the proof bench remain the compatibility oracle for
later Phase 054 waves.

## Landed Changes

- Added `crates/z00z_storage/src/backend/mod.rs` with the low-level durable
  seam traits:
  - `ReadTxn`
  - `WriteTxn`
  - `StorageBackend`
  - `JournalBackend`
- Moved the private backend error definition to
  `crates/z00z_storage/src/backend/error.rs` while keeping the live internal
  symbol stable through `pub(crate) use crate::backend::error::StoreBackendError;`
  in `crates/z00z_storage/src/error.rs`.
- Exported the backend module from `crates/z00z_storage/src/lib.rs` without
  changing the semantic settlement facade.
- Reworked `crates/z00z_storage/benches/assets_proofs.rs` so proof generation
  is exercised through the semantic `SettlementTreeBackend` surface rather than
  through concrete store shape assumptions.
- Expanded storage guardrails:
  - `crates/z00z_storage/tests/test_live_guardrails.rs` now proves the seam
    traits compile, confirms the backend layer stays below semantic types, and
    locks the `StoreBackendError` move boundary.
  - `crates/z00z_storage/tests/test_downstream_guardrails.rs` now rejects
    downstream dependence on `StorageBackend`, `JournalBackend`, `ReadTxn`,
    `WriteTxn`, and `StoreBackendError`.

## Validation

Executed and passed on the current tree while closing this slice:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo fmt --all`
- `cargo test -p z00z_storage --release --test test_live_guardrails --test test_downstream_guardrails`
- `cargo bench -p z00z_storage --bench assets_proofs --no-run`

The plan-mandated broad workspace command is stale against the live manifest on
this repository state:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed failure:

- `error: none of the selected packages contains these features: test-fast, wallet_debug_dump`

This was treated as a live-worktree feature-name mismatch rather than as a
storage regression. Later `054-02` validation re-confirmed these guardrails on
the extracted seam with the current repo feature set.

## Review Loop

- Review pass 1 checked the seam diff and confirmed that semantic settlement
  exports were not replaced by backend-root authority.
- Review pass 2 rechecked downstream and source-shape guardrails and found no
  additional significant issues.
- Review pass 3 rechecked diff hygiene with `git diff --check`; no significant
  issues remained, giving the required consecutive clean passes.

## Closeout

The proof/public compatibility gate is now frozen before adapter extraction.
That guardrail baseline is the dependency for `054-02` and later storage-heavy
waves.
