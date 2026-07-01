# Phase 030 Deferred Items

## 030-01 Follow-Ups

- Resolved in the continuation pass: live wallet session constructors now funnel concrete `RedbWltStore` creation through one helper seam instead of repeating backend wiring across the session build variants.
- Resolved in the continuation pass: the remaining wallet service tests for this area now stay on seam-boundary assertions or trait-backed doubles instead of pinning new behavior to repeated concrete RedB construction sites.

## 030-06 Follow-Ups

- Resolved in the continuation pass: `KeyManager::unlock_from_storage()` now rebuilds an encrypted-seed-backed runtime state, and regression coverage verifies that `to_state()` plus `change_password()` still work after storage unlock.

## 030-09 Follow-Ups

- Resolved in the continuation pass: isolated long-test measurement in `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` now has a bounded timeout so post-summary collection records stuck tests instead of hanging indefinitely.

## 030-14 Follow-Ups

- Resolved in the continuation pass: load or save or export or import persistence paths now recover wallet-bound identity from the live session map or `.wlt` discovery metadata instead of silently rebinding to `resolve_wallet_identity()`. Export/import payloads also preserve persisted wallet identity for round-trips.
- Resolved in the continuation pass: wallet examples now document hashed filenames via `wallet_<wallet_id_hex>.wlt`, matching the runtime path built by `compute_wallet_file_id(...)`.
- Resolved in the continuation pass: older inactive `wallet_service_store_*` residue files that sat outside the active include graph were removed, eliminating the malformed seam hazard if those legacy files were accidentally reopened.

## 030-23 Follow-Ups

- Re-triage in the continuation pass found the `experimental-zkpack` feature gate fix already present in the live `z00z_crypto` tree. The old note was stale and is no longer an active deferred item for Phase 030.
