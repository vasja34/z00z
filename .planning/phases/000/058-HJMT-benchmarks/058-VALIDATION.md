---
phase: 058
slug: hjmt-benchmarks
status: audited
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-15
updated: 2026-06-15
---

# Phase 058 - Validation Strategy

Final closeout validation record for Phase 058. Refreshed on 2026-06-15 after
the final fixture-family and verdict-sync slice landed and after the
post-closeout Nyquist audit reconciled the missing exact-home command evidence.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust workspace `cargo test` plus repository bootstrap checks and manual closeout review passes |
| **Config file** | Workspace [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | ~3000 seconds, workspace and cache dependent |

## Sampling Rate

- After every Rust or test-affecting change: run
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first.
- After every Phase 058 execution slice: run the slice-local release suites
  from the owning `058-0X-PLAN.md`, then run `cargo test --release`.
- Before final closeout: rerun bootstrap, the full `058-07` closeout packet,
  `cargo test --release`, `cargo doc --no-deps`, and `git diff --check`.
- Run `/GSD-Review-Tasks-Execution` in manual fallback mode at least three
  times and stop only after at least two consecutive passes show no
  significant issues.

## Final Closeout Command Packet

Bootstrap-first rule was honored on the final tree.

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -
  passed, then passed again after the `test_hjmt_import_export.rs`
  compatibility fix.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture` -
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture` -
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture` -
  passed.
- `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_split_brain_fencing -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_backend_conformance -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_import_export -- --nocapture` -
  passed after the manifest-reader compatibility fix.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_storage_boundary -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_privacy -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_evidence -- --nocapture` -
  passed.
- `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture` -
  passed.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_commit -- --nocapture` -
  passed during the 2026-06-15 validation refresh that reconciled the phase
  packet with the exact live homes from `058-03`.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_recovery -- --nocapture` -
  passed during the same validation refresh.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_transition_proofs -- --nocapture` -
  passed during the same validation refresh for the `058-06` proof-boundary
  exact homes.
- `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_privacy_regression -- --nocapture` -
  passed during the same validation refresh.
- `cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_e2e -- --nocapture` -
  passed during the same validation refresh.
- The canonical measured helper runner remained green on the accepted evidence
  home under `crates/z00z_storage/outputs/settlement/`, including the live
  `settlement_shard`, `settlement_hjmt`, and
  `settlement_hjmt_insert_single` helper packets driven by
  `./crates/z00z_storage/scripts/run_storage_settlement_bench.py`.
- `cargo test --release` - passed for the workspace on the final tree and
  again during the 2026-06-15 validation refresh.
- `cargo doc --no-deps` - passed with only pre-existing rustdoc warnings
  outside the Phase 058 scope.
- `git diff --check` - clean.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Evidence | Status |
|---------|------|------|-------------|------------|-----------------|-----------|----------|--------|
| `058-01-01` | `058-01` | 1 | `058-G1` | `058-01 threat_model` | Evidence, fixture, and archive-home claims stay source-honest on one canonical ledger. | integration + workspace | `058-01-SUMMARY.md` | ✅ green |
| `058-02-01` | `058-02` | 2 | `058-G2`, `058-G3` | `058-02 threat_model` | Release-lane simulator observability and stage sync stay on one public packet path. | integration + runtime | `058-02-SUMMARY.md` | ✅ green |
| `058-03-01` | `058-03` | 3 | `058-G4`, `058-G9`, `058-G10` | `058-03 threat_model` | Config changes are live, import/export is exact-home, and bad startup or bad state fails closed. | integration + storage | `058-03-SUMMARY.md` | ✅ green |
| `058-04-01` | `058-04` | 4 | `058-G5`, `058-G6` | `058-04 threat_model` | Runtime and publication packets stay on one lineage with failover and validator or watcher continuity. | integration + release lane | `058-04-SUMMARY.md` | ✅ green |
| `058-05-01` | `058-05` | 5 | `058-G7`, `058-G8`, `058-G12` | `058-05 threat_model` | Benchmark lanes stay honest, heavy-only scope stays explicit, and score packets do not outrun measured evidence. | tests + bench gates | `058-05-SUMMARY.md` | ✅ green |
| `058-06-01` | `058-06` | 6 | `058-G11` | `058-06 threat_model` | Scope birth, wallet proof boundary, historical replay, and occupancy privacy stay bound to imported artifacts. | integration + simulator | `058-06-SUMMARY.md` | ✅ green |
| `058-07-01` | `058-07` | 7 | `058-G13` | `058-07 threat_model` | Final fixture matrix and verdict sync stay honest without creating a second authority path. | integration + closeout | `058-07-SUMMARY.md` | ✅ green |

- `058-01-01 commands:` `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`, and `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture`.
- `058-02-01 commands:` `cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_runtime_config -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture`, and `cargo run --release -p z00z_simulator --bin scenario_1 --features test-params-fast`.
- `058-03-01 commands:` `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture`, `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_process -- --nocapture`, `cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_runtime_config -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_import_export -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_storage_boundary -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_backend_conformance -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_commit -- --nocapture`, and `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_recovery -- --nocapture`.
- `058-04-01 commands:` `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_planner -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_publish -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_join -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_migrate -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_split_brain_fencing -- --nocapture`, `cargo test -p z00z_validators --release --test test_hjmt_publication_contract -- --nocapture`, `cargo test -p z00z_watchers --release --test test_hjmt_publication_contract -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture`, and `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture`.
- `058-05-01 commands:` `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture`, `cargo bench -p z00z_storage --bench settlement_proofs --no-run`, `cargo bench -p z00z_storage --bench settlement_hjmt --no-run`, `cargo bench -p z00z_storage --bench settlement_shard --no-run`, `cargo bench -p z00z_storage --bench settlement_nested --no-run`, and `cargo bench -p z00z_storage --bench adaptive_policy_bench --no-run`.
- `058-06-01 commands:` `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_scope_birth -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_privacy -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_evidence -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_stage7_jmt_wallet_scan -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture`, `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario1_stage_surface -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_transition_proofs -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_privacy_regression -- --nocapture`, and `cargo test -p z00z_simulator --release --features test-params-fast --test test_hjmt_e2e -- --nocapture`.
- `058-07-01 commands:` `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_proof_negative -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`, `cargo test -p z00z_aggregators --release --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_bench_lanes -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_backend_conformance -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_import_export -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_storage_boundary -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_historical_proofs -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_privacy -- --nocapture`, `cargo test -p z00z_storage --release --features test-params-fast --test test_occupancy_evidence -- --nocapture`, and `cargo test -p z00z_simulator --release --features test-params-fast --test test_scenario_settlement -- --nocapture`.

## Manual-Only Verifications

All Phase 058 behaviors now have automated verification on one canonical path.

## Validation Audit 2026-06-15

| Metric | Count |
|--------|-------|
| Gaps found | 5 |
| Resolved | 5 |
| Escalated | 0 |

The workspace-first `/doublecheck` pass compared every plan-owned `--test`
command from `058-01-PLAN.md` through `058-07-PLAN.md` against the slice
summaries and this phase validation packet. The only gaps were missing command
evidence for `test_hjmt_batch_commit`, `test_hjmt_batch_recovery`,
`test_hjmt_transition_proofs`, `test_hjmt_privacy_regression`, and
`test_hjmt_e2e`; those suites were rerun green on 2026-06-15, the full
workspace `cargo test --release` gate reran green, and the phase artifacts
were synchronized.

A follow-up `/doublecheck` refresh on 2026-06-15 found three additional
phase-authority drifts: `test_hjmt_split_brain_fencing` was still missing from
the `058-04` validation packet despite being a live exact home, the final
evidence docs still classified `hist_flow.json` and `occ_flow.json` as
`pending_exact_home` after the release packet had already started emitting and
verifying them, and `058-CONTEXT.md` still grouped `hist_flow.json` and
`occ_flow.json` with the remaining `asset_flow.json` or `right_flow.json`
pending-home rows. Those drifts were corrected without changing the repository
verdict or introducing a second authority path.

## Review Loop

Manual fallback for `/.github/prompts/gsd-review-tasks-execution.prompt.md`
was used because the slash prompt is not a callable tool in this environment.

- Pass 1 found two real closeout issues:
  the final `058-07-SUMMARY.md`, `058-SUMMARY.md`, and `058-VALIDATION.md`
  artifacts were still missing even though `ROADMAP.md` and `STATE.md`
  already referenced them, and `test_hjmt_import_export.rs` still assumed the
  legacy `cases` manifest shape while the live `SRL` and `CPP` fixtures had
  moved to `golden` or `tamper`.
- Pass 2 reran the code and planning audit after the fixes against
  `058-TODO.md`, `058-CONTEXT.md`, `058-07-PLAN.md`,
  `058-EVIDENCE-LEDGER.md`, `ROADMAP.md`, `STATE.md`, and the touched fixture
  manifests. No significant issues remained.
- Pass 3 repeated the same audit after the broad validation wave and final
  closeout docs. No significant issues remained.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation Sign-Off

- [x] All seven Phase 058 execution tasks have automated verification
  recorded.
- [x] All plan-owned `--test` suites are now reflected in the slice summaries
  or in this phase validation packet.
- [x] No manual-only Phase 058 behaviors remain.
- [x] Bootstrap-first rule was honored on the final tree.
- [x] The final closeout release suites are green.
- [x] `cargo test --release` passed on the final tree.
- [x] `cargo doc --no-deps` passed with only pre-existing warnings outside
  scope.
- [x] `git diff --check` is clean.
- [x] The final verdict is `integrated upgrade` and matches the closed
  evidence ledger.

Approval: approved 2026-06-16
