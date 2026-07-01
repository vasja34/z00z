# 060 Gap-Closing Technical Specification

[TOC]

Date: 2026-06-19
Scope: `.planning/phases/060-Gaps-Closing`
Status: draft planning input
Language rule: English-only technical content

## 🎯 Purpose

This file converts every `🐞-TODO:` marker from the Phase 060 documents into a
self-contained technical specification that can be broken down into execution
plans later.

This specification is intentionally opinionated:

- it names the real gap that exists in the current repository;
- it rejects false gaps that would create churn without improving the system;
- it gives ordered work, non-goals, and exit criteria;
- it embeds the necessary source excerpts instead of requiring readers to
  reconstruct the intent from multiple documents.

All conclusions below were doublechecked directly against the workspace code,
docs, reports, and tests. No concrete claim below depends on graph material.

## 📚 Source TODO Inventory

The Phase 060 input set, excluding this SPEC file itself, contains `14` explicit
`🐞-TODO:` markers. Several are continuation markers rather than separate
scopes, so they normalize into four workstreams.

| Source marker | Translated signal | Normalized workstream | Required disposition |
| --- | --- | --- | --- |
| `060-TZ1.md` authority marker | Config authority is not unified between `assets_config.yaml` and `genesis_config*.yaml`. | A | Close dual-authority and owner-boundary drift in `z00z_core`. |
| `060-TZ1.md` recheck marker | Re-check the whole `z00z_core` structure and do not invent modules without proof. | A | Keep only evidence-backed structural changes. |
| `060-TZ1.md` blank conclusion marker | Convert the structural analysis into a target structure. | A | Turn analysis into ordered cleanup and explicit non-goals. |
| `060-TZ1.md` repeated structure prompt | Re-run the same architecture question from first principles. | A | Same as above; no second separate project. |
| `060-TZ1.md` HJMT contract marker | Verify whether a shard is a process and whether benchmark readings are being interpreted correctly. | B | Clarify the live process model and fix evidence semantics. |
| `060-TZ1.md` measurement marker | Re-run HJMT measurements and prove real parallelism. | B | Add topology coverage and provenance-separated throughput artifacts first. |
| `060-TZ1.md` HJMT conclusion marker | Convert the HJMT observations into action items. | B | Target scheduler or publication path only after topology and measurement hygiene are fixed. |
| `060-TZ2.md` next-move marker | The next move is `l0-docs`, then project-owned `l4-supply-chain`, then adversarial high-risk scenarios. | C | Keep this order for verification closure. |
| `060-TZ2.md` wallet storage marker | Define the optimal wallet storage structure for assets, rights, and vouchers. | D | Do not redesign persistence; build on the typed inventory already live. |
| `060-TZ2.md` profile marker | Collect rights and vouchers profiles, compliance surfaces, and MVP functionality across the docs. | D | Publish a concrete MVP profile matrix and required validator or wallet behavior. |
| `060-z00z-verification-report.md` risk register marker | Convert risk-register output into fixable work. | C | Split gate failures from human-review hypotheses. |
| `060-z00z-verification-report.md` adversarial marker | Turn the adversarial report into actionable closure work. | C | Convert each high finding into a claim, harness, or closure memo. |
| `060-z00z-verification-report.md` fixable findings marker | Project-owned blockers need a concrete plan. | C | Separate docs, supply-chain, and adversarial closure. |
| `060-z00z-verification-report.md` recommended actions marker | Recommendations must become implementation-ready tasks, including the profiling top-slowest acceleration advice. | C | Replace broad advice with ordered closure criteria and verification-pipeline performance work. |

### ✅ Inventory self-audit

This audit is part of the spec, not a separate note. It records how the three
Phase 060 source documents are consumed.

| Source document | Explicit markers | SPEC sections that consume it | Completeness check |
| --- | ---: | --- | --- |
| `060-TZ1.md` | `7` | Workstreams A and B | Architecture, config authority, module ownership, HJMT topology, failover, measurement semantics, one-shard-per-process trade-off, and HJMT performance baseline are all represented. |
| `060-TZ2.md` | `3` | Workstreams C and D | Verification closure order, wallet storage posture, rights and voucher MVP profiles, self-custody lock semantics, and fail-closed profile tests are represented. |
| `060-z00z-verification-report.md` | `4` | Workstream C | `l0-docs`, `l4-supply-chain`, adversarial high findings, protected-vendor handling, and top-slowest verification-pipeline performance work are represented. |

If later planning splits this file into execution phases, the split must preserve
this mapping so no original source marker becomes an orphaned note.

## 🧱 Workstream A — `z00z_core` Authority And Module Ownership Cleanup

### 🔎 A source signals

Translated from the Russian source notes in `060-TZ1.md`:

- "Config authority is not fully unified: part lives in `assets_config.yaml`,
  part in `genesis_config*.yaml`."
- "Re-check the `z00z_core` structure. Maybe parts of `assets/` belong in a
  common layer or terminal-leaf layer. Do not recommend extra modules unless
  the codebase actually needs them."
- "The main problem is not 'too few folders'. It is that `assets` still
  combines asset-domain logic, compatibility facades, and part of the
  generalized object vocabulary."

### 📌 A workspace evidence

Direct repository evidence:

- `crates/z00z_core/src/genesis/README.md`: "This module is the single genesis
  orchestration boundary for the live Z00Z object model."
- `crates/z00z_core/src/genesis/README.md`: "Genesis now coordinates four typed
  object lanes under one deterministic export path: Assets, Rights, Policies,
  Vouchers."
- `crates/z00z_core/src/assets/assets_config.yaml` header:
  "`# Assets: Coins, Tokens, NFTs, Void sinks, Rights`"
- `crates/z00z_core/README.md`:
  "`src/assets/assets_config.yaml` – Asset definitions (Coin, Token, NFT,
  Void)"
- `crates/z00z_core/src/rights/mod.rs`:
  `pub use crate::assets::right_config::{RightClassConfig, RightsConfigEntry};`
- `crates/z00z_storage/src/settlement/leaf.rs`: "This leaf keeps the committed
  payload identical to the core asset-leaf contract while preserving a
  storage-owned type boundary. Storage-facing code must import this type
  instead of depending on `z00z_core::assets::AssetLeaf` directly."

### ✅ A actual gap

The real `z00z_core` gap is not "missing config files" and not "missing
modules". It is the combination of four narrower problems:

1. `z00z_core::genesis` is already the canonical bootstrap authority, but the
   repository still exposes a second quasi-authoritative story through
   `src/assets/assets_config.yaml` and stale README language.
2. `rights` vocabulary exists as a top-level module, but `rights` config types
   still live under `assets`, so the owner boundary is unfinished.
3. `assets/` still acts as a compatibility umbrella for `actions`, `policies`,
   `vouchers`, and some generalized object vocabulary through shim modules and
   re-exports.
4. Documentation and examples drift faster than module ownership, so readers
   cannot tell which YAML or namespace is authoritative and which one is
   compatibility-only.

### 🚫 A false gaps to reject

Do not do the following as part of Phase 060:

- Do not add `actions_config.yaml`, `policies_config.yaml`,
  `vouchers_config.yaml`, or `rights_config.yaml` just to make the names look
  symmetric. That would create more authority surfaces, not fewer.
- Do not create a second canonical JSON helper or move ownership away from
  `z00z_utils::codec` in this phase. The live canonicalization helper is
  already shared by `z00z_core` descriptor hashing and must stay single-sourced.
- Do not collapse `AssetLeaf` and storage `TerminalLeaf` into one generic type.
  The current split is deliberate and healthy.
- Do not rename or split `genesis_policies.rs` on the theory that it is
  "asset-only". The live file is already a generalized genesis policy compiler.

### ⚙️ A required work

#### 🔑 A1. Freeze the canonical authority contract

The repository must declare one authoritative bootstrap contract:

- `z00z_core::genesis` is the only canonical bootstrap authority for assets,
  rights, policies, and vouchers.
- `GenesisConfig` is the canonical typed manifest.
- Any other YAML surface is either:
  - asset-registry data;
  - example or fixture data;
  - compatibility-only data.

Required edits in the future implementation phase:

- update `crates/z00z_core/README.md`;
- update `crates/z00z_core/src/assets/mod.rs` module docs;
- keep `crates/z00z_core/src/genesis/README.md` as the primary authority note;
- make the authority statement explicit in any example docs that still imply
  that `assets_config.yaml` is the bootstrap source of truth.

#### 🧭 A2. Finish the `rights` owner move

Target end state:

- `RightClassConfig`, `RightsConfigEntry`, and any rights-config loader or
  parser surface are owned by `crates/z00z_core/src/rights/`.
- `assets/right_config.rs` becomes:
  - either a compatibility re-export layer with deprecation comments;
  - or a deleted compatibility surface after all internal imports are moved.

Required migration steps:

1. Move config types and parsing logic into `rights/`.
2. Update internal imports across the repository to the new owner path.
3. Keep transitional re-exports only if public crate compatibility requires
   them.
4. Add one regression test that fails if `rights` config types again originate
   from `assets`.

#### 🧹 A3. Demote shim-first imports

Current shim layers under `assets/` for `action_pool`, `policy_descriptor`, and
`voucher_config` should be treated as compatibility-only, not as the preferred
authoring path.

Required end state:

- new code imports from `actions`, `policies`, and `vouchers` directly;
- internal `z00z_core` docs stop recommending shim-first imports;
- if the shims remain for compatibility, they must be marked as such in module
  docs.

Recommended enforcement:

- add a lightweight grep-based test or review rule that rejects new internal
  imports from compatibility shims when a real owner module exists.

#### 🧪 A4. Remove the dual-authority YAML risk

Phase 060 should not leave `rights:` in the default `src/assets/assets_config.yaml`
as an implied bootstrap authority.

Required target:

- production or operator-facing bootstrap flow must not require `rights:`
  entries in `src/assets/assets_config.yaml`;
- `rights` bootstrap data belongs under `genesis` typed config;
- if the repository still needs a mixed example file for tests or migration,
  that file must be:
  - clearly labeled compatibility or fixture-only;
  - generated from the canonical genesis fixture or kept under a fixture path
    rather than the default asset-registry path.

This is the point where the structural tension actually closes. Without this
step, README cleanup alone will not remove authority drift.

#### 📝 A5. Repair documentation and fixture drift

At minimum, the implementation phase must update:

- `crates/z00z_core/README.md`;
- any CLI help or docs that treat `assets_config.yaml` as asset-only while the
  file still contains `rights`;
- any examples that imply a second bootstrap story;
- fixture tests so they assert the intended boundary rather than the accidental
  current shape.

### ✅ A acceptance criteria

This workstream is complete only when all of the following are true:

- `z00z_core::genesis` is explicitly documented as the single bootstrap
  authority.
- Internal repository imports no longer source `rights` config types from
  `assets`.
- Compatibility shims are either demoted and documented or removed.
- `src/assets/assets_config.yaml` no longer acts as an implied second bootstrap
  authority for `rights`.
- Docs and fixtures no longer tell contradictory stories about the bootstrap
  boundary.

### 🧪 A verification anchors

The implementation phase should verify this work against the existing genesis
test anchors already documented in `crates/z00z_core/src/genesis/README.md`,
including:

- `cargo test -p z00z_core --release --features deterministic-rng test_genesis_manifest_phase059_fixture -- --nocapture`
- `cargo test -p z00z_core --release test_policy_descriptor -- --nocapture`
- `cargo test -p z00z_core --release test_voucher_config -- --nocapture`
- `cargo test -p z00z_core --release test_rights_config -- --nocapture`

## 🌐 Workstream B — HJMT Topology, Failover, And Measurement Evidence

### 🔎 B source signals

Translated from the Russian source notes in `060-TZ1.md`:

- "Make sure each shard is a separate process and prove real HJMT parallelism."
- "Add a `3A7S` topology, then kill one process and work as `2A7S`, then add
  two aggregators and work as `5A7S`, matching the RAID-like HJMT operational
  idea from the documents."
- "Re-run the measurements, but only after checking what is actually being
  measured."

Source measurement baseline that must stay visible in later plans:

| Source metric family | Current source note value | SPEC interpretation |
| --- | --- | --- |
| Search or read median | typical `3.5-5.1 ms`; `post_reload_lookup` `1.57 ms` | Read path is not the primary Phase 060 bottleneck. |
| Insert median | typical `9-12 ms`; `mixed_batch` `11.90 ms` | Mutation cost is material but must be separated from publication and sync cost. |
| Delete median | typical `5.9-13.4 ms` | Delete path needs normal regression coverage, not a separate architecture change. |
| Synthetic proof verify | single inclusion `19.7 us`, deletion `31.1 us`, nonexistence `34.5 us` | Proof verification is not the current end-to-end TPS ceiling. |
| Shard scaling | `durable_root_published_tps`: `1` shard `1048`, `2` `1949`, `4` `3207`, `8` `5540`, `16` `6555`, `sim_5a7s` `6510` | Shard parallelism has real global throughput evidence, but promotion claims must use durable-root TPS, not worker-local TPS. |
| Coordination overhead | `blocked_time_us` rises from `3998` to `14035`; hot-shard throughput falls from `1420 TPS` to `640 TPS` | More shards are not free; topology changes need blocked-time and hot-shard checks. |
| `scenario_1` runtime | wall `70.15s`, internal stage total `65.35s`, max RSS `571 MB`, CPU `173%` | End-to-end runs need stage breakdown and resource reporting. |
| `scenario_1` heavy stages | `hjmt_settlement_examples` `38.82s`, `checkpoint_apply_storage` `14.54s`, `claim_prepare` `7.99s`, `tx_prepare` `2.53s` | Optimization should target durable state, checkpoint, and publication paths before proof verification. |
| Cache behavior | hits `243393`, misses `26089`, hit rate about `90.3%`, `root_reuse_ratio` `0.9969` | Cache is useful but does not by itself remove end-to-end latency. |
| Async scheduler | `serial_batch` median `4.46 ms`, `parallel_batch` median `7.77 ms`, live `scheduler_metrics.max_active = 1` | Async correctness exists, but operational parallelism is not proven as a production speedup. |

The source conclusion is therefore preserved as a Phase 060 constraint: the main
HJMT ceiling is currently the durable state path, especially `journal_sync`,
`apply_ops`, checkpoint/publication work, and Stage 13 artifact work, not
cryptographic proof verification.

### 📌 B workspace evidence

Direct repository evidence:

- `crates/z00z_rollup_node/src/config.rs`:
  `pub struct AggProc { ... pub shards: Vec<ShardOwn>, ... }`
- `crates/z00z_rollup_node/src/config.rs`:
  "aggregator {} shards must share one expected_journal_lineage because
  journal_path is process-scoped"
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`:
  `process_contract_keeps_explicit_paths_per_aggregator()`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`:
  `process_ids.len() == 5` while `shard_ids.len() == 7`
- `reports/.../profiling/hjmt-summary.json`:
  `"measured": false` and
  `"reason": "No run-root settlement throughput artifact was produced; do not infer TPS from proof-size or one-shot verify-time samples."`

Relevant design intent already present in the repository:

- `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md`: "The benchmark harness MUST
  record two numbers separately: worker-local mutation throughput and
  durable-root-published throughput. Only the second number can support
  user-facing TPS claims."
- `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md`: the operational note preserves
  a RAID10-like deployment shape with shard striping plus same-lineage standby
  capacity per shard.

### ✅ B actual gap

There are five real HJMT gaps:

1. The source note assumes "one shard = one OS process", but the live contract
   is "one aggregator = one OS process, one aggregator may own multiple
   shards". This mismatch must be closed explicitly.
2. The repository does not yet expose an explicit YAML-controlled shard
   execution mapping. Production should keep the current aggregator-owned
   mapping by default, while allowing an opt-in one-shard-per-process profile
   only after topology and measurement evidence exists.
3. Join or reshuffle coverage is incomplete because decommission or full-owner
   removal is not explicitly covered.
4. The repository is missing the specific multi-step fail-down and fail-up
   scenario requested by the source note: `3A7S -> 2A7S -> 5A7S`.
5. Performance evidence is being mixed across incompatible measurement lanes:
   Criterion microbench timings, `/usr/bin/time -v` whole-command timings,
   scenario runtime splits, and user-facing throughput claims.

### 🚫 B false gaps to reject

Do not treat the following as bugs unless the architecture is intentionally
changed:

- The current one-aggregator-many-shards process model is not a latent failure.
  It is the implemented runtime contract.
- The absence of `1 shard = 1 process` is not, by itself, a Phase 060 defect.
  Making it the default before evidence exists is rejected. It may be added as
  an opt-in YAML deployment profile with explicit validation and A/B evidence.
- Proof-size wins and verifier timings do not establish end-to-end TPS.
- Scheduler parallelism must not be claimed as operationally proven while live
  evidence still shows `max_active = 1`.

### ⚙️ B required work

#### 📘 B1. Publish the live topology contract

Phase 060 must remove ambiguity between docs, tests, and expectations.

Required contract statement:

- runtime process model: one aggregator equals one OS process by default;
- shard ownership model: one aggregator may own multiple shards by default;
- standby model: standby takeover must preserve shard lineage;
- journal contract: `journal_path` is process-scoped, so all shards in one
  aggregator share the same expected journal lineage constraint.

If stakeholders still want `1 shard = 1 process`, the request must be treated as
an explicit deployment profile, not as a hidden measurement assumption.

#### ⚙️ B2. Add YAML-selectable shard execution mapping

Add an aggregator YAML option that controls physical shard execution mapping
without changing protocol truth, route truth, or recovery truth.

Required default:

```yaml
execution:
  shard_mapping: "aggregator_owned"
```

Required enum semantics:

| Value | Meaning | Production status |
| --- | --- | --- |
| `aggregator_owned` | Current behavior: each aggregator is one OS process and may own multiple primary shards. | Default production profile. |
| `shard_process` | Opt-in behavior: each primary shard must be represented by a distinct aggregator process with exactly one primary shard in its `shards` list. | Experimental or operator-selected profile until A/B evidence promotes it. |

Implementation requirements:

- keep absent `execution.shard_mapping` backward-compatible with
  `aggregator_owned`;
- reject unknown values and mixed values across one HJMT home;
- in `aggregator_owned`, preserve the existing validation that all shards inside
  one aggregator share one expected journal lineage because `journal_path` is
  process-scoped;
- in `shard_process`, reject any primary-owner aggregator with more than one
  primary shard;
- keep standby, routing-generation, same-lineage takeover, and local durable
  journal rules unchanged;
- do not introduce a shared cross-aggregator WAL or promote process placement to
  protocol truth;
- update the manifest or process-map evidence so reports show which mapping was
  used for a run.

Recommended code shape for the later implementation phase:

- add a small `AggExecutionCfg` and `ShardMapping` enum to
  `crates/z00z_rollup_node/src/config.rs`;
- parse it from `aggregator-config.yaml` with default `aggregator_owned`;
- add config tests for default, explicit `aggregator_owned`, valid
  `shard_process`, invalid mixed mapping, and invalid multi-shard
  `shard_process`;
- add one generated fixture home that represents the `shard_process` profile
  without changing the canonical `SIM-5A7S` default fixture;
- include the selected mapping in benchmark and scenario evidence artifacts.

Complexity assessment:

- Expected codebase complexity is moderate if the work is limited to YAML
  parsing, config validation, fixture generation, and benchmark/report labels.
- Complexity becomes high if the implementation tries to change storage
  durability, recovery boundaries, route-table authority, or publication truth.
  Those changes are out of scope for this workstream.

#### 🧪 B3. Add explicit decommission and removal coverage

The repository already covers several join or transfer paths, but Phase 060 must
add the missing removal case:

- fully remove an aggregator that owns multiple shards;
- redistribute all affected shards;
- clean every standby or route-table reference to the removed aggregator;
- preserve same-lineage failover rules;
- prove that no stale route or standby edge remains after the topology change.

Recommended artifact shape:

- one dedicated test focused on topology mutation or route cleanup;
- one integration test that checks manifest or runtime evidence after
  decommission.

#### 🔄 B4. Add the requested `3A7S -> 2A7S -> 5A7S` scenario

This is the Phase 060 centerpiece for HJMT topology closure.

Required scenario behavior:

1. start from a lawful `3A7S` topology;
2. execute representative HJMT work on that topology;
3. remove one aggregator and continue as `2A7S` without violating lineage or
   route ownership;
4. add two aggregators and continue as `5A7S`;
5. verify that the public or durable evidence still reflects lawful shard
   ownership, standby assignments, and publication behavior.

Required assertions:

- all `7` shards remain owned throughout the transitions;
- same-lineage failover constraints hold at every step;
- the process map or manifest shows the intended process count at each step;
- route generation increments correctly;
- the removed aggregator does not remain in standby or owner tables;
- re-expansion to `5A7S` does not silently rewrite prior lineage.

#### 📏 B5. Separate evidence lanes before re-measuring

Phase 060 must stop mixing these four measurement classes:

1. Criterion measured closure timings;
2. whole-command resource timings from `/usr/bin/time -v`;
3. scenario stage-runtime splits;
4. user-facing throughput claims.

Required report rules:

- whole-command timings must be labeled as end-to-end command resource usage,
  not pure algorithmic throughput;
- throughput claims must be backed by an explicit run-root artifact that reports
  `durable_root_published_tps`;
- proof-size or single-proof verify-time reports must not be used as TPS
  evidence;
- every benchmark report must say which measurement lane it belongs to.

#### 🚀 B6. Re-run measurements only after topology and provenance close

Once B1 through B5 are complete, rerun the relevant HJMT evidence and update
the docs.

Only then is it valid to decide whether the next optimization wave belongs in:

- scheduler concurrency;
- journal or commit path;
- publication path;
- checkpoint artifact work.

Required performance promotion gate for `shard_process`:

- compare `aggregator_owned` and `shard_process` on the same hardware, profile,
  shard count, operation mix, cache mode, persistence mode, and route generation;
- report `durable_root_published_tps`, `worker_local_tps`, `hjmt_journal_sync`,
  publication latency, blocked time, RSS, CPU utilization, restart time, and
  failover recovery time;
- reject any promotion claim that improves only `worker_local_tps` while
  `durable_root_published_tps`, journal sync, publication latency, or failover
  cost regresses materially;
- keep the default at `aggregator_owned` unless the measured durable-root path
  and operational recovery evidence both improve or remain neutral.

### ✅ B acceptance criteria

This workstream is complete only when all of the following are true:

- repository docs and tests agree on the live process model;
- `aggregator-config.yaml` supports an explicit shard execution mapping with
  `aggregator_owned` as the backward-compatible production default;
- the optional `shard_process` profile rejects invalid multi-primary process
  configs and records its selected mapping in evidence artifacts;
- no `shard_process` production-default promotion is accepted without the A/B
  performance and failover gate described in B6;
- decommission or removal coverage exists and passes;
- the `3A7S -> 2A7S -> 5A7S` scenario exists and proves lawful fail-down and
  fail-up;
- run-root reporting separates microbench, command-resource, scenario-runtime,
  and throughput evidence;
- no document makes user-facing TPS claims without a matching throughput
  artifact.

### 🧪 B verification anchors

The implementation phase should verify at least:

- `cargo test -p z00z_rollup_node --release test_hjmt_process -- --nocapture`
- `cargo test -p z00z_rollup_node --release test_hjmt_topology -- --nocapture`
- config tests covering default `aggregator_owned`, explicit
  `aggregator_owned`, valid `shard_process`, mixed mapping rejection, and
  multi-primary `shard_process` rejection
- `cargo test -p z00z_simulator --release test_scenario_settlement -- --nocapture`
- the new decommission and RAID-like scenario tests added by this workstream

## 🛡️ Workstream C — Verification Gate Closure

### 🔎 C source signals

Translated from `060-TZ2.md`:

- "The next move is already clear: close `l0-docs`, then project-owned
  advisories in `l4-supply-chain`, then process the `11` high-risk crypto
  scenarios from the adversarial review."

Translated from the Phase 060 verification report TODO markers:

- "Fix the risk register."
- "Convert the adversarial security review into project-owned fixable work."
- "Turn recommended actions into concrete closure steps."
- "Profiling shows the slowest top 5% of events consume 64.52% of measured
  runtime; prioritize acceleration work on those events."

### 📌 C workspace evidence

Direct repository evidence:

- `reports/.../logs/l0-docs.log`: `ERROR: no mdBook book.toml found`
- `reports/.../logs/l0-docs.log`: `Summary: 97 error(s)`
- `reports/.../logs/l0-docs.log`: `ZINV references: 0`
- `.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`: strict mode fails
  when neither `$SPECS_ROOT/book/book.toml` nor repository `book.toml` exists.
- `reports/.../supply-chain/supply-chain-project.md`: project advisories:
  `bincode 2.0.1`, `derivative 2.2.0`, `instant 0.1.13`, `paste 1.0.15`
- `reports/.../supply-chain/reviewed-advisories.toml`: `reviewed = []`
- `reports/.../logs/l4-supply-chain.log`: `Vetting Succeeded (776 exempted)`
  and the log explicitly says the exemptions must be reviewed or shrunk before
  vet coverage is trusted.
- `reports/.../logs/l4-adversarial-review.log`: `findings: 392 total, high=11`
- `reports/.../security/adversarial-review.md`: top project-owned high
  hypotheses include checkpoint lineage and delta integrity; PaymentRequest
  replay and compact-request rebinding; stealth delivery and inbox notification
  confusion.
- `reports/.../security/adversarial-summary.json`: `high_risk_count` is `11`,
  while `top_findings` lists `10` entries; the detailed markdown high-finding
  section includes the missing vendor-owned unsafe-code row. Phase 060 must
  reconcile this count/list mismatch before treating adversarial closure as
  complete.
- `reports/.../profiling/summary.json`: slowest top `5%` is `4` events consuming
  `3036.124s` / `4705.708s` total (`64.52%`): `l3-verify-fast`,
  `test:workspace`, `l4-supply-chain`, and `supply:semver:origin/main`.
- `reports/.../profiling/resources-summary.json`: top CPU-total gates are
  `l3-verify-fast`, `l4-unsafe`, `l4-fuzz`, `l4-supply-chain`, and
  `l4-constant-time`; top RSS gates are `l4-fuzz`, `l3-verify-fast`,
  `l4-unsafe`, `l4-constant-time`, and `l4-supply-chain`.

### ⚙️ C1. Close `l0-docs`

#### ✅ C1 actual gap

The current docs gate mixes one governance issue with two genuine content
issues:

1. the gate assumes mdBook topology in strict mode, but the repository does not
   currently provide a `book.toml`;
2. the checked Markdown set has a real style-lint backlog;
3. the traceability layer sees invariants but zero `ZINV` references.

#### 🔒 C1 required Phase 060 decision

Phase 060 must choose and codify one docs posture. The recommended posture is:

- mdBook is optional until the repository explicitly declares a book root;
- missing `book.toml` must not fail strict `l0-docs` unless:
  - a repo-level `book.toml` exists; or
  - a dedicated env or manifest flag explicitly requires mdBook.

This is the stronger closure than "add a dummy book" because it aligns the gate
with the live repository topology instead of creating artificial doc structure.

#### 🧹 C1 required work

1. Update `check-docs.sh` so strict mode only fails on missing mdBook when the
   repository intentionally opts into mdBook.
2. Fix the actual Markdownlint backlog in the files already listed by the log.
   The first closure pass must cover at least:
   - `docs/tech-papers/benchmarks.md`
   - `docs/tech-papers/refactor-recomendations.md`
   - `docs/tech-papers/TODO-Wallet-idea.md`
   - `docs/tech-papers/Z00Z-Multi-DA-and-Checkpoint-Architecture.md`
   - `docs/tech-papers/Z00Z-Multi-DA-Celestia-ecosystem-addons.md`
3. Add explicit `ZINV` references where security-critical docs are supposed to
   justify code claims.

#### 🧾 C1 required traceability outcome

Phase 060 must not stop at "lint is green". It must also connect at least the
following topics to explicit invariant anchors:

- genesis bootstrap authority;
- HJMT topology or lineage rules;
- wallet object-family boundary;
- right or voucher fail-closed behavior;
- checkpoint or replay-sensitive security claims.

#### ✅ C1 acceptance criteria

- `Z00Z_L0_STRICT=1` docs gate passes.
- Missing mdBook no longer fails strict mode unless the repo explicitly opts in
  to mdBook.
- The current checked Markdown backlog is closed.
- `ZINV references` is greater than zero and tied to real security-critical
  docs.

### ⚙️ C2. Close `l4-supply-chain`

#### ✅ C2 actual gap

The supply-chain gate is blocked for three different reasons:

1. four project-owned advisories are unresolved and unreviewed;
2. the repository has no advisory review records at all;
3. `cargo-vet` is in bootstrap mode with `776` exemptions, so the current
   success signal is not mature trust.

#### 🧭 C2 advisory-specific closure rules

Required Phase 060 handling:

- `bincode 2.0.1 / RUSTSEC-2025-0141`
  - treat as highest priority because it is a project-owned dependency anchored
    in `z00z_utils` and fans out widely through `z00z_core` and `z00z_wallets`;
  - closure must be either:
    - dependency replacement or upgrade to a maintained alternative; or
    - an explicit reviewed exception with owner, reason, scope, and sunset.

- `paste 1.0.15 / RUSTSEC-2024-0436`
  - treat as transitive through the `z00z_crypto` tree;
  - closure must inventory whether the dependency can be upgraded out of the
    current `p3-*` chain or must stay as a reviewed exception.

- `derivative 2.2.0 / RUSTSEC-2024-0388`
  - report anchors it to `z00z_wallets`;
  - closure must confirm the live ancestry and either remove the dependency or
    record a reviewed exception with scope and deprecation path.

- `instant 0.1.13 / RUSTSEC-2024-0384`
  - report anchors it to `z00z_wallets`;
  - closure must confirm the live ancestry and either remove the dependency or
    record a reviewed exception with scope and deprecation path.

- `bincode 1.3.3 / RUSTSEC-2025-0141` in vendor trees
  - direct vendor edits are forbidden;
  - closure may only be wrapper, pin, upstream, or isolation strategy.

#### 🧾 C2 required work

1. Populate `reviewed-advisories.toml` with real records instead of an empty
   list.
2. For each project-owned advisory, inventory:
   - dependency ancestry;
   - code path criticality;
   - replacement cost;
   - temporary exception conditions if replacement is not immediate.
3. Reduce or justify cargo-vet bootstrap exemptions until the repository can
   claim meaningful vet trust.
4. Re-run the supply-chain gate only after both advisory review records and
   exemption review exist.

#### ✅ C2 acceptance criteria

- No project-owned advisory remains both unresolved and unreviewed.
- `reviewed-advisories.toml` contains repository-owned decisions.
- Vendor findings are tracked only through wrapper or upstream actions.
- Cargo-vet trust is no longer represented solely by a bootstrap exemption set.

### ⚙️ C3. Close `l4-adversarial-review`

#### ✅ C3 actual gap

The adversarial report is not a proof of exploits. It is a prioritized attack
surface hypothesis set. The real gap is the absence of closure artifacts for the
`11` high findings, plus one report consistency issue: `high_risk_count = 11`
must remain reconciled with the rendered high-finding list and the JSON
`top_findings` list.

#### 📌 C3 high-finding normalization

Phase 060 should treat the `11` high findings as two classes:

- `7` project-owned items that require concrete closure artifacts:
  - checkpoint lineage and delta integrity;
  - PaymentRequest replay and compact-request rebinding;
  - stealth delivery and inbox notification confusion;
  - crate-level concentration in `crates/z00z_storage`;
  - module-level concentration in `crates/z00z_storage/src/settlement`;
  - nondeterministic-source hypothesis in `hjmt_scheduler.rs`;
  - nondeterministic-source hypothesis in `timing.rs`.

- `4` protected-vendor items that require wrapper, isolation, or upstream
  tracking only:
  - crate-level concentration in `crates/z00z_crypto/tari/crypto`;
  - module-level concentration in `.../ristretto`;
  - proof-adjacent logging in `ristretto_keys.rs`;
  - unsafe block review in `ristretto_keys.rs`.

#### 🧪 C3 required closure artifact for each project-owned high

Each project-owned high finding must end in exactly one of these outcomes:

1. a failing test or harness that reproduces the risk, followed by a fix;
2. a negative or property harness that proves the hypothesized bypass does not
   occur under the intended contract;
3. a short closure memo that explains why the finding is a false positive and
   cites the exact code or test anchors that discharge it.

Required artifact fields:

- finding id;
- owner crate or module;
- threat statement;
- reproduction or proof strategy;
- linked tests or proof artifacts;
- final status: fixed, disproved, or accepted-risk.

#### 🎯 C3 priority order

The first three findings to close must remain the same ones already elevated by
the report:

1. checkpoint lineage and delta integrity;
2. PaymentRequest replay and compact-request rebinding;
3. stealth delivery and inbox notification confusion.

Only after those are owned and artifact-backed should the broader storage or
timing concentration findings be treated as closed.

#### ✅ C3 acceptance criteria

- Every high finding has an owner and a closure artifact.
- Project-owned highs are not left as prose-only hypotheses.
- Vendor highs are tracked without editing protected vendor code.
- The adversarial summary count, JSON `top_findings`, and markdown high-finding
  list are reconciled or the mismatch is documented as a report-generation bug
  with no hidden unowned high finding.
- A rerun of the adversarial review can still produce medium findings, but the
  current high findings are either fixed, disproved, or formally accepted with
  evidence.

### 🧪 C verification anchors

The implementation phase should verify at least:

- `Z00Z_L0_STRICT=1 ./.github/skills/z00z-l0-spec-gate/scripts/check-docs.sh`
- `Z00Z_L4_STRICT=1 ./.github/skills/z00z-l4-security-engineering-gate/scripts/audit-supply-chain.sh`
- `python3 ./.github/skills/z00z-verification-orchestrator/scripts/run-security-brainstorm.py --root . --scope-kind project --target-root-rel "" --verification-root reports/z00z-verification-orchestrator-<stamp>/verification<stamp> --summary-out reports/z00z-verification-orchestrator-<stamp>/security/adversarial-summary.json --report-out reports/z00z-verification-orchestrator-<stamp>/security/adversarial-review.md`
- a report-consistency check proving that adversarial `high_risk_count`, JSON
  `top_findings`, and rendered markdown high-finding rows agree or that any
  mismatch is documented and tracked.

### ⚙️ C4. Reduce verification-pipeline runtime for the top slowest events

#### ✅ C4 actual gap

The Phase 060 verification report contains performance guidance, but it must not
stay as broad advice. The real gap is that the slowest verification events are
not yet converted into an optimization backlog with measurable before/after
criteria.

Measured bottleneck classes:

| Priority | Evidence row | Main cost shape | Required disposition |
| --- | --- | --- | --- |
| 1 | `l3-verify-fast` wall `1073.121s`, CPU total `6101.64s`, FS out `64464048` | compile-heavy Rust gate plus repeated artifact output | split compile-heavy and execution-heavy work; reuse a stable target dir and feature set. |
| 2 | `test:workspace` wall `995.056s` | full workspace release test cycle | prebuild shared test binaries once and avoid duplicate compile+run work across gates. |
| 3 | `l4-supply-chain` wall `490.596s`, exit `FAIL` | dependency graph, audit, semver, vet, and unsafe scans recomputing related workspace metadata | batch dependency and unsafe scans after one metadata resolution pass. |
| 4 | `supply:semver:origin/main` wall `477.351s` | semver check isolated as expensive subcommand | cache or scope semver baselines and avoid rerunning unchanged crates. |
| 5 | CPU/RSS-heavy gates: `l4-unsafe`, `l4-fuzz`, `l4-constant-time` | high CPU or memory pressure even when wall time is lower | add resource budgets and avoid moving wall-time cost into RSS or CPU-total regressions. |

#### 🧭 C4 required work

1. Add a verification-pipeline performance inventory that records, for every
   gate and expensive subcommand:
   - wall time;
   - CPU total;
   - CPU percent;
   - max RSS;
   - filesystem output;
   - target directory and cache directory used;
   - whether the command compiled, executed, analyzed, or only formatted.
2. Normalize Rust gate execution around one stable release feature set so Cargo
   can reuse compiled artifacts across `l3-verify-fast`, workspace tests, Kani,
   Miri, fuzz, unsafe, and constant-time gates where tool constraints allow it.
3. Split compile-heavy gates from execution-heavy gates:
   - build or prebuild shared test binaries once;
   - run test or verification slices against the already-built artifacts when
     the tool supports that model;
   - keep tool-specific target directories only where required for soundness.
4. Refactor supply-chain checks to share one workspace dependency metadata pass:
   - cargo audit classification;
   - reviewed-advisory classification;
   - vendor/project split;
   - cargo-vet review;
   - semver baseline check;
   - unsafe/vendor scans when they only need the same inventory.
5. Treat `l4-fuzz`, `l4-unsafe`, and `l4-constant-time` as resource-pressure
   risks even if they are not all top wall-clock events:
   - set memory and CPU-total budgets;
   - record whether any optimization shifts time into RSS, FS output, or CPU
     total;
   - keep fuzz and constant-time evidence valid rather than weakening them to
     reduce runtime.

#### 🚫 C4 false wins to reject

- Do not reduce verification runtime by skipping security gates without a
  replacement evidence path.
- Do not compare runs that use different feature sets, target directories,
  cache roots, or hardware as if they prove an optimization.
- Do not claim a win from wall-clock reduction if CPU total, max RSS, filesystem
  output, or failure rate regresses materially.
- Do not merge Kani, Miri, fuzz, constant-time, or semver checks into a shared
  path if the tool's soundness depends on an isolated target directory or
  profile.

#### ✅ C4 acceptance criteria

- The top slowest report section is represented by concrete work items, not only
  prose recommendations.
- A before/after report exists for `l3-verify-fast`, `test:workspace`,
  `l4-supply-chain`, `supply:semver:origin/main`, and the CPU/RSS-heavy
  `l4-unsafe`, `l4-fuzz`, `l4-constant-time` gates.
- The optimized pipeline preserves the same pass/fail semantics and artifact
  contract as the baseline run.
- At least one of these is true for each optimized path:
  - wall time improves;
  - repeated compile work is removed;
  - metadata resolution is shared;
  - resource budgets are documented and no longer unbounded.
- No accepted optimization weakens fuzz, constant-time, unsafe, semver, or
  formal-verification evidence.

### ⏱️ Required order inside Workstream C

The source note order is correct and should remain the execution order:

1. close `l0-docs`;
2. close project-owned `l4-supply-chain` advisories and vet trust;
3. close the `11` adversarial high findings, starting with the top three
   cross-crate hypotheses.
4. optimize the verification-pipeline top slowest events after the gate
   semantics are stable enough to compare before/after runs honestly.

## 👛 Workstream D — Wallet Typed Inventory And MVP Rights Or Vouchers Profile Matrix

### 🔎 D source signals

Translated from `060-TZ2.md`:

- "Define the optimal wallet storage structure for assets, rights, and
  vouchers."
- "Collect rights and vouchers profiles, compliance surfaces, actions, and the
  MVP-important functionality across all documents."

Translated from the wallet-idea note already embedded in `060-TZ2.md`:

- "Coins stay in the wallet. A `RightLeaf` encumbers them. Normal spend is
  blocked. Only policy-approved transitions are allowed: unlock, redelegate,
  challenge, slash, reward claim."

Source lock primitive details that must be carried into later plans:

```text
AssetLeaf / owned note
  -> locked asset state remains under holder control
  -> RightLeaf::ValidatorMandate or lock profile is created
  -> payload_commitment binds locked asset commitments + amount commitment + terms
  -> transition_policy_id defines unlock/redelegate/slash/reward rules
```

The source note maps the lock profile onto these existing right fields:

- `right_class = ValidatorMandate`;
- `holder_commitment` binds the holder of the lock;
- `control_commitment` binds who can perform approved transitions;
- `beneficiary_commitment` binds reward or benefit destination;
- `payload_commitment` binds locked assets, amount, and terms;
- `valid_from` and `valid_until` bind the lock window;
- `challenge_from` and `challenge_until` bind dispute or slashing windows;
- `use_nonce` is the anti-replay lock id;
- `transition_policy_id` binds unlock, redelegate, reward, and slash grammar;
- `revocation_policy_id`, `disclosure_policy_id`, and
  `retention_policy_id` bind emergency termination, selective disclosure, and
  evidence-retention behavior.

The minimum `StakeLockRight` source model is:

```text
StakeLockRight v1:
  type: ValidatorMandate
  locked_asset_commitment
  locked_amount_commitment
  validator_or_pool_scope
  holder_commitment
  reward_beneficiary_commitment
  lock_start
  lock_until
  redelegation_policy_id
  unlock_policy_id
  reward_policy_id
  optional_disclosure_policy_id
```

The preferred v1 construction is rewrapped locked asset plus
`RightLeaf::ValidatorMandate`, because it keeps the user's coins under
self-custody while making the ordinary spend path reject locked value at the
protocol-visible boundary. Pure wallet UI soft-locks are explicitly rejected.

### 📌 D workspace evidence

Direct repository evidence:

- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`: "The wallet therefore
  exposes three projections on one inventory plane: spendable cash assets;
  voucher claims and voucher lifecycle state; right authority inventory and
  right lifecycle state."
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`: "Unknown-policy objects
  remain in durable quarantine and are excluded from spendable balance."
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`:
  `"asset rows must continue using wallet_asset_store cash persistence"`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`:
  "`wallet.asset.*` remains cash-only." and
  "`wallet.object.*` is the typed object namespace ..."
- `crates/z00z_core/src/assets/assets_config.yaml`: live right classes already
  present in fixtures: `machine_capability`, `data_access`,
  `service_entitlement`, `validator_mandate`, `one_time_use`

Cross-document product or policy intent already present in the repository:

- `docs/Z00Z-Tokenomics-Incentives-Whitepaper.md`: "`FeeCredit`: A
  non-transferable prepaid processing entitlement backed by locked or budgeted
  `Z00Z`."
- `docs/Z00Z-Litepaper.md`: "`Agent spending envelope`: A bounded private
  mandate that gives an agent a task-scoped budget, fee capacity, and action
  limits without granting full wallet authority."
- `docs/Z00Z-UseCases-Whitepaper.md`: the protocol families already include
  policy-shaped money and claims, private organizational settlement, private
  distribution, and service or machine or agent rights.

### ✅ D actual gap

The wallet gap is not persistence design. The typed wallet inventory is already
live.

The real remaining gap is this:

- Phase 059 built the object-family storage plane;
- Phase 060 must decide which rights or voucher profiles matter first, what
  their action grammar is, what compliance or disclosure surfaces they need,
  and which wallet or validator behaviors are required for MVP.

In other words, the missing piece is the product or policy matrix on top of the
existing object model.

### 🚫 D false gaps to reject

Do not do the following in Phase 060:

- Do not create a second wallet database for rights or vouchers.
- Do not redesign `.wlt` persistence from scratch.
- Do not introduce new primitive leaves like `StakeLeaf`, `EscrowLeaf`, or
  `ReserveLeaf` just because new profiles are needed.
- Do not rely on a UI-only soft lock for staking or lock flows.
- Do not start with fully slashable self-custody staking as v1.

### 🎛️ MVP scope decisions

Phase 060 should lock these scope decisions:

1. `AssetLeaf` stays the private value object.
2. `RightLeaf` stays the bounded control or permission or encumbrance object.
3. `VoucherLeaf` stays the conditional-value or redeemable-claim object.
4. The wallet remains one typed inventory plane with three projections, not
   multiple local authorities.
5. The first new high-value profile is `validator_mandate`-based
   self-custody locking, but v1 should be non-slashable or challenge-bounded,
   not a full slashable economic-security bond.

### 📐 Required MVP profile matrix

Phase 060 must publish one explicit profile matrix that later plans can
implement. The minimum recommended matrix is:

| Profile | Family | Repository anchor | MVP actions | MVP compliance or policy surfaces | Required fail-closed rules |
| --- | --- | --- | --- | --- | --- |
| `fee_credit_v1` | Voucher | `FeeCredit` term in tokenomics paper | issue, transfer if allowed, redeem for fee lane, expire, refund if policy allows | backing source, non-transferable vs transferable flag, expiry, sponsor scope | must reject voucher-as-cash, missing backing, replay, stale root |
| `service_entitlement_v1` | Right | live `service_entitlement` class | grant, transfer if policy allows, consume, revoke, expire | disclosure policy, retention policy, provider scope, beneficiary scope | must reject out-of-scope right, revoked or expired right, unknown policy |
| `data_access_v1` | Right | live `data_access` class | grant, consume, expire, challenge, revoke | disclosure policy, retention policy, audit trail, challenge window | must reject expired access, challenge-window misuse, wrong beneficiary |
| `agent_budget_v1` | Right | live `machine_capability` and `one_time_use`, plus litepaper `agent spending envelope` | delegate, spend within bounded action set, consume quota, revoke | amount or quota bound, action whitelist, service scope, expiry | must reject over-budget action, unauthorized action family, consumed right reuse |
| `validator_mandate_lock_v1` | Right plus asset-state rule | live `validator_mandate` class plus wallet idea note | lock, unlock, redelegate, reward-claim, challenge-bounded revoke | payload commitment to locked asset or amount, lock window, beneficiary, challenge window | ordinary spend must reject locked asset; unlock must reject replay or stale right |
| `transferable_claim_v1` | Voucher | Phase 059 voucher object model | offer, accept, transfer if allowed, partial redeem, redeem, reject, expire | backing or reserve source, accept-policy, transferability, residual handling | must reject wrong-family proof, double redeem, expired use, unknown policy |

This matrix is the minimum. Additional profiles may be added later, but Phase
060 should not leave MVP semantics unprioritized.

### ⚙️ D required work

#### 🧾 D1. Publish the authoritative profile catalog

Required output:

- one repository-owned document or spec table that defines each MVP profile;
- the object family used by the profile;
- mandatory policy ids and optional policy ids;
- allowed actions;
- wallet-visible states;
- validator or watcher reject surfaces.

This catalog should merge the live codebase with the documented product intent
already present in the whitepapers and wallet notes.

#### 🗂️ D2. Define wallet projection semantics per profile

For each MVP profile, specify:

- how it appears under `wallet.object.*`;
- whether it also affects `wallet.asset.*` balance visibility;
- when it becomes `Quarantined`;
- how lifecycle status maps to spendability or usability.

Required special handling:

- any profile whose policy availability is not `Available` must remain
  quarantined;
- lock profiles must affect ordinary asset selection, not only object listing.

#### 🔒 D3. Specify `validator_mandate_lock_v1`

This is the most important new profile to define concretely.

Required Phase 060 decision:

- v1 uses rewrapped or otherwise protocol-visible locked asset semantics plus
  `RightLeaf::ValidatorMandate`;
- ordinary spend of the locked asset must fail at the builder and validator
  boundary;
- the initial version must be non-slashable or challenge-bounded;
- full slashable bond logic is deferred until a separate proof model exists.

Required design outputs:

- object fields bound into `payload_commitment`;
- wallet-visible locked-state semantics;
- unlock or redelegate transition grammar;
- reward-claim relation to the active right;
- explicit non-goals for v1.

#### 🧪 D4. Expand validator, watcher, and simulator coverage

For each MVP profile, add or plan tests that prove:

- wrong-family proof fails closed;
- unknown policy remains quarantined;
- right-as-value and voucher-as-cash are rejected;
- missing or expired or revoked or consumed rights are rejected;
- lock profile blocks ordinary spend;
- redeem or unlock replay is rejected.
- unlock after expiry is accepted only through the approved unlock transition;
- active stake or lock proof can be verified without revealing unrelated wallet
  assets;
- unrelated assets remain selectable and hidden from the lock profile.

#### 📦 D5. Keep the one-plane wallet authority

Every profile extension must preserve:

- `.wlt` as canonical wallet authority;
- `WalletExportPack` as canonical transfer bundle;
- `wallet.object.*` as the typed object namespace;
- `wallet.asset.*` as cash-only projection;
- no second wallet-local authority surface.

### ✅ D acceptance criteria

This workstream is complete only when all of the following are true:

- one explicit MVP profile matrix exists and is repository-owned;
- the matrix covers actions, policy surfaces, lifecycle rules, and fail-closed
  behavior;
- the first lock or staking-like profile is specified on top of the current
  object model, not via a new primitive leaf;
- wallet, validator, and watcher expectations are aligned for each profile;
- no new design reintroduces a second wallet authority plane.

### 🧪 D verification anchors

The implementation phase should preserve and extend the current typed-object
coverage already present in:

- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`
- `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`
- `crates/z00z_runtime/watchers/tests/test_object_alerts.rs`
- `crates/z00z_simulator/tests/test_scenario1_object_flows.rs`

## ⚠️ Risk Controls And Pitfalls

These controls are mandatory when splitting this SPEC into implementation plans.

| Area | Pitfall | Mitigation required by this SPEC |
| --- | --- | --- |
| A | Adding symmetric `*_config.yaml` files creates more authority planes. | Keep `z00z_core::genesis` as the canonical bootstrap authority and treat any remaining asset YAML as registry, fixture, or compatibility data. |
| A | Duplicating or re-homing the canonical JSON helper without proof can reintroduce drift in descriptor hashing semantics. | Keep one owner path in `z00z_utils::codec` and reject any second helper or silent re-home unless a later cross-crate design proves the need. |
| A | Compatibility shims in `assets/` continue to look like preferred authoring paths. | Move internal imports to real owner modules and add review or grep enforcement against new shim-first imports. |
| B | Treating `1 shard = 1 process` as already-required behavior breaks the live aggregator contract. | Add it only as opt-in YAML `shard_process`; keep `aggregator_owned` as default until A/B evidence justifies promotion. |
| B | A throughput win is claimed from worker-local metrics while durable publication regresses. | Promote only on `durable_root_published_tps`, journal sync, publication latency, blocked time, restart time, and failover recovery time. |
| B | Benchmark lanes are mixed into one story. | Label Criterion closure timings, whole-command `/usr/bin/time`, scenario splits, and user-facing TPS separately. |
| B | Async scheduler parallelism is claimed without live evidence. | Require `scheduler_metrics.max_active`, durable-root TPS, and route or process-map evidence from the same run. |
| C | `l0-docs` is "fixed" by adding a dummy book root that does not represent repo topology. | Make mdBook failure opt-in and still close the real Markdown and invariant-traceability backlog. |
| C | Verification pipeline runtime is reduced by skipping security evidence. | Keep the same pass/fail semantics and artifact contract; no C4 optimization may weaken fuzz, unsafe, constant-time, semver, or formal gates. |
| C | Adversarial findings are treated as proven exploits or dismissed as noise. | Every high finding must become a claim, harness, fix, disproval artifact, or accepted-risk memo. |
| C | Protected vendor issues are "fixed" by editing vendor code. | Use wrappers, pins, upstream actions, isolation, or documented exceptions only. |
| D | Wallet UI soft-lock is mistaken for protocol-level locked stake. | Locked profiles must affect builder and validator acceptance, not only local UI selection. |
| D | New profile demand creates new primitive leaves or a second wallet database. | Keep `AssetLeaf`, `RightLeaf`, `VoucherLeaf`, `.wlt`, `WalletExportPack`, and `wallet.object.*` as the authority model. |
| Cross-workstream | Plans drift from source TODO intent after decomposition. | Preserve the Source TODO Inventory mapping and publish a closure memo that records fully closed, deferred, and accepted-risk items. |

## ⏱️ Global Execution Order

The recommended Phase 060 execution order is wave-based rather than purely
serial.

### 🚦 Wave 1 — Contract clarity

1. Close `C1` docs-gate posture and current Markdown backlog.
2. Freeze `A1` canonical bootstrap authority in `z00z_core`.
3. Freeze `B1` live HJMT topology contract.
4. Specify `B2` YAML shard execution mapping with `aggregator_owned` as the
   production default.

### 🔧 Wave 2 — Owner boundaries and testable topology

1. Execute `A2` and `A3` rights-owner and shim cleanup.
2. Execute `B3` decommission coverage.
3. Execute `B4` `3A7S -> 2A7S -> 5A7S` scenario.
4. Execute `C2` supply-chain advisory review and cargo-vet maturity work.

### 📐 Wave 3 — Product matrix and security closure

1. Execute `D1` and `D2` profile catalog and wallet projection semantics.
2. Execute `D3` `validator_mandate_lock_v1` specification.
3. Execute `C3` adversarial high-finding closure, starting with the top three
   cross-crate hypotheses.
4. Execute `B5` and `B6` measurement-lane normalization and HJMT reruns.

### ✅ Wave 4 — Pipeline performance and final attestation rerun

1. Execute `C4` verification-pipeline performance work once gate semantics are
   stable enough for honest before/after comparison.
2. Re-run `l0-docs`.
3. Re-run `l4-supply-chain`.
4. Re-run the adversarial review.
5. Re-run the targeted `z00z_core`, HJMT, wallet, validator, watcher, and
   simulator tests touched by the above waves.
6. Publish one short Phase 060 closure memo that records:
   - which TODO markers were fully closed;
   - which were split into later phases by design;
   - which risks remain accepted and why.

## ✅ Doublechecked Evidence Basis

This specification was produced by checking the repository directly against the
Phase 060 TODO markers. The main evidence set includes:

- `.planning/phases/060-Gaps-Closing/060-TZ1.md`
- `.planning/phases/060-Gaps-Closing/060-TZ2.md`
- `.planning/phases/060-Gaps-Closing/060-z00z-verification-report.md`
- `crates/z00z_core/src/genesis/README.md`
- `crates/z00z_core/src/assets/assets_config.yaml`
- `crates/z00z_core/README.md`
- `crates/z00z_core/src/assets/mod.rs`
- `crates/z00z_core/src/assets/right_config.rs`
- `crates/z00z_core/src/genesis/genesis_rights.rs`
- `crates/z00z_core/src/rights/mod.rs`
- `crates/z00z_storage/src/settlement/leaf.rs`
- `crates/z00z_rollup_node/src/config.rs`
- `crates/z00z_rollup_node/tests/test_hjmt_process.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `docs/tech-papers/done/Z00Z-HJMT-Upgrade.md`
- `crates/z00z_storage/benches/settlement_shard.rs`
- `crates/z00z_storage/scripts/run_storage_settlement_bench.py`
- `reports/z00z-verification-orchestrator-20260618-170025/logs/l0-docs.log`
- `reports/z00z-verification-orchestrator-20260618-170025/logs/l4-supply-chain.log`
- `reports/z00z-verification-orchestrator-20260618-170025/logs/l4-adversarial-review.log`
- `reports/z00z-verification-orchestrator-20260618-170025/profiling/summary.json`
- `reports/z00z-verification-orchestrator-20260618-170025/profiling/resources-summary.json`
- `reports/z00z-verification-orchestrator-20260618-170025/supply-chain/supply-chain-project.md`
- `reports/z00z-verification-orchestrator-20260618-170025/supply-chain/reviewed-advisories.toml`
- `reports/z00z-verification-orchestrator-20260618-170025/security/adversarial-review.md`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`
- `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`
- `docs/tech-papers/TODO-Wallet-idea.md`
- `docs/Z00Z-Tokenomics-Incentives-Whitepaper.md`
- `docs/Z00Z-Litepaper.md`
- `docs/Z00Z-UseCases-Whitepaper.md`

The spec intentionally refuses several tempting but incorrect remediations:

- adding symmetric YAML config files without authority need;
- moving consensus-critical canonicalization into generic utils too early;
- treating "one shard = one process" as already-required behavior;
- redesigning wallet persistence instead of completing the profile layer;
- treating adversarial heuristics as proof without closure artifacts.
