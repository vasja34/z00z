---
phase: 061-Wallet-Refactoring
plan: 061-03
status: complete
completed_at: 2026-06-23
next_plan: 061-04
summary_artifact_for: .planning/phases/061-Wallet-Refactoring/061-03-PLAN.md
---

# 061-03 Summary: RedB Store Facade Move And Persistence Anchor Preservation

## Completed Scope

`061-03` is complete for the RedB backend relocation slice.

This slice moved the concrete backend from the nested
`src/db/redb_wallet_store/**` tree into the flat `src/redb_store/*.rs` layout
while preserving the caller-visible facade at `crate::db::redb_wallet_store`:

- `src/db/mod.rs` now exposes the preserved facade through
  `#[path = "../redb_store/mod.rs"] pub mod redb_wallet_store;`.
- The concrete backend now lives under one-level `src/redb_store/*.rs` files.
- The backend-only nested Rust subtree under `src/db/redb_wallet_store/**` was
  removed.
- The RedB schema moved from `src/db/redb/schema/redb-schema.yaml` to the
  canonical non-`src/` home at `crates/z00z_wallets/schemas/redb-schema.yaml`.
- Anchor-sensitive tests and source-inspection fixtures were updated atomically
  to the new backend and schema locations.

## Files Changed

- `crates/z00z_wallets/src/db/mod.rs`
- `crates/z00z_wallets/src/db/wallet_store_crypto.rs`
- `crates/z00z_wallets/src/redb_store/*.rs`
- `crates/z00z_wallets/schemas/redb-schema.yaml`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`
- `crates/z00z_wallets/tests/test_wallet_persistence_no_std.rs`
- `crates/z00z_wallets/tests/test_wallet_persist_nostd_fs.rs`
- `crates/z00z_wallets/tests/test_rename_guards.rs`

Retired old paths in this slice:

- `crates/z00z_wallets/src/db/redb_wallet_store/**`
- `crates/z00z_wallets/src/db/redb/schema/redb-schema.yaml`

## Boundary Kept

- `crate::db::redb_wallet_store` remains the stable caller-visible path.
- Shared `db` helpers such as `wallet_store_crypto*`, `wallet_store`, and
  related shared modules were not absorbed into `src/redb_store/`.
- No RPC or WalletService flattening work was mixed into this slice.
- No parallel backend authority layer or duplicate schema home was introduced.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited the physical move, facade preservation, stale nested-path
  removal, source-inspection test rewrites, and whitespace hygiene. No
  significant issues were found.
- Pass 2 audited the flat `src/redb_store/*.rs` inventory, stale old-path
  references, canonical schema home, and external include anchors for
  `TODO-Wallet-idea.md`, `assets_config.yaml`, and `WALLET-GUIDE.md`. No
  significant issues were found.
- Pass 3 rechecked the facade wiring, module-path glue, canonical schema-file
  existence, and diff whitespace on the final tree. No significant issues were
  found.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

- Mandatory bootstrap gate passed before broader validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo check --release -p z00z_wallets --all-targets --all-features` passed.
- `cargo test --release -p z00z_wallets --all-targets --all-features` passed.
- `find crates/z00z_wallets/src -type f -path '*/redb_wallet_store/*'`
  returned no matches.
- `rg -n "src/db/redb_wallet_store|db/redb_wallet_store/|db/redb/schema/redb-schema.yaml|src/db/redb/schema/redb-schema.yaml|redb/schema/redb-schema.yaml|redb_wallet_store/debug/debug_export.rs" crates/z00z_wallets/src crates/z00z_wallets/tests docs -g '*.rs' -g '*.md' -g '*.yaml'`
  returned no matches.
- `rg -n "redb_wallet_store|redb_store" crates/z00z_wallets/tests/test_wallet_persistence_no_std.rs crates/z00z_wallets/tests/test_wallet_persist_nostd_fs.rs crates/z00z_wallets/tests/test_rename_guards.rs`
  confirmed the source-inspection tests now point at the canonical
  `src/redb_store/...` locations.
- `rg -n "schemas/redb-schema.yaml|TODO-Wallet-idea.md|assets_config.yaml|WALLET-GUIDE.md" crates/z00z_wallets/src/redb_store/test_redb_store.rs crates/z00z_wallets/src/wallet/WALLET-GUIDE.md crates/z00z_wallets/src/db/wallet_store_crypto.rs`
  confirmed the schema, wallet-guide, and external include anchors still
  resolve intentionally.
- `git diff --check -- crates/z00z_wallets/src/db crates/z00z_wallets/src/redb_store crates/z00z_wallets/schemas/redb-schema.yaml crates/z00z_wallets/src/wallet/WALLET-GUIDE.md crates/z00z_wallets/tests/test_wallet_persistence_no_std.rs crates/z00z_wallets/tests/test_wallet_persist_nostd_fs.rs crates/z00z_wallets/tests/test_rename_guards.rs`
  is clean.

## Result

`061-03` is complete. Phase 061 advances to `061-04-PLAN.md` for the RPC
support-tree and wallet-config-anchor move while keeping the RedB backend on
one canonical flat implementation path.
