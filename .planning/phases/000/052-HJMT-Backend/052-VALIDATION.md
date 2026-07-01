---
phase: 052
slug: 052-hjmt-backend
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-29
updated: 2026-05-29
---

# Phase 052 - Validation Strategy

Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust workspace `cargo test` + storage benchmark compile and run lanes |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | ~1800 seconds |

## Sampling Rate

- After every Rust or test-affecting task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- After every plan wave: run `cargo test --release --features test-fast --features wallet_debug_dump`
- Before `/gsd-verify-work`: full suite must be green
- Max feedback latency: 1800 seconds

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 052-01-01 | 01 | 1 | `PH52-BACKEND-MODE` | `T-052-01` | Compatibility default, forest or dual-verify explicit, unknown mode rejects. | unit + integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite --test test_phase051_guardrails -- --nocapture` | ✅ | ✅ green |
| 052-01-02 | 01 | 1 | `PH52-BUCKET-POLICY` | `T-052-02` / `T-052-03` | Fixed bucket policy deterministic, versioned, and not caller authority. | unit + integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture` | ✅ | ✅ green |
| 052-02-01 | 02 | 2 | `PH52-FOREST-LAYOUT` | `T-052-05` / `T-052-07` | Real forest tree layout stays storage-private. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_backend -- --nocapture` | ✅ | ✅ green |
| 052-02-02 | 02 | 2 | `PH52-BATCH-PLANNER` | `T-052-06` | Insert or delete planner rejects before mutation and preserves state. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_backend -- --nocapture` | ✅ | ✅ green |
| 052-03-01 | 03 | 3 | `PH52-JOURNAL-RECOVERY` | `T-052-08` | Child-before-parent durability and crash replay stay fail-closed. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_recovery -- --nocapture` | ✅ | ✅ green |
| 052-03-02 | 03 | 3 | `PH52-RELOAD-INDEX` | `T-052-09` / `T-052-10` | Reload rebuilds index from durable rows and rejects root drift. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_redb_rehydrate --test test_checkpoint_root_binding -- --nocapture` | ✅ | ✅ green |
| 052-04-01 | 04 | 4 | `PH52-PROOF-ENVELOPE` | `T-052-11` | Storage-owned forest proof envelope verifies chained semantic segments. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_proofs -- --nocapture` | ✅ | ✅ green |
| 052-04-02 | 04 | 4 | `PH52-ABSENCE-PROOFS` | `T-052-12` / `T-052-13` | Deletion and non-existence stay real fail-closed or explicit unsupported. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_forest_proofs --test test_phase051_golden_corpus -- --nocapture` | ✅ | ✅ green |
| 052-05-01 | 05 | 5 | `PH52-EQUIVALENCE` | `T-052-14` | Compatibility stays oracle and dual-verify makes drift fatal. | integration | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase051_golden_corpus --test test_phase052_forest_backend -- --nocapture` | ✅ | ✅ green |
| 052-05-02 | 05 | 5 | `PH52-CHECKPOINT-GUARDRAILS` | `T-052-15` / `T-052-16` | Downstream consumers stay semantic-root and proof-authority clients only. | integration + source guardrail | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails --test test_checkpoint_root_binding -- --nocapture` | ✅ | ✅ green |
| 052-06-01 | 06 | 6 | `PH52-ROLLOUT-BENCHMARKS` | `T-052-17` / `T-052-18` | Compatibility default persists and benchmark evidence is recorded through landed harness. | integration + benchmark | `cargo test -p z00z_simulator --release --features wallet_debug_dump scenario_1 -- --nocapture` | ✅ | ✅ green |
| 052-06-02 | 06 | 6 | `PH52-CLOSEOUT` | `T-052-19` | Full validation, review loop evidence, and honest deferred scope are recorded. | full suite | `cargo test --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 052-07-01 | 07 | 7 | `PH52-GREEN-AUDIT` | `T-052-20` / `T-052-21` / `T-052-22` | Green-state audit blocks silent promotion of unfinished or non-live scope. | source guardrail | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture` | ✅ | ✅ green |
| 052-08-01 | 08 | 8 | `PH52-ADAPTIVE-BUCKETS-FOLLOWUP` | `T-052-23` / `T-052-24` / `T-052-25` | Adaptive split, merge, migration, and crash semantics stay future-only. | source guardrail | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture` | ✅ | ✅ green |
| 052-09-01 | 09 | 9 | `PH52-OCCUPANCY-METADATA-FOLLOWUP` | `T-052-26` / `T-052-27` / `T-052-28` | Proof-visible occupancy counters remain blocked and absent from live proof metadata. | source guardrail | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture` | ✅ | ✅ green |
| 052-10-01 | 10 | 10 | `PH52-GENERALIZED-ROOT-FOLLOWUP` | `T-052-29` / `T-052-30` / `T-052-31` | `AssetStateRoot` remains live oracle; `SettlementStateRoot` stays future-only migration work. | source guardrail | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture` | ✅ | ✅ green |
| 052-11-01 | 11 | 11 | `PH52-RIGHTLEAF-FEEENVELOPE-FOLLOWUP` | `T-052-32` / `T-052-33` / `T-052-34` | `RightLeaf` and `FeeEnvelope` stay separate future-only contracts, not live exports. | source guardrail | `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture` | ✅ | ✅ green |

Status values: `⬜ pending`, `✅ green`, `❌ red`, `⚠️ flaky`

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

## Manual-Only Verifications

All phase behaviors have automated verification.

## Validation Audit 2026-05-29

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

Gap resolved:
- Added phase-artifact consistency coverage to [test_phase052_guardrails.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/test_phase052_guardrails.rs) so plans `052-07` through `052-11` are validated by automated guardrails rather than summary text only.

## Execution Evidence 2026-05-29

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> green
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_phase052_guardrails -- --nocapture` -> green (`10 passed`)
- `cargo test --release --features test-fast --features wallet_debug_dump` -> green
- `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump` -> green (`scenario_1.result: success`, `stage_count=13`)

## Validation Sign-Off

- [x] All tasks have automated verify or existing infrastructure coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency < 1800s
- [x] `nyquist_compliant: true` set in frontmatter

Approval: approved 2026-05-29
