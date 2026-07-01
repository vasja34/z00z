# 047-TODO

Self-contained backlog contract:

- this file is the canonical execution backlog for Phase 047;
- `047-wallet-redesign-spec.md` remains the normative design authority;
- every numbered task must re-read the exact spec line blocks embedded in its
  `047-0N-PLAN.md` task packet before implementation starts;
- no live code path may keep Snapshot or `claimed_assets` as a second wallet
  asset authority after the cutover lands;
- no task may ship placeholder behavior, simulator-only storage truths, or
  hardcoded runtime defaults where the spec now defines the behavior.

Execution rules:

- execute `047-01` through `047-08` in order unless a plan explicitly states a
  narrower dependency;
- keep the wallet authority inside encrypted `.wlt` objects plus explicit JSONL
  tx-history sidecar semantics while tx history remains outside `.wlt`;
- keep `recv_range(...)` plus `StealthOutputScanner` as the wallet-side
  ownership authority;
- keep secrets in the `secrets` table only;
- keep Stage 13, runtime wallet code, backup/restore, and YAML default cutover
  in the same patch series so no document or simulator string contradicts the
  implementation;
- do not touch `crates/z00z_crypto/tari/**`;
- when a task needs a commit, use `/z00z-git-versioning`.

## Decision Summary

1. `OwnedAssetPayload` objects are the live wallet-owned asset authority.
2. `WalletProfilePayload` replaces the live semantic role currently played by
   Snapshot for non-asset metadata.
3. `WalletPersistenceState.claimed_assets` must leave the live write path and
   remain only as a one-shot migration or compatibility import if strictly
   required.
4. JSONL tx history remains an explicit sidecar plane until a later refactor
   moves tx records fully into `.wlt`.
5. Asset reservations must live on asset records and tx state, not only be
   inferred from JSONL.
6. Scan hit persistence and scan cursor movement should commit atomically; if a
   temporary two-step fallback remains, replay must be idempotent and tested.
7. Runtime defaults must move to `wallet_config.yaml` plus explicit env
   overrides, not stay as hardcoded Rust literals.
8. Simulator Stage 13 and all migrated tests must stop naming Snapshot as the
   live claimed-asset authority.

## Dependency Chain

1. `047-01` Schema and payload groundwork
2. `047-02` Low-level object upsert and index API
3. `047-03` Wallet profile replacement and runtime config cutover
4. `047-04` Owned asset store authority
5. `047-05` Receive and scan integration
6. `047-06` Transaction build, reservation, cancel, reconcile, and asset views
7. `047-07` Backup, restore, export, and compatibility bridge removal
8. `047-08` Simulator, docs, existing-test migration, and final validation

Hard dependencies:

- `047-02` depends on `047-01`
- `047-03` depends on `047-01` and `047-02`
- `047-04` depends on `047-01` and `047-02`
- `047-05` depends on `047-03` and `047-04`
- `047-06` depends on `047-04` and `047-05`
- `047-07` depends on `047-03`, `047-04`, and `047-06`
- `047-08` depends on `047-01` through `047-07`

## Plan Roster

- `047-01-PLAN.md` — align Rust/YAML schema, object kinds, payload versions,
  index vocabulary, and debug decode support.
- `047-02-PLAN.md` — land production object-by-id writes, index builders, and
  indexed object reads.
- `047-03-PLAN.md` — replace live Snapshot metadata with `WalletProfilePayload`
  and cut runtime defaults over to YAML-backed config.
- `047-04-PLAN.md` — make `OwnedAssetPayload` plus `WalletAssetStore` the live
  wallet-owned asset authority.
- `047-05-PLAN.md` — rewire `recv_range(...)`, `recv_route(...)`, and scan
  cursor persistence to the new asset store.
- `047-06-PLAN.md` — move build/cancel/reconcile and asset-facing RPC views to
  owned-asset status authority.
- `047-07-PLAN.md` — export and restore profile, owned assets, scan state, and
  JSONL history with staged promotion and migration safety.
- `047-08-PLAN.md` — cut Stage 13 and docs over to the new storage truth, then
  migrate existing tests and close the validation matrix.

## Full Spec Coverage Index

The complete section-by-section crosswalk from
`047-wallet-redesign-spec.md` into this backlog and the eight plan files lives
in `047-SPEC-COVERAGE.md`. Treat that file as the explicit proof that every
major spec section, migration phase, validation gate, and acceptance block has
an owner inside the Phase 047 planning packet.

## Requirement Routing

| Requirement group | Planned owner |
| --- | --- |
| `REQ-012`, `REQ-013`, `REQ-014` | `047-01`, `047-02` |
| `REQ-003`, `REQ-004`, `REQ-005`, `REQ-014` | `047-02`, `047-04` |
| `REQ-001`, `REQ-002`, `REQ-015` | `047-03`, `047-04`, `047-07` |
| `REQ-016`, `REQ-017`, `REQ-018` | `047-03`, `047-07` |
| `REQ-010`, `REQ-011` | `047-05` |
| `REQ-006`, `REQ-007`, `REQ-008`, `REQ-009` | `047-06`, `047-07` |
| `REQ-019`, `REQ-020` | every plan, with final cutover closure in `047-08` |

## Acceptance Routing

| Acceptance criteria | Planned owner |
| --- | --- |
| `AC-001` | `047-03` |
| `AC-002`, `AC-003`, `AC-004` | `047-04`, `047-05` |
| `AC-005`, `AC-006`, `AC-007`, `AC-008`, `AC-009` | `047-06` |
| `AC-010`, `AC-011` | `047-07` |
| `AC-012` | `047-08` |
| `AC-013` | `047-03`, `047-07` |

## Mandatory Existing-Test Migration

The phase is incomplete until the following existing suites no longer encode
Snapshot or `claimed_assets` as the post-cutover live authority:

1. `crates/z00z_wallets/src/services/wallet_service_tests.rs`
2. `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_tests.rs`
3. `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs`
4. `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs`
5. `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/tests.rs`
6. `crates/z00z_wallets/src/db/redb_wallet_store/tests.rs`
7. `crates/z00z_simulator/src/scenario_1/stage_13_wallet_tx/tests.rs`
8. `crates/z00z_wallets/src/backup/backup_importer_impl/tests.rs`
9. `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs`
10. Scenario 1 contract/config/report fixtures under
    `crates/z00z_simulator/src/scenario_1/`

## Verification Contract

Every `<task type="auto">` in every `047-0N-PLAN.md` must carry this command
order:

1. Run the fail-fast gate first:

   ```bash
   ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh
   ```

2. If the fail-fast gate fails, stop, fix the issue, and rerun it before any
   broader validation.

3. Then run the broad Rust gate for this refactor:

   ```bash
   cargo test --release --features test-fast --features wallet_debug_dump
   ```

4. Then run `/.github/prompts/gsd-review-tasks-execution.prompt.md` through
   `/GSD-Review-Tasks-Execution` at least 3 times in YOLO mode.

5. Continue fixing issues and rerunning the review prompt until at least
   2 consecutive runs report no significant issues.

6. If a task needs a commit, use `/z00z-git-versioning` before creating it.

## Drift Bars

- do not add a simulator-only asset table;
- do not make remote JMT ownership detection authoritative;
- do not store secret material inside asset, profile, tx, or backup objects;
- do not infer pending reservations only from JSONL once owned-asset records
  exist;
- do not overload old asset index semantics when the new owned-asset query needs
  a dedicated index variant;
- do not keep the `WalletPersistenceState.claimed_assets` vector on the normal
  live write path after cutover;
- do not leave hardcoded live defaults in create/open/runtime/backup/recovery
  code paths once the YAML contract is expanded;
- do not ship a green suite that still presents both Snapshot and
  `OwnedAssetPayload` as simultaneous live authorities.
