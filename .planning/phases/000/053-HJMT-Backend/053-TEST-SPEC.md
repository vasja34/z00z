---
phase: 053-HJMT-Backend
artifact: test-spec
status: implemented
source:
  - 053-CONTEXT.md
  - 053-TODO.md
  - 053-SUMMARY.md
  - 053-01-PLAN.md
  - 053-20-PLAN.md
updated: 2026-06-06
---

<!-- markdownlint-disable MD060 -->

# Phase 053 Test Specification: HJMT Backend

**Phase:** `053-HJMT-Backend`
**Status:** implemented and packet-synced
**Authority:** `053-CONTEXT.md`, `053-TODO.md`, `053-SUMMARY.md`, and the
live repository test homes listed below

## Purpose

This document records the live test surface that proves the Phase 053 HJMT
generalized settlement backend. It is no longer a speculative implementation
brief. The repository already runs on the settlement-native surface, so this
spec now maps Phase 053 requirements to the real test suites that currently
own the proof.

The main rule for future maintenance is simple: prefer the existing live test
homes even when the filename is generic. Phase 053 coverage is intentionally
anchored to the production seam that owns the behavior, not to a requirement
that every file carry a `phase053` prefix.

## Workflow Status

- **Operating mode:** implemented.
- **Phase state:** summary-backed complete through `053-20` and final
  `053-SUMMARY.md`.
- **Packet sync delta on 2026-06-06:** added the remaining dedicated
  canonical-authority lanes for rights config, genesis rights determinism,
  genesis settlement manifest export, and storage ingestion of generated
  genesis rights.
- **Interpretation rule:** when this spec and an older plan disagree about a
  filename, the current repository test file wins.

## Non-Negotiable Test Rules

- Tests must call production constructors, production storage APIs, production
  proof verifiers, production genesis loaders, and production simulator flows.
- Tests must not recreate proof decoding, root construction, journal replay,
  cache authority, or RedB row semantics in a parallel test model.
- Negative cases should mutate production-generated outputs whenever possible.
- Downstream checks must verify through semantic APIs, not raw tree ids,
  namespace bytes, RedB keys, branch order, or internal bucket layout.
- Docs and source-shape tests must continue rejecting stale future-only,
  compatibility, simple-JMT, or legacy-runtime wording for live Phase 053
  behavior.

## Mandatory Verification Order

Every Rust or test-affecting change that touches this packet must verify in
this order:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
cargo test --release --features test-fast --features wallet_debug_dump
```

When a narrower command is relevant, run it after the bootstrap gate and
before or alongside the broad cargo command.

Every execution slice must also run
`/.github/prompts/gsd-review-tasks-execution.prompt.md`
(`/GSD-Review-Tasks-Execution`) in YOLO mode at least three times and continue
until at least two consecutive runs report no significant issues.

## Live Test Homes

| Area | Live test homes | What they prove |
| --- | --- | --- |
| Guardrails, purge, and default gate | `crates/z00z_storage/tests/test_live_guardrails.rs`, `crates/z00z_storage/tests/test_default_gate.rs` | Live settlement exports exist, stale compatibility lanes stay rejected, docs cannot drift back to future-only language, and HJMT remains the default runtime. |
| Roots, store API, checkpoints, fee support | `crates/z00z_storage/tests/test_settlement_root.rs`, `crates/z00z_storage/tests/test_store_api.rs`, `crates/z00z_storage/tests/test_checkpoint_store_api.rs`, `crates/z00z_storage/tests/test_fee_envelope.rs`, `crates/z00z_storage/tests/test_fee_replay.rs` | `SettlementStateRoot` generation, semantic store mutations, checkpoint binding, fee-envelope separation, replay protection, and reject-state preservation. |
| Proof families and adaptive policy | `crates/z00z_storage/tests/test_hjmt_live_proof_families.rs`, `crates/z00z_storage/tests/test_hjmt_adaptive_policy_proofs.rs` | Inclusion, deletion, non-existence, split, merge, policy-transition, stale-epoch rejection, and proof-family binding. |
| Cache, scheduler, occupancy, reload | `crates/z00z_storage/tests/test_forest_cache.rs`, `crates/z00z_storage/tests/test_cache_recompute.rs`, `crates/z00z_storage/tests/test_async_scheduler.rs`, `crates/z00z_storage/tests/test_occupancy_privacy.rs`, `crates/z00z_storage/tests/test_occupancy_evidence.rs`, `crates/z00z_storage/tests/test_redb_reload.rs` | Cache correctness, fail-closed recomputation, deterministic parallel work, privacy-reviewed occupancy evidence, reload durability, and historical-proof lanes. |
| Core config and genesis authorities | `crates/z00z_core/tests/assets/test_rights_config.rs`, `crates/z00z_core/tests/genesis/test_genesis_rights.rs`, `crates/z00z_core/tests/genesis/test_genesis_manifest.rs`, `crates/z00z_core/tests/genesis/test_settlement_corpus.rs` | Canonical `rights:` parsing, deterministic genesis rights on all canonical networks, manifest export, replay-digest stability, and collision-free combined settlement corpus generation. |
| Storage ingestion and corpus fixtures | `crates/z00z_storage/tests/test_genesis_ingestion.rs`, `crates/z00z_storage/tests/test_golden_corpus.rs`, `crates/z00z_storage/tests/test_property_corpus.rs`, `crates/z00z_storage/tests/test_fuzz_seeds.rs` | Generated genesis rights can be ingested into the live store, canonical mixed asset/right corpus stays stable, property lanes preserve invariants, and malformed proof inputs fail closed. |
| Docs, benches, and metrics | `crates/z00z_storage/tests/test_readme_examples.rs`, `crates/z00z_storage/tests/test_bench_lanes.rs`, `crates/z00z_storage/tests/test_metrics.rs` | README/example snippets remain executable, benchmark lanes compile, and recorded measurement surfaces stay aligned with the live HJMT API. |
| Downstream and scenario integration | `crates/z00z_storage/tests/test_downstream_guardrails.rs`, `crates/z00z_simulator/tests/test_scenario_settlement.rs`, `crates/z00z_simulator/tests/test_stage7_jmt_wallet_scan.rs`, `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`, `crates/z00z_simulator/tests/test_s7_examples.rs`, `crates/z00z_wallets/tests/test_s7_examples.rs` | Semantic downstream use, wallet rejection of unrelated `RightLeaf`, Stage 1/4/6/11/13 scenario truth, artifact replay, and example-output stability. |

## Requirement Coverage Map

| Requirement | Primary proof homes |
| --- | --- |
| `PH53-01` live-contract guardrails | `test_live_guardrails.rs`, `test_default_gate.rs` |
| `PH53-02` settlement root generation | `test_settlement_root.rs`, `test_checkpoint_store_api.rs` |
| `PH53-03` `SettlementPath` and `RightLeaf` | `test_store_api.rs`, `test_hjmt_live_proof_families.rs`, `test_genesis_ingestion.rs` |
| `PH53-04` `FeeEnvelope` separation | `test_fee_envelope.rs`, `test_fee_replay.rs` |
| `PH53-05` store API hard cutover | `test_store_api.rs`, `test_default_gate.rs`, `test_downstream_guardrails.rs` |
| `PH53-06` YAML and genesis rights | `test_rights_config.rs`, `test_genesis_rights.rs`, `test_settlement_corpus.rs`, `test_genesis_manifest.rs`, `test_genesis_ingestion.rs` |
| `PH53-07` proof envelope v2 | `test_hjmt_live_proof_families.rs`, `test_settlement_root.rs` |
| `PH53-08` adaptive buckets and policy proofs | `test_hjmt_adaptive_policy_proofs.rs`, `test_bench_lanes.rs` |
| `PH53-09` occupancy privacy | `test_occupancy_privacy.rs`, `test_occupancy_evidence.rs` |
| `PH53-10` cache plane | `test_forest_cache.rs`, `test_cache_recompute.rs` |
| `PH53-11` scheduler | `test_async_scheduler.rs` |
| `PH53-12` journal and recovery | `test_hjmt_adaptive_policy_proofs.rs`, `test_redb_reload.rs` |
| `PH53-13` RedB reload and historical proofs | `test_redb_reload.rs`, `test_bench_lanes.rs` |
| `PH53-14` downstream integration | `test_downstream_guardrails.rs`, `test_stage7_jmt_wallet_scan.rs`, `test_s7_examples.rs` |
| `PH53-15` scenario examples | `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, `test_s7_examples.rs` |
| `PH53-16` corpus, property, fuzz | `test_golden_corpus.rs`, `test_property_corpus.rs`, `test_fuzz_seeds.rs` |
| `PH53-17` benchmarks and metrics | `test_bench_lanes.rs`, `test_metrics.rs` |
| `PH53-18` docs and examples | `test_readme_examples.rs`, `test_live_guardrails.rs` |
| `PH53-19` closeout and default gate | `test_default_gate.rs`, broad release gate |
| `PH53-20` legacy purge | `test_live_guardrails.rs`, `053-SUMMARY.md` |

## Canonical Targeted Commands

Use these commands when a change is local to one requirement family:

```bash
cargo test -p z00z_core --release --features test-fast --test assets_tests test_rights_config -- --nocapture
cargo test -p z00z_core --release --features test-fast --test genesis_tests test_genesis_rights -- --nocapture
cargo test -p z00z_core --release --features test-fast --test genesis_tests test_settlement_corpus -- --nocapture
cargo test -p z00z_core --release --features test-fast --test genesis_tests test_genesis_manifest -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_genesis_ingestion -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_settlement_root -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_hjmt_live_proof_families -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_forest_cache -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_async_scheduler -- --nocapture
cargo test -p z00z_simulator --release --features test-fast test_scenario_settlement -- --nocapture
```

## Packet Sync Rules

- Do not create duplicate `phase053` file names when an existing generic test
  home already owns the behavior.
- Do not reintroduce deleted phase-prefixed placeholder homes for store API or
  fee-envelope coverage; the live homes are `test_store_api.rs` and
  `test_fee_envelope.rs`.
- Keep canonical config and genesis authorities on the `z00z_core` side, store
  ingestion on the `z00z_storage` side, and scenario truth on the simulator
  side.
- If a future task expands validator-specific coverage, attach it to a real
  runnable validator test harness first; do not create empty placeholder homes
  only to satisfy a filename expectation.

## 2026-06-06 Additive Coverage

The last missing dedicated Phase 053 authority lanes now exist in the tree:

- `crates/z00z_core/tests/assets/test_rights_config.rs`
- `crates/z00z_core/tests/genesis/test_genesis_rights.rs`
- `crates/z00z_core/tests/genesis/test_genesis_manifest.rs`
- `crates/z00z_storage/tests/test_genesis_ingestion.rs`

These close the old packet drift where the spec required canonical rights and
genesis authority coverage but still pointed at future or nonexistent test
homes.
