# Phase 037 Receive Architecture

## 🎯 Purpose

This document freezes the implemented Phase 037 receive baseline before any
conditional extensions are attempted. It records the live receive lane,
compatibility-only surfaces, and explicitly non-parity seams so later plans do
not reopen the baseline as greenfield work.

## ✅ Implemented Baseline

### 🔑 Canonical Receive Lane

The canonical Phase 037 receive lane is `WalletService::recv_range(...)` in
`crates/z00z_wallets/src/services/wallet_service_actions_receive.rs`.

Today that lane already does all of the following:

- derives live receiver keys through `live_receiver_keys(...)`
- loads the resumable wallet-native cursor through `load_scan_state(...)`
- constructs `StealthOutputScanner` from those live keys
- registers request-aware liveness metadata through `add_request(...)`
- scans checkpoint chunks via `scan_range(...)`, which uses the live
  `StealthOutputScanner::scan_leaf(...)` detector while preserving
  `core::address::leaf_scan` as the canonical sibling full-leaf ownership
  contract
- computes the in-range replay start through `recv_range_start(...)`
- replays only the in-scope leaves through `claim_scan_hits(...)` before
  persistence is attempted
- converts detected leaves into claim candidates with `recv_claim_asset(...)`
- persists claimed hits only through
  `recv_route(..., ReceiveNext::PersistClaim)`
- persists the wallet-native scan cursor through `ScanStatePayload`

### 🗺️ Live Module Map

- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` owns
  `recv_range(...)`, `claim_scan_hits(...)`, and the canonical service-side
  receive orchestration.
- `crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs`
  owns `recv_route(...)` and the explicit `ReportOnly` versus
  `PersistClaim` split.
- `crates/z00z_wallets/src/services/wallet_service_store_support.rs` owns
  `recv_claim_asset(...)`, including the compatibility scrub that removes
  stealth-bound owner fields before claimed persistence.
- `crates/z00z_wallets/src/core/address/leaf_scan.rs` owns the canonical
  full-leaf detection contract exported for sibling receive surfaces.
- `crates/z00z_wallets/src/core/address/stealth_scanner.rs` owns the
  wallet-runtime detector orchestration and request-aware scan entry points.
- `crates/z00z_wallets/src/core/address/optimized_scanner.rs` owns the
  optional batching wrapper over the same canonical detector; it does not own
  a separate receive authority.
- `crates/z00z_wallets/src/core/chain/scan_engine_impl.rs` remains a stub-only
  non-parity seam and is not part of the implemented receive lane.

### ⚙️ Live Scanner Knobs

The scanner surface is driven by implemented knobs, not by a separate public
configuration stack:

- `max_ckpt` bounds how much checkpoint work a range scan will consume per
  pass.
- `DoSMitigation` shapes the live scan policy used by the detector surface.
- `background_scan_strategy()` selects the current execution strategy from
  cache pressure and tag-context state.
- `add_request(...)` remains liveness metadata only; strict tag-only receive
  still requires concrete `Tag16Context` values registered through
  `add_tag_context(...)`.
- request-aware candidate materialization is deterministic and expiry-aware:
  the active request set is ordered, expired requests are pruned before
  registration, and the `req_id = None` fallback stays last.
- tag16, inbox, and parallel scan ideas are strategy inputs over the same
  canonical detector, not separate ownership engines.

### 🔄 Receive Flow Ledger

The implemented receive flow is:

`recv_range(...)`
-> `live_receiver_keys(...)`
-> `load_scan_state(...)`
-> `StealthOutputScanner::from_keys(...)`
-> `scanner.add_request(...)`
-> `scanner.scan_range(...)`
-> `StealthOutputScanner::scan_leaf(...)`
-> `recv_range_start(...)`
-> `claim_scan_hits(...)`
-> `StealthOutputScanner::scan_leaf(...)` replay for in-range persistence gating
-> `recv_claim_asset(...)`
-> `recv_route(..., ReceiveNext::PersistClaim)`
-> `save_scan_state(...)`

This is already the live request-aware receive path. Later plans may tighten
ordering, hint usage, and observability around it, but they must extend this
lane rather than replace it. When a future strict tag-only fast path is used,
it must also materialize concrete `add_tag_context(...)` entries; active
request registration alone is not sufficient for strict tag-only ownership
claims.

Receive observability stays intentionally shallow at this boundary. The live
adapter treats `ReceiveReject::NotMine` as non-alerting, while
`InvalidInput`, `InvalidProof`, and `RuntimeFail` remain actionable receive
failures that may reach warning-level logs. The phase does not introduce a
separate detection-error tree for that split; it keeps the severity guidance
in the reject vocabulary and the RPC adapter log site.

The deterministic request-candidate policy is now implemented in the live
receive lane: request-bearing candidates are evaluated in a stable order,
expired requests are pruned before scanning, and the generic fallback stays
last. Phase 037 still does not expose a concrete live inbox or hint source for
this lane, so inbox-assisted receive remains future-only in this phase and no
`wallet_service_actions_receive_inbox.rs` sibling module is part of the live
implementation.

### 🧾 Compatibility-Only Surfaces

The following surfaces remain compatibility-only in Phase 037 and must not be
described as the canonical privacy lane:

- `WalletService::scan_asset_report(...)`
- `WalletService::receive_asset(...)`
- outward `wallet.asset.receive_asset` RPC handling, which remains a
  compatibility-only single-asset lane even though it reconstructs ownership
  through `AssetRpcImpl::receive_asset_impl(...)`, `scan_asset_report(...)`,
  and `receiver_keys(...)` instead of routing through the placeholder
  `WalletService::receive_asset(...)` facade

These surfaces may reuse live receiver keys or detector helpers, but they do
not define the canonical receive architecture.

## ⚠️ Explicitly Non-Parity Seams

### 🛑 Scan Engine

`crates/z00z_wallets/src/core/chain/scan_engine_impl.rs` remains a stub-only,
proposed scan-engine surface in Phase 037. It returns not-implemented errors
for `scan(...)` and `scan_range(...)` and does not yet delegate to the
canonical `recv_range(...)` lane.

Until a future phase lands a thin delegate over the live receive path,
`ScanEngineImpl` must be treated as non-parity and proposed-only.

### ♻️ Optional Batching Wrapper

`crates/z00z_wallets/src/core/address/optimized_scanner.rs` remains an
optional batching wrapper over the same canonical `StealthOutputScanner`
detector. It may change execution strategy, but it does not own receiver
crypto validation, claimed persistence, or a second receive pipeline. The
canonical Phase 037 receive lane remains `WalletService::recv_range(...)`.

### 🧱 Non-Canonical Duplicate Surfaces

- `crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs` remains
  a non-canonical duplicate. `wallet_service_actions.rs` does not include it in
  the live `WalletService` include stack, so it must not be treated as receive
  ownership authority unless a future phase explicitly wires it in.
- `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs` remains a
  dead duplicate. `asset_impl.rs` binds the canonical test module from
  `test_asset_impl_suite.rs` via `#[path = "test_asset_impl_suite.rs"] mod
  asset_impl_tests;`, so the standalone `asset_impl_tests.rs` file must not be
  treated as the bound test-surface authority.

### 💾 Storage Model (Section 6.4)

Phase 037 keeps the storage story split into the following buckets:

#### ✅ Implemented

- wallet-native claimed-asset persistence remains the current canonical
  receive target
- `recv_route(..., ReceiveNext::PersistClaim)` remains the only persistence
  gate for claimed receive output
- `recv_claim_asset(...)` keeps the compatibility scrub that removes
  stealth-bound owner fields before the claim crosses the persistence
  boundary
- `wallet_service_store_load_restore.rs` restores claimed assets from wallet
  snapshots and deduplicates them on load, but it does not define a third
  receive persistence layer

#### ♻️ Future-Unification

- `crates/z00z_wallets/src/core/storage/asset_storage.rs` remains a
  future-unification seam only
- if later phases unify receive persistence there, they must adapt the
  existing `AssetStorage` boundaries rather than inventing a new receive tree
- unification does not change the current rule that Phase 037 has exactly one
  canonical claimed-persistence target

#### 🕰️ Stale

- the older `SpendableAsset`/SQLite storage story is historical vocabulary
  only
- no stale storage sketch should be read as mandatory current Phase 037 work
- any future rewrite must be backed by code and tests before it replaces the
  implemented wallet-native target

## 📌 Architectural Boundaries

- ownership detection stays in `StealthOutputScanner`
- `OptimizedScanner` stays optional batching over that same detector and never
  becomes the canonical receive authority in Phase 037
- canonical full-leaf ownership checks stay rooted in `core::address::leaf_scan`
- strict tag-only prefiltering requires explicit `Tag16Context` registration;
  `add_request(...)` alone remains liveness metadata only
- claim persistence stays behind `recv_route(...)`
- `recv_claim_asset(...)` performs the compatibility scrub before any claimed
  asset crosses the persistence boundary
- scan cursor persistence stays in `ScanStatePayload`
- range progress reporting stays in `ScanRangeOut` and `ScanRangeStat`; the
  cursor lives in `ScanRangeStat.cursor` and owned outputs stay in
  `ScanRangeOut.outputs`
- `ReceiveReject::NotMine` is non-alerting; `InvalidInput`, `InvalidProof`,
  and `RuntimeFail` remain actionable warning-level receive failures
- request metadata may influence candidate selection but does not create a
  second ownership authority by itself
- inbox hints remain future-only until a real live source exists; the ordered
  request-candidate policy is already deterministic and expiry-aware
- receive detection and classification stay separate from downstream import,
  tx validation, and proof-verification boundaries
- compatibility surfaces must remain explicitly compatibility-only unless a
  later plan promotes them with code and validation

## ⏳ Superseded Or Future-Only Variants

No trait-based scanner stack is live Phase 037 implementation today. Historical
or planning-only names such as `Receiver`, `ReceptionConfig`,
`ReceptionResult`, callback or event receive APIs, `ScanConfig`,
`DoSMitigationConfig`, `receiver::scanner`, `receiver::storage`,
`OutputScanner`, `FullScanner`, and `HybridScanner` are superseded vocabulary
unless later plans add real code behind those names. The live authority chain
remains the module map above.

## ⏭️ Follow-On Work

Later Phase 037 plans may:

- add service-boundary hint plumbing only if a real inbox source exists
- expand tests around detector boundaries and persistence gates

They must not rewrite the canonical baseline captured above.
