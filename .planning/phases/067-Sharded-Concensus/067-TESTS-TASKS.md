---
phase: 067
artifact: tests-tasks
status: planned
updated: 2026-07-04
---

# Phase 067 Test Tasks

This document converts the Phase 067 planning packet into an implementation-ready test worklist. It is intentionally phase-local and does not claim that the underlying runtime behavior already exists.

## 🎯 Execution Model

- Implement tests slice by slice in `TS-01` through `TS-09` order.
- If a target test file already exists, extend it in place.
- If a target test file is listed in the plan packet but does not exist yet, create it at the proposed path instead of inventing a parallel home.
- Keep one runtime authority. Do not duplicate route, placement, certificate, publication, theorem, or validator logic inside tests.
- `scenario_11` is the only new end-to-end harness for this phase. Do not fold this work into `scenario_1`.

## 📍 Current Workspace Reality

### ✅ Existing anchors that should be extended

- `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_shard_routing.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
- `crates/z00z_runtime/aggregators/tests/test_commit_subject.rs`
- `crates/z00z_runtime/aggregators/tests/test_shard_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`
- `crates/z00z_runtime/aggregators/tests/test_local_quorum_certificate.rs`
- `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
- `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
- `crates/z00z_simulator/tests/test_scenario_11.rs`

### 🆕 Planned test homes that are missing today

- `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs`
- `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`
- `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`
- `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
- `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`
- `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`

## 🔒 Shared Gates

- Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` before each verification pass.
- Run `cargo test --release` whenever a slice touches broad Rust or shared test surfaces.
- Keep all repo artifacts in English.
- Use existing project abstractions and real crypto primitives.
- Reject any implementation that closes on placeholder digests, fixture-only byte comparisons, or report strings without live state evidence.

## 🧪 Ordered Tasks

### ✅ TT-00 Packet Integrity And Source Lock

- Goal:
  - confirm that the engineer is implementing from the locked packet only.
- Read first:
  - `067-TODO.md`
  - `067-CONTEXT.md`
  - `067-01-PLAN.md` through `067-09-PLAN.md`
  - `.planning/phases/090-New-Scenarios/066-TODO.md` section `15`
  - `crates/z00z_runtime/aggregators/README.md`
- Required checks:
  - preserve the exact `PHASE-0` through `PHASE-8` mapping;
  - keep `scenario_11` independent;
  - do not add a second authority document.
- Completion signal:
  - all later tasks reference only packet-owned anchors and current code owners.

### ✅ TT-01 TS-01 Terminology And Boundary Cleanup

- Target files:
  - `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_preflight.rs`
  - existing quorum freeze test seam in `z00z_aggregators`
- Required cases:

| Case id | Target | What it must prove | Pass condition |
| --- | --- | --- | --- |
| `067-T01-01` | topology | `secondary` config fields load successfully | the topology graph is identical to the pre-rename ownership model |
| `067-T01-01a` | generated homes | generated temp homes derived from `sim_5a7s` also load with `secondary` fields only | generated runtime homes load without alias keys |
| `067-T01-02` | preflight | duplicate secondary ids reject | load fails with a deterministic error |
| `067-T01-03` | preflight | unknown secondary ids reject | load fails before runtime start |
| `067-T01-04` | preflight | primary id cannot appear inside the secondary set | load fails deterministically |
| `067-T01-05` | preflight | stale `standby` keys reject | parser fails; no compatibility alias is accepted |
| `067-T01-06` | consensus parity | split-brain freeze behavior is unchanged by the rename | the same conflict still freezes or rejects |
| `067-T01-07` | grep audit | no active `standby` naming survives | grep returns zero active hits |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast test_quorum_freezes_term_roots -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_preflight -- --nocapture`
  - `cargo test --release`
  - `rg -n "standby|TakeoverStandby|standby_ids" crates/z00z_runtime crates/z00z_rollup_node crates/z00z_simulator config/hjmt_runtime/sim_5a7s --glob '!**/*.md'`
- Anti-placeholder reminders:
  - a doc-only rename does not satisfy this slice;
  - a comment-only CFT/BFT wording change does not satisfy this slice;
  - dual alias fields such as `standby_ids` plus `secondary_ids` are forbidden.

### ✅ TT-02 TS-02 Commit Subject And Certificate Types

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_commit_subject.rs`
  - `crates/z00z_runtime/aggregators/tests/test_shard_quorum_certificate.rs`
- Required positive cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T02-01` | same live fixture yields the same subject digest every time | repeated encodes match byte-for-byte |
| `067-T02-02` | valid active vote set yields one certificate | one certificate is built and validates |
| `067-T02-02a` | active placement restriction | the certificate builder accepts only the active placement members for one shard and one generation | any off-shard, inactive, or stale-generation member causes rejection |

- Required mutation cases:

| Case id | Mutation | Pass condition |
| --- | --- | --- |
| `067-T02-03` | route digest drift | subject digest changes or validation rejects |
| `067-T02-04` | generation drift | subject digest changes or validation rejects |
| `067-T02-05` | root drift | subject digest changes or validation rejects |
| `067-T02-06` | lineage drift | subject digest changes or validation rejects |
| `067-T02-07` | proof version drift | subject digest changes or validation rejects |
| `067-T02-08` | policy generation drift | subject digest changes or validation rejects |
| `067-T02-09` | wrong voter role | vote or certificate rejects |
| `067-T02-10` | duplicate voter | certificate rejects |
| `067-T02-11` | inactive voter | certificate rejects |
| `067-T02-12` | mixed membership digest | certificate rejects |
| `067-T02-13` | mixed term | certificate rejects |
| `067-T02-14` | below quorum | certificate rejects |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_commit_subject -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_shard_quorum_certificate -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast test_quorum_freezes_term_roots -- --nocapture`
- Anti-placeholder reminders:
  - constant, debug-string, JSON-order, fixture-name, or hard-coded expected digests are invalid;
  - a certificate that wraps only voter ids without canonical vote material is invalid.

### ✅ TT-03 TS-03 Secondary Replay Verifier

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_secondary_replay_verifier.rs`
  - `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
  - existing journal coverage in `test_hjmt_dist_journal`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T03-01` | exact primary subject is replayed and accepted | verifier returns an explicit accept result |
| `067-T03-02` | route drift rejects before vote creation | reject code is deterministic |
| `067-T03-03` | planner digest drift rejects | reject code is deterministic |
| `067-T03-04` | root drift rejects | reject code is deterministic |
| `067-T03-05` | lineage drift rejects | reject code is deterministic |
| `067-T03-06` | proof-version drift rejects | reject code is deterministic |
| `067-T03-07` | policy-generation drift rejects | reject code is deterministic |
| `067-T03-08` | publication-binding drift rejects | reject code is deterministic |
| `067-T03-09` | theorem-digest drift rejects | reject code is deterministic |
| `067-T03-10` | stale secondary state rejects | no vote is created |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_secondary_replay_verifier -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dist_journal -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
- Anti-placeholder reminders:
  - fixture-byte comparison without recomputing the subject does not satisfy this slice;
  - vote creation from hard-coded expected digests does not satisfy this slice.

### ✅ TT-04 TS-04 Local Quorum Certificate Integration

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
  - `crates/z00z_runtime/aggregators/tests/test_local_quorum_certificate.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T04-01` | honest local quorum returns one certificate-bound decision | certificate exists and commit result matches the honest path |
| `067-T04-02` | same-term conflicting subjects cannot both commit | freeze or reject occurs before dual commit |
| `067-T04-03` | duplicate voter is rejected | certificate formation fails |
| `067-T04-04` | removed voter is rejected | certificate formation fails |
| `067-T04-05` | joined-but-not-ready secondary is rejected | certificate formation fails |
| `067-T04-06` | mixed membership digest is rejected | certificate formation fails |
| `067-T04-07` | mixed term is rejected | certificate formation fails |
| `067-T04-08` | certificate path preserves parity with legacy honest decision | same final commit decision is observed |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_consensus -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_local_quorum_certificate -- --nocapture`
- Anti-placeholder reminders:
  - wrapping the old majority result in a synthetic certificate does not satisfy this slice;
  - leaving `ConsensusCommit` as the only proof object does not satisfy this slice.

### ✅ TT-05 TS-05 Scenario 11 Base End-To-End Harness

- Target files:
  - `crates/z00z_simulator/src/scenario_11/mod.rs`
  - `crates/z00z_simulator/src/scenario_11/report.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required end-to-end cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T05-01` | one-shard happy path | one valid package reaches one accepted validator verdict with one `2-of-3` certificate |
| `067-T05-02` | dual-primary owner path | one owner may control two shards without merging their quorums |
| `067-T05-03` | all-shard sweep | all seven shards emit owner, route, membership, and verdict evidence |
| `067-T05-04` | one secondary offline | the shard either forms a real honest quorum or fails closed |
| `067-T05-05` | primary crash before quorum | no certificate and no DA publication occur |
| `067-T05-06` | primary crash after quorum before DA | the exact same certificate is resumed into publication |
| `067-T05-07` | stale secondary | replay rejects before vote creation |
| `067-T05-08` | wrong route or planner digest | scenario rejects with deterministic evidence |
| `067-T05-09` | wrong dispatch owner or shard-owner mismatch | scenario rejects with deterministic evidence |
| `067-T05-10` | report honesty | output explicitly rejects unsupported BFT or Celestia claims |

- Required artifact assertions:
  - `package_ingress_report.json` and `route_plan_report.json` carry the same package identity.
  - `commit_subject.json`, `secondary_replay_votes.json`, `quorum_certificate.json`, `local_da_binding.json`, and `validator_verdict_report.json` carry one subject digest.
  - `fault_matrix.json` records every failure case with expected and observed status.
  - `report_honesty.json` marks unsupported claims as forbidden.
- Commands:
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_dispatch -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_shard_routing -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- Anti-placeholder reminders:
  - extending `scenario_1` with extra stage fields does not satisfy this slice;
  - emitting report files without live vote, certificate, DA, and validator evidence does not satisfy this slice;
  - any global-five-aggregator quorum counting is forbidden.

### ✅ TT-06 TS-06 Join Removal And Rotation Simulation

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_join.rs`
  - `crates/z00z_runtime/aggregators/tests/test_recovery_failover.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_failover_same_lineage.rs`
  - `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T06-01` | observer catch-up to ready secondary | observer cannot vote before readiness, then can vote after readiness |
| `067-T06-02` | planned rotation at checkpoint or generation boundary | old primary stops committing after activation |
| `067-T06-02a` | rolling secondary replacement continuity | once the replacement is ready, the next shard commit uses the new committee without route drift or quorum split |
| `067-T06-03` | removed member rejection | removed member cannot vote or count toward quorum |
| `067-T06-04` | emergency takeover with matching lineage and generation | takeover succeeds only with exact required lineage and generation |
| `067-T06-04a` | rolling primary takeover continuity | takeover on one shard allows the next lawful commit there while unrelated shards continue producing certificates |
| `067-T06-05` | stale-lineage takeover reject | fail-closed before vote counting |
| `067-T06-06` | stale-generation takeover reject | fail-closed before vote counting |
| `067-T06-06a` | stale route-generation takeover reject | fail-closed before vote counting |
| `067-T06-07` | divergent-root takeover reject | fail-closed before vote counting |
| `067-T06-08` | mixed-generation certificate reject | certificate validation fails |
| `067-T06-08a` | mixed-lineage vote-set reject | certificate validation fails before commit |
| `067-T06-09` | exact publication resume after crash | only the exact prior certificate/publication state resumes |
| `067-T06-10` | partition and heal | no conflicting certificate is synthesized during minority isolation |
| `067-T06-11` | offline-member minority | no synthetic quorum appears |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_join -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_recovery_failover -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_failover_same_lineage -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_hjmt_route_rollout -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- Anti-placeholder reminders:
  - toggling role names without readiness or lineage enforcement does not satisfy this slice;
  - a failover path that ignores route generation does not satisfy this slice;
  - simulator-only lifecycle assertions without runtime placement or recovery enforcement do not satisfy this slice.

### ✅ TT-07 TS-07 Validator And Theorem Binding

- Target files:
  - `crates/z00z_rollup_node/tests/test_da_local_sim.rs`
  - `crates/z00z_rollup_node/tests/test_da_local_quorum_binding.rs`
  - `crates/z00z_rollup_node/tests/test_rollup_theorem_guard.rs`
  - `crates/z00z_runtime/aggregators/tests/test_publication_binding.rs`
  - `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T07-01` | DA publish stores certificate binding | publication record contains a non-placeholder digest or reference |
| `067-T07-02` | DA resolve preserves certificate binding | resolved batch still refers to the same binding |
| `067-T07-03` | validator happy path | validator accepts only when certificate, theorem, publication, and ordered batch share one subject |
| `067-T07-04` | missing certificate reject | validator or resolve path rejects |
| `067-T07-05` | mismatched certificate reject | validator or resolve path rejects |
| `067-T07-06` | detached publication reject | theorem or validator path rejects |
| `067-T07-07` | stale certificate from inactive or mixed membership reject | validator path rejects |
| `067-T07-08` | local CFT proof harness | exhaustive or proof-based test shows agreement does not depend on trusting the primary |

- Commands:
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_sim -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_da_local_quorum_binding -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_rollup_theorem_guard -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_publication_binding -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
- Anti-placeholder reminders:
  - a constant or zero certificate digest in DA records does not satisfy this slice;
  - validator acceptance that ignores the certificate binding does not satisfy this slice.

### ✅ TT-08 TS-08 Network And Signature Adapter

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_signature_adapter.rs`
  - `crates/z00z_runtime/aggregators/tests/test_transport_adapter.rs`
  - `crates/z00z_runtime/aggregators/tests/test_equivocation_evidence.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T08-01` | correct signer and digest validation | valid signed vote is accepted only after replay |
| `067-T08-02` | wrong signer reject | signature verification fails deterministically |
| `067-T08-03` | wrong membership digest reject | signature or vote validation fails |
| `067-T08-04` | wrong subject digest reject | signature or vote validation fails |
| `067-T08-05` | transport cannot bypass replay | transport-delivered vote without replay rejects |
| `067-T08-06` | duplicate message idempotency | duplicate delivery does not double count |
| `067-T08-07` | equivocation evidence | conflicting same-voter votes emit deterministic evidence containing both vote materials |
| `067-T08-08` | payload withholding | evidence or degraded state is emitted; no silent success |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_signature_adapter -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_transport_adapter -- --nocapture`
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_equivocation_evidence -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- Anti-placeholder reminders:
  - a trait stub with no exercising tests does not satisfy this slice;
  - missing-payload paths may not create synthetic votes.

### ✅ TT-09 TS-09 BFT And Celestia Backend

- Target files:
  - `crates/z00z_runtime/aggregators/tests/test_bft_committee_rules.rs`
  - `crates/z00z_rollup_node/tests/test_celestia_local_binding.rs`
  - `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`
  - `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`
  - `crates/z00z_simulator/tests/test_scenario_11.rs`
- Required cases:

| Case id | What it demonstrates | Pass condition |
| --- | --- | --- |
| `067-T09-01` | `3f+1` committee membership rule | valid BFT mode only starts with `3f+1` members |
| `067-T09-02` | below-`3f+1` reject | BFT mode rejects deterministically |
| `067-T09-03` | `2f+1` quorum rule | valid BFT certificate requires `2f+1` votes |
| `067-T09-04` | below-`2f+1` reject | BFT certificate validation fails |
| `067-T09-05` | Celestia-local happy path | blob resolution returns the same artifact contract as local DA |
| `067-T09-06` | wrong blob commitment reject | resolution fails deterministically |
| `067-T09-07` | wrong namespace reject | resolution fails deterministically |
| `067-T09-08` | missing payload during challenge window | reject or degraded mode is recorded |
| `067-T09-09` | unanchored height or settlement delay | degraded mode is recorded instead of overclaiming finality |
| `067-T09-10` | detached certificate or blob binding reject | validator path rejects without proposer trust |
| `067-T09-11` | commit-certificate verification failure or state-root mismatch | validator or backend verification rejects deterministically |

- Commands:
  - `cargo test --release -p z00z_aggregators --features test-params-fast --test test_bft_committee_rules -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_celestia_local_binding -- --nocapture`
  - `cargo test --release -p z00z_rollup_node --features test-params-fast --test test_hjmt_topology -- --nocapture`
  - `cargo test --release -p z00z_validators --test test_hjmt_publication_contract -- --nocapture`
  - `cargo test --release -p z00z_simulator --features test-params-fast --features wallet_debug_tools --test scenario_11 -- --nocapture`
- Anti-placeholder reminders:
  - renaming a `2-of-3` local quorum as BFT does not satisfy this slice;
  - feature flags or adapter stubs without `3f+1` and `2f+1` proof do not satisfy this slice;
  - a Celestia-local resolver that returns constant or detached artifacts does not satisfy this slice.

### ✅ TT-10 Final Coverage And Honesty Audit

- Confirm that:
  - every `TS-01` through `TS-09` case above exists in a test file or simulator scenario row;
  - every required JSON artifact from `scenario_11` is produced and asserted;
  - every report or documentation surface stays honest about local CFT, simulated transport, and simulated backend guarantees;
  - no test introduced duplicate route, placement, theorem, or validator logic instead of reusing project owners.
- Recommended final commands:
  - `cargo test --release`
  - targeted slice commands for any changed area
  - packet grep checks for `standby`, overclaim keywords, and missing `scenario_11` artifact names

## ✅ Completion Checklist

- `TS-01` through `TS-09` are implemented with positive and negative coverage.
- `scenario_11` emits the full artifact set with stable evidence fields.
- every cryptographic, membership, lineage, and publication-binding invariant has at least one positive proof and one reject path.
- no closure relies on comments, TODOs, feature flags, or constant digests.
