---
goal: Wallet source tree rename and flattening plan
date_created: 2026-06-21
last_updated: 2026-06-24
status: 'Complete'
scope: crates/z00z_wallets/src
---

# Wallet Rust Source Renaming Plan

🎯 **Goal:** Flatten `crates/z00z_wallets/src` so Rust files live only at `src/*.rs` or `src/<domain>/*.rs`, while keeping file names semantically accurate and avoiding broad cosmetic renames.

📌 **Scope:** This is a plan only. No Rust source file is moved or edited by this document. Files not listed in the decision table keep their current paths.

> Live execution note: future or target design wording in this TODO and the
> referenced design corpus is live mandatory Phase 061 scope for the current
> tree. Keep one canonical module or function path per behavior.

> z00z_wallets/src/wallet_config.yaml
> z00z_wallets/🔐-разбор-WLT.md
> z00z_wallets/🔐-разбор-кошелька-Z00Z.md
> z00z_wallets/tests/test_common

## ✅ Constraints

- **CON-001:** `src/` may contain only one directory level: `src/domain/file.rs` is allowed; `src/domain/sub/file.rs` is not allowed.
- **CON-002:** Functional grouping must use file prefixes, for example `bip32_constants.rs` and `bip32_path_builder.rs`.
- **CON-003:** Rename only when flattening requires it, when a name loses context after flattening, or when the current name is demonstrably misleading.
- **CON-004:** Keep module identifiers in `snake_case` and at five words or fewer.
- **CON-005:** Preserve caller-visible facades during implementation, especially `adapters::rpc`, `db::redb_wallet_store`, `core::*`, and `services::WalletService`.
- **CON-006:** Do not expose new crate-root public modules only to preserve old paths; use `#[path = ...]` facade declarations when a moved implementation must remain reachable through an existing public namespace.
- **CON-007:** Test-only Rust files must keep the `test_` prefix.
- **CON-008:** Keep `src/redb_store/` limited to the concrete native RedB wallet-store backend; shared persistence crypto and schema contracts stay under `src/db/` with neutral wallet-store naming.
- **CON-009:** Module/file renames must not change persisted `.wlt` domain labels, schema versions, KDF labels, or hash-domain strings such as `z00z.crypto.redb_wallet_crypto...`; those require a separate migration decision.
- **CON-010:** Avoid repeating a moved directory's domain word inside the filename unless it prevents a real target conflict or preserves a distinct concept.

## ⚖️ Decision Basis

| Code | Decision | Pros | Cons | Outcome |
|---|---|---|---|---|
| F1 | Flatten inside the existing domain directory. | Minimal public-surface churn; keeps domain ownership obvious. | Adds prefixes where subdirectory context disappears. | Use for normal domain subtrees. |
| F2 | Move a deep subsystem to a new top-level one-level directory. | Prevents overlong module names; keeps files readable. | Requires explicit path-based facade wiring. | Use `#[path = ...]` from the old facade module for `rpc` and `redb_store`. |
| R1 | Correct an actual spelling/semantic mismatch. | Aligns names with visible registry/domain terms. | Requires reference updates. | Use only for confirmed mismatch. |
| R2 | Shorten a module name that violates the five-word limit. | Restores repository naming compliance. | Requires include/module declaration updates. | Use for the two six-word service files. |
| R3 | Replace generic or unclear names. | Makes responsibility clear after losing subdirectory context. | Small reference churn. | Use for `nfc_utils`, `claim_own`, and the claim tx helper split. |
| R4 | Remove misleading RedB prefix from shared persistence crypto. | Makes shared WASM/native crypto ownership clear; keeps `redb_store` backend-only. | Requires internal references to move from `db::redb_wallet_crypto` to `db::wallet_store_crypto`. | Rename files/module to `wallet_store_crypto*`, not `redb_store/*`. |
| R5 | Remove redundant RPC qualifier after moving files into `src/rpc`. | Avoids names like `rpc/method_tx_rpc_*`; keeps method grouping clear. | Requires module reference updates in RPC method implementations. | Drop `_rpc_` where `src/rpc` already supplies the context; use `method_tx_support` for the helper file that would otherwise collide with `method_tx_impl`. |
| D1 | Remove unused empty placeholder. | Avoids preserving meaningless numeric filenames. | Requires confirming no runtime reference. | Use for `app_settings_tab_2.rs`. |
| D2 | Remove duplicate storage files. | Removes duplicate source of truth. | Requires confirming module exports already use canonical files. | Use for receipt/scan duplicate `storage*.rs`. |
| D3 | Close the stale aggregate include lane as a preflight drift note. | Prevents later slices from reviving a non-existent compatibility artifact. | Requires confirming no module declaration still depends on the historical lane. | Use for the already-absent `wallet_service_types.rs` aggregate row during `061-01`. |
| D4 | Keep the live one-level WalletService seam files after shard flattening. | Preserves named internal module boundaries and source-anchor tests without reviving nested trees. | Retains thin `#[path]`/`include!` seam files under `src/services/*.rs`. | Use for the canonical `src/services/{wallet_service_actions,wallet_service_session,wallet_service_store,wallet_service_types_*,test_wallet_service}.rs` seam files after nested shard retirement. |

## 📋 Rename Decision Table

| # | old-path | new-path | Decision | Rationale |
|---:|---|---|---|---|
| 1 | `crates/z00z_wallets/src/adapters/rpc/app_dispatcher_wiring.rs` | `crates/z00z_wallets/src/rpc/app_dispatcher_wiring.rs` | F2 | Move RPC implementation to one-level src/rpc with adapters facade re-export. |
| 2 | `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs` | `crates/z00z_wallets/src/rpc/dispatcher_handlers.rs` | F2 | Move RPC implementation to one-level src/rpc with adapters facade re-export. |
| 3 | `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs` | `crates/z00z_wallets/src/rpc/error_mapping.rs` | F2 | Move RPC implementation to one-level src/rpc with adapters facade re-export. |
| 4 | `crates/z00z_wallets/src/adapters/rpc/logging/config.rs` | `crates/z00z_wallets/src/rpc/logging_config.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 5 | `crates/z00z_wallets/src/adapters/rpc/logging/middleware/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_logging_middleware.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 6 | `crates/z00z_wallets/src/adapters/rpc/logging/middleware.rs` | `crates/z00z_wallets/src/rpc/logging_middleware.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 7 | `crates/z00z_wallets/src/adapters/rpc/logging/mod.rs` | `crates/z00z_wallets/src/rpc/logging.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 8 | `crates/z00z_wallets/src/adapters/rpc/logging/policy.rs` | `crates/z00z_wallets/src/rpc/logging_policy.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 9 | `crates/z00z_wallets/src/adapters/rpc/logging/record.rs` | `crates/z00z_wallets/src/rpc/logging_record.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 10 | `crates/z00z_wallets/src/adapters/rpc/logging/summarize/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_logging_summarize.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 11 | `crates/z00z_wallets/src/adapters/rpc/logging/summarize.rs` | `crates/z00z_wallets/src/rpc/logging_summarize.rs` | F2 | Flatten RPC logging subtree while preserving logging prefix. |
| 12 | `crates/z00z_wallets/src/adapters/rpc/methods/app.rs` | `crates/z00z_wallets/src/rpc/method_app.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 13 | `crates/z00z_wallets/src/adapters/rpc/methods/app_impl.rs` | `crates/z00z_wallets/src/rpc/method_app_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 14 | `crates/z00z_wallets/src/adapters/rpc/methods/asset.rs` | `crates/z00z_wallets/src/rpc/method_asset.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 15 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_server.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_server.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 16 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_support_assets.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_support_assets.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 17 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_support_claims.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_support_claims.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 18 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/asset_impl_support_state.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_support_state.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 19 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl/test_asset_impl.rs` | `crates/z00z_wallets/src/rpc/test_asset_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 20 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 21 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_catalog.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_server_catalog.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 22 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_ops.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_server_ops.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 23 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs` | `crates/z00z_wallets/src/rpc/method_asset_impl_server_transfer.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 24 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_balance.rs` | `crates/z00z_wallets/src/rpc/method_asset_balance.rs` | R5 | Drop redundant RPC qualifier after moving asset method helpers under `src/rpc`. |
| 25 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_caches.rs` | `crates/z00z_wallets/src/rpc/method_asset_caches.rs` | R5 | Drop redundant RPC qualifier after moving asset method helpers under `src/rpc`. |
| 26 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_history.rs` | `crates/z00z_wallets/src/rpc/method_asset_history.rs` | R5 | Drop redundant RPC qualifier after moving asset method helpers under `src/rpc`. |
| 27 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_rate_limits.rs` | `crates/z00z_wallets/src/rpc/method_asset_rate_limits.rs` | R5 | Drop redundant RPC qualifier after moving asset method helpers under `src/rpc`. |
| 28 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_registry.rs` | `crates/z00z_wallets/src/rpc/method_asset_registry.rs` | R5 | Drop redundant RPC qualifier after moving asset method helpers under `src/rpc`. |
| 29 | `crates/z00z_wallets/src/adapters/rpc/methods/asset_rpc_stakes.rs` | `crates/z00z_wallets/src/rpc/method_asset_stakes.rs` | R5 | Drop redundant RPC qualifier after moving asset method helpers under `src/rpc`. |
| 30 | `crates/z00z_wallets/src/adapters/rpc/methods/backup.rs` | `crates/z00z_wallets/src/rpc/method_backup.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 31 | `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_backup_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 32 | `crates/z00z_wallets/src/adapters/rpc/methods/backup_impl.rs` | `crates/z00z_wallets/src/rpc/method_backup_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 33 | `crates/z00z_wallets/src/adapters/rpc/methods/chain.rs` | `crates/z00z_wallets/src/rpc/method_chain.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 34 | `crates/z00z_wallets/src/adapters/rpc/methods/chain_impl.rs` | `crates/z00z_wallets/src/rpc/method_chain_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 35 | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs` | `crates/z00z_wallets/src/rpc/method_key.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 36 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs` | `crates/z00z_wallets/src/rpc/method_key_impl_server.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 37 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs` | `crates/z00z_wallets/src/rpc/method_key_impl_server_admin.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 38 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_derive.rs` | `crates/z00z_wallets/src/rpc/method_key_impl_server_derive.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 39 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_requests.rs` | `crates/z00z_wallets/src/rpc/method_key_impl_server_requests.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 40 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs` | `crates/z00z_wallets/src/rpc/method_key_impl_support.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 41 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_key_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 42 | `crates/z00z_wallets/src/adapters/rpc/methods/key_impl.rs` | `crates/z00z_wallets/src/rpc/method_key_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 43 | `crates/z00z_wallets/src/adapters/rpc/methods/mod.rs` | `crates/z00z_wallets/src/rpc/methods.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 44 | `crates/z00z_wallets/src/adapters/rpc/methods/network.rs` | `crates/z00z_wallets/src/rpc/method_network.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 45 | `crates/z00z_wallets/src/adapters/rpc/methods/network_impl.rs` | `crates/z00z_wallets/src/rpc/method_network_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 46 | `crates/z00z_wallets/src/adapters/rpc/methods/object.rs` | `crates/z00z_wallets/src/rpc/method_object.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 47 | `crates/z00z_wallets/src/adapters/rpc/methods/object_impl.rs` | `crates/z00z_wallets/src/rpc/method_object_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 48 | `crates/z00z_wallets/src/adapters/rpc/methods/ownership_check.rs` | `crates/z00z_wallets/src/rpc/method_ownership_check.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 49 | `crates/z00z_wallets/src/adapters/rpc/methods/storage.rs` | `crates/z00z_wallets/src/rpc/method_storage.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 50 | `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_storage_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 51 | `crates/z00z_wallets/src/adapters/rpc/methods/storage_impl.rs` | `crates/z00z_wallets/src/rpc/method_storage_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 52 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_broadcast_body.rs` | `crates/z00z_wallets/src/rpc/test_tx_broadcast_body.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 53 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_body.rs` | `crates/z00z_wallets/src/rpc/test_tx_history_body.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 54 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_cursor_filters.rs` | `crates/z00z_wallets/src/rpc/test_tx_history_cursor_filters.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 55 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_history_receipt_sort.rs` | `crates/z00z_wallets/src/rpc/test_tx_history_receipt_sort.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 56 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_pending_body.rs` | `crates/z00z_wallets/src/rpc/test_tx_pending_body.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 57 | `crates/z00z_wallets/src/adapters/rpc/methods/test_tx_send_body.rs` | `crates/z00z_wallets/src/rpc/test_tx_send_body.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 58 | `crates/z00z_wallets/src/adapters/rpc/methods/tx.rs` | `crates/z00z_wallets/src/rpc/method_tx.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 59 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_tx_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 60 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tests/test_tx_impl_body.rs` | `crates/z00z_wallets/src/rpc/test_tx_impl_body.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 61 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl/tx_impl_server.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl_server.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 62 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 63 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_finalize.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl_server_finalize.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 64 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_helpers.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl_server_helpers.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 65 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_history.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl_server_history.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 66 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl_server_lifecycle.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 67 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_send.rs` | `crates/z00z_wallets/src/rpc/method_tx_impl_server_send.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 68 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_admission.rs` | `crates/z00z_wallets/src/rpc/method_tx_admission.rs` | R5 | Drop redundant RPC qualifier after moving tx method helpers under `src/rpc`. |
| 69 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_broadcast.rs` | `crates/z00z_wallets/src/rpc/method_tx_broadcast.rs` | R5 | Drop redundant RPC qualifier after moving tx method helpers under `src/rpc`. |
| 70 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_idempotency.rs` | `crates/z00z_wallets/src/rpc/method_tx_idempotency.rs` | R5 | Drop redundant RPC qualifier after moving tx method helpers under `src/rpc`. |
| 71 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_impl.rs` | `crates/z00z_wallets/src/rpc/method_tx_support.rs` | R5 | Rename shared tx helper module without redundant RPC qualifier and avoid collision with `method_tx_impl.rs`. |
| 72 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_rate_limits.rs` | `crates/z00z_wallets/src/rpc/method_tx_rate_limits.rs` | R5 | Drop redundant RPC qualifier after moving tx method helpers under `src/rpc`. |
| 73 | `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs` | `crates/z00z_wallets/src/rpc/method_tx_storage.rs` | R5 | Drop redundant RPC qualifier after moving tx method helpers under `src/rpc`. |
| 74 | `crates/z00z_wallets/src/adapters/rpc/methods/wallet.rs` | `crates/z00z_wallets/src/rpc/method_wallet.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 75 | `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_wallet_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 76 | `crates/z00z_wallets/src/adapters/rpc/methods/wallet_impl.rs` | `crates/z00z_wallets/src/rpc/method_wallet_impl.rs` | F2 | Flatten RPC methods subtree using method/test prefixes. |
| 77 | `crates/z00z_wallets/src/adapters/rpc/mod.rs` | `crates/z00z_wallets/src/rpc/mod.rs` | F2 | Move RPC implementation to one-level src/rpc with adapters facade re-export. |
| 78 | `crates/z00z_wallets/src/adapters/rpc/types/app.rs` | `crates/z00z_wallets/src/rpc/types_app.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 79 | `crates/z00z_wallets/src/adapters/rpc/types/asset.rs` | `crates/z00z_wallets/src/rpc/types_asset.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 80 | `crates/z00z_wallets/src/adapters/rpc/types/backup.rs` | `crates/z00z_wallets/src/rpc/types_backup.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 81 | `crates/z00z_wallets/src/adapters/rpc/types/chain.rs` | `crates/z00z_wallets/src/rpc/types_chain.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 82 | `crates/z00z_wallets/src/adapters/rpc/types/common/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_types_common.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 83 | `crates/z00z_wallets/src/adapters/rpc/types/common.rs` | `crates/z00z_wallets/src/rpc/types_common.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 84 | `crates/z00z_wallets/src/adapters/rpc/types/events/events_impl.rs` | `crates/z00z_wallets/src/rpc/types_events_impl.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 85 | `crates/z00z_wallets/src/adapters/rpc/types/events/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_types_events.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 86 | `crates/z00z_wallets/src/adapters/rpc/types/events.rs` | `crates/z00z_wallets/src/rpc/types_events.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 87 | `crates/z00z_wallets/src/adapters/rpc/types/key/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_types_key.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 88 | `crates/z00z_wallets/src/adapters/rpc/types/key.rs` | `crates/z00z_wallets/src/rpc/types_key.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 89 | `crates/z00z_wallets/src/adapters/rpc/types/mod.rs` | `crates/z00z_wallets/src/rpc/types.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 90 | `crates/z00z_wallets/src/adapters/rpc/types/network.rs` | `crates/z00z_wallets/src/rpc/types_network.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 91 | `crates/z00z_wallets/src/adapters/rpc/types/object.rs` | `crates/z00z_wallets/src/rpc/types_object.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 92 | `crates/z00z_wallets/src/adapters/rpc/types/security/test_mod.rs` | `crates/z00z_wallets/src/rpc/test_types_security.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 93 | `crates/z00z_wallets/src/adapters/rpc/types/security.rs` | `crates/z00z_wallets/src/rpc/types_security.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 94 | `crates/z00z_wallets/src/adapters/rpc/types/storage.rs` | `crates/z00z_wallets/src/rpc/types_storage.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 95 | `crates/z00z_wallets/src/adapters/rpc/types/tx.rs` | `crates/z00z_wallets/src/rpc/types_tx.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 96 | `crates/z00z_wallets/src/adapters/rpc/types/wallet.rs` | `crates/z00z_wallets/src/rpc/types_wallet.rs` | F2 | Flatten RPC DTO subtree using types prefix. |
| 97 | `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring/wallet_dispatcher_wiring_register.rs` | `crates/z00z_wallets/src/rpc/wallet_dispatcher_wiring_register.rs` | F2 | Flatten dispatcher registration helper under src/rpc. |
| 98 | `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs` | `crates/z00z_wallets/src/rpc/wallet_dispatcher_wiring.rs` | F2 | Move RPC implementation to one-level src/rpc with adapters facade re-export. |
| 99 | `crates/z00z_wallets/src/backup/backup_importer_impl/mod.rs` | `crates/z00z_wallets/src/backup/backup_importer_impl.rs` | F1 | Flatten backup importer implementation subtree. |
| 100 | `crates/z00z_wallets/src/backup/backup_importer_impl/test_mod.rs` | `crates/z00z_wallets/src/backup/test_backup_importer_impl.rs` | F1 | Flatten backup importer implementation subtree. |
| 101 | `crates/z00z_wallets/src/backup/crypto/wallet_backup_kdf.rs` | `crates/z00z_wallets/src/backup/wallet_backup_kdf.rs` | F1 | Flatten backup crypto fragment under backup prefix. |
| 102 | `crates/z00z_wallets/src/backup/export/backup_exporter_verify.rs` | `crates/z00z_wallets/src/backup/backup_exporter_verify.rs` | F1 | Flatten backup export helpers and tests. |
| 103 | `crates/z00z_wallets/src/backup/export/test_backup_exporter_suite.rs` | `crates/z00z_wallets/src/backup/test_backup_exporter_suite.rs` | F1 | Flatten backup export helpers and tests. |
| 104 | `crates/z00z_wallets/src/backup/wallet_backup/mod.rs` | `crates/z00z_wallets/src/backup/wallet_backup.rs` | F1 | Flatten wallet backup crypto facade. |
| 105 | `crates/z00z_wallets/src/backup/wallet_backup/test_mod.rs` | `crates/z00z_wallets/src/backup/test_wallet_backup.rs` | F1 | Flatten wallet backup crypto facade. |
| 106 | `crates/z00z_wallets/src/chain/broadcast/broadcast_impl.rs` | `crates/z00z_wallets/src/chain/broadcast_impl.rs` | F1 | Flatten chain submodule. |
| 107 | `crates/z00z_wallets/src/claim/nullifier_store/test_mod.rs` | `crates/z00z_wallets/src/claim/test_nullifier_store.rs` | F1 | Flatten claim submodule. |
| 108 | `crates/z00z_wallets/src/claim/registry/claim_registry.rs` | `crates/z00z_wallets/src/claim/claim_registry.rs` | F1 | Flatten claim submodule. |
| 109 | `crates/z00z_wallets/src/db/codecs/index_codecs_body.rs` | `crates/z00z_wallets/src/db/index_codecs_body.rs` | F1 | Flatten db codec fragment. |
| 110 | `crates/z00z_wallets/src/db/index_codecs/mod.rs` | `crates/z00z_wallets/src/db/index_codecs.rs` | F1 | Flatten db index codec subtree. |
| 111 | `crates/z00z_wallets/src/db/index_codecs/test_mod.rs` | `crates/z00z_wallets/src/db/test_index_codecs.rs` | F1 | Flatten db index codec subtree. |
| 112 | `crates/z00z_wallets/src/db/index_codecs/tx_time.rs` | `crates/z00z_wallets/src/db/index_codecs_tx_time.rs` | F1 | Flatten db index codec subtree. |
| 113 | `crates/z00z_wallets/src/db/redb_wallet_crypto/aad_ops.rs` | `crates/z00z_wallets/src/db/wallet_store_crypto_aad.rs` | R4 | Rename shared wallet persistence crypto AAD helpers away from the misleading RedB backend prefix. |
| 114 | `crates/z00z_wallets/src/db/redb_wallet_crypto/kdf_helpers.rs` | `crates/z00z_wallets/src/db/wallet_store_crypto_kdf.rs` | R4 | Rename shared wallet persistence crypto KDF helpers away from the misleading RedB backend prefix. |
| 115 | `crates/z00z_wallets/src/db/redb_wallet_crypto/mod.rs` | `crates/z00z_wallets/src/db/wallet_store_crypto.rs` | R4 | Rename shared wallet persistence crypto facade away from the misleading RedB backend prefix. |
| 116 | `crates/z00z_wallets/src/db/redb_wallet_crypto/models.rs` | `crates/z00z_wallets/src/db/wallet_store_crypto_models.rs` | R4 | Rename shared wallet persistence crypto models away from the misleading RedB backend prefix. |
| 117 | `crates/z00z_wallets/src/db/redb_wallet_crypto/test_mod.rs` | `crates/z00z_wallets/src/db/test_wallet_store_crypto.rs` | R4 | Rename shared wallet persistence crypto tests away from the misleading RedB backend prefix. |
| 118 | `crates/z00z_wallets/src/db/redb_wallet_store/codecs.rs` | `crates/z00z_wallets/src/redb_store/codecs.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 119 | `crates/z00z_wallets/src/db/redb_wallet_store/crypto_ops/mod.rs` | `crates/z00z_wallets/src/redb_store/crypto_ops.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 120 | `crates/z00z_wallets/src/db/redb_wallet_store/crypto_ops/seed_reveal.rs` | `crates/z00z_wallets/src/redb_store/crypto_seed_reveal.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 121 | `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_export.rs` | `crates/z00z_wallets/src/redb_store/debug_export.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 122 | `crates/z00z_wallets/src/db/redb_wallet_store/debug/debug_types.rs` | `crates/z00z_wallets/src/redb_store/debug_types.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 123 | `crates/z00z_wallets/src/db/redb_wallet_store/debug/mod.rs` | `crates/z00z_wallets/src/redb_store/debug.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 124 | `crates/z00z_wallets/src/db/redb_wallet_store/meta.rs` | `crates/z00z_wallets/src/redb_store/meta.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 125 | `crates/z00z_wallets/src/db/redb_wallet_store/migrations.rs` | `crates/z00z_wallets/src/redb_store/migrations.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 126 | `crates/z00z_wallets/src/db/redb_wallet_store/mod.rs` | `crates/z00z_wallets/src/redb_store/mod.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 127 | `crates/z00z_wallets/src/db/redb_wallet_store/mutations/create.rs` | `crates/z00z_wallets/src/redb_store/mutations_create.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 128 | `crates/z00z_wallets/src/db/redb_wallet_store/mutations/initial_objects.rs` | `crates/z00z_wallets/src/redb_store/mutations_initial_objects.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 129 | `crates/z00z_wallets/src/db/redb_wallet_store/mutations/mod.rs` | `crates/z00z_wallets/src/redb_store/mutations.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 130 | `crates/z00z_wallets/src/db/redb_wallet_store/mutations/upserts.rs` | `crates/z00z_wallets/src/redb_store/mutations_upserts.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 131 | `crates/z00z_wallets/src/db/redb_wallet_store/objects/mod.rs` | `crates/z00z_wallets/src/redb_store/objects.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 132 | `crates/z00z_wallets/src/db/redb_wallet_store/open/discovery.rs` | `crates/z00z_wallets/src/redb_store/open_discovery.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 133 | `crates/z00z_wallets/src/db/redb_wallet_store/open/mod.rs` | `crates/z00z_wallets/src/redb_store/open.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 134 | `crates/z00z_wallets/src/db/redb_wallet_store/open/open_session.rs` | `crates/z00z_wallets/src/redb_store/open_session.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 135 | `crates/z00z_wallets/src/db/redb_wallet_store/owned_assets.rs` | `crates/z00z_wallets/src/redb_store/owned_assets.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 136 | `crates/z00z_wallets/src/db/redb_wallet_store/owned_objects.rs` | `crates/z00z_wallets/src/redb_store/owned_objects.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 137 | `crates/z00z_wallets/src/db/redb_wallet_store/profile.rs` | `crates/z00z_wallets/src/redb_store/profile.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 138 | `crates/z00z_wallets/src/db/redb_wallet_store/queries.rs` | `crates/z00z_wallets/src/redb_store/queries.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 139 | `crates/z00z_wallets/src/db/redb_wallet_store/session.rs` | `crates/z00z_wallets/src/redb_store/session.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 140 | `crates/z00z_wallets/src/db/redb_wallet_store/tables.rs` | `crates/z00z_wallets/src/redb_store/tables.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 141 | `crates/z00z_wallets/src/db/redb_wallet_store/test_mod.rs` | `crates/z00z_wallets/src/redb_store/test_redb_store.rs` | F2 | Move RedB wallet-store backend to one-level src/redb_store to avoid overlong module names. |
| 142 | `crates/z00z_wallets/src/db/storage_backend/mod.rs` | `crates/z00z_wallets/src/db/storage_backend.rs` | F1 | Flatten storage backend subtree. |
| 143 | `crates/z00z_wallets/src/db/storage_backend/test_mod.rs` | `crates/z00z_wallets/src/db/test_storage_backend.rs` | F1 | Flatten storage backend subtree. |
| 144 | `crates/z00z_wallets/src/domains/definitions/test_mod.rs` | `crates/z00z_wallets/src/domains/test_definitions.rs` | F1 | Flatten domains test subtree. |
| 145 | `crates/z00z_wallets/src/domains/hashing/test_mod.rs` | `crates/z00z_wallets/src/domains/test_hashing.rs` | F1 | Flatten domains test subtree. |
| 146 | `crates/z00z_wallets/src/egui_views/app_settings_tab_2.rs` | `N/A - remove duplicate or stale Rust file during implementation` | D1 | Unused empty duplicate of app_settings_tab.rs; numeric suffix has no semantic role. |
| 147 | `crates/z00z_wallets/src/egui_views/wallet_tab_stacking.rs` | `crates/z00z_wallets/src/egui_views/wallet_tab_staking.rs` | R1 | Fix spelling mismatch with wallet_staking tab id and Staking label. |
| 148 | `crates/z00z_wallets/src/key/bip/bip32.rs` | `crates/z00z_wallets/src/key/bip32.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 149 | `crates/z00z_wallets/src/key/bip/bip32_constants.rs` | `crates/z00z_wallets/src/key/bip32_constants.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 150 | `crates/z00z_wallets/src/key/bip/bip32_key_deriver.rs` | `crates/z00z_wallets/src/key/bip32_key_deriver.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 151 | `crates/z00z_wallets/src/key/bip/bip32_manager.rs` | `crates/z00z_wallets/src/key/bip32_manager.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 152 | `crates/z00z_wallets/src/key/bip/bip32_path.rs` | `crates/z00z_wallets/src/key/bip32_path.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 153 | `crates/z00z_wallets/src/key/bip/bip32_path_builder.rs` | `crates/z00z_wallets/src/key/bip32_path_builder.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 154 | `crates/z00z_wallets/src/key/bip/bip32_path_builder_helpers.rs` | `crates/z00z_wallets/src/key/bip32_path_builder_helpers.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 155 | `crates/z00z_wallets/src/key/bip/bip32_path_errors.rs` | `crates/z00z_wallets/src/key/bip32_path_errors.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 156 | `crates/z00z_wallets/src/key/bip/bip32_path_serde.rs` | `crates/z00z_wallets/src/key/bip32_path_serde.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 157 | `crates/z00z_wallets/src/key/bip/bip32_path_validator.rs` | `crates/z00z_wallets/src/key/bip32_path_validator.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 158 | `crates/z00z_wallets/src/key/bip/bip32_path_value.rs` | `crates/z00z_wallets/src/key/bip32_path_value.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 159 | `crates/z00z_wallets/src/key/bip/bip32_ristretto_bridge.rs` | `crates/z00z_wallets/src/key/bip32_ristretto_bridge.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 160 | `crates/z00z_wallets/src/key/bip/mod.rs` | `crates/z00z_wallets/src/key/bip.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 161 | `crates/z00z_wallets/src/key/bip/test_bip32_manager.inc.rs` | `crates/z00z_wallets/src/key/test_bip32_manager.inc.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 162 | `crates/z00z_wallets/src/key/bip/test_bip32_manager_entropy.inc.rs` | `crates/z00z_wallets/src/key/test_bip32_manager_entropy.inc.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 163 | `crates/z00z_wallets/src/key/manager/key_cache.rs` | `crates/z00z_wallets/src/key/key_cache.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 164 | `crates/z00z_wallets/src/key/manager/key_manager.rs` | `crates/z00z_wallets/src/key/key_manager.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 165 | `crates/z00z_wallets/src/key/manager/key_manager_impl.rs` | `crates/z00z_wallets/src/key/key_manager_impl.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 166 | `crates/z00z_wallets/src/key/manager/key_manager_impl_cache.rs` | `crates/z00z_wallets/src/key/key_manager_impl_cache.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 167 | `crates/z00z_wallets/src/key/manager/key_manager_impl_cache_validation.rs` | `crates/z00z_wallets/src/key/key_manager_impl_cache_validation.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 168 | `crates/z00z_wallets/src/key/manager/key_manager_impl_gap.rs` | `crates/z00z_wallets/src/key/key_manager_impl_gap.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 169 | `crates/z00z_wallets/src/key/manager/key_manager_impl_state.rs` | `crates/z00z_wallets/src/key/key_manager_impl_state.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 170 | `crates/z00z_wallets/src/key/manager/key_manager_impl_system.rs` | `crates/z00z_wallets/src/key/key_manager_impl_system.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 171 | `crates/z00z_wallets/src/key/manager/key_manager_impl_trait.rs` | `crates/z00z_wallets/src/key/key_manager_impl_trait.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 172 | `crates/z00z_wallets/src/key/manager/key_manager_redb.rs` | `crates/z00z_wallets/src/key/key_manager_redb.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 173 | `crates/z00z_wallets/src/key/manager/key_manager_redb_wallet.rs` | `crates/z00z_wallets/src/key/key_manager_redb_wallet.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 174 | `crates/z00z_wallets/src/key/manager/key_state.rs` | `crates/z00z_wallets/src/key/key_state.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 175 | `crates/z00z_wallets/src/key/manager/mod.rs` | `crates/z00z_wallets/src/key/manager.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 176 | `crates/z00z_wallets/src/key/manager/test_key_manager_impl_suite.rs` | `crates/z00z_wallets/src/key/test_key_manager_impl_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 177 | `crates/z00z_wallets/src/key/manager/test_key_manager_password_suite.rs` | `crates/z00z_wallets/src/key/test_key_manager_password_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 178 | `crates/z00z_wallets/src/key/manager/test_key_manager_redb_suite.rs` | `crates/z00z_wallets/src/key/test_key_manager_redb_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 179 | `crates/z00z_wallets/src/key/receiver/mod.rs` | `crates/z00z_wallets/src/key/receiver.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 180 | `crates/z00z_wallets/src/key/receiver/stealth_keys.rs` | `crates/z00z_wallets/src/key/stealth_keys.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 181 | `crates/z00z_wallets/src/key/receiver/stealth_keys_identity.rs` | `crates/z00z_wallets/src/key/stealth_keys_identity.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 182 | `crates/z00z_wallets/src/key/receiver/stealth_keys_receiver.rs` | `crates/z00z_wallets/src/key/stealth_keys_receiver.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 183 | `crates/z00z_wallets/src/key/receiver/stealth_keys_secret.rs` | `crates/z00z_wallets/src/key/stealth_keys_secret.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 184 | `crates/z00z_wallets/src/key/receiver/test_stealth_keys_suite.rs` | `crates/z00z_wallets/src/key/test_stealth_keys_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 185 | `crates/z00z_wallets/src/key/seed/mod.rs` | `crates/z00z_wallets/src/key/seed.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 186 | `crates/z00z_wallets/src/key/seed/seed_backup_format.rs` | `crates/z00z_wallets/src/key/seed_backup_format.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 187 | `crates/z00z_wallets/src/key/seed/seed_backup_format_errors.rs` | `crates/z00z_wallets/src/key/seed_backup_format_errors.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 188 | `crates/z00z_wallets/src/key/seed/seed_backup_format_phrase.rs` | `crates/z00z_wallets/src/key/seed_backup_format_phrase.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 189 | `crates/z00z_wallets/src/key/seed/seed_cipher.rs` | `crates/z00z_wallets/src/key/seed_cipher.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 190 | `crates/z00z_wallets/src/key/seed/seed_cipher_container.rs` | `crates/z00z_wallets/src/key/seed_cipher_container.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 191 | `crates/z00z_wallets/src/key/seed/seed_cipher_container_crypto.rs` | `crates/z00z_wallets/src/key/seed_cipher_container_crypto.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 192 | `crates/z00z_wallets/src/key/seed/seed_cipher_ids.rs` | `crates/z00z_wallets/src/key/seed_cipher_ids.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 193 | `crates/z00z_wallets/src/key/seed/seed_cipher_params.rs` | `crates/z00z_wallets/src/key/seed_cipher_params.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 194 | `crates/z00z_wallets/src/key/seed/seed_cipher_persistence.rs` | `crates/z00z_wallets/src/key/seed_cipher_persistence.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 195 | `crates/z00z_wallets/src/key/seed/seed_cipher_types.rs` | `crates/z00z_wallets/src/key/seed_cipher_types.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 196 | `crates/z00z_wallets/src/key/seed/seed_entropy.rs` | `crates/z00z_wallets/src/key/seed_entropy.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 197 | `crates/z00z_wallets/src/key/seed/seed_mnemonic.rs` | `crates/z00z_wallets/src/key/seed_mnemonic.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 198 | `crates/z00z_wallets/src/key/seed/test_seed_backup_format_basic.rs` | `crates/z00z_wallets/src/key/test_seed_backup_format_basic.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 199 | `crates/z00z_wallets/src/key/seed/test_seed_backup_format_language.rs` | `crates/z00z_wallets/src/key/test_seed_backup_format_language.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 200 | `crates/z00z_wallets/src/key/seed/test_seed_backup_format_suite.rs` | `crates/z00z_wallets/src/key/test_seed_backup_format_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 201 | `crates/z00z_wallets/src/key/seed/test_seed_cipher_basic_suite.rs` | `crates/z00z_wallets/src/key/test_seed_cipher_basic_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 202 | `crates/z00z_wallets/src/key/seed/test_seed_cipher_metadata_suite.rs` | `crates/z00z_wallets/src/key/test_seed_cipher_metadata_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 203 | `crates/z00z_wallets/src/key/seed/test_seed_cipher_reencrypt_suite.rs` | `crates/z00z_wallets/src/key/test_seed_cipher_reencrypt_suite.rs` | F1 | Flatten key submodule while retaining existing domain prefixes. |
| 204 | `crates/z00z_wallets/src/persistence/assets/asset_storage.rs` | `crates/z00z_wallets/src/persistence/asset_storage.rs` | F1 | Flatten persistence storage family. |
| 205 | `crates/z00z_wallets/src/persistence/assets/asset_storage_impl/test_mod.rs` | `crates/z00z_wallets/src/persistence/test_asset_storage_impl.rs` | F1 | Flatten persistence storage family. |
| 206 | `crates/z00z_wallets/src/persistence/assets/asset_storage_impl.rs` | `crates/z00z_wallets/src/persistence/asset_storage_impl.rs` | F1 | Flatten persistence storage family. |
| 207 | `crates/z00z_wallets/src/persistence/assets/mod.rs` | `crates/z00z_wallets/src/persistence/assets.rs` | F1 | Flatten persistence storage family. |
| 208 | `crates/z00z_wallets/src/persistence/receipts/mod.rs` | `crates/z00z_wallets/src/persistence/receipts.rs` | F1 | Flatten persistence storage family. |
| 209 | `crates/z00z_wallets/src/persistence/receipts/receipt_storage.rs` | `crates/z00z_wallets/src/persistence/receipt_storage.rs` | F1 | Flatten persistence storage family. |
| 210 | `crates/z00z_wallets/src/persistence/receipts/receipt_storage_impl.rs` | `crates/z00z_wallets/src/persistence/receipt_storage_impl.rs` | F1 | Flatten persistence storage family. |
| 211 | `crates/z00z_wallets/src/persistence/receipts/storage.rs` | `N/A - remove duplicate or stale Rust file during implementation` | D2 | Duplicate of receipt_storage.rs except final newline; keep canonical receipt_storage.rs. |
| 212 | `crates/z00z_wallets/src/persistence/receipts/storage_impl.rs` | `N/A - remove duplicate or stale Rust file during implementation` | D2 | Duplicate of receipt_storage_impl.rs except final newline; keep canonical receipt_storage_impl.rs. |
| 213 | `crates/z00z_wallets/src/persistence/scans/mod.rs` | `crates/z00z_wallets/src/persistence/scans.rs` | F1 | Flatten persistence storage family. |
| 214 | `crates/z00z_wallets/src/persistence/scans/scan_storage.rs` | `crates/z00z_wallets/src/persistence/scan_storage.rs` | F1 | Flatten persistence storage family. |
| 215 | `crates/z00z_wallets/src/persistence/scans/scan_storage_impl.rs` | `crates/z00z_wallets/src/persistence/scan_storage_impl.rs` | F1 | Flatten persistence storage family. |
| 216 | `crates/z00z_wallets/src/persistence/scans/storage.rs` | `N/A - remove duplicate or stale Rust file during implementation` | D2 | Duplicate of scan_storage.rs except final newline; keep canonical scan_storage.rs. |
| 217 | `crates/z00z_wallets/src/persistence/scans/storage_impl.rs` | `N/A - remove duplicate or stale Rust file during implementation` | D2 | Duplicate implementation lane superseded by scan_storage_impl.rs. |
| 218 | `crates/z00z_wallets/src/persistence/tx/mod.rs` | `crates/z00z_wallets/src/persistence/tx.rs` | F1 | Flatten persistence storage family. |
| 219 | `crates/z00z_wallets/src/persistence/tx/tx_storage.rs` | `crates/z00z_wallets/src/persistence/tx_storage.rs` | F1 | Flatten persistence storage family. |
| 220 | `crates/z00z_wallets/src/persistence/tx/tx_storage_impl.rs` | `crates/z00z_wallets/src/persistence/tx_storage_impl.rs` | F1 | Flatten persistence storage family. |
| 221 | `crates/z00z_wallets/src/receiver/card/mod.rs` | `crates/z00z_wallets/src/receiver/card.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 222 | `crates/z00z_wallets/src/receiver/card/nfc_utils.rs` | `crates/z00z_wallets/src/receiver/nfc_ndef.rs` | R3 | Replace generic utils with the actual NFC NDEF record responsibility. |
| 223 | `crates/z00z_wallets/src/receiver/card/stealth_card/test_stealth_card.rs` | `crates/z00z_wallets/src/receiver/test_stealth_card.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 224 | `crates/z00z_wallets/src/receiver/card/stealth_card.rs` | `crates/z00z_wallets/src/receiver/stealth_card.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 225 | `crates/z00z_wallets/src/receiver/card/stealth_card_codec.rs` | `crates/z00z_wallets/src/receiver/stealth_card_codec.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 226 | `crates/z00z_wallets/src/receiver/card/stealth_trust.rs` | `crates/z00z_wallets/src/receiver/stealth_trust.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 227 | `crates/z00z_wallets/src/receiver/card/test_stealth_trust_suite.rs` | `crates/z00z_wallets/src/receiver/test_stealth_trust_suite.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 228 | `crates/z00z_wallets/src/receiver/manager/canonical_state.rs` | `crates/z00z_wallets/src/receiver/manager_canonical_state.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 229 | `crates/z00z_wallets/src/receiver/manager/eviction_listener.rs` | `crates/z00z_wallets/src/receiver/manager_eviction_listener.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 230 | `crates/z00z_wallets/src/receiver/manager/mod.rs` | `crates/z00z_wallets/src/receiver/manager.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 231 | `crates/z00z_wallets/src/receiver/manager/rate_limiter_bucket.rs` | `crates/z00z_wallets/src/receiver/manager_rate_limiter_bucket.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 232 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_cache.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_cache.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 233 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_config.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_config.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 234 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 235 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_async.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_async.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 236 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_builder.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_builder.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 237 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_runtime_derive.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_runtime_derive.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 238 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_runtime_maintenance.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_runtime_maintenance.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 239 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_state.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_state.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 240 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_state_io.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_state_io.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 241 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_impl_trait_impl.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_impl_trait_impl.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 242 | `crates/z00z_wallets/src/receiver/manager/receiver_manager_trait.rs` | `crates/z00z_wallets/src/receiver/receiver_manager_trait.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 243 | `crates/z00z_wallets/src/receiver/manager/test_canonical_state_suite.rs` | `crates/z00z_wallets/src/receiver/test_canonical_state_suite.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 244 | `crates/z00z_wallets/src/receiver/manager/test_receiver_manager_suite.rs` | `crates/z00z_wallets/src/receiver/test_receiver_manager_suite.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 245 | `crates/z00z_wallets/src/receiver/ownership/claim_own.rs` | `crates/z00z_wallets/src/receiver/stealth_ownership_check.rs` | R3 | Replace unclear claim_own grammar with the actual stealth ownership check. |
| 246 | `crates/z00z_wallets/src/receiver/ownership/mod.rs` | `crates/z00z_wallets/src/receiver/ownership.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 247 | `crates/z00z_wallets/src/receiver/request/mod.rs` | `crates/z00z_wallets/src/receiver/request.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 248 | `crates/z00z_wallets/src/receiver/request/stealth_request_crypto.rs` | `crates/z00z_wallets/src/receiver/stealth_request_crypto.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 249 | `crates/z00z_wallets/src/receiver/request/stealth_request_parse.rs` | `crates/z00z_wallets/src/receiver/stealth_request_parse.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 250 | `crates/z00z_wallets/src/receiver/request/stealth_request_transport.rs` | `crates/z00z_wallets/src/receiver/stealth_request_transport.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 251 | `crates/z00z_wallets/src/receiver/request/stealth_request_types.rs` | `crates/z00z_wallets/src/receiver/stealth_request_types.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 252 | `crates/z00z_wallets/src/receiver/request/test_stealth_request.rs` | `crates/z00z_wallets/src/receiver/test_stealth_request.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 253 | `crates/z00z_wallets/src/receiver/scan/ephemeral_cache.rs` | `crates/z00z_wallets/src/receiver/scan_ephemeral_cache.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 254 | `crates/z00z_wallets/src/receiver/scan/leaf_scan.rs` | `crates/z00z_wallets/src/receiver/leaf_scan.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 255 | `crates/z00z_wallets/src/receiver/scan/mod.rs` | `crates/z00z_wallets/src/receiver/scan.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 256 | `crates/z00z_wallets/src/receiver/scan/optimized_scanner.rs` | `crates/z00z_wallets/src/receiver/scan_optimized_scanner.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 257 | `crates/z00z_wallets/src/receiver/scan/rate_limiter.rs` | `crates/z00z_wallets/src/receiver/scan_rate_limiter.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 258 | `crates/z00z_wallets/src/receiver/scan/stealth_scan_support/test_stealth_scan_support_suite.rs` | `crates/z00z_wallets/src/receiver/test_stealth_scan_support_suite.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 259 | `crates/z00z_wallets/src/receiver/scan/stealth_scan_support.rs` | `crates/z00z_wallets/src/receiver/stealth_scan_support.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 260 | `crates/z00z_wallets/src/receiver/scan/stealth_scanner/test_stealth_scanner.rs` | `crates/z00z_wallets/src/receiver/test_stealth_scanner.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 261 | `crates/z00z_wallets/src/receiver/scan/stealth_scanner.rs` | `crates/z00z_wallets/src/receiver/stealth_scanner.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 262 | `crates/z00z_wallets/src/receiver/scan/types.rs` | `crates/z00z_wallets/src/receiver/scan_types.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 263 | `crates/z00z_wallets/src/receiver/scan/types_range.rs` | `crates/z00z_wallets/src/receiver/scan_types_range.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 264 | `crates/z00z_wallets/src/receiver/scan/types_receive.rs` | `crates/z00z_wallets/src/receiver/scan_types_receive.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 265 | `crates/z00z_wallets/src/receiver/scan/types_tag_cache.rs` | `crates/z00z_wallets/src/receiver/scan_types_tag_cache.rs` | F1 | Flatten receiver submodule while retaining receiver-specific prefixes. |
| 266 | `crates/z00z_wallets/src/security/vault/file_key_store.rs` | `crates/z00z_wallets/src/security/vault_file_key_store.rs` | F1 | Flatten security vault subtree. |
| 267 | `crates/z00z_wallets/src/security/vault/mod.rs` | `crates/z00z_wallets/src/security/vault.rs` | F1 | Flatten security vault subtree. |
| 268 | `crates/z00z_wallets/src/security/vault/secret_store.rs` | `crates/z00z_wallets/src/security/vault_secret_store.rs` | F1 | Flatten security vault subtree. |
| 269 | `crates/z00z_wallets/src/security/vault/secret_store_impl.rs` | `crates/z00z_wallets/src/security/vault_secret_store_impl.rs` | F1 | Flatten security vault subtree. |
| 270 | `crates/z00z_wallets/src/services/app/app_chain_network.rs` | `crates/z00z_wallets/src/services/app_chain_network.rs` | F1 | Flatten service implementation shard. |
| 271 | `crates/z00z_wallets/src/services/app/app_kernel.rs` | `crates/z00z_wallets/src/services/app_kernel.rs` | F1 | Flatten service implementation shard. |
| 272 | `crates/z00z_wallets/src/services/app/app_seed_password.rs` | `crates/z00z_wallets/src/services/app_seed_password.rs` | F1 | Flatten service implementation shard. |
| 273 | `crates/z00z_wallets/src/services/app/app_service_impl.rs` | `crates/z00z_wallets/src/services/app_service_impl.rs` | F1 | Flatten service implementation shard. |
| 274 | `crates/z00z_wallets/src/services/app/app_wallet_lifecycle.rs` | `crates/z00z_wallets/src/services/app_wallet_lifecycle.rs` | F1 | Flatten service implementation shard. |
| 275 | `crates/z00z_wallets/src/services/app/test_app_service_suite.rs` | `crates/z00z_wallets/src/services/test_app_service_suite.rs` | F1 | Flatten service implementation shard. |
| 276 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_assets.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_assets.rs` | F1 | Flatten service implementation shard. |
| 277 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_backup.rs` | F1 | Flatten service implementation shard. |
| 278 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_backup_rpc.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_backup_rpc.rs` | F1 | Flatten service implementation shard. |
| 279 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_hardening.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_hardening.rs` | F1 | Flatten service implementation shard. |
| 280 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_reachability.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs` | F1 | Flatten service implementation shard. |
| 281 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receive.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` | F1 | Flatten service implementation shard. |
| 282 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_receiver.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs` | F1 | Flatten service implementation shard. |
| 283 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_rpc.rs` | F1 | Flatten service implementation shard. |
| 284 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_runtime.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs` | F1 | Flatten service implementation shard. |
| 285 | `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_tofu.rs` | `crates/z00z_wallets/src/services/wallet_service_actions_tofu.rs` | F1 | Flatten service implementation shard. |
| 286 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_build.rs` | `crates/z00z_wallets/src/services/wallet_service_session_build.rs` | F1 | Flatten service implementation shard. |
| 287 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction.rs` | `crates/z00z_wallets/src/services/wallet_service_session_construction.rs` | F1 | Flatten service implementation shard. |
| 288 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction_helpers.rs` | `crates/z00z_wallets/src/services/wallet_service_session_construction_helpers.rs` | F1 | Flatten service implementation shard. |
| 289 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_construction_variants.rs` | `crates/z00z_wallets/src/services/wallet_service_session_construction_variants.rs` | F1 | Flatten service implementation shard. |
| 290 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs` | `crates/z00z_wallets/src/services/wallet_service_session_derivation.rs` | F1 | Flatten service implementation shard. |
| 291 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs` | `crates/z00z_wallets/src/services/wallet_service_session_derivation_recovery.rs` | F1 | Flatten service implementation shard. |
| 292 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_guards.rs` | `crates/z00z_wallets/src/services/wallet_service_session_guards.rs` | F1 | Flatten service implementation shard. |
| 293 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_lifecycle.rs` | `crates/z00z_wallets/src/services/wallet_service_session_lifecycle.rs` | F1 | Flatten service implementation shard. |
| 294 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_password.rs` | `crates/z00z_wallets/src/services/wallet_service_session_password.rs` | F1 | Flatten service implementation shard. |
| 295 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs` | `crates/z00z_wallets/src/services/wallet_service_session_rotation.rs` | F1 | Flatten service implementation shard. |
| 296 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime.rs` | `crates/z00z_wallets/src/services/wallet_service_session_runtime.rs` | F1 | Flatten service implementation shard. |
| 297 | `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_runtime_limits.rs` | `crates/z00z_wallets/src/services/wallet_service_session_runtime_limits.rs` | F1 | Flatten service implementation shard. |
| 298 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock.rs` | `crates/z00z_wallets/src/services/wallet_service_store_create_unlock.rs` | F1 | Flatten service implementation shard. |
| 299 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_create_unlock_open.rs` | `crates/z00z_wallets/src/services/wallet_service_store_open_source.rs` | R2 | Reduce six-word module name and match open_wallet_source behavior. |
| 300 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_load_restore.rs` | `crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs` | F1 | Flatten service implementation shard. |
| 301 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack.rs` | `crates/z00z_wallets/src/services/wallet_service_store_persistence_pack.rs` | F1 | Flatten service implementation shard. |
| 302 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_profile.rs` | `crates/z00z_wallets/src/services/wallet_service_store_export_pack.rs` | R2 | Reduce six-word module name and match WalletExportPack construction behavior. |
| 303 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs` | `crates/z00z_wallets/src/services/wallet_service_store_support.rs` | F1 | Flatten service implementation shard. |
| 304 | `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_transfer_import.rs` | `crates/z00z_wallets/src/services/wallet_service_store_transfer_import.rs` | F1 | Flatten service implementation shard. |
| 305 | `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs` | `crates/z00z_wallets/src/services/test_wallet_paths_suite.rs` | F1 | Flatten service implementation shard. |
| 306 | `N/A - no live aggregate file in the current workspace` | `N/A - stale planner row closed during 061-01 preflight` | D3 | Historical aggregate include lane is already absent; later slices must not invent it as a second authority path. |
| 307 | `crates/z00z_wallets/src/services/test_wallet_service.rs` | `crates/z00z_wallets/src/services/test_wallet_service.rs` | D4 | Retain the canonical one-level in-module WalletService test seam after nested shard retirement. |
| 308 | `crates/z00z_wallets/src/services/wallet_service_actions.rs` | `crates/z00z_wallets/src/services/wallet_service_actions.rs` | D4 | Retain the canonical one-level WalletService actions seam after nested shard retirement. |
| 309 | `crates/z00z_wallets/src/services/wallet_service_session.rs` | `crates/z00z_wallets/src/services/wallet_service_session.rs` | D4 | Retain the canonical one-level WalletService session seam after nested shard retirement. |
| 310 | `crates/z00z_wallets/src/services/wallet_service_store.rs` | `crates/z00z_wallets/src/services/wallet_service_store.rs` | D4 | Retain the canonical one-level WalletService store seam after nested shard retirement. |
| 311 | `crates/z00z_wallets/src/services/wallet_service_types_core.rs` | `crates/z00z_wallets/src/services/wallet_service_types_core.rs` | D4 | Retain the canonical one-level WalletService core-types seam after nested shard retirement. |
| 312 | `crates/z00z_wallets/src/services/wallet_service_types_reachability.rs` | `crates/z00z_wallets/src/services/wallet_service_types_reachability.rs` | D4 | Retain the canonical one-level WalletService reachability seam after nested shard retirement. |
| 313 | `crates/z00z_wallets/src/services/wallet_service_types_state.rs` | `crates/z00z_wallets/src/services/wallet_service_types_state.rs` | D4 | Retain the canonical one-level WalletService state seam after nested shard retirement. |
| 314 | `crates/z00z_wallets/src/stealth/crypto/ecdh.rs` | `crates/z00z_wallets/src/stealth/crypto_ecdh.rs` | F1 | Flatten stealth crypto/output subtree. |
| 315 | `crates/z00z_wallets/src/stealth/crypto/ecdh_validation.rs` | `crates/z00z_wallets/src/stealth/crypto_ecdh_validation.rs` | F1 | Flatten stealth crypto/output subtree. |
| 316 | `crates/z00z_wallets/src/stealth/crypto/encoding.rs` | `crates/z00z_wallets/src/stealth/crypto_encoding.rs` | F1 | Flatten stealth crypto/output subtree. |
| 317 | `crates/z00z_wallets/src/stealth/crypto/ephemeral.rs` | `crates/z00z_wallets/src/stealth/crypto_ephemeral.rs` | F1 | Flatten stealth crypto/output subtree. |
| 318 | `crates/z00z_wallets/src/stealth/crypto/mod.rs` | `crates/z00z_wallets/src/stealth/crypto.rs` | F1 | Flatten stealth crypto/output subtree. |
| 319 | `crates/z00z_wallets/src/stealth/zkpack/mod.rs` | `crates/z00z_wallets/src/stealth/zkpack.rs` | F1 | Flatten the live stealth zkpack subtree without reviving removed facade paths. |
| 320 | `crates/z00z_wallets/src/stealth/zkpack/test_mod.rs` | `crates/z00z_wallets/src/stealth/test_zkpack.rs` | F1 | Flatten the live stealth zkpack subtree without reviving removed facade paths. |
| 321 | `crates/z00z_wallets/src/stealth/output/mod.rs` | `crates/z00z_wallets/src/stealth/output.rs` | F1 | Flatten stealth crypto/output subtree. |
| 322 | `crates/z00z_wallets/src/stealth/output/output_build.rs` | `crates/z00z_wallets/src/stealth/output_build.rs` | F1 | Flatten stealth crypto/output subtree. |
| 323 | `crates/z00z_wallets/src/stealth/output/tests/test_extra.rs` | `crates/z00z_wallets/src/stealth/test_output_extra.rs` | F1 | Flatten stealth crypto/output subtree. |
| 324 | `crates/z00z_wallets/src/stealth/output/tests/test_mod.rs` | `crates/z00z_wallets/src/stealth/test_output.rs` | F1 | Flatten stealth crypto/output subtree. |
| 325 | `crates/z00z_wallets/src/tx/asset_selector/mod.rs` | `crates/z00z_wallets/src/tx/asset_selector.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 326 | `crates/z00z_wallets/src/tx/asset_selector/multi/test_mod.rs` | `crates/z00z_wallets/src/tx/test_asset_selector_multi.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 327 | `crates/z00z_wallets/src/tx/asset_selector/multi.rs` | `crates/z00z_wallets/src/tx/asset_selector_multi.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 328 | `crates/z00z_wallets/src/tx/asset_selector/test_mod.rs` | `crates/z00z_wallets/src/tx/test_asset_selector.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 329 | `crates/z00z_wallets/src/tx/claim_helpers.rs` | `crates/z00z_wallets/src/tx/claim_tx_hashing.rs` | R3 | Replace generic claim helper name with its digest, scope-hash, and nonce derivation responsibility. |
| 330 | `crates/z00z_wallets/src/tx/claim/claim_tx_helpers.rs` | `crates/z00z_wallets/src/tx/claim_tx_statement.rs` | R3 | Avoid ambiguous helper pair after flattening; this shard builds claim proof statements. |
| 331 | `crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl.rs` | `crates/z00z_wallets/src/tx/claim_tx_verifier_impl.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 332 | `crates/z00z_wallets/src/tx/claim/claim_tx_verifier_impl_proof.rs` | `crates/z00z_wallets/src/tx/claim_tx_verifier_impl_proof.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 333 | `crates/z00z_wallets/src/tx/claim_tx/mod.rs` | `crates/z00z_wallets/src/tx/claim_tx.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 334 | `crates/z00z_wallets/src/tx/claim_tx/test_claim_tx.rs` | `crates/z00z_wallets/src/tx/test_claim_tx.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 335 | `crates/z00z_wallets/src/tx/fee_estimator/mod.rs` | `crates/z00z_wallets/src/tx/fee_estimator.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 336 | `crates/z00z_wallets/src/tx/fee_estimator/test_mod.rs` | `crates/z00z_wallets/src/tx/test_fee_estimator.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 337 | `crates/z00z_wallets/src/tx/state_update/mod.rs` | `crates/z00z_wallets/src/tx/state_update.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 338 | `crates/z00z_wallets/src/tx/state_update/test_mod.rs` | `crates/z00z_wallets/src/tx/test_state_update.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 339 | `crates/z00z_wallets/src/tx/tx_verifier/mod.rs` | `crates/z00z_wallets/src/tx/tx_verifier.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 340 | `crates/z00z_wallets/src/tx/tx_verifier/test_mod.rs` | `crates/z00z_wallets/src/tx/test_tx_verifier.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 341 | `crates/z00z_wallets/src/tx/verify/tx_verifier_helpers.rs` | `crates/z00z_wallets/src/tx/tx_verifier_helpers.rs` | F1 | Flatten tx submodule while preserving tx prefixes. |
| 342 | `crates/z00z_wallets/src/wallet/entity/wallet_entity.rs` | `crates/z00z_wallets/src/wallet/wallet_entity.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 343 | `crates/z00z_wallets/src/wallet/entity/wallet_entity_asset_api.rs` | `crates/z00z_wallets/src/wallet/wallet_entity_asset_api.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 344 | `crates/z00z_wallets/src/wallet/entity/wallet_entity_constructor.rs` | `crates/z00z_wallets/src/wallet/wallet_entity_constructor.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 345 | `crates/z00z_wallets/src/wallet/entity/wallet_entity_core.rs` | `crates/z00z_wallets/src/wallet/wallet_entity_core.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 346 | `crates/z00z_wallets/src/wallet/entity/wallet_entity_wallet_api.rs` | `crates/z00z_wallets/src/wallet/wallet_entity_wallet_api.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 347 | `crates/z00z_wallets/src/wallet/errors/errors_impls.rs` | `crates/z00z_wallets/src/wallet/errors_impls.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 348 | `crates/z00z_wallets/src/wallet/errors/errors_types.rs` | `crates/z00z_wallets/src/wallet/errors_types.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 349 | `crates/z00z_wallets/src/wallet/errors/test_errors_suite.rs` | `crates/z00z_wallets/src/wallet/test_errors_suite.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 350 | `crates/z00z_wallets/src/wallet/persistence/persistence_types.rs` | `crates/z00z_wallets/src/wallet/persistence_types.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 351 | `crates/z00z_wallets/src/wallet/responses/stub_responses_asset.rs` | `crates/z00z_wallets/src/wallet/stub_responses_asset.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 352 | `crates/z00z_wallets/src/wallet/responses/stub_responses_backup.rs` | `crates/z00z_wallets/src/wallet/stub_responses_backup.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 353 | `crates/z00z_wallets/src/wallet/responses/stub_responses_tx.rs` | `crates/z00z_wallets/src/wallet/stub_responses_tx.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |
| 354 | `crates/z00z_wallets/src/wallet/responses/stub_responses_wallet.rs` | `crates/z00z_wallets/src/wallet/stub_responses_wallet.rs` | F1 | Flatten wallet submodule while preserving wallet prefixes. |

## 🧭 Implementation Notes

- **IMP-001:** Add `src/rpc/` and keep `adapters::rpc` by changing `src/adapters/mod.rs` to `#[path = "../rpc/mod.rs"] pub mod rpc;`; do not add a new public crate-root `rpc` module unless a separate public API decision approves it.
- **IMP-002:** Add `src/redb_store/` only for the concrete native RedB wallet-store backend, and keep `db::redb_wallet_store` by changing `src/db/mod.rs` to `#[path = "../redb_store/mod.rs"] pub mod redb_wallet_store;`; do not add a new public crate-root `redb_store` module unless a separate public API decision approves it.
- **IMP-003:** Update every affected `mod`, `#[path]`, and `include!` path in the same implementation wave as its file move.
- **IMP-004:** Do not combine this structural wave with public API renames beyond the table decisions.
- **IMP-005:** For rows with `N/A`, implementation should remove the old file only after references are proven absent or replaced.
- **IMP-006:** Non-Rust nested artifacts under `src/` should move out of `src` in the same cleanup milestone: schemas to `crates/z00z_wallets/schemas/`, key/wallet/domain docs to the canonical flat `crates/z00z_wallets/docs/` home, and any remaining EGUI reference bundle to `crates/z00z_wallets/docs/` unless the live code already proves a different single-owner asset home.
- **IMP-007:** Rename the shared crypto module from `db::redb_wallet_crypto` to `db::wallet_store_crypto` and update internal Rust references; do not move this shared WASM/native persistence contract into `src/redb_store/`.
- **IMP-008:** Preserve persisted RedB wallet compatibility: the R4 rename is a Rust module/file naming change only and must not rename domain-separation labels, schema constants, stored metadata keys, or serialized record formats.
- **IMP-009:** For R5 rows, update RPC module declarations and imports from `asset_rpc_*` / `tx_rpc_*` to the planned `method_asset_*` / `method_tx_*` names in the same wave.
- **IMP-010:** For claim tx helper split rows, update `tx/claim_tx.rs`, `tx/mod.rs`, and included verifier shards so digest/hash/nonce references use `claim_tx_hashing` and statement/proof helpers use `claim_tx_statement`.

## ✅ Verification Plan

- **VER-001:** Before implementation, run `find crates/z00z_wallets/src -type f -name "*.rs" | sort` and compare against this table.
- **VER-002:** After implementation, run `find crates/z00z_wallets/src -mindepth 3 -type f -name "*.rs"` and require zero output.
- **VER-003:** After implementation, run a duplicate-target audit for all `mod` and `#[path]` declarations touched by the move.
- **VER-004:** Run `cargo fmt --all --check`.
- **VER-005:** Run `cargo check --release -p z00z_wallets --all-targets --all-features`.
- **VER-006:** Run `cargo test --release -p z00z_wallets --all-targets --all-features`.
- **VER-007:** Run `rg -n "adapters::rpc|db::redb_wallet_store|wallet_service::" crates/z00z_wallets/src -g "*.rs"` and confirm compatibility paths are intentional.
- **VER-008:** Run `rg -n "db::redb_wallet_crypto|crate::db::redb_wallet_crypto|pub mod redb_wallet_crypto" crates/z00z_wallets/src -g "*.rs"` and require zero Rust module-path references after the R4 rename.
- **VER-009:** Run `rg -n "z00z\\.crypto\\.redb_wallet_crypto|RedbWallet.*Domain|Z00ZRedbWalletAadIdDomain" crates/z00z_wallets/src -g "*.rs"` and confirm persistent domain labels remain intentionally unchanged.
- **VER-010:** Run `rg -n "asset_rpc_|tx_rpc_" crates/z00z_wallets/src/rpc -g "*.rs"` after implementation and require zero stale module references except unchanged JSON-RPC method strings or test names that intentionally describe external RPC behavior.
- **VER-011:** Run `rg -n "claim_helpers|claim_tx_helpers" crates/z00z_wallets/src/tx -g "*.rs"` after implementation and require zero stale module references.

## 🔎 Table Generation Checks

- **CHECK-001:** Rust source files scanned at the 061-01 live preflight: `497`.
- **CHECK-002:** Rename/remove decisions recorded: `354`.
- **CHECK-003:** Proposed `new-path` collisions: `0`.
- **CHECK-004:** Proposed Rust paths deeper than `src/<dir>/<file>.rs`: `0`.
- **CHECK-005:** Proposed module filenames over five words: `0`.
- **CHECK-006:** Proposed target paths conflicting with existing unlisted Rust files: `0`.
- **CHECK-007:** Listed `old-path` entries missing from the current workspace after 061-01 drift correction: `0`.
- **CHECK-008:** Current nested Rust files not covered by the table after 061-01 drift correction: `0`.
- **CHECK-009:** Proposed `src/rpc` filenames with redundant `_rpc_` qualifier: `0`.
- **CHECK-010:** Proposed ambiguous `claim_helpers` / `claim_tx_helpers` target filenames: `0`.
