---
phase: 029-crypto-audit-wallets
artifact: reconciliation
status: current-tree-reconciled
source_inputs:
  - 029-FUSION.md
  - 029-CONTEXT.md
generated_at: 2026-03-30
---

<!-- markdownlint-disable MD041 -->

## Scope

📌 This artifact now records the reconciled live execution result for Phase 029
against the current `z00z_wallets` tree after the remediation waves landed.

📌 Every fused finding cluster is classified as one of:

- `still_open`
- `partially_closed`
- `validation_only`
- `stale_in_current_tree`
- `explicit_open_decision`
- `out_of_phase_followup`

📌 `029-RESEARCH.md` does not exist in the phase directory. Every statement below
is therefore grounded in current-tree code evidence, `029-FUSION.md`, and
`029-CONTEXT.md` only.

📌 All file and line references below are a current-tree snapshot taken on
2026-03-30. Any later follow-up must still re-verify exact line ranges if the
wallet sources move again.

## Evidence Quality

📌 Confidence labels used in this document:

| Label | Meaning | Examples in this phase |
| --- | --- | --- |
| `current_tree_evidence_only` | Supported directly by current Rust files or tests | `compute_seed_salt(...)` runtime use, `WalletExportPack { seed_phrase: String }`, `RuntimeValidationResult { valid, error }` |
| `source_ambiguity` | The fused report raised the issue, but the current tree no longer shows the same shape or the evidence is incomplete | historical `chain_service` `await.unwrap()` crash characterization, `wallet_id` public-derivability risk, `CipherSeedContainer` wrapper governance |

## Source Ambiguity Summary

📌 The following findings remain intentionally unresolved at Gate 0 and must not
be upgraded into fact without new current-tree evidence:

| Claim | Why ambiguous now | Resolution rule |
| --- | --- | --- |
| Historical `chain_service` runtime crash characterization | The current runtime methods return `Result`; visible `unwrap()` usage is confined to tests | Keep downgraded unless wave `029-04` finds an operator-reachable panic path |
| `wallet_id` public-derivability materially worsens offline attack cost | Deterministic salt use is proven, but the public derivability assumption is not | Apply the deterministic-salt mitigation anyway; keep the threat-model claim as an open decision |
| Wallet-owned nonce truncation affects an external protocol or circuit contract | Required Phase 029 files show 24-byte XChaCha20 nonce handling, but Gate 0 did not prove whether any other wallet-owned container truncates a wider nonce into a protocol-bound shape | Keep as a manual-review open decision unless a later wave touches a conflicting cipher container |
| `CipherSeedContainer` needs a new wrapper type | Gate 0 evidence shows the current seed seam, but does not prove a new wrapper is required | Only introduce a wrapper if wave `029-06` can justify it without format drift |

📌 Current-tree conclusion: the strongest remaining work is no longer phase
closure. The executed waves already closed the plan-owned runtime seams, and
the live Phase 029 target is now narrower and more precise:

1. Canonical live view-key derivation is closed in the hot paths and guarded by
   dedicated contract tests.
2. Backup KDF governance now uses an explicit self-describing contract aligned
   with RedB V2, while legacy compatibility remains bounded behind explicit
   helpers.
3. New-write seed salt is wallet-owned random persisted state; deterministic
   `compute_seed_salt(...)` remains a legacy compatibility helper instead of
   the write policy.
4. Gap-limit arithmetic, receiver-secret usability validation,
   `FileKeyStore` password ownership, digest framing, warning-capable
   validation DTOs, backup metadata policy, and chain-bound restore identity
   are implemented and regression-covered.
5. Remaining ambiguity is limited to explicit policy or follow-up decisions,
   not active Phase 029 blockers.

## Finding Matrix

| Cluster | Fusion refs | Current-tree evidence | Live status | Severity disposition | Execution target |
| --- | --- | --- | --- | --- | --- |
| Service-layer `wallet_service` panic risk | `FUS-05` | Runtime seed reveal, export, backup, and restore paths propagate `WalletError` through typed async boundaries; the dense `expect()` cluster remains confined to lower test or fixture helpers in `services/wallet_service.rs` | `stale_in_current_tree` | blocker downgraded to test-helper inventory only | closed in `029-04` |
| Service-layer `chain_service` `await.unwrap()` crash claim | `FUS-05` | Current runtime methods in `services/chain_service.rs:80-110` return `Result<ChainType, ChainServiceError>`; visible `unwrap()` use is confined to the `#[cfg(test)]` module (`source_ambiguity`) | `stale_in_current_tree` | downgrade to reconciliation evidence | `029-04` inventory only |
| Canonical live view-key path | `FUS-07.1` | `core/stealth/output.rs` and `core/tx/spending.rs` use `derive_view_secret_key(...)`, while `derive_rotated_view_secret_key(...)` remains confined to explicit rotation logic in `core/key/stealth_keys.rs` | `stale_in_current_tree` | hot-path contract is closed and retained as regression-only proof | closed in `029-02` |
| Sender/scanner mixed-family ambiguity question | `FUS-07.1`, `FUS-10.5` | Sender, scanner, and spend hot paths stay on the canonical live derivation family, and the proof now lives in `test_view_key_contract.rs`, `test_e2e_send_scan.rs`, and `test_rpc_key_derive_e2e.rs` | `validation_only` | keep as preserved call-graph proof, not rewrite work | closed in `029-02` |
| Gap-limit invariant masking | `FUS-07.2` | `gap_span(...)` and `next_gap(...)` in `core/key/key_manager.rs` now use checked arithmetic and return `StateCorrupted`; the key-manager regression suite covers the impossible-state paths | `stale_in_current_tree` | invariant bug is closed and remains regression-covered | closed in `029-05` |
| Cache TTL enforcement | `FUS-07.2` | `DERIVED_KEY_TTL_SECONDS` is defined at `core/key/key_manager.rs:102`; TTL checks and eviction exist at `core/key/key_manager.rs:1347-1450` with dedicated tests nearby | `validation_only` | landed hardening, keep as proof target not rewrite target | `029-05` |
| Receiver-secret validation-at-construction | `FUS-07.3` | `ReceiverSecret::from_raw(...)` now calls `validate_usable()`, and `ReceiverKeys::from_receiver_secret(...)` preserves the same fail-closed contract before key material is exposed | `stale_in_current_tree` | object-boundary issue is closed and regression-covered | closed in `029-05` |
| Legacy `WalletEncryption` salt semantics | `FUS-06.1` | `WalletEncryption::derive_key(...)` still keeps repetition padding for its compatibility surface, while RedB V2 and backup `v4` now carry the governed self-describing contract for persisted wallet and backup state | `partially_closed` | persistence and backup blockers are closed; compatibility/export seam remains explicit by policy | closed in `029-03` for persisted state |
| RedB V2 KDF baseline and unknown-version rejection | `FUS-06.1`, `FUS-06.2` | `db/redb_wallet_crypto.rs:114-160` defines `KdfParams`, versions, and `validate_untrusted_persisted()`; this is the strongest current path | `validation_only` | use as canonical baseline; no redesign needed | `029-03` |
| Backup KDF self-description | `FUS-06.2` | `core/backup/wallet_backup.rs` defines `BackupKdf`, and `core/backup/backup_exporter_impl.rs` persists explicit algorithm, salt, cost fields, and `salt_pad` semantics through `BackupKdfField::Params` | `stale_in_current_tree` | storage-contract gap is closed and regression-covered | closed in `029-03` |
| Unknown backup-KDF rejection | `FUS-06.2` | Import and verification paths resolve nested backup KDF metadata through `BackupKdf::to_params()` and reject unsupported versions before decrypt work starts | `stale_in_current_tree` | eager validation gap is closed | closed in `029-03` |
| V1 `.wlt` persisted rewrite on unlock | `FUS-06.3`, `FUS-11.1` | `db/redb_wallet_store.rs:140-194` already rewrites V1 to V2 and flushes the work file; the open path invokes it at `db/redb_wallet_store.rs:3245-3252` | `validation_only` | code exists; Phase 029 needs regression proof and clearer governance | `029-03` |
| Deterministic seed salt for new writes and reveal/export flows | `FUS-06.3`, `FUS-11.1` | `WalletService` now owns random persisted salts via `wallet_seed_salts`, `make_seed_salt()`, and `seed_salt_for_save()`; `compute_seed_salt(...)` remains present as a legacy compatibility helper only | `stale_in_current_tree` | new-write privacy gap is closed | closed in `029-04` |
| `WalletExportPack` plaintext mnemonic boundary | `FUS-08.2`, `FUS-11.1` | `WalletExportPack` still holds `seed_phrase: String` in memory, but `core/wallet/snapshot.rs` now documents it as an encrypted transport seam only, and regression tests prove the phrase stays out of public metadata and headers | `partially_closed` | policy closure is explicit; in-memory restore bundle remains intentional | closed in `029-06` |
| `FileKeyStore` password ownership | `FUS-08.1`, `FUS-11.1` | `core/storage/file_key_store.rs` now wraps password-based persistence as `EncryptionScheme::Password(SafePassword)` instead of cloneable raw bytes | `stale_in_current_tree` | secret-wrapper gap is closed | closed in `029-05` |
| Mnemonic handling inside service flows | `FUS-08.2` | The service has stronger outer seams in `services/seed_phrase.rs`, but export/import still materialize `String` mnemonic state through `WalletExportPack` and restore helpers | `partially_closed` | keep final closure in the export-boundary wave | `029-06` |
| `reveal_receiver_secret()` misuse risk | `FUS-08.2` | `ReceiverKeys::reveal_receiver_secret()` is still public in `core/key/stealth_keys.rs:387-389`; the current plan package does not require removing it yet | `explicit_open_decision` | record and revisit only if Phase 029 changes force API narrowing | summary or follow-up |
| Backup metadata privacy (`wallet_id`, `network`, `created_at`) | `FUS-06.2`, `FUS-08.3` | Backup `v4` now redacts public-header `network`, keeps `chain` inside the encrypted payload, and proves restore identity through targeted tests; `wallet_id` remains public by the explicit metadata policy | `partially_closed` | privacy policy is explicit and regression-covered | closed in `029-06` |
| Entropy-warning surfacing | `FUS-08.2`, `FUS-11` | `core/key/seed.rs` now exposes `validate_entropy_result(...)`, preserving non-fatal warnings through `RuntimeValidationResult::valid_with_warnings(...)` instead of log-only handling | `stale_in_current_tree` | validation-surface gap is closed | closed in `029-06` |
| Warning-capable validation surface | `FUS-08.2`, `FUS-11` | `RuntimeValidationResult` now carries `valid`, `warnings`, and `error`, and the wallet or RPC validation call sites use the explicit warned versus invalid helpers | `stale_in_current_tree` | API surface choice is closed | closed in `029-06` |
| Digest framing ambiguity | `FUS-09.1`, `FUS-11.1` | `build_tx_package_digest(...)` now uses `frame_str`, `frame_bytes`, and `frame_u32_le` as the canonical digest rule, with dedicated collision regression coverage | `stale_in_current_tree` | hashing ambiguity is closed | closed in `029-06` |
| `generate_identity_keypair()` public warning surface | `FUS-09.2` | The function is still public in `core/key/stealth_keys.rs:272-284`, but now carries an explicit warning block in docs | `validation_only` | doc/facade hygiene only unless wave 06 already touches the facade | `029-06` docs pass |
| `derive_s_out` facade contract | `FUS-09.2` | The fusion notes a naming/doc issue, but current-tree evidence collected for Phase 029 did not show a wallet-owned exploit or immediate drift seam in the required files | `out_of_phase_followup` | keep as audit debt unless a touched file requires local clarification | backlog if needed |
| Feature-gated debug or verbose privacy exposure | `FUS-09.3` | `core/key/key_manager.rs` documents `verbose-logging` as non-production-only, `db/redb_wallet_store.rs` keeps debug tooling feature-gated, and the final plan summary records the policy closure | `validation_only` | retain as documentation truthfulness, not active remediation | closed in `029-06` |
| `wallet_id` public-derivability question | `FUS-10.1` | Current tree confirms deterministic salt use, but does not prove whether `wallet_id` is externally derivable enough to amplify offline attack cost materially (`source_ambiguity`) | `explicit_open_decision` | mitigation direction is already fixed: stop using deterministic new-write salt | `029-04` plus summary note |
| `CipherSeedContainer` wrapper governance | `FUS-10.3` | Current tree shows `core/key/seed.rs` remains the seed container seam, but Phase 029 evidence did not justify inventing a new wrapper on Gate 0 alone (`source_ambiguity`) | `explicit_open_decision` | only close if wave 06 can do so without new format drift | `029-06` decision note |

## Runtime Versus Test Panic Inventory

📌 The service-layer panic situation is no longer one undifferentiated blocker. The
current tree splits into runtime-safe entrypoints and test-only helper density.

| File | Evidence | Classification | Required handling |
| --- | --- | --- | --- |
| `services/chain_service.rs` | Runtime methods return `Result` or deterministic values; visible `unwrap()` is in `#[cfg(test)]` network-switch tests | `test_only` | keep the downgrade in this artifact; do not spend wave time replacing test asserts |
| `services/wallet_service.rs` seed reveal/export runtime paths | `spawn_blocking(...).await?...` and `WalletError` mapping in runtime paths | `runtime_safe_currently` | preserve behavior and guard with tests |
| `services/wallet_service.rs` helper block around legacy backup/test scaffolding | Dense `expect()` use in helper code around `services/wallet_service.rs:5588-5715` | `test_or_fixture_only` | keep out of blocker-grade panic rewrites unless a runtime caller reuses the helper |
| `services/wallet_paths.rs` config resolution | `panic!("invalid wallet chain ...")` on invalid configured chain string | `operator_reachable_config_boundary` | note as non-fusion but related fail-closed follow-up if touched during wave 04 |

📌 Runtime panic closure for Phase 029 should therefore focus on operator-reachable
wallet flows, not on broad mechanical removal of test `expect()`.

## Requirement To File Map

📌 Minimum execution inventory per requirement:

📌 The earlier `proposed new file` labels are now resolved. The phase-owned test
targets below exist in the current tree, and the shared-anchor mapping remains
useful only for traceability.

| Shared phase test | Created by | Reused by |
| --- | --- | --- |
| `tests/test_backup_kdf_contract.rs` | `029-03` via `PH29-KDF` | `PH29-BACKUP` |
| `tests/test_wallet_export_pack_boundary.rs` | `029-06` via `PH29-BACKUP` | `PH29-SECRET` |
| `tests/test_receiver_secret_validation.rs` | `029-05` via `PH29-KEYMGR` | `PH29-SECRET` |

| Requirement | Minimum modules | Minimum tests | Optional expansion only if needed |
| --- | --- | --- | --- |
| `PH29-RECON` | `029-FUSION.md`, `029-CONTEXT.md`, `services/wallet_service.rs`, `services/chain_service.rs`, `core/key/key_manager.rs`, `core/key/stealth_keys.rs`, `core/security/encryption.rs`, `core/backup/backup_exporter_impl.rs`, `core/backup/backup_importer_impl.rs`, `core/backup/wallet_backup.rs`, `core/wallet/snapshot.rs`, `db/redb_wallet_crypto.rs`, `db/redb_wallet_store.rs`, `core/storage/file_key_store.rs`, `adapters/rpc/types/common.rs`, `core/hashing.rs`, `core/tx/tx_verifier.rs` | artifact-only inventory in this plan; no new Rust test target | `wallet_paths.rs`, `services/seed_phrase.rs`, plan/test docs only if a later wave expands operator-reachable scope |
| `PH29-VIEWKEY` | `core/key/stealth_keys.rs`, `core/stealth/output.rs`, `core/tx/spending.rs`, `services/wallet_service.rs` | `tests/test_view_key_contract.rs`, `tests/test_e2e_send_scan.rs`, `tests/test_rpc_key_derive_e2e.rs` | facade exports if naming or visibility changes require them |
| `PH29-KDF` | `core/security/encryption.rs`, `db/redb_wallet_crypto.rs`, `db/redb_wallet_store.rs`, `core/backup/backup_exporter_impl.rs`, `core/backup/backup_importer_impl.rs`, `core/backup/wallet_backup.rs` | `tests/test_backup_kdf_contract.rs`, `tests/test_wallet_kdf_migration.rs`, `tests/test_wallet_persistence_backup_service.rs`, `tests/test_redb_wlt_open.rs` | `tests/test_wlt_validator.rs` for malformed-contract coverage |
| `PH29-BACKUP` | `core/backup/backup_exporter.rs`, `core/backup/backup_exporter_impl.rs`, `core/backup/backup_importer_impl.rs`, `core/wallet/snapshot.rs`, `services/wallet_service.rs` | `tests/test_backup_kdf_contract.rs`, `tests/test_backup_metadata_policy.rs`, `tests/test_wallet_export_pack_boundary.rs`, `tests/test_backup_restore_identity.rs`, `tests/test_wallet_persistence_backup_service.rs` | README/docs if public policy becomes explicit |
| `PH29-PANIC` | `services/wallet_service.rs`, `services/chain_service.rs`, `services/wallet_paths.rs`, this reconciliation artifact's runtime-versus-test panic inventory | `tests/test_wallet_service_errors.rs` | no extra module unless wave 04 converts config panic into a typed failure path |
| `PH29-SEEDSALT` | `core/hashing.rs`, `core/wallet/snapshot.rs`, `db/redb_wallet_store.rs`, `services/wallet_service.rs` | `tests/test_seed_salt_policy.rs`, `tests/test_show_seed_phrase_plaintext.rs` | RPC wrappers if response metadata must change |
| `PH29-KEYMGR` | `core/key/key_manager.rs`, `core/key/stealth_keys.rs` | `tests/test_key_manager.rs`, `tests/test_receiver_secret_validation.rs` | `core/key/bip32.rs` and `core/domains.rs` only if wave `029-05` or `029-06` touches low-severity derivation or domain-label cleanup |
| `PH29-SECRET` | `core/storage/file_key_store.rs`, `core/key/stealth_keys.rs`, `core/key/seed.rs`, `core/wallet/snapshot.rs`, `services/seed_phrase.rs` | `tests/test_file_key_store.rs`, `tests/test_receiver_secret_validation.rs`, `tests/test_show_seed_phrase_plaintext.rs`, `tests/test_wallet_export_pack_boundary.rs` | docs if outer export seam becomes the chosen policy |
| `PH29-DIGEST` | `core/hashing.rs`, `core/tx/tx_verifier.rs` | `tests/test_tx_digest_framing.rs`, `tests/test_tx_tamper.rs` | none |
| `PH29-VALIDATION` | `core/security/password.rs`, `core/key/seed.rs`, `adapters/rpc/types/common.rs` | `tests/test_runtime_validation_result.rs`, `tests/test_wlt_validator.rs` | RPC method support files if DTO shape changes propagate |

## Lower-Severity Provisions And Open Decisions

📌 Items that do not become direct code waves are still mapped explicitly.

| Provision | Status | Resolution path |
| --- | --- | --- |
| `verbose-logging` / debug feature privacy note | `covered_in_later_plan` | wave `029-06` doc and regression pass |
| sequential proof verification in `ProverImpl` is operationally slower than batch verify | `out_of_phase` | performance-only note; Phase 029 keeps it outside wallet-secret remediation unless a later task broadens into proof-performance work |
| `VIEW_KEY_ACCOUNT_OFFSET = 100_000` is a policy convention, not a crypto boundary | `covered_in_later_plan` | keep as documentation truthfulness in wave `029-06` if touched |
| `generate_identity_keypair()` warning contract | `covered_in_later_plan` | doc/facade verification in wave `029-06` |
| `RistrettoBridgeDomain` version-label consistency | `out_of_phase` | current label already uses a versioned domain string; revisit only if wave `029-06` changes facade/domain governance around derivation contracts |
| spot-check ordering for key-manager cache reads | `out_of_phase` | current fusion classifies this as advisory-only documentation debt; revisit only if wave `029-05` changes cache concurrency semantics |
| brittle `NonZeroUsize::new(...).unwrap()` on constant cache size | `covered_in_later_plan` | wave `029-05` may simplify it while touching key-manager construction, but it is not a standalone blocker |
| nonce-truncation manual-review question for wallet-owned cipher containers | `open_decision` | no truncation seam is proven in required Gate 0 files; revisit only if a later wave touches a conflicting container or protocol contract |
| `derive_s_out` facade security wording | `out_of_phase` | no live exploit or drift seam proved in Gate 0; only address if touched incidentally |
| `wallet_id` public-derivability question | `open_decision` | do not block mitigation; record in summary if still unresolved |
| `CipherSeedContainer` wrapper governance | `open_decision` | only resolve if current-tree change can stay backward-compatible |

## Executed Wave Order

📌 The executed Phase 029 waves preserved this order and closed the plan-owned
runtime seams without reopening scope:

1. `029-02` closed the live view-key contract.
2. `029-03` closed backup KDF governance and persisted wallet migration.
3. `029-04` closed the runtime panic inventory and deterministic new-write seed-salt policy.
4. `029-05` closed the key-manager invariant and secret-wrapper boundaries.
5. `029-06` closed digest framing, warning-capable validation, backup metadata policy,
   mnemonic export policy, and chain-bound restore identity.

📌 Current-tree reconciliation is satisfied because the remaining rows are now
either validation evidence, explicit policy decisions, or out-of-phase follow-up.
