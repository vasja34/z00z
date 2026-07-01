---
phase: 061-Wallet-Refactoring
plan: 061-06
status: complete
completed_at: 2026-06-23
next_plan: 061-07
summary_artifact_for: .planning/phases/061-Wallet-Refactoring/061-06-PLAN.md
---

# 061-06 Summary: RPC Method Implementation Flattening And Helper Renames

## Completed Scope

`061-06` is complete for the RPC-method flattening slice.

This slice flattened the old
`crates/z00z_wallets/src/adapters/rpc/methods/**` implementation tree into the
one-level `crates/z00z_wallets/src/rpc/` method layout required by
`061-TODO.md`, while preserving the caller-visible RPC facade:

- The old nested RPC method tree was retired and replaced with flat
  `src/rpc/method_*.rs` and `src/rpc/test_*.rs` files.
- `asset_rpc_*` helpers were renamed to `method_asset_*`.
- `tx_rpc_*` helpers were renamed to `method_tx_*`.
- The shared tx helper moved from `tx_rpc_impl.rs` to the canonical
  `method_tx_support.rs` path to avoid collision with `method_tx_impl.rs`.
- `src/rpc/mod.rs` and `src/rpc/methods.rs` now point directly at the flat
  layout while preserving the existing `adapters::rpc::methods::*` module
  surface.
- Nested include chains for asset, key, and tx method implementations were
  rewired to the new one-level file home in the same slice.
- Path-sensitive tests, wallet docs, and Phase 061 planning/context artifacts
  were updated atomically to the new canonical method-file paths.
- `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py` was corrected to
  read the live `src/rpc/` and flat service/core layout after the move so the
  replay-audit coverage remains bound to the current codebase instead of the
  retired tree.

## Files Changed

- `crates/z00z_wallets/src/rpc/mod.rs`
- `crates/z00z_wallets/src/rpc/methods.rs`
- `crates/z00z_wallets/src/rpc/method_*.rs`
- `crates/z00z_wallets/src/rpc/test_*.rs`
- `crates/z00z_wallets/tests/test_rpc_truth.rs`
- `crates/z00z_wallets/tests/test_output_reception.rs`
- `crates/z00z_wallets/tests/test_rename_guards.rs`
- `crates/z00z_wallets/src/services/test_wallet_service.rs`
- `crates/z00z_wallets/scripts/audit_rpc_method_wiring.py`
- `crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md`
- `crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md`
- `.planning/phases/061-Wallet-Refactoring/061-CONTEXT.md`
- `.planning/phases/061-Wallet-Refactoring/061-06-PLAN.md`

Retired old paths in this slice:

- `crates/z00z_wallets/src/adapters/rpc/methods/**`

## Boundary Kept

- The `adapters::rpc` facade remains the canonical caller-visible wallet RPC
  boundary.
- `register_all_wallet_rpc_methods` and `register_all_app_rpc_methods` remain
  the canonical registration entrypoints.
- This slice did not reopen RPC support/logging/types ownership except for the
  import and path rewiring required by the method move.
- External JSON-RPC behavior and method naming were not redefined in this
  slice; the work stayed structural.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited the flat `src/rpc` inventory, helper-name residue, and stale
  relative include paths. The first pass exposed stale include targets and the
  replay-audit script still pointing at retired paths; both were fixed in the
  same slice.
- Pass 2 audited whitespace hygiene, flat-tree truth, and old subtree
  retirement. `git diff --check` was clean and
  `find crates/z00z_wallets/src/adapters/rpc -mindepth 1 -maxdepth 3` returned
  no live child files or directories.
- Pass 3 audited registration entrypoints and stale helper-name residue.
  `register_all_wallet_rpc_methods` and `register_all_app_rpc_methods` remained
  live on the preserved facade, and stale `asset_rpc_` / `tx_rpc_` helper hits
  were gone from live Rust code.

Two consecutive clean review passes were achieved on passes 2 and 3. A final
closeout sweep also confirmed clean touched-file whitespace, an empty retired
RPC subtree, and no live stale helper or include residue outside intentional
historical planning references and one unchanged test function name that still
describes external RPC behavior.

## Validation

- Mandatory bootstrap gate passed before broader validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo check --release -p z00z_wallets --all-targets --all-features` passed.
- `cargo test --release -p z00z_wallets --test test_rpc_logging_replay_audit -- --nocapture`
  passed after the audit-script path correction.
- `cargo test --release -p z00z_wallets --all-targets --all-features` passed.
- `git diff --check -- crates/z00z_wallets/src/rpc crates/z00z_wallets/tests crates/z00z_wallets/scripts/audit_rpc_method_wiring.py .planning/phases/061-Wallet-Refactoring/061-CONTEXT.md .planning/phases/061-Wallet-Refactoring/061-06-PLAN.md`
  is clean.
- `find crates/z00z_wallets/src/adapters/rpc -mindepth 1 -maxdepth 3 | sort`
  confirmed the retired nested methods tree is gone.
- `rg -n "asset_rpc_|tx_rpc_|src/adapters/rpc/methods|include!\\(\"\\.\\./|include!\\(\"\\.\\./\\.\\./" crates/z00z_wallets/src/rpc crates/z00z_wallets/src/services crates/z00z_wallets/tests .planning/phases/061-Wallet-Refactoring -g '*.rs' -g '*.md'`
  found only intentional historical references in older plan artifacts and one
  unchanged test function name in `src/rpc/test_asset_impl.rs`.

## Result

`061-06` is complete. Phase 061 advances to `061-07-PLAN.md` for the
receiver, persistence, and security-vault flattening wave while keeping one
canonical RPC method path per behavior and preserving the existing wallet RPC
facade.
