---
phase: 062-Gaps-Closing-2
artifact: test-spec
status: planning-ready
source: .planning/phases/062-Gaps-Closing-2/062-TODO.md, .planning/phases/062-Gaps-Closing-2/062-CONTEXT.md, .planning/phases/062-Gaps-Closing-2/062-COVERAGE.md, .planning/phases/062-Gaps-Closing-2/062-01-PLAN.md..062-27-PLAN.md
updated: 2026-06-27
---

<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD041 MD047 -->

# Phase 062 Test Specification

## 🎯 Purpose

This document turns the Phase 062 planning packet into an executable test
contract for the next engineer or agent. `.planning/phases/062-Gaps-Closing-2/062-TODO.md` remains the canonical
task inventory. `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` remains the anti-drift mirror. The grouped
`PLAN-062-G01` through `PLAN-062-G27` files remain the execution packet. This
file defines:

- which end-to-end behaviors must be proven;
- which unit and integration seams own the proof;
- which realistic examples demonstrate correct execution;
- which negative and failure paths must reject or fail closed;
- which roots, proofs, signatures, commitments, and policy invariants must be
  observed;
- which pass signals are measurable enough to prevent placeholder closure.

The anti-drift mirror requirement is literal for the TODO meta-sections:
`Verdict`, `Normative Language`, `Source Corpus`, `Count Answer`, `Required GSD
Plan Groups`, `Current Wallet Path Rewrite Map`, `Current Code Evidence
Anchors`, `Canonical Task Inventory`, and `Verification Checklist` must remain
auditable from `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md`, with any workspace-only renames called out as
separate live-resolution notes instead of silent rewrites.

For Phase 062, the referenced design corpus (`.planning/phases/062-Gaps-Closing-2/GAPS.md`, `.planning/phases/062-Gaps-Closing-2/asset-only.md`,
`.planning/phases/062-Gaps-Closing-2/HJMT-REPORT.md`, `.planning/phases/062-Gaps-Closing-2/HJMT-RAID -Sharding.md`,
`.planning/phases/062-Gaps-Closing-2/HJMT-Sharding-Storage-Techpaper.md`, `.planning/phases/062-Gaps-Closing-2/HJMT-структуры.md`, and
`.planning/phases/062-Gaps-Closing-2/Z00Z-Thin-Transaction-Mode.md`) is live phase authority wherever
`.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, or the grouped plans cite
it. Do not downgrade cited design requirements into optional or future-only
guidance.

Phase 062 is not a new product layer. It is a closure phase over the existing
storage, wallet, HJMT, thin-mode, and genesis surfaces. The required tests
must therefore reuse truthful homes in the current codebase. Do not create a
parallel authority plane for settlement roots, wallet lifecycle, object
inventory, HJMT routing, thin-mode semantics, or node-facing wallet behavior.

In this repository, E2E means live simulator, runtime, storage, validator,
watcher, wallet, or local node-simulation execution using real project
primitives. Browser automation is not relevant to Phase 062.

## 📌 Coverage Contract

The coverage rule for this spec is strict:

1. `062-S01` through `062-S27` map one-to-one to `PLAN-062-G01` through
   `PLAN-062-G27`.
2. Every `TASK-NNN` remains owned by exactly one grouped plan through
   `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`.
3. This spec does not replace `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`; it adds scenario boundaries,
   proof paths, examples, negative cases, and measurable pass conditions.
4. A scenario is incomplete if any task in its owning grouped plan lacks a
   truthful test home, failure path, or evidence artifact.
5. `062-S01` and `062-S22` must fail if `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` drops a required TODO
   meta-section or rewrites a literal TODO mirror without a separate
   live-workspace resolution note.

## ⚙️ Classification Summary

| Class | Meaning in Phase 062 | Representative homes | Use when |
| --- | --- | --- | --- |
| TDD / unit | Pure contract, DTO, parser, validator, taxonomy, manifest-loader, digest, signature, fee, or policy behavior | `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`, `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`, `crates/z00z_wallets/src/rpc/test_tx_history_cursor_filters.rs`, `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs`, `crates/z00z_core/tests/test_genesis_manifest_refs.rs`, `crates/z00z_wallets/tests/test_thin_index.rs`, `crates/z00z_wallets/src/tx/test_fee_estimator.rs` | One seam can be proven without a multi-process or multi-component run. |
| Integration / scenario | Multi-file runtime behavior across wallet, storage, validator, watcher, or RPC boundaries | `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_wallets/src/rpc/test_asset_impl.rs`, `crates/z00z_wallets/tests/test_chain_broadcast_retry.rs` | One result depends on state transition across subsystem boundaries. |
| Simulator / local-network E2E | Real project logic executed through simulator, local node simulation, or local distributed transport simulation | `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_dist_journal.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs`, `crates/z00z_wallets/tests/test_remote_scan_worker.rs` | The behavior requires restart, tamper, route rollout, journal replication, or simulated node/network boundaries. |
| Diagnostics / evidence | Grep guardrails, docs normalization, closure register, report checks, and final verification commands | `.planning/phases/Z00Z-IMPL-PHASES.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, `cargo fmt`, `cargo clippy --release --all-targets --all-features`, `cargo test --release --all`, `cargo doc --release --no-deps` | The proof is in the artifact packet or gate output rather than one Rust assertion. |
| Skip | Inputs or forbidden surfaces, not runtime targets | `.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md`, `062-*.md`, `crates/z00z_crypto/tari/**` | The file is planning authority only, or the surface is read-only vendor code. |

## ⏰ Ordered Scenario Packet

The execution order must match the grouped plan packet:

1. `062-S01` / `PLAN-062-G01`: plan contract, verification commands, coverage,
   and evidence format preservation.
2. `062-S02` / `PLAN-062-G02`: settlement-root authority and backend env
   normalization.
3. `062-S03` / `PLAN-062-G03`: checkpoint, claim-root, publication, restart,
   and tamper evidence.
4. `062-S04` / `PLAN-062-G04`: benchmark, proof-size, and measurement
   overclaim guardrails.
5. `062-S05` / `PLAN-062-G05`: wallet lifecycle DTOs and durable tx history.
6. `062-S06` / `PLAN-062-G06`: typed receive/import/verify error taxonomy.
7. `062-S07` / `PLAN-062-G07`: receive scan outcome, cursor persistence, and
   worker no-mutation boundaries.
8. `062-S08` / `PLAN-062-G08`: request-bound inbox helper as non-authoritative
   metadata only.
9. `062-S09` / `PLAN-062-G09`: simulator wallet lifecycle evidence and
   redaction.
10. `062-S10` / `PLAN-062-G10`: field-native or Poseidon2 live-claim closure.
11. `062-S11` / `PLAN-062-G11`: privacy, reveal, backup, export, and logging
    hygiene.
12. `062-S12` / `PLAN-062-G12`: cash or object separation, fee envelope, and
    voucher lifecycle proof.
13. `062-S13` / `PLAN-062-G13`: local adapter, payroll or B2B, agentic rights,
    and machine capability simulations.
14. `062-S14` / `PLAN-062-G14`: root manifest split, referenced YAML merge,
    schema, and action-catalog decision.
15. `062-S15` / `PLAN-062-G15`: shared object vocabulary, generic errors, and
    owner-test relocation.
16. `062-S16` / `PLAN-062-G16`: local HJMT route, proof, journal, and
    publication boundary closure.
17. `062-S17` / `PLAN-062-G17`: distributed HJMT replication, quorum,
    standby, and membership simulation.
18. `062-S18` / `PLAN-062-G18`: distributed HJMT route rollout, scheduler,
    remote dispatch, locks, and observability.
19. `062-S19` / `PLAN-062-G19`: thin signed-index DTO, snapshot, and
    authentication model.
20. `062-S20` / `PLAN-062-G20`: thin cache, expansion, fallback defaults, and
    builder sharing.
21. `062-S21` / `PLAN-062-G21`: thin restart, equivalence, wrong-index, and
    privacy negatives.
22. `062-S22` / `PLAN-062-G22`: closure register, drift normalization, and
    broad validation.
23. `062-S23` / `PLAN-062-G23`: wallet `ChainClient` local node simulation.
24. `062-S24` / `PLAN-062-G24`: broadcast retry, polling, and durable tx-store
    integration.
25. `062-S25` / `PLAN-062-G25`: live-fee source simulation for fee estimator.
26. `062-S26` / `PLAN-062-G26`: remote scan worker trust-boundary simulation.
27. `062-S27` / `PLAN-062-G27`: wallet daily-spend and confirmation policy.

Every scenario validates in the same gate order:

1. `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
2. slice-owned narrow commands and simulator scenarios
3. `cargo test --release` when Rust or test behavior changes
4. `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times, fixing all
   issues and warnings, and continuing until `2` consecutive runs are clean
5. `/z00z-git-versioning` when a commit is required

All cargo validations in this packet run in release mode when cargo supports
it.

## 🔑 Required Invariants

| Invariant | How Phase 062 must prove it |
| --- | --- |
| `.planning/phases/062-Gaps-Closing-2/062-TODO.md` is normative | Scenario ownership, pass signals, and coverage remain traceable to the grouped plan packet and `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`; no scenario may silently drop a task. |
| TODO meta-sections stay auditable | `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` preserves the TODO meta headings and distinguishes literal mirrors from live workspace resolution notes. |
| Referenced design docs become live authority when cited | The Phase 062 packet treats cited HJMT, thin-mode, asset/core, and gap-analysis docs as current requirements and maps them onto one canonical workspace path instead of keeping them in future-only limbo. |
| Settlement-root authority stays singular | Storage docs, env guards, tests, and live code reject a second public semantic root and keep backend mode internal. |
| Claim-root, checkpoint, and publication bindings reuse one proof path | Tamper, restart, and reload tests must converge on the same verifier path instead of independent ad hoc checks. |
| Performance evidence never becomes semantic authority | Proof-size, TPS, latency, CPU, RSS, and journal-sync measurements stay informational and are rejected when unmeasured or mislabeled. |
| Wallet receive, import, and history remain one durable authority lane | Lifecycle DTOs, row taxonomy, durable JSONL projection, and restart convergence must all derive from real persisted state, not pending memory views. |
| Unsupported-version, parse, verify, and import failures are typed | Stable wallet and RPC error codes must replace string-only failures for receive, verify, and import paths. |
| Worker and inbox helpers stay subordinate | Worker evidence, inbox hints, and remote scan helpers must never mutate authoritative wallet state before full local validation. |
| Simulator evidence must be redacted | Wallet or HJMT simulator artifacts must not leak secrets, hidden keys, or unbounded metadata. |
| Field-native or Poseidon2 claims require real code or removal | Cryptographic claims cannot survive as future wording or comment-level promises. |
| Privacy claims stay wallet-local | Logging, backup, export, request, and package surfaces may prove bounded reveal and redaction, but may not imply transport anonymity. |
| `wallet.object.*` owns typed inventory; `wallet.asset.*` stays cash-only | Voucher, right, and object inventory must not leak into cash RPCs or value arithmetic. |
| Local adapter and rights simulations stay local | Payroll, B2B, useful-work, agentic-rights, and machine-capability examples must reuse real local primitives without implying live chain, bridge, or DA deployment. |
| `GenesisConfig` remains the only bootstrap authority | A root manifest and referenced subfiles are valid only when they deterministically rehydrate into the same canonical genesis shape. |
| Shared vocabulary and generic errors belong to truthful owner modules | Ownership cleanup may re-export for compatibility, but must not leave object or genesis concepts misleadingly asset-owned. |
| `config/hjmt_runtime` remains runtime or orchestration fixture ownership | Storage owns proof, journal, route, and commit semantics, but not runtime placement or fixture authority. |
| Local HJMT closure precedes distributed claims | Route digests, shard proofs, journal history, validator or watcher checks, and public API boundaries must be locally closed before multi-aggregator simulation. |
| Distributed HJMT simulation uses real planner, journal, route, proof, and recovery primitives | Only transport and fault or clock scheduling may be faked; consensus, lineage, route activation, and failover semantics must stay real. |
| Thin mode is an authenticated helper, not a second theorem | Signed snapshots, digest pinning, expiry, checkpoint binding, fallback, and equivalence must preserve canonical thick semantics. |
| Full-system wallet node behaviors may be simulated locally but not omitted | Node RPC, broadcast, fee sourcing, remote scan, and spend policy must be executable against local deterministic node or network simulation. |
| Final closure is phase-failing on unresolved local blockers | Future-only prose, stale terminology, untyped errors, unmeasured claims, or adapter-only gaps outside the explicitly allowed boundary fail closure. |

## ✅ Scenario Matrix

| Scenario ID | Plan | Class | Primary homes | What it proves |
| --- | --- | --- | --- | --- |
| `062-S01` | `PLAN-062-G01` | Diagnostics / evidence | `.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, plan corpus | The grouped plan packet preserves task coverage, evidence gates, and verification commands exactly. |
| `062-S02` | `PLAN-062-G02` | TDD + diagnostics | storage root types, backend guards, HJMT backend tests | Storage root naming and backend env behavior converge on one semantic authority. |
| `062-S03` | `PLAN-062-G03` | Integration + simulator E2E | checkpoint store, watcher publication, simulator checkpoint tests | Claim-root, checkpoint, and publication bindings reject tamper and survive restart. |
| `062-S04` | `PLAN-062-G04` | Diagnostics + integration | bench-lane tests, stage surface tests, measurement guard | Throughput or proof-size claims remain bounded to measured artifacts. |
| `062-S05` | `PLAN-062-G05` | TDD + integration | wallet tx storage, lifecycle DTOs, history suites | Durable history and lifecycle fields stay restart-safe and authoritative. |
| `062-S06` | `PLAN-062-G06` | TDD + integration | wallet error mapping, import security, replay protection | Decode, verify, import, and RPC surfaces expose deterministic typed failures. |
| `062-S07` | `PLAN-062-G07` | Integration | scan status DTOs, receive service, restart suites | Receive outcome and cursor persistence stay authoritative; worker evidence cannot mutate state early. |
| `062-S08` | `PLAN-062-G08` | Integration | request validation, inbox helper, e2e request tests | Inbox metadata ordering is useful but cannot become receive authority. |
| `062-S09` | `PLAN-062-G09` | Simulator E2E + diagnostics | scenario_1 wallet flows, logging policy tests | Wallet simulator evidence is redacted, joined, and traceable to live lower-level primitives. |
| `062-S10` | `PLAN-062-G10` | TDD / crypto gate | zkpack, golden tag16, asset-pack tests | Field-native live claims are either fully proven or explicitly removed. |
| `062-S11` | `PLAN-062-G11` | Integration + diagnostics | logging, backup metadata, view-key, request tests | Privacy and selective disclosure claims are bounded to real wallet surfaces without leak regressions. |
| `062-S12` | `PLAN-062-G12` | Integration | object package contract, object RPC, validator object-policy tests | Cash or object boundaries, fee envelopes, and voucher lifecycle stay fail-closed. |
| `062-S13` | `PLAN-062-G13` | Local-simulation integration | object package, rights policy, validator tests | Local business-right and capability scenarios reuse real local primitives without live-chain drift. |
| `062-S14` | `PLAN-062-G14` | TDD + integration | genesis config, manifest loader, golden tests | Referenced YAML manifests merge deterministically into the single canonical genesis authority. |
| `062-S15` | `PLAN-062-G15` | TDD / ownership hygiene | object family, generic errors, registry suite | Shared vocabulary, errors, and tests live under truthful owners without concept drift. |
| `062-S16` | `PLAN-062-G16` | Integration + simulator E2E | HJMT planner, proof batch, journal, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | Local HJMT route, proof, publication, and ownership boundaries close on real primitives. |
| `062-S17` | `PLAN-062-G17` | Local-network E2E | distributed journal and consensus tests, HJMT topology | Journal replication, quorum, standby catch-up, membership, and split-brain handling are locally simulated with real state. |
| `062-S18` | `PLAN-062-G18` | Local-network E2E | `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | Route activation, dispatch, locks, and observability behave lawfully under local distributed simulation. |
| `062-S19` | `PLAN-062-G19` | TDD + integration | thin DTOs, thin index, tx store integration | Thin signed-index snapshots are authenticated, checkpoint-bound, and non-authoritative. |
| `062-S20` | `PLAN-062-G20` | Integration | thin cache, thin builder, thin-mode tests | Thin expands before runtime admission and shares one transaction meaning with thick mode. |
| `062-S21` | `PLAN-062-G21` | Integration + negative suite | thin fallback, privacy, equivalence, wrong-index tests | Restart, fallback, privacy, and wrong-index failures stay safe and typed. |
| `062-S22` | `PLAN-062-G22` | Diagnostics / evidence | closure register docs, final guard tests, repo-wide gates | No stale or future-only claim survives final closeout, and broad validation is explicit. |
| `062-S23` | `PLAN-062-G23` | Local node-simulation E2E | chain client, local node sim, chain RPC tests | Wallet node RPC capability works locally and no longer hides behind not-implemented stubs. |
| `062-S24` | `PLAN-062-G24` | Local node-simulation integration | broadcast impl, tx runtime state, broadcast retry tests | Broadcast lifecycle persists retries, rejections, replacement, and confirmation transitions durably. |
| `062-S25` | `PLAN-062-G25` | Local node-simulation TDD + integration | `crates/z00z_wallets/src/tx/fee_estimator.rs`, `crates/z00z_wallets/tests/test_fee_rate_source.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs` | Fee-rate sourcing is simulated live, cached safely, and fail-safe on stale or pathological data. |
| `062-S26` | `PLAN-062-G26` | Local node-simulation integration | scan engine, receive service, remote-scan worker tests | Remote scan workers stay helpers only and cannot bypass local verification or persistence rules. |
| `062-S27` | `PLAN-062-G27` | Integration + policy E2E | wallet policy, tx storage, broadcast retry suite | Daily-spend and confirmation rules are enforced through durable history and restart-safe aggregation. |

## ✅ Detailed Scenario Contracts

### ✅ `062-S01` Plan Contract, Coverage, And Evidence Preservation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, `.planning/phases/062-Gaps-Closing-2/062-01-PLAN.md`, grouped plan corpus | TODO inventory -> literal context mirror -> grouped plan packet -> coverage appendix and gate fields | Task count remains `125`, grouped plan count remains `27`, every grouped plan carries `simulation_gate`, `anti_placeholder_gate`, and `evidence_gate`, and `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` preserves the TODO meta headings | Missing or duplicate task ownership, missing gate sections, missing acceptance commands in a grouped plan, or a dropped TODO meta-section in context | `rg` counts match `125` tasks and `27` grouped plans; grouped plans expose the required gate fields; `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md` preserves the required TODO meta sections and separates literal mirrors from live-workspace resolution notes |

### ✅ `062-S02` Settlement Root Authority And Backend Guardrails

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_storage/src/settlement/root_types.md`, `crates/z00z_storage/src/backend/mod.rs`, `crates/z00z_storage/src/settlement/store.rs`, `crates/z00z_storage/src/settlement/hjmt_config.rs`, `crates/z00z_storage/tests/test_hjmt_backend_conformance.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs` | canonical root naming -> backend env normalization -> guardrail tests -> docs grep | `Z00Z_SETTLEMENT_BACKEND_MODE` accepts unset or `hjmt`; internal backend root names do not appear as second public truth | `Z00Z_STORAGE_BACKEND` survives as current wording, or docs imply a second semantic root | Storage tests pass and grep results show one semantic settlement-root story only |

### ✅ `062-S03` Checkpoint, Claim-Root, Publication, Restart, And Tamper Closure

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_storage/tests/test_claim_source_proof.rs`, `crates/z00z_storage/tests/test_checkpoint_root_binding.rs`, `crates/z00z_storage/tests/test_checkpoint_finalization.rs`, `crates/z00z_simulator/tests/scenario_1/test_checkpoint_acceptance.rs`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs` | claim-root proof -> checkpoint binding -> publication artifact -> restart/reload -> simulator evidence | Valid claim-root binds to checkpoint and survives reload; simulator publication reuses storage and watcher proof paths | Tampered checkpoint artifacts, stale publication bindings, wrong prior root, or restart-induced divergence | Storage and simulator tests pass; tamper vectors fail closed; restart evidence shows consistent checkpoint/root continuity |

### ✅ `062-S04` Benchmark, Proof-Size, And Measurement Guardrails

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`, `crates/z00z_rollup_node/tests/test_hjmt_node_lifecycle.rs`, `crates/z00z_simulator/src/scenario_1/runner_verify.rs`, `crates/z00z_storage/benches/settlement_benches.md`, `.planning/phases/062-Gaps-Closing-2/062-04-SUMMARY.md` | measured runtime output -> labeling guard -> docs or report claims | Durable-root-published throughput, journal sync, latency, RSS, and CPU are published as measured metadata only | Proof-size or TPS claims appear without measured artifacts, or an observer sidecar becomes semantic authority | Guard tests pass, claim grep shows only measured wording, and informational metrics are explicitly separated from correctness claims |

### ✅ `062-S05` Wallet Lifecycle DTOs And Durable Tx History

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/rpc/tx_types.rs`, `crates/z00z_wallets/src/persistence/tx_storage.rs`, `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`, `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`, `crates/z00z_wallets/src/rpc/tx_rpc_server_history.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_wallets/src/rpc/test_tx_history_cursor_filters.rs`, `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs`, `crates/z00z_wallets/src/rpc/test_tx_pending_suite.rs` | durable tx rows -> lifecycle projection -> RPC surfaces -> restart replay/fold convergence | Send, details, reconcile, import, and verify surfaces expose the same durable lifecycle view; tombstones and current view survive restart | Lifecycle exists only in memory, replay breaks hash or fold semantics, or tombstone or status updates regress on restart | Integration and suite tests pass and demonstrate durable JSONL projection, row-hash continuity, fold correctness, and stable terminal states |

### ✅ `062-S06` Typed Unsupported-Version, Verify, Import, And Parse Taxonomy

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/rpc/error_mapping.rs`, `crates/z00z_wallets/src/receiver/asset_receive_types.rs`, verifier seam, `crates/z00z_wallets/tests/test_import_error_taxonomy.rs`, `crates/z00z_wallets/tests/test_asset_import_security.rs`, `crates/z00z_wallets/tests/test_asset_replay_protection.rs` | decode -> verify report -> import -> RPC mapping | Unsupported receive versions, metadata mismatch, replay, and malformed packages surface stable typed errors with no state mutation | String-only failures survive, or verify/report mutates wallet state before import | Taxonomy and replay tests pass; error surfaces are typed, deterministic, and non-mutating |

### ✅ `062-S07` Receive Outcome DTO, Cursor Persistence, And Worker No-Mutation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/rpc/chain_types.rs`, `crates/z00z_wallets/src/rpc/chain_rpc_impl.rs`, `crates/z00z_wallets/src/services/wallet_actions_receive.rs`, `crates/z00z_wallets/src/redb_store/owned_assets.rs`, `crates/z00z_wallets/tests/test_asset_scanner_flow.rs`, `crates/z00z_wallets/tests/test_asset_scanner_cache.rs`, `crates/z00z_wallets/src/services/test_wallet_service.rs`, `crates/z00z_wallets/tests/test_remote_scan_worker.rs` | authoritative receive scan -> public status projection -> cursor persistence -> validated worker re-entry | `RuntimeReceiveScanOutcome` reports last outcome, restart resumes from the persisted cursor, and validated worker hints re-enter the same receive lane through `recv_range_from_worker(...)` | Invalid worker evidence mutates assets or cursor, stale resume restarts at the wrong boundary, or public status diverges from authoritative receive logic | Receive and restart suites pass and prove worker subordination, cursor atomicity, and stable outcome projection |

### ✅ `062-S08` Request-Bound Inbox Helper As Metadata Only

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/receiver/request.rs`, `crates/z00z_wallets/src/receiver/request_inbox.rs`, `crates/z00z_wallets/src/services/wallet_actions_receive.rs`, `crates/z00z_wallets/tests/test_stealth_request.rs`, `crates/z00z_wallets/tests/test_e2e_req_flow.rs`, `crates/z00z_wallets/tests/test_view_key_contract.rs` | validated request -> inbox ordering -> wallet-local metadata -> receive lane join | Inbox stores validated request hints in order and helps UX without replacing the authoritative receive scan | Expired or replayed requests mutate receive state, or inbox ordering becomes the authoritative receive ledger | Request and E2E tests pass; grep and docs confirm inbox is non-authoritative and off-consensus |

### ✅ `062-S09` Simulator Wallet Lifecycle Evidence And Redaction

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`, `crates/z00z_simulator/tests/scenario_1/test_scenario1_stage_surface.rs`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`, `crates/z00z_wallets/tests/test_asset_scanner_flow.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs` | wallet lower-level proofs -> simulator scenario join -> artifact bundle and redaction | Simulator artifacts include lifecycle, scan, import, and publication digests without leaking secrets or re-owning wallet truth | Simulator output leaks hidden material or claims authority over durable wallet state | Simulator and logging policy tests pass; artifacts are redacted and traceable to real wallet/storage primitives |

### ✅ `062-S10` Field-Native Or Poseidon2 Live-Claim Closure

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/stealth/zkpack.rs`, `crates/z00z_core/src/assets/version.rs`, `crates/z00z_core/src/assets/leaf.rs`, `crates/z00z_wallets/tests/test_zkpack.rs`, `crates/z00z_wallets/tests/test_asset_pack_v2_memo.rs`, `crates/z00z_wallets/tests/test_golden_tag16.rs` | canonical package wire path -> negative cases -> live-claim normalization | Canonical package behavior is proven on the real crypto wire path, and future field-native claims are either implemented with tests or removed | Future Poseidon2 or field-native wording survives without code, or negatives miss tamper/version/replay cases | Package tests pass and docs or code no longer overclaim unimplemented field-native behavior |

### ✅ `062-S11` Privacy, Reveal, Logging, Backup, And Export Hygiene

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/receiver/request.rs`, `crates/z00z_wallets/src/services/wallet_store_persistence_pack.rs`, `crates/z00z_wallets/tests/test_rpc_logging_risk_policy.rs`, `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`, `crates/z00z_wallets/tests/test_view_key_contract.rs`, `crates/z00z_wallets/docs/WALLET-GUIDE.md` | wallet-local secret boundaries -> reveal rules -> logging/backup/export surfaces -> docs | Selective disclosure and backup metadata remain bounded to documented wallet-local surfaces with stable redaction | Logs, backups, reports, or docs leak secret material or imply transport anonymity | Logging, backup, request, and guide checks pass; grep guards show no unowned privacy overclaim |

### ✅ `062-S12` Cash/Object Separation, Fee Envelope, And Voucher Lifecycle

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_storage/src/settlement/record.rs`, `crates/z00z_storage/src/settlement/tx_plan_types.rs`, `crates/z00z_storage/src/settlement/object_package_contract.rs`, `crates/z00z_wallets/src/rpc/object_types.rs`, `crates/z00z_wallets/src/rpc/object_rpc_impl.rs`, `crates/z00z_wallets/src/redb_store/owned_objects.rs`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`, `crates/z00z_wallets/src/rpc/test_asset_impl.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | typed inventory -> package validation -> validator verdict -> wallet RPC projection | Object inventory stays on `wallet.object.*`; cash RPC rejects voucher and right inventory; fee envelope and voucher lifecycle use existing object surfaces | Voucher-as-cash, right-as-value, unknown policy, missing right, or fee-boundary violations slip through | Object and validator tests pass with explicit negative coverage for cash/object boundary breaches |

### ✅ `062-S13` Local Adapter, Payroll/B2B, Agentic Rights, And Machine Capability

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_core/src/rights/right_policy.rs`, `crates/z00z_core/src/assets/assets_config.yaml`, `crates/z00z_storage/src/settlement/object_package_contract.rs`, `crates/z00z_storage/src/settlement/record.rs`, `crates/z00z_wallets/src/redb_store/owned_objects.rs`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`, `crates/z00z_wallets/src/rpc/test_asset_impl.rs` | local object/right/voucher primitives -> local adapter boundaries -> right or capability simulation | Payroll/B2B or useful-work scenarios remain local; agentic rights and machine capabilities reuse live right fixtures and policy verdicts | Local adapter work implies live chain or DA behavior, or capability reuse/replay/wrong-action failures are untested | Local-simulation tests and docs prove bounded local semantics only, with explicit replay and forged-metadata rejection |

### ✅ `062-S14` Root Manifest Split, Referenced YAML Merge, And Action Decision

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_core/src/genesis/genesis_config.rs`, `crates/z00z_core/src/genesis/genesis_config_validate.rs`, `crates/z00z_core/src/genesis/README.md`, `crates/z00z_core/src/assets/assets_config.yaml`, `crates/z00z_core/src/genesis/manifest_ref_loader.rs`, `crates/z00z_core/src/genesis/test_genesis_suite.rs`, `crates/z00z_core/tests/test_genesis_manifest_refs.rs`, `crates/z00z_core/tests/test_genesis_manifest_goldens.rs` | root manifest -> referenced subfiles -> deterministic merge -> canonical `GenesisConfig` -> schema/golden validation | Referenced subfiles assemble into the same canonical genesis shape and keep compatibility `crates/z00z_core/src/assets/assets_config.yaml` secondary | Duplicate keys, invalid paths, schema drift, or a new bootstrap authority beside `GenesisConfig` | Genesis fixture, refs, and golden tests pass, and the intentionally unsupported actions companion under `crates/z00z_core/src/genesis/` is documented explicitly rather than implied by accident |

### ✅ `062-S15` Shared Vocabulary, Generic Error Ownership, And Test Relocation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_core/src/assets/object_family.rs`, `crates/z00z_core/src/assets/asset_error.rs`, `crates/z00z_core/src/assets/mod.rs`, `crates/z00z_storage/src/settlement/record.rs`, `crates/z00z_core/src/assets/test_registry_suite.rs`, `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs` | owner move or re-export -> compile/import continuity -> owner-module tests | Shared object vocabulary and generic validation errors are re-homed without breaking callers or semantics | Object or genesis concepts remain misleadingly asset-owned, or moved tests lose the truthful owner surface | Registry and validator suites pass and grep confirms truthful ownership with only compatibility re-exports where needed |

### ✅ `062-S16` Local HJMT Proof, Route, Publication, And Ownership Boundary

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_runtime/aggregators/src/batch_planner.rs`, `crates/z00z_storage/src/settlement/proof_batch.rs`, `crates/z00z_storage/src/settlement/proof_batch_verify.rs`, `crates/z00z_storage/src/settlement/hjmt_journal.rs`, `crates/z00z_storage/src/settlement/hjmt_commit.rs`, `crates/z00z_runtime/validators/tests/test_hjmt_publication_contract.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `crates/z00z_simulator/tests/scenario_1/test_hjmt_e2e.rs`, `config/hjmt_runtime/sim_5a7s/manifest.json` | route snapshot -> proof batch -> publication checker -> validator/watcher view -> runtime fixture ownership | Route digests are deterministic, cross-shard batches stay rejected, historical proofs stay valid under historical context, and wallet sees public proofs only | Tampered route digests, stale lineage, cross-shard implicit fallback, or raw RedB details leaking into wallet-facing seams | HJMT publication and topology tests pass and ownership docs keep runtime placement outside storage semantics |

### ✅ `062-S17` Distributed HJMT Replication, Quorum, Standby, And Membership

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_runtime/aggregators/src/dist_sim.rs`, `crates/z00z_runtime/aggregators/src/consensus_adapter.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_dist_journal.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_consensus.rs`, `crates/z00z_rollup_node/tests/test_hjmt_topology.rs`, `config/hjmt_runtime/sim_5a7s/manifest.json` | real journal entries -> multi-aggregator transport sim -> quorum/term or lineage resolution -> standby catch-up -> membership transitions | Delay/drop/reorder, partition/heal, standby catch-up, join/leave/decommission, and same-shard competing roots resolve deterministically or fail closed with telemetry | Distributed claims stay prose-only, or fake consensus bypasses real journal/root inputs | Aggregator and topology tests pass, showing real planner/storage/journal/proof semantics under deterministic transport simulation |

### ✅ `062-S18` Distributed HJMT Route Rollout, Scheduler, Dispatch, Locks, And Observability

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_runtime/aggregators/src/dist_dispatch.rs`, `crates/z00z_runtime/aggregators/src/dist_scheduler.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_route_rollout.rs`, `crates/z00z_runtime/aggregators/tests/test_hjmt_dispatch.rs`, `crates/z00z_runtime/watchers/tests/test_hjmt_publication_contract.rs`, `config/hjmt_runtime/sim_5a7s/manifest.json` | route generation -> process acknowledgements -> scheduler waves -> dispatch ownership -> lock enforcement -> drift telemetry | Mixed-generation routes are rejected, dispatch reaches the owning worker only, locks reject shared-root hazards, and observability records stalls or disputes without becoming authority | Wrong-owner or duplicate dispatch, late joiner on stale route, cross-shard implicit success, or observability replacing proof | Route rollout and dispatch tests pass with explicit rejection of mixed generation, stale locks, cross-shard requests, and wrong-owner dispatch |

### ✅ `062-S19` Thin DTO, Signed Snapshot, And Authentication Model

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/tx/thin_types.rs`, `crates/z00z_wallets/src/tx/thin_index.rs`, `crates/z00z_wallets/src/tx/thin_snapshot.rs`, `crates/z00z_wallets/src/rpc/tx_types.rs`, `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`, `crates/z00z_wallets/tests/test_thin_index.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | thin DTO -> signed snapshot -> digest pin -> checkpoint context -> typed API projection | Thin snapshot entries are signed, digest-pinned, checkpoint-bound, and explicitly non-authoritative helpers | Unsigned, stale, wrong-root, or naming-drifted thin surfaces survive as live behavior | Thin index tests pass and docs or APIs use current root and package names without creating a second authority |

### ✅ `062-S20` Thin Cache, Expansion, Fallback Default, And Builder Sharing

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/tx/thin_cache.rs`, `crates/z00z_wallets/src/tx/thin_builder.rs`, `crates/z00z_wallets/src/tx/thin_index.rs`, `crates/z00z_wallets/src/rpc/tx_rpc_impl.rs`, `crates/z00z_wallets/tests/test_thin_modes.rs`, `crates/z00z_wallets/tests/test_thin_cache.rs`, `crates/z00z_wallets/tests/test_thin_index.rs` | authenticated helper -> cache pin/refresh -> expansion before runtime -> shared builder semantics | Thin expands into the existing runtime vocabulary before admission and produces the same transaction meaning as thick mode | Cache uncertainty does not fall back to thick, or thin-specific runtime objects appear downstream | Thin mode and cache tests pass and grep confirms no `ThinWorkItem`, thin verdict, or second theorem was added |

### ✅ `062-S21` Thin Restart, Equivalence, Wrong-Index, Fallback, And Privacy

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/tests/test_thin_fallback.rs`, `crates/z00z_wallets/tests/test_thin_privacy.rs`, `crates/z00z_wallets/tests/test_thin_equivalence.rs`, `crates/z00z_wallets/tests/test_thin_index.rs`, `crates/z00z_wallets/src/tx/thin_cache.rs` | cached helper state -> restart -> authenticated restore or thick fallback -> semantic equivalence -> privacy guard | Restart after valid snapshot stays authenticated; stale or equivocated helper data falls back to thick without semantic drift | Wrong, withheld, expired, equivocated, or helper-only privacy leaks survive as successful thin behavior | Thin restart, equivalence, fallback, and privacy suites pass and show typed fail-closed behavior for every wrong-index class |

### ✅ `062-S22` Closure Register, Drift Normalization, And Broad Validation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, `.planning/phases/Z00Z-IMPL-PHASES.md`, `crates/z00z_wallets/tests/test_spec_terms_guard.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs`, `.planning/phases/062-Gaps-Closing-2/062-VALIDATION.md` | per-slice evidence -> closure register -> stale-term grep -> focused validation -> broad repo validation | Final packet classifies live, compatibility, simulation-only, adapter-only, and removed-claim terms and keeps exact blocker evidence | Future-only or research-only claims remain in closeout language, or broad failures are silently omitted | Focused guard tests and broad repo commands pass, or exact unrelated failures are recorded with no ambiguity about Phase 062 status |

### ✅ `062-S23` Wallet `ChainClient` Local Node Simulation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/chain/chain_client.rs`, `crates/z00z_wallets/src/chain/chain_client_impl.rs`, `crates/z00z_wallets/src/chain/local_node_sim.rs`, `crates/z00z_wallets/src/rpc/chain_rpc_impl.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs`, `crates/z00z_wallets/tests/test_direct_tx_receive.rs` | wallet chain client -> local node state -> RPC method coverage -> typed failure mapping | Tip, block, header, submit, transaction status, and network-info queries work against deterministic local node state | Node RPC seams remain `not implemented`, or missing block/tx/network info paths return untyped failures | Chain-client simulation and receive integration tests pass and grep confirms live node RPC methods exist |

### ✅ `062-S24` Broadcast Retry, Polling, Confirmation, And Durable Tx-Store Lifecycle

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/chain/broadcast_impl.rs`, `crates/z00z_wallets/src/persistence/tx_storage.rs`, `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`, `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`, `crates/z00z_wallets/src/rpc/test_tx_broadcast_suite.rs`, `crates/z00z_wallets/tests/test_chain_broadcast_retry.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs` | submit -> durable pending state -> retry/poll loop -> confirmation or replacement transition -> history projection | Timeout retries, duplicate submissions, rejection, replacement, and confirmation persist correct lifecycle rows and status updates | Timeout-only closure, string-only retry handling, or lifecycle state that never lands durably | Broadcast suites pass and prove durable state transitions for each broadcast outcome class |

### ✅ `062-S25` Simulated-Live Fee Source For Fee Estimation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/tx/fee_estimator.rs`, `crates/z00z_wallets/src/tx/test_fee_estimator.rs`, `crates/z00z_wallets/tests/test_fee_rate_source.rs`, `crates/z00z_wallets/tests/test_chain_client_sim.rs` | local node fee source -> cache and TTL -> estimator use -> fail-safe fallback | Fresh simulated fee data is used, cache and fallback semantics are explicit, and weight math remains on the real estimator path | Static fee assumptions survive while live-fee claims remain, or zero/stale/spiking data is accepted unsafely | Fee estimator and fee-source tests pass, showing cache, fallback, stale, zero, and spike behavior explicitly |

### ✅ `062-S26` Remote Scan Worker Trust-Boundary Simulation

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/chain/scan_engine.rs`, `crates/z00z_wallets/src/chain/local_node_sim.rs`, `crates/z00z_wallets/src/chain/scan_engine_impl.rs`, `crates/z00z_wallets/src/services/wallet_actions_receive.rs`, `crates/z00z_wallets/tests/test_remote_scan_worker.rs`, `crates/z00z_wallets/src/services/test_wallet_service.rs` | local node/chain simulation -> remote worker hint -> `recv_range_from_worker(...)` -> strict local validation -> no-mutation persistence gate | Valid remote hints accelerate scan orchestration but re-enter the same authoritative receive path before mutation | Malicious, stale, replayed, or restart-shifted remote results mutate state or bypass local verification | Remote scan worker and receive suites pass and show no-mutation on every rejected worker path |

### ✅ `062-S27` Wallet Daily-Spend And Confirmation Policy Enforcement

| Homes | Proof path | Positive examples | Negative examples | Pass conditions |
| --- | --- | --- | --- | --- |
| `crates/z00z_wallets/src/wallet/policy.rs`, `crates/z00z_wallets/src/persistence/tx_storage.rs`, `crates/z00z_wallets/src/persistence/tx_storage_impl.rs`, `crates/z00z_wallets/tests/test_wallet_policy.rs`, `crates/z00z_wallets/tests/test_tx_store_integration.rs`, `crates/z00z_wallets/tests/test_chain_broadcast_retry.rs` | durable tx history -> aggregated spend windows -> confirmation requirement -> restart restore -> rejection surfaces | Daily cap enforcement survives restart, multi-send aggregation counts correctly, and confirmation requirements gate spending paths explicitly | Policy remains TODO or UI-only text, or restart drops the accumulated daily-spend state | Wallet policy, tx-store, and broadcast-linked tests pass and prove fail-closed enforcement across restart and aggregate send flows |

## 🔁 Test File Placement And Reuse Rules

| Area | Reuse first | New home allowed only when |
| --- | --- | --- |
| Planning and closeout gates | `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, grouped plans, `crates/z00z_wallets/tests/test_spec_terms_guard.rs`, `crates/z00z_storage/tests/test_live_guardrails.rs` | A focused consistency helper is required and still writes into the current planning or guardrail authority surfaces. |
| Storage root and checkpoint closure | existing storage tests and simulator checkpoint tests | The missing seam cannot be expressed in the current storage or simulator test homes without hiding the semantic boundary. |
| Wallet lifecycle, import, and scan | existing wallet RPC suites, receive tests, tx-store integration, and persistence seams | A new test file isolates a real seam without inventing a second wallet authority plane. |
| Privacy, backup, and request metadata | existing logging, backup, request, and view-key tests | A new helper is required to express a leak class already implied by the normative docs. |
| Object, voucher, rights, and fee envelope | existing object package, validator, wallet object-RPC, and simulator object-flow tests | The missing negative case cannot be stated honestly in the current owner test home. |
| Genesis manifest and ownership cleanup | existing genesis and registry suites | A new targeted loader or golden test is required to prove deterministic manifest assembly or owner relocation. |
| Local HJMT | existing planner, proof, journal, publication, topology, and simulator HJMT tests | A focused route/proof/history seam cannot be added cleanly without losing its boundary. |
| Distributed HJMT | `crates/z00z_runtime/aggregators/tests`, `crates/z00z_rollup_node/tests`, and `config/hjmt_runtime/sim_5a7s/manifest.json` | New tests still live under the current HJMT runtime or aggregator tree and reuse live planner, journal, proof, and route primitives. |
| Thin mode | current wallet tx and storage seams plus thin-specific helper tests | A helper test is needed to isolate snapshot authentication, fallback, or privacy risk without creating thin-specific runtime semantics. |
| Full-system wallet/node simulation | existing wallet chain, broadcast, fee, scan, and policy tests | New files are allowed only to express deterministic local node or network simulation and must still drive real wallet primitives. |

## 🚫 Skip And Reservation Rules

| Item | Status | Reason |
| --- | --- | --- |
| `.planning/phases/062-Gaps-Closing-2/062-TODO.md`, `.planning/phases/062-Gaps-Closing-2/062-CONTEXT.md`, `.planning/phases/062-Gaps-Closing-2/062-COVERAGE.md`, and grouped plans as runtime targets | Skip | They are planning authorities and inputs, not runtime assertions. |
| `crates/z00z_crypto/tari/**` | Skip forever | Protected vendor code is read-only. Closure may happen only through wrappers, pins, tests around exposed seams, or live-claim normalization. |
| A second settlement-root public API | Forbidden | Phase 062 must preserve one semantic settlement-root authority. |
| A second wallet authority plane for history, scan, objects, or policy | Forbidden | Wallet helpers, inboxes, remote workers, and node simulations remain subordinate to existing authority seams. |
| A second HJMT proof, route, or publication truth path | Forbidden | Local and distributed HJMT tests must reuse the current planner/storage/journal/publication primitives. |
| Thin-specific runtime semantics downstream of builder expansion | Forbidden | Thin mode is an authenticated helper, not a second transaction theorem. |
| Docs-only or compile-only closure for runtime behavior | Forbidden | Phase 062 requires executable proof, simulator evidence, or explicit live-claim removal where the normative task allows it. |

<verify>

1. Run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` as
   the mandatory fail-fast gate before any broader validation.
2. Re-run packet consistency checks so this spec points only at live canonical
   paths where code already exists:
   `bash -lc 'for p in crates/z00z_wallets/src/receiver/request_inbox.rs crates/z00z_core/src/genesis/manifest_ref_loader.rs crates/z00z_runtime/aggregators/src/dist_sim.rs crates/z00z_runtime/aggregators/src/consensus_adapter.rs crates/z00z_runtime/aggregators/src/dist_dispatch.rs crates/z00z_runtime/aggregators/src/dist_scheduler.rs crates/z00z_wallets/src/tx/thin_types.rs crates/z00z_wallets/src/tx/thin_snapshot.rs crates/z00z_wallets/src/tx/thin_cache.rs crates/z00z_wallets/src/tx/thin_builder.rs crates/z00z_wallets/src/chain/local_node_sim.rs crates/z00z_wallets/tests/test_wallet_policy.rs; do test -e "$p" || { echo "missing path: $p"; exit 1; }; done'`
   and
   `rg -n "RuntimeReceiveScanOutcome" crates/z00z_wallets .planning/phases/062-Gaps-Closing-2/062-TEST-SPEC.md`
3. Run `cargo test --release` when the touched slice changes Rust, tests,
   simulator behavior, public APIs, serialization, or verification scripts.
   Use release mode for every cargo validation command in this packet when
   cargo supports it.
4. Run `./.github/prompts/gsd-review-tasks-execution.prompt.md`
   (`/GSD-Review-Tasks-Execution`) in YOLO mode at least `3` times. Fix all
   issues and warnings and continue until at least `2` consecutive runs show
   no significant issues.
5. If a commit is required after verification, use `/z00z-git-versioning`.

## ✅ Completion Criteria

| Criterion | Pass condition |
| --- | --- |
| Scenario coverage | `062-S01` through `062-S27` all exist and still map one-to-one to `PLAN-062-G01` through `PLAN-062-G27`. |
| Task coverage | The owning grouped plan for every `TASK-001` through `TASK-125` has at least one truthful test home, one negative path, and one measurable pass signal. |
| No concept drift | Test placement follows existing owner seams and does not introduce a second wallet, HJMT, genesis, or thin-mode authority layer. |
| Real-primitive proof | Cryptographic, checkpoint, journal, route, publication, node, fee, scan, and policy scenarios run on real project primitives, with only allowed transport or scheduler boundaries simulated. |
| Negative closure | Every scenario names the failure modes that must reject or fail closed, and the planned tests make those failures observable. |
| Final gate | The next engineer or agent can implement the test coverage without guessing scenario boundaries, success criteria, anchors, or evidence expectations. |
