---
phase: 029-crypto-audit-wallets
plan: "06"
subsystem: wallets
tags: [wallets, digest, validation, backup, metadata, export-boundary]
requires:
  - phase: 029-crypto-audit-wallets
    provides: 029-04 and 029-05 hardening baseline for final ambiguity closure
provides:
  - explicitly framed transaction package digest semantics
  - warning-capable runtime validation result contract with observable seed-entropy warnings
  - explicit backup metadata privacy policy with encrypted payload ownership
  - documented encrypted-only WalletExportPack boundary plus regression coverage
affects: []
tech-stack:
  added: []
  patterns: [frame helper reuse, aligned warning DTOs, redacted public headers, explicit encrypted mnemonic boundary]
key-files:
  created:
    - .planning/phases/029-crypto-audit-wallets/029-06-SUMMARY.md
    - crates/z00z_wallets/tests/test_tx_digest_framing.rs
    - crates/z00z_wallets/tests/test_runtime_validation_result.rs
    - crates/z00z_wallets/tests/test_backup_metadata_policy.rs
    - crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs
    - crates/z00z_wallets/tests/test_backup_restore_identity.rs
  modified:
    - crates/z00z_wallets/README.md
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/types/common.rs
    - crates/z00z_wallets/src/adapters/rpc/types/key.rs
    - crates/z00z_wallets/src/core/backup/backup_exporter.rs
    - crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs
    - crates/z00z_wallets/src/core/backup/backup_importer_impl.rs
    - crates/z00z_wallets/src/core/security/password.rs
    - crates/z00z_wallets/src/core/tx/tx_verifier.rs
    - crates/z00z_wallets/src/core/wallet/snapshot.rs
    - crates/z00z_wallets/src/core/wallet/wallet.rs
key-decisions:
  - "Use the existing frame_* helpers as the only canonical digest framing rule for build_tx_package_digest()."
  - "Expose warnings explicitly through RuntimeValidationResult instead of collapsing non-fatal validation findings into logs or binary DTO state."
  - "Redact privacy-sensitive backup metadata from public headers and keep the effective network inside the encrypted payload."
  - "Treat WalletExportPack as an encrypted transport seam only and document that plaintext seed_phrase must not cross public metadata surfaces."
patterns-established:
  - "One digest builder: all tx package consumers share the framed tx_verifier builder instead of ad hoc concatenation."
  - "Observable policy closure: validation warnings, backup metadata, and mnemonic export boundaries are explicit in code, tests, and crate docs."
requirements-completed: [PH29-DIGEST, PH29-VALIDATION, PH29-BACKUP, PH29-SECRET]
duration: checkpointed
completed: 2026-03-30
---

# Phase 029 Plan 06: Digest And Metadata Policy Summary

Explicit tx digest framing, warning-capable runtime validation, and an encrypted-only backup or mnemonic boundary with tested metadata privacy rules.

## Performance

- **Duration:** checkpointed
- **Started:** not recorded in resumed session
- **Completed:** 2026-03-30T16:45:23Z
- **Tasks:** 2
- **Files modified:** 18

## Accomplishments

- 📌 Reframed `build_tx_package_digest()` to use the canonical wallet framing helpers so ambiguous variable-length tuples can no longer collide.
- 📌 Extended `RuntimeValidationResult` with warnings and added a seed-entropy validation helper that exposes non-fatal warnings instead of leaving them log-only.
- 📌 Finalized backup format version 4 with public-header `network` redaction and encrypted-payload ownership of the effective restore identity (`network` plus `chain`).
- 📌 Documented `WalletExportPack` as an encrypted transport boundary only and added focused tests for digest framing, warning DTOs, metadata policy, and export-boundary behavior.

## Task Commits

The resumed execution wave landed the plan-owned implementation in one version-managed commit because the validated code and regression files were already present in a single working-tree slice:

1. **Task 1: Reframe `build_tx_package_digest()` with the existing wallet hashing helpers** - `a3a26e08` (feat, shared implementation commit)
2. **Task 2: Finalize warning-capable validation and backup-metadata disclosure policy** - `a3a26e08` (feat, shared implementation commit)

**Plan metadata:** recorded in the final docs commit for Plan 06 closure

## Files Created Or Modified

- `crates/z00z_wallets/src/core/tx/tx_verifier.rs` - canonical framed digest builder and regression anchor against context-boundary collisions.
- `crates/z00z_wallets/src/adapters/rpc/types/common.rs` - warning-capable `RuntimeValidationResult` helpers for valid, warned, and invalid states.
- `crates/z00z_wallets/src/core/key/seed.rs` - explicit entropy validation result helper that preserves non-fatal warnings as runtime data.
- `crates/z00z_wallets/src/core/security/password.rs` - password validation aligned to the explicit warning-capable DTO helpers.
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs`, `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs`, and `crates/z00z_wallets/src/adapters/rpc/types/key.rs` - RPC validation call sites aligned to the final DTO contract.
- `crates/z00z_wallets/src/core/backup/backup_exporter.rs` - documented v4 public-header redaction semantics.
- `crates/z00z_wallets/src/core/backup/backup_exporter_impl.rs` - explicit backup payload v4 with encrypted restore-identity ownership and public metadata minimization.
- `crates/z00z_wallets/src/core/backup/backup_importer_impl.rs` - explicit v1, v2, v3, and v4 compatibility handling with restore identity recovered from the encrypted payload.
- `crates/z00z_wallets/src/core/wallet/snapshot.rs` - explicit encrypted-only `WalletExportPack` boundary documentation.
- `crates/z00z_wallets/src/core/wallet/wallet.rs` - placeholder validation path aligned to the explicit RuntimeValidationResult helpers.
- `crates/z00z_wallets/README.md` - crate docs updated with backup header privacy and export-boundary policy.
- `crates/z00z_wallets/tests/test_tx_digest_framing.rs` - focused regression coverage for ambiguous digest inputs.
- `crates/z00z_wallets/tests/test_runtime_validation_result.rs` - DTO contract coverage for warnings and invalid states.
- `crates/z00z_wallets/tests/test_backup_metadata_policy.rs` - regression coverage for public-header privacy versus encrypted metadata ownership.
- `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs` - regression coverage for the explicit encrypted mnemonic/export boundary.
- `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs` - verification-only RPC backup tests updated to the current encrypted export payload contract.

## Decisions Made

- 📌 Transaction package digests now have one explicit framing rule that reuses `frame_str`, `frame_bytes`, and `frame_u32_le`.
- 📌 Validation warnings remain first-class runtime data rather than hidden review knowledge or log-only output.
- 📌 Public backup headers stay minimal while privacy-sensitive restore context lives inside the encrypted payload.
- 📌 `WalletExportPack` is allowed only as an encrypted outer export seam; it is not a public metadata or plaintext default boundary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated stale RPC backup-import tests to the current encrypted export contract**

- **Found during:** final release-style crate verification
- **Issue:** `wallet_import_accepts_valid_backup` and `wallet_add_accepts_valid_backup` still constructed a superseded backup shape and then retried with the wrong password after switching to the new payload.
- **Fix:** Reworked both tests to generate a real encrypted export via `export_wallet_payload(...)`, serialize the current `RuntimeEncryptedResponse`, and use the actual export password when invoking the RPC import or add path.
- **Files modified:** `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs`
- **Verification:** `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump`
- **Committed in:** `a3a26e08`

---

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** The fix did not widen production scope, but it was required to prove the final metadata policy against the current encrypted export boundary instead of a stale legacy fixture.

## Issues Encountered

- The working tree already contained the validated plan-owned implementation before formal execute-phase closure, so the plan landed as one shared implementation commit instead of two isolated task commits.
- Per-file Codacy analysis on the verification-driven RPC test update reported only pre-existing complexity warnings outside the current fix.

## User Setup Required

None.

## Next Phase Readiness

- 📌 Phase 029 now has explicit digest, validation, backup metadata, and mnemonic-boundary policy coverage across code, tests, and docs.
- 📌 The broader release-style wallet crate gate is green, including the previously stale RPC backup-import regression anchors.

## Self-Check

PASSED

- Verified artifact exists: `crates/z00z_wallets/tests/test_tx_digest_framing.rs`
- Verified artifact exists: `crates/z00z_wallets/tests/test_runtime_validation_result.rs`
- Verified artifact exists: `crates/z00z_wallets/tests/test_backup_metadata_policy.rs`
- Verified artifact exists: `crates/z00z_wallets/tests/test_wallet_export_pack_boundary.rs`
- Verified summary exists: `.planning/phases/029-crypto-audit-wallets/029-06-SUMMARY.md`
- Verified task commit exists: `a3a26e08`

---
*Phase: 029-crypto-audit-wallets*
*Completed: 2026-03-30*
