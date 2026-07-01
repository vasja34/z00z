---
phase: 053-HJMT-Backend
reviewed: 2026-05-30T21:21:24Z
depth: deep
files_reviewed: 17
files_reviewed_list:
  - docs/tech-papers/Z00Z-HJMT-Design.md
  - crates/z00z_storage/src/settlement/proof.rs
  - crates/z00z_storage/src/serialization/mod.rs
  - crates/z00z_storage/src/settlement/redb_backend_hjmt.rs
  - crates/z00z_storage/src/settlement/redb_backend_helpers.rs
  - crates/z00z_storage/src/settlement/redb_backend_validate.rs
  - crates/z00z_storage/src/settlement/store_rows.rs
  - crates/z00z_storage/src/settlement/live_recovery_tests.rs
  - crates/z00z_storage/tests/test_default_gate.rs
  - crates/z00z_storage/tests/test_fee_replay.rs
  - crates/z00z_storage/tests/test_live_guardrails.rs
  - crates/z00z_storage/tests/test_readme_examples.rs
  - crates/z00z_storage/tests/test_redb_reload.rs
  - crates/z00z_storage/tests/test_right_leaf.rs
  - crates/z00z_storage/tests/test_store_api.rs
  - crates/z00z_storage/tests/test_serialization_roundtrip.rs
  - crates/z00z_storage/tests/test_serialization_restore.rs
findings:
  critical: 1
  warning: 1
  info: 0
  total: 2
status: issues_found
---

# Phase 053: Code Review Report

**Reviewed:** 2026-05-30T21:21:24Z
**Depth:** deep
**Files Reviewed:** 17
**Status:** issues_found

## Summary

This pass re-checked the live Phase 053 HJMT backend state in current code and
tests, not historical summaries. The runtime/storage side now looks clean on
the requested recheck surfaces: RedB forest durable reload passes, settlement-
native terminal-row rehydrate is enforced, `RightLeaf` survives durable reload
and serialization artifact roundtrip/restore, claim-row and fee-replay drift are
rejected, journal rollback/recovery tests pass, and flat-root metadata drift is
fail-closed. `crates/z00z_storage/src/settlement/proof.rs` matches the landed live
forest inclusion contract, and `crates/z00z_storage/src/serialization/mod.rs`
matches the intended `test-fast` artifact-builder surface.

Phase 053 is still not clean for hard cutover because the canonical JMT design
document still contains live-flow prose and diagrams that describe archived
asset-centric contracts as if they were current Phase 053 behavior.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: The canonical JMT design doc still uses archived asset-centric contracts inside live Phase 053 flow and use-case sections

**Classification:** BLOCKER  
**Files:** `docs/tech-papers/Z00Z-HJMT-Design.md`

**Issue:** The normative requirement block correctly says the live Phase 053
public contract is `SettlementStateRoot` plus `SettlementPath`
(`JMT-REQ-001`/`JMT-REQ-002`), but later live sections still say the insert flow
validates `AssetPath` and `AssetLeaf`, the delete flow resolves `AssetPath`, two
diagrams still say `Publish AssetStateRoot / SettlementStateRoot later`, and the
private-cash use-case prose says wallets continue to handle `AssetPath` and
`AssetLeaf`. Those are not framed as archived compatibility notes. They directly
contradict the landed storage API and the live enforcement/tests around
settlement-native paths and roots.

**Fix:** Rewrite the live execution and use-case sections to use
`SettlementPath`, `SettlementLeaf`/terminal settlement leaves, and
`SettlementStateRoot` only. Any remaining `AssetPath` / `AssetStateRoot`
references in those sections should be explicitly marked archived compatibility
evidence, not live contract text.

## Warnings

### WR-01: Guardrail coverage does not pin the stale live sections that drifted in the canonical design doc

**Classification:** WARNING  
**Files:** `crates/z00z_storage/tests/test_live_guardrails.rs`, `crates/z00z_storage/tests/test_readme_examples.rs`, `docs/tech-papers/Z00Z-HJMT-Design.md`

**Issue:** The current guardrail pass checks claims, roadmap, requirements, and
acceptance sections in the design doc, but it does not pin Section 6 insert/
delete flow prose or the Section 11.1 wallet/use-case prose. That omission is
why the stale `AssetPath` / `AssetStateRoot later` language survived even though
the rest of the Phase 053 doc was cleaned up and the runtime tests are green.

**Fix:** Extend the Phase 053 doc guardrails to assert that live flow/use-case
sections use settlement-native nouns only and do not reintroduce
`AssetPath`/`AssetStateRoot` as current behavior.

## Validation

- `cargo test -p z00z_storage --release --features test-fast --test test_redb_reload --test test_fee_replay --test test_store_api --test test_right_leaf --test test_live_guardrails --test test_default_gate --test test_serialization_roundtrip --test test_serialization_restore -- --nocapture` passed.
- `cargo test -p z00z_storage --release --features test-fast test_live_forest_ -- --nocapture` passed.
- Focused results:
  - `test_redb_reload`: 6 passed.
  - `test_fee_replay`: 4 passed.
  - `test_store_api`: 3 passed.
  - `test_right_leaf`: 11 passed.
  - `test_live_guardrails`: 9 passed.
  - `test_default_gate`: 3 passed.
  - `test_serialization_roundtrip`: 5 passed, including `test_roundtrip_preserves_right_leaf_payloads`.
  - `test_serialization_restore`: 6 passed, including `test_restore_handles_right_leaf_artifact`.
  - `assets::store::live_recovery_tests`: 4 passed, including claim-row reload and journal rollback recovery.
- Non-blocking quality note: the targeted validation still emits compiler
  `dead_code` warnings for unused helper surfaces under
  `crates/z00z_storage/src/settlement/`.

---

_Reviewed: 2026-05-30T21:21:24Z_  
_Reviewer: GitHub Copilot (gsd-code-reviewer)_  
_Depth: deep_
