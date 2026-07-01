# Phase 047 Attack Surface Remediation

**Finding ID:** `AS-20260520-001`
**Updated:** 2026-05-21
**Status:** remediated
**Result:** implemented, validated, and doublechecked

## 🎯 Target Finding

The accepted Phase 047 finding was the Stage 3 claim-lane debug dump leak: `export_wallet_debug_{name}.json` persisted decrypted `seed_phrase` and `plaintext_b64` fields into the normal claim artifact lane, and the adjacent `wallets_export_import/export_wallet_debug_post_claim.json` sink used the same exporter shape.

## ⚙️ Ranked Mitigation Candidates

### ✅ Candidate 1: Source-level redaction plus sink verification

- Stop serializing decrypted secret rows from `DebugWalletDump` at the wallet exporter layer.
- Keep simulator sink-level scrubbing as defense in depth.
- Upgrade runtime verification to assert redaction for both claim-lane and post-claim JSON sinks.
- Add focused regressions for both sinks.

**Pros:** closes the root persistence path, preserves existing JSON artifact locations needed by current asset/proof consumers, and fails closed at runtime.

**Cons:** the exporter still carries plaintext-capable internal secret-handling code paths in memory, even though it no longer writes them to disk.

**Decision:** selected

### ⚠️ Candidate 2: Move Stage 3 dumps to the private Stage 2 lane

- Relocate Stage 3 debug JSON artifacts out of `outputs/claim/`.

**Pros:** restores a strict private-lane boundary.

**Cons:** changes current artifact locations, widens compatibility risk for Phase 047 claim consumers, and does not help the adjacent post-claim sink by itself.

**Decision:** rejected

### ⚠️ Candidate 3: Disable Stage 3 debug dumps entirely

- Remove the Stage 3 and post-claim debug JSON outputs under `wallet_debug_dump`.

**Pros:** strongest reduction of persisted debug surface.

**Cons:** drops current debug artifact functionality and exceeds the narrow compatibility-preserving fix needed for the accepted finding.

**Decision:** rejected

## ✅ Landed Changes

- `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_types.rs`
  - Added `secrets_redacted: bool` to `DebugWalletDump`.
- `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_export.rs`
  - The final serialized dump now writes `secrets: []` and `secrets_redacted: true` instead of persisting decrypted secret rows.
- `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs`
  - Kept sink-level redaction for claim and post-claim debug JSON files as defense in depth.
- `crates/z00z_simulator/src/scenario_1/runner_verify.rs`
  - Upgraded verification from file-existence checks to content checks for both `outputs/claim/export_wallet_debug_{name}.json` and `wallets_export_import/export_wallet_debug_post_claim.json`.
  - The verifier now rejects any dump that still contains `seed_phrase`, `plaintext_b64`, non-empty `secrets[]`, or missing `secrets_redacted=true`.
- `crates/z00z_simulator/tests/test_stage2_secret_artifacts.rs`
  - Added a regression test for Stage 3 claim-lane redaction.
- `crates/z00z_simulator/tests/test_claim_post.rs`
  - Added a regression test for post-claim debug dump redaction.
- `crates/z00z_simulator/src/scenario_1/stage_3_finalize.rs`
  - Updated the Stage 3 comment to describe the lane as redacted instead of private.

## ✅ Validation A

Command:

```bash
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_stage3_claim_debug_dumps_redact_wallet_secrets -- --nocapture
```

Result:

- Passed.
- The focused scenario run completed with Stage 3 and Stage 13 success.
- Claim-lane dumps remained present, parseable, and redacted.
- Runtime verification covered both the claim lane and the post-claim debug lane.

## ✅ Validation B

Commands:

```bash
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_post_claim_debug_dump_redacts_wallet_secrets -- --nocapture
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_post_claim_byte_identity -- --nocapture
```

Result:

- Passed.
- The post-claim debug dump remains present for the feature-gated flow but no longer persists wallet secret plaintext.
- The post-claim export/import path still completes without breaking adjacent artifact expectations.

## 🔔 Historical Artifact Cleanup

- Existing leaked `export_wallet_debug*.json` artifacts under `crates/z00z_simulator/src/scenario_1/outputs/` were moved to trash.
- Existing leaked `export_wallet_debug*.json` artifacts under `crates/z00z_simulator/target/stage3_acceptance/` were moved to trash.
- A post-cleanup scan of `crates/z00z_simulator/target/stage3_acceptance/` returned `0` remaining `export_wallet_debug*.json` files.

## ✅ Doublecheck

**Date:** 2026-05-21
**Verdict:** PASS-WITH-NOTES

Doublecheck confirmed that the on-disk persistence issue is closed:

- final wallet debug serialization now emits `secrets: []` with `secrets_redacted: true`
- simulator sink-level scrub remains in place as defense in depth
- runtime verification covers both claim and post-claim sinks
- regression tests assert the same contract
- live leaked debug JSON artifacts are no longer present in the checked workspace paths

## ⚠️ Residual Risk

- The exporter still contains plaintext-capable `DebugSecretEntry` fields and internal secret decryption logic before final serialization drops the rows.
- The accepted attack surface was on-disk persistence, and that persistence is now closed, but exporter internals are not yet reduced to a by-construction no-plaintext schema.
- Historical artifacts outside this workspace, such as external CI archives or copied local outputs, remain readable until separately scrubbed or rotated.

## ✅ Final Decision

Candidate 1 is accepted and implemented. The accepted Phase 047 finding `AS-20260520-001` is remediated for the live repository state, with focused validation and a follow-up doublecheck recorded here.
