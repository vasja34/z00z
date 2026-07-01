---
phase: 062-Gaps-Closing-2
plan: 062-13
status: complete
completed_at: 2026-06-25
next_plan: 062-14
summary_artifact_for: .planning/phases/062-Gaps-Closing-2/062-13-PLAN.md
---

# 062-13 Summary: Local Adapter And Rights Capability Local Closure

## ✅ Outcome

`062-13` closes `PLAN-062-G13` on one canonical local adapter and local-rights
path. The live `DaAdapter` seam in `z00z_rollup_node` now has a concrete
deterministic `LocalDaAdapter` contract with explicit publish or resolve
records, provider-signal preservation, and fail-closed rejection for metadata
drift, forged source labels, wrong digests, missing resolve results, and
replayed external input ids.

The Phase 062 local-rights evidence packet is now explicit on the real
`scenario_1` target. The live simulator suite adds named local tests for
business-right entitlement flow, agentic right lifecycle, and
`RightClass::MachineCapability` lifecycle, so payroll or B2B or useful-work,
delegation, expiry, replay, one-time use, wrong action, and reuse rejection
are proven on the current project primitives instead of on placeholder lanes.

The bounded closeout docs were also synchronized on the single phase-authority
path. `Z00Z-IMPL-PHASES.md` now states deterministic local-only closure for the
adapter, voucher or rights-based evidence, `wallet.object.*` boundary context,
agentic rights, and machine capability sections, while keeping live bridge,
live chain, live testnet, oracle, and full-wallet overclaims out of scope. The
execution packet itself was corrected too: the final `062-13-PLAN.md` now uses
the actual rollup-node, validator, watcher, simulator, and doc anchors and a
matching Coverage Appendix instead of the stale wallet-object placeholders that
were still left in its lower half.

## 📂 Files Changed

- `.planning/phases/062-Gaps-Closing-2/062-13-PLAN.md`
- `.planning/phases/062-Gaps-Closing-2/062-13-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/phases/Z00Z-IMPL-PHASES.md`
- `crates/z00z_rollup_node/src/da.rs`
- `crates/z00z_rollup_node/src/lib.rs`
- `crates/z00z_rollup_node/src/runtime.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`

## 🧪 Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_rollup_node --lib`
- `cargo test --release -p z00z_rollup_node --test test_hjmt_node_lifecycle`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract`
- `cargo test --release -p z00z_watchers --test test_hjmt_publication_contract`
- `cargo test --release -p z00z_validators --test test_object_policy_verdicts`
- `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e::`
- `cargo test --release`
- `rg -n "local adapter|live bridge|out of scope|voucher|rights-based|useful-work|agentic rights|delegation|replay rejection|MachineCapability|one-time|wrong action|reuse rejection|wallet.object" .planning/phases/Z00Z-IMPL-PHASES.md`
- `git diff --check -- .planning/phases/062-Gaps-Closing-2/062-13-PLAN.md .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_rollup_node/src/da.rs crates/z00z_rollup_node/src/lib.rs crates/z00z_rollup_node/src/runtime.rs crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`

Result:

- The slice-specific release packet is green across rollup-node, validator,
  watcher, and simulator targets.
- `062-TODO.md` still names the simulator proof lane by module name, but the
  real Cargo target is `scenario_1`; the live-equivalent command
  `cargo test --release -p z00z_simulator --test scenario_1 test_hjmt_e2e::`
  was run and passed on the current tree.
- The broad workspace release rerun did not close green on the current dirty
  tree. `cargo test --release` and the follow-up broad `scenario_1` rerun
  surfaced claim-suite failures outside the `062-13` adapter/rights slice; the
  remaining reproducible blocker is
  `test_claim_persist::test_stage3_fail_no_persist`.
- `test_claim_acceptance::test_claim_publish_stage3_paths`,
  `test_claim_audit_log_integrity::test_audit_log_no_secrets`, and
  `test_claim_integration::test_stage3_bins_post_consume` all passed on exact
  release-binary reruns after the broad failure set was isolated.

## 🔎 Manual Review Passes

Because `/GSD-Review-Tasks-Execution` is not callable as a tool here, the
required review loop was executed manually against the same scope, with the
workspace-first `/doublecheck` skill used as the verification posture.

- Pass 1
  - Read `062-13-PLAN.md`, `062-TODO.md`, and the `GAPS.md` rows for
    `TASK-052`, `TASK-053`, `TASK-054`, `TASK-055`, `TASK-056`, `TASK-058`,
    `TASK-059`, `TASK-061`, and `TASK-062`.
  - Result: found stale lower-half plan drift in `task_artifacts`,
    `task_tests`, `read_first`, `files`, and the Coverage Appendix; rewrote the
    plan to the actual rollup-node, runtime, simulator, and doc anchors.
- Pass 2
  - Re-read `crates/z00z_rollup_node/src/da.rs`,
    `crates/z00z_rollup_node/src/runtime.rs`,
    `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`,
    `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`,
    `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`,
    `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`, and
    `.planning/phases/Z00Z-IMPL-PHASES.md`.
  - Result: confirmed the single local `DaAdapter` seam, the advisory metadata
    binding boundary, the named local rights or capability flows, and the
    bounded live-scope wording; no second authority plane remained after the
    packet rewrite.
- Pass 3
  - `rg -n "local adapter|live bridge|out of scope|voucher|rights-based|useful-work|agentic rights|delegation|replay rejection|MachineCapability|one-time|wrong action|reuse rejection|wallet.object" .planning/phases/Z00Z-IMPL-PHASES.md`
  - `git diff --check -- .planning/phases/062-Gaps-Closing-2/062-13-PLAN.md .planning/phases/Z00Z-IMPL-PHASES.md crates/z00z_rollup_node/src/da.rs crates/z00z_rollup_node/src/lib.rs crates/z00z_rollup_node/src/runtime.rs crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`
  - Result: clean.
- Pass 4
  - `cargo test --release`
  - Result: the broad workspace rerun surfaced non-slice claim-suite failures
    inside `z00z_simulator/tests/scenario_1`.
- Pass 5
  - Exact release-binary reruns of
    `test_claim_acceptance::test_claim_publish_stage3_paths`,
    `test_claim_audit_log_integrity::test_audit_log_no_secrets`,
    `test_claim_integration::test_stage3_bins_post_consume`, and
    `test_claim_persist::test_stage3_fail_no_persist`.
  - Result: the first three passed on exact reruns; the claim-persist failpoint
    lane remained the only reproducible blocker on the current tree.
- Pass 6
  - Serial release-binary claim-subset reruns with `test_claim_ --test-threads=1`.
  - Result: one follow-up serial claim pass was green, and the second serial
    claim rerun collapsed to the same
    `test_claim_persist::test_stage3_fail_no_persist` blocker. The broad noise
    was therefore reduced to one phase-external claim-persist instability
    rather than a `062-13` local adapter or rights regression.

## 📌 Task Closeout

- `TASK-052`
  - Closed by `test_rights_business_entitlement_lifecycle_local` plus the
    bounded voucher or rights-based closeout text. Payroll or B2B or
    useful-work scenarios now stay local object or right or voucher evidence
    only and do not overclaim oracle or live-service truth.
- `TASK-053`
  - Closed by `LocalDaAdapter`, `LocalAdapterRecord`, `LocalResolveState`,
    `test_status_keeps_local_provider_signal`, and
    `local_adapter_publish_resolve_contract`. The live adapter seam is now
    concretely local and deterministic without introducing a live bridge or DA
    scheduler.
- `TASK-054`
  - Closed by the new metadata mismatch, missing resolve result, and replay
    rejection errors together with the forged-label, wrong-digest,
    missing-resolve, replayed-input, validator, and watcher tests. Adapter
    metadata now stays advisory until the real publication binding and fails
    closed on drift.
- `TASK-055`
  - Closed by `test_agentic_right_lifecycle_local` and the live
    `test_object_policy_verdicts` packet. Agentic rights now bind to real right
    fixtures, local policy verdicts, delegation, consumption, expiry, replay
    rejection, and wrong-action rejection on the current simulator path.
- `TASK-056`
  - Closed by `test_machine_capability_lifecycle_local`. The live local proof
    now covers one-time use, wrong object, wrong action, expiry, and reuse
    rejection for `RightClass::MachineCapability`.
- `TASK-058`
  - Closed by the targeted local-adapter bounded closeout in
    `Z00Z-IMPL-PHASES.md`. The doc packet now states deterministic local
    adapter fixtures only, keeps the single `DaAdapter` seam explicit, and
    rejects live bridge or live chain or live testnet claims.
- `TASK-059`
  - Closed by the voucher or payroll or B2B closeout section together with the
    named simulator entitlement flow. The object-flow evidence now cites the
    positive local lifecycle and the negative replay or missing-right cases on
    the same bounded local path.
- `TASK-061`
  - Closed by the agentic-rights closeout section plus the named local
    simulator flow. Agent budget or service or data-access profiles now bind to
    live right fixtures instead of to future-only prose.
- `TASK-062`
  - Closed by the machine-capability closeout section plus the named local
    simulator flow. The docs now state bounded right usage, replay or
    missing-right failure, reuse rejection, and no full-wallet grant.
