---
phase: 059-Core-Upgrade
artifact: tests-tasks
status: implemented-closeout
source: 059-TEST-SPEC.md
updated: 2026-06-18
---

# Phase 059 Test Tasks

**Phase:** 059-Core-Upgrade
**Date:** 2026-06-16
**Source:** `059-TEST-SPEC.md`

## Workflow Status

This artifact is the implementation-backed closeout packet for test work. The
numbered Phase 059 summaries now prove that Waves `T0` through `T5` landed on
the live repository seams. `T6` owns the final evidence ledger, UAT packet,
docs sync, release-only reruns, and phase-summary closeout.

Future/target wording from `059-TODO.md` and the referenced corpus remains
mandatory Phase 059 scope in this task packet. Here `target` is planning
vocabulary for not-yet-landed code, not optional backlog.

Closeout rule: when an early planning alias differs from the landed filename,
this packet must point to the landed canonical home instead of preserving the
placeholder alias as if it were still live.

## Execution Strategy

Test implementation must follow the object dependency graph:

1. Lock existing anchors and regression baselines before adding new object
   families.
2. Land core descriptors/config/genesis tests before storage, wallet, runtime,
   or simulator tests that depend on those types, and bind each concept to one
   canonical core module home (`assets`, `actions`, `policies`, `rights`,
   `vauchers`) instead of duplicating semantic anchors.
3. Land storage leaf/proof/delta tests before wallet and simulator flows that
   rely on family-specific roots and proofs.
4. Land runtime validator/watcher tests before treating simulator flows as
   accepted protocol behavior.
5. Land wallet projection/RPC/backup tests before exposing user-facing object
   actions.
6. Land simulator Alice/Bob/Charlie E2E tests only after core, storage, wallet,
   and runtime seams have real Phase 059 APIs.

The landed packet now follows that order:

- `059-02` closed the canonical core vocabulary roots.
- `059-03` closed typed genesis policies/vouchers/publication.
- `059-04` and `059-05` closed storage family/proof/delta boundaries.
- `059-06` closed runtime validator/watcher/rollup object verdict surfaces.
- `059-07` and `059-08` closed wallet inventory/RPC/backup object flows.
- `059-09` closed simulator Alice/Bob/Charlie object-family evidence.

## Task Waves

### Wave T0: Harness And Reuse Lock-In

- Inspect and preserve existing anchors listed in `059-TEST-SPEC.md`.
- Add `059-EVIDENCE-LEDGER.md` rows for every `059-*` test ID before closing
  implementation.
- Confirm no test introduces a separate object root, object store, wallet cash
  model, or simulator runner outside existing seams.
- Completion gate: all existing baseline tests selected for extension still
  compile before new object-family assertions are added.

### Wave T1: Core Object Vocabulary, Policies, And Genesis

- Owns: `059-UT-CORE-*`, `059-PATH-001`, `059-NEG-001`, `059-NEG-002`,
  `059-NEG-006`, `059-NEG-008`, `059-NEG-013`, `059-REG-002`.
- Files: core assets/genesis tests plus canonical `actions` or `policies` or
  `rights` or `vauchers` test files named in the concrete file anchors below.
- Completion gate: object descriptors are canonical, deterministic, and
  reject unbacked vouchers, value-bearing rights, arbitrary native cash action
  pools, and descriptor mutation.

### Wave T2: Storage Leaf, Proof, Delta, And Recovery

- Owns: `059-UT-STOR-*`, `059-PATH-002`, `059-PATH-005`, `059-NEG-003`,
  `059-NEG-004`, `059-NEG-009`, `059-NEG-010`, `059-PROP-002`,
  `059-PROP-003`, `059-REG-003`.
- Files: settlement model/proof/batch/recovery tests plus typed-delta tests.
- Completion gate: Terminal, Right, and Voucher families share one
  `SettlementStateRoot` and one `SettlementPath` without tag drift.

### Wave T3: Runtime, Aggregator, Watcher, And Rollup Evidence

- Owns: `059-UT-RUN-*`, `059-PATH-004`, `059-PATH-005`, `059-PATH-008`,
  `059-NEG-005`, `059-NEG-007`, `059-NEG-014`, `059-REG-006`.
- Files: runtime validator/watcher tests, aggregator carriage tests, and
  rollup publication tests.
- Completion gate: every rejection has a precise verdict, root behavior, and
  watcher/publication evidence without wallet-secret leakage.

### Wave T4: Wallet Inventory, Projection, RPC, And Backup

- Owns: `059-UT-WAL-*`, `059-PATH-003`, `059-PATH-004`, `059-PATH-007`,
  `059-NEG-005`, `059-NEG-007`, `059-NEG-011`, `059-NEG-012`,
  `059-PROP-004`, `059-REG-004`.
- Files: wallet DB/service/RPC/backup tests named in the anchors below.
- Completion gate: spendable cash includes Assets only; Vouchers, Rights, and
  unknown-policy objects remain separate, durable projections.

### Wave T5: Simulator Alice/Bob/Charlie E2E Evidence

- Owns: `059-E2E-SIM-001` through `059-E2E-SIM-015`, `059-PATH-006`,
  `059-REG-001`, `059-REG-005`.
- Files: simulator `scenario_1` stage tests and
  `test_scenario1_object_flows.rs`.
- Completion gate: each scenario produces persisted fixtures, roots, verdicts,
  wallet projections, watcher records, and fix reports for negative paths.

### Wave T6: Property/Fuzz, Ledger, Review, And Release Closeout

- Owns: `059-PROP-001` through `059-PROP-004`, all `059-NEG-*`, all
  `059-REG-*`, all `059-PATH-*`, and final release evidence.
- Files: crate-local property/fuzz targets, simulator evidence index, docs, and
  `059-EVIDENCE-LEDGER.md`.
- Completion gate: every test ID maps to an implementation file, command,
  simulator artifact, existing equivalent, or explicit deferral.

## Task Routing

| Plan | Test task ownership |
|---|---|
| `059-02` | Core object vocabulary, descriptor hashing, native cash policy rejection, value-bearing right rejection. |
| `059-03` | Genesis schema, deterministic rights/vouchers/policies, manifest export, compatibility for existing configs. |
| `059-04` | Voucher leaf codec, family tags, proof/nonexistence/batch proof, cache decode compatibility. |
| `059-05` | Typed mixed-object deltas, conservation, residual voucher, double redeem, right zero-value, fee boundary. |
| `059-06` | Validator verdicts, aggregator package carriage, watcher alerts, rollup publication evidence. |
| `059-07` | Wallet DB migration, typed payloads, indexes, durable quarantine, old asset payload compatibility. |
| `059-08` | Wallet scan/receive/package/RPC/backup behavior and cash projection separation. |
| `059-09` | Simulator Alice/Bob/Charlie positive and negative object interaction paths. |
| `059-10` | Full coverage closure, release-mode simulator evidence, fuzz/property smoke, docs and final gates. |

## Test ID Ownership

| Plan | Required test IDs |
|---|---|
| `059-02` | `059-UT-CORE-001`, `059-UT-CORE-002`, `059-UT-CORE-003`, `059-UT-CORE-004`, `059-NEG-001`, `059-NEG-006`, `059-NEG-008`, `059-NEG-013` |
| `059-03` | `059-UT-CORE-005`, `059-UT-CORE-006`, `059-UT-CORE-007`, `059-PATH-001`, `059-PATH-002`, `059-NEG-002`, `059-REG-002` |
| `059-04` | `059-UT-STOR-001`, `059-UT-STOR-002`, `059-UT-STOR-003`, `059-UT-STOR-005`, `059-PATH-002`, `059-PATH-005`, `059-NEG-009`, `059-PROP-002`, `059-REG-003` |
| `059-05` | `059-UT-STOR-004`, `059-NEG-003`, `059-NEG-004`, `059-NEG-010`, `059-PROP-003` |
| `059-06` | `059-UT-RUN-001`, `059-UT-RUN-002`, `059-UT-RUN-003`, `059-UT-RUN-004`, `059-PATH-004`, `059-PATH-005`, `059-PATH-008`, `059-NEG-005`, `059-NEG-007`, `059-NEG-014`, `059-REG-006` |
| `059-07` | `059-UT-WAL-001`, `059-UT-WAL-002`, `059-UT-WAL-005`, `059-PATH-003`, `059-PATH-007`, `059-NEG-011`, `059-PROP-004`, `059-REG-004` |
| `059-08` | `059-UT-WAL-003`, `059-UT-WAL-004`, `059-UT-WAL-005`, `059-PATH-003`, `059-PATH-004`, `059-PATH-007`, `059-NEG-005`, `059-NEG-007`, `059-NEG-012` |
| `059-09` | `059-E2E-SIM-001`, `059-E2E-SIM-002`, `059-E2E-SIM-003`, `059-E2E-SIM-004`, `059-E2E-SIM-005`, `059-E2E-SIM-006`, `059-E2E-SIM-007`, `059-E2E-SIM-008`, `059-E2E-SIM-009`, `059-E2E-SIM-010`, `059-E2E-SIM-011`, `059-E2E-SIM-012`, `059-E2E-SIM-013`, `059-E2E-SIM-014`, `059-E2E-SIM-015`, `059-PATH-006`, `059-REG-001`, `059-REG-005`, plus persisted positive and negative simulator artifacts |
| `059-10` | `059-PATH-001` through `059-PATH-008`, `059-PROP-001` through `059-PROP-004`, all `059-NEG-*`, all `059-REG-*`, final release evidence, final `059-EVIDENCE-LEDGER.md` traceability |

Every ID in `059-TEST-SPEC.md` must be represented in one of these plan rows,
implemented as a test, linked to an equivalent existing test, or marked as an
explicit deferral in `059-EVIDENCE-LEDGER.md`.

If compatibility re-exports remain temporarily, the ledger must still name one
canonical semantic owner path and mark every other path as compatibility-only.

## Required Targeted Commands

The executor should select the narrowest relevant commands for each slice, but
these command families must be represented across the phase:

- `cargo test -p z00z_core --release`
- `cargo test -p z00z_storage --release`
- `cargo test -p z00z_wallets --release`
- `cargo test -p z00z_simulator --release`
- `cargo test -p z00z_aggregators --release`
- `cargo test -p z00z_validators --release`
- `cargo test -p z00z_watchers --release`
- `cargo test -p z00z_rollup_node --release`
- `cargo test --release`

Before every command family above, run:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

After targeted and full release tests, run:

- `./.github/prompts/gsd-review-tasks-execution.prompt.md`
  (`/GSD-Review-Tasks-Execution`) in YOLO mode at least 3 times, fixing all
  issues and warnings, stopping only after 2 consecutive runs show no
  significant code issues.

## Minimum Landed Test Areas And Anchor Patterns

| Area | Files or patterns |
|---|---|
| Core | `crates/z00z_core/src/assets/test_*`, `crates/z00z_core/src/{actions,policies,rights,vauchers}/test_*`, `crates/z00z_core/src/genesis/test_*`, plus compatibility harnesses only where migration requires them. |
| Storage | `crates/z00z_storage/src/settlement/test_*`, `crates/z00z_storage/tests/test_*settlement*`, proof/batch/cache/recovery/fuzz targets. |
| Wallet DB | `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`, migration tests, backup import/export tests. |
| Wallet services/RPC | `crates/z00z_wallets/src/adapters/rpc/methods/test_*`, wallet receive/package/action tests. |
| Runtime | `crates/z00z_runtime/validators/tests/*`, `crates/z00z_runtime/watchers/tests/*`, aggregator tests where package carriage changes. |
| Simulator | `crates/z00z_simulator/tests/*`, `crates/z00z_simulator/src/scenario_1/*test*`, shared cases under `src/test_support`. |
| Fuzz/property | Core/storage fuzz targets for descriptor hash, leaf codec, proof envelope, and conservation/residual arithmetic. |

## Concrete Canonical Test File Anchors

| Test ID group | Required file anchors |
|---|---|
| `059-UT-CORE-001` through `059-UT-CORE-004` | Extend `crates/z00z_core/src/assets/test_policy_descriptor.rs`, `crates/z00z_core/src/assets/test_voucher_config.rs`, `crates/z00z_core/src/rights/test_rights_config.rs`, and keep `crates/z00z_core/tests/assets/test_rights_config.rs` only as a compatibility harness while migration is in flight. |
| `059-UT-CORE-005` through `059-UT-CORE-007` | Extend `crates/z00z_core/tests/genesis/test_config.rs`, `test_validation.rs`, `test_cross_network_isolation.rs`, `test_genesis_vouchers.rs`, and `test_genesis_policies.rs`. |
| `059-UT-STOR-001` through `059-UT-STOR-005` | Extend `crates/z00z_storage/tests/test_settlement_leaf.rs`, `crates/z00z_storage/src/settlement/test_model.rs`, `crates/z00z_storage/tests/test_batch_proof_support.rs`, `crates/z00z_storage/tests/test_store_api.rs`, and recovery tests. |
| `059-UT-WAL-001` through `059-UT-WAL-005` | Extend `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs`, `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs`, wallet service, RPC, backup, migration, and quarantine tests. |
| `059-UT-RUN-001` through `059-UT-RUN-004` | Extend `crates/z00z_runtime/validators/tests/test_object_policy_verdicts.rs`, `crates/z00z_runtime/watchers/tests/test_object_alerts.rs`, plus aggregator and rollup publication tests. |
| `059-E2E-SIM-001` through `059-E2E-SIM-015` | Extend `crates/z00z_simulator/tests/test_scenario1_object_flows.rs`, related `test_scenario1_*` integration files, `scenario_1` stage tests, and stage artifact verification. |
| `059-PATH-*` | Cross-crate integration tests and simulator artifacts proving core, storage, wallet, runtime, rollup, and simulator path contracts without creating a parallel object layer. |
| `059-PROP-*` | Resolve to crate-local property/fuzz coverage for descriptor hash stability, leaf/proof envelopes, conservation/residual arithmetic, and wallet status transitions. |
| `059-REG-*` | Extend existing asset, rights, storage compatibility, wallet payload, simulator stage-order, and checkpoint finality tests without replacing current coverage. |

## Canonical Artifact Names Or Equivalent Live Anchors

Simulator and final closeout must resolve every test ID to deterministic
artifact names or to an equivalent live anchor set:

| Artifact | Required content |
|---|---|
| `phase059-fixtures.json` | Deterministic actors, object ids, descriptor hashes, roots, policies, and fee envelope references. |
| `phase059-positive-flows.json` | `059-E2E-SIM-001` through accepted positive paths with prior root, expected root, wallet projections, and verdicts. |
| `phase059-negative-flows.json` | All rejected paths with rejection reason, unchanged root, watcher alert, and proposed fix. |
| `phase059-proof-evidence.json` | Inclusion/deletion/nonexistence/batch proof evidence for Terminal, Right, and Voucher families. |
| `phase059-wallet-projections.json` | Cash, voucher, right, quarantine, backup/restore state before and after critical actions. |
| `phase059-critical-paths.json` | Mapping from `059-PATH-*` to stages, roots, verdicts, wallet projections, artifacts, and pass/fail evidence. |
| `059-EVIDENCE-LEDGER.md` | Links every test ID, command, implementation file, simulator artifact, TODO bullet/table group, D-ID, and corpus constraint. |

## Regression Rules

- Existing asset transfer tests must still pass and still represent clean cash.
- Existing right genesis tests must still pass and must gain zero-value and
  forbidden-field coverage.
- Old terminal/right storage rows and proofs must remain decodable and must not
  be reinterpreted as vouchers.
- Existing wallet asset payload version remains readable.
- Simulator stage order remains stable unless a plan explicitly changes the
  runner contract and tests the new order.
