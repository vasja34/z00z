---
phase: 030
plan: 16
subsystem: z00z_wallets address
summary: Reduce remaining address-domain residue below the continuation band while preserving the shallow address facade.
tags:
  - phase-030
  - wallet-address
  - refactor
  - stealth
requirements:
  - PH30-SEAMS
  - PH30-FACADE
  - PH30-VERIFY
affects:
  - crates/z00z_wallets/src/core/address
provides:
  - Smaller address-domain roots below the >400 continuation band
  - Explicit seams for request/card/trust/snapshot/address transport and serde ownership
key_files:
  created:
    - crates/z00z_wallets/src/core/address/canonical_snapshot_tests.rs
    - crates/z00z_wallets/src/core/address/stealth_card_codec.rs
    - crates/z00z_wallets/src/core/address/stealth_request_crypto.rs
    - crates/z00z_wallets/src/core/address/stealth_request_transport.rs
    - crates/z00z_wallets/src/core/address/stealth_request_types.rs
    - crates/z00z_wallets/src/core/address/stealth_trust_tests.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address_serde.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address_transport.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address_serde.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address_transport.rs
  modified:
    - crates/z00z_wallets/src/core/address/canonical_snapshot.rs
    - crates/z00z_wallets/src/core/address/stealth_card.rs
    - crates/z00z_wallets/src/core/address/stealth_request.rs
    - crates/z00z_wallets/src/core/address/stealth_trust.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address.rs
decisions:
  - Keep core::address as the shallow caller contract while moving transport, serde, tests, and auxiliary types into sibling seams.
  - Keep canonical snapshot production logic in the root and move tests to an extracted include while satisfying clippy ordering rules.
  - Reduce stealth_request further by extracting declaration-heavy request types into a dedicated seam after transport and crypto helpers were already split.
metrics:
  duration: current-session
  completed_at: 2026-04-01
  tasks_completed: 2/2
---

# Phase 030 Plan 16: Address Residue Split Summary

Reduced the remaining oversized address-domain roots below the continuation band without changing the shallow `core::address` facade or caller-visible request and address behavior.

## Outcomes

- `stealth_request.rs` now delegates crypto helpers, transport helpers, and declaration-heavy request types into sibling seams and dropped from 592 lines to 277 lines.
- `canonical_snapshot.rs` keeps the production codec in the root and moved tests to `canonical_snapshot_tests.rs`, finishing at 324 lines.
- `stealth_card.rs`, `stealth_trust.rs`, `z00z_dual_address.rs`, and `z00z_single_address.rs` now hold only their primary production ownership while transport, serde, or tests live in dedicated files.
- Every Task 2 root is now below the `>400` continuation band:
  - `stealth_request.rs`: 277
  - `canonical_snapshot.rs`: 324
  - `stealth_card.rs`: 307
  - `stealth_trust.rs`: 247
  - `z00z_dual_address.rs`: 253
  - `z00z_single_address.rs`: 215

## Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --release --test test_addr_rate_limit_integration -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_deterministic_derivation_across_restarts -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_view_key_contract -- --nocapture`
- `cargo test --release --features test-fast --features wallet_debug_dump`
- `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- Domain ownership drift scan passed for `crates/z00z_wallets/src/core/address` with no reintroduced `WalletTag16Hash`, `WalletLeafAdHash`, `hash_zk::<`, or `hash_domain!` usage.
- Three YOLO review passes completed. Passes 2 and 3 were consecutive clean passes with no significant issues found.

## Deviations from Plan

### Auto-fixed Issues

1. `[Rule 3 - Blocking issue]` Fixed duplicated and malformed `canonical_snapshot.rs` content created during the initial test extraction, then restored the production codec and moved the test include to the end of the file to satisfy clippy.
2. `[Rule 3 - Blocking issue]` Corrected `include!` placement for `z00z_dual_address.rs` and `z00z_single_address.rs` so extracted transport `impl` blocks compile at module scope.
3. `[Rule 3 - Blocking issue]` Removed redundant `use super::*;` lines from include seams after verification exposed them as warnings.
4. `[Rule 3 - Blocking issue]` Fixed `cargo fmt --check` fallout and reran `full_verify --max-safe-run` to green.

## Known Stubs

None.

## Deferred Issues

None found within the scope of this plan.

## Self-Check: PASSED

- Summary file created at `.planning/phases/030-refactor-long-files/030-16-SUMMARY.md`
- All Task 2 roots verified below the continuation band
- Targeted wallet address regressions passed
- `full_verify.sh --max-safe-run` passed with `planned=313 skipped=21 failed=0`
