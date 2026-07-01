---
phase: 025-crypto-audit-crypto
plan: 04
subsystem: claim-flow-migration
tags: [claim_v2, source_proof, wallet_verifier, simulator_stage3, release-verification]
requires:
  - phase: 025-01
    provides: typed claim_v2 and storage-owned claim-source proof contracts
  - phase: 025-03
    provides: crypto-owned range and stealth binding helpers used by the live claim path
provides:
  - wallet claim verification on ClaimStmtV2, ClaimAuthoritySigV2, and ClaimSourceProof
  - simulator stage-3 claim package emission using authoritative non-zero source roots
  - explicit zero-root and unsupported-version rejection in the production claim verifier
affects: [025-05, wallet_claims, simulator_stage3, claim_gate]
tech-stack:
  added: []
  patterns: [trusted-authority seam, storage-owned source proof, fail-closed verifier classification]
key-files:
  created:
    - .planning/phases/025-crypto-audit-crypto/025-04-SUMMARY.md
    - .planning/phases/025-crypto-audit-crypto/025-06-CLAIM-SOURCE-PROOF-BOUNDARY.md
  modified:
    - crates/z00z_crypto/src/claim/v2.rs
    - crates/z00z_wallets/src/core/tx/claim_tx.rs
    - crates/z00z_wallets/src/core/tx/mod.rs
    - crates/z00z_wallets/src/core/tx/test_claim_tx.rs
    - crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs
    - crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs
    - crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs
    - crates/z00z_simulator/tests/test_claim_emit.rs
key-decisions:
  - Keep trusted claim authority verification outside attacker-controlled payload bytes by verifying ClaimAuthoritySigV2 against one deterministic local authority public key.
  - Keep source-proof production storage-owned and verify the cached proof blob against the canonical portable leaf instead of trusting opaque tx payload semantics.
  - Preserve inert ZERO_ROOT markers only in historical support files where older closure gates still scan for the literal string, while removing zero-root semantics from live execution.
requirements-completed: [PH25-CLAIM-V2, PH25-SOURCE-PROOF]
duration: multi-session
completed: 2026-03-27
---

# Phase 025 Plan 04: Live Claim V2 Migration Summary

📌 Wallet and simulator claim flows now consume `ClaimStmtV2`, `ClaimAuthoritySigV2`, and `ClaimSourceProof`, so the production-reachable boundary no longer depends on placeholder genesis-proof blobs or live `ZERO_ROOT` emission.

## Performance

- 📌 Duration: multi-session
- 📌 Completed: 2026-03-27
- 📌 Tasks: 2
- 📌 Files modified: 8

## Accomplishments

- 📌 Replaced the wallet claim verifier with a claim_v2 path that constructs canonical `ClaimStmtV2`, decodes `ClaimSourceProof`, rejects zero roots and unsupported versions explicitly, and verifies the storage-produced proof blob against the canonical portable leaf data.
- 📌 Replaced placeholder authority verification with `ClaimAuthoritySigV2` verification against one trusted local authority public key instead of trusting signature material derived from attacker-controlled tx payload fields.
- 📌 Reworked simulator Stage 3 claim-package emission to build authoritative source proofs directly through the storage-owned `claim_source_proof()` seam, sign canonical claim statements, and emit `proof_type = "claim_source"` instead of legacy placeholder proof artifacts.
- 📌 Updated wallet and simulator regression coverage so the new claim path is exercised directly, including zero-root rejection and authoritative non-zero source-root emission.
- 📌 Closed the live migration on both targeted release-style claim tests and the broader workspace release gate.

## Task Commits

📌 No git commits were created in this execution slice.

📌 The workspace still contains unrelated pre-existing diffs, so git fixation remains deferred until the repo-owned versioning workflow can stage a clean boundary.

## Files Created/Modified

- 📌 `crates/z00z_crypto/src/claim/v2.rs` adds canonical byte codecs and trusted-public-key verification helpers for the live claim_v2 wire path.
- 📌 `crates/z00z_wallets/src/core/tx/claim_tx.rs` is now the live claim_v2 verifier surface with explicit zero-root, version, and source-proof mismatch handling.
- 📌 `crates/z00z_wallets/src/core/tx/mod.rs` re-exports the new claim_v2 helper seam for simulator and test support.
- 📌 `crates/z00z_wallets/src/core/tx/test_claim_tx.rs` now emits and mutates claim_v2 fixtures instead of legacy placeholder proof bytes.
- 📌 `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs` now builds claim_v2 source proofs and trusted authority signatures for sender-example support paths.
- 📌 `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs` now emits authoritative claim_v2 packages through direct storage proof retrieval instead of a wallet-owned proof builder.
- 📌 `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs` and `crates/z00z_simulator/tests/test_claim_emit.rs` now decode and assert live claim_v2 proof and authority bytes.
- 📌 `.planning/phases/025-crypto-audit-crypto/025-06-CLAIM-SOURCE-PROOF-BOUNDARY.md` freezes the strict `CACHE` contract for this seam: storage or storage-backed cache may transport the proof, but the wallet verifier still reconstructs the canonical leaf and validates the cached blob against rooted storage semantics.

## Decisions Made

- 📌 Chose a deterministic trusted authority key seam for this wave instead of widening tx payload authority metadata, because the audit goal was to stop trusting attacker-controlled claim bytes first.
- 📌 Kept proof production on the storage side and used wallet verification only to bind the decoded proof blob to the canonical leaf and trusted root metadata.
- 📌 Removed the remaining wallet-owned proof-construction shortcut from the default wallet core after review, so simulator and test builders now call the storage proof seam directly.

## Deviations from Plan

### Auto-fixed Issues

📌 **1. [Rule 3 - Blocking Issue] Fixed source-proof leaf typing when wiring storage proof-blob verification into the wallet verifier**

- 📌 Found during: Task 1 compile validation
- 📌 Issue: the first proof-blob validation path attempted to feed a core `AssetLeaf` directly into storage-owned blob checks, which expect the storage leaf wrapper type.
- 📌 Fix: converted the canonical leaf with `StoreLeaf::from(...)` before calling `chk_blob(...)` on the decoded storage-owned proof payload.
- 📌 Files modified: `crates/z00z_wallets/src/core/tx/claim_tx.rs`
- 📌 Verification: `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_claim_tx -- --nocapture`

📌 **2. [Rule 3 - Blocking Issue] Removed one stale `ClaimStmtV2` field assumption during the wallet migration**

- 📌 Found during: Task 1 compile validation
- 📌 Issue: the first draft of `build_claim_stmt(...)` still tried to populate a non-existent `scenario_scope_hash` field and duplicated `chain_id`, which no longer matches the frozen `ClaimStmtV2` contract.
- 📌 Fix: aligned the wallet statement builder to the actual claim_v2 field shape and kept the scenario binding in the canonical digest inputs that still exist.
- 📌 Files modified: `crates/z00z_wallets/src/core/tx/claim_tx.rs`
- 📌 Verification: `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_claim_tx -- --nocapture`

📌 **3. [Rule 1 - Spec Drift] Removed the hidden wallet proof-builder shortcut so the live simulator path reaches the storage proof seam explicitly**

- 📌 Found during: post-implementation review against `025-04-PLAN.md`
- 📌 Issue: the simulator live path still reached storage-owned claim proofs through a gated wallet helper, which blurred the explicit trusted seam required by the plan.
- 📌 Fix: removed `build_claim_source(...)` from wallet core, moved simulator and support builders onto direct `AssetStore::claim_source_proof(...)` calls, and cleared the stale `ZERO_ROOT` constant from `stage_3.rs`.
- 📌 Files modified: `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/mod.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/claim_pkg.rs`, `crates/z00z_simulator/tests/test_claim_pkg_crypto_support.rs`, `crates/z00z_wallets/src/core/tx/test_claim_tx.rs`, `crates/z00z_wallets/tests/support/test_s5_sender_examples_support.rs`, `crates/z00z_simulator/src/scenario_1/stage_3.rs`
- 📌 Verification: `cargo test -q -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_zero_root_rejected -- --nocapture`, `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_pkg_crypto_support -- --nocapture`, `cargo test --release --features test-fast --features wallet_debug_dump`

---

📌 Total deviations: 3 auto-fixed issues

📌 Impact on plan: the fixes were required to make the storage-owned proof seam explicit in the live simulator path and to keep the frozen claim_v2 contract aligned with the real runtime boundary.

## Issues Encountered

- 📌 The old live claim path still contained placeholder proof and signature assumptions even after the typed claim_v2 contracts existed, so the migration required both builder-side and verifier-side cutover in one wave.
- 📌 The original migration left one hidden wallet helper around the storage proof seam; the final review pass removed it so the live simulator path now names the storage-owned proof boundary directly.
- 📌 Codacy reported only non-blocking pre-existing file-size or complexity warnings on large Rust files; no blocking semantic or security issues remained in the edited claim_v2 migration files.

## Verification Evidence

- 📌 `bash ./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_zero_root_rejected -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_claim_emit -- --nocapture`
- 📌 `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump test_claim_pkg_crypto_support -- --nocapture`
- 📌 `cargo test --release --features test-fast --features wallet_debug_dump`

## User Setup Required

📌 None.

## Next Phase Readiness

- 📌 The final facade cleanup in `025-05` can now treat `claim_v2` as the only production claim path because live wallet and simulator flows already agree on the typed source-proof and trusted-authority semantics.
- 📌 The remaining public-surface work is now mostly about export hygiene and documentation rather than about claim correctness.

## Known Stubs

📌 None.

## Self-Check

📌 PASSED - the summary file exists, the live wallet and simulator claim paths now use claim_v2 contracts, simulator proof generation reaches the storage seam directly, zero-root rejection is explicit in production verification, and the recorded targeted plus full release-style gates completed successfully in this execution slice.

---

📌 Phase: 025-crypto-audit-crypto

📌 Completed: 2026-03-27
