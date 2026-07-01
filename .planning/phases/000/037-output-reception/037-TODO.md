# 037-TODO

## 🎯 Mandatory Implementation Tasks

This file is the execution backlog for Phase 037. Every item below is written
as a mandatory implementation or documentation task. Do not re-open the old
question/recommendation loop while executing this backlog.

### Task 0. Freeze the implemented Phase 037 baseline before extending it

**Objective:** Turn the live receive baseline into an explicit execution
constraint so Phase 037 does not accidentally re-open implemented surfaces as
greenfield work.

**Mandatory steps:**

1. Preserve [`WalletService::recv_range(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
  as the current canonical request-aware range-receive authority.
2. Preserve [`StealthOutputScanner`](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
  as the current canonical ownership-detection core.
3. Preserve [`recv_route(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)
  as the explicit receive-to-persist gate.
4. Preserve wallet-native claimed-asset persistence and restore behavior as the
  current baseline until an explicit unification change is implemented.
5. Treat request-aware receive and `tag16` request helpers as already-existing
  implementation surfaces, not as speculative design ideas.
6. Treat [`ScanStatePayload`](../../../crates/z00z_wallets/src/core/storage/scan_storage.rs)
  resume/load/save behavior as implemented baseline, not as new Phase 037
  scope.
7. Preserve [`WalletService::receiver_keys(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs)
  as the canonical live receiver-key derivation boundary for service and RPC
  receive flows.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [wallet_service_actions_reachability.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)
- [wallet_service_actions_receiver.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs)
- [wallet_service_store_load_restore.rs](../../../crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs)
- [scan_storage.rs](../../../crates/z00z_wallets/src/core/storage/scan_storage.rs)

**Acceptance checks:**

- Phase 037 docs explicitly mark this baseline as implemented.
- No Phase 037 task text reopens these surfaces as missing greenfield work.

### Task 1. Keep `WalletService::recv_range(...)` as the canonical receive path

**Objective:** Preserve one canonical range-receive authority and remove any
ambiguity that `ScanEngineImpl` is already equivalent.

**Mandatory steps:**

1. Treat [`WalletService::recv_range(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
  as the canonical implemented request-aware receive lane.
2. Update the Phase 037 receive documentation so all ownership tables, flow
  diagrams, and module references point to the live service path.
3. Mark [`WalletService::receive_asset(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
  and the outward RPC `wallet.asset.receive_asset` path as compatibility-only
  single-asset lanes, not equivalent privacy lanes.
4. Do not describe [`ScanEngineImpl`](../../../crates/z00z_wallets/src/core/chain/scan_engine_impl.rs)
  as current functionality until it stops returning not-implemented errors.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [scan_engine_impl.rs](../../../crates/z00z_wallets/src/core/chain/scan_engine_impl.rs)

**Acceptance checks:**

- Phase docs name `recv_range(...)` as canonical.
- No Phase 037 text claims `ScanEngineImpl` parity unless parity is implemented.

### Task 2. Materialize request-bound `Tag16Context` explicitly

**Objective:** Enforce that strict tag-only scanning requires concrete
`Tag16Context` registration, not only `add_request(...)`.

**Mandatory steps:**

1. Keep `add_request(...)` as request liveness metadata only.
2. Add or document a dedicated context-materialization step before any strict
  tag-only scan path is allowed.
3. Ensure any future tag-only entry point rejects request-bound flows that do
  not provide concrete `Tag16Context` values.
4. Update the Phase 037 doc to say that tag16 prefiltering is request-aware
  only after explicit context registration.

**Target files:**

- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [types.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner/types.rs)

**Reference snippet:**

```rust
let mut scanner = StealthOutputScanner::from_keys(&recv_keys);
scanner.add_request(&request);
scanner.add_tag_context(tag16, Tag16Context { k_dh, req_id: Some(request.req_id) });
```

**Acceptance checks:**

- Strict tag-only path requires `add_tag_context(...)`.
- Docs no longer imply that `add_request(...)` alone is enough.

### Task 3. Keep proof verification downstream of ownership detection

**Objective:** Prevent receive detection from silently turning into a full proof
verifier.

**Mandatory steps:**

1. Keep `scan_report(...)`, `scan_leaf(...)`, and range scanning focused on
  ownership detection and receive classification only.
2. Do not add range-proof verification into the canonical receive detector.
3. Update the Phase 037 receive documentation so `Detected` means ownership
  detection, not final import or spend validation.
4. If new entry points are added, route proof verification to downstream import,
  tx-validation, or explicit verification boundaries.

**Target files:**

- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [output.rs](../../../crates/z00z_wallets/src/core/stealth/output.rs)

**Acceptance checks:**

- No new receive-path code verifies range proofs inline.
- Docs distinguish detection from downstream proof validation.

### Task 4. Preserve explicit `ReceiveNext::PersistClaim` gating

**Objective:** Keep persistence as an explicit decision, not an automatic side
effect of detection.

**Mandatory steps:**

1. Keep [`recv_route(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)
  as the frozen receive-to-persist gate.
2. Do not auto-persist from detection-only results.
3. Ensure all new receive flows call `recv_route(...)` or an equivalent wrapper
  instead of writing claimed assets directly.
4. Keep the Phase 037 doc explicit that `ReportOnly` and `PersistClaim` remain
  separate outcomes.
5. Preserve the existing compatibility scrub performed by
  [`recv_claim_asset(...)`](../../../crates/z00z_wallets/src/services/wallet_service_store_support.rs)
  when runtime scan output carries stealth-bound signature fields that must not
  cross the canonical claimed-asset persistence boundary unchanged.

**Target files:**

- [wallet_service_actions_reachability.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)
- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [wallet_service_store_support.rs](../../../crates/z00z_wallets/src/services/wallet_service_store_support.rs)
- [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/test_wallet_service_suite.rs)

**Reference snippet:**

```rust
self.recv_route(wallet_id, asset, ReceiveNext::PersistClaim).await?;
```

**Acceptance checks:**

- Detection-only paths do not mutate claimed storage.
- `ReportOnly` and `PersistClaim` remain separately testable.
- The persistence boundary does not silently drop the existing compatibility
  scrub rule for structurally valid claimed assets.

### Task 5. Re-baseline Phase 037 architecture documentation to live code

**Objective:** Remove stale trait-stack and module-path claims from the phase
document.

**Mandatory steps:**

1. Replace `receiver::scanner`-style ownership language with the current live
  code ownership:
  `WalletService::recv_range(...)`, `core::address::leaf_scan`, and
  `core::address::StealthOutputScanner`.
2. Update module-path tables, diagrams, and implementation maps in the Phase
  037 receive documentation.
3. Mark trait-based scanner variants as superseded or future-only unless code is
  added in this phase.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [scan_engine_impl.rs](../../../crates/z00z_wallets/src/core/chain/scan_engine_impl.rs)

**Acceptance checks:**

- The document no longer presents stale module trees as current implementation.
- Every implementation path named in the doc resolves to a live file.

### Task 6. Implement inbox-assisted receive only at the service or adapter boundary

**Objective:** Add inbox-hint support without creating a second ownership logic
stack, but only after a concrete live inbox or hint source exists and Task 15
closes deterministic ordered non-expired candidate selection.

**Selected branch for Phase 037:** explicit defer only. Keep inbox-assisted
receive future-only in this phase because no concrete live inbox or hint source
is verified and Task 15 remains open.

**Branch gate:** If no concrete live inbox or hint source can be verified in
the codebase, or if Task 15 remains open, keep inbox-assisted receive
future-only in Phase 037 and do not create a speculative receive-inbox module.

**Mandatory steps:**

1. Verify a concrete live inbox or hint source exists and that Task 15 has
  closed deterministic ordered non-expired candidate selection before taking
  the implementation branch.
2. Add inbox-hint ingestion as candidate-selection metadata only.
3. Keep final ownership confirmation in the canonical detector path.
4. Place inbox plumbing in a new sibling service or adapter module, not inside
  `core/address/*` detection primitives.
5. Wire inbox-assisted receive into the existing `recv_range(...)` orchestration
  instead of creating a second scan/detect/store pipeline.
6. Keep inbox semantics notify-only: it may supply locator or candidate hints,
  but it must not become an ownership authority or a stable receiver-identity
  registry.
7. If helper routing is introduced, derive it from request-bound context rather
  than wallet-global or stable receiver identifiers.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- New sibling service or adapter module under
  [crates/z00z_wallets/src/services/](../../../crates/z00z_wallets/src/services)

**Tests to add if the implementation branch opens:**

- hint hit
- hint miss
- false-positive hint
- proof that inbox hints never bypass canonical ownership detection
- proof that helper routing remains request-bound and does not depend on stable
  receiver identity

### Task 7. Resolve `ScanEngineImpl` by either de-scoping it or making it a thin delegate

**Objective:** Eliminate the current half-present state where the phase doc
describes a scan engine that still returns not-implemented errors.

**Selected branch for Phase 037:** explicit de-scope only. Keep
`ScanEngineImpl` proposed-only and stub-only in this phase.

**Mandatory steps:**

1. Keep `ScanEngineImpl` stub-only and proposed-only in Phase 037 docs and
  touched code.
1. Remove or avoid any wording that implies current parity with
  `WalletService::recv_range(...)`.
1. Do not introduce a live scan-engine delegate in this phase.
1. Do not duplicate ownership detection, rejection mapping, cursor
  persistence, or claimed-asset persistence logic inside the scan engine.

**Target files:**

- [scan_engine_impl.rs](../../../crates/z00z_wallets/src/core/chain/scan_engine_impl.rs)
- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)

**Tests to add if a future phase promotes the delegate branch:**

- parity with `recv_range(...)`
- cursor resume parity
- claimed-asset persistence parity

**Acceptance checks:**

- `ScanEngineImpl` remains documented as stub-only and proposed-only.
- No touched Phase 037 doc claims live scan-engine parity.
- No new scan-engine delegate implementation lands in this phase.

### Task 8. Decide whether `OptimizedScanner` stays optional or becomes canonical

**Objective:** Remove the current ambiguous state where parallel scanning exists
but is not the documented primary path.

**Mandatory steps:**

1. Choose one path and document it explicitly: keep `OptimizedScanner`
  optional or integrate it behind an explicit strategy or threshold in the
  canonical receive flow.
1. If the canonical-integration path is chosen, preserve identical `ScanRangeOut`, cursor semantics,
  rejection semantics, and persistence behavior.
1. Do not introduce separate ownership or persistence implementations while
  adding parallel execution.

**Target files:**

- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [optimized_scanner.rs](../../../crates/z00z_wallets/src/core/address/optimized_scanner.rs)
- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)

**Tests to add if promoted to canonical:**

- deterministic output parity
- stats parity
- DoS policy parity

### Task 9. Add only the missing tests that cover unresolved Phase 037 gaps

**Objective:** Expand coverage where functionality is missing, without cloning
tests that already exist.

**Planning anchors:** Execute this task against the approved planning package in
[037-TEST-SPEC.md](./037-TEST-SPEC.md),
[037-TESTS-TASKS.md](./037-TESTS-TASKS.md), and
[037-TEST-PLAN.md](./037-TEST-PLAN.md). Treat those files as the canonical
test-scope contract before adding or extending any Rust test.

**Mandatory steps:**

1. Reuse existing restart, persistence, and route-gate coverage in
  [test_wallet_service_suite.rs](../../../crates/z00z_wallets/src/services/test_wallet_service_suite.rs).
2. Reuse existing outward receive coverage in
  [test_asset_impl_suite.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs)
  for exact `asset_id` semantics, public status mapping, and adapter/service parity.
3. Add new tests only for new inbox paths, scan-engine parity, or promoted
  canonical parallel scanning.
4. Prefer extending existing wallet-service tests for service-boundary behavior.
5. Place new focused integration tests under
  [crates/z00z_wallets/tests/](../../../crates/z00z_wallets/tests) when a new
  entry point is introduced.

**Do not duplicate:**

- restart/resume cursor tests
- claimed-asset restart tests
- `ReceiveNext::PersistClaim` gate tests
- tag16 cache or request-binding tests that already exist

### Task 10. Enforce crypto and security guardrails on every new receive path

**Objective:** Preserve the security invariants already encoded in the current
receive and stealth-output flow.

**Mandatory steps:**

1. Preserve constant-time comparison for tag and owner-tag checks.
2. Treat tag16 and inbox hits as prefilters only, never as ownership proof.
3. Keep proof verification downstream of ownership detection.
4. Keep `ReceiveNext::PersistClaim` as the explicit persistence gate.
5. Route every new receive flow through the existing canonical detection core
  instead of creating parallel ownership logic.

**Target files:**

- [output.rs](../../../crates/z00z_wallets/src/core/stealth/output.rs)
- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [wallet_service_actions_reachability.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)

**Reference snippet:**

```rust
pub fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
   left.ct_eq(right).into()
}
```

### Task 11. Resolve the Section 6.4 storage-model drift without creating a third persistence layer

**Objective:** Converge the planned storage text onto one canonical persistence
target.

**Mandatory steps:**

1. Reclassify Section 6.4 into `implemented`, `future-unification`, and `stale`
  parts.
1. Choose one canonical receive persistence target: keep wallet-native
  claimed-asset persistence or migrate receive persistence onto the existing
  [`core::storage::AssetStorage`](../../../crates/z00z_wallets/src/core/storage/asset_storage.rs)
  abstraction.
1. If unification is chosen, adapt existing asset-storage types and boundaries
  instead of adding a new `receiver::storage` tree.
1. Update the Phase 037 receive documentation so it no longer presents the
  full `SpendableAsset`/SQLite design as mandatory current Phase 037 work.
1. Preserve the current compatibility scrub behavior at the persistence boundary
  unless an explicit replacement is implemented and tested during unification.

**Target files:**

- [asset_storage.rs](../../../crates/z00z_wallets/src/core/storage/asset_storage.rs)
- [asset_storage_impl.rs](../../../crates/z00z_wallets/src/core/storage/asset_storage_impl.rs)
- [wallet_service_actions_reachability.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)
- [wallet_service_store_load_restore.rs](../../../crates/z00z_wallets/src/services/wallet_service_store_load_restore.rs)

**Tests to add if storage is unified:**

- balance parity
- claimed-asset visibility parity
- spent-state transition parity
- restart/restore parity

### Task 12. Resolve the Section 6.5 reception-API drift with a thin facade or documentation rebase

**Objective:** Prevent the phase doc from describing a second orchestration API
as if it already exists.

**Mandatory steps:**

1. Mark the documented `Receiver`, `ReceptionConfig`, `ReceptionResult`, and
  callback/event API as proposed-only until code exists.
2. If an explicit high-level facade is still needed, implement it as a thin
  wrapper over [`WalletService::recv_range(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs),
  [`WalletService::scan_asset_report(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs),
  outward `wallet.asset.receive_asset` mapping, and claimed-asset queries.
3. Keep progress tracking, cursor persistence, and claim persistence delegated to
  existing wallet-service boundaries.
4. Keep [`WalletService::receive_asset(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
  documented as a compatibility-only reachability surface, not as the canonical
  privacy receive API.
5. Do not build a second scan/detect/store orchestration stack.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [wallet_service_actions_reachability.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_reachability.rs)
- [asset_impl_server.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server.rs)
- [asset_impl_server_transfer.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs)
- [test_asset_impl_suite.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs)

**Reference snippet:**

```rust
// Desired shape if a facade is added:
// ReceiverFacade::scan_range(...)   -> WalletService::recv_range(...)
// ReceiverFacade::scan_asset(...)   -> WalletService::scan_asset_report(...)
// wallet.asset.receive_asset(...)   -> AssetRpcImpl::receive_asset_impl(...)
// ReceiverFacade::claimed(...)      -> WalletService::list_claimed_assets(...)
```

**Tests to add if facade is introduced:**

- output-status parity
- exact `asset_id` lookup parity
- cursor progression parity
- partial-failure reporting parity
- claimed-asset visibility parity

### Task 13. Resolve the scanner-config and DoS-policy drift against the live scanner surface

**Objective:** Prevent Phase 037 from describing a public scanner configuration
stack that does not match the implemented receive primitives.

**Mandatory steps:**

1. Decide whether Phase 037 keeps a public scan-configuration facade or rebases
  documentation onto the current live knobs:
  `max_ckpt`, `DoSMitigation`, explicit request registration, and
  `background_scan_strategy()`.
2. If a facade is kept, implement it as a thin adapter over
  [`StealthOutputScanner`](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
  and [`OptimizedScanner`](../../../crates/z00z_wallets/src/core/address/optimized_scanner.rs)
  instead of creating a new scanner stack.
3. Either route canonical range scanning through an explicit DoS policy or mark
  the richer `DoSMitigationConfig` and `ScanConfig` design as future-only.
4. Keep tag16 optimization, inbox hints, and parallel scanning documented as
  strategy inputs, not as separate ownership engines.

**Target files:**

- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [optimized_scanner.rs](../../../crates/z00z_wallets/src/core/address/optimized_scanner.rs)
- [types_range.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner/types_range.rs)

**Reference snippet:**

```rust
let mitigation = DoSMitigation::new(1_000, 100, 250);
let outputs = scanner.scan_with_dos_protection(&checkpoint.leaves, &mitigation);
```

**Acceptance checks:**

- Phase docs do not present `ScanConfig` or `DoSMitigationConfig` as current
  implementation unless code exists.
- Canonical receive ownership names one explicit DoS-policy surface.
- Parallel and tag16 knobs are documented as adapters over the same detection
  core.

### Task 14. Translate surviving ECC ideas into live surfaces and quarantine stale names

**Objective:** Preserve useful historical receive ideas only after translating
them into the current repository architecture, while blocking stale type and
module names from re-entering canonical Phase 037 truth.

**Mandatory steps:**

1. Keep the `created_delta` plus DA-fetch mental model only in terms that map to
  current checkpoint and data-availability surfaces.
2. Keep inbox ideas constrained to notify-only candidate selection and locator
  hints.
3. Treat request-bound helper routing as the only acceptable direction for any
  future assisted receive flow.
4. Do not reintroduce stale names such as `receiver::scanner`,
  `receiver::storage`, `Receiver`, `ReceptionResult`, `OutputScanner`,
  `FullScanner`, `HybridScanner`, `ScanConfig`, or `DoSMitigationConfig` as
  current implementation unless code is added in this phase.
5. When historical draft ideas are retained in docs, translate them into the
  live canonical surfaces before turning them into backlog text.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [stealth_scanner.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner.rs)
- [scan_engine_impl.rs](../../../crates/z00z_wallets/src/core/chain/scan_engine_impl.rs)
- [asset_storage.rs](../../../crates/z00z_wallets/src/core/storage/asset_storage.rs)

**Acceptance checks:**

- Phase docs retain useful historical ideas only in live repository terms.
- Stale ECC names are either removed or explicitly marked proposed-only.
- No Phase 037 text silently upgrades speculative draft signatures into current
  API truth.

### Task 15. Make request-candidate ordering deterministic and expiry-aware

**Objective:** Remove the hidden nondeterminism where request-aware scanning can
iterate active `req_id` values in unspecified order.

**Mandatory steps:**

1. Replace or wrap the current active-request iteration so canonical
  request-aware scanning uses a stable candidate order.
2. Filter expired requests before candidate iteration.
3. Keep the `ReceiverCard` / `req_id = None` fallback explicit and last in the
  candidate order whenever the direct-scan path depends on multi-request
  iteration.
4. Rebase the Phase 037 receive documentation onto the implemented ordering
  policy instead of leaving the Tari-style ordering as an unverified claim.

**Target files:**

- [types_tag_cache.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner/types_tag_cache.rs)
- [stealth_scan_support.rs](../../../crates/z00z_wallets/src/core/address/stealth_scan_support.rs)
- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)

**Reference snippet:**

```rust
// Required end-state shape:
// active requests -> stable ordered candidates -> non-expired req_ids -> None fallback last
for req_id in ordered_req_candidates.iter() {
    // first match wins
}
```

**Tests to add:**

- expired request ids are skipped
- request iteration order is deterministic
- `req_id = None` fallback is tried last
- first successful candidate terminates iteration

**Acceptance checks:**

- Canonical request-aware scanning no longer depends on `HashSet` iteration
  order.
- Docs describe the same ordering and expiry policy that code enforces.

### Task 16. Add receive-path observability only for actionable rejections

**Objective:** Preserve the stable receive taxonomy while making actionable
failures measurable and noisy `NotMine` outcomes silent.

**Mandatory steps:**

1. Keep high-volume `NotMine` outcomes out of counters and alerts.
2. Map stable receive rejections and runtime failures onto explicit log codes,
  counters, or both using repository-standard metrics surfaces.
3. Rebase the metrics section of the Phase 037 receive documentation onto the
  current `ReceiveReject` / `ReceiveReport` vocabulary unless a richer
  `DetectionError` tree is implemented in code.
4. Add operator-facing severity guidance for at least `InvalidInput`,
  `InvalidProof`, and `RuntimeFail` paths.

**Target files:**

- [types_receive.rs](../../../crates/z00z_wallets/src/core/address/stealth_scanner/types_receive.rs)
- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [asset_impl_server_transfer.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs)

**Acceptance checks:**

- Actionable receive failures have stable log or metric hooks.
- `NotMine` remains non-alerting.
- Phase docs do not promise a richer rejection-metrics stack than the code
  actually exposes.

### Task 17. Finish the canonical progress, partial-success, and callback contract

**Objective:** Make progress tracking and callback semantics converge on one
implemented receive surface instead of split proposed APIs.

**Mandatory steps:**

1. Keep resume persistence anchored to the existing
  `ScanStatePayload` load/save path used by
  [`recv_range(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs).
2. Decide whether Phase 037 exposes progress and callback behavior only through
  current `ScanRangeOut` / `ScanRangeStat` service-boundary reporting while
  `ScanEngine` and `ScanStorage` remain proposed-only, or whether a thin facade
  is added on top.
3. If callbacks remain in scope, wire them to canonical cursor/stat progression
  instead of inventing a second `ReceptionResult` event pipeline.
4. Rebase the error-handling and partial-success language in the Phase 037
  receive documentation onto the live `ScanRangeOut`, `ReceiveReport`, and
  `WalletResult` contract unless a richer facade is implemented.

**Target files:**

- [wallet_service_actions_receive.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs)
- [scan_engine.rs](../../../crates/z00z_wallets/src/core/chain/scan_engine.rs)
- [scan_engine_impl.rs](../../../crates/z00z_wallets/src/core/chain/scan_engine_impl.rs)
- [scan_storage.rs](../../../crates/z00z_wallets/src/core/storage/scan_storage.rs)

**Tests to add if callbacks or facade progress stay in scope:**

- cursor resume remains monotonic across chunked scans
- reported progress matches persisted cursor advancement
- callback invocation order matches checkpoint progression
- partial-success reporting stays aligned with canonical receive statuses

**Acceptance checks:**

- Phase docs point to one implemented progress/callback contract.
- Cursor persistence, progress reporting, and partial-success semantics no
  longer rely on a proposed-only `Receiver` API.

### Task 18. Rebase the public RPC receive surface onto the refactored service boundary

**Objective:** Keep the user-facing single-asset receive API aligned with the
post-refactor service split and exact `asset_id` semantics.

**Mandatory steps:**

1. Treat outward `wallet.asset.receive_asset` as the current public single-asset
  receive API and document it separately from canonical range receive.
2. Keep [`AssetRpcImpl::receive_asset_impl(...)`](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs)
  mapped onto [`WalletService::scan_asset_report(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receive.rs),
  [`WalletService::receiver_keys(...)`](../../../crates/z00z_wallets/src/services/wallet_service_actions_receiver.rs),
  and stable outward status mapping.
3. Preserve exact `asset_id` lookup semantics and keep definition-id queries
  rejected unless a new public contract is explicitly introduced.
4. Reuse existing adapter tests as the canonical public receive evidence before
  adding new wrapper APIs.

**Target files:**

- [asset_impl_server.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server.rs)
- [asset_impl_server_transfer.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_server_transfer.rs)
- [asset_impl_support_assets.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_support_assets.rs)
- [wallet_dispatcher_wiring_register.rs](../../../crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs)
- [test_asset_impl_suite.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs)

**Tests to reuse or extend:**

- `asset_receive_api_sync`
- `asset_receive_path_parity`
- `asset_receive_exact_asset_id_survives_definition_collision`
- `asset_receive_rejects_definition_id_query`

**Acceptance checks:**

- Phase docs name the RPC adapter as the public single-asset receive surface.
- Docs do not present `WalletService::receive_asset(...)` as the canonical
  privacy lane.
- Exact `asset_id` semantics remain covered by live adapter tests.

### Task 19. Quarantine orphaned duplicate receive surfaces that are no longer wired

**Objective:** Prevent unused duplicate files from reintroducing stale receive
ownership claims after the refactor.

**Mandatory steps:**

1. Treat [`wallet_service_actions_runtime.rs`](../../../crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs)
  as an orphaned duplicate surface until it is either wired into
  [`wallet_service_actions.rs`](../../../crates/z00z_wallets/src/services/wallet_service_actions.rs)
  or removed.
2. Treat [`asset_impl_tests.rs`](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs)
  as an orphaned duplicate test surface while
  [`asset_impl.rs`](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs)
  continues to bind tests through
  [`test_asset_impl_suite.rs`](../../../crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs).
3. Rebase Phase 037 docs and future task mapping onto wired live seams only,
  and explicitly mark orphaned duplicates as cleanup candidates rather than
  canonical implementation truth.
4. If any orphaned duplicate is kept intentionally, add a repository-local
  note explaining why it remains and how it must not be used for ownership or
  test-surface documentation.

**Target files:**

- [wallet_service_actions.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions.rs)
- [wallet_service_actions_runtime.rs](../../../crates/z00z_wallets/src/services/wallet_service_actions_runtime.rs)
- [asset_impl.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs)
- [asset_impl_tests.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/asset_impl_tests.rs)
- [test_asset_impl_suite.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/test_asset_impl_suite.rs)

**Acceptance checks:**

- Phase 037 docs and TODO links do not use orphaned duplicate files as the
  canonical receive boundary.
- Any intentionally retained duplicate file is marked non-canonical and has an
  explicit keep/remove decision.

## ✅ Completion Rule

Phase 037 is not complete until the implementation and documentation both agree
on the same canonical receive path, persistence boundary, and test surface.
Any future test implementation must stay aligned with
[037-TEST-SPEC.md](./037-TEST-SPEC.md),
[037-TESTS-TASKS.md](./037-TESTS-TASKS.md), and
[037-TEST-PLAN.md](./037-TEST-PLAN.md) unless those planning artifacts are
updated first.
