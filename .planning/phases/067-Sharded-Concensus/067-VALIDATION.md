---
phase: 067
slug: 067-sharded-concensus
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-06
---

# Phase 067 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test --release` plus Python audit script and shell smoke harness |
| **Config file** | workspace `Cargo.toml`; phase-local packet under `.planning/phases/067-Sharded-Concensus/` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh && cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture && bash scripts/hjmt_local_devnet.sh --profile sim_5a7s --smoke --timeout 30 && python3 scripts/audit/audit_067_claims.py` |
| **Estimated runtime** | ~360 seconds |

---

## Sampling Rate

- **After every task commit:** Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- **After every plan wave:** Run `cargo test --release` plus the phase-local `scenario_11`, devnet-smoke, and claim-audit commands
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 360 seconds

---

## Per-Task Verification Map

Phase 067 has no `TASK-NNN` rows in its normative packet. Nyquist validation
therefore maps the completed phase to the authoritative verification slices
`TS-01` through `TS-19` from `067-TEST-SPEC.md`.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `TS-01` | `067-01` | `T1` | `PHASE-0 terminology and boundary honesty` | `T-067-01` | Active runtime and config surfaces keep one `secondary` vocabulary and reject stale `standby` names. | integration plus guard | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_terminology_guard -- --nocapture && cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_preflight -- --nocapture` | ✅ | ✅ green |
| `TS-02` | `067-02` | `T1` | `PHASE-1 deterministic subject and certificate contract` | `T-067-01, T-067-03` | Canonical subject, vote, and QC digests change on drift and reject malformed members or vote sets. | unit | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_commit_subject --test test_shard_quorum_certificate -- --nocapture` | ✅ | ✅ green |
| `TS-03` | `067-03` | `T1` | `PHASE-2 replay-before-vote gate` | `T-067-02, T-067-04` | Secondaries recompute the primary subject locally and reject drift before any vote is created. | unit plus recovery integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture` | ✅ | ✅ green |
| `TS-04` | `067-04` | `T1` | `PHASE-3 local shard QC integration` | `T-067-03, T-067-04` | Local shard quorum emits one real certificate and freezes or rejects conflicting or stale vote sets. | integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus --test test_local_quorum_certificate -- --nocapture` | ✅ | ✅ green |
| `TS-05` | `067-05` | `T2` | `PHASE-4 scenario_11 end-to-end path` | `T-067-01, T-067-02, T-067-06, T-067-07, T-067-08` | One package and one subject digest stay coherent from ingress through validator verdict and report artifacts. | end-to-end | `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture` | ✅ | ✅ green |
| `TS-06` | `067-06` | `T2` | `PHASE-5 lifecycle, join, readiness, and takeover safety` | `T-067-01, T-067-04` | Join, observer readiness, rotation, removal, and failover preserve route, generation, and lineage constraints. | integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_join --test test_recovery_failover -- --nocapture` | ✅ | ✅ green |
| `TS-07` | `067-07` | `T3` | `PHASE-6 DA and validator theorem binding` | `T-067-06, T-067-07` | Publication, subject, QC, theorem bundle, and validator flow stay bound to one canonical digest set. | DA plus validator integration | `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_quorum_binding -- --nocapture && cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture` | ✅ | ✅ green |
| `TS-08` | `067-08` | `T3` | `PHASE-7 signatures, transport, and equivocation evidence` | `T-067-02, T-067-05` | Signed envelopes, replay gates, and structured equivocation or payload evidence prevent injected or tampered votes from counting. | unit plus integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_signature_adapter --test test_transport_adapter --test test_equivocation_evidence -- --nocapture` | ✅ | ✅ green |
| `TS-09` | `067-09` | `T3` | `PHASE-8 BFT and Celestia-local claim boundary` | `T-067-03, T-067-06` | Simulated BFT math and Celestia-local artifact completeness are executable local contracts, not prose-only claims. | integration plus negative binding | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_bft_committee_rules -- --nocapture && cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture` | ✅ | ✅ green |
| `TS-10` | `067-10` | `T4` | `VERDICT-LCS-01 runnable process surface` | `T-067-01` | Runtime process entrypoints, canonical paths, and shard mapping contracts are executable and reject shadow or malformed paths. | integration | `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process -- --nocapture` | ✅ | ✅ green |
| `TS-11` | `067-11` | `T4` | `VERDICT-LCS-02 durable consensus evidence store` | `T-067-04, T-067-05` | Votes, QC anchors, and restart state round-trip through the canonical store or fail closed on drift or corruption. | storage integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_store -- --nocapture && cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_recovery_restart -- --nocapture` | ✅ | ✅ green |
| `TS-12` | `067-12` | `T4` | `VERDICT-LCS-03 planner authority boundary` | `T-067-01, T-067-08` | Every node recomputes planner truth from canonical local inputs and rejects stale route or mixed-generation drift. | planner integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_planner_authority -- --nocapture` | ✅ | ✅ green |
| `TS-13` | `067-13` | `T5` | `VERDICT-LCS-04 multi-process devnet harness` | `T-067-04` | Local process and devnet flows prove distinct identities, restart handling, stale-dir rejection, and smoke-path continuity. | process integration | `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process_devnet -- --nocapture` | ✅ | ✅ green |
| `TS-14` | `067-14` | `T5` | `VERDICT-LCS-05 network fault matrix` | `T-067-02, T-067-05` | Delay, replay, partition, heal, and reconnect simulation cannot manufacture fresh counted votes or silent conflicts. | transport fault integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_fault_matrix -- --nocapture` | ✅ | ✅ green |
| `TS-15` | `067-15` | `T5` | `VERDICT-LCS-06 HotStuff-local backend contract` | `T-067-03` | Local backend proves deterministic leader, timeout, view-change, and validator-bound backend-QC behavior. | backend integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hotstuff_local_backend -- --nocapture` | ✅ | ✅ green |
| `TS-16` | `067-16` | `T5` | `VERDICT-LCS-07 Celestia-local artifact conformance` | `T-067-06, T-067-07` | Local DA and Celestia-local artifacts remain complete and fail closed on namespace, commitment, payload, anchor, or validator drift. | artifact integration | `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding --test test_da_local_sim -- --nocapture` | ✅ | ✅ green |
| `TS-17` | `067-17` | `T6` | `VERDICT-LCS-08 structured evidence registry` | `T-067-05` | Every required safety failure emits digest-bound structured evidence and malformed records reject. | unit plus integration | `cargo test --release -p z00z_aggregators --features test-params-fast --test test_structured_evidence_registry -- --nocapture` | ✅ | ✅ green |
| `TS-18` | `067-18` | `T6` | `VERDICT-LCS-09 claim registry and report honesty` | `T-067-08` | Glossary claims are machine-auditable and report honesty rejects unsupported production-strength language. | audit plus simulator | `python3 scripts/audit/audit_067_claims.py && cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture` | ✅ | ✅ green |
| `TS-19` | `067-19/20/21` | `T6` | `VERDICT-LCS-10 final local-conformance gate` | `T-067-01, T-067-02, T-067-04, T-067-06, T-067-07, T-067-08` | Final closure consumes the addendum packet, devnet smoke, report honesty, claim audit, and diff hygiene on one canonical path. | end-to-end plus packet audit | `bash scripts/hjmt_local_devnet.sh --profile sim_5a7s --smoke --timeout 30 && cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture && git diff --check` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Audit 2026-07-06

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

### Audit Evidence

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` completed green and ended with `=== BOOTSTRAP COMPLETE ===`.
- `node .github/gsd-core/bin/gsd-tools.cjs loop render-hooks verify:post --raw` returned `_No active hooks at verify:post._` even though `.planning/config.json` keeps `"nyquist_validation": true`; this validation pass therefore ran as an explicit user-invoked manual fallback rather than a hook-driven step.
- `python3 scripts/audit/audit_067_claims.py` completed green with `claim audit ok: 50 glossary terms, 11 verdict terms, 61 registry rows`.
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_process_devnet -- --nocapture` completed green with `6` passing tests.
- `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture` completed green with `5` passing tests.
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_store -- --nocapture` completed green with `3` passing tests.
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_fault_matrix -- --nocapture` completed green with `5` passing tests.
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hotstuff_local_backend -- --nocapture` completed green with `5` passing tests.
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_terminology_guard -- --nocapture` completed green with `1` passing test.
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_commit_subject --test test_shard_quorum_certificate --test test_hjmt_consensus --test test_local_quorum_certificate --test test_hjmt_join --test test_signature_adapter --test test_transport_adapter --test test_equivocation_evidence --test test_bft_committee_rules --test test_structured_evidence_registry -- --nocapture` completed green with `30` passing tests.
- `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology --test test_hjmt_preflight --test test_hjmt_process --test test_da_local_quorum_binding --test test_celestia_local_binding --test test_da_local_sim -- --nocapture` completed green with `64` passing tests.
- Same-cycle corroborating reruns from the immediately preceding Phase 067 security pass also remained green for `test_planner_authority`, `test_secondary_replay_verifier`, `test_recovery_failover`, `test_consensus_recovery_restart`, `test_rollup_theorem_guard`, and `test_hjmt_publication_contract`.
- `bash scripts/hjmt_local_devnet.sh --profile sim_5a7s --smoke --timeout 30` completed green and emitted `reports/hjmt-local-devnet/sim-5a7s-20260706T151732Z/process-devnet-evidence.json`.
- `rg -n "standby|TakeoverStandby|standby_ids" crates/z00z_runtime crates/z00z_rollup_node crates/z00z_simulator config/hjmt_runtime/sim_5a7s --glob '!**/*.md'` returned no active hits; this is a successful zero-match audit.
- `git diff --check` completed green with no whitespace or patch-format errors.
- The plan-mandated `/GSD-Review-Tasks-Execution` runner is still not callable in the current runtime or hits prompt-token limits in the recorded summaries; this remains workflow-infrastructure debt, not a missing automated behavior test for Phase 067 itself.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 360s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-06
