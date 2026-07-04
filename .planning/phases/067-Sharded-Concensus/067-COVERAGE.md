# Phase 067 Coverage Appendix

**Source:** `.planning/phases/067-Sharded-Concensus/067-TODO.md`
**Generated:** 2026-07-03
**Status:** Planning coverage ledger

## Coverage Audit

- Unique `TASK-NNN` identifiers in `067-TODO.md`: `0`
- Required GSD Plan Groups: `9`
- Required group source: `14.1` through `14.9`
- Coverage rule: each required implementation group maps to exactly one
  `067-NN-PLAN.md`
- Exact traceability source:
  `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md` locks all `19`
  H2 sections, all `55` H3 sections, `447` dash-list bullets, and `80`
  numbered-list items from `067-TODO.md`.
- Duplicate task status: none
- Missing task status: none
- Planning fail condition: if `TASK-NNN` rows appear later, or if any required
  group is renamed, split, merged, or dropped, this appendix and all plan
  mappings must be regenerated before execution.

## Task-To-Plan Coverage Table

| Task row | PLAN id | Source refs | Inputs | Artifacts | Tests | Expected results | Simulation requirement | Anti-placeholder proof | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PHASE-0` Terminology and boundary cleanup | `067-01` | `067-TODO.md` `3.6`, `3.7`, `14.1`, `16`, `17`, `19`; `090-New-Scenarios` `15.3` | live `standby` names, CFT/BFT drift, `sim_5a7s` config and tests | renamed runtime/config/docs/tests, no-alias guard, updated topology fixtures | targeted aggregator and rollup topology tests, active-code grep audit | one live protocol term: `secondary aggregator`; local seam described as CFT until stronger proof exists | local config/home loading and quorum tests only; no external network | active code, config, and tests change together; no alias or doc-only rename | complete |
| `PHASE-1` Commit subject and certificate types | `067-02` | `067-TODO.md` `14.2`, `9.4`, `9.5`, `9.6`, `19`; `090-New-Scenarios` `15.5`, `15.9` | route digest, placement digest, plan digest, state roots, lineage, proof metadata | `commit_subject.rs`, `shard_vote.rs`, `shard_quorum_certificate.rs`, root exports, digest tests | new unit tests for canonical encode, drift sensitivity, vote and certificate rejection | quorum artifacts become first-class and deterministic | repeat-encode and mutation fixtures with real route/placement/recovery inputs | real binary encoding, domain bytes, and negative tests; no constant digest | complete |
| `PHASE-2` Secondary replay verifier | `067-03` | `067-TODO.md` `14.3`, `9.3`, `13.1`, `19`; `090-New-Scenarios` `15.5`, `15.9`, `15.10` | ingress-normalized package, route lookup, placement view, recovery record, publication binding, theorem digest | `secondary_replay.rs`, verifier result types, replay tests, fixture helpers | new unit/integration tests for exact replay acceptance and drift rejection | secondary votes mean independent deterministic replay | local replay uses real planner, recovery, DA, and theorem inputs; no copied primary bytes | vote creation must depend on recomputed subject, not fixture constants | complete |
| `PHASE-3` Local quorum certificate integration | `067-04` | `067-TODO.md` `14.4`, `4.3`, `10.3`, `19`; `090-New-Scenarios` `15.10` | current `ConsensusAdapter`, live membership rules, new vote/certificate artifacts | updated `consensus_adapter.rs`, extended commit path, consensus tests, publication handoff checks | targeted consensus tests for honest parity, split-brain freeze, removed/unready voter rejection | local majority path and certificate path agree on honest inputs | same-term and mixed-membership local conflicts simulated with real records | certificate path must drive real commit decisions; no shadow DTO path | complete |
| `PHASE-4` End-to-end `sim_5a7s` harness | `067-05` | `067-TODO.md` `14.5`, `13`, `19`; `090-New-Scenarios` `15.1`-`15.15` | wallet-style package fixture, route table, placement table, recovery, local DA, validator | independent `scenario_11` home, route-bound publication evidence, quorum JSON artifacts, package-to-validator harness | unit, integration, and E2E `scenario_11` tests including happy path, dual-primary path, all-shard sweep, offline-owner defer, and crash resume | one local package-to-validator flow binds one subject digest through replay, certificate, DA, and validator while offline-owner paths defer instead of rerouting | real `sim_5a7s` routing, planning, publication, and validator boundaries with only external transport faked | evidence files must be produced from live runs; no scenario_1 piggyback or report-only closure | complete |
| `PHASE-5` Join, removal, and rotation simulation | `067-06` | `067-TODO.md` `14.6`, `10.4`-`10.7`, `19`; `090-New-Scenarios` `15.11`, `15.12` | ready-state transitions, recovery lineage, route generation, takeover rules | extended join/failover/route-rollout tests, scenario fault matrix, transition evidence | targeted runtime tests plus scenario_11 crash, stale, and mixed-generation cases | topology changes become certificate-safe and fail closed when state drifts | simulated join, removal, planned rotation, emergency takeover, partition/heal, restart | no transition may succeed through docs or config only; tests must observe real vote eligibility changes | planned |
| `PHASE-6` Validator and theorem binding | `067-07` | `067-TODO.md` `14.7`, `4.5`, `11`, `12`, `19`; `090-New-Scenarios` `15.5`, `15.10`, `15.14`, `15.15` | local DA publication, theorem bundle, checkpoint flow, resolved batch, certificate digest | DA binding fields, validator gate, theorem/link checks, new rollup and validator tests | validator rejects missing, detached, stale, or mismatched certificate binding | local DA resolve and validator acceptance use real publication/theorem/certificate state | resolved batches and theorem bundles must be validated against live certificate digests | no constant certificate digest or ignored gate path may survive tests | planned |
| `PHASE-7` Network and signature adapter | `067-08` | `067-TODO.md` `14.8`, `7.1`, `8.6`, `12`, `19`; `090-New-Scenarios` `15.7`, `15.13`, `15.14` | current vote path, local replay verifier, `z00z_crypto` domain and signature primitives | signature trait, deterministic simulator signature seam, in-memory vote transport, equivocation evidence format, transport tests | local tests for real signatures, transport mediation, and equivocation evidence | external transport is faked locally, but replay, signatures, and evidence are real | in-memory or loopback transport only; no live libp2p required | transport cannot bypass replay verifier; equivocation evidence must be emitted from live conflicting votes | planned |
| `PHASE-8` BFT and Celestia local backend | `067-09` | `067-TODO.md` `14.9`, `8.1`-`8.12`, `11.2`, `12`; `090-New-Scenarios` `15.7`, `15.13`, `15.15` | proven local subject interface, simulated larger committees, local external-DA adapter, validator gate | local BFT backend adapter, local Celestia-style blob adapter, 3f+1 committee fixtures, scenario or rollup tests | simulated BFT quorum tests, local Celestia resolution tests, validator independence tests | all external network and DA behavior stays local and deterministic while using real subject, vote, publication, and validator logic | simulated 7/10/13-node committees and local blob retrieval only | no future-only backend claims; tests must prove 3f+1 or 2f+1 semantics and artifact equality | planned |

## Exact Mapping Assertion

Each required group appears once and only once:

- `PHASE-0` -> `067-01-PLAN.md`
- `PHASE-1` -> `067-02-PLAN.md`
- `PHASE-2` -> `067-03-PLAN.md`
- `PHASE-3` -> `067-04-PLAN.md`
- `PHASE-4` -> `067-05-PLAN.md`
- `PHASE-5` -> `067-06-PLAN.md`
- `PHASE-6` -> `067-07-PLAN.md`
- `PHASE-7` -> `067-08-PLAN.md`
- `PHASE-8` -> `067-09-PLAN.md`
