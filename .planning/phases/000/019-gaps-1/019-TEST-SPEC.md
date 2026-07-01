---
phase: 019-gaps-1
artifact: test-spec
status: fallback-ready
source: plans-and-code-seams
updated: 2026-03-25
---

# Phase 019 Test Spec

## Purpose

📌 This document defines the Rust integration and end-to-end acceptance
coverage required for Phase 019.

📌 It is intended to be directly usable by another engineer or agent without
guessing scenario boundaries, state transitions, proof anchors, or rejection
criteria.

📌 Phase 019 coverage is Rust integration coverage, not browser automation.
The end-to-end proof for this phase must be established through canonical
wallet and simulator flows, targeted `cargo test` entry points, and explicit
assertions on replay protection, receive taxonomy, and backup semantics.

## Workflow Status

⚠️ Strict `gsd-add-tests` generation is blocked because
`.planning/phases/019-gaps-1/` does not yet contain any `019-*-SUMMARY.md`
artifact and does not yet contain a phase-local `*-VERIFICATION.md`.

📌 This fallback test spec therefore uses these inputs:

- `.planning/phases/019-gaps-1/019-CONTEXT.md`
- `.planning/phases/019-gaps-1/019-RESEARCH.md`
- `.planning/phases/019-gaps-1/019-VALIDATION.md`
- `.planning/phases/019-gaps-1/019-01-PLAN.md`
- `.planning/phases/019-gaps-1/019-02-PLAN.md`
- `.planning/phases/019-gaps-1/019-03-PLAN.md`
- `.planning/phases/019-gaps-1/todo.md`
- Existing test anchors in `crates/z00z_simulator/tests/`,
  `crates/z00z_wallets/tests/`, and the current inline module tests.

## Classification

### TDD And Integration Targets

- `crates/z00z_storage/src/assets/store.rs`
  because Phase 019 must prove one canonical asset-plus-nullifier transition
  and fail-closed behavior at the storage seam.
- `crates/z00z_simulator/src/claim_pkg_consumer.rs`
  because it owns the canonical publish path that calls `apply_ops(...)` and
  finalizes replay-protection state.
- `crates/z00z_simulator/src/claim_pkg_store.rs`
  because it owns reserve, reload, rollback, and finalize orchestration for
  claim nullifiers.
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs`
  because it must expose report-first runtime receive semantics without
  malformed-input downgrade.
- `crates/z00z_wallets/src/core/address/leaf_scan.rs`
  because canonical leaf report and helper semantics must remain explicit and
  non-interchangeable.
- `crates/z00z_wallets/src/services/wallet_service.rs`
  because it is the authoritative runtime receive and backup boundary.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`
  because direct public receive adapters must migrate in the same phase.
- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs`
  because tx-output verification must not rely on ambiguous partial-stealth
  behavior.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs`
  because the active backup format must converge on `WalletExportPack`.
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs`
  because restore must support active V2 semantics and legacy V1 readability.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup.rs`
  because public JSON-RPC contract wording must match the guaranteed restore
  set.
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs`
  because public backup create, list, and restore behavior must remain
  integration-testable.
- `crates/z00z_wallets/src/adapters/rpc/types/backup.rs`
  because response DTOs must stay aligned with explicit restore semantics.
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`
   because dispatcher registration is part of the real public JSON-RPC backup
   surface and must stay aligned with the active restore contract.
- `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs`
   because dispatcher-bound backup roundtrips are a public transport seam, not
   just an internal helper.

### E2E Browser Targets

- None.

📌 End-to-end behavior for this phase is storage, proof, and wallet-service
driven. It must be proven through Rust integration tests and realistic service
or RPC roundtrips, not browser automation.

### Skip Targets

- The planning markdown files themselves
  because they are specification inputs, not executable logic.
- General configuration and roadmap files
  because they should be verified through behavior, not by direct test
  generation.
- Pure logging and unrelated wallet persistence tests
  unless they are explicitly needed to prove backup contract text or RPC
  transport behavior introduced by Phase 019.

## Existing Test Structure

📌 The repository already uses focused Rust integration files under:

- `crates/z00z_simulator/tests/`
- `crates/z00z_wallets/tests/`

📌 The project convention is one focused integration file per seam, named
`test_*.rs`, executed with targeted `cargo test -p <crate> --test <name>`
commands.

📌 Phase 019 also has important inline test anchors inside source files:

- `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/backup.rs`
- `crates/z00z_wallets/src/services/wallet_service.rs`

📌 Inline tests should remain lightweight regression anchors. The phase-level
contract should be proven primarily in integration tests under `tests/`.

## Canonical Test Commands

📌 The minimum execution anchors for Phase 019 are:

- `cargo test -p z00z_simulator --test test_stage3_nullifier_store -- --nocapture`
- `cargo test -p z00z_simulator --test test_claim_tx_pipeline -- --nocapture`
- `cargo test -p z00z_wallets --test test_e2e_runtime_parity -- --nocapture`
- `cargo test -p z00z_wallets --test test_stealth_scanner_prefilter -- --nocapture`
- `cargo test -p z00z_wallets test_export_import_wallet_payload -- --nocapture`
- `cargo test -p z00z_wallets backup_impl::tests::test_backup_create_list_restore -- --nocapture`
- `cargo test -p z00z_wallets test_runtime_restore_backup_response -- --nocapture`
- `cargo test -p z00z_wallets --test test_rpc_dispatcher_roundtrip -- --nocapture`
- `cargo test -p z00z_wallets tx_verify_skips_partial_stealth -- --nocapture`
- `cargo test -p z00z_wallets && cargo test -p z00z_simulator`

📌 Release-lane parity should reuse the same targets with
`--release --features test-fast --features wallet_debug_dump` where supported.

## Required End-To-End Behaviors

📌 The following end-to-end behaviors are mandatory for Phase 019 and must be
proven explicitly rather than inferred from local unit assertions.

| Behavior | Requirement | Primary Path | Pass Signal | Fail Signal |
| ---- | ---- | ---- | ---- | ---- |
| Claim replay protection survives the full publish lane | PH19-NULL | `write_claim_bundle(...) -> publish_claims_store(...) -> AssetStore::apply_ops(...) -> finalize_nulls(...)` | first publish succeeds, outputs exist, nullifier becomes `Spent`, replay publish is rejected | duplicate publish succeeds, nullifier remains ambiguous, or state partially advances |
| JMT asset absence is not treated as replay state | PH19-NULL | claim bundle replay and missing-reservation flows | explicit missing reservation or replay rejection | any path accepts claim reuse because no current leaf exists |
| Runtime receive path is report-first and explicit | PH19-SCAN | `wallet_service.recv_one(...)` and migrated direct adapters | owned, foreign, invalid-input, and invalid-proof cases stay distinct | malformed or proof-invalid input silently degrades to `NotMine`, `MaybeMine`, or empty pack |
| Direct tx verification cannot misclassify partial stealth payloads | PH19-SCAN | `TxRpcImpl::build_owned_out(...)` via `tx_verify_skips_partial_stealth` | partial stealth output is skipped safely and yields no owned output | partial stealth payload becomes owned or leaks ambiguous receive semantics |
| Active backup format restores the guaranteed set | PH19-BACKUP | `create_backup(...)` or `export_wallet_payload(...)` -> restore/import into a fresh service instance | restored wallet identity and snapshot state are coherent, active-format guarantee is explicit | restore claims success without reloading guaranteed state |
| Legacy V1 remains readable without overclaiming semantics | PH19-BACKUP | `restore_backup(...)` V1 fallback path | legacy payload is accepted under explicit legacy semantics | V1 input is unreadable or falsely advertised as full-state restore |

## Critical Integration Paths

📌 Another engineer should treat these as the canonical execution paths for
Phase 019 coverage.

1. Nullifier path:
   `derive_nullifier(...) -> write_claim_bundle(...) -> publish_claims_store(...) -> AssetStore::apply_ops(...) -> finalize_nulls(...)`
2. Receive path:
   `receiver_scan_report(...) <-> wallet_service.recv_one(...) <-> direct adapter migration in asset_impl.rs and tx_impl.rs`
3. Backup path:
   `create_backup(...) or export_wallet_payload(...) -> encrypted payload/container -> restore_backup(...) or import_wallet_payload(...)`
4. Dispatcher-bound backup path:
   `wallet_dispatcher_wiring.rs -> wallet.backup.create_backup / restore_backup -> BackupRpcImpl -> WalletService`

📌 Any new test added for Phase 019 should name which one of these four paths
it is proving. If the test does not anchor to one of them, it is probably a
secondary regression, not a phase-closing E2E proof.

## Scenario Oracle Rules

📌 Each scenario in this file must have a machine-checkable pass/fail oracle.
The following rules are mandatory.

1. A scenario passes only when it proves both behavior and invariant.
2. A scenario fails if it only observes logs or comments without asserting
   state, status, or artifact effects.
3. A rejection scenario passes only when the rejected path is explicit and no
   partial-success artifact remains.
4. A migration scenario passes only when old ambiguous semantics are gone or
   explicitly labeled compatibility-only.
5. A backup scenario passes only when the restored result matches the promised
   restore breadth for that format version.

## Existing Test Anchors To Reuse

📌 Reuse and extend these existing anchors instead of duplicating them:

- `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`
- `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`
- `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs`
- `crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs`
- `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
- `crates/z00z_wallets/tests/test_rpc_types_serialization.rs`
- `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs`

📌 Keep these inline tests as fast regression hooks:

- `tx_verify_skips_partial_stealth`
- `test_backup_create_list_restore`
- `test_runtime_restore_backup_response`
- `test_export_import_wallet_payload`

## Test Files To Add Or Extend

### 1. Extend `crates/z00z_simulator/tests/test_stage3_nullifier_store.rs`

📌 This file should continue proving the reservation store behavior before and
around canonical publish.

Tests to add or strengthen:

1. `restart_replay_rejects_same_claim_identity`
   Demonstrates: replay protection survives restart or rebinding of the
   nullifier files.
   Success conditions:
   - the same claim bundle is accepted once and rejected on replay;
   - the rejection explicitly mentions nullifier replay;
   - the row remains in `Reserved` or `Spent` state as appropriate and never
     disappears silently.

2. `corrupt_nullifier_rows_fail_closed`
   Demonstrates: malformed persisted nullifier state cannot be interpreted as
   available claim capacity.
   Success conditions:
   - loading corrupt row JSON fails closed;
   - the error is typed or textually anchored to row-load failure;
   - no fallback path returns `NotFound`, `Available`, or similar success.

3. `write_fault_rolls_back_reservation`
   Demonstrates: serialization, write, or verify faults do not leak reserved
   replay state.
   Success conditions:
   - forced fault returns an error;
   - the nullifier row is absent after rollback;
   - the bundle file is absent when the write is not valid.

### 2. Extend `crates/z00z_simulator/tests/test_claim_tx_pipeline.rs`

📌 This file must become the canonical publish-path regression suite for
Phase 019 nullifier ownership.

Tests to add or strengthen:

1. `publish_claim_store_marks_nullifier_spent_only_after_apply`
   Demonstrates: nullifier state and asset insertion advance together on the
   canonical publish path.
   Success conditions:
   - `publish_claims_store(...)` succeeds;
   - inserted leaves exist in `AssetStore`;
   - the corresponding nullifier entries are `Spent`;
   - `tx_digest_hex` remains bound to the stored claim identity.

2. `publish_claim_store_rejects_replay_after_success`
   Demonstrates: replay is rejected on the same direct publish path that talks
   to `AssetStore::apply_ops(...)`.
   Success conditions:
   - first publish succeeds;
   - second publish fails;
   - the error explicitly mentions nullifier replay rejection.

3. `publish_claim_store_requires_prior_reservation`
   Demonstrates: direct publish cannot bypass reservation semantics.
   Success conditions:
   - a bundle written without reservation fails;
   - the error explicitly mentions missing reservation.

4. `duplicate_asset_id_rejects_before_partial_publish`
   Demonstrates: canonical publish does not leave mixed state when output
   identity collides.
   Success conditions:
   - duplicate asset ids are rejected before partial commit;
   - no output leaf is committed;
   - no corresponding nullifier becomes `Spent`.

### 3. Extend `crates/z00z_wallets/tests/test_e2e_runtime_parity.rs`

📌 This file should remain the authoritative parity gate for canonical leaf
scan and runtime receive behavior.

Tests to add or strengthen:

1. `runtime_report_matches_canonical_report_for_owned_asset`
   Demonstrates: canonical leaf report and runtime receive report remain in
   parity for a valid owned asset.
   Success conditions:
   - canonical report equals runtime report;
   - status is `Detected`;
   - amount, asset id, serial id, blinding, and stealth outputs remain
     consistent.

2. `runtime_report_matches_canonical_not_mine_for_foreign_asset`
   Demonstrates: foreign ownership remains explicit and does not look like an
   invalid proof or malformed input.
   Success conditions:
   - canonical report equals runtime report;
   - status is `NotMine`;
   - reject reason is `NotMine`.

3. `runtime_invalid_input_is_not_silently_downgraded`
   Demonstrates: malformed runtime shape is rejected as `InvalidInput` on the
   authoritative public path.
   Success conditions:
   - malformed runtime asset causes receive rejection;
   - status or reject surface identifies invalid input;
   - the same case must not appear as `NotMine`, `MaybeMine`, or empty pack.

4. `runtime_invalid_proof_remains_explicit`
   Demonstrates: proof failure remains distinct from malformed input and from
   ownership absence.
   Success conditions:
   - canonical report and runtime report both identify proof failure;
   - the result is not downgraded to `NotMine`.

### 4. Extend `crates/z00z_wallets/tests/test_stealth_scanner_prefilter.rs`

📌 This file should prove the negative receive paths and malformed-input
boundaries at scanner level.

Tests to add or strengthen:

1. `malformed_r_pub_is_invalid_input_on_report_surface`
   Demonstrates: malformed compressed point bytes are not a foreign-wallet
   classification when using the authoritative report surface.
   Success conditions:
   - malformed `r_pub` returns explicit invalid-input rejection;
   - the same path does not silently become `NotMine` on the authoritative
     contract.

2. `missing_required_fields_are_invalid_input_not_not_mine`
   Demonstrates: missing `r_pub`, `owner_tag`, or `enc_pack` is treated as a
   shape error on the authoritative runtime path.
   Success conditions:
   - each missing-field case yields `InvalidInput`;
   - helper-level raw `ScanResult` behavior, if retained, is explicitly marked
     compatibility-only and not reused as the acceptance oracle.

3. `tampered_tag16_or_owner_binding_does_not_become_detected`
   Demonstrates: tag and ownership tamper cases remain cryptographically safe.
   Success conditions:
   - the result is never `Detected`;
   - a `MaybeMine` path is allowed only where the plan explicitly keeps it as a
     compatibility helper;
   - the authoritative report surface still preserves explicit rejection.

### 5. Add `crates/z00z_wallets/tests/test_phase19_backup_contract.rs`

📌 Add one dedicated integration file for backup convergence instead of relying
only on inline module tests.

Tests to implement:

1. `backup_v2_roundtrip_restores_guaranteed_state`
   Demonstrates: the active backup contract restores the guaranteed set for the
   active format.
   Success conditions:
   - exported backup or payload can be restored into a fresh service instance;
   - restored wallet id or root identity is coherent;
   - wallet snapshot settings or equivalent persisted state survive roundtrip;
   - the test explicitly notes that the guarantee is root secret material,
     snapshot state, and versioned restore metadata.

2. `backup_v1_legacy_input_remains_readable`
   Demonstrates: migration does not strand existing backup files.
   Success conditions:
   - legacy V1 payload can still be parsed or restored through the supported
     compatibility path;
   - the response clearly reflects legacy semantics;
   - the implementation does not falsely report full-state restoration for a
     metadata-only payload.

3. `backup_wrong_password_fails_without_partial_restore`
   Demonstrates: encrypted backup restore remains fail-closed.
   Success conditions:
   - wrong password returns explicit error;
   - no new wallet is materialized;
   - no partial restore artifacts appear on disk.

4. `backup_rpc_contract_does_not_overpromise_journals_or_history`
   Demonstrates: public contract text and response semantics match the real
   restore breadth.
   Success conditions:
   - response DTOs and RPC contract surfaces do not imply restoration of caches,
     journals, or transaction history unless actually implemented;
   - the success path still proves the guaranteed restore set.

### 6. Extend `crates/z00z_wallets/tests/test_rpc_types_serialization.rs`

📌 This file should prove that public response surfaces remain serialization
stable while semantics tighten.

Tests to add or strengthen:

1. `backup_restore_response_roundtrip_preserves_explicit_status_fields`
   Demonstrates: status and wallet identity remain serialization-stable after
   backup contract updates.

2. `backup_create_response_roundtrip_preserves_active_format_signal`
   Demonstrates: response fields such as `encrypted` and `backup_path` remain
   consistent and machine-readable.

3. `legacy_readability_does_not_change_response_shape`
   Demonstrates: V1 fallback behavior does not force ambiguous public DTO shape.

### 7. Extend `crates/z00z_wallets/tests/test_rpc_dispatcher_roundtrip.rs`

📌 This file should prove that the public dispatcher wiring preserves the same
backup contract semantics as the direct RPC implementation tests.

Tests to add or strengthen:

1. `dispatcher_backup_roundtrip_matches_direct_contract`
   Demonstrates: dispatcher-registered backup create and restore calls preserve
   the same success and identity semantics as direct backup RPC invocation.
   Success conditions:
   - a wallet can be created and unlocked through the same dispatcher fixture;
   - `wallet.backup.create_backup` returns a real backup path and explicit
     success;
   - `wallet.backup.restore_backup` returns the expected wallet id;
   - the response does not imply unsupported restore breadth.

2. `dispatcher_backup_wrong_password_fails_explicitly`
   Demonstrates: transport-level JSON-RPC does not blur authentication failure
   into generic success or an unrelated error shape.
   Success conditions:
   - wrong password returns an explicit RPC error;
   - no follow-on success artifact is produced.

## Realistic Examples That Must Be Demonstrated

### Example A: Claim Replay Lifecycle

📌 Scenario: a claim bundle is reserved, written, verified, published into the
canonical store, then replayed.

What it demonstrates:

- where nullifiers come from in the claim flow;
- why JMT leaf absence is not the same as replay-protection state;
- that `Reserved` becomes `Spent` only after canonical publish succeeds.

Required assertions:

- nullifier derivation is deterministic for the same claim identity;
- first publish succeeds and writes canonical outputs;
- replay attempt fails with explicit nullifier-replay reason;
- no second publish mutates store state.

### Example B: Runtime Receive Taxonomy Walk

📌 Scenario: one valid owned asset, one foreign asset, one malformed runtime
asset, and one proof-invalid asset pass through canonical and runtime paths.

What it demonstrates:

- which path is authoritative;
- how `Detected`, `NotMine`, `InvalidInput`, and invalid-proof cases differ;
- which adapters must remain in parity.

Required assertions:

- valid owned runtime report equals canonical report;
- foreign runtime report equals canonical report;
- malformed runtime input is `InvalidInput`;
- proof-invalid runtime input is not silently downgraded to `NotMine`.

### Example C: Backup Convergence Walk

📌 Scenario: create active-format backup, restore it into a new wallet service,
then attempt restore from a legacy V1 payload.

What it demonstrates:

- what the active public backup contract restores;
- how legacy compatibility behaves;
- which unsupported extras remain outside the guarantee.

Required assertions:

- active-format restore brings back guaranteed state;
- wrong password fails cleanly;
- V1 legacy input remains readable;
- RPC or service response does not claim restoration of journals, caches, or
  history unless implemented.

## Cryptographic And Soundness Invariants

📌 The following invariants are mandatory and must be asserted explicitly where
relevant.

1. Nullifier determinism:
   the same `(claim_id, recipient_owner, chain_id)` produces the same
   nullifier.
2. Replay-protection separation:
   JMT leaf absence must not be treated as equivalent to nullifier availability.
3. Atomicity:
   canonical asset publication and replay-protection finalization advance
   together or not at all.
4. Proof-before-ownership discipline:
   invalid-proof and malformed-input cases must not collapse into ordinary
   ownership absence on the authoritative path.
5. Backup confidentiality:
   wrong-password restore must fail without partial plaintext recovery or silent
   wallet creation.
6. Public contract honesty:
   the active restore promise must match implemented state restoration breadth.

## Negative Scenarios That Must Exist

📌 These failures are mandatory because they prove soundness instead of only the
happy path.

1. Corrupt nullifier rows fail closed.
2. Publish without reservation is rejected.
3. Replay after successful publish is rejected.
4. Duplicate asset id across claim outputs is rejected before partial publish.
5. Malformed runtime receive input is rejected as `InvalidInput`.
6. Invalid proof remains explicit and is not downgraded to `NotMine`.
7. Partial stealth payload in tx verification is skipped safely and does not
   produce owned output.
8. Wrong-password backup restore fails without partial restore.
9. Legacy V1 backup input remains readable but does not overclaim full-state
   restoration.
10. Phase-level close gate fails if nullifier, receive, or backup suites are
    red.

## Measurable Completion Criteria

📌 Phase 019 end-to-end coverage is complete only when all of the following are
true.

1. Nullifier lane proves deterministic derive, reserve, rollback, reload,
   publish, replay rejection, and spent-state persistence.
2. Receive lane proves owned parity, foreign parity, malformed-input rejection,
   explicit invalid-proof handling, and direct tx verification safety.
3. Backup lane proves active-format guaranteed restore, legacy V1 readability,
   wrong-password failure, honest public contract semantics, and dispatcher-
   bound JSON-RPC roundtrip parity.
4. Every scenario names its success conditions and failure conditions in code,
   not only in comments.
5. The quick-run gate and the full suite gate can be executed without guessing
   which files or tests represent the phase contract.

## Implementation Notes For The Next Engineer

📌 Prefer extending existing focused integration files before adding new ones,
except for the dedicated backup contract file where a phase-specific acceptance
surface is clearer than further inflating inline tests.

📌 Keep test names descriptive and phase-local. Each test should prove one
contract boundary, not a vague cluster of behaviors.

📌 When a helper path remains intentionally compatibility-only, state that in
the test name or assertion text so future reviewers do not mistake helper
behavior for the authoritative public contract.
