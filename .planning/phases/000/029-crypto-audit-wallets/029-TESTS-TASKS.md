---
phase: 029-crypto-audit-wallets
artifact: tests-tasks
status: executed-and-verified
source: 029-TEST-SPEC.md
updated: 2026-03-30
---

# Phase 029 Tests Tasks

## Purpose

📌 This document translates `029-TEST-SPEC.md` into one concrete
implementation order for Phase 029 test work.

📌 It is intended to be directly usable by another engineer or agent without
guessing which scenario comes first, which files should be extended versus
created, which assertions are phase-closing, or which validation commands must
run after each step.

📌 This artifact is a planning breakdown for test implementation only. It does
not approve implementation shortcuts, alternate test seams, or speculative API
shapes beyond what is already supported by the current tree and the Phase 029
plans.

📌 The execution order captured here has now been realized by the summary-backed
Phase 029 plans and verified through the green command bundle recorded in
`029-VERIFICATION.md`.

## Scope Inputs

📌 This breakdown is derived from:

- `.planning/phases/000/029-crypto-audit-wallets/029-TEST-SPEC.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-CONTEXT.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-FUSION.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-VERIFICATION.md`
- Phase-local TODO: not present for Phase 029; deferred checks are therefore
   taken from verification notes plus the numbered plans below
- `.planning/phases/000/029-crypto-audit-wallets/029-01-PLAN.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-02-PLAN.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-03-PLAN.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-04-PLAN.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-05-PLAN.md`
- `.planning/phases/000/029-crypto-audit-wallets/029-06-PLAN.md`

📌 `029-RESEARCH.md` is still absent. Where a scenario could otherwise depend
on research-only detail, this document keeps the task anchored to current-tree
files, current tests, and explicitly proposed new test files only.

## Execution Strategy

📌 Phase 029 test work should be implemented in dependency order, not in simple
file-name order.

📌 The recommended sequence is:

1. Establish canonical live-path tests before persistence migration tests.
2. Establish persistence and backup KDF tests before runtime panic and seed-salt
   rollout tests.
3. Establish loud invariant and secret-wrapper tests before final ambiguity
   closure tests.
4. Close digest, validation, metadata, and mnemonic-boundary tests only after
   upstream contracts are frozen by the earlier scenarios.

📌 The implementation order below is therefore the canonical order for Phase
029 unless a later Phase 029 summary records an explicit deviation.

## Task Waves

### Wave T0: Harness Confirmation And Reuse Lock-In

📌 Objective: confirm which existing files are being extended and which proposed
files must be created fresh.

📌 Files to inspect before test writing:

- `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
- `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
- `crates/z00z_wallets/tests/test_key_manager.rs`
- `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs`
- `crates/z00z_wallets/tests/test_tx_tamper.rs`
- `crates/z00z_wallets/tests/test_wlt_validator.rs`

📌 Deliverables:

- one explicit extend-versus-create decision per Phase 029 scenario;
- one shared fixture note for wallet create, unlock, backup, and seed-reveal
  setup helpers;
- one note identifying any existing helper that can be reused without creating
  a parallel fixture system.

📌 Completion gate:

- no scenario still has an ambiguous test home;
- no proposed test file duplicates an existing seam without a stated reason.

### Wave T1: Scenario 029-E2E-01 Canonical Live View-Key Lock-Step

📌 Priority: highest test priority.

📌 Why first: sender, scanner, and spend alignment is the main protocol-proof
surface for Phase 029 and provides the live-path contract used by later
receiver-secret validation work.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_e2e_send_scan.rs`
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`

📌 Create this file:

- `crates/z00z_wallets/tests/test_view_key_contract.rs`

📌 Required implementation tasks:

1. Add one canonical live-path fixture that builds a sender output for one
   owned receiver and one foreign receiver.
2. Add one assertion path that proves owned detection stays `Mine` and foreign
   detection stays `NotMine`.
3. Add one spend-verification assertion bound to the same live derivation path.
4. Add one negative test proving that a versioned helper substituted into a
   live path fails explicitly.
5. Add one source-guard assertion or equivalent proof that hot-path files do
   not reference `derive_view_key_versioned(...)` directly.

📌 Required success conditions:

- same `ReceiverSecret` yields one live derived key across send, scan, and
  spend assertions;
- foreign receiver remains explicitly rejected;
- versioned helper remains available only through an explicit non-default path.

📌 Required command gate:

```bash
cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture
cargo test -p z00z_wallets --release --test test_e2e_send_scan -- --nocapture
cargo test -p z00z_wallets --release --test test_rpc_key_derive_e2e -- --nocapture --test-threads=1
```

### Wave T2: Scenario 029-E2E-02 Legacy `.wlt` Rewrite And Scenario 029-E2E-03 Backup KDF Contract

📌 Priority: second.

📌 Why here: wallet and backup persistence semantics must be frozen before the
seed-salt and metadata-policy tests can become authoritative.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_wallet_persistence_backup_service.rs`
- `crates/z00z_wallets/tests/test_redb_wlt_open.rs`
- `crates/z00z_wallets/tests/test_wlt_validator.rs`

📌 Create these files:

- `crates/z00z_wallets/tests/test_wallet_kdf_migration.rs`
- `crates/z00z_wallets/tests/test_backup_kdf_contract.rs`
- `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`
- `crates/z00z_wallets/tests/test_backup_restore_identity.rs`

📌 Required implementation tasks for `.wlt` migration:

1. Create or reuse one explicit legacy `.wlt` fixture.
2. Prove compatibility open succeeds for that fixture.
3. Prove accepted unlock or open rewrites persisted state to V2.
4. Reopen the rewritten `.wlt` and assert the new representation is now the
   canonical source of truth.
5. Add one negative test for unknown wallet KDF version rejection before costly
   derivation.

📌 Required implementation tasks for backup KDF governance:

1. Export one backup under the new contract.
2. Assert that the header contains explicit KDF description fields.
3. Import that backup into a fresh service instance and assert restored state.
4. Add one negative test for unknown backup KDF version rejection before decrypt.
5. Add one policy test for chosen metadata visibility: minimal public header or
   encrypted private metadata.
6. Add one restore-identity test proving encrypted backup import preserves the
   expected wallet identity contract after recovery.

📌 Required success conditions:

- accepted legacy wallet path rewrites once and reopens once under V2;
- unknown wallet and backup versions fail early;
- backup KDF contract is self-describing and observable in tests;
- backup restore-identity behavior is proven explicitly under the recovered
   wallet path.

📌 Required command gate:

```bash
cargo test -p z00z_wallets --release --test test_wallet_kdf_migration -- --nocapture
cargo test -p z00z_wallets --release --test test_backup_kdf_contract -- --nocapture
cargo test -p z00z_wallets --release --test test_backup_metadata_policy -- --nocapture
cargo test -p z00z_wallets --release --test test_backup_restore_identity -- --nocapture
cargo test -p z00z_wallets --release --test test_wallet_persistence_backup_service -- --nocapture
```

### Wave T3: Scenario 029-E2E-04 Runtime Wallet Failures And Scenario 029-E2E-05 Seed-Salt Rollout

📌 Priority: third.

📌 Why here: runtime error behavior and random seed-salt rollout depend on the
canonical persistence contract already established in Wave T2.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs`

📌 Create these files:

- `crates/z00z_wallets/tests/test_wallet_service_errors.rs`
- `crates/z00z_wallets/tests/test_seed_salt_policy.rs`

📌 Required implementation tasks for runtime error closure:

1. Create one wallet-service failure fixture for serialization, backup, decrypt,
   or load failure.
2. Assert typed `WalletError` propagation instead of panic.
3. Assert no partial artifact remains after failure.
4. Add one classification-facing test note if `chain_service` remains downgraded
   rather than code-fixed.

📌 Required implementation tasks for seed-salt rollout:

1. Create or rewrite one wallet through the new-write path.
2. Assert persisted metadata carries one random wallet-owned 16-byte seed salt.
3. Exercise one reveal or export path and assert it reuses the persisted salt.
4. Add one negative test proving that the new-write path no longer depends on
   `compute_seed_salt(...)` as its governing boundary.
5. Keep deterministic salt reachable only through explicit legacy fallback tests.

📌 Required success conditions:

- runtime service failures return typed errors with no partial success state;
- two new-write paths do not collapse to deterministic `wallet_id` salt logic;
- reveal or export behavior proves persisted salt reuse.

📌 Required command gate:

```bash
cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture
cargo test -p z00z_wallets --release --test test_seed_salt_policy -- --nocapture
cargo test -p z00z_wallets --release --test test_show_seed_phrase_plaintext -- --nocapture
```

### Wave T4: Scenario 029-E2E-06 Loud Invariants And Secret-Wrapper Boundaries

📌 Priority: fourth.

📌 Why here: this wave depends on the canonical live-path and persistence
contracts from Waves T1 through T3, but it must land before final DTO and
mnemonic-boundary closure.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_key_manager.rs`

📌 Create these files:

- `crates/z00z_wallets/tests/test_receiver_secret_validation.rs`
- `crates/z00z_wallets/tests/test_file_key_store.rs`

📌 Required implementation tasks:

1. Add one impossible BIP-44 state case that must fail with typed corruption or
   gap-limit error.
2. Add one receiver-secret creation failure for unusable bytes.
3. Add one load or decrypt boundary that rejects unusable receiver secrets.
4. Add one file-key-store persistence roundtrip that consumes `SafePassword` or
   an equivalent zeroizing wrapper.
5. Add one negative assertion that cloneable plaintext password bytes are no
   longer the persistence contract.

📌 Required success conditions:

- impossible counter order fails loudly;
- invalid receiver secrets do not escape constructor or load boundaries;
- password-bearing persistence uses a zeroizing wrapper and no convenience
  derives imply plaintext copying.

📌 Required command gate:

```bash
cargo test -p z00z_wallets --release --test test_key_manager -- --nocapture
cargo test -p z00z_wallets --release --test test_receiver_secret_validation -- --nocapture
cargo test -p z00z_wallets --release --test test_file_key_store -- --nocapture
```

### Wave T5: Scenario 029-E2E-07 Digest Framing And Scenario 029-E2E-08 Validation Or Boundary Closure

📌 Priority: final.

📌 Why last: these scenarios close the remaining ambiguity surfaces and depend
on upstream contracts already being stable.

📌 Extend these files:

- `crates/z00z_wallets/tests/test_tx_tamper.rs`
- `crates/z00z_wallets/tests/test_wlt_validator.rs`
- `crates/z00z_wallets/tests/test_show_seed_phrase_plaintext.rs`

📌 Create these files:

- `crates/z00z_wallets/tests/test_tx_digest_framing.rs`
- `crates/z00z_wallets/tests/test_runtime_validation_result.rs`
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`

📌 Required implementation tasks for digest framing:

1. Add one ambiguous tuple pair case such as `("AB", "C")` versus
   `("A", "BC")`.
2. Assert the canonical digest builder produces different digests.
3. Assert the implementation uses existing `frame_*` helpers.
4. Reuse tamper tests to confirm mutated witness or proof inputs still reject.

📌 Required implementation tasks for validation and boundary closure:

1. Add one accepted validation case and one warning or rejection case.
2. Assert core and RPC DTO semantics remain aligned.
3. Assert backup metadata policy matches the chosen explicit contract.
4. Assert `WalletExportPack` is either an explicit outer plaintext seam or is
   replaced by a stronger wallet-owned boundary already supported by the tree.
5. Assert ciphertext and metadata do not leak plaintext seed phrase
   unexpectedly.
6. Assert privacy-sensitive debug or verbose feature gates are documented
   consistently in code and README where the phase requires it.

📌 Required success conditions:

- digest framing is injective for covered tuple shapes;
- tampered proof material still fails at the typed wallet boundary;
- validation DTO, metadata policy, and mnemonic boundary are explicit and
  machine-asserted.

📌 Required command gate:

```bash
cargo test -p z00z_wallets --release --test test_tx_digest_framing -- --nocapture
cargo test -p z00z_wallets --release --test test_runtime_validation_result -- --nocapture
cargo test -p z00z_wallets --release --test test_wallet_export_pack_boundary -- --nocapture
cargo test -p z00z_wallets --release --test test_tx_tamper -- --nocapture
```

## Scenario-To-File Matrix

📌 Use this matrix as the non-ambiguous ownership map.

| Scenario | Primary Existing Files | Primary Proposed Files | Must Prove |
| --- | --- | --- | --- |
| 029-E2E-01 | `test_e2e_send_scan.rs`, `test_rpc_key_derive_e2e.rs` | `test_view_key_contract.rs` | one live view-key path across send, scan, and spend |
| 029-E2E-02 | `test_wallet_persistence_backup_service.rs`, `test_redb_wlt_open.rs` | `test_wallet_kdf_migration.rs` | persisted V1-to-V2 rewrite plus reopen |
| 029-E2E-03 | `test_wallet_persistence_backup_service.rs`, `test_wlt_validator.rs` | `test_backup_kdf_contract.rs`, `test_backup_metadata_policy.rs`, `test_backup_restore_identity.rs` | self-describing backup KDF plus explicit metadata policy and restore-identity coverage |
| 029-E2E-04 | none | `test_wallet_service_errors.rs` | typed runtime failures with no partial artifacts |
| 029-E2E-05 | `test_show_seed_phrase_plaintext.rs` | `test_seed_salt_policy.rs` | random persisted seed salt plus bounded legacy fallback |
| 029-E2E-06 | `test_key_manager.rs` | `test_receiver_secret_validation.rs`, `test_file_key_store.rs` | loud invariant failure and zeroizing persistence boundary |
| 029-E2E-07 | `test_tx_tamper.rs` | `test_tx_digest_framing.rs` | framed digest injectivity plus tamper rejection |
| 029-E2E-08 | `test_wlt_validator.rs`, `test_show_seed_phrase_plaintext.rs` | `test_runtime_validation_result.rs`, `test_wallet_export_pack_boundary.rs` | explicit DTO, metadata, and mnemonic boundary |

## Cross-Wave Rules

📌 These rules apply to every wave above.

1. Use one focused test file per new seam instead of a mixed omnibus suite.
1. Reuse existing wallet helpers and fixtures where they already prove the
   right runtime contract.
1. Keep negative scenarios explicit and named after the rejected behavior, not
   after the helper function used to trigger it.
1. When a scenario needs source guards, keep them minimal and bind them only to
   the hot-path files explicitly named in the Phase 029 plans.
1. Do not introduce a second public contract in tests that the code does not
   already support.
1. If `CipherSeedContainer` governance still cannot be closed from the current
   tree, keep the final tests aligned with the explicit open decision rather
   than inventing a new container format.

## Final Closure Gate

📌 Phase 029 test implementation is ready for execution only when all of the
following are true:

- each of the eight scenarios has one explicit home in either an existing or a
  proposed test file;
- each scenario has one command gate and one machine-checkable pass oracle;
- each scenario includes at least one rejection or failure path where Phase 029
  requires fail-closed behavior;
- later-wave tests do not depend on unresolved upstream ambiguity from earlier
  waves.

📌 If a later implementer cannot complete a scenario without inventing an
unsupported API, file, or data shape, that block must be recorded as an
explicit open decision in the relevant Phase 029 summary instead of being
silently resolved inside test code.
