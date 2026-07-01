---
phase: 029-crypto-audit-wallets
artifact: panic-inventory
status: current-tree
generated_at: 2026-03-30
---

<!-- markdownlint-disable MD041 -->

## Scope

📌 This inventory classifies every current-tree `expect()` or `unwrap()` in the
wallet service panic-review scope for `029-04`.

📌 Classifications used here:

- `runtime-fixed`: operator-reachable before this wave, now routed to typed
  `WalletError` or RPC error.
- `runtime-safe`: current runtime path already returns bounded errors.
- `test-only`: only reachable from `#[cfg(test)]` code.
- `fixture-only`: helper or test-fixture code, not part of operator flow.
- `follow-up`: adjacent risk recorded but not in the fused blocker set.

## Findings

| File | Evidence | Classification | Closure |
| --- | --- | --- | --- |
| `services/wallet_service.rs` seed reveal/export flows | `show_seed_phrase(...)`, `build_wallet_export_pack(...)`, `load_wallet(...)`, and `unlock_wallet_in_memory(...)` return `WalletResult` and already map blocking/task failures into bounded `WalletError` values | `runtime-safe` | kept on typed errors; no live `expect()` or `unwrap()` remains in these paths |
| `services/wallet_service.rs` dense `expect()` block around the high 5500+ range | all visible calls sit under test helpers and fixture builders, not public runtime methods | `fixture-only` | explicitly left unchanged |
| `services/chain_service.rs` `unwrap()` lines in the 260+ range | all visible uses are inside the `#[cfg(test)]` module | `test-only` | stale crash claim downgraded with current-tree evidence |
| `services/wallet_paths.rs` invalid chain config path | `resolve_wallet_chain_type()` previously panicked on invalid `Z00Z_WALLET_CHAIN` / config chain values, and that resolver fed wallet-service and RPC address derivation paths | `runtime-fixed` | new `resolve_wallet_chain_type_checked()` returns typed `WalletError::InvalidConfig`; runtime call sites now fail closed |

## Runtime Boundary Notes

📌 The strongest live panic risk in this wave was not inside
`services/chain_service.rs`. It was the config boundary in
`services/wallet_paths.rs`, because wallet-service and RPC code used that value
to derive addresses and chain ids.

📌 `services/chain_service.rs` no longer carries an operator-reachable
`await.unwrap()` shape in the current tree. The remaining unwraps are test
assertions only.

## Closure Status

📌 Gate C for `PH29-PANIC` is satisfied when the invalid-chain runtime path now
returns a typed failure and the remaining panic sites are classified as
non-runtime with file-accurate evidence.
