# Phase 053: HJMT Backend - Context

**Gathered:** 2026-05-29
**Status:** ready-for-execution-planning
**Source:** PRD Express Path (`.planning/phases/053-HJMT-Backend/053-TODO.md`)

<domain>

## Phase Boundary

Phase 053 is a dev hard cutover from the Phase 052 asset-centric HJMT runtime to the production generalized settlement backend. The live backend must expose `SettlementStateRoot`, `SettlementPath`, `TerminalId`, `SettlementLeaf`, `RightLeaf`, `FeeEnvelope`, proof envelope generation 2, live deletion and non-existence proofs, adaptive bucket policy proofs, occupancy privacy evidence, production cache and scheduler machinery, RedB and journal durability, downstream integration, scenario examples, benchmark evidence, documentation, and legacy storage purge.

The phase must reuse `.planning/phases/053-HJMT-Backend/` as the only Phase 053 directory. `053-TODO.md` is the canonical PRD and the numbered plan packet preserves its 20 ordered implementation slices.

</domain>

<decisions>

## Implementation Decisions

### D-01: Live-contract guardrails
- Replace Phase 052 future-only guardrails with Phase 053 guardrails that require live exports for `SettlementStateRoot`, `RightLeaf`, `FeeEnvelope`, `AdaptiveBucket`, `BucketEpoch`, `SplitProof`, `MergeProof`, and `PolicyTransitionProof`, while continuing to block physical-layout authority leakage.

### D-02: Settlement root generation
- `SettlementStateRoot` and `RootGeneration::SettlementV1` become the canonical storage-family settlement root generation for checkpoint, proof, journal, and store binding. Mixed-generation and downgrade paths must fail closed.

### D-03: Settlement terminal contracts
- `SettlementPath`, `TerminalId`, `SettlementLeaf`, and `RightLeaf` become live terminal contracts. `RightLeaf` is narrow and typed, and it must not become a fee container, wallet authority, legal-prose object, workflow record, or generic key-value object.

### D-04: FeeEnvelope separation
- `FeeEnvelope` is a separate processing-support contract for payer, sponsor, budget, fee domain, expiry, nonce, replay protection, transition binding, and failure semantics. It must not prove ownership, right validity, or wallet control.

### D-05: Store API cutover
- Replace asset-centric store/runtime APIs with generalized HJMT settlement APIs. This is a hard cutover: no old-storage conversion lane, no compatibility adapter lane, and no dual default mode are part of the green state.

### D-06: Core YAML and genesis settlement corpus
- Add validated `rights:` support to core asset and genesis YAML, deterministic rights generation, `RightsConfigEntry`, `GenesisRightsConfig`, and `GenesisSettlementCorpus`. Asset-only canonical genesis examples are invalid for Phase 053.

### D-07: Proof envelope generation 2
- Storage owns proof envelope generation 2 for inclusion, deletion, and non-existence. Non-existence proofs bind root generation, path index, default commitment, proof family, proof version, and journal checkpoint.

### D-08: Adaptive bucket policy proofs
- Adaptive buckets are live in Phase 053 with `BucketEpoch`, split proofs, merge proofs, policy transition proofs, rollback rules, recovery rules, and verifier-owned old/new policy binding.

### D-09: Occupancy privacy
- Occupancy evidence must be privacy-reviewed and bucket-bounded. Proof-visible metadata must not expose exact global leaf counts, raw activity deltas, or deterministic user-level timing signals.

### D-10: Forest cache plane
- Add private cache layers for stable roots, parent leaves, terminal encodings, proof segments, journal digests, policy transition evidence, and reload warmup. Cache hits must be recomputable and fail closed.

### D-11: Forest scheduler
- Add a bounded async/parallel scheduler for child commits, proof generation, policy-transition work, and warm reload without nondeterministic roots or unbounded RedB blocking.

### D-12: Journal and durable policy state
- Journal rows must bind root generation, proof version, bucket epoch, policy transition id, fee replay digest, and cache digest. Recovery must reject partial, downgraded, or mismatched rows.

### D-13: RedB reload and historical proofs
- RedB persistence must store generalized settlement rows, reload them into the same semantic state, reject Phase 052 rows with typed unsupported-generation errors, support historical proofs, and warm cache rows safely.

### D-14: Downstream integration
- Checkpoint, snapshot, claim-source, wallet, validator, runtime, and simulator callers must use semantic settlement APIs. Downstream code must not depend on tree ids, bucket ids, namespace bytes, branch ordering, or backend roots as authority.

### D-15: Scenario 1 examples
- `scenario_1` must include production settlement examples, tamper reports, proof-size reports, cache/scheduler metrics, replay roots, and genesis settlement manifest evidence.

### D-16: Corpus, property, and fuzz coverage
- Add golden corpus, property tests, and fuzz seeds for settlement leaves, rights, fees, proofs, adaptive buckets, reload, and downstream rejection behavior.

### D-17: Benchmarks and metrics
- Add benchmark and metrics evidence for fixed/adaptive buckets, proof families, cache hits/misses, scheduler throughput, reload warmup, and downstream scenario performance.

### D-18: Documentation and API examples
- Update API docs, root taxonomy docs, JMT design status, examples, and hard-cutover notes so docs match the live generalized settlement backend and do not preserve Phase 052 as a runtime story.

### D-19: Closeout and default gate
- Close the phase only after the canonical verification order is green, the generalized backend is default, and no Phase 052 future-only or compatibility claims remain in active docs or tests.

### D-20: Legacy storage purge
- Remove superseded compatibility/simple-JMT storage tails, compatibility projection, legacy backend modes, old proof families, stale serialization projections, and dead docs/tests after generalized tests are green.

### D-21: Mandatory verification order
- Every Rust or test-affecting `<task type="auto">` must run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` first as the fail-fast gate, stop/fix/rerun on failure, then run `cargo test --release --features test-fast --features wallet_debug_dump` when relevant, then run `/GSD-Review-Tasks-Execution` in YOLO mode at least three times and continue until at least two consecutive runs show no significant issues.

### D-22: Exact TODO bullet coverage
- Every checklist bullet, nested detail, acceptance bullet, test bullet, non-goal, failure condition, completion-gate item, and verification-order item in `053-TODO.md` is normative for this phase. Each `053-XX-PLAN.md` owns the complete matching `053-XX` TODO subsection, not only the summarized bullets copied into the plan. Executors must read the matching TODO subsection before editing and must treat any omitted detail in the plan prose as still required.

### D-23: No duplicate or parallel implementation layer
- Phase 053 must extend or replace the existing repository seams in place. It must not duplicate the existing codebase logic, fork a parallel storage/proof/genesis/simulator layer, park old runtime lanes under new names, create compatibility readers, or introduce a second authority plane. Any genuinely new module is allowed only when the existing codebase has no owning seam for that responsibility, and the plan must label it as proposed while wiring it through existing public entrypoints.

### D-24: Crypto and security invariants
- Settlement roots, proof envelopes, rights, fee envelopes, adaptive bucket proofs, journals, RedB rows, cache digests, scheduler batches, checkpoint metadata, scenario artifacts, and benchmark metrics must use canonical serialization, explicit domain separation, root-generation binding, downgrade rejection, replay protection, typed fail-closed errors, and storage-owned verifier APIs. Downstream crates must not decode raw proof internals, treat debug metrics as authority, expose raw occupancy counters, or leak private keys, seeds, witness data, payload plaintext, or unredacted private material in artifacts.

### D-25: Source corpus path resolution
- `053-TODO.md` records the Phase 052 source corpus under `.planning/phases/052-HJMT-Backend/...`. In the current worktree, those same Phase 052 files exist under `.planning/phases/000/052-HJMT-Backend/...`. Executors must read the existing current-worktree files under `.planning/phases/000/052-HJMT-Backend/...` when the original `053-TODO.md` path is missing, without creating duplicate phase directories or rewriting the user's move/delete state.

### D-26: Full TODO bullet-class coverage
- All 813 dash-list bullets currently present in `053-TODO.md` are normative, not only the 396 unchecked implementation checklist bullets under `053-01` through `053-20`. This includes source-doc bullets, nested field lists, named examples, artifact lists, performance/cache/benchmark matrix bullets, test-tier bullets, Scenario 1 matrix bullets, command bullets, explicit non-goals, failure conditions, completion-gate bullets, and doublecheck notes. If an executor finds a `053-TODO.md` bullet that is not explicitly copied into a plan, D-22 and D-26 still make that bullet mandatory through the matching plan owner or the applicable global section.

### the agent's Discretion
- Exact internal helper names, module split within the already identified storage/core/simulator crates, additional focused regression tests, and benchmark sampling mechanics are implementation choices as long as they preserve the decisions above and the repository Design Foundation.

</decisions>

<canonical_refs>

## Canonical References

Downstream agents MUST read these before implementing the relevant plans.

### Phase authority
- `.planning/phases/053-HJMT-Backend/053-TODO.md` - canonical Phase 053 PRD, implementation order, acceptance definitions, non-goals, failure conditions, and verification order.
- `.planning/ROADMAP.md` - active Phase 053 goal, derived requirements, plan list, and phase boundaries.
- `.planning/STATE.md` - current active planning state and prior decisions.

### HJMT design and predecessor phases
- `docs/tech-papers/Z00Z-HJMT-Design.md` - topology, root semantics, right/fee separation, proof-family rules, cache/performance targets, privacy constraints, and rollout guidance.
- `.planning/phases/000/051-HJMT-Facade/051-TODO.md` - facade/root vocabulary and downstream authority boundaries preserved from Phase 051.
- `.planning/phases/052-HJMT-Backend/052-TODO.md` - asset-centric Phase 052 backend scope and evidence.
- `.planning/phases/052-HJMT-Backend/052-08-PLAN.md` - adaptive buckets were future-only in Phase 052 and are live in Phase 053.
- `.planning/phases/052-HJMT-Backend/052-09-PLAN.md` - occupancy privacy was future-only in Phase 052 and is live in Phase 053.
- `.planning/phases/052-HJMT-Backend/052-10-PLAN.md` - generalized settlement root was future-only in Phase 052 and is live in Phase 053.
- `.planning/phases/052-HJMT-Backend/052-11-PLAN.md` - `RightLeaf` and `FeeEnvelope` were future-only in Phase 052 and are live in Phase 053.
- `.planning/phases/000/052-HJMT-Backend/052-TODO.md` - current-worktree location for the Phase 052 source corpus when the original `053-TODO.md` path is missing.
- `.planning/phases/000/052-HJMT-Backend/052-08-PLAN.md` - current-worktree adaptive bucket future-candidate source.
- `.planning/phases/000/052-HJMT-Backend/052-09-PLAN.md` - current-worktree occupancy privacy future-candidate source.
- `.planning/phases/000/052-HJMT-Backend/052-10-PLAN.md` - current-worktree generalized settlement root future-candidate source.
- `.planning/phases/000/052-HJMT-Backend/052-11-PLAN.md` - current-worktree `RightLeaf` and `FeeEnvelope` future-candidate source.

### Current storage anchors

- `crates/z00z_storage/tests/test_live_guardrails.rs` - live export, doc-drift, and source-shape guardrails for Phase 053 contracts.
- `crates/z00z_storage/src/settlement/mod.rs` - public storage generalized-settlement export surface.
- `crates/z00z_storage/src/settlement/types_identity.rs` - current settlement root/path/bucket identity types.
- `crates/z00z_storage/src/settlement/types_record.rs` - current settlement record/root/proof types.
- `crates/z00z_storage/src/settlement/proof.rs` - current proof envelope and proof-family enforcement.
- `crates/z00z_storage/src/settlement/store.rs` - current public settlement store API.
- `crates/z00z_storage/src/settlement/hjmt_config.rs` - current backend mode selection and stale-name rejection.
- `crates/z00z_storage/src/settlement/hjmt_commit.rs` - current child-before-parent commit path.
- `crates/z00z_storage/src/settlement/hjmt_journal.rs` - current HJMT forest journal rows.
- `crates/z00z_storage/src/settlement/hjmt_policy.rs` - current bucket policy logic.
- `crates/z00z_storage/src/settlement/hjmt_proof.rs` - current HJMT forest proof production.
- `crates/z00z_storage/src/settlement/hjmt_store.rs` - current HJMT forest store implementation.
- `crates/z00z_storage/src/settlement/redb_backend_hjmt.rs` - current durable HJMT forest RedB path.
- `crates/z00z_storage/src/settlement/redb_backend_state.rs` - current durable state rows.
- `crates/z00z_storage/src/serialization/build.rs` - current settlement-native serialization builder rooted on `store.settlement_root()`.

### Current core/genesis anchors

- `crates/z00z_core/src/assets/assets_config.yaml` - current canonical asset example config.
- `crates/z00z_core/src/assets/assets_config_schema.yaml` - current asset config schema.
- `crates/z00z_core/src/assets/assets_config.rs` - current asset config types.
- `crates/z00z_core/src/assets/assets_config_load.rs` - current asset config loader.
- `crates/z00z_core/src/genesis/genesis_config.rs` - current asset-only genesis parser.
- `crates/z00z_core/src/genesis/genesis_config_schema.yaml` - current genesis schema.
- `crates/z00z_core/src/genesis/genesis_config_devnet.yaml` - current canonical devnet genesis config.
- `crates/z00z_core/src/genesis/genesis_config_devnet_small.yaml` - current small devnet genesis config used by scenario 1.
- `crates/z00z_core/src/genesis/genesis_config_testnet.yaml` - current testnet genesis config.
- `crates/z00z_core/src/genesis/genesis_config_mainnet.yaml` - current mainnet genesis config.
- `crates/z00z_core/src/genesis/genesis_derivation.rs` - current deterministic genesis derivation.
- `crates/z00z_core/src/genesis/genesis_accumulator.rs` - current asset-only genesis accumulator.
- `crates/z00z_core/src/genesis/genesis_run.rs` - current genesis export/run path.
- `crates/z00z_core/src/genesis/genesis_verification.rs` - current genesis verification.

### Current simulator and downstream anchors

- `crates/z00z_simulator/src/scenario_1/scenario_config.yaml` - current scenario 1 config, including genesis input.
- `crates/z00z_simulator/src/scenario_1/scenario_design.yaml` - scenario design contract that must describe Stage 1 rights generation, Stage 3/4 claim or publish treatment, Stage 11 right rejection, and Stage 13 settlement examples.
- `crates/z00z_simulator/src/scenario_1/stage_13.rs` - current scenario 1 storage replay stage.
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/flow.rs` - current Stage 13 flow wiring.
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/storage.rs` - current Stage 13 storage helper.
- `crates/z00z_simulator/src/scenario_1/stage_13_utils/tamper.rs` - current Stage 13 tamper helper.
- `crates/z00z_storage/src/checkpoint` - checkpoint callers that must bind semantic settlement roots.
- `crates/z00z_storage/src/snapshot` - snapshot callers that must bind semantic settlement roots.
- `crates/z00z_wallets` - wallet callers that must reject `RightLeaf` as a spendable asset.

</canonical_refs>

<todo_bullet_coverage>

## TODO Bullet Coverage Contract

The numbered plans own every bullet from the matching `053-TODO.md` implementation section:

| TODO section | Plan owner | Coverage rule |
| --- | --- | --- |
| `053-01` | `053-01-PLAN.md` | All guardrail, source-shape, alias, shim, adapter, docs, and state-update bullets are mandatory. |
| `053-02` | `053-02-PLAN.md` | All root generation, checkpoint, proof binding, downgrade, mixed-generation, reload, and production-mode bullets are mandatory. |
| `053-03` | `053-03-PLAN.md` | All `TerminalId`, `SettlementPath`, `SettlementLeaf`, `RightLeaf`, `serde(deny_unknown_fields)`, domain separation, right taxonomy, transition, fee-exclusion, and reject-test bullets are mandatory. |
| `053-04` | `053-04-PLAN.md` | All `FeeEnvelope`, payer/sponsor, fee-domain, budget, expiry, replay, state-preservation, durable replay, API clarity, and reject-test bullets are mandatory. |
| `053-05` | `053-05-PLAN.md` | All generalized backend trait, old `AssetTreeBackend` replacement, put/delete/get/lookup/list/prove/apply-batch, right transition, path-index, typed error, mode-selection, old caller removal, and in-place rewrite bullets are mandatory. |
| `053-06` | `053-06-PLAN.md` | All asset/genesis YAML, `rights:`, schema, canonical filename, `RightsConfigEntry`, parser, deterministic terminal/payload, export/report/log, storage ingestion, scenario Stage 1/3/4/11/13, design YAML, runner verification, and config-test bullets are mandatory. |
| `053-07` | `053-07-PLAN.md` | All proof envelope, inclusion, deletion, non-existence, default commitment, present-key rejection, local-not-found rejection, historical proof, measurement, storage-owned decoder, and proof-family reject-test bullets are mandatory. |
| `053-08` | `053-08-PLAN.md` | All `BucketEpoch`, `AdaptiveBucket`, split/merge eligibility, hysteresis, `SplitProof`, `MergeProof`, `PolicyTransitionProof`, historical epoch, stale proof, crash recovery, benchmark-before-default, and reject-test bullets are mandatory. |
| `053-09` | `053-09-PLAN.md` | All local exact counter, proof-visible bounded evidence, range/threshold/commitment, binding, ordinary inclusion exclusion, correlation, raw-count guardrail, diagnostics, and privacy-test bullets are mandatory. |
| `053-10` | `053-10-PLAN.md` | All cache module, required cache layers, dirty-set invalidation, recompute hooks, memory/eviction, warmup, rollback/proof-version clear, no-secret cache, metrics, and cache-test bullets are mandatory. |
| `053-11` | `053-11-PLAN.md` | All scheduler, bounded parallel terminal/proof/policy work, deterministic parent sorting, RedB blocking executor, cancellation, rollback, backpressure, sync entrypoint, and scheduler-test bullets are mandatory. |
| `053-12` | `053-12-PLAN.md` | All journal field, adaptive status, child-before-parent, right/fee/deletion/non-existence/split/merge/policy recovery, digest validation, fail-closed, and interruption-test bullets are mandatory. |
| `053-13` | `053-13-PLAN.md` | All RedB terminal/right/fee/adaptive/proof/root row, path-index rebuild, cache warm validation, historical proof, no Phase 052 reader, helper deletion, reload, stale-cache, and old-row rejection bullets are mandatory. |
| `053-14` | `053-14-PLAN.md` | All checkpoint, snapshot, claim/right source, wallet, validator, linked-liability, OnionNet separation, downstream guardrail, proof API, and wallet/right rejection bullets are mandatory. |
| `053-15` | `053-15-PLAN.md` | All config lane, design-scenario, Stage 13 module, examples 1-8, artifact schema, fee fields, cache/scheduler metrics, deterministic output directory, required artifacts, schema fields, typed error artifacts, reload-debug, runner negative fixtures, source-shape checks, storage-owned API, and artifact-test bullets are mandatory. |
| `053-16` | `053-16-PLAN.md` | All mixed corpus, operation generator, independent model oracle, legacy reject corpus, reordering property, state-preservation, fuzz seed, model oracle, and reject-test bullets are mandatory. |
| `053-17` | `053-17-PLAN.md` | All benchmark harness, search/read/insert/delete/proof/proof-size/cache/workload/scheduler/adaptive metric lanes, evidence-only metric, regression/default gates, measured-lane, and `assets_benches.md` bullets are mandatory. |
| `053-18` | `053-18-PLAN.md` | All storage docs, API examples, operator notes, hard-cutover notes, removed-code docs, privacy docs, old-mode exclusion, state-update, source-shape, and API-example test bullets are mandatory. |
| `053-19` | `053-19-PLAN.md` | All full verification, focused storage, simulator, bench compile, measured bench, review loop, Cargo manifest, default switch, closeout summary, and evidence bullets are mandatory. |
| `053-20` | `053-20-PLAN.md` | All simple-JMT/compat storage removal, `AssetBackendMode` branch deletion, compatibility projection/proof deletion, dead tests/benches/docs/fixtures/helper deletion, grep audits, removed-module record, and source-shape audit bullets are mandatory. |

The global sections in `053-TODO.md` are also mandatory for every applicable plan: Mission, Source Reading Notes, Non-Negotiable Rules, Live-Code Acceptance Definition, Target Architecture, Required Live Contracts, Performance And Cache Contract, Required Test Matrix, Test Tier Contract, Scenario 1 Example Matrix, Verification Order, Explicit Non-Goals, Failure Conditions, Completion Gate, and Doublecheck Notes. D-26 extends this contract to every dash-list bullet in those global sections and nested lists.

</todo_bullet_coverage>

<specifics>

## Specific Ideas

- The target root chain is `SettlementStateRoot -> SettlementDefinitionLeaf -> SettlementSerialLeaf -> AdaptiveBucketRootLeaf -> SettlementLeaf::Asset(AssetLeaf) | SettlementLeaf::Right(RightLeaf)`.
- `FeeEnvelope` is processed as a separate support contract and is not embedded in `RightLeaf`.
- Proof envelope generation 2 must cover inclusion, deletion, and non-existence, and must reject present-key absence claims.
- Adaptive bucket proof families include split, merge, and policy transition proofs with `BucketEpoch`.
- Scenario 1 must produce `hjmt_settlement_examples.json`, `hjmt_tamper_report.json`, `hjmt_proof_size_report.json`, `hjmt_cache_scheduler_metrics.json`, `hjmt_replay_roots.json`, and `genesis_settlement_manifest.json`.
- Phase 053 closeout must run the bootstrap gate before broader cargo validation and must run the execution-review prompt repeatedly until two consecutive clean runs after at least three YOLO-mode runs.

</specifics>

<deferred>

## Deferred Ideas

None. Old-storage conversion, compatibility adapter lanes, asset-only canonical genesis, generic rights objects, fee-inside-right semantics, proof-visible raw occupancy counters, and downstream physical-layout authority are explicit non-goals rather than deferred work.

</deferred>

---

*Phase: 053-HJMT-Backend*
*Context gathered: 2026-05-29 via PRD Express Path*
