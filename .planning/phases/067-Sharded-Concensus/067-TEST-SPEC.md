---
phase: 067
artifact: test-spec
status: planned
updated: 2026-07-03
---

# Phase 067 Test Specification

This document defines the phase-local verification contract for Phase 067. It is derived from `067-TODO.md`, `067-CONTEXT.md`, `067-01-PLAN.md` through `067-09-PLAN.md`, the locked Scenario 11 source row in `.planning/phases/090-New-Scenarios/066-TODO.md`, and the live boundary document `crates/z00z_runtime/aggregators/README.md`.

## 🎯 Scope

- Prove the shard-local quorum-certificate workflow end to end with real project primitives.
- Treat the Phase 067 planning packet as normative. There are no `TASK-NNN` rows in this phase; the required verification slices are `PHASE-0` through `PHASE-8`.
- Keep one implementation authority. Do not create a parallel consensus layer, duplicate DTO family, duplicate report writer, or alternate validator gate.
- Keep `scenario_11` independent from `scenario_1`, but follow the existing simulator test layout where it helps reuse repository patterns without reusing scenario logic.
- Allow simulation only for external transport, external DA transport, remote-process boundaries, and deterministic fault scheduling. Routing, placement, replay, certificate validation, publication binding, validator checks, lineage, and state paths must stay real.

## 📚 Attached Source Corpus

### ✅ Normative planning sources

- `.planning/phases/067-Sharded-Concensus/067-TODO.md`
- `.planning/phases/067-Sharded-Concensus/067-CONTEXT.md`
- `.planning/phases/067-Sharded-Concensus/067-01-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-02-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-03-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-04-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-05-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-06-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-07-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-08-PLAN.md`
- `.planning/phases/067-Sharded-Concensus/067-09-PLAN.md`

### 🔒 Coverage and drift guards

- `.planning/phases/067-Sharded-Concensus/067-COVERAGE.md`
- `.planning/phases/067-Sharded-Concensus/067-SOURCE-AUDIT.md`
- `.planning/phases/067-Sharded-Concensus/067-PLAN-REVIEW.md`

### 🔗 Required linked source rows

- `.planning/phases/090-New-Scenarios/066-TODO.md` section `15` (`scenario_11`)
- `crates/z00z_runtime/aggregators/README.md`

### ⚠️ Non-authoritative supporting material

- `.planning/phases/067-Sharded-Concensus/wiki -results.md`
- The stale `Agg-Concensus-Spec.md` reference remains drift evidence only and must not be used as a replacement authority.

## 🧭 Coverage Contract

Phase 067 requires exactly nine verification slices. Each slice maps one-to-one to a required plan group and must stay traceable to its source packet.

| Test slice | Required group | Plan | Primary proof target | Required levels |
| --- | --- | --- | --- | --- |
| `TS-01` | `PHASE-0` | `067-01` | terminology and boundary cleanup | config tests, topology integration, grep audit |
| `TS-02` | `PHASE-1` | `067-02` | deterministic `CommitSubject`, `ShardVote`, and `ShardQuorumCertificate` | unit and artifact validation |
| `TS-03` | `PHASE-2` | `067-03` | `SecondaryReplayVerifier` fail-closed gating | unit and recovery integration |
| `TS-04` | `PHASE-3` | `067-04` | live local consensus emits a real certificate | integration and conflict safety |
| `TS-05` | `PHASE-4` | `067-05` | independent `scenario_11` package-to-validator flow | simulator E2E, routing, DA, validator |
| `TS-06` | `PHASE-5` | `067-06` | join, readiness, rotation, takeover, removal safety | runtime integration and simulator lifecycle E2E |
| `TS-07` | `PHASE-6` | `067-07` | certificate-aware DA and validator theorem binding | DA integration, validator contract, proof harness |
| `TS-08` | `PHASE-7` | `067-08` | signature, transport, equivocation, payload withholding | adapter tests and simulator conformance |
| `TS-09` | `PHASE-8` | `067-09` | simulated BFT and Celestia backend claims | committee rules, blob binding, degraded-mode E2E |

No slice may close on compile-only evidence, docs-only evidence, or report-only artifacts.

## 🔐 Global Invariants

| Invariant | What must be observed | Proof surfaces |
| --- | --- | --- |
| `INV-01 terminology honesty` | active code, config, tests, and errors use `secondary`, not `standby` | `TS-01`, grep audit, topology/preflight tests |
| `INV-02 local CFT honesty` | the live path is shard-local deterministic CFT, not network BFT | `TS-01`, `TS-05`, `report_honesty.json` |
| `INV-03 canonical subject digest` | the same live subject always encodes to the same domain-separated digest | `TS-02`, `TS-03`, `TS-05` |
| `INV-04 drift sensitivity` | route, generation, root, lineage, proof, policy, publication, theorem, signer, or membership drift changes the digest or rejects | `TS-02`, `TS-03`, `TS-07`, `TS-08`, `TS-09` |
| `INV-05 replay-before-vote` | no vote may be created without replaying the exact primary subject against local state | `TS-03`, `TS-05`, `TS-08` |
| `INV-06 local quorum only` | each shard forms its own `2-of-3` certificate; no global `5-of-5` quorum exists | `TS-04`, `TS-05`, `TS-06` |
| `INV-07 same-term conflict safety` | conflicting same-term subjects freeze or reject before dual commit | `TS-01`, `TS-04`, `TS-08` |
| `INV-08 membership and generation safety` | only active, ready, same-generation members may contribute to a certificate | `TS-02`, `TS-04`, `TS-06`, `TS-09` |
| `INV-09 lineage and restart safety` | takeover and resume require matching lineage, generation, root, and exact certificate/publication state | `TS-06`, `TS-05` |
| `INV-10 publication and validator alignment` | certificate, publication binding, theorem bundle, resolved batch, and validator verdict refer to one subject | `TS-05`, `TS-07`, `TS-09` |
| `INV-11 transport cannot bypass core checks` | in-memory delivery cannot inject or count unreplayed votes | `TS-08` |
| `INV-12 overclaim rejection` | reports must not claim live BFT, Celestia finality, slashing, or production signatures unless the simulated proof slice explicitly covers that claim | `TS-05`, `TS-09` |

## 🧪 Slice Contracts

### ✅ TS-01 Terminology And Boundary Cleanup

- Maps to `PHASE-0` and `067-01`.
- Demonstrates that the live seam uses one vocabulary and one honest boundary statement.
- Must extend current config and topology owners instead of introducing compatibility aliases.
- Positive proof:
  - `sim_5a7s` config reload succeeds with `secondary` fields only.
  - generated temporary homes derived from `sim_5a7s` also reload with renamed `secondary` fields only.
  - topology and preflight tests still derive the same shard ownership and generation invariants.
  - split-brain freeze behavior is unchanged after the breaking rename.
- Negative proof:
  - stale `standby` keys reject at parse time.
  - unknown, duplicate, or self-referential secondary ids reject.
  - any active code or config hit from `rg -n "standby|TakeoverStandby|standby_ids"` fails the slice.
- Success conditions:
  - no active alias survives;
  - targeted tests pass;
  - grep audit is clean;
  - docs and error strings describe local CFT only.

### ✅ TS-02 Commit Subject And Certificate Types

- Maps to `PHASE-1` and `067-02`.
- Demonstrates that quorum artifacts are real runtime-owned types with deterministic binary encodings and domain separation.
- Required mutation matrix:
  - route digest
  - placement generation
  - shard root
  - lineage
  - proof version
  - policy generation
  - active membership digest
  - term
  - voter identity and role
- Positive proof:
  - the same live subject re-encodes to the same digest.
  - a valid vote set from active members yields one certificate.
  - the certificate builder accepts only the active placement members for one shard in one generation.
- Negative proof:
  - duplicate voters reject;
  - inactive voters reject;
  - wrong role rejects;
  - mixed membership digests reject;
  - mixed terms reject;
  - below-quorum vote sets reject.
- Success conditions:
  - `CommitSubject`, `ShardVote`, and `ShardQuorumCertificate` are exported through the existing aggregator facade;
  - tests prove both digest stability and drift sensitivity;
  - certificate validation fails closed before any DA or validator step.

### ✅ TS-03 Secondary Replay Verifier

- Maps to `PHASE-2` and `067-03`.
- Demonstrates that secondaries recompute the primary subject from live inputs and refuse to vote on drifted local state.
- Critical integration path:
  - ingress -> planner -> placement -> recovery -> publication binding -> theorem digest -> replay verdict.
- Positive proof:
  - a ready secondary accepts the exact primary subject.
  - route, plan, root, lineage, proof, policy, publication binding, and theorem digest mutations are exercised independently.
- Negative proof:
  - wrong route;
  - wrong plan digest;
  - wrong root;
  - wrong lineage;
  - wrong proof version;
  - wrong policy generation;
  - wrong publication binding;
  - wrong theorem digest;
  - stale secondary recovery state.
- Success conditions:
  - every named drift axis blocks vote creation before signing;
  - reject outputs are stable enough for deterministic fault-matrix assertions;
  - later transport and simulator slices consume the replay result instead of bypassing it.

### ✅ TS-04 Local Quorum Certificate Integration

- Maps to `PHASE-3` and `067-04`.
- Demonstrates that the live shard-local commit path emits a real `ShardQuorumCertificate` rather than a wrapped majority result.
- Positive proof:
  - honest local quorum returns one certificate-bound commit decision;
  - the certificate path yields the same honest decision as the current local majority seam;
  - the membership digest in the certificate matches exactly one shard and one active generation.
- Negative proof:
  - duplicate, removed, joined-but-not-ready, mixed-membership, or mixed-term voters reject;
  - conflicting same-term subjects cannot both commit.
- Success conditions:
  - the live commit path exposes or references the certificate artifact;
  - same-term freeze remains intact;
  - no active commit path can bypass certificate validation.

### ✅ TS-05 Scenario 11 End To End Harness

- Maps to `PHASE-4` and `067-05`.
- Demonstrates the full local package-to-validator path in an independent `scenario_11`.
- Critical end-to-end behaviors that must be proven:
  - package ingress, routing, planning, replay, certificate formation, DA publication, and validator verdict all bind the same subject;
  - each shard forms its own `2-of-3` local quorum;
  - owner-path resolution remains correct when one aggregator owns two shards;
  - no global `5-of-5` quorum is counted anywhere.
- Positive proof:
  - one-shard happy path;
  - dual-primary owner path;
  - all-shard sweep across the full `sim_5a7s` topology;
  - one-secondary-offline honest `2-of-3` path;
  - post-quorum pre-DA resume from the exact certificate.
- Negative proof:
  - pre-quorum primary crash prevents certificate and DA publication;
  - stale secondary rejects;
  - wrong route, wrong dispatch owner, wrong planner digest, or wrong subject drift rejects;
  - report honesty rejects unsupported BFT, Celestia, slashing, production-signature, or public-finality claims.
- Success conditions:
  - `scenario_11` is independent from `scenario_1`;
  - JSON evidence proves end-to-end alignment;
  - honesty output prevents overclaim.

### ✅ TS-06 Join Removal And Rotation Simulation

- Maps to `PHASE-5` and `067-06`.
- Demonstrates safe lifecycle transitions under the same digest, generation, and lineage rules as steady-state quorum.
- Positive proof:
  - observer catch-up to ready-secondary voting state;
  - planned rotation at a checkpoint or generation boundary;
  - rolling secondary replacement keeps the same shard live once the replacement becomes ready;
  - emergency takeover after a primary crash with matching lineage and generation;
  - takeover on one shard preserves unrelated shard continuity and does not merge committees;
  - restart after crash resumes only the exact proven certificate/publication state;
  - partition-heal path recovers without conflicting certificates.
- Negative proof:
  - unready observer vote rejects;
  - removed member vote rejects;
  - mixed-generation certificate rejects;
  - mixed-lineage vote sets reject;
  - old primary commit after rotation rejects;
  - stale lineage, stale generation, or stale route-generation takeover rejects;
  - divergent-root takeover rejects;
  - detached-certificate resume rejects;
  - offline minority cannot synthesize a quorum.
- Success conditions:
  - readiness is proven before voting;
  - old primaries cannot keep committing after rotation;
  - same-shard continuity is proven across replacement or takeover with no route drift or double-commit;
  - unaffected shards keep progressing while another shard rotates or heals;
  - lifecycle telemetry is deterministic and scenario-owned.

### ✅ TS-07 Validator And Theorem Binding

- Maps to `PHASE-6` and `067-07`.
- Demonstrates that downstream acceptance requires the quorum artifact and not proposer trust alone.
- Critical integration path:
  - certificate-producing commit -> local DA publication -> local DA resolve -> theorem bundle -> validator checkpoint/engine/verdict.
- Positive proof:
  - publication carries a certificate digest or reference;
  - resolve preserves the same binding;
  - validator accepts only when certificate, theorem, publication binding, and ordered batch share one subject digest;
  - a proof or exhaustive small-state harness demonstrates local CFT agreement.
- Negative proof:
  - missing certificate rejects when the gate is enabled;
  - detached or mismatched certificate rejects at resolve or validator boundary;
  - detached publication binding rejects theorem validation;
  - stale certificate from inactive or mixed membership rejects.
- Success conditions:
  - current DTO families remain authoritative;
  - certificate-aware binding survives publish and resolve;
  - validator trust shifts to proof-bearing subject consistency.

### ✅ TS-08 Network And Signature Adapter

- Maps to `PHASE-7` and `067-08`.
- Demonstrates future transport seams without weakening local replay-first semantics.
- Positive proof:
  - local conformance runs with in-memory transport only;
  - signature validation accepts only the correct signer and digest bindings;
  - conflicting same-voter same-term different-subject votes emit deterministic equivocation evidence;
  - payload withholding emits deterministic evidence or degraded state.
- Negative proof:
  - a transport-delivered vote without replay verification rejects;
  - duplicate or replayed messages do not double count;
  - wrong signer, wrong membership digest, or wrong subject digest rejects;
  - missing payload cannot silently pass or create synthetic votes.
- Success conditions:
  - replay remains the only path to vote creation;
  - evidence is locally serializable and contains real conflicting vote material;
  - `scenario_11` stays green using only deterministic local adapters.

### ✅ TS-09 BFT And Celestia Backend

- Maps to `PHASE-8` and `067-09`.
- Demonstrates that all future BFT and Celestia claims are backed by local executable proof instead of naming.
- Positive proof:
  - local deterministic tests prove `3f+1` membership and `2f+1` quorum rules;
  - Celestia-compatible local blob resolution yields the same artifact contract as local DA;
  - validator acceptance remains independent of trusting the primary;
  - challenge-window, unanchored-height, payload-retention, and degraded-mode behavior are asserted by tests and evidence.
- Negative proof:
  - below-`3f+1` membership rejects BFT mode;
  - below-`2f+1` vote set rejects BFT certificate;
  - wrong blob commitment or wrong namespace rejects local Celestia resolution;
  - missing payload during challenge window rejects or enters degraded mode;
  - unanchored-height or settlement-delay limits enter degraded mode instead of overclaiming finality;
  - commit-certificate verification failure or state-root mismatch rejects;
  - detached certificate or detached blob binding rejects at the validator boundary.
- Success conditions:
  - no external network is required to prove the simulated backend contract;
  - the proven commit-subject interface remains the only backend seam;
  - report honesty remains truthful about what is simulated versus live.

## 🌐 Scenario 11 E2E Matrix

`scenario_11` is the phase-local end-to-end proof harness. It must accumulate the following explicit scenarios.

| Scenario id | What it demonstrates | Required assertions | Required evidence |
| --- | --- | --- | --- |
| `E2E-01` | one-shard happy path | one package, one subject digest, one `2-of-3` certificate, one accepted validator verdict | all JSON artifacts except failure-only rows |
| `E2E-02` | dual-primary owner isolation | one aggregator owning two shards does not merge shard committees or quorum counts | `route_plan_report.json`, `placement_membership.json`, `quorum_certificate.json` |
| `E2E-03` | all-shard sweep | all seven shards derive owner, secondaries, route, and quorum through live logic | per-shard rows in route, membership, and validator evidence |
| `E2E-04` | primary crash before quorum | no certificate and no DA publication occur | `fault_matrix.json`, absence or reject state in certificate and DA artifacts |
| `E2E-05` | primary crash after quorum before DA | the exact existing certificate is resumed into publication; no recomputed or detached publication is allowed | `quorum_certificate.json`, `local_da_binding.json`, restart telemetry |
| `E2E-06` | one secondary offline | honest quorum still succeeds only if a real `2-of-3` remains; otherwise fail closed | vote set size, quorum count, fail-closed telemetry |
| `E2E-07` | one secondary stale | replay verifier rejects before vote creation | `secondary_replay_votes.json`, `fault_matrix.json` |
| `E2E-08` | route, plan, root, lineage, proof, policy, publication, and theorem drift matrix | each mutation changes the digest or produces a stable reject reason | `commit_subject.json`, `secondary_replay_votes.json`, `validator_verdict_report.json` |
| `E2E-09` | observer join, readiness, rotation, removal, and takeover | no unready or removed member contributes; old primary retires; takeover requires matching lineage and generation | `placement_membership.json`, `fault_matrix.json` |
| `E2E-09a` | rolling secondary replacement continuity | after readiness, the replacement secondary participates in the next shard commit without route drift or committee split | `placement_membership.json`, `quorum_certificate.json`, continuity rows in `fault_matrix.json` |
| `E2E-10` | restart, partition, heal, offline minority, divergent root | no conflicting certificate is synthesized; resume requires exact prior state | `fault_matrix.json`, lifecycle telemetry rows |
| `E2E-10a` | rolling primary takeover continuity | one shard may fail over to a lawful new primary while unrelated shards continue producing lawful certificates | per-shard lifecycle rows in `fault_matrix.json`, later shard certificates |
| `E2E-11` | equivocation and payload withholding | conflicting signed votes produce evidence; missing payload produces evidence or degraded state | evidence rows inside `fault_matrix.json` and adapter outputs |
| `E2E-12` | simulated BFT and Celestia degraded modes | `3f+1` and `2f+1` claims, challenge window, unanchored height, and blob mismatch behavior are locally proven | backend-specific rows in `fault_matrix.json` plus blob-binding artifacts |

## 📦 Evidence Artifact Contract

Every implementation must emit deterministic, repository-local JSON artifacts with enough structure to prove correctness and enough honesty fields to block overclaim.

| Artifact | Minimum required content | What it proves |
| --- | --- | --- |
| `scenario_11/quorum/package_ingress_report.json` | package id, package digest, ingress shard, owner, ingress timestamp or deterministic tick, route input summary | the package that entered the workflow is the same one routed downstream |
| `scenario_11/quorum/route_plan_report.json` | route digest, planner digest, shard owner, owner path, generation, dispatch decision | route and planner outputs used by replay and dispatch |
| `scenario_11/quorum/placement_membership.json` | shard id, primary id, secondaries, readiness state, membership digest, generation, lineage | the exact committee used by replay and certificate validation |
| `scenario_11/quorum/commit_subject.json` | canonical field list, domain/version bytes, subject digest, route digest, root, lineage, proof version, policy generation | the exact subject all later steps must share |
| `scenario_11/quorum/secondary_replay_votes.json` | per-voter replay verdict, reject code, signer id, vote digest, accept/reject reason | replay-before-vote and deterministic reject evidence |
| `scenario_11/quorum/quorum_certificate.json` | shard id, term, membership digest, quorum threshold, voter ids, vote digests, certificate digest | one real local certificate bound to one subject |
| `scenario_11/quorum/local_da_binding.json` | publication id, resolved batch id, certificate digest/reference, subject digest, theorem digest or binding reference | DA publication and resolution preserve the quorum artifact |
| `scenario_11/quorum/validator_verdict_report.json` | validator verdict, certificate digest, publication binding, theorem digest, ordered batch id, subject digest | validator acceptance or rejection is subject-consistent |
| `scenario_11/quorum/fault_matrix.json` | scenario id, fault id, expected status, observed status, reject code, evidence refs, degraded-mode flag when relevant | each required failure path was executed and matched expectation |
| `scenario_11/quorum/report_honesty.json` | explicit list of supported claims, explicit list of forbidden claims, simulated-vs-live markers | the report does not overclaim BFT, Celestia finality, slashing, or production transport guarantees |

## 🚫 Anti-Placeholder Gates

- No report-only closure. Every report field must be backed by live runtime, simulator, DA, or validator state.
- No alias closure. `secondary` adoption is incomplete if `standby` survives anywhere active.
- No doc-only terminology closure. A docs-only rename or a comment-only CFT/BFT wording update is insufficient.
- No compatibility-alias closure. Dual fields such as `standby_ids` plus `secondary_ids` are invalid.
- No synthetic quorum closure. A wrapper around the old majority result is not a certificate.
- No majority-proof closure. Keeping `ConsensusCommit` as the only proof object is insufficient.
- No fixture-byte closure. Replay must recompute the subject from live inputs.
- No hard-coded-digest closure. Constant, debug-string, JSON-order, fixture-name, zero, or precomputed expected digests are invalid proof.
- No voter-id-only closure. A certificate that wraps voter ids without canonical vote material is insufficient.
- No trust-primary closure. Downstream acceptance must bind certificate, publication, theorem, and ordered batch.
- No adapter-stub closure. Signature, transport, BFT, or Celestia slices require executable local proof, not traits alone.
- No transport-bypass closure. Network delivery may not inject votes before replay verification.
- No simulator-only lifecycle closure. Join, rotation, and takeover claims require runtime placement and recovery enforcement, not scenario-only assertions.
- No renamed-BFT closure. Renaming a `2-of-3` local quorum as BFT without `3f+1` and `2f+1` proof does not close the backend slice.
- No detached-backend closure. A Celestia-local resolver may not return constant or detached artifacts.
- No scenario drift closure. `scenario_11` must stay independent and phase-owned; it may not hide logic inside `scenario_1`.

## ✅ Exit Criteria

Phase 067 test coverage is ready for implementation only when all of the following are true:

- every `TS-01` through `TS-09` slice has explicit test homes, commands, positive cases, negative cases, and pass conditions;
- every required `scenario_11` artifact has a schema contract and a proof purpose;
- every crypto- and state-binding invariant has at least one positive proof path and one fail-closed path;
- every future-facing claim is marked simulated until backed by local executable evidence;
- the packet does not introduce a second authority or a parallel code path.

## 📎 Coverage Appendix

| Slice | Plan | Primary sources | Core artifacts | Core commands | Expected result |
| --- | --- | --- | --- | --- | --- |
| `TS-01` | `067-01` | TODO, CONTEXT, `066-TODO.md`, plan 01 | runtime/config rename, topology tests | topology, preflight, quorum freeze, grep audit, `cargo test --release` | one active `secondary` vocabulary and honest CFT wording |
| `TS-02` | `067-02` | TODO, CONTEXT, `066-TODO.md`, plan 02 | `commit_subject`, `shard_vote`, `shard_quorum_certificate` tests | commit subject and certificate tests | deterministic digest and fail-closed certificate validation |
| `TS-03` | `067-03` | TODO, CONTEXT, `066-TODO.md`, plan 03 | `secondary_replay` test seam | replay, dist journal, recovery failover tests | replay-backed voting and stable reject reasons |
| `TS-04` | `067-04` | TODO, CONTEXT, `066-TODO.md`, plan 04 | consensus adapter and certificate tests | consensus, failover same lineage, local certificate tests | live commit path emits real quorum proof |
| `TS-05` | `067-05` | TODO, CONTEXT, `066-TODO.md` section 15, plan 05 | `scenario_11` plus JSON evidence | simulator, dispatch, routing, DA, topology, validator tests | independent package-to-validator local proof path |
| `TS-06` | `067-06` | TODO, CONTEXT, `066-TODO.md`, plan 06 | join/failover/rotation tests and lifecycle evidence | join, failover, route-rollout, `scenario_11` | safe lifecycle changes with no stale or mixed certificate |
| `TS-07` | `067-07` | TODO, CONTEXT, `066-TODO.md`, plan 07 | DA binding, theorem guard, validator contract tests | DA local sim, DA quorum binding, theorem guard, publication binding, validator contract | validator requires certificate-aware subject binding |
| `TS-08` | `067-08` | TODO, CONTEXT, `066-TODO.md`, plan 08 | signature, transport, equivocation evidence tests | adapter tests and `scenario_11` | transport hooks preserve replay-first semantics |
| `TS-09` | `067-09` | TODO, CONTEXT, `066-TODO.md`, plan 09, aggregator README | BFT committee rules, Celestia-local binding, degraded-mode evidence | BFT, Celestia, topology, validator, `scenario_11` | future backend claims are backed by local proof |
