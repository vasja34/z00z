---
phase: 047-wallet-redesign
reviewed: 2026-05-21T00:00:00Z
depth: deep
files_reviewed: 20
files_reviewed_list:
  - crates/z00z_wallets/src/db/redb_wallet_store/session.rs
  - crates/z00z_wallets/src/db/redb_wallet_store/meta.rs
  - crates/z00z_wallets/src/db/redb_wallet_store/open/open_session.rs
  - crates/z00z_wallets/src/db/redb_wallet_store/open/mod.rs
  - crates/z00z_wallets/src/db/schema_keys.rs
  - crates/z00z_wallets/src/services/session_service.rs
  - crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs
  - crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs
  - crates/z00z_wallets/src/services/wallet_service_types_core.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs
  - crates/z00z_wallets/src/adapters/rpc/methods/key_impl/tests.rs
  - crates/z00z_wallets/src/adapters/rpc/types/key.rs
  - crates/z00z_wallets/src/wallet/entity/wallet_entity_key_api.rs
  - crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime_limits.rs
  - crates/z00z_wallets/src/services/wallet/session/wallet_service_session_guards.rs
  - crates/z00z_wallets/src/db/redb_wallet_store/crypto_ops/mod.rs
  - crates/z00z_wallets/src/db/redb_wallet_store/objects/mod.rs
  - crates/z00z_wallets/src/db/redb_wallet_store/queries.rs
  - crates/z00z_wallets/src/services/wallet_service_session.rs
findings:
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Phase 047: Code Review Report

## Narrative Findings (AI reviewer)

### CR-01: Crash-recovery reopen clears the durable rotation marker before proving the archived wallet can reopen cleanly

**File:** `crates/z00z_wallets/src/db/redb_wallet_store/open/open_session.rs`
**Function:** `open_wlt_with_deps`
**Issue:** When `rotation_in_progress` is present, the reopen path calls `finalize_rotation_marker_on_db(...)` first and only then runs `verify_archived_wallet_copy(...)`. That ordering clears `wallet.rotation_in_progress` on the live archive before the post-finalization archive has actually been re-opened and verified.
**Why this matters:** This violates the slice contract that the marker is cleared only after successful reopen verification. If `verify_archived_wallet_copy(...)` fails after the flush, the wallet file has already lost the only persisted signal that recovery was incomplete. Subsequent opens will treat the archive as fully finalized instead of an interrupted rotation, which is a real crash-safety regression.

### CR-02: Persisted rotation leaves all index tables bound to the old root

**File:** `crates/z00z_wallets/src/db/redb_wallet_store/session.rs`
**Function:** `WalletSession::rotate_master_key_persisted`
**Issue:** The durable rewrite only rewraps `SECRETS_TABLE` rows and `OBJECTS_TABLE` rows. It never rebuilds persisted index tables or the index manifest, even though index keys are generated from `derived_keys.index_key` in `crates/z00z_wallets/src/db/redb_wallet_store/objects/mod.rs::write_object_with_indexes` and exact lookups are matched against the current `derived_keys.index_key` in `crates/z00z_wallets/src/db/redb_wallet_store/queries.rs::index_row_matches_query`.
**Why this matters:** After a successful rotation, the wallet derives a new `index_key`, but all existing persisted index rows are still HMACed with the old one. That means label/status/receiver/asset/tx index lookups can silently stop matching immediately after rotation or restart, even though the open-path verification still passes because it validates objects but not indexes. This is a real post-rotation correctness break in persisted state.

### WR-01: The public wallet entity rotation API is still a success-shaped no-op

**File:** `crates/z00z_wallets/src/wallet/entity/wallet_entity_key_api.rs`
**Function:** `rotate_master_key`
**Issue:** The exported wallet entity method still returns a placeholder `RuntimeRotateKeyResponse` with empty fingerprint, `rotated_at = 0`, and `records_rewrapped = 0`, while doing no password check, confirmation check, audit, rate-limit enforcement, or persisted rewrite.
**Why this matters:** The RPC/service surface now advertises durable persisted rotation, but this public entity surface still exposes a callable no-op with a success-shaped receipt. Any in-process caller that uses the entity layer instead of the RPC adapter gets a false contract and bypasses the actual guarded rotation path.

---

_Reviewed: 2026-05-21T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
