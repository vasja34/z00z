# 067 Claim Audit

Status: final closeout audit synchronized with the recorded rerun  
Updated: 2026-07-06  
Scope: claim registry, component-presence truth, and final report-honesty inputs

## Inputs

- `.planning/phases/067-Sharded-Concensus/067-TODO.md`
- `.planning/phases/067-Sharded-Concensus/067-verdict.md`
- `.planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`
- `.planning/phases/067-Sharded-Concensus/067-19-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-20-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-21-PLAN.md`
- `scripts/audit/audit_067_claims.py`
- `crates/z00z_simulator/src/scenario_11/report.rs`
- `crates/z00z_simulator/src/scenario_11/mod.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

## Outputs

- deterministic pass or fail result for registry completeness
- deterministic pass or fail result for duplicate or malformed rows
- deterministic pass or fail result for missing verdict or glossary terms
- component-presence truth for the final Phase 067 closure packet
- frozen non-claims that remain outside the executable local scope

## Claim-Level Contract

- Every Phase 067 glossary term MUST have one row in `067-GLOSSARY-CLAIMS.md`.
- Every row MUST provide `term`, `code owner`, `artifact/API`, `positive test`,
  `negative test`, `claim level`, `evidence refs`, and `plan id`.
- Allowed claim levels are `live`, `simulated-full`, `live-claim-removed`, and
  `not-claimed`.
- `report_honesty.json` MUST expose one claim-level row per registry term.
- Bare overclaim strings MUST remain forbidden even when the underlying local
  seam exists.
- Unsupported production surfaces MUST stay visible in
  `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md`.

## Component Presence Matrix

| Component | Present | Partial | Stub | Missing | Evidence path | Required action |
| --- | --- | --- | --- | --- | --- | --- |
| `IngressBoundary` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/ingress.rs`; `crates/z00z_simulator/tests/test_scenario_11.rs::scenario11_happy_path_consistent` | keep as the only package-normalization entry point |
| `ShardRouteTable`, `BatchPlanner`, `PlannerAuthority` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/batch_planner.rs`; `crates/z00z_runtime/aggregators/tests/test_planner_authority.rs` | keep planner truth deterministic and fail closed on drift |
| `DistScheduler` and `DistDispatch` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/dist_scheduler.rs`; `crates/z00z_runtime/aggregators/src/dist_dispatch.rs`; scenario fault id `primary_offline_before_dispatch` | preserve defer-not-reroute owner semantics |
| `CommitSubject`, `JournalCandidate`, `SecondaryReplayVerifier`, `ShardVote`, `ShardQuorumCertificate`, `ConsensusAdapter` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/commit_subject.rs`; `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`; `crates/z00z_runtime/aggregators/tests/test_local_quorum_certificate.rs` | preserve one canonical local quorum truth path |
| `ConsensusStore` and `RecoveryBoundary` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/consensus_store.rs`; `crates/z00z_runtime/aggregators/src/recovery.rs`; `crates/z00z_runtime/aggregators/tests/test_consensus_recovery_restart.rs`; `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs` | keep the exact-certificate resume and anti-failback evidence bound to the recorded rerun |
| `VoteSigner`, signature verifier, and `VoteTransport` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/signature.rs`; `crates/z00z_runtime/aggregators/src/transport.rs`; `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`; `crates/z00z_runtime/aggregators/tests/test_transport_fault_matrix.rs` | preserve non-authoritative helper status and replay-before-vote gate |
| `EvidenceRecord` and structured evidence registry | yes | no | no | no | `crates/z00z_runtime/aggregators/src/evidence.rs`; `crates/z00z_runtime/aggregators/tests/test_structured_evidence_registry.rs` | keep digest-bound evidence ids and forbid string-only closure |
| `BftCommittee`, `BftEngine`, and `HotstuffLocal` | yes | no | no | no | `crates/z00z_runtime/aggregators/src/bft_committee.rs`; `crates/z00z_runtime/aggregators/src/bft_engine.rs`; `crates/z00z_runtime/aggregators/src/hotstuff_local.rs`; `crates/z00z_runtime/aggregators/tests/test_hotstuff_local_backend.rs` | keep claim level at local executable only; no network BFT overclaim |
| `LocalDaAdapter`, `CelestiaLocalAdapter`, and validator binding | yes | no | no | no | `crates/z00z_rollup_node/src/da.rs`; `crates/z00z_rollup_node/src/celestia_local.rs`; `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs` | keep detached DA/theorem paths fail closed |
| `scenario_11`, `fault_matrix.json`, and `report_honesty.json` | yes | no | no | no | `crates/z00z_simulator/src/scenario_11/mod.rs`; `crates/z00z_simulator/src/scenario_11/report.rs`; `crates/z00z_simulator/tests/test_scenario_11.rs` | keep the flow-alias mapping explicit where addendum names differ from scenario ids |
| `067-GLOSSARY-CLAIMS.md` | yes | no | no | no | `.planning/phases/067-Sharded-Concensus/067-GLOSSARY-CLAIMS.md`; `python3 scripts/audit/audit_067_claims.py` | keep registry synced with allowed claim levels |
| `067-FINAL-CONFORMANCE.md` | yes | no | no | no | `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md`; `.planning/phases/067-Sharded-Concensus/067-19-SUMMARY.md`; `.planning/phases/067-Sharded-Concensus/067-21-SUMMARY.md` | keep the recorded rerun packet authoritative |

## Command

```bash
python3 scripts/audit/audit_067_claims.py
```

## Evidence Gate

- The audit command MUST pass with no missing or duplicate registry rows.
- Claim-level wording in `067-GLOSSARY-CLAIMS.md`,
  `067-CLAIM-AUDIT.md`, `067-FINAL-CONFORMANCE.md`, and
  `report_honesty.json` MUST stay consistent.
- Shared scenario rows used as aliases for stricter addendum flow ids MUST stay
  labeled explicitly in the final conformance document.

## Current Audit Result

- `python3 scripts/audit/audit_067_claims.py` passes on the final branch state.
- Output: `claim audit ok: 50 glossary terms, 11 verdict terms, 61 registry rows`.
- The final `report_honesty.json` packet records `37 live`,
  `18 simulated-full`, `6 live-claim-removed`, and `0 not-claimed` rows.
- The recorded rerun artifacts under `reports/phase-067/20260706T120602Z/`
  and `reports/hjmt-local-devnet/20260706T120602Z/` are now consumed by
  `.planning/phases/067-Sharded-Concensus/067-FINAL-CONFORMANCE.md`.

## Frozen Non-Claims

- `planner HA` remains `live-claim-removed` unless a separate durable and
  tested planner service exists.
- External network BFT, production HotStuff, external Celestia provider
  operation or finality, slashing, and economic or public finality remain
  non-live until independently implemented and tested.
- Production signatures remain non-claimed at the external-network scope; the
  phase proves only the local signer or verifier seam and replay gate.
