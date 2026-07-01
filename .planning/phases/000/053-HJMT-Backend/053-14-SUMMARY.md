---
phase: 053-HJMT-Backend
plan: 053-14
status: complete
completed_at: 2026-06-02
next_plan: 053-15
requirements:
  - PH53-14
summary_artifact_for: .planning/phases/053-HJMT-Backend/053-14-PLAN.md
---

<!-- markdownlint-disable MD060 -->

# 053-14 Summary: Downstream Settlement Integration

## Completed Scope

`053-14` is complete for the downstream settlement-integration slice.

The live storage seam now carries right-aware claim-source contracts through the
same settlement-owned HJMT proof surface as assets. `claim_source_contract_for_item`
no longer rejects `RightLeaf` membership behind an asset-only precheck, and the
Phase-owned proof tests now show that a right claim proof preserves
`SettlementLeafFamily::Right`, the exact settlement path, the exact right leaf,
and the storage-owned scan surface.

The downstream checkpoint and snapshot boundaries are now explicitly protected
against proof-family drift. Phase-owned coverage proves that snapshot entry
construction rejects a witness with a tampered HJMT non-existence family even
when the typed settlement item itself is valid, while right-leaf snapshot reload
continues to round-trip through the settlement-native artifact path.

The downstream source-shape guardrails are also complete for the publication
boundary. Checkpoint, snapshot, and storage settlement sources remain free of
`PublicationRecord`, `PublicationRequest`, and `OnionNet` semantics, while the
runtime publication and watcher modules remain separate from storage-owned tree
identities, bucket identities, raw proof decoders, and legacy asset-root
authority.

Linked-liability closeout is now explicit in live tests. A right claim proof now
stays path-local and family-local under unrelated wallet inventory noise:
adding unrelated assets changes the global settlement root, but it does not
change the typed right proof item, its right-family scan surface, or its local
definition/serial proof identity.

## Scoped Boundary

This summary closes the downstream checkpoint or snapshot or wallet or
validator integration slice only. It does not claim the Scenario 1 production
examples, benchmark closeout, documentation closeout, or later legacy-purge
work owned by subsequent numbered plans.

## Review Loop

The required `GSD-Review-Tasks-Execution` loop completed for `053-14` through
manual YOLO-equivalent review passes because the slash prompt is not directly
invokable in this runtime.

- Review pass 1 found a live contract gap: `claim_source_contract_for_item`
  still rejected `RightLeaf` membership through an asset-only guard, so the
  settlement claim-source contract was not actually right-aware.
- Review pass 2 found missing owned evidence for the downstream proof-family
  rejection and publication-separation bullets, so Phase-owned guardrail and
  snapshot rejection coverage was added.
- Review pass 3 found the slice still lacked direct linked-liability locality
  evidence under unrelated inventory noise, so the claim-source suite was
  extended with a right-family path-local regression test.
- Review pass 4 reran the task on the post-fix tree and found no significant
  remaining issues.
- Review pass 5 reran the same task after full validation and found no
  significant remaining issues.

Two consecutive post-fix review passes were clean.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed on
  the final tree.
- `cargo test -p z00z_storage --release --features test-fast --test test_claim_source_proof --test test_downstream_guardrails`
  passed.
- `cargo test -p z00z_storage --release --features test-fast` passed on the
  final tree.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed on the final tree, including the previously flaky
  `test_stage4_output_crypto` surface and the full workspace release matrix.

## Result

`053-14` is complete for the owned slice. Phase 053 advances to
`053-15-PLAN.md` for Scenario 1 production examples.
