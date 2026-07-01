---
phase: 053-HJMT-Backend
plan: 053-03
status: complete
completed_at: 2026-05-30
next_plan: 053-04
requirements:
  - PH53-03
summary_artifact_for: .planning/phases/053-HJMT-Backend/053-03-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 053-03 Summary: Settlement Paths And Right Leaves

## Completed Scope

`053-03` is complete for the Phase 053 settlement terminal-contract cutover.
Storage now exposes live `TerminalId`, `SettlementPath`, `SettlementLeaf`, and
`RightLeaf` contracts with family-tagged canonical encoding, typed terminal-id
validation, explicit asset-versus-right payload separation, and fail-closed
right transition checks. Asset-lane hashing, storage proof inputs, storage-row
rehydration, and asset-only decode paths all consume the tagged settlement-leaf
encoding, so asset-shaped bytes can no longer silently pass as a right leaf or
vice versa.

`RightLeaf` remains narrow and typed. The live contract now carries right class,
issuer/provider scope, holder and beneficiary commitments, payload commitment,
validity window, challenge window, replay marker, revocation/transition policy,
and disclosure/retention policy identifiers. The action validator fails closed
for wrong holder binding, replayed one-time use, revoked transitions, expired
or not-yet-valid transitions, missing transition policy, missing challenge
policy, and challenge requests outside the committed challenge window.

The implementation reused existing `z00z_storage` seams in place. No parallel
settlement authority, no fee-inside-right adapter, and no duplicate asset-only
codec lane were introduced. Full live replacement of downstream
`AssetPath`/`AssetStateRoot` consumer surfaces and live mixed asset/right
terminal persistence remain bounded to later Phase 053 plans.

## Files Changed

- `crates/z00z_storage/src/settlement/leaf.rs`
- `crates/z00z_storage/src/settlement/model.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/store_query.rs`
- `crates/z00z_storage/src/settlement/store_rows.rs`
- `crates/z00z_storage/src/settlement/store_types.rs`
- `crates/z00z_storage/src/settlement/types_identity.rs`
- `crates/z00z_storage/src/settlement/types_record.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_right_leaf.rs`
- `crates/z00z_storage/tests/test_settlement_leaf.rs`

## Boundary Kept Intact

- `SettlementLeaf` is a real tagged terminal family, not a rename or wrapper
  alias for `AssetLeaf`.
- Family markers are committed in canonical payload bytes and therefore change
  asset-lane hashes and proof inputs.
- Asset-only storage readers reject `RightLeaf` bytes instead of coercing them
  through the old asset lane.
- `RightLeaf` stays separate from `FeeEnvelope`; fee budget, payer, sponsor,
  relay, and processing-support semantics are not part of the live right leaf.
- The right taxonomy stays narrow and explicit: machine, agent, external-right,
  data-access, service-credit, human-permit, and liability-domain families.
- Phase 053 still does not claim downstream wallet/validator/runtime mixed-right
  execution is complete; later plans own live right ingestion, fee support,
  proof-envelope v2, and consumer cutover.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 reviewed `053-TODO.md` section `053-03`, `053-03-PLAN.md`, and the
  touched storage/test files. It found one plan-scope gap: `RightLeaf` still
  lacked a committed challenge-window surface and transition actions did not
  fail closed on missing transition policy. Those gaps were fixed in
  `types_record.rs` and covered by new focused tests.
- Pass 2 rechecked the final implementation with `code-reviewer` and
  `crypto-architect` criteria: tagged canonical encoding, proof/hash boundary
  consistency, asset-lane rejection of right bytes, typed path validation,
  narrow right taxonomy, and no fee/right semantic drift. No significant
  `053-03` issues remained.
- Pass 3 ran a workspace-first `doublecheck` pass against the closeout claims
  using `053-TODO.md`, `docs/tech-papers/Z00Z-HJMT-Design.md`, the touched storage sources,
  the focused tests, and the validation output. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3 after the
Pass 1 spec-gap fix.

## Validation

All Rust validation required for this plan is green in the canonical order.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed as
  the mandatory fail-fast gate.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_live_guardrails --test test_settlement_leaf --test test_right_leaf --test test_settlement_root`
  passed: 28 passed.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed for the workspace, including doc-tests.

## Result

`053-03` is complete. Phase 053 can advance to `053-04-PLAN.md` for
`FeeEnvelope` as the separate live processing-support contract. This summary
does not claim live fee support, mixed-right storage mutation, right proof
families beyond asset-lane tagging, or downstream right-consumer execution are
already complete; those remain owned by later numbered Phase 053 plans.
