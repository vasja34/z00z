---
phase: 018-a-b-c
artifact: test-spec
status: fallback-ready
source: plans-and-code-seams
updated: 2026-03-24
---

# Phase 018 Test Spec

## Purpose

📌 This document defines the missing Rust integration and end-to-end acceptance
coverage for Phase 018 when `018-SUMMARY.md` and `018-VERIFICATION.md` are not
yet available.

📌 It is intended to be directly usable by another engineer or agent without
guessing scenario boundaries, artifact names, proof anchors, or failure
criteria.

📌 Phase 018 coverage is Rust integration coverage, not browser automation.
End-to-end behavior must be proven through `z00z_simulator::scenario_1::runner`
and by asserting machine-readable artifacts emitted under
`crates/z00z_simulator/src/scenario_1/outputs`.

## Workflow Status

⚠️ Strict `gsd-add-tests` generation is blocked because
`.planning/phases/018-a-b-c/` does not yet contain `018-*-SUMMARY.md` and does
not contain a phase-local `*-VERIFICATION.md`.

📌 The classification and plan below therefore use these fallback inputs:

- `.planning/phases/018-a-b-c/018-CONTEXT.md`
- `.planning/phases/018-a-b-c/018-01-PLAN.md`
- `.planning/phases/018-a-b-c/018-02-PLAN.md`
- `.planning/phases/018-a-b-c/018-03-PLAN.md`
- `.planning/phases/018-a-b-c/todo.md`
- Existing test anchors in `crates/z00z_simulator/tests/`

## Classification

### TDD And Integration Targets

- `crates/z00z_simulator/src/scenario_1/stage_4.rs`
  because it owns canonical Stage 4 continuity and fail-closed root handling.
- `crates/z00z_simulator/src/scenario_1/storage_view.rs`
  because it must expose a deterministic proposed canonical `ledger_path`
  artifact.
- `crates/z00z_simulator/src/scenario_1/jmt_wallet_scan.rs`
  because it is the proposed reusable committed-state proof scan helper.
- `crates/z00z_simulator/src/scenario_1/stage_7.rs`
  because it must drive Charlie runtime update, wallet evidence refresh, and
  the wallet-balance invariant gate.
- `crates/z00z_simulator/src/scenario_1/stage_4_utils/mod.rs`
  because post-apply wallet report and invariant logic should reuse the
  existing Stage 4 helper surface.
- `crates/z00z_simulator/src/scenario_1/stage_6.rs`
  because it anchors proof-bearing exec input material for later binding tests.
- `crates/z00z_simulator/src/scenario_1/stage_8.rs`
  because it must expose finalized checkpoint artifact, link, audit, and
  checkpoint summary fields.

### E2E Browser Targets

- None.

📌 End-to-end proof for this phase must remain in Rust integration tests because
the scenario is file-artifact and storage driven rather than browser driven.

### Skip Targets

- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
  because it is orchestration configuration and should only be verified through
  scenario execution outcomes.
- Existing `tests/*.rs` files already listed in the plan metadata
  because they are targets for extension or comparison, not implementation
  files to classify.

## Existing Test Anchors

📌 Reuse and extend the conventions already present in:

- `crates/z00z_simulator/tests/test_stage4_root_support.rs`
- `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
- `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

📌 The established project convention is one focused integration file per seam,
named `test_*.rs`, executed with targeted `cargo test -p z00z_simulator --test`
commands.

## Test Files To Add Or Extend

### 1. Add `crates/z00z_simulator/tests/test_stage4_chain_path.rs`

📌 This file proves that Stage 4 is rebased on the full claim-backed store and
that continuity is exported as one canonical machine-readable path rather than
being inferred from separate observational exports.

Tests to implement:

1. `stage4_emits_canonical_ledger_path_from_full_claim_store`
   Demonstrates: Stage 4 continuity is derived from the same full claim-backed
   live store that underlies claim publication.
   Success conditions:
   - the emitted proposed artifact such as `outputs/storage/ledger_path.json`
     exists;
   - it contains proposed fields such as `claim_root_hex`, `prep_root_hex`,
     and `post_apply_root_hex` or implementation-equivalent explicit names;
   - the path also carries `draft_id_hex`, `exec_input_id_hex`, and
     `checkpoint_id_hex` when those later-stage values are available;
   - the values align with `claim_post`, `pre_tx`, `checkpoint_s7.json`, and
     `checkpoint_s8.json` instead of diverging.

2. `stage4_rejects_claim_to_prep_root_drift_before_transport`
   Demonstrates: continuity drift fails closed before transport artifacts are
   accepted.
   Success conditions:
   - Stage 4 returns `StageResult::Fail`;
   - the error mentions root drift or continuity mismatch;
   - transport artifacts such as `checkpoint_prep.json` and
     `tx_alice_to_bob_pkg.json` are absent.

3. `ledger_path_is_canonical_and_observational_exports_remain_secondary`
   Demonstrates: `claim_post`, `pre_tx`, and `post_tx` are inspection surfaces,
   while `ledger_path` is the explicit acceptance artifact.
   Success conditions:
   - `claim_post`, `pre_tx`, and `post_tx` summaries may exist unchanged;
   - the test reads `ledger_path` as the sole continuity contract;
   - the contract does not require reconstructing continuity from three
     unrelated summary files.

Required assertions:

- root hex values are 64 hex characters;
- IDs remain machine-readable strings;
- continuity fields are explicit, not nested inside human-readable text;
- the artifact is deterministic across identical runs.

### 2. Add `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`

📌 This file proves the new committed-state wallet scan path and the Charlie
runtime completion story.

Tests to implement:

1. `jmt_wallet_scan_requires_proof_before_ownership_detection`
   Demonstrates: candidate rows are proof-validated before stealth ownership
   detection is attempted.
   Success conditions:
   - the helper enumerates canonical committed rows from `post_tx` storage;
   - each owned row is backed by `proof_blob(...)` and `chk_blob(...)` success;
   - the scan artifact records that proof verification happened before wallet
     ownership classification.

2. `jmt_wallet_scan_rejects_detached_leaf_only_mode`
   Demonstrates: leaf ownership hints are insufficient when not bound to the
   committed store.
   Success conditions:
   - tampered or detached leaf data is rejected;
   - the error mentions proof, committed state, or inclusion mismatch;
   - Charlie runtime does not update after rejection.

3. `stage7_refreshes_charlie_runtime_from_committed_post_tx_state`
   Demonstrates: Charlie state changes only after Stage 7 canonical apply and
   only through the proof-validated JMT scan path.
   Success conditions:
   - Stage 7 succeeds;
   - refreshed wallet evidence shows Charlie changes in standard report
     surfaces;
   - the change is absent when the proof-validated path is disabled or tampered.

4. `stage7_emits_machine_readable_scan_explanation`
   Demonstrates: the phase explicitly documents `leaf scan` versus `JMT scan`
   in one acceptance artifact.

   Success conditions: a proposed machine-readable file such as
   `outputs/transactions/charlie_jmt_scan.json` exists; it contains an explicit
   scan mode or explanation field; and it records committed root, verified row
   count, owned row count, and rejection reason count or
   implementation-equivalent fields.

Suggested artifact contract:

```json
{
  "scan_mode": "proof_validated_jmt",
  "committed_root_hex": "...",
  "verified_rows": 0,
  "owned_rows": 0,
  "ownership_basis": "proof_then_stealth",
  "leaf_scan_equivalent": false,
  "explanation": "Committed-store proof verification happens before ownership detection"
}
```

### 3. Extend `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

📌 This file should remain the top-level acceptance gate for the whole scenario.

New tests or assertions to add:

1. `unified_gate_proves_claim_to_finalize_lane`
   Demonstrates: the single happy path is
   `claim -> prep -> apply -> Charlie JMT scan -> wallet invariant -> finalize`.
   Success conditions:
   - Stage 4 through Stage 8 all return `StageResult::Ok`;
   - the proposed `ledger_path` artifact exists and agrees with Stage 7 and
     Stage 8 outputs;
   - the finalized checkpoint directory contains artifact, link, and audit
     outputs.

2. `unified_gate_requires_wallet_balance_invariant_success`
   Demonstrates: final acceptance is blocked if post-apply wallet totals are not
   coherent.
   Success conditions:
   - invariant failure causes the unified test to fail;
   - final Stage 8 acceptance artifacts are absent or marked rejected;
   - Charlie evidence is not accepted as valid output.

3. `unified_gate_requires_charlie_scan_artifact`
   Demonstrates: the proof-complete Charlie story is part of acceptance.
   Success conditions:
   - the unified path asserts the Charlie scan artifact exists;
   - the artifact explicitly distinguishes committed-store JMT scanning from
     detached leaf scanning.

### 4. Extend `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`

📌 This file already covers draft versus final checkpoint behavior and should be
extended to assert the new Stage 8 summary surface.

New tests or assertions to add:

1. `opaque_final_mode_reports_summary_paths`
   Demonstrates: finalized `OpaqueTest` output exposes concrete summary fields
   for artifact, link, and audit paths.
   Success conditions:
   - `checkpoint_s8.json` reports `checkpoint_id_hex`;
   - it also reports proposed fields such as `artifact_path`, `link_path`, and
     `audit_path` or implementation-equivalent explicit field names;
   - the referenced paths exist and point to the finalized outputs.

2. `draft_only_mode_keeps_path_fields_empty`
   Demonstrates: draft and final contracts remain separate.
   Success conditions:
   - `status` remains `draft_only`;
   - `checkpoint_id_hex` is null;
   - path fields are absent or null;
   - no finalized artifact, link, or audit output is emitted.

3. `stage8_blocks_finalization_after_exec_or_snapshot_tamper`
   Demonstrates: proof-path binding survives into final publication.
   Success conditions:
   - tampering `exec_input_id_hex` or `snapshot_id_hex` causes Stage 8 failure;
   - finalized outputs are not emitted;
   - failure text mentions binding, exec input, snapshot, or checkpoint refs.

### 5. Add `crates/z00z_simulator/tests/test_stage8_proof_path.rs`

📌 This file proves that proof-bearing transaction material is consistently bound
from Stage 6 through Stage 8.

Tests to implement:

1. `proof_material_is_preserved_from_exec_input_to_artifact`
   Demonstrates: the same proof-bearing transaction path survives Stage 6 exec
   input, Stage 7 draft apply, and Stage 8 final publication.
   Success conditions:
   - the proof bytes or canonical proof identity in exec input match the final
     artifact proof payload;
   - the link and audit point to the same checkpoint id.

2. `proof_path_rejects_placeholder_only_story`
   Demonstrates: Phase 018 acceptance does not accept non-empty bytes alone as a
   sufficient story.
   Success conditions:
   - tests assert binding across stage outputs, not just that proof fields are
     non-empty;
   - replacing identity-bearing fields without updating the chain causes
     failure.

3. `proof_path_agrees_with_ledger_path_artifact`
   Demonstrates: proof propagation and continuity reporting describe the same
   canonical lane.
   Success conditions:
   - the proposed `ledger_path.exec_input_id_hex` field agrees with Stage 6
     bridge and Stage 7 summary;
   - the proposed `ledger_path.checkpoint_id_hex` field agrees with Stage 8
     summary once finalized.

## Wallet Invariant Contract

📌 Phase 018 must introduce one explicit scenario-level wallet-balance invariant
gate over refreshed wallet evidence.

Minimum invariant coverage:

1. Alice balance decrease equals the committed spend plus fee contribution for
   the canonical transfer path.
2. Bob confirmed receive totals reflect the committed transfer outputs.
3. Charlie changes appear only when the committed post-apply JMT scan finds and
   verifies owned rows.
4. No actor can gain or lose items in contradiction with the refreshed diff or
   confirmed evidence files.
5. Final acceptance fails if any refreshed wallet report contradicts the
   committed post-apply state.

Preferred assertion anchors:

- `outputs/transactions/wallets_state_before.json`
- `outputs/transactions/wallets_state_after.json`
- `outputs/transactions/wallets_state_diff.json`
- `outputs/transactions/wallets_confirmed.json`
- `outputs/transactions/wallets_state_report.md`

## Negative Scenarios That Must Exist

📌 These failures are mandatory because they prove soundness, not just the happy
path.

1. Claim root and Stage 4 prep root drift.
2. Detached leaf ownership data without committed-store proof.
3. Charlie runtime update attempted before Stage 7 canonical apply.
4. Wallet invariant failure after refreshed evidence.
5. Stage 8 finalization with tampered `exec_input_id_hex`.
6. Stage 8 finalization with tampered `snapshot_id_hex`.
7. Draft-only mode incorrectly presenting finalized outputs.

## Commands

📌 Use the repo-preferred release-style commands for Phase 018 verification.

📌 `test_stage7_jmt_wallet_scan.rs`, `test_stage6_checkpoint_final_gate.rs`,
`test_stage8_proof_path.rs`, and `test_scenario1_unified_gate.rs` should be
treated as release-style validation seams.

📌 Without the release/test-fast profile, these files either report `0 tests`,
return early under debug assertions, or hit the expected debug-profile range
proof guard. For those seams, the authoritative targeted commands are the
release-style commands below.

Targeted commands:

```bash
cargo test -p z00z_simulator --test test_stage4_chain_path -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage6_checkpoint_final_gate -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture
```

Release gates:

```bash
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_chain_path -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage7_jmt_wallet_scan -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage8_proof_path -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario1_unified_gate -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
```

## Definition Of Done

✅ Phase 018 has one explicit continuity artifact instead of inferred continuity.

📌 The current preferred proposed filename for that artifact is
`outputs/storage/ledger_path.json`, but implementation may use an equivalent
machine-readable path if the contract remains explicit and deterministic.

✅ Charlie runtime completion is proven through committed-store proof validation
before ownership detection.

✅ The `leaf scan` versus `JMT scan` distinction is visible in machine-readable
acceptance output.

✅ The wallet-balance invariant is enforced at scenario level.

✅ Stage 8 finalization reports sealed artifact, link, and audit surfaces and
rejects tampered binding paths.

✅ One unified integration gate proves the full canonical lane from claim
publication to finalized checkpoint publication.
