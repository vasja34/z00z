# Garbage Filter Audit

## 🔑 Classification Rules

- `InProduction = TRUE`: active current path or active compatibility/migration path proven by live callers.
- `InProduction = FALSE`: debug-only, test-only, or dead support path not used by the main processing flow.
- `Comments = GARBAGE`: safe candidate for removal review because it is test-only or a stale no-op helper.

| Path | Name | Kind | VariationOf | AllVariants | Role | InProduction | Evidence | Comments |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `crates/z00z_crypto/src/claim/statement.rs` | `GenesisClaimStatement` | `struct` | `claim statement surface` | `GenesisClaimStatement, ClaimStmt` | Legacy Stage-3 claim statement | `TRUE` | Exported from `z00z_crypto::lib`; used by prover and verifier | Legacy flow still shipped beside V2 |
| `crates/z00z_crypto/src/claim/statement.rs` | `statement_hash` | `fn` | `claim statement hash` | `statement_hash, claim_stmt_hash_v2` | Legacy statement hash helper | `TRUE` | Called by legacy claim prover and verifier | Keep as active legacy path |
| `crates/z00z_crypto/src/claim/v2.rs` | `ClaimAuthoritySig` | `struct` | `claim authority signature` | `GenesisClaimAuthoritySig, ClaimAuthoritySig` | Canonical statement-signing authority signature | `TRUE` | Exported from `z00z_crypto::lib`; the live claim statement-signing surface uses it directly | Current path, not garbage |
| `crates/z00z_crypto/src/claim/proof.rs` | `GenesisClaimProof` | `struct` | `claim proof object` | `GenesisClaimProof, ClaimSourceProof` | Legacy Stage-3 proof object | `TRUE` | Exported from `z00z_crypto::lib`; used by legacy claim verifier | Legacy path still present |
| `crates/z00z_storage/src/assets/store_internal/test_whitebox_proofs.rs` | `LegacyProofBlob` | `struct` | `proof blob wire` | `ProofBlob, ProofBlobV0, LegacyProofBlob` | Whitebox legacy fixture blob | `FALSE` | Exists only in test/whitebox module | GARBAGE |
| `crates/z00z_storage/src/assets/store_internal/redb_backend_state.rs` | `ClaimNullRecV0` | `struct` | `claim nullifier row` | `ClaimNullRec, ClaimNullRecV0` | Legacy persisted row upgrade shape | `TRUE` | RedB backend decodes `ClaimNullRecV0` then upgrades into `ClaimNullRec` | Active migration path |
| `crates/z00z_storage/src/checkpoint/ids.rs` | `ArtWire` (top-level shell) | `struct` | `checkpoint artifact statement wire` | `LegacyArtWire, ArtWire, CheckpointStmtV1` | Test-only top-level artifact shell used in ID tests | `FALSE` | Present only as the top-level test shell in `ids.rs`; the surviving local replacement is `UnsupportedVersionArtWire` | GARBAGE |
| `crates/z00z_wallets/src/core/backup/backup_wire.rs` | `BackupContainer` | `struct` | `wallet backup container` | `LegacyBackupContainer, BackupContainer` | Canonical encrypted-backup wire container | `TRUE` | Decoded and validated by backup exporter/importer verification and import flows | Current backup wire surface |
| `crates/z00z_wallets/src/core/backup/wallet_backup.rs` | `derive_key_with_kdf` | `fn` | `wallet backup key derivation` | `derive_key, derive_key_legacy_v1, derive_key_with_kdf` | Canonical KDF-dispatched derivation entrypoint | `TRUE` | Used by exporter, importer, and verification helpers | Main live entrypoint |
| `crates/z00z_wallets/src/core/backup/wallet_backup.rs` | `derive_key_legacy_v1` | `fn` | `wallet backup key derivation` | `derive_key, derive_key_legacy_v1, derive_key_with_kdf` | Public convenience wrapper for legacy KDF | `FALSE` | Only direct hits were test calls; live importer uses `derive_key_with_kdf()` instead | Support-only; removal possible after API review |
| `crates/z00z_wallets/src/core/key/seed_cipher_params.rs` | `Argon2idParams` | `struct` | `wallet seed KDF params` | `Argon2idParams` | Persisted seed cipher parameter shape | `TRUE` | Used by wallet seed cipher parameter handling and persistence | Current persisted parameter surface |
| `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs` | `derive_key_v1_repetition_padding` | `fn` | `wallet RedB master-key derivation` | `derive_key_v1_repetition_padding, derive_key_v2_zero_padding` | Legacy salt-padding derivation | `TRUE` | `redb_wallet_crypto.rs` dispatches to V1 for old persisted version | Active migration path |
| `crates/z00z_wallets/src/db/redb_wallet_crypto_kdf_helpers.rs` | `derive_wallet_keys_v1` | `fn` | `wallet RedB HKDF expansion` | `derive_wallet_keys_v1, derive_wallet_keys_v2` | Legacy HKDF-info expansion | `TRUE` | `redb_wallet_crypto.rs` dispatches on `HKDF_INFO_VER_V1` | Active migration path |
| `crates/z00z_wallets/src/db/redb_wallet_store_debug_export.rs` | `debug_export_wallet` | `fn` | `debug wallet dump pipeline` | `debug_export_wallet, verify_debug_wallets, enrich_debug_dump_with_assets` | Wallet debug dump exporter | `FALSE` | Re-exported only behind `wallet_debug_tools`; consumed by feature-gated simulator debug-dump and post-claim export paths | DEBUG-ONLY |
| `crates/z00z_simulator/src/scenario_1/runner_verify.rs` | `verify_debug_wallets` | `fn` | `debug wallet dump pipeline` | `debug_export_wallet, verify_debug_wallets, enrich_debug_dump_with_assets` | Scenario-side debug dump verifier | `FALSE` | Called only from simulator verification flow | DEBUG-ONLY |
| `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs` | `enrich_debug_dump_with_assets` | `fn` | `debug wallet dump pipeline` | `debug_export_wallet, verify_debug_wallets, enrich_debug_dump_with_assets` | Adds asset snapshots to debug dumps | `FALSE` | Local simulator helper only | DEBUG-ONLY |
| `crates/z00z_wallets/src/core/tx/state_checkpoint.rs` | `_keep_checkpoint_draft` | `fn` | `checkpoint draft keepalive helper` | `_keep_checkpoint_draft` | No-op keeper stub | `FALSE` | No direct live callers; function body is empty | GARBAGE |

## ⚠️ Filtering Notes

- Do not remove version-suffixed types only because the suffix looks old. In this repository many `V0` and `V1` names are still the active compatibility boundary.
- The strongest keep-set is in `z00z_storage` and `z00z_wallets`, where legacy decode and migration ladders are intentionally live.
- The cleanest garbage candidates found in this pass are `LegacyProofBlob`, the top-level `ArtWire` shell in `crates/z00z_storage/src/checkpoint/ids.rs`, and `_keep_checkpoint_draft`.
- Simulator debug dump helpers are working code, but they are not part of the production processing path.

## Garbage Classification Freeze - 2026-04-12

- `035-a3-garbage-filter.md` is the sole classification authority for the
  garbage lane in Phase 035.
- `InProduction = FALSE` plus `GARBAGE` is the only default immediate removal
  signal in this lane.
- `DEBUG-ONLY` stays a reviewed non-production lane and cannot become a delete
  wave until the full cluster is intentionally reviewed together.
- `InProduction = TRUE` remains a keep-set or review-only signal, even when a
  row looks legacy by name.
- The stronger user target `leave only current production-path` remains source
  drift in this phase slice until the canonical table itself demotes the live
  compatibility or migration rows.

## Hard-Garbage Removal Cluster - 2026-04-12

The first hard-garbage removal wave stays intentionally narrow.

- `LegacyProofBlob` is removed only as a top-level stale whitebox helper shape;
  the legacy decode test may keep an inline local legacy shell where needed.
- `ArtWire` is removed only as the top-level ID-test artifact shell in
  `crates/z00z_storage/src/checkpoint/ids.rs`; the unsupported-version test in
  that file may keep its local `UnsupportedVersionArtWire` replacement where
  needed.
- `_keep_checkpoint_draft` is removed completely because it is an empty
  keepalive stub with no live callers.
- No debug-only cluster item and no `InProduction = TRUE` compatibility or
  migration seam enters this hard-garbage wave.

## Debug-Dump Retirement Review - 2026-04-12

- `debug_export_wallet`, `verify_debug_wallets`, and
  `enrich_debug_dump_with_assets` stay one reviewed non-production cluster.
- The cluster remains outside the default production path, but it is still
  consumed by simulator debug-dump emission, simulator verification, and
  post-claim inspection under `wallet_debug_dump` or `wallet_debug_tools`
  gates.
- Phase 035 does not retire any one member of this trio in isolation.
- Until the full simulator-side debug story is intentionally rewritten, the
  truthful status is `explicitly deferred with a source-backed reason`, not
  `ready for deletion now`.

## Compatibility And Migration Keep-Set Freeze - 2026-04-12

- Every `InProduction = TRUE` compatibility or migration row remains outside
  the immediate delete lane in this phase slice.
- The strongest keep-set remains in storage and wallet migration ladders,
  especially `ClaimNullRecV0`, `derive_key_with_kdf`,
  `derive_key_v1_repetition_padding`, and `derive_wallet_keys_v1`.
- Legacy claim surfaces in `z00z_crypto` stay frozen as live compatibility
  seams until a canonical source update says otherwise.
- `derive_key_legacy_v1` is not promoted into the garbage lane by name alone;
  it remains support-only and subject to explicit API review before any
  removal plan can be generated honestly.

## Current-Path-Only Source Drift Handoff - 2026-04-12

- The stronger user target `leave only current production-path` still exceeds
  the current canonical table.
- A truthful current-path-only delete wave would first require source-side
  demotion of the live compatibility or migration rows that the table still
  marks `InProduction = TRUE`.
- The minimum demotion set currently includes:
  `GenesisClaimStatement`, `statement_hash`, `GenesisClaimProof`,
  `ClaimNullRecV0`, `derive_key_v1_repetition_padding`, and
  `derive_wallet_keys_v1`.
- `derive_key_legacy_v1` additionally requires explicit API-review resolution
  before a future stronger cleanup can classify it as removable instead of
  support-only.
- This handoff is advisory only: update the canonical table first, then
  regenerate any stronger delete backlog.

## Garbage-Filter Validation Wave - 2026-04-12

- The validation sweep covered the expanded planning surface for this garbage
  lane: `ROADMAP.md`, `STATE.md`, `035-09-PLAN.md`, `035-09-SUMMARY.md`,
  `035-a3-garbage-filter.md`, and `035-TODO.md`.
- The same validation wave also preserved live-code evidence: the explicit
  inventory grep over the named garbage, debug, and keep-set rows plus manual
  verification that the keep-set rows still have live-caller evidence and the
  hard-garbage rows do not.
- The sweep resolved stale symbol names, stale reviewed-surface file lists,
  stale continuity metadata, and stale completion-gate wording until the
  garbage lane no longer overclaimed its closeout state.
- The hard-garbage cluster remains limited to `LegacyProofBlob`, the top-level
  `ArtWire` shell in `crates/z00z_storage/src/checkpoint/ids.rs`, and
  `_keep_checkpoint_draft`.
- The reviewed non-production debug trio remains explicitly deferred as one
  simulator-backed cluster, and the compatibility or migration keep-set stays
  frozen out of the delete lane.

## Current-Path Closure Gate - 2026-04-12

- The garbage lane closes only on validated hard-garbage removal plus explicit
  deferral of the debug trio; it does not claim blanket retirement of legacy
  compatibility or migration seams.
- The stronger user target `leave only current production-path` remains source
  drift, not approved execution truth, while `GenesisClaimStatement`,
  `statement_hash`, `GenesisClaimProof`, `ClaimNullRecV0`,
  `derive_key_v1_repetition_padding`, and `derive_wallet_keys_v1` remain live
  canonical keep-set rows.
- The expanded planning surface already exceeded the minimum three-pass review
  requirement while correcting drift across five blocked passes, and garbage
  lane closure is accepted only on clean passes 6 and 7, the first two
  consecutive clean read-only passes that follow those five blocked
  corrections on the same six-file planning surface.

## 🔗 TODO One-To-One Mapping

| 035-3 section | Task coverage | Mapping note |
| --- | --- | --- |
| `Classification Rules` | `035-15`; `035-16`; `035-17`; `035-18`; `035-19`; `035-20`; `035-21` | the whole delete-or-keep lane is derived from `InProduction` and comment labels |
| `Filtering Notes` | `035-15`; `035-16`; `035-17`; `035-18`; `035-19`; `035-20`; `035-21` | the keep-set, debug-only lane, and current-path-only drift warning all become explicit tasks |
| `Debug-Dump Retirement Review - 2026-04-12` | `035-17`; `035-20`; `035-21` | the debug trio is deferred as one simulator-backed non-production cluster |
| `Compatibility And Migration Keep-Set Freeze - 2026-04-12` | `035-18`; `035-19`; `035-20`; `035-21` | live compatibility and migration seams stay frozen out of the delete lane |
| `Current-Path-Only Source Drift Handoff - 2026-04-12` | `035-19`; `035-21` | stronger cleanup remains blocked until the canonical table itself demotes the keep-set |
| `Garbage-Filter Validation Wave - 2026-04-12` | `035-20`; `035-21` | the expanded planning surface is validated before garbage-lane closure is claimed |
| `Current-Path Closure Gate - 2026-04-12` | `035-21` | the lane closes only on hard-garbage removal plus explicit debug-cluster deferral |
