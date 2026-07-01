---
phase: 061-Wallet-Refactoring
plan: 061-04
status: complete
completed_at: 2026-06-23
next_plan: 061-05
summary_artifact_for: .planning/phases/061-Wallet-Refactoring/061-04-PLAN.md
---

# 061-04 Summary: RPC Facade Support Move And Wallet Config Authority Relocation

## Completed Scope

`061-04` is complete for the RPC support-tree and wallet-config-anchor slice.

This slice moved the non-method RPC support tree into the flat
`crates/z00z_wallets/src/rpc/*.rs` layout while preserving the caller-visible
compatibility surface at `crate::adapters::rpc`:

- `crates/z00z_wallets/src/adapters/mod.rs` now preserves the facade through
  `#[path = "../rpc/mod.rs"] pub mod rpc;`.
- The physical support, logging, DTO/types, error-mapping, and dispatcher
  files now live under one-level `crates/z00z_wallets/src/rpc/*.rs`.
- The RPC methods implementation tree stays under
  `crates/z00z_wallets/src/adapters/rpc/methods/**` for the next slice.
- The embedded default wallet config moved from
  `crates/z00z_wallets/src/wallet_config.yaml` to the canonical non-`src/`
  home at `crates/z00z_wallets/config/wallet_config.yaml`.
- `crates/z00z_wallets/src/wallet_config_support.rs` now provides the single
  shared embedded-content and runtime-path authority used by wallet paths,
  RPC logging, and the key-RPC env-setup test helper.

## Files Changed

- `crates/z00z_wallets/src/adapters/mod.rs`
- `crates/z00z_wallets/src/rpc/*.rs`
- `crates/z00z_wallets/src/wallet_config_support.rs`
- `crates/z00z_wallets/config/wallet_config.yaml`
- `crates/z00z_wallets/src/services/wallet_paths.rs`
- `crates/z00z_wallets/src/services/wallet/tests/test_wallet_paths_suite.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_mod.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_replay_audit.rs`
- `crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md`
- `.planning/phases/061-Wallet-Refactoring/061-CONTEXT.md`
- `.planning/phases/061-Wallet-Refactoring/061-04-PLAN.md`

Retired old paths in this slice:

- `crates/z00z_wallets/src/adapters/rpc/logging/**`
- `crates/z00z_wallets/src/adapters/rpc/types/**`
- `crates/z00z_wallets/src/adapters/rpc/app_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs`
- `crates/z00z_wallets/src/adapters/rpc/error_mapping.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring/**`
- `crates/z00z_wallets/src/wallet_config.yaml`

## Boundary Kept

- `crate::adapters::rpc` remains the stable caller-visible path.
- No new crate-root public `rpc` API surface was introduced.
- RPC method implementation files were not flattened in this slice.
- Wallet config authority now has one canonical non-`src/` home plus one
  shared support helper, with no duplicate embedded-default source.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited stale-path residue versus live authority paths. Only
  historical references in `061-TODO.md` and older summary text remained; no
  live code or live Phase 061 authority artifact pointed at stale RPC-support
  or wallet-config paths.
- Pass 2 audited the physical tree split. `src/adapters/rpc/**` retained only
  `methods/**`, `src/rpc/*.rs` held the flattened support files, and
  `git diff --check` was clean.
- Pass 3 rechecked the Phase 061 authority packet. `061-CONTEXT.md` and
  `061-04-PLAN.md` point at the live `src/rpc/*.rs` and
  `config/wallet_config.yaml` locations, matching the code tree.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

- Mandatory bootstrap gate passed before broader validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo check --release -p z00z_wallets --all-targets --all-features` passed.
- `cargo test --release -p z00z_wallets --all-targets --all-features` passed.
- The post-move regression in
  `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/test_mod.rs` was
  fixed by switching the env-setup helper from the stale
  `src/wallet_config.yaml` anchor to
  `crate::wallet_config_support::default_wallet_config_path()`, after which
  bootstrap and the full release test packet reran green.
- `find crates/z00z_wallets/src/adapters/rpc -maxdepth 2 -type f | sort`
  confirmed that the remaining nested tree is the `methods/**` lane only.
- `find crates/z00z_wallets/src/rpc -maxdepth 1 -type f | sort` confirmed the
  flat support-file inventory under `src/rpc/*.rs`.
- `rg -n "src/wallet_config\\.yaml|/src/wallet_config\\.yaml|src/adapters/rpc/logging/|src/adapters/rpc/types/|src/adapters/rpc/wallet_dispatcher_wiring/" crates/z00z_wallets .planning/phases/061-Wallet-Refactoring -g '*.rs' -g '*.md' -g '*.yaml' -g '*.inc'`
  found only intentional historical references in `061-TODO.md` and older
  summaries, not live-code or live-plan authority drift.

## Result

`061-04` is complete. Phase 061 advances to `061-05-PLAN.md` for the
WalletService internal flattening slice while keeping the RPC compatibility
facade and wallet-config authority on one canonical path each.
