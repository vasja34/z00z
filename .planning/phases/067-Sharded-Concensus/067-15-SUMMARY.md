---
phase: 067
plan: 067-15
status: complete
completed_at: 2026-07-06
next_plan: 067-16
summary_artifact_for: .planning/phases/067-Sharded-Concensus/067-15-PLAN.md
---

<!-- markdownlint-disable MD022 MD031 MD032 MD033 MD041 MD047 MD055 MD056 MD060 -->

# 067-15 Summary: HotStuff Like Local Backend Contract

## Outcome

`067-15` is complete.

`VERDICT-LCS-06` now closes on one executable local HotStuff-like backend path.
`hotstuff_local.rs` owns the canonical view or leader or timeout or view-change
or backend-QC state machine, `BftCommit` now carries one canonical
subject-bound commit bundle from the same seam, and validator binding still
fails closed unless the backend QC agrees with the live shard certificate,
theorem digest, publication-binding digest, and checkpoint-bearing validator
decision path.

The closeout keeps the honesty boundary explicit. `067-15` proves a local
deterministic HotStuff-like contract behind the already-landed commit-subject,
replay, transport, and validator seams, but it still does not claim production
HotStuff, real P2P networking, or external finality. Those remain later
verdict-lane work, with `067-16` now the next canonical execution lane.

## Files Changed

- `.planning/ROADMAP.md`
- `.planning/STATE.md`
- `.planning/phases/067-Sharded-Concensus/067-15-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-15-SUMMARY.md`
- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `crates/z00z_runtime/aggregators/README.md`
- `crates/z00z_runtime/aggregators/src/bft_engine.rs`
- `crates/z00z_runtime/aggregators/src/hotstuff_local.rs`
- `crates/z00z_runtime/aggregators/src/lib.rs`
- `crates/z00z_runtime/aggregators/tests/test_hotstuff_local_backend.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`
- `crates/z00z_wallets/src/redb_store/mod.rs`

## Landed Changes

- Canonical HotStuff-like backend state machine
  - Added `HotstuffProposal`, `HotstuffTimeout`, `HotstuffViewChange`,
    `HotstuffLeaderConflict`, `HotstuffQc`, `HotstuffCommit`, and
    `HotstuffLocal` under one runtime-owned module.
  - The backend now enforces deterministic leader rotation, subject-bound
    proposals, structured timeout evidence, view changes, leader-conflict
    evidence, and backend QC construction over the canonical shard
    certificate path.
- Canonical validator-binding path stays intact
  - `BftCommit::new(...)` now centralizes the subject-bound commit bundle so
    the HotStuff-local backend and lower-level BFT path share one live commit
    shape.
  - `HotstuffLocal::bind_validator(...)` now proves the backend QC cannot
    bypass the live validator decision seam.
- Honest module and scenario claim surface
  - `crates/z00z_runtime/aggregators/README.md` now documents
    `hotstuff_local` as the canonical local backend path and keeps
    `bft_engine` scoped to threshold math only.
  - `crates/z00z_simulator/src/scenario_11/mod.rs` and
    `crates/z00z_simulator/tests/test_scenario_11.rs` now record and assert
    that production HotStuff stays a non-claim until a real backend crate is
    installed and exercised.
- Dedicated backend coverage
  - Added `test_hotstuff_local_backend.rs` to cover leader rotation, timeout
    to view-change flow, conflicting leader proposals, validator binding, and
    subject-guard rejection on the live local seam.
- Release-gate repair
  - Restored the grouped crate-private wallet debug-export re-export shape in
    `crates/z00z_wallets/src/redb_store/mod.rs` so
    `test_debug_export_surface_is_internal_only` stays green on the final
    release tree.

## Validation

Commands green during the `067-15` closeout cycle:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hotstuff_local_backend -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_bft_committee_rules -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_fault_matrix -- --nocapture`
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- `cargo test --release`
- `bash scripts/audit/audit_release_feature_guards.sh`

Broad release-gate note:

- The final current-cycle `cargo test --release` rerun completed green on the
  final `067-15` tree after restoring the grouped crate-private wallet
  `redb_store` debug-export surface required by
  `test_production_hardening`.
- `bash scripts/audit/audit_release_feature_guards.sh` reran green after that
  repair on the same final tree.

## Manual Review Passes

The required `/GSD-Review-Tasks-Execution` loop was attempted three times in
YOLO mode, but the current runner again did not provide a usable automated
review path for this slice.

- Attempt 1
  - `timeout 90s gsd --no-session --extension .github --print '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-15-PLAN.md current_task="067-15-T1" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.
- Attempt 2
  - `bash -lc '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-15-PLAN.md current_task="067-15-T1" --yolo'`
  - Result: failed with `/GSD-Review-Tasks-Execution: No such file or directory`
- Attempt 3
  - `timeout 90s gsd -p '/GSD-Review-Tasks-Execution current_spec=.planning/phases/067-Sharded-Concensus/067-15-PLAN.md current_task="HotStuff Like Local Backend Contract" --yolo'`
  - Result: exited with code `1` and produced no stdout or stderr.

Equivalent workspace-first manual review was executed with the
`/code-reviewer` checklist and `/doublecheck` three-layer posture against the
same scope.

- Pass 1
  - Re-ran anchored grep for the canonical backend markers `HotstuffLocal`,
    `HotstuffProposal`, `HotstuffTimeout`, `HotstuffViewChange`,
    `HotstuffLeaderConflict`, `HotstuffQc`, `HotstuffCommit`,
    `collect_qc`, `advance_view`, and `bind_validator` across the touched
    runtime and test surfaces.
  - Result: clean. One canonical HotStuff-local module and one canonical
    export path remained under the reviewed scope.
- Pass 2
  - Re-ran anchored grep for the live honesty and threshold markers
    `production HotStuff`, `hotstuff_local`, `HotStuff-like`,
    `BftTwoFPlusOne`, `3f+1`, `2f+1`, and `view change` across the touched
    runtime, simulator, and plan surfaces.
  - Result: clean. The code, tests, README, simulator honesty report, and
    plan packet agree on one local simulated-full HotStuff-like claim and one
    explicit production non-claim.
- Pass 3
  - Ran `git diff --check` across the touched `067-15` code, the grouped
    wallet release-guard repair, and planning artifacts after the final
    status sync.
  - Result: clean.
- Pass 4
  - Re-read `067-15-PLAN.md`, `067-15-SUMMARY.md`, `067-COVERAGE.md`,
    `067-verdict.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md` after
    the final status sync.
  - Result: clean.

Passes 3 and 4 were consecutive clean manual review runs after the final
closeout sync.

## Closeout

`067-15` closes `VERDICT-LCS-06` by making the HotStuff-like backend term an
executable local state machine instead of a generic BFT label, while keeping
the canonical shard certificate and validator decision path intact.

`067-16` is now the next canonical execution lane.
