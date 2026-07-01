---
phase: 053-HJMT-Backend
artifact: tests-tasks
status: implemented
source:
  - 053-TEST-SPEC.md
  - 053-TODO.md
  - 053-SUMMARY.md
updated: 2026-06-06
---

<!-- markdownlint-disable MD060 -->

# Phase 053 Test Implementation Tasks

**Phase:** `053-HJMT-Backend`
**Status:** implemented and packet-synced
**Companion spec:** `053-TEST-SPEC.md`

## Objective

This file is now an execution ledger for the test work that proves Phase 053.
It records which live repository suites own each numbered requirement and what
must still remain true when those suites are edited in the future.

## Rules For Future Changes

- Reuse the existing live test home that already owns the behavior.
- Do not create duplicate `phase053` wrappers around a generic test file that
  already exercises the production seam.
- Keep new technical content in English.
- Do not edit `crates/z00z_crypto/tari/`.
- When a task requires a git commit, use `/z00z-git-versioning`.

## Verify Block Template

Every Rust or test-affecting change that touches these tasks must end with:

```bash
./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
```

If that fails, stop, fix, and rerun it before any broader validation.

Then run targeted commands for the touched area, followed by:

```bash
cargo test --release --features test-fast --features wallet_debug_dump
```

Finally run `/.github/prompts/gsd-review-tasks-execution.prompt.md`
(`/GSD-Review-Tasks-Execution`) in YOLO mode at least three times and continue
until at least two consecutive runs show no significant issues or warnings.

## Execution Ledger

| Task | Status | Live owners | Current truth |
| --- | --- | --- | --- |
| `T053-01` guardrails and live contract shape | implemented | `test_live_guardrails.rs`, `test_default_gate.rs` | Live settlement exports are required, stale aliases stay rejected, and docs cannot drift back to future-only wording. |
| `T053-02` settlement root generation and checkpoints | implemented | `test_settlement_root.rs`, `test_checkpoint_store_api.rs` | `SettlementStateRoot` and generation binding are live; old root/generation substitutions reject. |
| `T053-03` settlement path and right leaf surface | implemented | `test_store_api.rs`, `test_hjmt_live_proof_families.rs`, `test_genesis_ingestion.rs` | Asset/right families share one settlement surface without fee/right confusion or path-family drift. |
| `T053-04` fee envelope and replay protection | implemented | `test_fee_envelope.rs`, `test_fee_replay.rs` | Fee support remains separate from ownership and replay rejection survives reload. |
| `T053-05` store API hard cutover | implemented | `test_store_api.rs`, `test_downstream_guardrails.rs`, `test_default_gate.rs` | HJMT is the only live storage mode and downstream callers must use semantic APIs. |
| `T053-06` YAML, genesis rights, and ingestion | implemented | `test_rights_config.rs`, `test_genesis_rights.rs`, `test_genesis_manifest.rs`, `test_settlement_corpus.rs`, `test_genesis_ingestion.rs` | Canonical config and genesis authorities generate deterministic mixed settlement corpus, export manifest evidence, and storage can ingest the produced rights. |
| `T053-07` proof envelope v2 | implemented | `test_hjmt_live_proof_families.rs`, `test_settlement_root.rs` | Inclusion, deletion, and non-existence families are live and reject wrong root, family, or binding. |
| `T053-08` adaptive buckets and policy proofs | implemented | `test_hjmt_adaptive_policy_proofs.rs`, `test_bench_lanes.rs` | Split, merge, epoch, stale-proof, and policy-transition lanes are exercised through production APIs. |
| `T053-09` occupancy privacy evidence | implemented | `test_occupancy_privacy.rs`, `test_occupancy_evidence.rs` | Proof-visible occupancy remains bounded and raw-counter leakage stays rejected. |
| `T053-10` forest cache plane | implemented | `test_forest_cache.rs`, `test_cache_recompute.rs` | Cache reuse is private optimization only and recomputation must fail closed on mismatch. |
| `T053-11` async scheduler | implemented | `test_async_scheduler.rs` | Parallel execution remains deterministic and rollback-safe. |
| `T053-12` journal and recovery | implemented | `test_hjmt_adaptive_policy_proofs.rs`, `test_redb_reload.rs` | Durable mutation and policy-transition recovery stay fail-closed. |
| `T053-13` RedB reload and historical proofs | implemented | `test_redb_reload.rs`, `test_bench_lanes.rs` | Reopen reproduces semantic state and rejects unsupported or stale row shapes. |
| `T053-14` downstream integration | implemented | `test_downstream_guardrails.rs`, `test_stage7_jmt_wallet_scan.rs`, `test_s7_examples.rs` | Downstream consumers verify semantic proofs first and do not treat `RightLeaf` as wallet-owned asset inventory. |
| `T053-15` scenario examples | implemented | `test_scenario_settlement.rs`, `test_scenario1_stage_surface.rs`, `test_s7_examples.rs` | Stage 1/4/6/11/13 scenario output is rights-aware, mixed-family, and replay-verifiable. |
| `T053-16` golden corpus, property, and fuzz seeds | implemented | `test_golden_corpus.rs`, `test_property_corpus.rs`, `test_fuzz_seeds.rs` | Canonical mixed corpus, randomized operation sequences, and malformed proof lanes remain stable. |
| `T053-17` benchmarks and metrics | implemented | `test_bench_lanes.rs`, `test_metrics.rs` | Required bench lanes compile and measurement surfaces remain wired to live HJMT code. |
| `T053-18` docs and API examples | implemented | `test_readme_examples.rs`, `test_live_guardrails.rs` | Public docs and README examples stay executable and cannot regress to stale semantics. |
| `T053-19` closeout and default gate | implemented | `test_default_gate.rs`, broad release gate | Unset backend mode resolves to HJMT, stale aliases reject, and the broad workspace release gate remains the final closeout proof. |
| `T053-20` legacy purge | implemented | `test_live_guardrails.rs`, `053-SUMMARY.md` | Compatibility/simple-JMT runtime tails stay deleted from the live tree and packet wording stays truthful. |

## 2026-06-06 Packet Sync Delta

These were the only remaining dedicated authority gaps when this file was
synced:

- Added `crates/z00z_core/tests/assets/test_rights_config.rs`
- Added `crates/z00z_core/tests/genesis/test_genesis_rights.rs`
- Added `crates/z00z_core/tests/genesis/test_genesis_manifest.rs`
- Added `crates/z00z_storage/tests/test_genesis_ingestion.rs`

The rest of the Phase 053 matrix was already live in the repository, but the
packet still pointed at planned or nonexistent target files.

## Canonical Targeted Commands

Use the narrowest relevant command set before the broad release gate:

```bash
cargo test -p z00z_core --release --features test-fast --test assets_tests test_rights_config -- --nocapture
cargo test -p z00z_core --release --features test-fast --test genesis_tests test_genesis_rights -- --nocapture
cargo test -p z00z_core --release --features test-fast --test genesis_tests test_settlement_corpus -- --nocapture
cargo test -p z00z_core --release --features test-fast --test genesis_tests test_genesis_manifest -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_genesis_ingestion -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_hjmt_live_proof_families -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_hjmt_adaptive_policy_proofs -- --nocapture
cargo test -p z00z_storage --release --features test-fast --test test_redb_reload -- --nocapture
cargo test -p z00z_simulator --release --features test-fast test_scenario_settlement -- --nocapture
```

## Maintenance Notes

- Keep `053-TEST-SPEC.md` and this ledger aligned with actual repository file
  names, not with hypothetical future names.
- If a new suite is added, update the owning row instead of appending a second
  conflicting target path for the same requirement.
- If validator-specific coverage is introduced later, attach it to a real
  runnable validator test harness first; do not create empty placeholder
  owners.
