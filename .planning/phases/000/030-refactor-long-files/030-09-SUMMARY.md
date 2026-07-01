---
phase: 030-refactor-long-files
plan: 09
subsystem: wallets-tx-rpc
tags: [rust, wallets, tx, rpc, facade, seams, validation-closeout]
requires:
  - phase: 030-01
    provides: stable wallet persistence split pattern
  - phase: 030-03
    provides: protected-surface split discipline
  - phase: 030-05
    provides: wallet facade preservation pattern
  - phase: 030-06
    provides: include-based wallet seam extraction pattern
  - phase: 030-07
    provides: service facade split and review-closeout posture
provides:
  - stable tx-domain facade with extracted state, claim, digest, error, and spend seams
  - stable wallet RPC facade with extracted transport-side caches, rate limits, idempotency, broadcast, and persistence seams
  - preserved claim-state and tx verification behavior across the split
affects: [030-10, 030-11, z00z_wallets, z00z_simulator]
tech-stack:
  added: []
  patterns: [include-based stable facades, transport-only rpc splits, tx-domain seam extraction, bootstrap-first release validation]
key-files:
  created:
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_balance.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_caches.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_history.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_rate_limits.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_registry.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_stakes.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_broadcast.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_idempotency.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_rate_limits.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs
    - crates/z00z_wallets/src/core/tx/claim_auth.rs
    - crates/z00z_wallets/src/core/tx/claim_errors.rs
    - crates/z00z_wallets/src/core/tx/claim_helpers.rs
    - crates/z00z_wallets/src/core/tx/claim_wire_types.rs
    - crates/z00z_wallets/src/core/tx/spend_rules.rs
    - crates/z00z_wallets/src/core/tx/spend_verification.rs
    - crates/z00z_wallets/src/core/tx/state_checkpoint.rs
    - crates/z00z_wallets/src/core/tx/state_errors.rs
    - crates/z00z_wallets/src/core/tx/state_resolved_input.rs
    - crates/z00z_wallets/src/core/tx/state_traits.rs
    - crates/z00z_wallets/src/core/tx/state_witness.rs
    - crates/z00z_wallets/src/core/tx/tx_digest.rs
    - crates/z00z_wallets/src/core/tx/tx_errors.rs
    - crates/z00z_wallets/src/core/tx/tx_wire_types.rs
  modified:
    - crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/mod.rs
    - crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs
    - crates/z00z_wallets/src/core/tx/claim_tx.rs
    - crates/z00z_wallets/src/core/tx/mod.rs
    - crates/z00z_wallets/src/core/tx/spending.rs
    - crates/z00z_wallets/src/core/tx/state_update.rs
    - crates/z00z_wallets/src/core/tx/tx_verifier.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Keep `asset_impl.rs`, `tx_impl.rs`, and `crates/z00z_wallets/src/core/tx/mod.rs` as stable entrypoints while extracted seam files own homogeneous transport or tx-domain responsibilities.
  - Preserve existing verification order explicitly instead of letting the split imply new reject-class precedence.
  - Treat the known post-summary `full_verify` long-test collector hang as an out-of-scope verifier issue once the clean max-safe summary is reached.
patterns-established:
  - "Wallet tx split closeout: protect the facade, extract real semantic seams, then close on targeted anchors plus a clean max-safe summary."
  - "Transport split rule: caches, rate limits, idempotency, broadcast, and persistence may move into RPC-side helpers, but tx and asset ownership stay out of transport files."
requirements-completed: [PH30-SEAMS, PH30-FACADE, PH30-VERIFY]
completed: 2026-04-01
---

# Phase 030 Plan 09 Summary

📌 Wallet transaction core and wallet RPC transport were split into coherent seam files while the caller-visible tx and RPC facades stayed stable.

## Accomplishments

- 📌 Extracted transport-only helpers from `asset_impl.rs` and `tx_impl.rs` into dedicated cache, rate-limit, registry, history, idempotency, broadcast, and persistence modules without moving wallet-service or tx-domain ownership into the RPC layer.
- 📌 Extracted tx-domain seams for claim auth and wire types, tx digest and wire types, state witness and checkpoint structures, and spend-rule or spend-verification logic behind the stable `crates/z00z_wallets/src/core/tx/mod.rs` facade.
- 📌 Preserved current verification behavior through the split, including claim-state anchors, tx digest framing, fee and pass and poison gates, view-key contracts, and spent-gate coverage.
- 📌 Closed the wave on green bootstrap, named targeted wallet anchors, a broad release-style `z00z_wallets` suite, and a clean `full_verify` max-safe summary of `313 planned, 21 skipped, 0 failed`.

## Task Commits

📌 No git commit was created in this closeout. The repository remains dirty with unrelated planning changes, and the repo rule requires the owned Z00Z git-versioning workflow instead of ad hoc `git commit` usage.

## Files Created/Modified

- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs` and `tx_impl.rs` remained the stable transport entrypoints while sibling seam files took ownership of cache, rate-limit, registry, broadcast, idempotency, and persistence responsibilities.
- `crates/z00z_wallets/src/core/tx/mod.rs` remained the stable tx facade while claim, state, digest, error, and spending logic moved into sibling seam files.
- `crates/z00z_wallets/src/core/tx/claim_tx.rs` kept the live claim verifier entrypoint and claim behavior ordering while source contracts moved into `claim_auth.rs`, `claim_errors.rs`, `claim_helpers.rs`, and `claim_wire_types.rs`.
- `crates/z00z_wallets/src/core/tx/state_update.rs`, `spending.rs`, and `tx_verifier.rs` became thin owner modules over extracted state, spend, and verifier seams.
- `reports/full_verify-report-long-running-tests.txt` now records the clean max-safe summary for this closeout run.

## Decisions Made

- 📌 Keep the current tx facade singular; do not introduce a second claim or verifier entrypoint during the split.
- 📌 Keep `ZERO_ROOT` declared in `claim_tx.rs` because source-shape closure gates treat that file as the authoritative track map for the transitional claim verifier surface.
- 📌 Accept the post-summary `full_verify` hang as an existing verifier-tail issue once the script has already emitted a clean max-safe summary and the long-running report captures that summary.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed cfg-mismatched claim authority export after the claim split**

- **Found during:** targeted `test_claim_state_core` validation
- **Issue:** `claim_tx.rs` re-exported `sign_claim_auth` unconditionally even though `claim_auth.rs` only defines it behind `#[cfg(any(test, doctest, feature = "claim-auth-sign"))]`, which broke release compilation for the targeted anchor.
- **Fix:** Left `claim_auth_pk` exported unconditionally and gated `sign_claim_auth` in `claim_tx.rs` behind the same cfg as the implementation.
- **Files modified:** `crates/z00z_wallets/src/core/tx/claim_tx.rs`
- **Verification:** `cargo test -p z00z_wallets --release --test test_claim_state_core -- --nocapture`
- **Committed in:** not committed in this closeout

**2. [Rule 1 - Bug] Fixed formatting fallout across the newly extracted wallet tx and RPC seams**

- **Found during:** `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- **Issue:** the first clean build after the split failed in `cargo fmt --check` across the extracted RPC and tx-core seam files.
- **Fix:** Ran `rustfmt --edition 2021` across the touched wallet RPC and tx-core files and re-validated the touched directories with Codacy.
- **Files modified:** touched files under `crates/z00z_wallets/src/adapters/rpc/methods/` and `crates/z00z_wallets/src/core/tx/`
- **Verification:** reran the targeted closure test, then reran `full_verify` to a clean max-safe summary
- **Committed in:** not committed in this closeout

**3. [Rule 1 - Bug] Restored the closure-gate source contract for `ZERO_ROOT` in `claim_tx.rs`**

- **Found during:** the first rerun of `full_verify` after formatting cleanup
- **Issue:** `test_s5_closure_gate` failed because the split moved the `ZERO_ROOT` declaration into `claim_wire_types.rs`, while the closure gate intentionally tracks `claim_tx.rs` as the authoritative claim-source map.
- **Fix:** restored the local `ZERO_ROOT` declaration in `claim_tx.rs` and removed the now-unused duplicate from `claim_wire_types.rs`.
- **Files modified:** `crates/z00z_wallets/src/core/tx/claim_tx.rs`, `crates/z00z_wallets/src/core/tx/claim_wire_types.rs`
- **Verification:** `cargo test -p z00z_wallets --release --test test_s5_closure_gate -- --nocapture`
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 3 auto-fixed issues
📌 Impact on plan: All fixes stayed inside the tx or RPC seam split and validation-closeout work required to close `PH30-SEAMS`, `PH30-FACADE`, and `PH30-VERIFY`.

## Known Stubs

📌 None detected in the touched Plan 09 tx and RPC seams.

## User Setup Required

📌 None - no external services, credentials, or manual environment preparation were required for this plan.

## Next Phase Readiness

- 📌 Plan 10 can now normalize caller-visible wallet tx and RPC paths against smaller stable seam files instead of mixed-concern monoliths.
- 📌 The tx facade remains singular, so the upcoming normalization wave can change imports and docs without reopening verifier ownership.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_wallets --release --test test_claim_state_core -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_claim_tx -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_tx_digest_framing -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_tx_fee -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_tx_pass -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_tx_poison -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_tx_spent_gate -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_s5_closure_gate -- --nocapture`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` reached a clean summary of `313 planned, 21 skipped, 0 failed`; the script then hit the known out-of-scope post-summary long-test collector hang.

## Self-Check

📌 PASSED: `030-09-SUMMARY.md` exists, `ROADMAP.md` now shows `9/12 plans executed` with `030-09-PLAN.md` checked off, and `STATE.md` now points to Phase 030 Plan 10 as the next active slot.
