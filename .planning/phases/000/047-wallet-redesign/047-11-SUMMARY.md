---
phase: 047-wallet-redesign
plan: 11
status: completed
updated: 2026-05-22
---

# ✅ 047-11 Summary

## ✅ Outcome

Phase 047 plan 11 now lands one evidence-only remote scan story end to end:

- `RemoteScanWorker` is a fetch-only seam that can return chunks, proof hints,
  and resume hints, but it cannot claim ownership or mutate wallet state.
- Worker-fed receive input is routed through the wallet-local authoritative
  `recv_range(...)` lane, with local scan evaluation remaining the only
  accepted-hit and cursor-write authority.
- Receive persistence now rejects stale expected-resume state and duplicate
  replay payload drift before cursor advancement.
- Adversarial coverage proves forged or desynced worker evidence fails closed,
  foreign chunks do not create claimed assets, and transport loss can fall back
  deterministically to the local-only lane.

## 🔧 Landed Changes

- Added the evidence-only worker contract surface in
  `crates/z00z_wallets/src/chain/scan_engine.rs` and kept
  `crates/z00z_wallets/src/chain/scan_engine_impl.rs` as a deferred fetch-only
  stub with explicit progress and callback behavior.
- Updated
  `crates/z00z_wallets/src/receiver/manager/receiver_manager_trait.rs` and
  `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs` to keep the
  ownership theorem explicit: remote inputs stay advisory, wallet-local scan
  stays authoritative.
- Reworked
  `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs`
  so worker evidence is validated up front, both receive entrypoints share the
  same authoritative core, and worker inputs cannot perform out-of-band cursor
  or asset mutations.
- Hardened
  `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` so
  `persist_scan_batch(...)` checks `expected_resume` and rejects duplicate
  stored payload-shape drift before writing the next scan cursor.
- Expanded
  `crates/z00z_wallets/src/services/wallet_service_tests.rs` with replay, gap,
  stale-cursor, empty-proof, missing-proof-hint, foreign-chunk, and
  transport-fallback coverage.

## 🧪 Validation

Executed and passed on the current tree:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump worker_ --lib`

Focused coverage now proves:

- replayed or desynced worker evidence does not advance cursor state;
- empty proof bytes and missing proof-hint checkpoint bindings fail closed;
- foreign chunks do not become claimed assets without wallet-local ownership
  detection;
- transport failure leaves state at origin and the same range can be completed
  deterministically via the local-only `recv_range(...)` lane.

Review loop status:

- three review passes completed for Task 3;
- pass 1 found missing transport-fallback coverage and landed the fix;
- passes 2 and 3 found no additional significant issues in the reviewed slice.

Diagnostics status:

- no errors were reported in the touched wallet test or receive/persistence
  files.

## ⚠️ Remaining Workspace Gate

The required full release command was started on the current tree:

- `cargo test --release --features test-fast --features wallet_debug_dump`

Observed broad-run status before manual stop:

- the run progressed cleanly through late wallet, claim, and simulator-adjacent
  suites, and no Task 11-local failure surfaced in the observed tail.

The known workspace-wide blocker was also rerun directly on the current tree and
still failed exactly as before:

- `cargo test -p z00z_simulator --test test_scenario1_stage_surface test_boundary_wording_stays_narrow --release --features test-fast --features wallet_debug_dump`

Failure footer:

- panic text: `public spend verifier wording must keep the boundary narrow while reflecting shipped nullifier closure`
- location: `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs:1077`

This blocker should be treated as external to 047-11 rather than a regression
from the wallet remote-scan worker changes.
