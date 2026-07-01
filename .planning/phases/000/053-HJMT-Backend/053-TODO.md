---
phase: 053-HJMT-Backend
title: Production HJMT Generalized Settlement Backend
status: complete
created: 2026-05-29
source_docs:
  - docs/tech-papers/Z00Z-HJMT-Design.md
  - crates/z00z_core/src/assets/assets_config.yaml
  - crates/z00z_core/src/assets/assets_config_schema.yaml
  - crates/z00z_core/src/genesis/genesis_config_devnet.yaml
  - crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml
  - crates/z00z_core/src/genesis/genesis_config_mainnet.yaml
  - crates/z00z_core/src/genesis/genesis_config_schema.yaml
  - crates/z00z_core/src/genesis/genesis_config_testnet.yaml
  - crates/z00z_simulator/src/scenario_1/scenario_config.yaml
  - .planning/phases/000/051-HJMT-Facade/051-TODO.md
  - .planning/phases/052-HJMT-Backend/052-TODO.md
  - .planning/phases/052-HJMT-Backend/052-08-PLAN.md
  - .planning/phases/052-HJMT-Backend/052-09-PLAN.md
  - .planning/phases/052-HJMT-Backend/052-10-PLAN.md
  - .planning/phases/052-HJMT-Backend/052-11-PLAN.md
  - .planning/STATE.md
---

<!-- markdownlint-disable MD032 MD033 MD060 -->

# 053 TODO: Production HJMT Generalized Settlement Backend

## 🎯 Mission

Phase 053 promotes the former Phase 052 follow-up packet into production
scope. The words in `docs/tech-papers/Z00Z-HJMT-Design.md` that previously described target
or future HJMT features are now mandatory live-code requirements for this
phase.

This phase must deliver a production HJMT settlement backend, not scaffolding:

- Phase 053 is a dev hard cutover. There is no legacy storage runtime, no
  old-storage conversion lane, no aliases, no shims, and no compatibility
  reader as a success criterion.
- `SettlementStateRoot` becomes the canonical generalized settlement root for
  the new generation.
- `RightLeaf` becomes a live terminal settlement object for bounded non-coin
  rights.
- `FeeEnvelope` becomes a live, separate processing-support contract.
- deletion and non-existence proof families become verifier-validating proof
  families.
- adaptive bucket epochs, split proofs, merge proofs, and policy-transition proofs
  become live protocol machinery.
- cache-aware and async/parallel forest execution becomes a production
  performance requirement, not a benchmark-only story.

The implementation may use deterministic fixtures in tests, but production code
must be real, typed, durable, fail-closed, and wired through the existing
storage, checkpoint, wallet, validator, and simulator boundaries.

## 📌 Source Reading Notes

These notes are implementation constraints extracted from the required sources.
They are not optional commentary.

| Source | Notes that drive Phase 053 |
| --- | --- |
| `docs/tech-papers/Z00Z-HJMT-Design.md` | The target topology is a bucketed root-chained JMT forest: definition tree, serial tree, bucket tree, and terminal tree. The target generalized view is `SettlementStateRoot -> Definition -> Serial -> AssetLeaf or RightLeaf`. |
| `docs/tech-papers/Z00Z-HJMT-Design.md` | `RightLeaf` is a narrow bounded settlement object for non-coin rights. It must not become a broad wallet authority, fee container, legal prose blob, workflow record, or generic key-value object. |
| `docs/tech-papers/Z00Z-HJMT-Design.md` | `FeeEnvelope` answers who pays for verification, batching, publication, or relay. It must stay separate from ownership and right validity. |
| `docs/tech-papers/Z00Z-HJMT-Design.md` | deletion and non-existence proofs must be proof objects. A node-local `not found` response is not a proof. Non-existence must bind the root, path, default commitment, index, and transcript version. |
| `docs/tech-papers/Z00Z-HJMT-Design.md` | adaptive buckets require epoch, policy-transition evidence, historical proof support, privacy, recovery, and benchmark rules before live rollout. Phase 053 supplies those rules and code. |
| `.planning/phases/000/051-HJMT-Facade/051-TODO.md` | Phase 051 created the old facade and oracle. Phase 053 must keep storage ownership and downstream physical-layout guardrails, but it must not keep a legacy runtime facade. |
| `.planning/phases/052-HJMT-Backend/052-TODO.md` | Phase 052 landed fixed-bucket forest mechanics, journal, recovery, inclusion proof envelope, dual verify, benchmarks, and scenario_1 coverage behind `AssetStore`. Phase 053 may reuse proven internals, but the live surface becomes HJMT-only. |
| `.planning/phases/052-HJMT-Backend/052-08-PLAN.md` | Adaptive bucket split, merge, policy-transition proof, epoch, stale-proof, recovery, and simulator duties were deliberately deferred. They are now in scope. |
| `.planning/phases/052-HJMT-Backend/052-09-PLAN.md` | Proof-visible occupancy metadata was privacy-sensitive and deferred. Phase 053 must implement a privacy-reviewed live form instead of leaking raw activity counters. |
| `.planning/phases/052-HJMT-Backend/052-10-PLAN.md` | `SettlementStateRoot` was blocked while Phase 052 stayed asset-centric. Phase 053 removes that blocker by replacing the dev runtime with the HJMT root directly, not by shipping an old-storage conversion lane. |
| `.planning/phases/052-HJMT-Backend/052-11-PLAN.md` | `RightLeaf` and `FeeEnvelope` were separate future candidates. Phase 053 implements both and keeps them separate. |
| `.planning/STATE.md` | Current state says Phase 052 is complete and those candidates remain non-live. Phase 053 must update state only after repository-backed code and tests land. |
| `crates/z00z_storage/src/serialization/build.rs` | Current serialization still builds a compatibility projection when the store is not already in compatibility mode. Phase 053 must rewrite serialization around the live HJMT surface and delete the compatibility projection path from production crates. |
| `crates/z00z_storage/src/settlement/proof.rs` | Current proof code still carries compatibility and forest envelope branches side by side. Phase 053 must land one live HJMT proof surface and remove dead compatibility-proof implementation paths instead of parking them under old names. |
| `crates/z00z_storage/src/settlement/types_identity.rs` | Live code currently has `AssetStateRoot`, `AssetPath`, `BucketId`, `BucketPolicy`, and fixed-bucket derivation. |
| `crates/z00z_storage/src/settlement/types_record.rs` | Live code currently has `DefinitionRootLeaf`, `SerialRootLeaf`, `BucketRootLeaf`, `StoreItem`, `SnapItem`, and `ProofItem` around `AssetLeaf`. |
| `crates/z00z_storage/src/settlement/proof.rs` | Live HJMT proof envelope is inclusion-only. `HjmtProofFamily::Deletion` and `HjmtProofFamily::NonExistence` reject as unsupported. |
| `crates/z00z_storage/src/settlement/hjmt_commit.rs` | Live HJMT forest commit is deterministic and child-before-parent, but terminal child commits run serially and caches are not a production contract. |
| `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs` | Live HJMT forest journal has `Prepared -> ChildrenCommitted -> ParentsCommitted -> RootPublished` and fail-closed recovery. Phase 053 must extend this to root generation, rights, fee envelopes, deletion/absence proofs, and bucket policy transitions. |
| `crates/z00z_storage/tests/test_live_guardrails.rs` | Live Phase 053 guardrails require `SettlementStateRoot`, `RightLeaf`, `FeeEnvelope`, `AdaptiveBucket`, `BucketEpoch`, `SplitProof`, `MergeProof`, and `PolicyTransitionProof`, and still reject fake aliases and downstream physical-layout authority leaks. |
| `crates/z00z_storage/tests/test_live_guardrails.rs` | Legacy-purge guardrails reject stale Phase 052 future-only wording, old human-facing forest names, and leftover compatibility-era documentation drift. |
| `crates/z00z_simulator/src/scenario_1/*` | Scenario 1 consumes storage through semantic APIs, proof blobs, snapshots, checkpoint roots, and storage replay. Phase 053 examples must preserve that boundary while adding generalized settlement use cases. |
| `crates/z00z_core/src/assets/assets_config.yaml` | Current asset config has only `assets:`. Phase 053 must make this canonical repository example config generate deterministic `rights:` examples beside assets without overloading `AssetClass` or putting fee semantics into right meaning. |
| `crates/z00z_core/src/assets/assets_config_schema.yaml` | Current schema defines assets only. Phase 053 must make `rights:` a validated canonical section here too so asset-config examples cannot silently drift from `RightLeaf` fields. |
| `crates/z00z_core/src/genesis/genesis_config_devnet.yaml` | Current devnet genesis config generates assets only. Phase 053 must add rights fixtures and deterministic generation rules. |
| `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml` | Scenario 1 currently points at the underscore file name. Phase 053 must update this canonical small config; do not create a divergent `genesis_config_devnet-small.yaml` copy. |
| `crates/z00z_core/src/genesis/genesis_config_testnet.yaml` | Current testnet genesis config is still asset-only. Phase 053 must add a rights example corpus that uses the same validated `rights:` contract as devnet while staying suitable for testnet documentation and regression tests. |
| `crates/z00z_core/src/genesis/genesis_config_mainnet.yaml` | Current mainnet genesis config is still asset-only. Phase 053 must add a curated rights example corpus under the same `rights:` schema so canonical repository genesis examples are consistent across chain types. |
| `crates/z00z_core/src/genesis/genesis_config_schema.yaml` | Current schema requires `assets` but has no first-class `rights:` family. Phase 053 must make `rights:` a validated canonical section for every repository genesis config in this phase scope. |
| `crates/z00z_core/src/genesis/genesis_config.rs` | Current parser reads only `assets: Vec<AssetConfigEntry>`. Phase 053 must parse assets and rights into a new settlement corpus shape. Backward-compatible parsing is not required for dev hard cutover. |
| `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` | Stage 1 currently loads `genesis_config_devnet_small.yaml`. Phase 053 scenario config must select rights-enabled genesis fixtures and drive later HJMT examples from those generated rights. |

## 🛑 Non-Negotiable Rules

- `docs/tech-papers/Z00Z-HJMT-Design.md` is the normative feature source for this phase.
- No placeholder `RightLeaf`, `FeeEnvelope`, deletion proof, non-existence
  proof, split proof, merge proof, policy-transition proof, or cache API may return
  fake success.
- Canonical repository genesis configs `devnet`, `devnet_small`, `testnet`,
  and `mainnet` must generate example rights in addition to assets. Asset-only
  genesis examples are not an acceptable Phase 053 green state.
- Canonical repository asset example config
  `crates/z00z_core/src/assets/assets_config.yaml` must generate example rights
  in addition to assets, and `assets_config_schema.yaml` must validate that
  `rights:` section. Asset-only asset-config examples are not an acceptable
  Phase 053 green state.
- Rewrite or update old storage code in place for dev hard cutover. Do not
  preserve superseded implementation families under parallel names such as
  `legacy`, `old`, `compat`, `simple_jmt`, `v1`, or `v2` just to avoid
  deletion. Live protocol/root/proof version fields are allowed; dead
  implementation lanes are not.
- YAML rights must use a separate `rights:` family or stronger equivalent. Do
  not encode rights by adding a fake `AssetClass::Right` variant or by placing
  right semantics into asset metadata.
- No legacy storage compatibility path may be a Phase 053 green path. Delete or
  replace legacy runtime storage code where it conflicts with the HJMT surface.
  Historical Phase 052 tests may remain only as clearly archived, non-live
  evidence outside production crate surfaces; otherwise they must be rewritten
  as HJMT tests or deleted.
- Gather and remove leftover simple-JMT or compatibility-storage tails from
  live crates: dead backend modes, compatibility projections, dual-verify
  helpers, flat-row proofs, unused reload/persist helpers, stale docs, and
  stale tests must be deleted rather than left as dormant code.
- Phase 053 must not add aliases, shims, legacy row readers, adapter facades,
  or conversion artifacts from `AssetStateRoot`/`AssetPath` into the new
  runtime.
- A cache is never authority. Every cached root, proof segment, policy result,
  or HJMT policy-transition artifact must be recomputable from durable
  committed state and must fail closed on mismatch.
- Public callers must not receive `TreeId`, `ForestTreeId`, namespace bytes,
  raw JMT roots, RedB row keys, or branch ordering as authority.
- `FeeEnvelope` must never prove ownership, right validity, or wallet control.
- `RightLeaf` must never contain fee authority. It may reference fee support
  only through explicit bounded protocol rules.
- Adaptive bucket proofs must bind prior root, next root, bucket epoch, prior
  policy, next policy, transitioned key range, parent roots, and journal digest.
- Proof-visible occupancy metadata must not become a public activity feed.
  Exact counters are local diagnostics unless an explicitly versioned proof
  policy commits a privacy-reviewed range or threshold form.
- All new production APIs must use typed errors. Do not add production
  `unwrap`, broad `expect`, or `panic` paths.

## ✅ Live-Code Acceptance Definition

For this phase, "implemented" means repository production code, not a fixture
stub or simulator-only demonstration.

A Phase 053 feature counts as live only when all applicable conditions are
true:

- the type or API is exported from the owning crate surface needed by real
  callers;
- canonical encoding, hashing, and versioning are implemented through the
  production codec path;
- validation returns typed errors and preserves state on every reject path;
- RedB persistence, reload, and drift rejection exist for durable state;
- proof generation and verification both use the storage-owned verifier;
- checkpoint, snapshot, wallet, validator, and simulator consumers use the
  semantic API instead of physical layout;
- deterministic fixtures use the same constructors, codecs, validators,
  stores, journals, and proof verifiers as non-fixture code;
- unit, integration, end-to-end, benchmark compile, and measured benchmark
  evidence exists where the feature affects that layer.

Fixtures are allowed only as deterministic inputs. A fixture must never replace
the live parser, live store, live proof builder, live verifier, live journal, or
live reload path.

## ⚙️ Target Architecture

Phase 053 replaces the dev storage runtime with the generalized HJMT
generation. It does not preserve a live legacy storage format.

```text
Phase 053 production generation:
  SettlementStateRoot
    SettlementDefinitionLeaf
      SettlementSerialLeaf
        AdaptiveBucketRootLeaf
          SettlementLeaf::Asset(AssetLeaf)
          SettlementLeaf::Right(RightLeaf)

Processing-support contract:
  FeeEnvelope
    binds payer/sponsor, budget, expiry, replay protection, and transition
    support to one settlement operation
    does not prove terminal right ownership
```

The public semantic contract becomes HJMT-only:

- root: `SettlementStateRoot`;
- path: `SettlementPath`;
- terminal leaves: `SettlementLeaf::Asset(AssetLeaf)` and
  `SettlementLeaf::Right(RightLeaf)`;
- processing support: `FeeEnvelope`;
- no public `AssetStateRoot`/`AssetPath` runtime alias or adapter surface.

The physical backend remains storage-owned:

- definition tree;
- serial tree;
- bucket tree;
- terminal tree per bucket;
- path index as rebuildable private lookup;
- cache plane as private acceleration;
- RedB rows and journal as durability, not public proof semantics.

## 🔑 Required Live Contracts

| Contract | Phase 053 requirement |
| --- | --- |
| `SettlementStateRoot` | New typed root with root generation, codec/proof generation, and checkpoint binding. It is storage-family settlement state only, not a universal protocol root. |
| `RootGeneration` | Single live `SettlementV1` generation for Phase 053 dev. Reject legacy root bytes, wrong-generation proof, downgrade, or mixed-generation checkpoint inputs. |
| `SettlementPath` | Canonical three-part path preserving definition, serial, and terminal identity. It replaces `AssetPath` on the live storage surface. |
| `TerminalId` | New terminal identifier used by generalized leaves. Asset ids remain valid terminal ids for asset leaves. |
| `SettlementLeaf` | Versioned enum or equivalent tagged family for `AssetLeaf` and `RightLeaf`. The leaf family marker is committed in hashes and proofs. |
| `RightLeaf` | Narrow, typed, non-coin right object with class, issuer/provider scope, holder/control binding, beneficiary binding when needed, expiry, challenge/revocation/transition policy, disclosure policy, and payload commitment. |
| `FeeEnvelope` | Separate processing-support object with payer/sponsor binding, budget, fee domain, expiry, nonce/replay protection, right/transition binding, and failure semantics. |
| `SettlementProofEnvelope` | Versioned proof envelope supporting inclusion, deletion, non-existence, split, merge, and HJMT policy-transition proof families. |
| `AdaptiveBucket` | Live bucket metadata with epoch, policy id, key range or prefix, parent serial binding, root binding, and privacy-reviewed occupancy evidence. |
| `BucketEpoch` | Monotonic generation for bucket policy and adaptive policy transitions. Bound into every bucket proof and historical proof. |
| `SplitProof` | Proof that one bucket split into child buckets under one committed epoch transition. |
| `MergeProof` | Proof that adjacent or policy-compatible buckets merged into one bucket under one committed epoch transition. |
| `PolicyTransitionProof` | Proof that roots and terminal leaves moved from one HJMT policy or epoch to another without silent loss, duplication, or replay. This is internal HJMT policy evidence, not an old-storage conversion artifact. |
| `BucketOccupancyEvidence` | Versioned privacy-reviewed evidence for adaptive thresholds. Exact local counters are diagnostic; proof-visible data must be policy-bound and tested for correlation risk. |
| `ForestCache` | Private cache plane for stable roots, parent leaves, bucket derivations, proof segments, journal digests, and HJMT policy-transition evidence. |
| `ForestScheduler` | Bounded async/parallel execution layer for child commits, proof generation, HJMT policy-transition work, and warm reload. |
| `RightsConfigEntry` | New core YAML config entry for deterministic example rights. It must map to `RightLeaf` fields without reusing `AssetConfigEntry` or `AssetClass`. |
| `GenesisRightsConfig` | Genesis config surface that generates right fixtures beside asset fixtures for devnet, devnet-small, testnet, and mainnet examples. It must bind chain id, genesis seed, right domain, right class, terminal id derivation, holder/control fixture, expiry policy, and disclosure policy. |
| `GenesisSettlementCorpus` | Combined generated asset/right corpus used by storage and scenario_1 to build `SettlementStateRoot`. Asset-only canonical genesis examples are invalid in Phase 053. |

## 💯 Performance And Cache Contract

Maximum performance is a phase requirement. The forest has many roots and
proof segments that do not change in a typical batch. Recomputing unchanged
subtrees is forbidden when the backend has enough committed state to reuse
them safely.

### Required Cache Layers

| Cache | Required behavior |
| --- | --- |
| `SubtreeRootCache` | Cache roots for unchanged definition, serial, bucket, and terminal subtrees by generation, epoch, tree id, version, policy id, and subtree key. |
| `ParentLeafCache` | Cache encoded `SettlementDefinitionLeaf`, `SettlementSerialLeaf`, and `AdaptiveBucketRootLeaf` values for unchanged child roots. |
| `TerminalLeafCache` | Cache canonical encodings and leaf hashes for `AssetLeaf` and `RightLeaf`, keyed by leaf family, terminal id, codec version, and payload hash. |
| `BucketDerivationCache` | Cache path-to-bucket derivation by root generation, path, policy id, epoch, hash domain, and bucket bits/range. |
| `ProofSegmentCache` | Cache branch proof bytes or decoded proof objects for unchanged definition, serial, bucket, and terminal proof segments. |
| `NonExistenceCache` | Cache default-slot openings and default commitment transcripts for absent paths under a specific root, path, epoch, and proof version. |
| `PolicyProofCache` | Cache split, merge, and policy-transition proof segments by prior root, next root, prior epoch, next epoch, affected bucket id/range, and journal digest. |
| `JournalDigestCache` | Cache child and parent digest inputs after canonical sorting, but bind every entry to journal status and root generation. |
| `PathIndexCache` | Cache `terminal_id -> SettlementPath` lookup results, but keep the persisted path index rebuildable and non-authoritative. |
| `WarmReloadCache` | Rehydrate cache entries from durable root rows and proof metadata after reload only after journal and digest validation succeed. |

### Cache Key Requirements

Every cache key that can influence a root, proof, or HJMT policy-transition
decision must bind all relevant dimensions:

- root generation;
- semantic root bytes;
- backend mode and backend generation;
- tree identity;
- definition id;
- serial id;
- terminal id;
- terminal leaf family;
- bucket policy id;
- bucket epoch;
- bucket id or adaptive bucket range;
- codec version;
- proof envelope version;
- hash domain version;
- default commitment version for absence proofs;
- journal version and journal digest when used after commit;
- split, merge, or policy-transition id when adaptive policy changes.

If a key omits one of these dimensions, a targeted test must prove the
omission is safe. Otherwise the implementation must fail review.

### Dirty-Set Invalidation

Phase 053 must implement dirty-set invalidation instead of broad cache flushes:

| Mutation | Invalidate |
| --- | --- |
| asset put/update/delete | terminal leaf cache for path, terminal bucket root, bucket parent leaf, serial root, definition root, semantic root, proof segments on the touched path |
| right create/update/revoke/consume/expire | right leaf cache for path, terminal bucket root, bucket parent leaf, serial root, definition root, semantic root, right transition proof segments |
| non-existence proof generation for absent key | no root invalidation; cache only root-bound default-slot proof segment |
| fee envelope accepted | fee-envelope validation cache for the transition id; no terminal leaf mutation unless the settlement transition commits one |
| fee envelope rejected | no root invalidation; rejection state must not pollute acceptance caches |
| bucket split | prior bucket proof cache, prior terminal bucket root, affected parent bucket tree, serial root, definition root, semantic root, policy proof cache for prior epoch |
| bucket merge | merged bucket caches, affected parent bucket tree, serial root, definition root, semantic root, policy proof cache for prior epoch |
| proof-version change | proof caches for the changed version; durable roots remain HJMT roots and must be re-verified before reuse |
| journal rollback | all cache entries for the rolled-back journal version and all entries derived from unpublished roots |

Untouched subtrees must remain reusable. A batch that touches one terminal
bucket must not recompute unrelated bucket roots, serial roots, definition
roots, proof segments, or HJMT policy-transition evidence.

### Async And Parallel Execution

Phase 053 must introduce a bounded production scheduler:

- CPU-heavy JMT hashing, leaf encoding, proof generation, default openings,
  split, merge, and policy-transition proof work runs through a bounded CPU
  pool.
- Blocking RedB work runs through `tokio::task::spawn_blocking` or an
  equivalent storage-owned blocking executor. Do not block async runtime
  worker threads on RedB transactions.
- Parent commits remain topologically ordered after child roots are known.
- Parallel child work must return deterministic results by sorting definition,
  serial, bucket, and terminal keys before parent updates.
- Cancellation must roll back uncommitted in-memory state and leave durable
  journal recovery fail-closed.
- The scheduler must expose backpressure for large batches instead of
  spawning unbounded tasks.
- Sync APIs may remain for ergonomics, but they must call the scheduler
  through a safe blocking bridge rather than duplicating mutation logic.

### Performance Metrics

The following metrics must be recorded in benchmark output and closeout notes:

- search/read p50/p95/p99 for `get_item`, `find_asset` or terminal-id lookup,
  `lookup`, paginated `list`, definition-filtered list, serial-filtered list,
  right-class-filtered list, absent-path lookup, and path-index rebuild lookup;
- insert p50/p95/p99 and throughput for single insert, batch insert, broad
  definitions, hot definition, hot serial, hot bucket, mixed asset/right
  insert, right create, and fee-supported right create;
- delete p50/p95/p99 and throughput for single delete, batch delete, broad
  definitions, hot definition, hot serial, right consume, right revoke, right
  expiry cleanup, and parent pruning;
- mixed workload p50/p95/p99 for insert plus search, delete plus search,
  proof generation plus writes, policy transition plus reads, and fee
  validation plus right transition;
- batch size, dirty path count, dirty bucket count, dirty serial count, dirty
  definition count;
- root reuse ratio;
- proof segment reuse ratio;
- cache hit, miss, insert, eviction, and invalidation counts per cache;
- cache memory footprint;
- child commit p50/p95/p99;
- parent commit p50/p95/p99;
- proof generation p50/p95/p99 by proof family for inclusion, deletion,
  non-existence, split, merge, policy transition, historical proof, and
  shared-parent multi-proof cases;
- proof verification p50/p95/p99 by proof family for inclusion, deletion,
  non-existence, split, merge, policy transition, historical proof, and
  tamper-reject cases;
- serialized proof size by proof family with component breakdown for root
  binding, definition segment, serial segment, bucket segment, terminal
  segment, default-commitment segment, policy-transition segment, and
  fee-support binding where present;
- batch proof aggregation metrics for shared definition, shared serial, shared
  bucket, mixed asset/right leaves, mixed inclusion/non-existence proofs, and
  adaptive policy-transition proofs;
- split, merge, and policy-transition prove/verify p50/p95/p99;
- adaptive bucket trigger cost, hysteresis decision cost, occupancy-evidence
  cost, stale-proof rejection cost, and historical proof support cost;
- reload warm-cache time and cold-cache time;
- cold-cache versus warm-cache delta for search, insert, delete, proof
  generation, proof verification, split, merge, policy transition, reload, and
  scenario_1 examples;
- memory, allocation, CPU, RedB I/O, and encoded-byte totals per workload;
- RedB blocking wait time;
- scheduler queue depth and rejected/backpressured batches;
- fixed-bucket HJMT and adaptive HJMT baselines on the same machine and
  workload. Old-storage baselines are not a Phase 053 acceptance signal.

### Required Benchmark Matrix

The benchmark harness must cover these HJMT-specific lanes:

| Lane | Required cases |
| --- | --- |
| Search and read | by full `SettlementPath`, by terminal id/path index, by definition, by serial, by right class, paginated list, absent path, post-reload lookup, cold cache, warm cache |
| Insert | single asset, single right, fee-supported right, many definitions, one hot definition, one hot serial, one hot bucket, mixed asset/right batch, duplicate reject |
| Delete and prune | single asset delete, batch delete, right consume, right revoke, expiry cleanup, empty bucket prune, empty serial prune, empty definition prune, missing delete reject |
| Proof generation | inclusion, deletion, non-existence, split, merge, policy transition, historical proof, shared-parent proof batch, mixed inclusion/non-existence batch |
| Proof verification | valid proof families, malformed bytes, wrong root generation, wrong epoch, wrong default commitment, stale policy-transition proof, present-key absence reject |
| Proof size | single leaf, shared definition, shared serial, shared bucket, mixed asset/right, deletion, non-existence, split, merge, policy transition, aggregated proof bundle |
| Adaptive buckets | split trigger, split no-op under hysteresis, merge trigger, merge no-op under hysteresis, policy transition, prior-policy historical proof |
| Cache | root reuse, proof segment reuse, dirty-small, dirty-hot, dirty-wide, invalidation, eviction, mismatch fail-closed, warm reload |
| Async scheduler | bounded child commits, bounded proof generation, queue backpressure, cancellation rollback, deterministic parent ordering, blocking RedB bridge |
| Durability | RedB persist, reload, crash at every journal status, policy-transition crash, fee replay reload, cache warmup after reload |
| End-to-end | scenario_1 generalized HJMT mode and adaptive HJMT mode |

## 🧭 PLAN Derivation Order

Future PLAN files derived from this TODO must preserve dependency order. A PLAN
may combine adjacent slices only when it still produces live code and tests for
each slice.

| Order | Slice | Required green signal before the next slice |
| --- | --- | --- |
| 1 | Guardrail flip and live contract exports | old future-only blockers are replaced by tests requiring real exports and rejecting fake aliases |
| 2 | Root generation, path, leaf, right, and fee contracts | canonical encodings, typed validation, and unit tests are green |
| 3 | Core YAML and genesis rights | devnet, devnet-small, testnet, and mainnet parse, generate, export, and report deterministic rights |
| 4 | HJMT store API and dev hard cutover | mixed asset/right store API works and legacy storage adapters are absent |
| 5 | Proof envelope v2 | inclusion, deletion, non-existence, and tamper matrices pass |
| 6 | Adaptive buckets and occupancy evidence | split, merge, policy transition, epoch, stale-proof, and privacy tests pass |
| 7 | Cache and scheduler | cache correctness, invalidation, async determinism, cancellation, and backpressure tests pass |
| 8 | Journal, RedB, reload, historical proofs | crash/reload/drift tests pass for roots, rights, fees, policy transitions, and cache warmup |
| 9 | Checkpoint, snapshot, wallet, validator | downstream code uses semantic APIs and rejects physical layout authority |
| 10 | scenario_1 examples and debug artifacts | end-to-end run writes schema-checked artifacts and runner tamper checks fail closed |
| 11 | Corpus, fuzz seeds, benchmarks, docs | broad tests, measured benchmarks, and docs match HJMT-only live code |
| 12 | Legacy purge and closeout | no simple-JMT or compatibility-storage tail remains in live crates; state and summary match the landed code |

## 🧩 Implementation Tasks

### 053-01 Replace Future-Only Guardrails With Live-Contract Guardrails

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 1.2, 2.4, 5.2, 12, and 14
- `.planning/phases/052-HJMT-Backend/052-08-PLAN.md`
- `.planning/phases/052-HJMT-Backend/052-09-PLAN.md`
- `.planning/phases/052-HJMT-Backend/052-10-PLAN.md`
- `.planning/phases/052-HJMT-Backend/052-11-PLAN.md`

- [x] Keep Phase 053 guardrail coverage split between
  `test_live_guardrails.rs` for live exports/source shape and
  `test_live_guardrails.rs` for stale future-only and legacy-name purge
  checks.
- [x] Preserve and strengthen downstream guardrails that prevent wallet,
  validator, runtime, and simulator crates from treating physical forest
  layout as authority.
- [x] Add guardrails that reject fake aliases, shims, and adapters:
  `SettlementStateRoot` must not be a bare `type` alias to `AssetStateRoot`,
  `RightLeaf` must not be a bare wrapper around `AssetLeaf`, and
  `FeeEnvelope` must not appear inside `RightLeaf`. No live storage module may
  expose an `AssetStateRoot` old-root shim.
- [x] Update `crates/z00z_storage/src/settlement/root-types.md` and
  `crates/z00z_storage/src/settlement/README.MD` so former future terms are
  documented as Phase 053 live generation contracts.
- [x] Update `.planning/STATE.md` only after implementation and verification
  evidence exists.

Files:

- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/src/settlement/root-types.md`
- `crates/z00z_storage/src/settlement/README.MD`
- `.planning/STATE.md`

Tests:

- [x] source-shape test requires live generalized exports in `z00z_storage`.
- [x] source-shape test rejects downstream imports of physical layout names.
- [x] source-shape test rejects fake alias, shim, and adapter implementations.
- [x] source-shape test proves old Phase 052 future-only wording is removed or
  explicitly marked historical.

Exit condition:

- The repository no longer blocks Phase 053 live HJMT contracts, but still
  blocks physical-layout authority leakage.

### 053-02 Settlement Root Generation And Hard Cutover Model

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 1.2, 5.2, 8.3, 9.3, 12, and 14

- [x] Add `SettlementStateRoot` with explicit generation metadata and root
  bytes.
- [x] Add `RootGeneration` or equivalent typed generation marker:
  `SettlementV1` is the only live Phase 053 generation.
- [x] Reject `AssetStateRoot`, `AssetPath`, and Phase 052 proof bytes at the
  new HJMT boundary. Do not add conversion adapters.
- [x] Add checkpoint statement binding for prior and next
  `SettlementStateRoot` in the new generation.
- [x] Add proof-envelope root binding for the `SettlementV1` generation.
- [x] Add downgrade rejection: a `SettlementV1` store or checkpoint must reject
  every old asset root, proof, or path input as unsupported.
- [x] Add mixed-generation rejection with state preservation. There is no
  proof that authorizes old storage roots in Phase 053.
- [x] Switch production generalized mode to the Phase 053 root contract only
  after all proof, reload, checkpoint, and scenario gates pass.

Files:

- `crates/z00z_storage/src/settlement/types_identity.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/checkpoint/*`
- `crates/z00z_storage/tests/test_settlement_root.rs`

Tests:

- [x] `test_settlement_root_is_not_asset_alias`
- [x] `test_root_generation_binds_proof`
- [x] `test_root_generation_binds_checkpoint`
- [x] `test_old_asset_inputs_rejected`
- [x] `test_downgrade_rejected_with_state_preserved`
- [x] `test_mixed_generation_rejected_without_legacy_lane`
- [x] reload tests for settlement root and wrong generation.

Exit condition:

- `SettlementStateRoot` is a real live root generation with checkpoint and
  proof binding, not renamed `AssetStateRoot`.

### 053-03 SettlementPath, TerminalId, SettlementLeaf, And RightLeaf

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 2.4, 2.4.1, 5.1, 11, 12, and Appendix A

- [x] Add `TerminalId` and `SettlementPath` preserving the canonical
  definition, serial, terminal hierarchy.
- [x] Add `SettlementLeaf` as a versioned tagged terminal family with at least
  `Asset(AssetLeaf)` and `Right(RightLeaf)`.
- [x] Add `RightLeaf` with typed fields for:
  - version;
  - terminal id;
  - right class;
  - issuer scope;
  - provider scope when relevant;
  - holder or control commitment;
  - beneficiary commitment when relevant;
  - payload commitment;
  - expiry or validity window;
  - one-time-use marker when relevant;
  - revocation or transition policy;
  - challenge window;
  - disclosure policy id;
  - retention policy id.
- [x] Add canonical encoding and `serde(deny_unknown_fields)` where serialized.
- [x] Add hash-domain separation between asset leaves and right leaves.
- [x] Add strict path and leaf consistency checks for `RightLeaf`.
- [x] Add right-class taxonomy that is narrow enough for machine, agent,
  access, external right, data, service-credit, and liability-domain use cases
  without becoming a generic KV store.
- [x] Add transition validation for create, transfer, consume, expire, revoke,
  and challenge outcomes.
- [x] Keep `RightLeaf` free of fee budget, payer, sponsor, relay, or
  publication fields.

Files:

- `crates/z00z_storage/src/settlement/types_identity.rs`
- `crates/z00z_storage/src/settlement/types_record.rs`
- `crates/z00z_storage/src/settlement/leaf.rs`
- `crates/z00z_storage/src/settlement/mod.rs`
- `crates/z00z_storage/tests/test_right_leaf.rs`
- `crates/z00z_storage/tests/test_settlement_leaf.rs`

Tests:

- [x] canonical right leaf encoding golden tests.
- [x] right path mismatch rejects.
- [x] wrong right class rejects.
- [x] expired right transition rejects.
- [x] revoked right transition rejects.
- [x] one-time-use replay rejects.
- [x] wrong holder/control binding rejects.
- [x] right leaf hash differs from equivalent asset-shaped bytes.
- [x] `FeeEnvelope` fields are absent from `RightLeaf`.

Exit condition:

- `RightLeaf` is a production terminal settlement object with typed semantics
  and fail-closed validation.

### 053-04 FeeEnvelope Contract And Separation From Rights

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 2.4, 2.4.1, 8.3, 11, 12, and 14

- [x] Add `FeeEnvelope` as a versioned processing-support object.
- [x] Include payer or sponsor binding, fee domain, budget, expiry, replay
  nonce, transition binding, and optional fee-credit or reserve reference.
- [x] Validate `FeeEnvelope` before the settlement transition commits.
- [x] Reject expired, insufficient, wrong sponsor, wrong transition, wrong
  domain, and replayed fee envelopes.
- [x] Ensure accepted fee support does not prove right ownership or mutate
  `RightLeaf` meaning.
- [x] Add durable replay protection for fee-envelope ids or nonces.
- [x] Add fee-envelope failure semantics that preserve state and do not emit
  partial forest commits.
- [x] Keep wallet, validator, and simulator APIs explicit about fee support as
  processing support only.

Files:

- `crates/z00z_storage/src/settlement/fee_envelope.rs`
- `crates/z00z_storage/src/settlement/types_record.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/tests/test_fee_envelope.rs`
- `crates/z00z_storage/tests/test_fee_replay.rs`

Tests:

- [x] valid fee envelope accepts before a right transition.
- [x] invalid fee envelope rejects before mutation.
- [x] expired fee support rejects.
- [x] insufficient fee support rejects.
- [x] wrong sponsor rejects.
- [x] wrong right or transition binding rejects.
- [x] fee replay rejects after reload.
- [x] fee support cannot be passed to a proof verifier as ownership evidence.

Exit condition:

- Fee support is live, durable, and separate from terminal right semantics.

### 053-05 HJMT Store API And Dev Hard Cutover

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 4.2, 7.2, 9.1, and 12

- [x] Add a storage-owned generalized backend trait for settlement operations.
- [x] Replace live `AssetTreeBackend` usage with the HJMT settlement backend.
  Archived tests may reference old names, but production storage code must not
  route through asset-generation adapters.
- [x] Add put, delete, get, lookup, list, prove, and apply-batch APIs for
  `SettlementPath` and `SettlementLeaf`.
- [x] Add right transition APIs for create, transfer, consume, revoke, expire,
  and challenge.
- [x] Add transition APIs that accept `FeeEnvelope` only where processing
  support is required.
- [x] Keep path index internal and rebuildable.
- [x] Add typed errors for unsupported generation, wrong leaf family,
  right-transition failure, fee failure, and HJMT policy-transition failure.
- [x] Add explicit generalized backend mode selection for Phase 053. Keep
  old mode names rejected. Production dev runtime must select HJMT directly.
- [x] Remove old asset callers from the live storage path. Callers must use
  `SettlementPath`, `SettlementLeaf`, and `SettlementStateRoot`.
- [x] Rewrite production storage entrypoints in place instead of keeping
  side-by-side old/new backend families under `legacy`, `compat`, `v1`, or
  `v2` names.

Files:

- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/hjmt_plan.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/store_query.rs`
- `crates/z00z_storage/tests/test_store_api.rs`

Tests:

- [x] old asset facade calls reject or are deleted from production code.
- [x] generalized put/get/list works for asset and right leaves.
- [x] asset API cannot mutate right leaves.
- [x] right API cannot treat assets as non-coin rights.
- [x] fee-requiring transitions reject without fee support.
- [x] backend mode parser accepts the Phase 053 generalized mode and rejects
  stale or misspelled names.
- [x] state preservation after every rejecting generalized operation.

Exit condition:

- The HJMT backend is the only live dev storage authority and no legacy asset
  facade is required for green status.

### 053-06 Core YAML, Genesis Rights, And Full-Stack Fixture Integration

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 2.4, 4.2, 8.2, 11, 12, and 13
- `crates/z00z_core/src/assets/assets_config.yaml`
- `crates/z00z_core/src/assets/assets_config_schema.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml`
- `crates/z00z_core/src/genesis/genesis_config_mainnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config_schema.yaml`
- `crates/z00z_core/src/genesis/genesis_config_testnet.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`

Phase 053 must update the core YAML inputs so every canonical repository genesis
example generates rights beside assets. This is not a comment-only config
change: loaders, validators, generators, exports, storage ingestion, and
scenario_1 must all consume the new rights fixtures through production types.

Required YAML fixture families:

| Right id | Right class | devnet-small count | devnet count | testnet count | mainnet count | Required transition coverage |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `machine_compute_capability` | `machine_capability` | 3 | 25 | 12 | 4 | create, transfer, consume, replay reject |
| `confidential_data_access` | `data_access` | 3 | 25 | 12 | 4 | create, expire, absence proof after expiry cleanup |
| `service_entitlement` | `service_entitlement` | 3 | 25 | 12 | 4 | create, transfer, revoke |
| `validator_mandate` | `validator_mandate` | 2 | 10 | 6 | 2 | create, challenge, revoke |
| `one_time_agent_action` | `one_time_use` | 3 | 25 | 12 | 4 | create, consume, second consume reject |

Every fixture family must bind `issuer_scope`, `holder_fixture`,
`control_fixture`, `domain_name`, `payload_commitment_seed`, `expiry_policy`,
`transition_policy`, `revocation_policy`, `disclosure_policy`, and
`metadata.purpose`. The holder/control fixtures must be deterministic labels
that scenario_1 can map to Alice, Bob, Charlie, or a non-wallet service actor.

- [x] Update `crates/z00z_core/src/assets/assets_config.yaml` with a top-level
  `rights:` section containing deterministic example rights for at least:
  machine capability, data access, service entitlement, validator mandate, and
  one expiring one-time-use right.
- [x] Treat `crates/z00z_core/src/assets/assets_config.yaml` as a canonical
  repository example input, not a side fixture. Asset-only example content is
  invalid after Phase 053 lands.
- [x] Update `crates/z00z_core/src/genesis/genesis_config_devnet.yaml` with the
  same rights families at devnet scale.
- [x] Update `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml`
  with small deterministic right counts suitable for scenario_1 and CI.
- [x] Update `crates/z00z_core/src/genesis/genesis_config_testnet.yaml` with
  the same `rights:` families at curated testnet scale.
- [x] Update `crates/z00z_core/src/genesis/genesis_config_mainnet.yaml` with
  the same `rights:` families at curated mainnet-example scale so repository
  canonical genesis examples stay consistent across chain types.
- [x] Do not create or use `genesis_config_devnet-small.yaml`; the canonical
  repository file is `genesis_config_devnet_small.yaml`. Add a guardrail test
  that fails if both names exist with divergent content.
- [x] Version the config schemas so `rights:` is a validated first-class
  section, not an ignored `additionalProperties` side channel.
- [x] Make `crates/z00z_core/src/assets/assets_config_schema.yaml` reject an
  empty or missing `rights:` section for the canonical asset-config example in
  Phase 053.
- [x] Add `RightsConfigEntry` or equivalent in `z00z_core` with fields mapping
  to `RightLeaf`: right id, right class, issuer/provider scope,
  holder/control fixture, beneficiary binding when needed, serial/terminal
  count, domain name, expiry policy, revocation policy, transition policy,
  disclosure policy, payload commitment seed, and metadata.
- [x] Replace `assets: Vec<AssetConfigEntry>`-only parsing with a settlement
  corpus config that requires assets and rights for every canonical repository
  asset-config and genesis example in Phase 053.
- [x] Reject rights that put payer, sponsor, relay fee, or fee budget fields
  inside right meaning. Fee support must remain `FeeEnvelope`.
- [x] Add deterministic right terminal-id derivation bound to genesis seed,
  chain id, right class, domain name, right index, and terminal generation.
- [x] Add deterministic payload commitments for fixture rights. Fixtures may be
  deterministic; production code must still use the same canonical encoding
  and hash domains as non-fixture rights.
- [x] Extend genesis generation from `generate_all_genesis_assets*` to a
  generalized corpus path that returns assets and rights, or add a new
  production `generate_genesis_settlement_corpus*` path. Do not fake rights in
  simulator-only JSON.
- [x] Extend genesis validation to reject duplicate right ids, duplicate
  terminal ids, asset/right terminal collisions, invalid right class, invalid
  expiry policy, invalid transition policy, malformed holder/control fixture,
  metadata type drift, and empty generalized rights corpora in any canonical
  genesis example.
- [x] Extend export output with rights artifacts, for example
  `genesis_rights_<class>.json`, `genesis_rights_<class>.bin`, and a combined
  `genesis_settlement_manifest.json` that binds asset count, right count,
  root generation, schema version, and generation seed hash.
- [x] Extend genesis reports and logs to include asset counts, right counts,
  per-right-class counts, settlement corpus hash, generation timings, and
  verification timings.
- [x] Extend storage ingestion so generated assets and rights become
  `SettlementLeaf` entries under `SettlementStateRoot`. Asset-only ingestion
  must reject in Phase 053 dev mode.
- [x] Update `scenario_1` Stage 1 to load rights-enabled
  `genesis_config_devnet_small.yaml`, generate rights, verify rights, and write
  rights artifacts into the scenario output sandbox.
- [x] Update `scenario_1` Stage 3/4 claim or publish flows so rights are either
  distributed to actors through a typed rights lane or explicitly carried as
  non-wallet-owned settlement fixtures. Do not silently drop generated rights.
- [x] Update `scenario_1` Stage 11 wallet scan so it proves storage inclusion
  first, checks leaf family, and rejects rights as spendable wallet assets.
- [x] Update `scenario_1` Stage 13 HJMT examples to use generated genesis
  rights from the YAML fixtures instead of ad hoc hardcoded rights.
- [x] Update scenario config and design YAML with explicit rights artifact
  paths, right counts, right classes, expected fixture owners, and negative
  right/asset confusion checks.

Files:

- `crates/z00z_core/Cargo.toml`
- `crates/z00z_core/src/assets/assets_config.yaml`
- `crates/z00z_core/src/assets/assets_config_schema.yaml`
- `crates/z00z_core/src/assets/mod.rs`
- `crates/z00z_core/src/assets/assets_config.rs`
- `crates/z00z_core/src/assets/assets_config_load.rs`
- `crates/z00z_core/src/genesis/genesis_config_devnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml`
- `crates/z00z_core/src/genesis/genesis_config_mainnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config_schema.yaml`
- `crates/z00z_core/src/genesis/genesis_config_testnet.yaml`
- `crates/z00z_core/src/genesis/genesis_config.rs`
- `crates/z00z_core/src/genesis/genesis_config_validate.rs`
- `crates/z00z_core/src/genesis/genesis.rs`
- `crates/z00z_core/src/genesis/mod.rs`
- `crates/z00z_core/src/genesis/genesis_run.rs`
- `crates/z00z_core/src/genesis/genesis_output_support.rs`
- `crates/z00z_core/src/assets/right_config.rs`
- `crates/z00z_core/src/genesis/genesis_rights.rs`
- `crates/z00z_core/src/genesis/genesis_settlement_manifest.rs`
- `crates/z00z_core/src/assets/test_rights_config.rs`
- `crates/z00z_core/src/genesis/test_genesis_rights.rs`
- `crates/z00z_core/src/genesis/test_genesis_manifest.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/stage_1.rs`
- `crates/z00z_simulator/src/scenario_1/stage_3.rs`
- `crates/z00z_simulator/src/scenario_1/stage_4.rs`
- `crates/z00z_simulator/src/scenario_1/stage_11_utils/jmt_wallet_scan.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_examples.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`

Tests:

- [x] assets config parser accepts assets plus rights.
- [x] assets config parser rejects malformed rights and fee fields inside
  rights.
- [x] assets config schema rejects unknown required right-field drift instead
  of accepting it through `additionalProperties`.
- [x] assets config schema rejects an empty or missing `rights:` section.
- [x] canonical `assets_config.yaml` parses assets plus rights.
- [x] genesis config schema rejects malformed or missing `rights:` entries in
  every canonical generalized genesis config.
- [x] genesis devnet config parses assets plus rights.
- [x] genesis devnet-small config parses assets plus rights.
- [x] genesis testnet config parses assets plus rights.
- [x] genesis mainnet config parses assets plus rights.
- [x] every canonical generalized genesis config rejects an empty or missing
  `rights:` section.
- [x] duplicate right id rejects.
- [x] duplicate terminal id rejects.
- [x] asset/right terminal collision rejects.
- [x] generated rights are deterministic for the same genesis seed and config.
- [x] changing right class, domain, index, or holder/control fixture changes
  the derived terminal id or payload commitment as expected.
- [x] genesis export writes rights artifacts and combined settlement manifest.
- [x] genesis report records right counts and settlement corpus hash.
- [x] combined manifest hash changes when any generated right changes.
- [x] rights output decode round-trips through the production codec.
- [x] storage ingestion creates `RightLeaf` entries from generated rights.
- [x] scenario_1 Stage 1 writes rights artifacts and records right counts.
- [x] scenario_1 Stage 3/4 does not silently drop generated rights.
- [x] scenario_1 Stage 11 rejects generated `RightLeaf` entries as spendable
  wallet assets.
- [x] scenario_1 Stage 13 examples verify generated rights through production
  HJMT proof APIs.
- [x] runner verification rejects missing rights artifact paths, wrong right
  counts, wrong right class, wrong holder/control fixture, and asset/right
  confusion.

Exit condition:

- Devnet, devnet-small, testnet, and mainnet YAML configs generate assets and
  rights, and the complete core -> storage -> simulator stack consumes those
  rights through production generalized HJMT code.

### 053-07 Proof Envelope V2: Inclusion, Deletion, And Non-Existence

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 5.1, 5.2, 5.3, 8.2, 12, and 13.3

- [x] Add `SettlementProofEnvelope` with versioned proof family.
- [x] Support inclusion proofs for `AssetLeaf` and `RightLeaf`.
- [x] Support deletion proofs that bind prior root, deleted path, old leaf
  hash, next root, affected child roots, affected parent roots, journal digest,
  and proof transcript.
- [x] Support non-existence proofs that open the derived slot to a canonical
  default commitment.
- [x] Define and commit `DEFAULT_VALUE_COMMITMENT` and
  `DEFAULT_CHILD_COMMITMENT` with domain-separated versioning.
- [x] Reject present-key non-existence proofs.
- [x] Reject node-local `not found` as proof.
- [x] Add historical proof verification by root generation and bucket epoch.
- [x] Add proof-size and verify-time measurement for each proof family.
- [x] Keep one storage-owned proof decoder. Downstream crates must not decode
  raw branch proofs.

Files:

- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/store_query.rs`
- `crates/z00z_storage/src/settlement/hjmt_proof.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/store_rows.rs`
- `crates/z00z_storage/src/settlement/tx_plan/tx_plan_types.rs`
- `crates/z00z_storage/src/settlement/tx_plan/tx_plan_batches.rs`
- `crates/z00z_storage/src/snapshot/store.rs`
- `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`

Tests:

- [x] inclusion proof accepts asset leaf.
- [x] inclusion proof accepts right leaf.
- [x] deletion proof accepts valid transition.
- [x] deletion proof rejects wrong prior root.
- [x] deletion proof rejects wrong next root.
- [x] deletion proof rejects wrong old leaf.
- [x] non-existence proof accepts empty tree.
- [x] non-existence proof accepts absent key after inserts.
- [x] non-existence proof rejects present key.
- [x] non-existence proof rejects tampered default value.
- [x] non-existence proof rejects tampered index.
- [x] non-existence proof rejects wrong bucket epoch.
- [x] mixed inclusion plus non-existence aggregation verifies.
- [x] Phase 052 proof bytes do not validate as proof v2.

Exit condition:

- Deletion and non-existence are live proof families with verifier-validating
  semantics.

### 053-08 Adaptive Buckets, BucketEpoch, SplitProof, MergeProof, PolicyTransitionProof

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 4.4, 6.3, 7.3, 8.1, 9.2, 13.2, 13.4,
  and 14

- [x] Add `BucketEpoch` with monotonic epoch and policy-generation binding.
- [x] Add `AdaptiveBucket` metadata with parent serial binding, bucket range
  or prefix, policy id, epoch, root, and privacy-reviewed occupancy evidence.
- [x] Add deterministic split eligibility and merge eligibility policy.
- [x] Add hysteresis to avoid split/merge oscillation.
- [x] Add `SplitProof` binding old bucket root, child bucket roots, old epoch,
  new epoch, policy id, key range, journal digest, and parent roots.
- [x] Add `MergeProof` binding old bucket roots, merged bucket root, old epoch,
  new epoch, policy id, key ranges, journal digest, and parent roots.
- [x] Add `PolicyTransitionProof` binding prior policy, next policy, prior
  root, next root, prior epoch, next epoch, transitioned terminal set
  commitment, and replay digest.
- [x] Add historical proof support across bucket epochs.
- [x] Add stale proof rejection when a proof omits required epoch or policy
  transition context.
- [x] Add crash recovery for split, merge, and policy-transition interruption
  points.
- [x] Add benchmark comparison against fixed buckets before enabling adaptive
  defaults.

Files:

- `crates/z00z_storage/src/settlement/types_record.rs`
- `crates/z00z_storage/src/settlement/hjmt_policy.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/src/settlement/store_rows.rs`
- `crates/z00z_storage/src/settlement/live_recovery_tests.rs`
- `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs`
- `crates/z00z_storage/tests/test_redb_reload.rs`

Tests:

- [x] split proof success.
- [x] split proof rejects wrong old root.
- [x] split proof rejects wrong child root.
- [x] split proof rejects wrong epoch.
- [x] merge proof success.
- [x] merge proof rejects non-adjacent or policy-incompatible buckets.
- [x] policy-transition proof success.
- [x] policy-transition proof rejects stale prior policy.
- [x] policy-transition proof rejects next policy drift.
- [x] historical proof verifies only under authorized epoch rules.
- [x] crash after split children rolls back or completes fail-closed.
- [x] crash after parent policy transition recovers deterministically.

Exit condition:

- Adaptive bucket lifecycle is live and proof-backed, with no silent policy
  rewrite or stale proof acceptance.

### 053-09 Occupancy Metadata Privacy And Adaptive Threshold Evidence

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 8.1, 12, 14, and Appendix A

- [x] Add local exact occupancy counters for scheduling and diagnostics.
- [x] Add proof-visible `BucketOccupancyEvidence` only as a versioned,
  policy-bound privacy-reviewed object.
- [x] Prefer range, threshold, or commitment forms over exact public counts.
- [x] Bind occupancy evidence to bucket id/range, epoch, policy id, root, and
  transcript version.
- [x] Ensure occupancy evidence is not required for ordinary inclusion proofs
  unless the proof family or adaptive transition needs it.
- [x] Add cross-proof correlation tests.
- [x] Add guardrails blocking raw `leaf_count` or equivalent exact public
  counters in production proof envelopes.
- [x] Add local metrics for exact counts that remain non-authoritative.

Files:

- `crates/z00z_storage/src/settlement/types_record.rs`
- `crates/z00z_storage/src/settlement/proof.rs`
- `crates/z00z_storage/src/settlement/hjmt_policy.rs`
- `crates/z00z_storage/tests/test_occupancy_privacy.rs`

Tests:

- [x] local exact occupancy is available only through diagnostic metrics.
- [x] proof-visible occupancy binds policy and epoch.
- [x] tampered occupancy evidence rejects.
- [x] exact counter field names are blocked in production proof payloads.
- [x] split and merge threshold proofs do not leak unrelated bucket counts.
- [x] repeated proofs do not expose an activity feed beyond authorized policy.

Exit condition:

- Adaptive policy has live occupancy evidence without turning bucket metadata
  into public activity telemetry.

### 053-10 Forest Cache Plane

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 6.1, 6.2, 6.3, 7.1, 9.2, and 13.4

- [x] Add `forest_cache.rs` or equivalent cache module under
  `store_internal`.
- [x] Implement required cache layers from the cache contract section.
- [x] Add dirty-set invalidation for terminal, parent, proof, journal, path
  index, and policy-transition caches.
- [x] Add cache verification hooks that recompute sampled entries and fail
  closed on mismatch.
- [x] Add cache memory limits and eviction policy.
- [x] Add cache warmup from durable root rows after journal validation.
- [x] Add cache clear on rollback and proof-version changes.
- [x] Ensure no cache entry stores plaintext secrets or unredacted private
  wallet data.
- [x] Expose cache metrics through storage-owned benchmark and diagnostics
  surfaces.

Files:

- `crates/z00z_storage/src/settlement/hjmt_cache.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/hjmt_proof.rs`
- `crates/z00z_storage/src/settlement/store_roots.rs`
- `crates/z00z_storage/tests/test_forest_cache.rs`
- `crates/z00z_storage/tests/test_cache_recompute.rs`

Tests:

- [x] unchanged bucket root reused after unrelated path update.
- [x] unchanged serial root reused after unrelated serial update.
- [x] unchanged definition root reused after unrelated definition update.
- [x] proof segment cache reused for shared parent paths.
- [x] non-existence default proof cache invalidates after matching insert.
- [x] split/merge/policy-transition cache invalidates affected bucket range
  only.
- [x] rollback clears unpublished cache entries.
- [x] cache mismatch fails closed.
- [x] cache memory limit evicts without changing roots or proofs.

Exit condition:

- Cache reuse is required, measured, and correctness-neutral.

### 053-11 Async Forest Scheduler And Parallel Commit Pipeline

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 6.1, 6.2, 6.3, 9.2, and 13.4

- [x] Add storage-owned scheduler for batch planning, child commits, parent
  commits, proof generation, and policy-transition work.
- [x] Execute independent terminal bucket commits in parallel with bounded
  concurrency.
- [x] Execute proof generation for independent paths in parallel with bounded
  concurrency.
- [x] Execute split, merge, and policy-transition proof work in parallel where
  dependency order permits.
- [x] Keep parent commits deterministic by sorting all child outputs before
  writing parent leaves.
- [x] Route blocking RedB work through a blocking executor.
- [x] Add cancellation and rollback behavior for in-flight batches.
- [x] Add backpressure for oversized work queues.
- [x] Keep sync API behavior stable by routing through one scheduler entrypoint.

Files:

- `crates/z00z_storage/src/settlement/hjmt_scheduler.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/hjmt_store.rs`
- `crates/z00z_storage/src/settlement/hjmt_proof.rs`
- `crates/z00z_storage/src/settlement/hjmt_policy.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/src/settlement/store_rows.rs`
- `crates/z00z_storage/tests/test_async_scheduler.rs`

Tests:

- [x] parallel terminal commits produce same root as serial commits.
- [x] parallel proof generation verifies every proof.
- [x] scheduler respects max concurrency.
- [x] scheduler backpressure rejects or queues deterministically.
- [x] cancellation preserves previous root.
- [x] RedB writes do not run on async worker threads.
- [x] parent root order is stable across task completion permutations.

Exit condition:

- Parallelism is a production implementation path, not only a benchmark
  harness that runs multiple independent stores.

### 053-12 Journal, Recovery, And Durable Policy State

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 7.1, 7.3, 9.3, and 13.2

- [x] Extend `HjmtCommitJournalEntry` with root generation, proof version,
  bucket epoch, policy-bound fee replay count/digest sealing, and durable
  replay-metadata validation boundaries.
- [x] Keep the existing four-state journal lifecycle for adaptive split,
  merge, and policy-transition work because the live child/parent digest
  boundary already reconstructs or rejects the transition deterministically.
- [x] Preserve child-before-parent publication in the HJMT journal.
- [x] Add recovery for interrupted right transitions.
- [x] Add recovery for interrupted fee-envelope acceptance.
- [x] Add recovery for interrupted deletion proof publication.
- [x] Keep non-existence proof cache warmup non-durable; recovery rejects
  persisted state drift without accepting diagnostic cache rows as authority.
- [x] Add recovery for split, merge, and policy-transition interruption points.
- [x] Validate journal digests against durable child rows, parent rows,
  policy-bound replay rows, and pending or active replay metadata before
  recovery accepts the state.
- [x] Fail closed on status regression, generation drift, epoch drift, digest
  mismatch, partial pending checkpoint state, or fee replay drift.

Files:

- `crates/z00z_storage/src/settlement/hjmt_journal.rs`
- `crates/z00z_storage/src/settlement/hjmt_commit.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/src/settlement/redb_backend_helpers.rs`
- `crates/z00z_storage/src/settlement/live_recovery_tests.rs`
- `crates/z00z_storage/tests/test_fee_replay.rs`
- `crates/z00z_storage/tests/test_redb_reload.rs`

Tests:

- [x] interruption before child commit rolls back.
- [x] interruption after child commit validates child digest.
- [x] interruption after parent commit publishes or rolls back safely.
- [x] interruption during right transition preserves or completes state.
- [x] interruption during fee acceptance preserves replay state.
- [x] interruption during split recovers deterministically.
- [x] interruption during merge recovers deterministically.
- [x] interruption during policy transition recovers deterministically.
- [x] tampered generation, epoch, policy-bound replay metadata, or fee digest
  rejects.

Exit condition:

- Recovery never exposes a root whose child roots, policy-transition rows,
  right rows, fee rows, or proof metadata are missing or stale.

### 053-13 RedB Persistence, Reload, Historical Proofs, And Cache Warmup

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 5.4, 7.1, 9.3, and 13.2

- [x] Persist generalized terminal rows with leaf family and generation.
- [x] Persist right transition rows where needed for deletion and historical
  proof verification.
- [x] Persist fee-envelope replay rows separately from terminal leaves.
- [x] Persist adaptive bucket rows by epoch.
- [x] Persist split, merge, and policy-transition proof metadata.
- [x] Persist root rows with generation and epoch.
- [x] Reload path index from committed terminal leaves, not from standalone
  path-index authority.
- [x] Reload cache warm entries only after root, journal, epoch, and digest
  validation.
- [x] Verify historical proofs against retained roots and epochs.
- [x] Do not add a Phase 052 row reader. Phase 053 dev stores initialize
  and reload only HJMT rows.
- [x] Delete compatibility/simple-JMT persistence helpers and row formats from
  live crates once HJMT rows are authoritative. Do not preserve dead reload or
  persist helpers under renamed modules.

Files:

- `crates/z00z_storage/src/settlement/store_rows.rs`
- `crates/z00z_storage/src/settlement/redb_backend_helpers.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/src/settlement/hjmt_cache.rs`
- `crates/z00z_storage/src/settlement/hjmt_proof.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/store_types.rs`
- `crates/z00z_storage/src/settlement/store_codec.rs`
- `crates/z00z_storage/src/error.rs`
- `crates/z00z_storage/src/serialization/artifact.rs`
- `crates/z00z_storage/src/serialization/restore.rs`
- `crates/z00z_storage/tests/test_redb_reload.rs`
- `crates/z00z_storage/tests/test_hjmt_proofs.rs`
- `crates/z00z_storage/tests/test_fee_replay.rs`

Tests:

- [x] reload generalized asset leaves.
- [x] reload right leaves.
- [x] reload fee replay rows.
- [x] reload adaptive bucket epochs.
- [x] reload split/merge/policy-transition rows.
- [x] reject path index drift.
- [x] reject stale cache warm entry.
- [x] historical proof verifies under old epoch.
- [x] historical proof rejects under wrong epoch.
- [x] Phase 052 forest rows are rejected by the Phase 053 dev store with a
  typed unsupported-generation error.

Exit condition:

- Durable storage can reload generalized settlement state and reject every
  drift class before exposing a root.

### 053-14 Checkpoint, Snapshot, Claim Source, Wallet, And Validator Integration

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 2.5, 2.6, 8.2, 8.3, 11, and 12

- [x] Add checkpoint artifacts that bind prior and next
  `SettlementStateRoot`.
- [x] Add snapshot entries for `SettlementLeaf` and proof v2.
- [x] Add claim-source and right-source contracts where rights are involved.
- [x] Ensure wallet code consumes `SettlementLeaf::Asset` through the HJMT API
  and rejects or routes `SettlementLeaf::Right` explicitly.
- [x] Ensure validators verify proof v2 through storage-owned APIs.
- [x] Ensure linked-liability proofs remain path-local or family-local.
- [x] Ensure OnionNet or publication code remains separated from storage
  semantics.
- [x] Add source-shape guardrails preventing downstream raw branch-proof
  decoders.

Files:

- `crates/z00z_storage/src/checkpoint/*`
- `crates/z00z_storage/src/snapshot/*`
- `crates/z00z_runtime/validators/src/*`
- `crates/z00z_wallets/src/*`
- `crates/z00z_storage/tests/test_checkpoint_store_api.rs`
- `crates/z00z_storage/tests/test_downstream_guardrails.rs`

Tests:

- [x] checkpoint accepts valid settlement root transition.
- [x] checkpoint rejects wrong generation.
- [x] checkpoint rejects wrong proof family.
- [x] snapshot proof v2 verifies after reload.
- [x] wallet asset scan works through the HJMT settlement API.
- [x] wallet rejects unrelated right leaf as owned asset.
- [x] validator uses storage proof API and not raw forest layout.
- [x] linked-liability proof does not expose unrelated wallet inventory.

Exit condition:

- Generalized HJMT root and proof semantics are integrated through public
  protocol surfaces without downstream physical authority.

### 053-15 Scenario 1 Production Examples

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 8.2, 8.3, 11, 12, and 13

Scenario 1 must demonstrate the generalized HJMT generation through real
storage APIs. The examples may use deterministic fixture leaves, but they must
exercise production code.

- [x] Add `stage13_hjmt_settlement_examples` config under
  `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`.
- [x] The scenario config must define the HJMT example lane explicitly:

  ```yaml
  stage13_hjmt_settlement_examples:
    enabled: true
    backend_modes: ["generalized", "adaptive"]
    rights_manifest_file: "genesis/genesis_settlement_manifest.json"
    output_dir: "hjmt"
    examples_file: "hjmt/hjmt_settlement_examples.json"
    tamper_report_file: "hjmt/hjmt_tamper_report.json"
    proof_size_report_file: "hjmt/hjmt_proof_size_report.json"
    cache_scheduler_metrics_file: "hjmt/hjmt_cache_scheduler_metrics.json"
    replay_roots_file: "hjmt/hjmt_replay_roots.json"
    expected_right_classes:
      - machine_capability
      - data_access
      - service_entitlement
      - validator_mandate
      - one_time_use
  ```

- [x] Add design-scenario steps that describe generalized settlement examples
  without claiming universal trustless closure beyond implemented code.
- [x] Add design-scenario steps for Stage 1 rights generation, Stage 3/4
  rights preservation or distribution, Stage 11 right-as-asset rejection, and
  Stage 13 HJMT proof/debug artifact verification.
- [x] Add `stage_13_utils/hjmt_examples.rs` or equivalent module.
- [x] Example 1: create a YAML-generated asset as `SettlementLeaf::Asset`
  under `SettlementStateRoot` and verify inclusion proof v2.
- [x] Example 2: load a YAML-generated external or machine capability right,
  create its `RightLeaf`, and verify inclusion proof v2.
- [x] Example 3: attach a valid `FeeEnvelope` to a right transition and prove
  fee support is not ownership.
- [x] Example 4: delete or consume a YAML-generated right and verify deletion
  proof.
- [x] Example 5: prove non-existence for an absent YAML right and reject a
  present-key non-existence proof.
- [x] Example 6: trigger deterministic adaptive split in a fixture hot serial
  and verify split proof.
- [x] Example 7: trigger deterministic merge or HJMT policy transition and
  verify policy-transition proof.
- [x] Example 8: emit cache and async execution metrics for the scenario run.
- [x] Every example must emit a schema-checked HJMT artifact with:
  `root_generation`, `settlement_state_root_hex`, prior/next root when
  applicable, `proof_envelope_version`, `proof_family`, `leaf_family`,
  `settlement_path`, `terminal_id`, `bucket_epoch`, `bucket_policy_id`,
  verifier status, and exact storage API used to verify it.
- [x] Fee examples must emit `fee_envelope_id`, fee domain, transition binding,
  payer or sponsor commitment, expiry, replay status, and a proof result that
  cannot be used as ownership evidence.
- [x] Cache and scheduler examples must emit bounded metrics: cache hit/miss,
  invalidation count, root reuse ratio, proof segment reuse ratio, scheduler
  queue depth, backpressure count, and deterministic parent ordering status.
- [x] Write scenario debug artifacts under a deterministic HJMT output
  directory:
  - `hjmt_settlement_examples.json`;
  - `hjmt_tamper_report.json`;
  - `hjmt_proof_size_report.json`;
  - `hjmt_cache_scheduler_metrics.json`;
  - `hjmt_replay_roots.json`;
  - `genesis_settlement_manifest.json` copied or linked from Stage 1 output.
- [x] Each debug artifact must include `schema_version`, `scenario_id`,
  `stage`, `example_id`, `root_generation`, `backend_mode`, `api_surface`,
  verifier status, and a redacted error object for failures.
- [x] Failure artifacts must name the failing storage API and typed error
  class without logging secrets, proof witness bytes, private wallet keys, or
  unredacted payload contents.
- [x] Add a reload-debug step that reopens the Stage 13 store and re-verifies
  every emitted example artifact against the persisted root.
- [x] Update `runner_verify.rs` to verify HJMT example artifact paths, roots,
  proof families, and status fields.
- [x] Extend `runner_verify.rs` with negative verification fixtures for wrong
  root generation, wrong root bytes, wrong proof family, wrong leaf family,
  wrong terminal path, wrong bucket epoch, stale policy-transition id, tampered
  default commitment, wrong fee transition binding, missing cache metrics, and
  missing scheduler determinism evidence.
- [x] Add scenario-local source-shape checks that reject raw physical layout
  names in the new HJMT example module.
- [x] Keep scenario code on storage-owned APIs. It must not import
  `ForestTreeId`, `TreeId`, `BucketId` as write authority, namespace bytes, or
  RedB row keys.

Files:

- `crates/z00z_simulator/Cargo.toml`
- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml`
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_examples.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_artifacts.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/hjmt_tamper.rs`
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/mod.rs`
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
- `crates/z00z_simulator/tests/test_s7_examples.rs`

Tests:

- [x] scenario example artifact contains `settlement_state_root_hex`.
- [x] scenario artifact schema rejects missing root generation, proof version,
  proof family, leaf family, path, terminal id, bucket epoch, and verifier
  status.
- [x] asset inclusion example verifies proof v2 under `SettlementStateRoot`.
- [x] right inclusion example verifies proof v2.
- [x] fee example proves fee support is separate from right ownership.
- [x] fee replay example rejects after reload.
- [x] deletion example verifies deletion proof.
- [x] absence example verifies absent key and rejects present key.
- [x] stale absence proof rejects after inserting the formerly absent path.
- [x] adaptive split or policy-transition example verifies proof.
- [x] stale split, merge, or policy-transition proof rejects after epoch
  change.
- [x] wallet scan rejects unrelated `RightLeaf` before ownership detection.
- [x] debug artifacts validate against their schemas.
- [x] reload-debug step verifies every emitted HJMT artifact after reopening
  the store.
- [x] redaction test proves debug artifacts do not contain private keys,
  witness bytes, unredacted payload contents, or raw RedB row keys.
- [x] runner verification catches tampered roots, proof family drift, fee
  ownership confusion, and missing cache metrics.

Exit condition:

- `scenario_1` contains production-code HJMT examples with verifier-checked
  artifacts.

### 053-16 Golden Corpus, Property Tests, And Fuzz Seeds

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 3.3, 9.3, and 13

- [x] Build an HJMT-only mixed asset/right corpus. Do not depend on the Phase
  052 asset-only corpus as an oracle.
- [x] Add deterministic operation generators for asset put/delete, right
  create/transfer/revoke/consume/expire, fee accept/reject, absence proofs,
  split, merge, and policy transition.
- [x] Add model oracle for generalized settlement roots independent of the
  production forest implementation.
- [x] Add a negative corpus proving legacy asset root/path/proof inputs reject
  without mutation.
- [x] Add property tests for operation reordering where semantics permit.
- [x] Add state-preservation checks after every reject path.
- [x] Add fuzz seeds for proof decoder malformed bytes, epoch drift, root drift,
  leaf-family drift, fee drift, and policy-transition replay.

Files:

- `crates/z00z_storage/tests/test_golden_corpus.rs`
- `crates/z00z_storage/tests/test_property_corpus.rs`
- `crates/z00z_storage/tests/test_fuzz_seeds.rs`
- `crates/z00z_storage/tests/test_settlement_corpus_support.inc`
- `crates/z00z_core/tests/genesis/test_settlement_corpus.rs`
- `crates/z00z_simulator/tests/test_scenario_settlement.rs`
- `crates/z00z_storage/fuzz/Cargo.toml`

Tests:

- [x] generated mixed operation sequences match model oracle.
- [x] legacy asset-only sequences reject at the HJMT boundary without state
  drift.
- [x] reject paths preserve root and rows.
- [x] malformed proof bytes reject without panic.
- [x] randomized split/merge/policy-transition sequences preserve terminal set.
- [x] generated fee failures do not mutate right leaves.

Exit condition:

- Generalized HJMT has an executable correctness corpus beyond hand-picked
  fixtures.

### 053-17 Benchmarks, Metrics, And Performance Gates

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 6.3, 9.2, 13.4
- `crates/z00z_storage/benches/assets/assets_benches.md`

- [x] Extend benchmark harnesses for generalized roots, right leaves, fee
  envelopes, deletion proofs, non-existence proofs, adaptive split/merge, and
  policy-transition proofs.
- [x] Add measured search/read lanes for full `SettlementPath`, terminal-id
  lookup, path-index lookup, definition-filtered list, serial-filtered list,
  right-class-filtered list, paginated list, absent-path lookup, post-reload
  lookup, cold cache, and warm cache.
- [x] Add measured insert lanes for single asset, single right, fee-supported
  right, broad definitions, hot definition, hot serial, hot bucket,
  mixed asset/right batch, duplicate reject, and rejected fee-supported
  transition.
- [x] Add measured delete/prune lanes for single asset delete, batch delete,
  right consume, right revoke, expiry cleanup, empty bucket prune, empty
  serial prune, empty definition prune, and missing delete reject.
- [x] Add proof-generation and proof-verification lanes for inclusion,
  deletion, non-existence, split, merge, policy transition, historical proof,
  shared-parent batch proof, mixed inclusion/non-existence batch proof, and
  malformed/tampered proof rejection.
- [x] Add proof-size lanes that record serialized byte size and component
  breakdown for root binding, definition segment, serial segment, bucket
  segment, terminal segment, default commitment, policy-transition segment,
  and fee-support binding where present.
- [x] Add cold-cache and warm-cache lanes for search, insert, delete, proof
  generation, proof verification, split, merge, policy transition, reload, and
  scenario_1 examples.
- [x] Add dirty-small, dirty-hot, dirty-wide, proof-heavy,
  policy-transition-heavy, reload, and recovery workloads.
- [x] Add async scheduler lanes that use one production store and production
  scheduler, not multiple independent store clones as the only concurrency
  signal.
- [x] Add HJMT-specific adaptive lanes for split trigger, split hysteresis
  no-op, merge trigger, merge hysteresis no-op, policy transition,
  prior-policy historical proof, stale-proof rejection, and
  occupancy-evidence cost.
- [x] Record cache hit/miss/invalidation and root/proof reuse ratios.
- [x] Record p50/p95/p99 and throughput for search/read, insert,
  delete/prune, commit, proof generation, proof verification, split, merge,
  policy transition, reload, and recovery.
- [x] Record memory, allocation, CPU, RedB I/O, encoded-byte totals, scheduler
  queue depth, backpressure events, and blocking RedB wait time.
- [x] Keep measured numbers as evidence, not protocol constants.
- [x] Add regression gate comparing fixed-bucket HJMT and adaptive HJMT on
  identical machine/workload baselines.
- [x] Require benchmark evidence before switching generalized HJMT to default.

Files:

- `crates/z00z_storage/Cargo.toml`
- `crates/z00z_storage/benches/assets/shard.rs`
- `crates/z00z_storage/benches/assets/nested.rs`
- `crates/z00z_storage/benches/assets/hjmt.rs`
- `crates/z00z_storage/benches/assets/proofs.rs`
- `crates/z00z_storage/benches/assets/assets_benches.md`
- `crates/z00z_storage/scripts/run_storage_assets_bench.py`
- `crates/z00z_storage/scripts/run_hjmt_bench.py`
- `crates/z00z_storage/outputs/assets/*` as ignored runtime output

Tests and commands:

- [x] `cargo bench -p z00z_storage --bench assets_shard --no-run`
- [x] `cargo bench -p z00z_storage --bench assets_nested --no-run`
- [x] `cargo bench -p z00z_storage --bench assets_hjmt --no-run`
- [x] `cargo bench -p z00z_storage --bench assets_proofs --no-run`
- [x] run measured fixed-bucket HJMT baseline.
- [x] run measured adaptive HJMT baseline.
- [x] run measured generalized Phase 053 cold-cache baseline.
- [x] run measured generalized Phase 053 warm-cache baseline.
- [x] run measured search/read workload.
- [x] run measured insert workload.
- [x] run measured delete/prune workload.
- [x] run measured adaptive policy-transition workload.
- [x] run measured proof-generation workload.
- [x] run measured proof-verification workload.
- [x] run measured proof-size workload with byte-size summary by proof family.
- [x] run measured cache reuse and invalidation workload.
- [x] run measured async scheduler and backpressure workload.
- [x] run measured reload and recovery workload.
- [x] run measured scenario_1 HJMT example workload.
- [x] record exact commands and outputs in `assets_benches.md`.

Exit condition:

- Performance claims are backed by measured search, insert, delete, proof
  generation, proof verification, proof-size, cache, async, adaptive policy
  transition, reload, recovery, and scenario_1 evidence.

### 053-18 Documentation, API Examples, And Hard-Cutover Notes

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` all implementation-relevant sections

- [x] Update storage documentation to show the live Phase 053 root and proof
  vocabulary.
- [x] Add API examples for HJMT asset creation, right creation, right
  transition, fee support, deletion proof, non-existence proof, split proof,
  merge proof, and policy-transition proof.
- [x] Add operator notes for enabling generalized HJMT mode, adaptive buckets,
  cache limits, scheduler limits, and metrics.
- [x] Add dev hard-cutover notes: old storage artifacts are unsupported, new
  dev data must be regenerated from rights-enabled YAML/genesis inputs, and no
  conversion shim is provided.
- [x] Document that superseded storage code is removed from live crates instead
  of being kept under `legacy`, `old`, `compat`, `v1`, or `v2` implementation
  names.
- [x] Document privacy rules for occupancy metadata.
- [x] Document that old asset mode is not a live Phase 053 surface.
- [x] Update planning state only after tests and benchmarks are green.

Files:

- `docs/tech-papers/Z00Z-HJMT-Design.md`
- `crates/z00z_storage/src/settlement/README.MD`
- `crates/z00z_storage/src/settlement/root-types.md`
- `crates/z00z_storage/tests/test_readme_examples.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `.planning/STATE.md`
- `.planning/phases/053-HJMT-Backend/053-18-SUMMARY.md`

Tests:

- [x] docs source-shape test rejects stale future-only language for Phase 053
  live contracts.
- [x] docs source-shape test preserves privacy and fee/right separation notes.
- [x] API examples compile or are covered by integration tests.

Exit condition:

- Documentation matches live production behavior and does not overclaim beyond
  verified code.

### 053-19 Closeout And Production Default Gate

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` section 9.3

- [x] Run the full verification order below.
- [x] Run focused storage tests after each storage slice.
- [x] Run simulator scenario tests after scenario changes.
- [x] Run benchmark compile gates before claiming performance coverage.
- [x] Run measured benchmark lanes before switching defaults.
- [x] Run review loops until no significant issue remains.
- [x] Verify `crates/z00z_core/Cargo.toml`,
  `crates/z00z_storage/Cargo.toml`, and `crates/z00z_simulator/Cargo.toml`
  expose every new test, bench, binary, or feature required by this phase.
- [x] Switch default backend or root generation only after root, proof,
  recovery, policy-transition, cache, async, checkpoint, simulator, and
  benchmark gates are green.
- [x] Record closeout evidence in `053-SUMMARY.md`.

Exit condition:

- Phase 053 is production-ready and the generalized HJMT generation can be the
  live target without relying on future-only claims.

### 053-20 Legacy Storage Purge And Dead Code Cleanup

Spec references:

- `docs/tech-papers/Z00Z-HJMT-Design.md` sections 9.3, 12, and 14

- [x] Remove superseded simple-JMT and compatibility-storage code from live
  crates instead of renaming it to `legacy`, `old`, `compat`, `v1`, or `v2`.
- [x] Delete production `AssetBackendMode` branches and parser names that keep
  compatibility, forest, or dual-verify storage lanes alive after the HJMT
  cutover.
- [x] Delete compatibility projection builders, compatibility proof-envelope
  code, flat pre-state row helpers, dead reload helpers, and dead serialization
  adapters once HJMT replacements are landed.
- [x] Delete unused tests, benches, docs, fixtures, and helper modules whose
  only purpose was preserving old storage behavior.
- [x] Run grep-backed source audits proving no live crate still exposes the old
  simple-JMT or compatibility storage lane except in explicitly historical
  archived tests or notes.
- [x] Record the removed modules, removed modes, and removed docs/tests in
  `053-SUMMARY.md`.

Files:

- `crates/z00z_storage/src/settlement/hjmt_config.rs`
- `crates/z00z_storage/src/settlement/store.rs`
- `crates/z00z_storage/src/settlement/redb_backend_helpers.rs`
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs`
- `crates/z00z_storage/src/serialization/build.rs`
- `crates/z00z_storage/src/serialization/artifact.rs`
- `crates/z00z_storage/src/serialization/restore.rs`
- `crates/z00z_storage/src/serialization/mod.rs`
- `crates/z00z_storage/tests/test_live_guardrails.rs`
- `crates/z00z_storage/tests/test_settlement_root.rs`
- `docs/tech-papers/Z00Z-HJMT-Design.md`
- `.planning/phases/053-HJMT-Backend/053-SUMMARY.md`

Tests:

- [x] source-shape audit rejects live `AssetBackendMode::Compatibility`.
- [x] source-shape audit rejects live `AssetBackendMode::DualVerify`.
- [x] source-shape audit rejects production compatibility projection code.
- [x] source-shape audit rejects newly parked storage implementations under
  `legacy`, `old`, `compat`, `v1`, or `v2` names.
- [x] public crate surfaces no longer export dead compatibility/simple-JMT
  storage helpers.

Exit condition:

- No live production crate retains the old simple-JMT or compatibility storage
  implementation as dead weight. Superseded code is removed, not renamed and
  parked.

## 🧪 Required Test Matrix

The phase is not complete until these tests exist and pass where applicable:

- root generation success and wrong-generation rejection;
- old asset root/path/proof rejection at the HJMT boundary;
- asset inclusion proof v2;
- right inclusion proof v2;
- deletion proof success and tamper matrix;
- non-existence proof success and tamper matrix;
- present-key absence rejection;
- default commitment transcript binding;
- fee envelope success and failure matrix;
- fee replay across reload;
- right transition create, transfer, consume, revoke, expire, and challenge;
- HJMT mixed asset/right corpus;
- generalized mixed asset/right corpus;
- assets/genesis YAML parser accepts assets plus rights;
- devnet, devnet-small, testnet, and mainnet genesis configs generate
  deterministic rights;
- genesis writes rights artifacts and combined settlement manifest;
- Cargo manifests expose all new Phase 053 tests, benches, and feature gates;
- storage ingestion creates `RightLeaf` entries from generated genesis rights;
- split proof success and tamper matrix;
- merge proof success and tamper matrix;
- policy-transition proof success and tamper matrix;
- historical proof by bucket epoch;
- stale proof rejection;
- path-index rebuild after generalized reload;
- RedB drift rejection for right rows, fee rows, policy-transition rows, and
  cache warm rows;
- journal recovery for every mutation and policy-transition stage;
- cache hit/miss/invalidation correctness;
- cache mismatch fail-closed behavior;
- async scheduler determinism;
- scheduler cancellation rollback;
- source-shape audits reject live compatibility/simple-JMT storage helpers,
  backend modes, and parked legacy implementations;
- downstream physical-layout authority guardrails;
- scenario_1 generalized HJMT examples;
- benchmark compile and measured evidence gates.

## 🧪 Test Tier Contract

Every PLAN derived from this TODO must state which tier it changes and must
land the matching tests in the same slice.

| Tier | Required coverage |
| --- | --- |
| Unit | canonical encodings, typed config parsing, path derivation, terminal-id derivation, right/fee validation, bucket policy decisions, cache-key construction, scheduler ordering, proof-family reject classes |
| Integration | core YAML -> genesis corpus, genesis corpus -> storage ingestion, storage put/get/list/prove/delete, RedB persist/reload, checkpoint/snapshot binding, wallet and validator semantic API use |
| End-to-end | scenario_1 rights-enabled genesis, typed rights lane or preserved fixture lane, wallet scan rejection of rights as assets, Stage 13 proof examples, reload-debug verification, runner tamper rejection |
| Property/fuzz | mixed asset/right operation sequences, proof decoder malformed bytes, stale epoch, root drift, leaf-family drift, fee replay, policy-transition replay, cache mismatch |
| Benchmark | search, insert, delete, proof generation, proof verification, proof size, adaptive policy transition, cache reuse, async scheduler, reload, recovery, scenario_1 workload |

No task may be marked complete with only unit tests when it changes durable
storage behavior, public proof behavior, YAML generation, or scenario-visible
contracts.

## 🎬 Scenario 1 Example Matrix

| Scenario surface | Required Phase 053 validation |
| --- | --- |
| Stage 1 genesis | `genesis_config_devnet_small.yaml` generates assets plus rights, writes rights artifacts, writes combined settlement manifest, and records right counts. |
| Stage 4 prep witness | Existing asset witness path is replaced by HJMT `SettlementLeaf::Asset` proof v2 checks. |
| Stage 3/4 claim lanes | Generated rights are either distributed through a typed rights lane or explicitly preserved as non-wallet-owned settlement fixtures; they are never silently dropped. |
| Stage 6 bundle handoff | Fee-envelope support can be attached to a generalized settlement transition without proving ownership. |
| Stage 11 wallet scan | Wallet scan verifies storage proof first, checks leaf family before ownership detection, and rejects unrelated `RightLeaf` as wallet-owned asset. |
| Stage 13 storage replay | Replay binds generated asset/right leaves, `SettlementStateRoot`, proof v2, deletion proof, non-existence proof, split proof, merge proof, policy-transition proof, fee replay rows, and cache warmup evidence. |
| Stage 13 HJMT examples | Deterministic examples use generated YAML rights and emit right inclusion, fee support, deletion, absence, split, merge, policy transition, cache, async, and proof-size evidence artifacts. |
| Stage 13 tamper fixtures | Fixture mutations cover root generation drift, root byte drift, proof family drift, leaf family drift, terminal path drift, default commitment drift, bucket epoch drift, stale policy-transition id, fee binding drift, and missing cache or scheduler metrics. |
| `runner_verify` | Tampering with root generation, proof family, fee binding, bucket epoch, generated right count, right class, holder/control fixture, proof bytes, proof size summary, or cache metrics must fail verification. |

Required scenario commands:

```bash
cargo test -p z00z_simulator --release --features wallet_debug_tools scenario_1
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_tools
```

The scenario must be run in generalized HJMT mode. If an adaptive HJMT mode is
separate from the fixed-bucket HJMT mode, it must also run.

## ✅ Verification Order

Run these commands as applicable during implementation and closeout:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test -p z00z_core --release --features test-fast --test assets_tests --test genesis_tests
cargo test -p z00z_storage --release --features test-fast --features wallet_debug_tools
cargo test -p z00z_simulator --release --features wallet_debug_tools scenario_1
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_tools
cargo bench -p z00z_storage --bench assets_shard --no-run
cargo bench -p z00z_storage --bench assets_nested --no-run
cargo bench -p z00z_storage --bench assets_hjmt --no-run
cargo bench -p z00z_storage --bench assets_proofs --no-run
cargo test --release --features test-fast --features wallet_debug_tools
```

Run scenario_1 at minimum in these live-mode checks:

```bash
Z00Z_SETTLEMENT_BACKEND_MODE=hjmt cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_tools
Z00Z_SETTLEMENT_BACKEND_MODE=compatibility cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_tools
```

The live mode must accept `hjmt` and reject stale aliases such as
`compatibility`, `forest`, and `dual-verify`.

If a code slice touches public Rust APIs or docs, also run:

```bash
cargo doc --no-deps
```

If a slice changes formatting-sensitive code, run:

```bash
cargo fmt
```

If a slice changes lint-sensitive code, run:

```bash
cargo clippy --all-targets --all-features
```

## 🚫 Explicit Non-Goals

- Do not expose physical tree layout as public authority.
- Do not turn `SettlementStateRoot` into a universal protocol root outside the
  storage-family settlement contract.
- Do not make `RightLeaf` a generic database object.
- Do not make `FeeEnvelope` ownership evidence.
- Do not accept placeholder proofs.
- Do not treat cache entries as durable truth.
- Do not switch production defaults before the full gate is green.
- Do not hide benchmark failures behind planning notes.
- Do not keep legacy storage, aliases, shims, legacy row readers, or adapter
  facades in the Phase 053 live dev path.

## 💥 Failure Conditions

The phase must be rejected if any of the following happens:

- `SettlementStateRoot` is only an alias for `AssetStateRoot`.
- `RightLeaf` is only an alias or wrapper for `AssetLeaf`.
- `FeeEnvelope` appears inside `RightLeaf` as right meaning.
- deletion or non-existence proof returns fake success.
- adaptive bucket split or merge changes roots without split/merge proof.
- policy transition changes bucket policy without proof.
- a stale proof verifies under the wrong epoch.
- a cache mismatch is repaired silently.
- a cache entry becomes root authority.
- async execution changes root determinism.
- RedB reload accepts drift in terminal, right, fee, policy-transition, or
  journal rows.
- legacy storage code remains in the live dev path.
- `AssetStateRoot`, `AssetPath`, or Phase 052 proof bytes are accepted by the
  Phase 053 runtime instead of rejected as unsupported generation.
- a legacy row reader, compatibility reader, alias, shim, or old asset adapter
  is needed for scenario_1 to pass.
- compatibility/simple-JMT storage code remains compiled into live crates under
  active or renamed lanes such as `compat`, `legacy`, `old`, `v1`, or `v2`.
- devnet, devnet-small, testnet, or mainnet genesis remains asset-only in
  generalized Phase 053 mode.
- `assets_config.yaml` remains asset-only or `assets_config_schema.yaml` does
  not require `rights:` in generalized Phase 053 mode.
- rights are encoded as fake assets, asset metadata, or an `AssetClass::Right`
  shortcut instead of a separate rights config and `RightLeaf` family.
- generated rights are ignored by storage ingestion or scenario_1.
- scenario_1 uses hardcoded rights while YAML-generated rights are absent or
  unverified.
- new tests, benches, or Phase 053 features are described in TODO but not
  wired through the relevant Cargo manifest.
- scenario_1 decodes raw physical proof layout.
- benchmark evidence is missing for search, insert, delete, proof generation,
  proof verification, proof sizes, cache, async, adaptive policy transition,
  reload, or recovery claims.

## ✅ Completion Gate

Phase 053 is complete only when:

- generalized HJMT root, leaf, proof, fee, adaptive bucket, cache, async,
  recovery, and persistence code is live;
- all old Phase 052 future-only candidates are implemented as HJMT live code
  or replaced by stronger HJMT-only guardrails;
- no compatibility reader, legacy row reader, alias, shim, or old asset adapter
  remains in the live dev storage path;
- no compatibility/simple-JMT storage lane or parked dead implementation
  remains in live crates under `compat`, `legacy`, `old`, `v1`, or `v2`
  names;
- `SettlementStateRoot` is checkpoint-bound and proof-bound;
- `RightLeaf` and `FeeEnvelope` are live but separate contracts;
- devnet, devnet-small, testnet, and mainnet YAML configs generate assets plus
  rights through validated schemas and production loaders;
- `assets_config.yaml` and `assets_config_schema.yaml` generate and validate
  assets plus rights as canonical repository example inputs;
- generated rights are persisted, exported, reported, ingested by storage, and
  used by scenario_1 HJMT examples;
- inclusion, deletion, and non-existence proofs verify fail-closed;
- split, merge, and policy-transition proofs verify fail-closed;
- occupancy metadata is privacy-reviewed and not an activity feed;
- caches reuse unchanged roots and proof segments with correct invalidation;
- async scheduler improves production execution without nondeterminism;
- reload and recovery reject every tampered durable row class;
- scenario_1 includes production-code generalized HJMT examples;
- benchmark evidence records search, insert, delete, proof generation, proof
  verification, proof sizes, cache, async, adaptive policy transition, reload,
  recovery, and scenario_1 behavior;
- docs and `.planning/STATE.md` match the live code.

## 🔍 Doublecheck Notes

This TODO was built from local repository evidence only. The key verified local
facts are:

- Phase 052 live code has `AssetBackendMode::Forest` and `DualVerify` behind
  `AssetStore`.
- Phase 052 live code has fixed `BucketPolicy`, `BucketId`, `BucketRootLeaf`,
  forest journal, forest proof envelope, and RedB recovery.
- Phase 052 proof code rejects deletion and non-existence proof families as
  unsupported.
- Existing Phase 052 guardrails block `SettlementStateRoot`, `RightLeaf`,
  `FeeEnvelope`, adaptive bucket types, and policy-transition proof types
  under old future-only names.
- Scenario 1 currently consumes storage through semantic roots, proof blobs,
  snapshots, and checkpoint replay, not physical tree layout.
- Current core asset and devnet genesis YAML files contain `assets:` but no
  `rights:` section.
- Current genesis config parsing stores `assets: Vec<AssetConfigEntry>` and
  has no typed rights config family.
- Current scenario_1 Stage 1 config points at
  `genesis_config_devnet_small.yaml`, not a hyphenated devnet-small file.
- Current storage backend mode parser recognizes compatibility, forest, and
  dual-verify; Phase 053 must add a generalized mode or intentionally name an
  equivalent production generalized mode.

Any future PLAN files derived from this TODO must cite exact code and test
anchors before implementation starts.
