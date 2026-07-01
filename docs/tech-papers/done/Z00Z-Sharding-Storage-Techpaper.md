# Z00Z HJMT Sharding, Runtime Placement, And Storage Techpaper

[TOC]

Version: 2026-06-05

Status: Informative architecture techpaper

Companion normative specification: [Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md)

Source migration: analytical material preserved from `TODO-HJMT-RAID-Sharding.md`

## Document Role

This paper preserves the analytical recommendations, option comparisons, and
rollout guidance that were collected while preparing the HJMT upgrade. The
normative rules live in [Z00Z-HJMT-Upgrade.md](Z00Z-HJMT-Upgrade.md). This
paper exists so the former TODO can be removed without losing:

- the rationale behind the runtime sharding model;
- the high-availability recommendation for aggregator placement;
- the backend and journal evolution strategy;
- the crate, module, and signature guidance for future implementation slices;
- the simulation and evidence recommendations that explain how to de-risk the
  rollout.

If this paper and the companion upgrade specification ever disagree, the
upgrade specification wins.

Whenever this paper uses strong language such as "must", "must not", or
"required", it is either:

- restating a rule that already belongs to the companion upgrade
  specification; or
- preserving a design recommendation that explains why the normative rule was
  chosen.

It is not intended to become a second competing normative source.

## Key Analytical Terms

| Term | Meaning in this paper |
| --- | --- |
| `AggregatorId` | Runtime identifier for one aggregator or shard executor. It is operational metadata, not protocol truth. |
| `ShardPlacementTable` | Runtime-only mapping from committed `ShardId` values to active and standby executors plus lineage expectations. |
| `ShardExecutor` | Runtime worker that executes one shard queue and replays one shard journal lineage. |
| `ShardGroupId` | Future replicated-journal group scope for one or more shards. It is an evolution seam, not version 1 protocol truth. |
| `StorageBackend` | Backend-neutral durable KV and transaction seam. |
| `JournalBackend` | Backend-neutral durable journal seam for append and replay. |
| Runtime placement | Operational execution placement under already-committed shard ownership. |
| Lawful failover | Same-lineage resume for the same `ShardId` and the same `routing_generation`. |

## Executive Recommendation

The recommended direction is:

1. Keep `z00z_rollup_node` as the orchestration root, keep aggregation as a
   runtime service, and keep `z00z_storage` as the semantic truth layer.
1. Treat RAID-like behavior as lawful same-lineage failover for a committed
   `ShardId`, not as transparent reroute or ad-hoc shard reassignment.
1. Keep the current `RedB`-backed storage baseline, but isolate it behind
   narrow `StorageBackend` and `JournalBackend` seams.
1. Prefer one `z00z_storage` crate with tighter internal module boundaries
   before attempting a large crate split.
1. Use one-machine multi-aggregator simulation and shared backend conformance
   tests as the first safety gate.
1. Add replicated journal later, most likely at `ShardGroupId` scope, through
   an adapter to an existing substrate rather than a custom consensus stack.

The core discipline is simple:

- `z00z_rollup_node` orchestrates.
- Aggregator runtime services admit, route, execute, and publish shard-local
  work.
- Validators and watchers remain downstream runtime consumers.
- `z00z_storage` owns semantic settlement truth and durable execution
  primitives.

The goal is not "more crates". The goal is preserving the correct owner at each
layer while making failover, journaling, and backend evolution safe.

## Current Workspace Baseline

The current repository already has the right top-level shape for this work.

| Current seam | Current role | Recommended interpretation |
| --- | --- | --- |
| `z00z_rollup_node` | Node lifecycle and service composition root | Keep as the orchestration super-layer. |
| `z00z_runtime/aggregators` | Aggregator runtime service boundary | Keep as the canonical runtime-service layer for ingress, planning, execution, and publication handoff. |
| `z00z_runtime/validators` | Checkpoint and verdict runtime service | Keep downstream from aggregation. |
| `z00z_runtime/watchers` | Observation and evidence export service | Keep operational and non-authoritative. |
| `z00z_storage` | Settlement facade, proof surface, and durable execution | Keep as the semantic truth layer and current storage facade. |

This means the repository does not need a new top-level "super-aggregator"
crate. The important work is boundary hardening, not authority inversion.

That said, structural shape is not the same thing as completed shard-safe
behavior. The current repository should still be read as:

- structurally aligned with the target layering;
- not yet proven by itself to provide protocol-level shard scaling;
- not yet a substitute for route vectors, failover evidence,
  root-of-shard-roots publication evidence, and backend conformance evidence.

Bounded scheduler parallelism is not the same as durable public-root
throughput, and current runtime seams are not evidence of finished failover.

## Whole-System Hierarchy

### Structural Ownership

The recommended hierarchy is:

```text
z00z_rollup_node
  -> runtime aggregator service
  -> runtime validator service
  -> runtime watcher service
  -> DA adapter surface

runtime aggregator service
  -> batch planner
  -> shard executor
  -> publication coordinator
  -> settlement engine contracts

settlement engine / HJMT contracts
  -> StorageBackend / JournalBackend wrappers
  -> current durable backend
```

The important negative rules are equally strong:

- `z00z_rollup_node` MUST NOT become shard truth.
- The aggregator layer MUST NOT self-authorize public state.
- Validators and watchers MUST NOT become alternate routing authorities.
- `z00z_storage` MUST NOT become the owner of runtime placement.

### Who Is "Super"

There are three different meanings of "super", and they must not be merged:

| Meaning | Current owner | Why it matters |
| --- | --- | --- |
| Orchestration super | `z00z_rollup_node` | It composes services and adapters into one running node. |
| Runtime execution super | Aggregator runtime service | It owns admission, shard routing application, shard execution, and publication handoff. |
| Semantic truth super | `z00z_storage` | It owns settlement proof contracts, committed roots, and durable execution semantics. |

Confusion appears when one layer tries to absorb another. The safe rule is:

- do not invent a new "super-aggregator" above `z00z_rollup_node`;
- do not move runtime planning into storage ownership;
- do not let runtime placement metadata masquerade as protocol truth.

## Runtime Sharding And Lawful Failover

### Recommended High-Availability Model

The correct mental model is RAID10-style shard placement:

- shard load is striped across multiple physical aggregators;
- each shard has one active writer and one or more standby executors;
- failover resumes the same `ShardId` under the same lineage;
- unaffected shards continue to operate without waiting for the failed shard,
  except at the configured publication boundary.

This is closer to "replicated shard executors" than to "transparent shard
reroute".

Pure RAID1-style full mirroring is not the preferred default scaling model. It
is simpler but wastes write-distribution potential and does not match the
document's shard-local execution intent as well as RAID10-style placement.

### What Counts As Lawful Failover

Failover is lawful only when the replacement executor continues:

- the same `ShardId`;
- the same `routing_generation`;
- the same journal lineage.

If any of those change, the operation is not failover anymore. It is route
migration and requires a committed route change plus normal checkpoint flow.

### What Must Not Happen

The following behavior should stay prohibited:

- silent reroute of a failed shard to another shard in the same routing
  generation;
- dynamic load-based shard ownership changes;
- two active writers for one committed shard lineage;
- using gossip, liveness, or membership records as proof authority;
- presenting synthetic mixed roots assembled from incompatible child states.

### Practical Interpretation Of "Seamless"

"Seamless" is valid only in a narrow sense:

- unaffected shards continue without interruption;
- a prepared standby may resume the failed shard if it can prove same-lineage
  continuation;
- new traffic for an unavailable shard may return retryable
  `shard-unavailable` until lawful resume occurs.

The system is not trying to hide shard ownership changes. It is trying to keep
public truth deterministic while limiting blast radius.

### Split-Brain Risk

The most important runtime risk is split-brain on a committed `ShardId`.
Because of that, the design should assume:

- one active writer per shard lineage;
- explicit fencing or leadership discipline before promotion;
- wrong-lineage rejection during recovery and failover tests;
- carry-forward of the last visible shard leaf when a shard cannot lawfully
  publish a new root.

## Planner Placement And Execution Path

### Planner Ownership

The batch planner belongs to the aggregator runtime layer, not to the storage
backend layer.

The planner is responsible for:

- canonicalizing incoming operations;
- resolving `SettlementPath -> route hash -> ShardId` through the committed
  route table;
- grouping candidate work by `ShardId`;
- rejecting cross-shard batches in version 1;
- emitting a deterministic `BatchPlanned` record for replay and recovery.

Storage may provide helper logic or durable support modules, but it should not
become the semantic owner of batch admission.

### Planner Versus Publication

Planner ownership and publication ownership are different responsibilities:

- the planner decides what enters a shard-local batch;
- the publication boundary decides which shard-local roots become public
  settlement state.

Those functions should not collapse into one runtime heuristic. Public truth
changes only when checkpoint publication is accepted.

### Recommended Flow

```text
Client or wallet
  -> Aggregator API
  -> BatchPlanner
  -> shard-local queue
  -> ShardExecutor
  -> settlement engine
  -> journal and durable commit
  -> publication coordinator
  -> checkpoint acceptance
  -> public SettlementStateRoot
```

This flow preserves a clean ownership chain:

- wallet submits through public API;
- aggregator owns admission and routing application;
- settlement owns execution and recovery semantics;
- storage wrappers own durable primitives only.

## Non-Cosmetic Implementation Gaps

The core architectural direction is sound, but the remaining work is not
cosmetic. The former TODO correctly identified several gaps that still matter
even after the normative upgrade paper was completed.

### The Important Distinction

The current state should be read as:

- the target architecture is mostly specified correctly;
- the implementation is not yet proven complete just because the structure is
  now documented;
- the dangerous failures are at the byte contract, recovery, failover, and
  publication boundaries.

### 1. Proof-Layer Gap

The batch multiproof layer remains a real workstream, not a naming cleanup.
The analytical recommendation was:

- current independent proof generation is not the end state;
- shared multiproof needs deterministic encoding, parser limits, atomic
  verification, tamper vectors, and benchmark evidence;
- a first implementation may be built from already-verified independent proof
  units before deeper witness reuse is moved closer to HJMT internals.

This matters because proof-size and verifier-throughput claims are not credible
until the shared proof envelope is real and reject behavior is hardened.

### 2. Root Model Gap

Root-of-shard-roots is a true storage-root model change, not a wrapper around
the existing state root. The design analysis treated this as a generation shift:

- earlier root generation: one semantic settlement root directly from the
  current HJMT state;
- later root generation: a public commitment over committed shard-root leaves.

That means the work still needs:

- canonical shard-leaf objects;
- route-table and policy bindings;
- checkpoint continuity rules;
- two-layer verification flow;
- migration and historical proof vectors across root generations.

### 3. Routing Gap

Committed route tables must become byte-deterministic artifacts. It is not
enough for routing ideas to be conceptually clear. Independent implementations
must converge on the same canonical bytes and the same migration behavior.

The same section of the design analysis also kept version 1 intentionally
strict:

- cross-shard batches remain rejected;
- no implicit distributed transaction protocol should appear as a fallback;
- executor placement changes are not a substitute for committed route
  migration.

### 4. Durability Gap

Durable throughput claims must be measured at the public-root durable boundary,
not at scheduler throughput, worker throughput, or local delta throughput.

The analytical warning preserved from the TODO is important:

- bounded parallel work scheduling is useful;
- it is not proof of durable settlement throughput;
- root publication still depends on parent recomposition, journal durability,
  and checkpoint-facing state transitions.

Any serious score claim therefore still depends on:

- crash matrices at every durable stage;
- separated worker-local versus public-root timings;
- evidence that recovery returns either the prior public root or the planned
  later root, but never a mixed synthetic state.

### 5. Validator Gate Gap

Checkpoint acceptance remains a real safety boundary. It still needs exact
evidence for:

- exact shard-set coverage;
- sorted and unique shard leaves;
- route-table digest binding;
- prior-root continuity;
- carry-forward byte equality for unchanged shard leaves;
- rejection of mixed old and new child roots.

This is not operational polish. It is the barrier that prevents invalid public
publication from looking superficially plausible.

### 6. Transition And History Gap

Adaptive split, merge, and policy-transition records are not enough by
themselves. The analytical requirement is stronger:

- adaptive transitions need verifier semantics and recovery semantics;
- bucket-layout changes must not reinterpret `SettlementPath` or `ShardId`;
- historical proofs must continue to verify under their historical route,
  policy, epoch, and root generation context.

Without this, storage optimization can silently damage historical verification.

### 7. Failover Gap

Lawful same-lineage failover is the correct model, but it is still only safe
when backed by:

- standby resume behavior;
- wrong-lineage rejection;
- split-brain fencing;
- shard-unavailable behavior;
- carry-forward rules for a shard that cannot yet republish.

This remains one of the highest-risk implementation surfaces because it is easy
to create a system that appears available while violating public-state
determinism.

### 8. Observability Gap

Operational visibility is required for production use, but it must stay
non-authoritative. The preserved analytical rule is:

- watchers and status surfaces should expose shard stall, freeze mode,
  route-table dispute, and recovery evidence;
- they must not become a substitute for committed route generations, shard
  leaves, or accepted checkpoints.

### Practical Reading Of These Gaps

The safest way to read the remaining work is:

1. the architecture does not need a new smart top-level layer;
1. the implementation still needs deterministic proof, route, journal,
   publication, and failover evidence;
1. readiness is achieved by closing those workstreams, not by adding more
   orchestration abstractions.

## Storage Boundary And Backend Strategy

### Storage Seams

`StorageBackend` and `JournalBackend` should remain deliberately boring. They
exist to supply:

- durable KV access;
- read and write transaction boundaries;
- append and replay for journal records;
- backend-neutral wrappers;
- conformance-friendly seams for tests and future migration.

They should not own:

- shard routing decisions;
- batch planning;
- proof semantics;
- checkpoint acceptance;
- wallet semantics;
- protocol-domain type ownership.

### Wallet And Aggregator Boundaries

The dependency rules should stay explicit:

```text
Wallet -> Aggregator API / verifier
Aggregator -> settlement contracts
Settlement -> StorageBackend / JournalBackend wrappers
Wrappers -> current durable backend
```

The forbidden paths are:

```text
Wallet -X-> raw backend internals
Wallet -X-> journal rows as proof authority
Aggregator -X-> redb::Database-style internals
Aggregator -X-> backend-vendor transaction types
```

### Protocol Types Versus Storage Types

Storage-only modules should not become the native home for protocol-domain
types. Keep a clear distinction:

| Protocol or domain types | Storage-only types |
| --- | --- |
| `SettlementPath` | `TableId` |
| `SettlementStateRoot` | `StorageNamespace` |
| `ShardId` | `TxnId` |
| `ShardRouteTableV1` | `JournalOffset` |
| `ShardRootLeafV1` | `DurableCheckpoint` |
| `BatchProofBlobV1` | backend-local codec or schema helpers |

This separation matters even if packaging stays consolidated.

### Current Backend Recommendation

For the current stage, the backend recommendation remains:

- keep `RedB` as the baseline backend;
- add backend abstraction now;
- migrate later only if equal-durability benchmarks show a real storage
  bottleneck.

The reason is practical, not ideological:

- the current HJMT work is already close to `RedB`;
- a premature backend migration adds risk before proof, route, and failover
  evidence is complete;
- the first architectural win is seam isolation, not vendor replacement.

## Packaging And Module Strategy

### Current Recommendation

The most practical packaging now is:

- keep one `z00z_storage` crate;
- tighten internal modules inside it;
- keep wrapper-only public exports;
- keep raw backend internals private or crate-private;
- keep future crate names illustrative until a migration plan explicitly
  adopts them.

This is better than an early split into many tiny storage crates because it
reduces coordination overhead while the architecture is still moving.

### Suggested Internal Storage Modules

A practical internal shape for `z00z_storage` is:

```text
z00z_storage/
  api/
  types/
  journal/
  tables/
  migration/
  codec/
  backends/
    redb/
    memory/
  testkit/
```

This keeps the main seam explicit without forcing immediate workspace
fragmentation.

### Illustrative Future Split

Future names such as the following are still useful as a conceptual map:

- `z00z-types`
- `z00z-hjmt`
- `z00z-settlement`
- `z00z-aggregator`
- `z00z-sim`

But they should be treated as architecture sketches only. They are not adopted
workspace targets unless a separate migration plan explicitly promotes them.

### Public Export Hygiene

Public storage exports may include wrappers such as:

- `RedbStorage`
- `MemoryStorage`
- `StorageBackend`
- `JournalBackend`

Public storage exports should not include raw vendor internals such as:

- `redb::Database`
- raw backend transactions;
- raw table definitions;
- internal schema names;
- backend-private cursor types.

### Signature Discipline

Sample signatures in architecture documents should stay illustrative until they
are promoted into versioned contracts with tests and vectors. They should:

- stay backend-neutral;
- avoid raw vendor types in public interfaces;
- use explicit names;
- remain consistent with repository naming rules;
- avoid accidentally freezing low-level APIs too early.

## Technology Options And Decision Gates

### Recommended Technology Posture

The technology posture should stay staged:

- preserve the current local baseline first;
- introduce narrow seams;
- prove correctness and recovery;
- add replication later behind the same seams;
- upgrade backend vendors only when benchmark pressure justifies it.

### Options Matrix

| Concern | Recommended now | Acceptable later | Notes |
| --- | --- | --- | --- |
| Local durable journal | Current `RedB`-based journal path | `orderwal` or another ordered WAL behind `JournalBackend` | Keep version 1 on the simplest durable path first. |
| Replicated journal | Not required for the first evidence-backed slice | `OpenRaft` at `ShardGroupId` scope | Introduce only after local lineage and failover correctness are proven. |
| Heavy replicated log scale-out | Not the first move | `tikv/raft-rs` plus `raft-engine` | Consider only if many replicated groups become necessary. |
| Membership and liveness | Non-authoritative runtime input only | `memberlist`, `chitchat`, or `foca` | Never treat discovery data as protocol truth. |
| Embedded KV backend | Keep `RedB` | `Fjall`, then `RocksDB` for heavier scale | Migrate only behind equal-durability benchmarks and conformance tests. |
| Experimental replicated log | Not part of the recommended first path | `OmniPaxos` or similar experimental substrate | Keep clearly optional and non-normative. |

### Recommended Production Target

The preserved design recommendation is more specific than "replication later".
The preferred long-term target is:

- RAID10-style shard-executor placement at the physical layer;
- consensus-style journal durability at the shard or shard-group layer;
- one active writer plus fenced standby promotion for each committed lineage.

In plain terms:

- do not stop at a plain sharded local journal and call it finished HA;
- do not treat asynchronous journal mirroring as the final correctness model;
- do treat a quorum-backed lineage model as the likely production destination.

The practical caveat is equally important:

- do not jump to one tiny consensus group per tiny shard on day one;
- prefer `ShardGroupId` scope or another grouped durability boundary first if
  the shard count would otherwise explode operational overhead.

### Transitional Journal Strategy

The former TODO distinguished three useful stages, and that distinction should
remain preserved:

1. shard-local durable journal as the first correctness and recovery baseline;
1. strictly fenced and preferably synchronous replicated journal as an
   acceptable intermediate implementation;
1. consensus-style shard or shard-group journal as the preferred
   production-grade end state.

This is why the techpaper treats a replicated journal as a seam rather than an
instant mandatory baseline. The architecture should still be designed so that
stage 2 can harden into stage 3 without invalidating shard identity or public
proof semantics.

### What Not To Build From Scratch

The design should avoid writing from scratch:

- Raft or Paxos implementations;
- leader election protocols;
- joint membership change logic;
- gossip membership protocols;
- partition-resolution protocols.

Custom work should stay on the Z00Z-specific layer:

- `ShardId` and route-table semantics;
- batch planning and canonical ordering;
- journal lineage rules;
- checkpoint publication objects;
- fail-closed verifier and recovery rules.

## Simulation And Evidence Program

### One-Machine Simulation Requirement

The first practical safety gate is a one-machine multi-aggregator simulation.
It should be able to run either:

- as one process with multiple runtime tasks; or
- as multiple local processes with independent database paths.

The essential requirement is not process count. The requirement is evidence for:

- lawful same-lineage failover;
- split-brain rejection;
- carry-forward of unchanged shard leaves;
- retryable unavailability for a failed shard;
- continued progress for unaffected shards.

### Minimal Simulation Topology

A useful first topology is:

```text
Agg-1: primary [A, B], standby [C]
Agg-2: primary [C, D], standby [A]
Agg-3: primary [E, F], standby [B, D]
```

Wallet and test clients should talk only to an aggregator API surface. They
should not talk to storage or backend internals directly.

### Required Evidence Families

The evidence program should include:

- route-table golden vectors;
- batch-planner determinism tests;
- failover vectors for same-lineage resume and wrong-lineage rejection;
- split-brain fencing tests;
- backend conformance fixtures for storage and journal backends;
- benchmark reports with equal durability settings;
- historical verification coverage across route and root generations.

## Phased Rollout Strategy

This paper preserves the sequencing logic from the former TODO, but expresses
it as phases rather than a contractual calendar.

### 1. Local Baseline Hardening

- keep the current `RedB`-backed baseline;
- add `StorageBackend` and `JournalBackend` seams;
- add a memory backend for conformance and simulation;
- keep raw backend internals private.

### 2. Deterministic Shard Admission

- stabilize `ShardId` and committed route-table usage;
- keep planner ownership in the aggregator runtime layer;
- reject cross-shard batches in version 1;
- emit deterministic `BatchPlanned` records.

### 3. Shard Journal And Runtime Failover

- introduce per-shard journal discipline;
- add lawful standby resume under the same lineage;
- add wrong-lineage and split-brain rejection;
- prove recovery at the shard publication boundary.

### 4. Root-Of-Shard-Roots Publication

- publish shard-local roots through committed shard-leaf objects;
- carry forward unchanged shard leaves byte-for-byte;
- keep validator acceptance fail-closed;
- preserve historical verification across route and root generations.

### 5. Simulation And Evidence Gate

- run one-machine multi-aggregator simulation;
- collect crash, failover, and publication evidence;
- prove storage or journal backends conform under the same behavioral suite.

### 6. Optional Replicated Journal Upgrade

- add a replicated journal only after local lineage and recovery are stable;
- prefer `ShardGroupId` scope before per-tiny-shard consensus groups;
- use an existing substrate such as `OpenRaft` behind `JournalBackend`.
- if a strictly synchronous replicated journal with explicit fencing is used as
  an interim stage, treat it as a transitional implementation rather than the
  final architectural claim.

### 7. Optional Backend Migration

- keep `RedB` unless benchmarks show a real bottleneck;
- evaluate `Fjall` first as the likely lighter alternative;
- move to `RocksDB` only if heavier production pressure justifies it.

### Schedule Realism And Critical Path

The former TODO contained several time estimates that looked inconsistent at
first glance. They are best understood as estimates for different scopes rather
than as one contradictory schedule.

The preserved interpretation is:

- a strong local prototype or partial integrated slice can be much faster;
- a full evidence-backed upgrade is materially slower because readiness
  requires code plus vectors, benchmarks, crash recovery, and failover
  evidence.

The critical path is not the easiest code to write. It is the cross-cutting
chain where routing, journaling, public roots, and failover all interact:

```text
ShardRouteTable
  -> per-shard journal
  -> shard root leaf
  -> checkpoint publication
  -> recovery matrix
  -> failover vectors
  -> durable public-root benchmarks
```

That is why a full readiness path is best thought of as a multi-month effort,
while a narrower prototype path can be much shorter.

### Informative Calendar Heuristic

This paper preserves the following non-binding heuristic:

- aggressive prototype path: roughly 8 to 10 weeks for a narrow integrated
  slice with local evidence and limited scope;
- broader evidence-backed full upgrade: materially longer, typically multiple
  months, because proof, route, journal, publication, transition, and recovery
  evidence all have to converge.

These numbers are informative only. They explain sequencing pressure and
critical-path weight; they are not a delivery promise.

## Anti-Patterns To Avoid

Avoid the following implementation directions:

- making runtime placement a source of proof truth;
- allowing wallet code to talk directly to storage internals;
- allowing aggregator code to depend directly on `RedB` vendor types;
- splitting storage into many crates before seams and tests are stable;
- using a replicated log or gossip library as if it were already Z00Z
  protocol logic;
- promising "transparent failover" when the system actually requires committed
  route migration;
- treating illustrative API sketches as adopted public contracts.

## Final Position

The practical target is:

- RAID10-style shard placement;
- same-lineage failover only;
- one storage crate first, stronger boundaries first;
- `RedB` now, benchmark gate before migration;
- local journal first, replicated journal later;
- no custom consensus or membership stack;
- evidence before scale claims.

That direction is strong enough to support a correct production architecture
later without forcing a premature workspace rewrite now.

## Appendix A. Current Workspace To Future Architecture Map

| Current repository seam | Future architectural interpretation |
| --- | --- |
| `z00z_rollup_node` | Orchestration root that composes runtime services and adapters. |
| `z00z_runtime/aggregators` | Runtime-service owner of batch admission, shard execution, and publication handoff. |
| `z00z_runtime/validators` | Runtime-service owner of checkpoint acceptance and verdict flow. |
| `z00z_runtime/watchers` | Runtime-service owner of operational observation and evidence export. |
| `z00z_storage` | Current semantic truth facade plus durable execution and proof-related surfaces. |

## Appendix B. Illustrative Future Package Sketch

This sketch is informative only.

```text
z00z-types
  protocol and domain types

z00z-hjmt
  tree logic and proof machinery

z00z-settlement
  commit, publication, and recovery semantics

z00z-storage
  backend-neutral storage facade plus backend modules

z00z-aggregator
  planner, shard executors, placement, and failover

z00z-sim
  local multi-aggregator simulation
```

The sketch is useful as a long-term mental model, but current repository seams
remain authoritative until an explicit migration plan says otherwise.

## Appendix C. Minimal Signature Guidance

Illustrative signatures should preserve seam intent without freezing vendor
details:

```rust
pub trait StorageBackend {
  type ReadTxn;
  type WriteTxn;

  fn begin_read(&self) -> Result<Self::ReadTxn, StoreErr>;
  fn begin_write(&self) -> Result<Self::WriteTxn, StoreErr>;
}

pub trait JournalBackend {
  fn append(&mut self, entry: &[u8]) -> Result<(), StoreErr>;
}
```

The architectural meaning is more important than the exact syntax:

- keep public signatures backend-neutral;
- keep planner and execution signatures runtime-oriented;
- keep wallet-facing signatures limited to public proofs, queries, and
  submission APIs;
- promote a sketch into a contract only when tests, vectors, and naming review
  are ready.

## Appendix D. Current Repository Ownership Anchors

These anchors preserve the repository-backed ownership guidance that informed
the techpaper.

| Concern | Current anchor |
| --- | --- |
| Node orchestration root | `crates/z00z_rollup_node/src/lifecycle.rs` |
| Aggregator ingress and planner ownership | `crates/z00z_runtime/aggregators/src/agg_ingress.rs` |
| Aggregator scheduling and shard execution orchestration | `crates/z00z_runtime/aggregators/src/agg_scheduler.rs` |
| Aggregator recovery and failover behavior | `crates/z00z_runtime/aggregators/src/agg_recovery.rs` |
| Durable HJMT journal stages | `crates/z00z_storage/src/settlement/hjmt/hjmt_journal.rs` |
| Durable HJMT commit stages | `crates/z00z_storage/src/settlement/hjmt/hjmt_commit.rs` |
| Proof envelope and verifier contracts | `crates/z00z_storage/src/settlement/proof.rs` and `crates/z00z_storage/src/settlement/hjmt/hjmt_proof.rs` |
| Typed identities and store codecs | `crates/z00z_storage/src/settlement/types_identity.rs`, `crates/z00z_storage/src/settlement/store/store_types.rs`, and `crates/z00z_storage/src/settlement/store/store_codec.rs` |
| Public storage facade and wrappers | `crates/z00z_storage/src/lib.rs`, `crates/z00z_storage/src/settlement/mod.rs`, and `crates/z00z_storage/src/settlement/store.rs` |
| Raw backend internals | `crates/z00z_storage/src/settlement/redb_backend/` |
| Validator checkpoint acceptance | `crates/z00z_runtime/validators/src/checkpoint_flow.rs` and `crates/z00z_runtime/validators/src/verdicts.rs` |
| Operational watcher surfaces | `crates/z00z_runtime/watchers/src/publication_watch.rs`, `crates/z00z_runtime/watchers/src/status_view.rs`, and `crates/z00z_runtime/watchers/src/evidence_export.rs` |
