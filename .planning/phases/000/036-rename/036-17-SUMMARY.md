---
phase: 036-rename
plan: 17
status: completed
updated: 2026-04-19
---

# 036-17 Summary

## Scope

This summary records the Wave 7 follow-on cleanup and closure for `036-17-PLAN.md` on the active `036-a1-versioning-spec-V3.md` inventory chain.

## Outcome

`036-17` closes the V3 follow-on inventory truthfully. The scoped storage, crypto, and wallet declarations were renamed or removed so the remaining version markers no longer appear in the owned files, while the encoded bytes, persisted values, and contract semantics stayed unchanged.

Representative closures include:

- `ProofBlobV0` and `ClaimNullRecV0` neutralized on the storage side
- `encode_single_v2`, `encode_dual_v2`, and `decode_v2` replaced by version-neutral address helpers
- `CLAIM_PROOF_V2` replaced by `CLAIM_SOURCE_PROOF_TAG`
- `export_public_material_v2` replaced by the canonical unsuffixed `wallet.key.export_public_material` lane
- wallet-local test and helper cleanup such as `test_snapshot_v3_verify_ok`, `test_receiver_card_record_v1_is_canonical_live_contract`, and `wallet_key_export_public_material_v2_is_canonical_live_contract`
- the final V3 inventory appendix note that records the clean residual scan proof

This closeout is intentionally scoped to the V3 inventory follow-on. It does not claim broader repository-wide version removal beyond the rows proven in scope.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test -p z00z_crypto --release --features test-fast --features wallet_debug_dump --quiet`: passed
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --quiet`: passed
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --quiet`: passed
- scoped residual scan across the V3 files: passed with no matches

## Review Loop

The required review passes were completed in YOLO mode during the live cleanup, and the final deterministic reruns plus residual scan did not introduce any material issue.

## Canonical Artifact Sync

- `.planning/phases/036-rename/036-a1-versioning-spec-V3.md`
- `.planning/phases/036-rename/036-17-SUMMARY.md`
- `.planning/STATE.md`
- `.planning/ROADMAP.md`

## Current Boundary

`036-17` is summary-backed complete. The live execution pointer advances to `036-18-PLAN.md`.
