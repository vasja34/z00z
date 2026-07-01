---
phase: 061-Wallet-Refactoring
plan: 061-05
status: complete
completed_at: 2026-06-23
next_plan: 061-06
summary_artifact_for: .planning/phases/061-Wallet-Refactoring/061-05-PLAN.md
---

# 061-05 Summary: WalletService Internal Flattening And Service Anchor Preservation

## Completed Scope

`061-05` is complete for the service-tree flattening slice.

This slice moved the internal `services/app/*` and
`services/wallet/{actions,session,store}/*` shard trees into one-level
`crates/z00z_wallets/src/services/*.rs` files while preserving the
caller-visible facades at `services::WalletService` and `services::AppService`:

- `app_service.rs` now includes the flat `app_*.rs` and
  `test_app_service_suite.rs` files from the one-level `src/services/` home.
- `wallet_service_actions.rs`, `wallet_service_session.rs`, and
  `wallet_service_store.rs` now include one-level shard files from the same
  directory instead of nested `wallet/actions/*`, `wallet/session/*`, and
  `wallet/store/*` trees.
- The service-source anchor test moved to
  `crates/z00z_wallets/src/services/test_app_service_suite.rs`.
- The store helper names were shortened as owned by `R2` in this slice:
  `wallet_service_store_create_unlock_open.rs` became
  `wallet_service_store_open_source.rs`, and
  `wallet_service_store_persistence_pack_profile.rs` became
  `wallet_service_store_export_pack.rs`.
- Source-anchor-sensitive tests and service-oriented docs were updated
  atomically to the new canonical paths.

## Files Changed

- `crates/z00z_wallets/src/services/app_service.rs`
- `crates/z00z_wallets/src/services/wallet_service.rs`
- `crates/z00z_wallets/src/services/wallet_service_actions.rs`
- `crates/z00z_wallets/src/services/wallet_service_session.rs`
- `crates/z00z_wallets/src/services/wallet_service_store.rs`
- `crates/z00z_wallets/src/services/app_*.rs`
- `crates/z00z_wallets/src/services/test_app_service_suite.rs`
- `crates/z00z_wallets/src/services/wallet_service_actions_*.rs`
- `crates/z00z_wallets/src/services/wallet_service_session_*.rs`
- `crates/z00z_wallets/src/services/wallet_service_store_*.rs`
- `crates/z00z_wallets/tests/test_rpc_truth.rs`
- `crates/z00z_wallets/tests/test_output_reception.rs`
- `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
- `crates/z00z_wallets/tests/test_scenario1_semantics.rs`
- `crates/z00z_wallets/src/wallet/WALLET-GUIDE.md`
- `crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md`
- `.planning/phases/061-Wallet-Refactoring/061-CONTEXT.md`
- `.planning/phases/061-Wallet-Refactoring/061-05-PLAN.md`

Retired old paths in this slice:

- `crates/z00z_wallets/src/services/app/**`
- `crates/z00z_wallets/src/services/wallet/actions/**`
- `crates/z00z_wallets/src/services/wallet/session/**`
- `crates/z00z_wallets/src/services/wallet/store/**`

## Boundary Kept

- `services::WalletService` remains the stable caller-visible wallet facade.
- `services::AppService` remains the stable app-service facade.
- The named internal seams `wallet_service_actions`, `wallet_service_session`,
  and `wallet_service_store` remain explicit modules; the slice did not revert
  them to one giant include-assembly surface.
- RPC method implementation churn was not mixed into this slice.
- The still-nested `crates/z00z_wallets/src/services/wallet/tests/**` lane was
  left untouched because it is outside the owned `actions/session/store`
  flattening scope of `061-05`.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited stale path residue across the repository. Remaining matches
  were limited to historical references in `061-TODO.md` and older `061-01`
  planning artifacts, not live code or live Phase 061 authority files.
- Pass 2 audited the physical tree and whitespace hygiene. The flat
  `src/services/*.rs` inventory is present, the nested owned shard trees are
  retired, `src/services/wallet/**` now retains only the out-of-scope
  `wallet/tests/**` lane, and `git diff --check` is clean.
- Pass 3 audited the live authority packet. `061-CONTEXT.md`,
  `061-05-PLAN.md`, tests, and wallet docs now point at the live flattened
  service files, including the `R2` store-name shortenings.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

- Mandatory bootstrap gate passed before broader validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo check --release -p z00z_wallets --all-targets --all-features` passed.
- `cargo test --release -p z00z_wallets --all-targets --all-features` passed.
- `find crates/z00z_wallets/src/services -maxdepth 1 -type f | sort` confirmed
  the one-level service shard inventory.
- `find crates/z00z_wallets/src/services/wallet -maxdepth 3 -type f | sort`
  confirmed that the remaining nested wallet-service subtree is only
  `wallet/tests/test_wallet_paths_suite.rs`, which is outside this slice.
- `rg -n "src/services/app/|src/services/wallet/(actions|session|store)/|services/wallet/(actions|session|store)/|wallet_service_store_create_unlock_open|wallet_service_store_persistence_pack_profile" crates/z00z_wallets/src crates/z00z_wallets/tests crates/z00z_wallets .planning/phases/061-Wallet-Refactoring -g '*.rs' -g '*.md' -g '*.yaml' -g '*.inc'`
  found only intentional historical references in `061-TODO.md` and older
  `061-01` planning artifacts.

## Result

`061-05` is complete. Phase 061 advances to `061-06-PLAN.md` for the RPC
methods and `_rpc_` helper flattening wave while keeping the service facades
and source-anchor contract stable.
