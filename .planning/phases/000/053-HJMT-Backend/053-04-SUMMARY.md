---
phase: 053-HJMT-Backend
plan: 053-04
status: complete
completed_at: 2026-05-30
next_plan: 053-05
requirements:
  - PH53-04
summary_artifact_for: .planning/phases/053-HJMT-Backend/053-04-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 053-04 Summary: FeeEnvelope Contract And Separation From Rights

## ✅ Completed Scope

`053-04` is complete for the Phase 053 live fee-support contract slice.
`FeeEnvelope` is now a real versioned processing-support object in
`z00z_storage`, separate from `RightLeaf`, ownership proofs, wallet-control
semantics, and terminal right meaning. The live contract now carries explicit
payer or sponsor bindings, `budget_units`, a committed budget digest,
`domain_id`, `transition_id`, expiry, replay key, optional `support_ref`, and a
committed `failure_policy_id`.

Validation now happens before settlement mutation and before durable commit.
The store derives fee-support context from the actual transition payload plus
its exact pre-state root and semantic version instead of trusting caller-made
transition digests. Invalid fee envelopes fail closed for stale expiry,
insufficient budget, wrong sponsor or payer binding, wrong domain, wrong
transition, missing support binding, partial actor context, and replayed or
stale envelopes.

Replay protection is durable across Compatibility, Forest, and DualVerify
backends. Fee replay rows are snapshotted, restored, journal-bound, persisted in
RedB, checked against full-row compatibility digests on reload, and guarded by
transactional durable-head validation. Forest recovery now preserves pending
checkpoint provenance for fee-attested commits and can publish recovered state
metadata after a `ParentsCommitted` interruption without losing dual-reload
correctness.

## ✅ Files Changed

- `crates/z00z_storage/src/settlement/fee_envelope.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/types_record.rs`
- `crates/z00z_storage/src/settlement/store_types.rs`
- `crates/z00z_storage/src/settlement/store_rows.rs`
- `crates/z00z_storage/src/settlement/store_query.rs`
- `crates/z00z_storage/src/settlement/store_codec.rs`
- legacy dual-verify runtime lane and batch helpers, now removed from the live tree
- `crates/z00z_storage/src/settlement/tx_plan_types.rs`
- `crates/z00z_storage/src/settlement/redb_backend.rs`
- `crates/z00z_storage/src/settlement/redb_backend_state.rs`
- `crates/z00z_storage/src/settlement/redb_backend_helpers.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/tests/test_fee_envelope.rs`
- `crates/z00z_storage/tests/test_fee_replay.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`

## ✅ Boundary Kept Intact

- `FeeEnvelope` remains separate from `RightLeaf`; fee support does not become
  right meaning, ownership evidence, wallet control, or spend authority.
- Fee rejection remains fail-closed: invalid fee support does not mutate
  settlement roots, does not publish partial forest commits, and does not leave
  replay rows behind on rejected transitions.
- Replay protection is bound to the exact transition and pre-state it supports,
  not just to a loose `StoreOp` payload.
- Compatibility and forest reload paths both validate persisted fee replay
  evidence instead of trusting missing or partially tampered replay rows.
- DualVerify keeps the compatibility and forest semantic states aligned while
  avoiding duplicate durable publication of the same version.

## ✅ Review Loop

Manual fallback was used for `.github/prompts/gsd-review-tasks-execution.prompt.md`
because the slash prompt is not a callable tool in this environment.

Multiple review passes were run against the Phase 053 authority. The main fixes
landed during that loop were:

- replacing digest-only budget handling with explicit `budget_units` plus
  `support_ref`
- making fee replay durability fail closed across Compatibility, Forest, and
  DualVerify reloads
- moving durable stale-handle protection into the active RedB write transaction
- tightening actor-binding validation while still allowing single-bound
  envelopes
- binding fee support to the exact pre-state root and semantic version
- preserving pending checkpoint provenance for attested forest recovery after a
  `ParentsCommitted` interruption
- restoring the module-local library test gate and extending focused Phase 053
  coverage for compatibility, dual reload, content tamper, and attested
  recovery

Two consecutive clean review passes were achieved at the end of the loop.

## ✅ Validation

The Phase 053 fee slice is green on the focused validation path.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` had
  already passed earlier in this execution thread as the mandatory fail-fast
  gate before the fee-support repair work continued.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --lib assets::fee_envelope::tests::`
  passed: 8 passed.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite backend_facade_contract::test_backend_mode_parsing_and_forest_boundaries`
  passed: 1 passed.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_fee_envelope`
  passed: 10 passed.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_fee_replay`
  passed: 7 passed.

The broad workspace command `cargo test --release --features test-fast --features wallet_debug_dump`
was not recovered as a closeout authority for this plan because unrelated
pre-existing `include_str!` path failures outside the fee slice still block that
workspace-wide gate.

## ✅ Result

`053-04` is complete. Phase 053 can advance to `053-05-PLAN.md` for the HJMT
store API and dev hard cutover work. This summary does not claim that unrelated
workspace test breakage outside the fee-support slice was fixed here.
