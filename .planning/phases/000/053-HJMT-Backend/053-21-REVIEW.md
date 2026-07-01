---
phase: 053-HJMT-Backend
reviewed: 2026-05-30T00:00:00Z
depth: deep
files_reviewed: 15
files_reviewed_list:
  - docs/tech-papers/Z00Z-HJMT-Design.md
  - crates/z00z_storage/src/settlement/proof.rs
  - crates/z00z_storage/src/serialization/mod.rs
  - crates/z00z_storage/src/settlement/hjmt_proof.rs
  - crates/z00z_storage/src/settlement/store_rows.rs
  - crates/z00z_storage/src/settlement/store.rs
  - crates/z00z_storage/src/settlement/live_recovery_tests.rs
  - crates/z00z_storage/tests/test_default_gate.rs
  - crates/z00z_storage/tests/test_live_guardrails.rs
  - crates/z00z_storage/tests/test_redb_reload.rs
  - crates/z00z_storage/tests/test_right_leaf.rs
  - crates/z00z_storage/tests/test_store_api.rs
  - crates/z00z_storage/tests/test_settlement_root.rs
  - crates/z00z_storage/tests/test_fee_envelope.rs
  - crates/z00z_storage/tests/test_phase052_recovery.rs
findings:
  critical: 1
  warning: 2
  info: 0
  total: 3
status: issues_found
---

# Phase 053: Code Review Report

**Reviewed:** 2026-05-30
**Depth:** deep
**Files Reviewed:** 15
**Status:** issues_found

## Summary

The live Phase 053 HJMT runtime surface is mostly sound: the settlement-native RedB reload path, RightLeaf round-trip, settlement-root proof lane, guardrails, and internal live recovery tests all pass under the current `test-fast` slice. The phase is still not clean for hard cutover because one current Phase 053 guardrail test is failing, the JMT design document still misstates the landed proof envelope, and an older recovery integration suite no longer compiles against the current storage visibility boundaries.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Current Phase 053 backend-mode guardrail is red on the live test surface

**File:** `crates/z00z_storage/tests/test_default_gate.rs`

**Issue:** The current Phase 053 suite fails in the default-gate assertion that
checks the live test-fast settlement builder export. The test requires the
literal substring `#[cfg(feature = "test-fast")]\npub use self::build::build_artifact;`,
but the live serialization module now includes an explanatory comment between
the attribute and the export in `crates/z00z_storage/src/serialization/mod.rs:25-28`.
The export still exists, but the guardrail is brittle enough to fail the suite
on a harmless formatting/comment change. This is a real blocker because the
current Phase 053 validation command is not green.

**Fix:** Replace the literal newline-adjacency assertion with a semantic check that tolerates the explanatory comment, or move the comment so the asserted layout matches again. The test should prove that `build_artifact` is test-fast-only and live-settlement-scoped, not that two lines remain adjacent forever.

## Warnings

### WR-01: The JMT design document still denies proof fields that the landed forest proof blob and tests intentionally keep

**File:** `docs/tech-papers/Z00Z-HJMT-Design.md`

**Issue:** The document says live HJMT "removes that compatibility-specific
binding shell," immediately after listing `backend_root + root_bind_ver +
root_bind` and standalone `asset_leaf_hash` as compatibility fields. The
landed forest proof code still carries those fields in the live envelope:
`ProofBlob::new_forest` stores `backend_root`, `root_bind_ver`, `root_bind`,
and `asset_leaf_hash` in `crates/z00z_storage/src/settlement/proof.rs`, and
the live Phase 053 fee-envelope test explicitly asserts those fields remain
present in `crates/z00z_storage/tests/test_fee_envelope.rs`. This is
documentation drift against the actual live verifier contract.

**Fix:** Update the design document to describe the landed Phase 053 forest proof envelope truthfully: the live proof adds the committed bucket layer but still preserves the diagnostic/backend binding fields and terminal hash in the current wire shape.

### WR-02: Recovery coverage moved internally, but the older integration suite is now dead and misleading

**File:** `crates/z00z_storage/tests/test_phase052_recovery.rs:10,187-423`

**Issue:** The old recovery integration suite no longer compiles because it
imports crate-private symbols (`AssetBackendMode`, `AssetListReq`,
`AssetPath`, `AssetTreeBackend`) and calls crate-private constructors such as
`AssetStore::load_with_backend_mode`, which now live behind crate-private
settlement internals. The equivalent live recovery coverage does exist and
passes in the private unit-test module
`crates/z00z_storage/src/settlement/live_recovery_tests.rs`, but leaving the
old integration file broken creates a false signal about Phase 053 coverage and
will fail all-features maintenance runs.

**Fix:** Either remove/retire the obsolete integration suite or rewrite it against the current public settlement API. If the intended coverage is now private-only, document that move and keep the integration lane out of active verification commands.

---

_Reviewed: 2026-05-30T00:00:00Z_
_Reviewer: GitHub Copilot (gsd-code-reviewer)_
_Depth: deep_
