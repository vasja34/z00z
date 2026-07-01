---
phase: 034-mix1-fixes
plan: 09
subsystem: optional-post-closure-sidecars
tags: [optional-sidecar, completed, inventory, rename-hygiene, legacy-collision-retirement, suffix-collapse]
requires:
  - phase: 034-08
    provides: main semantic closure already separated from any optional hygiene wave
provides:
  - bounded rename cleanup completion for 034-16
  - legacy collision retirement completion for 034-17
  - production-current suffix collapse completion for 034-18
  - truthful post-closure hygiene record kept outside the 034-08 semantic closure proof
affects: [034-09, 034-16, 034-17, 034-18, PH34-ID-SIGNATURE-HYGIENE, PH34-SUFFIX-COLLAPSE]
tech-stack:
  added: []
  patterns: [inventory-first execution, collision-first rename audit, blocked-sidecar truthfulness]
key-files:
  created:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-09-SUMMARY.md
  modified:
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-09-PLAN.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-TODO.md
    - /home/vadim/Projects/z00z/.planning/phases/034-mix1-fixes/034-suffixes-V1-Vn.md
    - /home/vadim/Projects/z00z/.github/copilot-instructions.md
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/key/seed_cipher_params.rs
    - /home/vadim/Projects/z00z/crates/z00z_wallets/src/core/backup/backup_wire.rs
    - /home/vadim/Projects/z00z/crates/z00z_crypto/src/claim/v2.rs
    - /home/vadim/Projects/z00z/crates/z00z_crypto/src/crypto_constants.rs
    - /home/vadim/Projects/z00z/crates/z00z_storage/src/checkpoint/artifact_stmt.rs
key-decisions:
  - "Plan 09 executes only after 034-08 semantic closure and remains closure-invisible evidence for the Q63/Q64/Q65/Q47 story."
  - "The required execution order is 034-16 bounded rename cleanup, then 034-17 legacy collision retirement, and only then 034-18 suffix collapse."
  - "Runtime fallout discovered during the hygiene wave must be fixed truthfully in production code and stale tests before Phase 034 planning can be marked complete."
requirements-completed: [034-09, 034-16, 034-17, 034-18]
status: completed
completed: 2026-04-10
reviewed: 2026-04-10T00:00:00+00:00
---

# Phase 034 Plan 09 Summary

## Outcome

Plan 09 is complete. The post-closure hygiene chain executed in the required
order and is now recorded truthfully without changing the semantic closure root
established by `034-08`.

## Accomplishments

- Re-read the Plan 09 execution contract, the `034-16` and `034-17` mandatory
  pre-read sections in `034-TODO.md`, the local Rust naming rules in
  `.github/copilot-instructions.md`, and the authoritative suffix inventory in
  `034-suffixes-V1-Vn.md` before executing the hygiene chain.
- Executed `034-16` as the bounded non-Tari identifier-hygiene wave across the
  active workspace surfaces.
- Executed `034-17` by retiring legacy unsuffixed collision blockers, including
  the old `ClaimAuthoritySig` and `BackupContainer` surfaces that prevented a
  truthful production-current collapse.
- Executed `034-18` by collapsing the remaining production-current suffixes,
  including the `CheckpointStmtV1` and `MAX_PROOF_SIZE_V1` tails, and aligned
  affected call sites, tests, and docs.
- Fixed runtime fallout exposed by the release-style reruns:
  - removed `Vec::with_capacity(limit)` overflow risk from
    `AssetStore::list()` for `AssetListReq::all(usize::MAX)` paths;
  - added a storage regression test covering the unbounded listing seam; and
  - updated the stale simulator claim-publish test contract to persist the
    canonical claim-membership store before asserting the missing-reservation
    failure path.
- Reconciled Phase 034 planning/state artifacts and source-shape guards so the
  executed hygiene chain is recorded under `034-09` while `034-08` remains the
  semantic closure anchor.

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed.
- `cargo test -p z00z_storage --release --test test_search_api -- --nocapture`
  passed with 6 tests, including the unbounded-limit regression.
- `cargo test -p z00z_simulator --release --features test-fast --test test_claim_tx_pipeline -- --nocapture`
  passed with 23 tests after the persisted claim-store test contract update.
- The exact user-required simulator release gate is rerun after the hygiene
  chain and must remain the final corroborating gate for this plan.

## Issues Encountered

- The release-style reruns surfaced real fallout that had to be fixed before
  the plan could close truthfully:
  - storage listing overflow on `usize::MAX` request paths; and
  - a stale simulator test that bypassed the now-required persisted
    claim-membership contract.
- The planning and source-shape guard surfaces were stale after the code landed
  and had to be synchronized to the completed `034-09` hygiene chain.

## Next Phase Readiness

- Plan 09 is complete and now owns the recorded post-closure hygiene truth for
  Phase 034.
- `034-08` remains the semantic closure anchor for Q63, Q64, Q65, and Q47.
- `034-15` remains optional, deferred, and non-semantic.
- The next canonical phase is `035`.

## Threat Flags

None remaining inside the completed Plan 09 scope after the executed hygiene
chain, fallout fixes, and planning-sync pass.

## Self-Check

PASSED for truthful completion recording: the required `034-16` -> `034-17` ->
`034-18` chain landed, fallout was fixed on the live tree, and the planning
surface now records the result without changing the semantic closure root.

---
*Phase: 034-mix1-fixes*
*Status: Completed*
*Reviewed: 2026-04-10*
