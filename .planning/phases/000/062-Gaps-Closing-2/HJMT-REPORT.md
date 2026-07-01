# HJMT Runtime, Journal, Planner, And Gap Closure

Date: 2026-06-23

> ja hochu proanalizirovat chto sdelanov 060-Gaps-Closing
> - chto proishodi v konfiguracii 3A7S -> 2A7S--> 5A7S; 
>
> - kak rabotaet journal_sync
>
> - kak voobwe rabotaet journal, chto zache, dlja chego tuda pishetsja i kak ispolzuetsja
>
> - HJMT rasperedelen, chto proishodit posle dogo kak aggregator obnovil derevo svoih shards, drugie aggregaori kotorie soderzhat te zhe shards kak oni vse sunchronizirujutsja; kak synchroniziruetsja i kakverifcieutsja novoe root-state? mne ne ponjaten mehanizm etog processa v detaljah
>
> - chto proishodit kogda est rassinhrom mezhdu aggregatorami na teh zhe shardh? skazhem oni vdrug poluchjili raznie roots
>
> - implementirovan li mechanizm raid-like arhitekturi i kak on rabotaet na praktike po kodovoj baze
>
> - kak garantirovat stabilnost i pravelnost raboti HJMT na raznig aggregatorah i raznih shardah
>
> - kak garantirovat rabotu planer? kak planer reshet chto v kakie sharti pisat i kak eto dohodit do raspredelennij setevih aggregatorov; ne vazhno chto poka net seti , no est raznie proess -kazhdij aggregator na svoem process bezhit nezavisimo i avtonomno
>
> - daj polnuju kartunu processov  s podrobni  objasneniem, primerami
>
> - poo hodu smotri i proveraj slabie mesta, gde mozhet legko porvatsja ili narushitsja celostnost; 
> 
>
>   
>**главный bottleneck не в proof verify, а в durable state path**: `journal_sync`, `apply_ops`, checkpoint/publication и Stage 13 artifact work.
> Если нужен следующий шаг, я бы бил  по `scheduler/commit/publication` path: там сейчас основной потолок.





Scope:

- Phase artifact scope: `.planning/phases/060-Gaps-Closing/`
- Runtime config scope: `config/hjmt_runtime/sim_5a7s/`
- Code scope: `crates/z00z_runtime/aggregators`, `crates/z00z_rollup_node`, `crates/z00z_storage`, `crates/z00z_runtime/validators`, `crates/z00z_runtime/watchers`

Method:

- This report uses repository docs, planning files, tests, and source code only.
- Graph material was not used as a concrete source for any claim.
- Claims were doublechecked against direct workspace code references and targeted tests.

## Executive Summary

Phase 060 has real HJMT closure work, but it is important to separate what is implemented from what is only constrained by contracts.

Implemented:

- Canonical `SIM-5A7S` runtime home with five OS-process aggregators and seven shards.
- Explicit `aggregator_owned` default plus opt-in `shard_process` validation.
- Deterministic route-table validation, canonical route digesting, generation-bound migration, and activation checkpoints.
- Tested `3A7S -> 2A7S -> 5A7S` fail-down/re-expand path. in code this middle stage is `staged_two_by_seven` and the test calls it fail-down.
- Local HJMT durable journal with staged statuses: `Prepared -> ChildrenCommitted -> ParentsCommitted -> RootPublished`.
- `hjmt_journal_sync` as the local persistence boundary that writes journal rows, HJMT rows, metadata, and published root into the Redb backend.
- Recovery fencing that rejects wrong generation, wrong lineage, wrong shard, wrong route digest, stale root, stale restart, standby-down, and split-brain takeover cases.
- Publication contracts that bind route-table digest, routing generation, exact shard set, activation checkpoint, shard root leaves, public root, and validator/watcher runtime route checks.

Not implemented as a distributed protocol:

- No live journal replication between aggregator processes was found.
- No network/gossip/quorum mechanism was found for same-shard aggregators to agree on a root.
- No automatic standby state catch-up was found.
- No atomic live route-table rollout protocol was found.
- `journal_sync` is not inter-aggregator synchronization; it is local durable backend synchronization.

The current architecture is best described as deterministic placement plus local crash-safe HJMT storage plus fail-closed recovery/publication contracts. It is not yet a full distributed HJMT replication or consensus layer.

## What 060 Actually Closed

The HJMT-relevant Phase 060 work is concentrated in four scenario families from the planning artifacts:

| Scenario | What it targets | Evidence |
| --- | --- | --- |
| `060-S03` | Process model, `aggregator_owned`, opt-in `shard_process` | `.planning/phases/060-Gaps-Closing/060-TEST-SPEC.md:147`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `crates/z00z_rollup_node/tests/test_hjmt_process.rs` |
| `060-S05` | Decommission, same-lineage failover, `3A7S -> 2A7S -> 5A7S` | `.planning/phases/060-Gaps-Closing/060-TESTS-TASKS.md:33`, `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:140` |
| `060-S12` | Durable route-bound recovery and exact publication coverage | `.planning/phases/060-Gaps-Closing/060-TEST-SPEC.md:210`, `crates/z00z_storage/src/settlement/proof_batch_verify.rs:90` |
| `060-S14` / `060-15` | Incomplete validator/watcher runtime states on the current publication path | `.planning/phases/060-Gaps-Closing/060-TESTS-TASKS.md:42`, `crates/z00z_runtime/validators/src/engine.rs:22`, `crates/z00z_runtime/watchers/src/engine.rs:54` |

The report below focuses on HJMT runtime, journal, planner, publication, and weak points, not the unrelated wallet/docs/supply-chain Phase 060 slices.

## SIM-5A7S Configuration

The checked-in runtime home is `config/hjmt_runtime/sim_5a7s/`.

The manifest declares:

- profile `SIM-5A7S`;
- `process_model: os_process`;
- `shard_mapping: aggregator_owned`;
- route-table digest `000c78634c31e624c5e194378e6c7613e916e1975ca901e5d6416325c1d617e1`;
- five aggregators: `0..4`;
- seven shards: `0..6`;
- primary placement where aggregator `0` owns shards `0,5`, aggregator `4` owns shards `4,6`, and the others own one shard each.

Evidence:

- `config/hjmt_runtime/sim_5a7s/manifest.json:3`
- `config/hjmt_runtime/sim_5a7s/manifest.json:5`
- `config/hjmt_runtime/sim_5a7s/manifest.json:6`
- `config/hjmt_runtime/sim_5a7s/manifest.json:7`
- `config/hjmt_runtime/sim_5a7s/manifest.json:11`
- `config/hjmt_runtime/sim_5a7s/manifest.json:18`
- `config/hjmt_runtime/sim_5a7s/manifest.json:27`

The planner config is central, generation `1`, shard-local only, cross-shard rejecting, with batch limits:

- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml:1`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml:2`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml:7`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml:8`
- `config/hjmt_runtime/sim_5a7s/planner/planner-config.yaml:11`

The storage config is HJMT generation `1`, full sync, flush per batch, and has a declared `lock_path` plus positive timeout:

- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml:1`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml:2`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml:8`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml:10`
- `config/hjmt_runtime/sim_5a7s/storage/storage-config.yaml:14`

Each aggregator config points at the shared route table and declares owned shards, expected journal lineage, lifecycle commands, startup checks, and process-local paths. For example, aggregator `0` owns shards `0` and `5` under `aggregator_owned`:

- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml:1`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml:5`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml:7`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml:10`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml:22`
- `config/hjmt_runtime/sim_5a7s/aggregators/agg-0/aggregator-config.yaml:25`

## `aggregator_owned` And `shard_process`

The code supports two execution mappings:

- `aggregator_owned`: one aggregator process may own multiple primary shards.
- `shard_process`: at most one primary shard per process.

The enum is defined in `crates/z00z_rollup_node/src/config.rs:54`. The validator enforces:

- all aggregators in one HJMT home share one `execution.shard_mapping`;
- planner routing generation equals aggregator routing generation;
- `aggregator_owned` requires all shards inside one process to share one expected journal lineage because `journal_path` is process-scoped;
- `shard_process` rejects a process with more than one primary shard.

Evidence:

- `crates/z00z_rollup_node/src/config.rs:904`
- `crates/z00z_rollup_node/src/config.rs:906`
- `crates/z00z_rollup_node/src/config.rs:917`
- `crates/z00z_rollup_node/src/config.rs:925`
- `crates/z00z_rollup_node/src/config.rs:930`
- `crates/z00z_rollup_node/src/config.rs:991`
- `crates/z00z_rollup_node/src/config.rs:997`

This means the current production-like default is not "one shard equals one process". It is "one aggregator process may own multiple shards". The one-shard-per-process model exists as an opt-in config mode and must be benchmarked separately before becoming a default.

## `3A7S -> 2A7S -> 5A7S`

The code fixture names are:

- `staged_three_by_seven`;
- `staged_two_by_seven`;
- `staged_five_by_seven`.

Evidence:

- `crates/z00z_runtime/aggregators/tests/test_hjmt_topology_support.rs:66`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_topology_support.rs:74`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_topology_support.rs:81`

The user phrase `2A7F` appears to mean the fail-down/failover middle stage. The repository implements that as `2A7S`:

- old stage: three aggregators, seven shards;
- middle stage: two aggregators, seven shards;
- final stage: canonical five aggregators, seven shards.

The exact staging:

| Stage | Aggregators | Shards | Notes |
| --- | ---: | ---: | --- |
| `3A7S` | `0,1,5` | `0..6` | Aggregator `5` owns shards `5,6` before decommission |
| `2A7S` | `0,1` | `0..6` | Aggregator `5` removed; all shards still owned |
| `5A7S` | `0..4` | `0..6` | Canonical final SIM-style layout |

The migration test binds route generations and previous-generation digests:

- old home generation `1`;
- middle home generation `2`;
- final home generation `3`;
- activation checkpoints `11`, `42`, `101`;
- `mid.validate_migration(old)` and `new.validate_migration(mid)` must pass.

Evidence:

- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:140`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:146`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:147`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:148`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:149`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:150`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:152`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:162`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:165`

The important guarantee here is not that state is streamed live between processes. The guarantee is that each topology step is generation-bound, digest-bound, activation-checkpoint-bound, and stale owner references are removed.

## Route Table And Planner

### Route Table

`ShardRouteTable` contains:

- `routing_generation`;
- sorted `shard_set`;
- range rules;
- optional `previous_generation_digest`;
- `activation_checkpoint`.

Evidence:

- `crates/z00z_runtime/aggregators/src/batch_planner.rs:80`

Validation enforces:

- non-empty shard set and rules;
- sorted unique shard ids;
- gap-free route coverage from `HASH_MIN` to `HASH_MAX`;
- no foreign shards;
- every declared shard is used;
- generation `0` has no previous digest, while later generations must have one.

Evidence:

- `crates/z00z_runtime/aggregators/src/batch_planner.rs:101`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:121`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:131`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:153`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:156`

Migration validation requires:

- previous table validates;
- new table validates;
- routing generation increases;
- `previous_generation_digest == prev.digest()`;
- activation checkpoint does not roll back.

Evidence:

- `crates/z00z_runtime/aggregators/src/batch_planner.rs:233`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:236`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:239`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:242`

### Work Item Routing

`WorkItem` stores both payload digest and admission digest. Object packages can rebind admission digest and intake id, but route key remains the payload digest.

Evidence:

- `crates/z00z_runtime/aggregators/src/types.rs:98`
- `crates/z00z_runtime/aggregators/src/types.rs:121`
- `crates/z00z_runtime/aggregators/src/types.rs:140`

This matters because route assignment is stable under object-package admission metadata changes.

### Batch Planning

`BatchPlanner::plan_batch` canonicalizes entries, looks up each route key, sorts entries deterministically, and rejects any batch spanning multiple shards.

Evidence:

- `crates/z00z_runtime/aggregators/src/batch_planner.rs:286`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:312`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:324`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:336`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:343`
- `crates/z00z_runtime/aggregators/src/batch_planner.rs:348`

The resulting planned batch contains:

- `batch_id`;
- `BatchRoute { shard_id, routing_generation }`;
- `route_table_digest`;
- intake ids;
- op count;
- `plan_digest`.

Evidence:

- `crates/z00z_runtime/aggregators/src/types.rs:210`

### Central Planner Versus Per-Aggregator Planner

The tests show central and per-aggregator planner modes are deterministic-equivalent on accepted and rejected profiles. That is a local deterministic equivalence proof, not a network distribution protocol.

Evidence:

- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs:86`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs:89`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs:116`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_planner.rs:128`

### Scheduler Limitation

`SchedulerBoundary::plan_waves` currently returns one wave containing all work items:

- `crates/z00z_runtime/aggregators/src/scheduler.rs:9`

So there is no real multi-wave or load-balancing scheduler in the analyzed code path.

## Planner To Aggregator Execution

After planning:

1. `ShardExecutor::route` checks that the placement table owns the planned shard route.
2. It emits a `ShardExecTicket` with state `Routed`.
3. Runtime state may move to `Running`, `RetryPending`, `RecoveryPending`, or `Completed`.
4. `OrderedBatch::exec_handoff` binds storage execution to `batch_id`, `shard_id`, `routing_generation`, and `route_table_digest`.

Evidence:

- `crates/z00z_runtime/aggregators/src/shard_exec.rs:51`
- `crates/z00z_runtime/aggregators/src/shard_exec.rs:61`
- `crates/z00z_runtime/aggregators/src/shard_exec.rs:9`
- `crates/z00z_runtime/aggregators/src/types.rs:233`
- `crates/z00z_runtime/aggregators/src/types.rs:240`

In the current repository, independent OS processes are represented by config homes, lifecycle commands, process ids, and validation. The code analyzed here does not contain a live network dispatch path that sends a planned batch to a remote aggregator over `listen_addr`.

## Journal Model

The HJMT journal is a local write-ahead / recovery record for the storage backend. It is also the source of the recovery lineage digest.

`HjmtCommitJournalEntry` records:

- version and bucket epoch;
- bucket policy id;
- root generation and proof version;
- previous and next semantic state roots;
- touched definitions, serials, and buckets;
- fee replay count/digest/digests;
- child and parent commit digests;
- commit status;
- optional route context.

Evidence:

- `crates/z00z_storage/src/settlement/hjmt_journal.rs:59`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:60`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:63`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:65`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:67`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:70`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:73`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:75`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:76`

The journal status progression is:

1. `Prepared`
2. `ChildrenCommitted`
3. `ParentsCommitted`
4. `RootPublished`

Evidence:

- `crates/z00z_storage/src/settlement/hjmt_journal.rs:28`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:36`

The journal digest is a domain-separated hash of canonical journal bytes:

- `crates/z00z_storage/src/settlement/hjmt_journal.rs:213`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:255`

If route context exists, the encoded journal includes:

- batch id;
- shard id;
- routing generation;
- route-table digest.

Evidence:

- `crates/z00z_storage/src/settlement/hjmt_journal.rs:246`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:247`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:248`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:249`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:250`

The active state must match a root-published journal:

- `crates/z00z_storage/src/settlement/hjmt_journal.rs:137`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:162`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs:167`

## `journal_sync`

`journal_sync` is the timed, blocking storage persistence step inside `commit_hjmt_plan_at`.

Flow inside `commit_hjmt_plan_at`:

1. Capture previous HJMT root.
2. Build write artifacts if persistence is enabled.
3. Snapshot in-memory store/cache/roots/state for rollback.
4. Commit terminal child batches.
5. Commit bucket, serial, and definition parent batches.
6. Commit path index if needed.
7. Update in-memory roots and semantic model.
8. Compute the live journal digest.
9. Build `HjmtPersistWork`.
10. Run `timing::run("hjmt_journal_sync", ...)`.
11. Call `backend.sync_hjmt_work(persist_work)`.

Evidence:

- `crates/z00z_storage/src/settlement/hjmt_commit.rs:553`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:563`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:571`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:580`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:583`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:602`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:611`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:620`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:629`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs:631`

Backend `sync_hjmt_work` recomputes child and parent digests, builds the journal entry, attaches route context if present, seals fee replay state, then writes stages:

1. `write_journal(entry Prepared)`;
2. `write_children(... ChildrenCommitted)`;
3. `write_parents(... ParentsCommitted)`;
4. `publish_root_work(... RootPublished)`.

Evidence:

- `crates/z00z_storage/src/backend/redb/hjmt.rs:76`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:86`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:89`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:91`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:104`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:123`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:136`

Local stale writers are guarded by active-version head checks:

- `crates/z00z_storage/src/backend/redb/mod.rs:107`
- `crates/z00z_storage/src/backend/redb/mod.rs:130`
- `crates/z00z_storage/src/backend/redb/mod.rs:138`

Crash recovery behavior:

- `Prepared`: validate empty pending state and roll back version.
- `ChildrenCommitted`: validate child stage and roll back version.
- `ParentsCommitted`: validate parent stage and publish pending root.
- `RootPublished` without active metadata is an error.

Evidence:

- `crates/z00z_storage/src/backend/redb/hjmt.rs:449`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:473`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:474`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:478`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:482`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:487`

Loaded HJMT state is revalidated against the journal:

- `crates/z00z_storage/src/backend/redb/hjmt.rs:498`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:513`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:521`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:626`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:648`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:653`
- `crates/z00z_storage/src/backend/redb/hjmt.rs:664`

Key point: `journal_sync` is local durable synchronization with Redb. It is not the mechanism that synchronizes multiple aggregators.

## Why The Journal Exists

The journal serves five separate purposes:

1. Crash safety: it records which persistence stage was reached.
2. Integrity: it binds child rows, parent rows, fee replay rows, and roots through digests.
3. Historical loading: state can only load when the journal is `RootPublished`.
4. Recovery lineage: `SettlementRecoveryState` exports `journal_lineage`, derived from the active journal digest.
5. Route-bound recovery: durable state can carry shard id, routing generation, route-table digest, and batch id through the journal route context.

`SettlementRecoveryState` includes version, state root, root/proof generation, bucket policy metadata, journal lineage, and optional route:

- `crates/z00z_storage/src/settlement/store.rs:243`
- `crates/z00z_storage/src/settlement/store.rs:250`

Route context is carried by `SettlementRouteCtx`:

- `crates/z00z_storage/src/settlement/store.rs:78`
- `crates/z00z_storage/src/settlement/store.rs:117`

The result is that a restart or standby takeover can compare current durable state against the recovery record and live placement.

## Root-State Publication

There are two different root concepts in play:

- shard-local HJMT root/state root, stored and journaled by each HJMT backend;
- public checkpoint root, built from shard root leaves under a route snapshot.

`PublicationRouteSnapshotV1` binds:

- routing generation;
- route-table digest;
- activation checkpoint;
- exact sorted shard ids.

Evidence:

- `crates/z00z_storage/src/settlement/proof_batch.rs:501`
- `crates/z00z_storage/src/settlement/proof_batch.rs:524`

`ShardRootLeafV1` binds:

- shard id;
- shard root;
- shard epoch;
- routing generation;
- route-table digest;
- policy digest;
- journal checkpoint;
- local sequence;
- transition flags.

Evidence:

- `crates/z00z_storage/src/settlement/proof_batch.rs:576`

`CheckpointPublicationV1` binds:

- root generation;
- publication mode;
- publication checkpoint;
- route-table digest;
- prior public root;
- ordered shard leaves.

Evidence:

- `crates/z00z_storage/src/settlement/proof_batch.rs:691`
- `crates/z00z_storage/src/settlement/proof_batch.rs:720`
- `crates/z00z_storage/src/settlement/proof_batch.rs:804`
- `crates/z00z_storage/src/settlement/proof_batch.rs:808`
- `crates/z00z_storage/src/settlement/proof_batch.rs:819`

The shared route checker enforces exact coverage and route binding:

- publication leaf count must equal route shard count;
- each leaf route-table digest must match;
- publication checkpoint must not precede activation checkpoint;
- leaf shard/generation must match the route snapshot;
- handoff rows must cover exactly the route shard set.

Evidence:

- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:90`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:96`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:100`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:121`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:126`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:156`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:163`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:170`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:173`
- `crates/z00z_storage/src/settlement/proof_batch_verify.rs:176`

Validator and watcher code reuse route binding:

- `crates/z00z_runtime/validators/src/checkpoint.rs:49`
- `crates/z00z_runtime/watchers/src/publication.rs:73`

This is the strongest current protection for public root-state verification.

## What Happens After An Aggregator Updates Its Shard Tree

Current code path:

1. Planner maps work to a shard using the route table.
2. `ShardExecutor` confirms local placement owns that shard/generation.
3. `OrderedBatch::exec_handoff` creates `SettlementExecHandoff` with route context.
4. Storage applies the HJMT plan and locally publishes a root through `journal_sync`.
5. The durable journal records route context and root transition.
6. Publication assembles shard root leaves under a route snapshot.
7. Validators and watchers check route binding, checkpoint binding, and runtime placement.

What does not happen in the analyzed code:

- The updated journal is not streamed to all standby aggregators.
- Other aggregators that list the shard as standby do not automatically pull the root.
- Same-shard aggregators do not run a quorum agreement to choose a root.
- The code does not resolve two competing published roots through an HJMT-native consensus layer.

The implemented model is "publish and verify" plus "same-lineage recovery fencing", not "replicate and agree".

## Same-Shard Aggregator Synchronization

Placement rows carry:

- route;
- primary aggregator id;
- standby list;
- expected journal lineage.

Evidence:

- `crates/z00z_runtime/aggregators/src/placement.rs:50`

The placement table returns placement only if both shard id and routing generation match:

- `crates/z00z_runtime/aggregators/src/placement.rs:141`

Recovery then compares:

- live placement generation;
- live primary owner;
- live expected journal lineage;
- current durable journal lineage;
- recorded recovery journal lineage;
- route shard id and routing generation;
- route-table digest;
- recovery batch id;
- recovery version;
- state root;
- root/proof/bucket metadata;
- requester role and standby readiness.

Evidence:

- `crates/z00z_runtime/aggregators/src/recovery.rs:80`
- `crates/z00z_runtime/aggregators/src/recovery.rs:87`
- `crates/z00z_runtime/aggregators/src/recovery.rs:94`
- `crates/z00z_runtime/aggregators/src/recovery.rs:120`
- `crates/z00z_runtime/aggregators/src/recovery.rs:131`
- `crates/z00z_runtime/aggregators/src/recovery.rs:138`
- `crates/z00z_runtime/aggregators/src/recovery.rs:148`
- `crates/z00z_runtime/aggregators/src/recovery.rs:155`
- `crates/z00z_runtime/aggregators/src/recovery.rs:162`
- `crates/z00z_runtime/aggregators/src/recovery.rs:173`

This is a fail-closed admission check for recovery. It does not itself copy missing state into the standby.

## Root Mismatch And Split-Brain Behavior

If aggregators disagree on the same shard root, the current code has several rejection layers.

At recovery:

- wrong generation rejects;
- live primary drift rejects;
- wrong lineage rejects;
- missing/wrong shard route rejects;
- route digest drift rejects;
- stale recovery version rejects;
- stale local root rejects;
- backend generation metadata drift rejects;
- primary restart by non-primary rejects;
- standby takeover by primary rejects;
- takeover by non-standby rejects;
- takeover by not-ready standby defers.

Evidence:

- `crates/z00z_runtime/aggregators/src/recovery.rs:80`
- `crates/z00z_runtime/aggregators/src/recovery.rs:87`
- `crates/z00z_runtime/aggregators/src/recovery.rs:94`
- `crates/z00z_runtime/aggregators/src/recovery.rs:120`
- `crates/z00z_runtime/aggregators/src/recovery.rs:131`
- `crates/z00z_runtime/aggregators/src/recovery.rs:148`
- `crates/z00z_runtime/aggregators/src/recovery.rs:155`
- `crates/z00z_runtime/aggregators/src/recovery.rs:162`
- `crates/z00z_runtime/aggregators/src/recovery.rs:173`
- `crates/z00z_runtime/aggregators/src/recovery.rs:183`
- `crates/z00z_runtime/aggregators/src/recovery.rs:197`

At validator:

- publication/checkpoint/batch drift rejects;
- route binding drift rejects;
- retry/recovery states or missing artifact become `Incomplete`, not accepted.

Evidence:

- `crates/z00z_runtime/validators/src/checkpoint.rs:21`
- `crates/z00z_runtime/validators/src/checkpoint.rs:33`
- `crates/z00z_runtime/validators/src/checkpoint.rs:43`
- `crates/z00z_runtime/validators/src/checkpoint.rs:49`
- `crates/z00z_runtime/validators/src/engine.rs:22`
- `crates/z00z_runtime/validators/src/engine.rs:85`

At watcher:

- missing verdict/binding is `ValidatorIncomplete`;
- hard mismatches become `InvalidBatch`;
- route mismatch rejects.

Evidence:

- `crates/z00z_runtime/watchers/src/publication.rs:45`
- `crates/z00z_runtime/watchers/src/publication.rs:62`
- `crates/z00z_runtime/watchers/src/publication.rs:73`
- `crates/z00z_runtime/watchers/src/engine.rs:72`
- `crates/z00z_runtime/watchers/src/engine.rs:77`

Important limitation: if two independent aggregators each manage to produce different locally valid roots, this code does not implement a root-choice consensus protocol. It provides local durability, route-bound publication checks, and fail-closed validation. Final conflict resolution would need an external ordering/DA/consensus layer or a new HJMT replication/quorum protocol.

## RAID-Like Architecture

The implementation has RAID-like ideas, but it is not a full RAID implementation.

RAID-like parts that exist:

- Shards are distributed across primary aggregators.
- Each shard has standby aggregators.
- Recovery can activate a standby when same-lineage checks pass.
- Decommission/fail-down/re-expand tests preserve all seven shards and remove stale owners/standbys.

Evidence:

- `config/hjmt_runtime/sim_5a7s/manifest.json:27`
- `crates/z00z_runtime/aggregators/src/placement.rs:50`
- `crates/z00z_runtime/aggregators/src/recovery.rs:183`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:83`
- `crates/z00z_runtime/aggregators/tests/test_hjmt_migrate.rs:140`

RAID-like parts not found:

- no parity reconstruction;
- no active mirroring of journal rows;
- no background resync;
- no quorum write;
- no read repair;
- no automatic catch-up for a standby that missed a journal entry.

Practical interpretation: this is closer to sharded primary ownership plus standby failover metadata than to RAID-1/RAID-5 data replication.

## How To Guarantee HJMT Stability Today

Current guarantees come from layered checks, not from one single mechanism.

Layer 1: deterministic planning

- route table validates and digests canonically;
- migration is generation/digest/checkpoint-bound;
- planner route key is stable;
- multi-shard batches reject.

Layer 2: process config and startup preflight

- all aggregators share one route generation;
- SIM-5A7S has exact aggregator and shard ids;
- mixed mapping rejects;
- duplicate primaries reject;
- wrong lineage rejects;
- wrong route digest rejects;
- handoff must cover the route table.

Evidence:

- `crates/z00z_rollup_node/src/config.rs:420`
- `crates/z00z_rollup_node/src/config.rs:451`
- `crates/z00z_rollup_node/src/config.rs:486`
- `crates/z00z_rollup_node/src/config.rs:1023`
- `crates/z00z_rollup_node/src/config.rs:1170`
- `crates/z00z_rollup_node/src/config.rs:1247`

Layer 3: local journaled persistence

- state is staged through a journal;
- local durable head cannot skip versions;
- loaded rows are validated against journal digests.

Layer 4: same-lineage recovery

- standby takeover is lawful only under matching lineage, route, root, and metadata.

Layer 5: publication verification

- public root is canonical over route-bound shard leaves;
- validators and watchers reject route/publication drift.

To guarantee stability beyond a single machine/process set, the missing next layer is a real distributed synchronization protocol:

- state transfer or journal replication;
- standby catch-up proof;
- quorum or authoritative ordering for same-shard roots;
- atomic route-table rollout;
- root conflict resolution rules.

## Planner Guarantees And Gaps

The planner can guarantee:

- deterministic route assignment from payload digest;
- stable object-package admission without route drift;
- canonical ordering of items;
- single-shard batch admission only;
- route generation and route-table digest binding into plan output;
- central and per-aggregator planner equivalence in tests.

The planner cannot yet guarantee:

- network delivery to the right remote aggregator;
- multi-shard execution;
- balanced scheduling;
- independent process state catch-up;
- atomic propagation of route-table upgrades.

The current delivery story is config-level: every process loads the same route table and config home; startup/preflight rejects drift. That is useful, but it is not enough for a real distributed runtime where each aggregator runs independently and receives live work over the network.

## Weak Points

| Area | Weak point | Why it matters | Recommended closure |
| --- | --- | --- | --- |
| Distributed sync | No live journal/root replication found between aggregators | A standby can be lawful in metadata but still not have the latest state unless state was externally provisioned | Add journal replication or state-transfer protocol with catch-up proof |
| Root consensus | No quorum or root-choice protocol found for competing same-shard roots | Two locally valid roots need deterministic conflict resolution | Define ordering/DA authority or HJMT quorum rules |
| Standby readiness | `StandbyState::ready` is metadata, not proof of synced durable state | Ready standby can be stale unless recovery state catches it | Bind readiness to latest journal lineage/version proof |
| Route rollout | Route migration is validated by files/tests, not atomic live process rollout | Independent OS processes can run different route tables during deployment | Add route-table rollout protocol with activation checkpoint and process acknowledgements |
| Scheduler | `SchedulerBoundary` returns one wave | No load-aware dispatch or parallel wave planning | Implement shard-aware scheduler only after measurement lanes are clean |
| Cross-shard work | Planner rejects multi-shard batches | Correct for current safety, but limits workflows | Keep rejecting until atomic cross-shard commit protocol exists |
| Storage lock | Config validates `lock_path`, but no storage-side file-lock enforcement was found in searched code | Two processes sharing one Redb root could rely only on backend behavior and head checks | Implement and test explicit process lock if shared storage roots are possible |
| `aggregator_owned` lineage | Multi-shard aggregator process requires one lineage across owned shards | Safe for process-scoped journal, but limits per-shard independent recovery | Use `shard_process` for strict per-shard isolation, gated by A/B evidence |
| Publication checker usage | Exact route coverage is strong only when code uses the shared checker | Future bypasses could accept weaker publication shapes | Keep guardrail tests around storage, validator, watcher, and preflight shared path |
| Performance bottleneck | Durable path includes `hjmt_journal_sync` | Throughput claims based only on worker-local speed are misleading | Keep `durable_root_published_tps`, `hjmt_journal_sync`, latency, blocked time, RSS, CPU in A/B evidence |

## Practical Process Examples

### Example 1: Normal Single-Shard Batch

1. Ingress creates a `WorkItem` with payload digest.
2. Planner uses payload digest as route key.
3. Route table maps the route key to shard `S` under generation `G`.
4. Planner rejects if other items route to a different shard.
5. `BatchPlanned` carries shard `S`, generation `G`, and route-table digest `D`.
6. `ShardExecutor` accepts only if local placement owns `(S,G)`.
7. `OrderedBatch::exec_handoff` gives storage route context `(batch_id,S,G,D)`.
8. Storage applies HJMT plan.
9. `journal_sync` persists journal and HJMT rows.
10. Publication builds shard leaf and public root.
11. Validator/watcher verify route binding.

### Example 2: Aggregator Restart

1. Process loads config and route table.
2. Storage exports `SettlementRecoveryState`.
3. Startup preflight compares expected lineage and route identity.
4. `RecoveryBoundary::resume` checks live placement, lineage, route, version, root, and metadata.
5. Restart succeeds only if requester is still the live primary owner.

### Example 3: Standby Takeover

1. Primary is no longer used.
2. Standby requests takeover.
3. Placement table must list requester as a ready standby.
4. Current durable state must match committed recovery record.
5. Journal lineage must match placement expected lineage.
6. Route-table digest and routing generation must match.
7. If all pass, recovery ticket activates standby as primary.

### Example 4: Split-Brain Root Drift

1. Aggregator A and B produce different roots for the same shard.
2. Recovery path rejects if one root does not match the committed recovery record.
3. Validator/watcher reject publication drift if route, checkpoint, public input, or binding does not match.
4. The code does not elect a winner between two independently valid roots. That requires a consensus/order layer not implemented here.

## Verification Run

Passing commands:

```text
cargo test -p z00z_aggregators --release --features test-params-fast -- --nocapture
```

Result: passed. Covered planner, route codec, migration, join, failover, split-brain fencing, publication handoff, and live guardrails.

```text
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_topology -- --nocapture
```

Result: passed. Covered canonical SIM-5A7S, opt-in `shard_process`, mixed mapping rejection, multi-primary rejection, lineage rejection.

```text
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_process -- --nocapture
```

Result: passed. Covered explicit paths per aggregator, mapping defaults, lifecycle command config references.

```text
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_batch_recovery -- --nocapture
```

Result: passed. Covered child/parent staged recovery.

```text
cargo test -p z00z_storage --release --features test-params-fast --test test_hjmt_root_generation publication_route -- --nocapture
```

Result: passed. Covered publication route accept, gap reject, and stale activation reject.

```text
cargo test -p z00z_validators --release --test test_hjmt_publication_contract -- --nocapture
```

Result: passed. Covered publication drift, route drift, digest drift, stale route activation, retry incomplete, and blob gap incomplete.

Note: `z00z_validators` does not expose the `test-params-fast` feature, so the validator test was run without that feature.

```text
cargo test -p z00z_watchers --release --test test_hjmt_publication_contract -- --nocapture
```

Result: passed. Covered missing verdict/binding incomplete, route digest/generation drift, checkpoint drift, exec drift, retry incomplete, and gap incomplete.

```text
cargo test -p z00z_rollup_node --release --features test-params-fast --test test_hjmt_preflight -- --nocapture
```

Result: passed. Covered canonical preflight, wrong lineage, wrong root/proof generation, foreign shard, missing route, wrong route digest, route digest drift, unknown standby, missing/unordered handoff, and startup block checks.

## Bottom Line

The current HJMT implementation is strong on deterministic configuration, local journaled durability, route-bound recovery, and publication verification. The weak point is not local correctness. The weak point is distributed operation: multiple autonomous aggregator processes do not yet have an implemented journal replication, standby catch-up, quorum, or atomic route rollout mechanism.

For practical correctness today, the system relies on:

- same route table everywhere;
- startup preflight fail-close;
- local Redb journal recovery;
- same-lineage failover;
- route-bound public root verification.

For a real distributed runtime, the next required design work is:

1. journal/state replication between primary and standby;
2. standby synced-state proof before takeover;
3. root conflict resolution or quorum;
4. atomic route-table activation across OS processes;
5. shard-aware scheduler and delivery path.
