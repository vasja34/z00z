---
phase: 034-mix1-fixes
reviewed: 2026-04-10T01:01:49Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - crates/z00z_wallets/src/core/stealth/output.rs
  - crates/z00z_wallets/src/core/stealth/test_output_extra.rs
  - crates/z00z_wallets/src/core/tx/mod.rs
  - crates/z00z_wallets/src/core/tx/output_flow.rs
  - crates/z00z_wallets/src/core/tx/builder.rs
  - crates/z00z_wallets/src/core/tx/witness_gate.rs
  - crates/z00z_wallets/src/core/tx/spend_verification.rs
  - crates/z00z_wallets/src/core/tx/tx_wire_types.rs
  - crates/z00z_wallets/src/lib.rs
  - crates/z00z_wallets/examples/wallet_reload.rs
  - crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs
  - crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs
  - crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs
  - crates/z00z_simulator/examples/simulator_interop/support.rs
  - .planning/phases/040-spend-proof/040-Spend-Proof-Spec.md
findings:
  critical: 0
  warning: 3
  info: 0
  total: 3
status: issues_found
---

# Phase 034-05 Code Review Report

**Reviewed:** 2026-04-10T01:01:49Z
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

I did not find a direct security or correctness regression in the sender-authority move itself. The material issues are concentrated in public-surface compatibility and spec drift around what the migration now guarantees.

## Warnings

### WR-01: The `core::tx` sender surface was removed as a hard breaking API change without a compatibility window

**File:** `crates/z00z_wallets/src/core/tx/mod.rs:11-23`
**File:** `crates/z00z_wallets/src/core/tx/mod.rs:67-72`
**Issue:** `builder` and `output_flow` were made private, and the old `tx` facade stopped re-exporting `build_output_leaf*`, `sender_create_output_for`, and `create_output_bundle*`. The legacy shims still exist internally in `builder.rs` and `output_flow.rs`, but downstream code importing the historical `z00z_wallets::tx::*` or `z00z_wallets::core::tx::builder::*` paths will now fail at compile time instead of getting a guided migration path. That is a material public API break, not just an internal cleanup.
**Fix:** Keep deprecated re-exports for one compatibility cycle and point them at the new `core::stealth` entry points, or explicitly treat this as a versioned breaking change and document the migration in the public crate surface.

### WR-02: `040-Spend-Proof-Spec.md` is now stale about `TxProofWire`

**File:** `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md:65-66`
**File:** `crates/z00z_wallets/src/core/tx/tx_wire_types.rs:109-124`
**Issue:** The spec still says “Non-empty `TxProofWire` for regular tx | Proposed only | Current `TxProofWire` is empty,” but the live wire already carries `spend: Option<SpendProofWire>` and `spend: Option<SpendAuthWire>`. That mismatch will mislead later phase work, audits, and wording-guard tests by documenting a capability as unimplemented when the code has already moved past that baseline.
**Fix:** Update the Phase 040 status table to reflect the shipped `spend` proof/auth fields and narrow the “proposed only” language to the still-missing theorem-level proof backend instead of the wire container itself.

### WR-03: The spec overstates sender-authority retirement; the live bind seam is still publicly owned by `core::tx`

**File:** `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md:60`
**File:** `.planning/phases/040-spend-proof/040-Spend-Proof-Spec.md:662`
**File:** `crates/z00z_wallets/src/core/tx/mod.rs:67-72`
**File:** `crates/z00z_simulator/src/scenario_1/stage_6_utils/bridge_output_router.rs:13-22`
**Issue:** The spec now says full output construction lives under `core::stealth`, but the live bridge flow still finishes new sender outputs through the public `tx::bind_output_wire` export. That means the construction seam is still split across `core::stealth` and `core::tx`, which is weaker than the “only public sender-construction authority” claim used in the Phase 034 requirement wording.
**Fix:** Either move the canonical wire-binding export under `core::stealth` and deprecate `tx::bind_output_wire`, or narrow the spec/phase wording to say that `core::stealth` owns leaf and bundle construction while `core::tx` still owns public wire binding.

---

_Reviewed: 2026-04-10T01:01:49Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
