---
phase: 052-HJMT-Backend
plan: 052-06
status: complete
completed: 2026-05-29
owner: Z00Z Storage
---

<!-- markdownlint-disable MD032 MD033 MD060 -->

# 052-06 Summary: Rollout Gating, Benchmarks, And Closeout

## Scope Delivered

- Kept `AssetBackendMode::Compatibility` as the default backend when
  `Z00Z_ASSET_BACKEND_MODE` is unset.
- Kept forest and dual-verify modes explicit through
  `Z00Z_ASSET_BACKEND_MODE=forest` and
  `Z00Z_ASSET_BACKEND_MODE=dual-verify`.
- Added fixed bucket-width rollout control through
  `Z00Z_ASSET_BUCKET_BITS` without adding adaptive split, merge, or migration
  behavior.
- Made durable dual-verify reload usable by persisting the forest path while
  preserving the compatibility semantic root as the active metadata root.
- Fixed forest-mode `scenario_1` storage-view serialization by building an
  inspection-only compatibility projection from public asset APIs, verifying
  the projected semantic root equals the live forest root, and keeping the
  serializer isolated from forest physical layout.
- Extended the landed benchmark harness and evidence home for bucket-width
  variants, async multi-insert, async multi-delete, inclusion proof timing,
  inclusion proof verification timing, absence rejection timing, recovery
  replay, nested forest variants, and runtime proof-size samples.
- Recorded Plan 06 benchmark evidence in
  `crates/z00z_storage/benches/assets/assets_benches.md`.

## Boundary Kept

- No public API accepts `TreeId`, namespace keys, branch ordering, raw backend
  roots, bucket ids, or physical layout as authority.
- `AssetStateRoot` remains the live semantic root and the compatibility oracle
  for backend equivalence.
- `CheckRoot`, `ProofBlob`, `chk_blob`, and storage-owned checkpoint contracts
  remain the downstream authority boundary.
- Deletion and non-existence proof families remain explicit fail-closed
  unsupported paths where the live wire format cannot yet validate them.
- Adaptive buckets, proof-visible occupancy counters, generalized settlement
  roots, `RightLeaf`, and `FeeEnvelope` remain first-class follow-up work, not
  live Phase 052 runtime exports.

## Benchmark Evidence

- Compile gates passed for `cargo bench -p z00z_storage --bench assets_shard
  --no-run` and `cargo bench -p z00z_storage --bench assets_nested --no-run`.
- Compatibility insert-many baseline:
  `crates/z00z_storage/outputs/assets/ph52_compat_insert_defs.md`.
- Forest bucket-width samples:
  `crates/z00z_storage/outputs/assets/ph52_forest_bucket4.md` and
  `crates/z00z_storage/outputs/assets/ph52_forest_bucket8_hot_serial.md`.
- Async multi-insert samples:
  `crates/z00z_storage/outputs/assets/ph52_forest_async_insert.md`.
- Async multi-delete samples:
  `crates/z00z_storage/outputs/assets/ph52_forest_async_delete.md`.
- Inclusion proof timing sample:
  `crates/z00z_storage/outputs/assets/ph52_forest_proof.md`.
- Inclusion proof verification timing sample:
  `crates/z00z_storage/outputs/assets/ph52_forest_verify.md`.
- Absence rejection timing sample:
  `crates/z00z_storage/outputs/assets/ph52_forest_absence.md`.
- Recovery replay sample:
  `crates/z00z_storage/outputs/assets/ph52_forest_recovery.md`.
- Nested forest hot-serial samples:
  `crates/z00z_storage/outputs/assets/ph52_nested_forest_hot_serial.md`.
- Runtime proof-size sample:
  `crates/z00z_storage/outputs/assets/assets_hjmt_proof_sizes.md`, with
  forest bucket bits `8`, inclusion proof sizes `1247` and `1314` bytes, and
  non-existence remaining `unsupported fail-closed`.

These numbers are implementation evidence only. They are not public
performance claims and must not become protocol constants.

## Validation

- Mandatory bootstrap gate passed before broader Plan 06 validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- After the forest serialization fix, bootstrap was rerun and passed again:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- After adding the proof verification benchmark timing lane, bootstrap was
  rerun and passed again:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`.
- Benchmark compile validation passed:
  `cargo bench -p z00z_storage --bench assets_shard --no-run`.
- Inclusion proof verification timing was measured through:
  `./crates/z00z_storage/scripts/run_storage_assets_bench.py --bench
  assets_shard --backend-mode forest --bucket-bits 8 --baseline
  ph52_forest_verify -- --sample-size 10 --measurement-time 1 --warm-up-time
  1 --noplot verify_many_assets`.
- Exact storage focused validation from the TODO passed:
  `cargo test -p z00z_storage --release --features test-fast`.
- Focused storage release validation passed:
  `cargo test -p z00z_storage --release --features test-fast --features
  wallet_debug_dump`.
- Simulator selector validation passed:
  `cargo test -p z00z_simulator --release --features wallet_debug_dump
  scenario_1`.
- Compatibility scenario run passed:
  `Z00Z_ASSET_BACKEND_MODE=compatibility cargo run --release -p
  z00z_simulator --bin scenario_1 --features wallet_debug_dump`.
- Forest scenario run passed:
  `Z00Z_ASSET_BACKEND_MODE=forest cargo run --release -p z00z_simulator
  --bin scenario_1 --features wallet_debug_dump`.
- Dual-verify scenario run passed:
  `Z00Z_ASSET_BACKEND_MODE=dual-verify cargo run --release -p z00z_simulator
  --bin scenario_1 --features wallet_debug_dump`.
- Broad release validation passed:
  `cargo test --release --features test-fast --features wallet_debug_dump`.
- The scenario binary runs completed all 13 stages and reported
  `scenario_1.result: success` in compatibility, forest, and dual-verify
  modes.

## Review Loop

- `/GSD-Review-Tasks-Execution` pass 1 found one significant closeout issue:
  the code and test gates were green, but `052-06-SUMMARY.md`, TODO checklist
  state, benchmark evidence, and roadmap/state handoff were still incomplete.
- The closeout artifacts were updated with source-backed evidence and explicit
  deferred work.
- `/GSD-Review-Tasks-Execution` pass 2 reported no significant issues after
  the evidence updates.
- `/GSD-Review-Tasks-Execution` pass 3 reported no significant issues after
  the roadmap/state handoff and diff checks.

## Deferred Follow-Ups

- `052-07` must audit the implemented `052-01` through `052-06` green state
  and keep deferred candidates first-class.
- `052-08` owns adaptive bucket split, merge, and migration proof machinery as
  a future candidate after fixed-bucket evidence.
- `052-09` owns proof-visible bucket occupancy metadata privacy review and any
  future design update.
- `052-10` owns generalized settlement-root migration design.
- `052-11` owns `RightLeaf` and `FeeEnvelope` protocol semantics.

## Next Plan

Execution moves to `052-07-PLAN.md` for the green-state audit and first-class
follow-up ledger.
