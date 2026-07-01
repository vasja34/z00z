---
phase: 054
slug: refactor-crates
status: audited
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-08
updated: 2026-06-09
---

# Phase 054 - Validation Strategy

Reconstructed Nyquist validation contract for Phase 054 from completed plan,
summary, security, and live test artifacts. Refreshed on 2026-06-09 after the
bounded `054-08` continuation closed the runtime digest-routing follow-up.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust workspace `cargo test` plus repository bootstrap checks and source guards |
| **Config file** | Workspace [Cargo.toml](/home/vadim/Projects/z00z/Cargo.toml) |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release` |
| **Estimated runtime** | ~1800 seconds, workspace and cache dependent |

## Sampling Rate

- After every Rust or test-affecting task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- After every plan wave: run the narrow task-local cargo tests from the owning `054-XX-PLAN.md`, then run `cargo test --release`; keep `cargo test --all --release -q` as historical closeout evidence for the 2026-06-08 baseline and re-run `cargo test --release --features test-fast --features wallet_debug_dump` only when checking the manifest-only feature-name blocker
- Before `/gsd-verify-work`: bootstrap plus the targeted phase suites and `cargo test --release` must be green
- Max feedback latency: bounded by bootstrap plus the targeted phase-local release suites, with the broad release workspace rerun reserved for convergence points
- Current blocker note: the exact feature-name command is stale on missing `test-fast` and `wallet_debug_dump` and is recorded as an informational manifest blocker, not a Phase 054 regression

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| `T054-01` | 01 | 1 | `PH54-01` | `T-054-01 / 02 / 03` | Backend seam stays below the semantic settlement surface and `StoreBackendError` remains the stable backend error contract. | integration + source guard | `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails --test test_downstream_guardrails -q` | ✅ | ✅ green |
| `T054-02` | 02 | 2 | `PH54-02` | `T-054-04 / 05 / 06` | Backend extraction preserves settlement semantics and does not widen fake or duplicate backend authority. | integration | `cargo test -p z00z_storage --release --features test-params-fast -q` | ✅ | ✅ green |
| `T054-03` | 03 | 3 | `PH54-03` | `T-054-07 / 08 / 09` | Runtime planner ownership stays metadata-only while storage keeps semantic planning helpers and proof authority. | cross-crate integration | `cargo test --release -p z00z_aggregators -q && cargo test -p z00z_storage --release --features test-params-fast -q` | ✅ | ✅ green |
| `T054-04` | 04 | 3 | `PH54-04` | `T-054-10 / 11 / 12` | Placement and shard execution remain runtime-owned operational metadata while node orchestration stays at the rollup root. | cross-crate integration | `cargo test --release -p z00z_aggregators -p z00z_validators -p z00z_watchers -q && cargo test -p z00z_rollup_node --release --features test-params-fast -q` | ✅ | ✅ green |
| `T054-05` | 05 | 4 | `PH54-05` | `T-054-13 / 14 / 15` | Storage cleanup leaves one canonical helper path and removes bridge or shim seams from the live store tree. | integration + source guard | `cargo test -p z00z_storage --release --features test-params-fast -q` | ✅ | ✅ green |
| `T054-06` | 06 | 5 | `PH54-06` | `T-054-16 / 17 / 18` | Rename fallout stays behavior-neutral, keeps one public facade, and does not reintroduce duplicate public paths. | cross-crate integration | `cargo test --release -p z00z_aggregators -p z00z_validators -p z00z_watchers -q && cargo test -p z00z_storage --release --features test-params-fast -q && cargo test -p z00z_rollup_node --release --features test-params-fast -q && cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards -q` | ✅ | ✅ green |
| `T054-07` | 07 | 6 | `PH54-07` | `T-054-19 / 20 / 21` | Docs, migration tables, and final path truth stay aligned to the landed topology with no alias or shim return path. | doc guard + integration | `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -q && cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards -q` | ✅ | ✅ green |
| `T054-08` | 08 | 7 | `PH54-08` | `AS-20260609-001 / T-054-22` | Runtime planner ingress rebinding stays fail closed, route lookup and `plan_digest` stay payload-bound, and the live public planner lane keeps one canonical `WorkItem` path with no digest-authority bypass. | integration + source guard | `cargo test -p z00z_aggregators --release -q && cargo test --release` | ✅ | ✅ green |

Status legend: `⬜ pending` · `✅ green` · `❌ red` · `⚠️ flaky`

## Wave 0 Requirements

Existing infrastructure covers all Phase 054 requirements.

No new framework install or helper harness was required beyond the repository
bootstrap gate, targeted crate release suites, and the existing source-guard
tests. The single missing Nyquist gap was closed by extending the live
guardrail suite instead of adding a parallel test surface.

## Manual-Only Verifications

Phase 054 behavior is automation-backed, but the final truth sync also required
local review passes over the planning packet.

Four local `/GSD-Review-Tasks-Execution`-equivalent passes were completed
during the final release-only refresh:

- Pass 1 rechecked the live code and path surfaces; no material alias or shim
  issue remained in the phase-owned runtime, node, or storage trees.
- Pass 2 found one material planning-truth issue: several closeout documents
  still claimed `cargo test --all` remained blocked even though the final
  release rerun had already passed.
- Pass 3 reran the same review after the doc fixes; no significant issues
  remained.
- Pass 4 repeated the review again; no significant issues remained, giving the
  required consecutive clean pair.

Four scoped `/GSD-Review-Tasks-Execution` passes were then completed for the
bounded `054-08` continuation:

- Pass 1 found one material regression-coverage gap: the code had already
  removed the public `WorkItem` bypass, but no external-facing source guard
  yet proved that invariant as a public API contract.
- Pass 2 reran the task review after adding the new
  `crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs` coverage and
  found only planning-truth drift in the attack-surface, security, validation,
  roadmap, and state artifacts.
- Pass 3 reran the code and docs audit after the truth sync; no significant
  issues remained.
- Pass 4 repeated the same audit again; no significant issues remained, giving
  the required consecutive clean pair for `054-08`.

## Validation Audit 2026-06-08

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

Gap resolved:

- Added canonical backend-path regression coverage to
  [test_live_guardrails.rs](/home/vadim/Projects/z00z/crates/z00z_storage/tests/test_live_guardrails.rs:401)
  with `test_store_backend_paths_stay_canonical()` so the live
  `crates/z00z_storage/src/settlement/store/*` owners now fail if
  `store_codec`, `store_roots`, or `store_mem` alias or shim paths reappear.

## Release-Only Revalidation Notes 2026-06-08

- The final release-only rerun found and fixed a suite-only simulator
  regression class: runtime wallet unlock paths now prefer the actor-owned
  canonical password field instead of a process-global password map.
- The same rerun serialized the claim-registry tests that share global rows and
  corrected the wallet deterministic-object contract so the non-fast release
  lane compares decrypted derivation payload semantics instead of random-nonce
  ciphertext bytes.

## Execution Evidence 2026-06-08

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` — passed
- `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -q` — passed after adding the canonical-path regression guard
- `cargo test --release -p z00z_aggregators -p z00z_validators -p z00z_watchers -q` — passed
- `cargo test -p z00z_storage --release --features test-params-fast -q` — passed
- `cargo test -p z00z_rollup_node --release --features test-params-fast -q` — passed
- `cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards -q` — passed
- `cargo test -p z00z_wallets --release -q` — passed
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools -q` — passed
- `cargo test --all --release -q` — passed
- `cargo test --release --features test-fast --features wallet_debug_dump` — failed immediately because the current workspace packages no longer expose `test-fast` or `wallet_debug_dump`
- `git diff --check` — passed

## Re-Audit 2026-06-08

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

State A re-audit result:

- No new requirement-to-test coverage gaps were found.
- The canonical-path regression guard remains live and the phase still enforces
  one canonical module-path story across the phase-owned storage, runtime, and
  node surfaces.

## Re-Audit Evidence 2026-06-08

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` — passed
- `rg -n "store_codec::|store_roots::|store_mem::| as store_codec| as store_roots| as store_mem" crates/z00z_storage/src/settlement/store crates/z00z_storage/tests/test_live_guardrails.rs` — no live store implementation hits; only the guard test literals matched
- `cargo test -p z00z_storage --release --features test-params-fast --test test_live_guardrails -q` — passed
- `cargo test --release -p z00z_aggregators -p z00z_validators -p z00z_watchers -q` — passed
- `cargo test -p z00z_rollup_node --release --features test-params-fast -q` — passed
- `cargo test -p z00z_wallets --release --features test-params-fast --test test_rename_guards -q` — passed
- `cargo test -p z00z_wallets --release -q` — passed
- `cargo test -p z00z_simulator --release --features test-params-fast --features wallet_debug_tools -q` — passed
- `cargo test --all --release -q` — passed
- `cargo test --release --features test-fast --features wallet_debug_dump` — still failed immediately because the current workspace packages do not expose `test-fast` or `wallet_debug_dump`
- `git diff --check` — passed

## Post-Closeout Continuation 2026-06-09

| Metric | Count |
|--------|-------|
| Gaps found | 1 |
| Resolved | 1 |
| Escalated | 0 |

Gap resolved:

- Added
  [test_live_guardrails.rs](/home/vadim/Projects/z00z/crates/z00z_runtime/aggregators/tests/test_live_guardrails.rs:1)
  so the live `z00z_aggregators` public lane now fails the release suite if
  `WorkItem` regains a public constructor or public fields, if ingress stops
  owning planner normalization, or if `BatchPlanner` returns to decoding raw
  caller digest strings for routing.

## Execution Evidence 2026-06-09

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` — passed
- `cargo test -p z00z_aggregators --release -q` — passed
- `cargo test --release` — passed
- `cargo doc --no-deps` — passed with pre-existing rustdoc warnings outside the
  `054-08` scope
- `git diff --check` — passed

## Validation Sign-Off

- [x] All tasks have automated verify coverage or existing infrastructure coverage
- [x] Sampling continuity: no 3 consecutive tasks without automated verify evidence
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Feedback latency stays bounded by bootstrap plus targeted phase-local checks
- [x] `nyquist_compliant: true` set in frontmatter

Approval: approved 2026-06-09

## Reconstruction Notes

This file was reconstructed under validate-phase State B because
`054-VALIDATION.md` was missing and `gsd-sdk` is unavailable in this executor
environment.

Inputs used for the reconstruction:

- `054-01-PLAN.md` through `054-08-PLAN.md`
- `054-01-SUMMARY.md` through `054-08-SUMMARY.md`
- `054-SUMMARY.md`
- `054-SOURCE-AUDIT.md`
- `054-SECURITY.md`

Gap audit result: `1` missing automated requirement-to-test coverage gap,
resolved in scope.
