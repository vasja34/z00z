---
phase: 067
slug: 067-sharded-concensus
status: verified
threats_open: 0
asvs_level: 1
register_authored_at_plan_time: false
created: 2026-07-06
---

# Phase 067 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

Retroactive STRIDE audit. Phase 067 has executed summaries but no formal
`<threat_model>` blocks in its numbered plans and no `## Threat Flags` sections
in its numbered summaries, so the register below was built from the live
implementation, the phase authority in
`.planning/phases/067-Sharded-Concensus/067-CONTEXT.md`, and release-mode
verification commands run on 2026-07-06.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Package ingress -> planner | Wallet-style ingress normalization must preserve one canonical route key and admission digest before planning or replay. | package digest, route key, batch id, ordered item digests |
| Planner / placement -> consensus | Route-table ownership, routing generation, and ready-member membership must stay identical across planning, subject construction, and quorum collection. | route table digest, routing generation, membership digest, subject digest |
| Recovery record -> durable state | Restart and takeover paths must prove lineage, root, proof-version, and policy-generation continuity against committed recovery state. | journal lineage, state root, proof version, bucket policy generation, batch id |
| Transport / replay -> vote service | In-memory vote transport may be delayed, replayed, or partial, but duplicate envelopes and stale replayed subjects must fail closed. | message id, voter id, payload digest, theorem digest, replay verdict |
| Local DA -> published batch | Publication metadata must stay bound to one subject, one certificate, one theorem digest, and one local payload contract. | publication binding digest, checkpoint id, certificate digest, theorem digest, payload digest |
| Rollup node -> validator theorem seam | The accepted path must cross one canonical theorem verifier and one validator-owned checkpoint flow. | theorem bundle, checkpoint link, publication route, subject, certificate, verdict |
| Scenario report -> claim registry | Simulation artifacts may describe only what local evidence proves and must reject overclaims or missing evidence refs. | evidence ids, artifact digests, claim levels, glossary terms |

## Threat Register

| Threat ID | Category | Component | Severity | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|----------|-------------|------------|----------|--------|
| T-067-01 | Tampering | planner authority and route ownership | high | mitigate | Reject route, generation, and membership drift before commit, recovery, or preflight; keep one canonical route and membership path. | `.planning/phases/067-Sharded-Concensus/067-CONTEXT.md:349-363`; `crates/z00z_runtime/aggregators/src/consensus_adapter.rs:110-163`; `crates/z00z_runtime/aggregators/src/recovery.rs:87-176`; `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs:74,104`; `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs:64,79,127` | closed |
| T-067-02 | Replay | secondary replay and duplicate vote transport | high | mitigate | Rebuild ingress-normalized work items locally, replay the exact subject, reject named drift axes with stable codes, and suppress duplicate envelopes before vote creation. | `crates/z00z_runtime/aggregators/src/secondary_replay.rs:80-150,154-278,304-333`; `crates/z00z_runtime/aggregators/src/service.rs:191-275`; `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs:78`; `crates/z00z_simulator/tests/test_scenario_11.rs:266-320` | closed |
| T-067-03 | Integrity | quorum certificate math and HotStuff-local backend | high | mitigate | Certificate construction and verification require one membership digest, valid signatures, unique voters, intersecting threshold math, and a canonical shard certificate wrapped by the local backend. | `crates/z00z_runtime/aggregators/src/shard_quorum_certificate.rs:93-213,228-300`; `crates/z00z_runtime/aggregators/src/hotstuff_local.rs:283-310,393-417,497-517`; `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs:27-72` | closed |
| T-067-04 | Availability | failover, restart, and stale-lineage reentry | high | mitigate | Recovery boundary rejects wrong generation, wrong lineage, stale root, stale restart metadata, and unlawful old-primary reentry or failback. | `crates/z00z_runtime/aggregators/src/recovery.rs:87-185`; `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs:329`; `crates/z00z_runtime/aggregators/tests/test_consensus_recovery_restart.rs:155`; `crates/z00z_simulator/tests/test_scenario_11.rs:221-239,417-429` | closed |
| T-067-05 | Repudiation | structured evidence registry and fault attribution | medium | mitigate | Evidence objects require canonical digests and artifact refs for equivocation, payload withholding, missing blob, wrong route, stale member, split-brain, and transport-fault outcomes. | `crates/z00z_runtime/aggregators/src/evidence.rs:113-151,206-236,273-305,401-435,467-503,535-566,610-701`; `crates/z00z_runtime/aggregators/tests/test_structured_evidence_registry.rs:56,68,98,107,117`; `crates/z00z_simulator/tests/test_scenario_11.rs:145-183,241-320` | closed |
| T-067-06 | Tampering | local DA publication and theorem binding | high | mitigate | Local DA publish and resolve recompute payload, publication, subject, certificate, and theorem digests and reject metadata drift or replay. | `crates/z00z_rollup_node/src/da.rs:179-232,236-327`; `crates/z00z_runtime/validators/src/checkpoint.rs:88-121`; `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs:8-23,58-72`; `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs:279-309` | closed |
| T-067-07 | Integrity | canonical validator theorem boundary | high | mitigate | `z00z_validators` owns `verify_settlement_theorem`; `z00z_rollup_node` only re-exports that path, and checkpoint flow rejects detached subject, certificate, publication, or theorem state. | `crates/z00z_runtime/validators/src/lib.rs:23-40`; `crates/z00z_runtime/validators/src/verdict.rs:115-129,239-255`; `crates/z00z_runtime/validators/src/checkpoint.rs:22-80,88-127`; `crates/z00z_rollup_node/src/lib.rs:45-46`; `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs:49,87,98,111` | closed |
| T-067-08 | Repudiation | report honesty and claim registry truth | medium | mitigate | Scenario 11 and the Phase 067 claim-audit script keep only evidence-backed live terms, reject overclaims, and require each registry row to carry executable evidence refs. | `crates/z00z_simulator/tests/test_scenario_11.rs:323-405`; `scripts/audit/audit_067_claims.py:16-26,46-87,133-156`; `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md:101-110,221-226` | closed |

*Status: open · closed · open - below high threshold (non-blocking)*
*Severity: critical > high > medium > low - only open threats at or above workflow.security_block_on count toward threats_open*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

## Unregistered Flags

None. No `067-*-SUMMARY.md` file currently includes a `## Threat Flags` section
that introduces an extra unmapped threat outside the register above.

## Accepted Risks Log

No accepted risks.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-06 | 8 | 8 | 0 | Codex (`/gsd-secure-phase 067`) |

## Audit Evidence

Commands executed during this audit pass:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_planner_authority -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- `cargo test --release -p z00z_aggregators --features test-params-fast --test test_consensus_recovery_restart -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_da_local_quorum_binding -- --nocapture`
- `cargo test --release -p z00z_rollup_node --test test_rollup_theorem_guard -- --nocapture`
- `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- `cargo test --release -p z00z_simulator --test scenario_11 -- --nocapture`
- `python3 scripts/audit/audit_067_claims.py`
- `git diff --check`

Observed results used for closure:

- `bootstrap_tests.sh` completed green and ended with `=== BOOTSTRAP COMPLETE ===`.
- `test_planner_authority` passed `5` release tests, including stale-route and mixed-generation rejects.
- `test_secondary_replay_verifier` passed `3` release tests, including exact replay acceptance plus stale-secondary rejection.
- `test_recovery_failover` passed `4` release tests, and `test_consensus_recovery_restart` passed `3` more for stale-root and exact-certificate restart recovery.
- `test_da_local_quorum_binding` passed `3` release tests, including detached-certificate rejection and majority-intersection proof.
- `test_rollup_theorem_guard` passed `11` release tests, including detached certificate and detached publication rejection on the validator path.
- `test_hjmt_publication_contract` passed `16` release tests for checkpoint-flow, route-drift, theorem-drift, and stale-membership gates.
- `scenario_11` passed `5` release tests covering happy path, fault matrix, claim registry, process-devnet fault contract, and report honesty.
- `audit_067_claims.py` completed green with `claim audit ok: 50 glossary terms, 11 verdict terms, 61 registry rows`.

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-06
