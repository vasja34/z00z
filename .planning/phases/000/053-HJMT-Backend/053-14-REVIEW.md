---
phase: 053-HJMT-Backend
reviewed: 2026-05-30T15:15:48Z
depth: deep
files_reviewed: 12
files_reviewed_list:
  - crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs
  - crates/z00z_simulator/src/scenario_1/stage_11_utils/stage_11_charlie.rs
  - crates/z00z_simulator/tests/test_scenario1_unified_gate.rs
  - crates/z00z_storage/Cargo.toml
  - crates/z00z_storage/src/settlement/README.MD
  - crates/z00z_storage/src/settlement/mod.rs
  - crates/z00z_storage/src/settlement/proof.rs
  - crates/z00z_storage/src/settlement/store.rs
  - crates/z00z_storage/src/settlement/store_query.rs
  - crates/z00z_storage/tests/test_live_guardrails.rs
  - crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs
  - crates/z00z_wallets/src/tx/state_witness.rs
findings:
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Phase 053-14: Code Review Report

**Reviewed:** 2026-05-30T15:15:48Z  
**Depth:** deep  
**Files Reviewed:** 12  
**Status:** issues_found

## Summary

I reviewed the live Phase 053 storage/simulator/wallet cutover slice around proof-surface cleanup, Stage 11 artifacts, and archive gating. I did not find any live simulator, wallet, or validator callers still invoking hidden `chk_blob` or `chk_item` compatibility helpers, and `cargo test -p z00z_storage --release --features test-fast -- --list` did not list the archived Phase 051/052 suites in the default gate. The remaining issues are a real mixed-leaf Stage 11 failure, one public proof helper that is still asset-lane-shaped, and README/source-shape drift.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: Stage 11 settlement scan still crashes on valid `RightLeaf` rows

**Severity:** BLOCKER  
**File:** `crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs:72`  
**File:** `crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs:78`  
**File:** `crates/z00z_storage/tests/test_live_guardrails.rs`

**Issue:** `load_post_tx_candidates()` now enumerates the generalized settlement surface via `list_settlement(SettlementListReq::all(...))`, but it immediately calls `item.asset_leaf()?` and turns any non-asset terminal into a hard error. That means a valid mixed settlement store will abort Stage 11 before proof validation or ownership detection instead of skipping or explicitly rejecting `RightLeaf` rows. The current guardrail only checks the Stage 11 label string, so this regression can ship green.

**Fix:** Treat Stage 11 as asset-only over a mixed settlement store: branch on `item.leaf()` or `item.asset_leaf()`, skip or record `RightLeaf` rows explicitly, and only construct `AssetPath` and `AssetLeaf` proof inputs after confirming `SettlementLeaf::Asset`. Add a guardrail or simulator test that seeds both asset and right leaves into `post_tx` storage and proves the scan artifact still succeeds.

### CR-02: Public `chk_blob_item_settlement` helper is still asset-lane-shaped

**Severity:** BLOCKER  
**File:** `crates/z00z_storage/src/settlement/mod.rs`
**File:** `crates/z00z_storage/src/settlement/proof.rs`

**Issue:** The live public boundary re-exports `chk_blob_item_settlement` as part of the settlement proof surface, but the helper still requires `path: &AssetPath` and `leaf: &AssetLeaf`. Right-family callers cannot use the advertised convenience API even though `SettlementLeaf` and `RightLeaf` are now first-class live contracts. This is a public-surface regression against the cutover contract: the alias names changed, but one exported settlement helper still encodes asset-lane assumptions.

**Fix:** Either make `chk_blob_item_settlement` genuinely settlement-native by accepting `SettlementPath` plus `SettlementLeaf` semantics, or stop exporting and documenting it as generalized settlement API. In either case, add right-leaf coverage so the public proof surface does not silently remain asset-only.

## Warnings

### WR-01: README module map and migration link no longer match the source tree

**Severity:** WARNING  
**File:** `crates/z00z_storage/src/settlement/README.MD`

**Issue:** The cleaned-up README still points readers at `types.rs`, `store_internal/tree_id.rs`, `store_internal/tree_store.rs`, `store_internal/tx_plan/mod.rs`, and `README-old.md`, but those files do not exist in the current tree. The actual implementation lives in `types_identity.rs`, `types_query.rs`, `types_record.rs`, and `store/*`. This is direct doc/source-shape drift inside the same cleanup slice.

**Fix:** Update the module map to the real filenames and either remove the `README-old.md` link or add the missing historical document before claiming the README reflects the live Phase 053 source shape.

---

_Reviewed: 2026-05-30T15:15:48Z_  
_Reviewer: the agent (gsd-code-reviewer)_  
_Depth: deep_
