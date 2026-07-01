---
phase: 047-wallet-redesign
plan: 10
status: completed
updated: 2026-05-21
---

# ✅ 047-10 Summary

## ✅ Outcome

Phase 047 plan 10 now lands one honest durable master-key rotation story:

- `wallet.key.rotate_master_key` no longer reports success on an in-memory
  rederive-only path;
- persisted rotation rewrites sealed wallet state under a new rotated root with
  explicit `rotation_in_progress` recovery semantics;
- archive-copy verification proves both marker-present and marker-cleared reopen
  behavior before the real marker is cleared;
- rollback restores secret rows, object rows, index manifest rows, and index
  table rows if rotation fails mid-flight;
- successful rotation revokes the live bearer session and returns a durable
  rewrite receipt instead of the old `keys_rederived` placeholder story.

## 🔧 Landed Changes

- Reworked `crates/z00z_wallets/src/db/redb_wallet_store/session.rs` so
  rotation snapshots capture secrets, objects, index manifests, and concrete
  index rows; the durable rewrite now verifies live DB state plus reopenable
  `.wlt` archive copies before clearing the rotation marker.
- Updated `crates/z00z_wallets/src/db/redb_wallet_store/open/open_session.rs`
  to share the crash-recovery reopen discipline: when
  `rotation_in_progress` exists, open-time recovery proves the marker-cleared
  archive-copy state before finalizing the real wallet.
- Added `write_object_with_index_key(...)` in
  `crates/z00z_wallets/src/db/redb_wallet_store/objects/mod.rs` and exposed
  payload-derived owned-asset index updates from
  `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` so rotated
  objects rebuild their secondary indexes under the new index key instead of
  reusing stale MAC-derived rows.
- Updated `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` to document the
  shipped rotate rate-limit truth: one successful confirmed rotation per wallet
  per hour, while password or confirmation failures do not consume the slot.

## 🧪 Validation

Executed and passed:

- `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump test_rotate_master_ --lib`
- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`

Focused coverage now proves:

- successful durable rotation receipt and session revocation;
- reopen after rotation under the rotated root;
- failpoint-triggered rollback and restart-safe recovery;
- stale-session rejection and rate-limit rollback semantics;
- archive-copy verification with marker-present and marker-cleared preflight
  states.

Review loop status:

- two consecutive reviewer reruns returned `No significant findings in reviewed slice.`

## ⚠️ Remaining Workspace Gate

The required full release command was rerun:

- `cargo test --release --features test-fast --features wallet_debug_dump`

The rerun confirmed the same workspace-wide blocker outside this slice:

- `crates/z00z_simulator/tests/test_scenario1_stage_surface.rs::test_boundary_wording_stays_narrow`

Failure footer:

- panic text: `public spend verifier wording must keep the boundary narrow while reflecting shipped nullifier closure`
- exit code: `101`

This failure should be treated as external to 047-10 rather than a regression
from the wallet durable-rotation changes.
