# Codebase Concerns

📌 **Analysis Date:** 2026-05-06

## Tech Debt

⚠️ **Wallet Runtime Is Split Between Real Persistence and Residual Phase-1 Stub Surfaces:**

- Issue: `z00z_wallets` still carries Phase-1 stub contracts and wording in public-facing and service-facing surfaces while also containing a large amount of real wallet logic. This creates planning ambiguity about which APIs are authoritative and which are transitional.
- Files: `crates/z00z_wallets/src/lib.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/src/services/chain_service.rs`, `crates/z00z_wallets/src/services/backup_service.rs`, `crates/z00z_wallets/src/services/network_service.rs`, `crates/z00z_wallets/src/services/storage_service.rs`, `crates/z00z_wallets/src/services/tx_service.rs`
- Impact: Future phases can easily extend the wrong seam, preserve placeholder behavior accidentally, or add new logic on top of temporary contracts instead of collapsing them.
- Fix approach: Define the canonical runtime boundary for each service domain, remove or isolate reachability-only methods, and split stub-only DTO/service behavior behind explicit feature gates or dedicated test-only modules.

⚠️ **Wallet Key Cache Adds a New Concurrency Boundary:**

- Issue: `KeyManagerImpl` now owns an LRU derivation cache plus in-flight derivation tracking in `crates/z00z_wallets/src/key/manager/key_manager_impl_cache.rs`. That logic is guarded by multiple locks, TTL checks, and recovery branches, so it behaves like a core runtime seam rather than a helper.
- Files: `crates/z00z_wallets/src/key/manager/key_manager_impl_cache.rs`, `crates/z00z_wallets/tests/test_stealth_scanner_cache.rs`, `crates/z00z_wallets/tests/test_e2e_req_flow.rs`
- Impact: cache collisions, lock-poison recovery, or stale derivation results could surface as inconsistent wallet key resolution or misrouted request and scan behavior under concurrency.
- Fix approach: Document cache invalidation and flight ownership rules, keep cache and flight helpers isolated behind the key-manager boundary, and preserve the cache-focused tests as the contract baseline.

⚠️ **Monolithic Files Concentrate Too Much Behavior in Single Edit Surfaces:**

- Issue: several core files are already far beyond a safe review and refactor size. Current hotspots include `crates/z00z_wallets/src/db/redb_wallet_store.rs` at 8906 lines, `crates/z00z_wallets/src/services/wallet_service.rs` at 7879 lines, `crates/z00z_simulator/src/scenario_1/stage_4.rs` at 2825 lines, `crates/z00z_core/src/genesis/genesis.rs` at 2029 lines, and `crates/z00z_wallets/src/core/tx/tx_verifier.rs` at 1178 lines.
- Files: `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_simulator/src/scenario_1/stage_4.rs`, `crates/z00z_core/src/genesis/genesis.rs`, `crates/z00z_wallets/src/core/tx/tx_verifier.rs`
- Impact: code review quality drops, regression radius is large, and ownership boundaries are blurred.
- Fix approach: carve these files by responsibility first, not by syntax. Persistence codec/migration, session state, backup flows, scan flows, tx semantic validation, and simulator artifact generation are all candidates for separate modules.

⚠️ **Documentation Drift Is Material in `z00z_wallets`:**

- Issue: the README describes a layout and dependency model that no longer matches the crate. It still references `wallet_worker.rs`, `.temp`, `www/`, and “SQLite Storage”, while the active code centers on `redb`, `sqlx`, `db/`, `wasm/`, `egui_views/`, and large native wallet persistence logic.
- Files: `crates/z00z_wallets/README.md`, `crates/z00z_wallets/Cargo.toml`, `crates/z00z_wallets/src/lib.rs`, `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/src/wasm/mod.rs`
- Impact: planning inputs are misleading, new contributors will search in the wrong places, and architecture decisions appear more settled than they are.
- Fix approach: rewrite the README around the actual module tree and the current RedB `.wlt` persistence model; move historical notes into an explicitly archival document.

⚠️ **Storage Keeps Internal Whitebox and Model Tests Inside the Crate Source Tree:**

- Issue: `z00z_storage` validates critical semantics through internal whitebox helpers and `src/assets/model_tests.rs` rather than only through blackbox integration boundaries.
- Files: `crates/z00z_storage/src/assets/model_tests.rs`, `crates/z00z_storage/src/assets/store_internal/whitebox_crud.rs`, `crates/z00z_storage/src/assets/store_internal/whitebox_help.rs`, `crates/z00z_storage/src/assets/store_internal/whitebox_paths.rs`, `crates/z00z_storage/src/assets/store_internal/whitebox_proofs.rs`, `crates/z00z_storage/src/assets/store_internal/whitebox_state.rs`
- Impact: internal representations become harder to change because tests bind to private structure instead of public guarantees.
- Fix approach: keep a small whitebox layer for algorithmic debugging, but move the canonical contract coverage to `tests/` through public APIs such as `AssetStore`, snapshot builders, and checkpoint stores.

⚠️ **Crypto “Hidden Backend” Claim Does Not Match the Public Surface:**

- Issue: `z00z_crypto` documents backend abstraction, but its public API re-exports many Tari concrete types and services directly.
- Files: `crates/z00z_crypto/src/lib.rs`, `crates/z00z_crypto/Cargo.toml`
- Impact: downstream crates are encouraged to couple to Tari-specific types even though the crate-level documentation positions the backend as swappable.
- Fix approach: distinguish stable Z00Z-owned crypto surface from backend escape hatches, then make Tari-specific exports explicitly advanced/internal if backend replacement is still a goal.

⚠️ **Reverse Test Coupling Exists Between Storage and Wallets:**

- Issue: `z00z_wallets` depends on `z00z_storage` at runtime, while `z00z_storage` depends on `z00z_wallets` in dev-dependencies.
- Files: `crates/z00z_wallets/Cargo.toml`, `crates/z00z_storage/Cargo.toml`
- Impact: integration tests can silently normalize cross-layer knowledge and make it harder to keep storage reusable as a lower-level crate.
- Fix approach: move shared fixtures/test DTOs into a neutral support crate or local test helpers that do not require `z00z_storage` to know about wallet feature flags.

## Known Bugs

⚠️ **Simulator Stage 6 Still Emits Demo-Only Placeholder Checkpoint Material:**

- Symptoms: Stage 6 explicitly documents that parts of its spent-key, fragment builder, aggregation, and digest path are placeholders and not canonical post-checkpoint state.
- Files: `crates/z00z_simulator/src/scenario_1/stage_6.rs`
- Trigger: running Scenario 1 through Stage 6 and treating its output as final checkpoint semantics rather than demo scaffolding.
- Workaround: treat Stage 6 outputs as simulator/demo artifacts only; do not plan downstream state commitments or external integrations on them.

⚠️ **Wallet Chain Scan State Is Process-Local and Resets on Restart:**

- Symptoms: scan jobs, progress, and derived chain tip information are kept only in memory and restart from an idle/default view.
- Files: `crates/z00z_wallets/src/services/chain_service.rs`
- Trigger: any restart, worker recycle, or multi-process usage.
- Workaround: use the scan APIs only as placeholder orchestration seams; do not rely on them for durable recovery, synchronization, or user-visible progress guarantees.

## Security Considerations

⚠️ **Debug Features Can Export or Expose Sensitive Wallet Material:**

- Risk: `wallet_debug_tools`, `wallet_debug_dump`, `verbose-logging`, and `eviction-logs` materially weaken privacy guarantees when enabled in the wrong environment.
- Files: `crates/z00z_wallets/Cargo.toml`, `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/src/core/key/key_manager.rs`, `crates/z00z_wallets/src/core/address/address_manager.rs`, `crates/z00z_simulator/Cargo.toml`
- Current mitigation: comments and feature-gate warnings state these are debug-only and should not be enabled in production.
- Recommendations: add CI checks that fail release profiles when these flags are enabled, and add a top-level security policy for feature gating across workspace crates.

⚠️ **Simulator Tests Depend on Wallet Debug Dump Behavior:**

- Risk: multiple simulator tests and helper paths are compiled only when `wallet_debug_dump` is enabled, which normalizes decrypted-wallet inspection as a test mechanism.
- Files: `crates/z00z_simulator/tests/test_claim_crypto.rs`, `crates/z00z_simulator/tests/test_claim_post.rs`, `crates/z00z_simulator/tests/test_claim_persist.rs`, `crates/z00z_simulator/tests/test_claim_integration.rs`, `crates/z00z_simulator/src/scenario_1/stage_3.rs`
- Current mitigation: feature gating keeps these paths opt-in.
- Recommendations: replace decrypted dump assertions with contract-level assertions on derived artifacts wherever possible, and reserve dump-based flows for narrow forensic tests.

⚠️ **Stub Security DTOs Still Exist on the Public RPC Type Surface:**

- Risk: default and helper values such as `PersistWalletId::default()` returning `stub-wallet-id` and `RuntimeEncryptedResponse::stub(...)` can leak into runtime-adjacent code if placeholder helpers are reused carelessly.
- Files: `crates/z00z_wallets/src/adapters/rpc/types/common.rs`, `crates/z00z_wallets/src/adapters/rpc/types/wallet.rs`
- Current mitigation: most usage is test/stub oriented.
- Recommendations: move stub helpers behind `cfg(test)` or a dedicated non-default debug feature to reduce accidental production reachability.

## Performance Bottlenecks

⚠️ **Whole-Wallet Persistence Rewrites the Entire `.wlt` Container:**

- Problem: wallet persistence opens a work file and re-encodes the whole RedB database through zstd back into the final `.wlt` file. Migration paths also flush the full container after metadata updates.
- Files: `crates/z00z_wallets/src/db/redb_wallet_store.rs`
- Cause: the design stores compressed whole-wallet containers rather than incrementally writable native structures.
- Improvement path: keep the security model but separate immutable packaging from hot-write storage, or introduce an append/checkpoint strategy so small metadata changes do not force full recompression.

⚠️ **Asset Store Retains Versioned Roots and Models In Memory:**

- Problem: `AssetStore` tracks `root_by_ver` and `model_by_ver` in addition to the current model and path map.
- Files: `crates/z00z_storage/src/assets/store.rs`
- Cause: historical version state is cached directly in process memory with no eviction or persistence boundary.
- Improvement path: define retention policy explicitly, or move historical replay material into a dedicated persistent backend rather than keeping all versions resident.

⚠️ **RPC Dispatcher Uses Serialize-Then-Deserialize Bridging for Typed Handlers:**

- Problem: parameter parsing and result serialization round-trip through `JsonCodec` into `serde_json::Value` even for in-process typed calls.
- Files: `crates/z00z_wallets/src/adapters/rpc/dispatcher_handlers.rs`
- Cause: the dispatcher normalizes everything through JSON-compatible transport shapes.
- Improvement path: keep the JSON boundary at the adapter edge and use typed handoff after initial decode to reduce allocation and runtime shape loss.

⚠️ **Simulator Stage 4 Is a Large Artifact-Oriented Orchestrator:**

- Problem: Stage 4 combines selection, tx building, storage proof preparation, RPC calls, file output, report generation, and validation in one pass.
- Files: `crates/z00z_simulator/src/scenario_1/stage_4.rs`
- Cause: scenario execution logic accumulated without being split into durable scenario services.
- Improvement path: extract transport calls, artifact I/O, output construction, and report generation into narrower modules with typed intermediate data.

## Fragile Areas

⚠️ **Wallet Persistence and Service State Are High-Risk Change Zones:**

- Files: `crates/z00z_wallets/src/db/redb_wallet_store.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`
- Why fragile: encryption metadata, KDF migration, session state, rate limits, address derivation counters, claimed assets, backup settings, and `.wlt` naming all sit near each other.
- Safe modification: change one responsibility at a time and verify both RPC-level and persistence-level behavior in the same phase.
- Test coverage: there is substantial test volume, but many tests are close to current implementation details and do not fully offset the blast radius of edits in these files.

⚠️ **Transaction Verification Is Split Across Local Package Checks and External State Semantics:**

- Files: `crates/z00z_wallets/src/core/tx/tx_verifier.rs`, `crates/z00z_storage/src/checkpoint/*.rs`, `crates/z00z_simulator/src/scenario_1/stage_4.rs`
- Why fragile: `TxProofWire` and `TxAuthWire` are currently empty structures, while comments say spend membership and reference semantics are resolved elsewhere.
- Safe modification: any change to tx semantics must update verifier, checkpoint preparation, and simulator artifact generation together.
- Test coverage: the verifier has many tests, but the cross-crate semantic boundary remains easy to desynchronize.

⚠️ **Genesis Logic Centralizes Too Many Policy Decisions:**

- Files: `crates/z00z_core/src/genesis/genesis.rs`, `crates/z00z_core/src/genesis/validator.rs`, `crates/z00z_core/bin/genesis/assets_allocator_cli.rs`
- Why fragile: network domains, seed validation, asset generation, export, timing, and CLI-oriented behaviors accumulate in one subsystem.
- Safe modification: isolate deterministic derivation, validation, and export paths before adding more policy branches.
- Test coverage: many tests exist, but pending TODOs in the validator and placeholder CLI pieces show the surface is still moving.

⚠️ **Storage Root and Path Semantics Are Correctness-Critical and Easy to Break:**

- Files: `crates/z00z_storage/src/assets/model.rs`, `crates/z00z_storage/src/assets/store.rs`, `crates/z00z_storage/README.md`
- Why fragile: canonical path identity, subtree root composition, and path rebinding rules are foundational for later checkpoint/snapshot phases.
- Safe modification: preserve public root semantics first and prove equivalence with blackbox store/snapshot tests before changing internal planner or cache logic.
- Test coverage: internal model tests are strong, but public persistence-style tests remain comparatively thinner because `AssetStore` is memory-backed.

## Scaling Limits

⚠️ **`AssetStore` Is Currently an In-Memory Semantic Store:**

- Current capacity: bounded by process memory; every version and model snapshot kept in `HashMap` state increases resident memory.
- Limit: long-running or high-churn workloads will accumulate history in memory without a clear compaction policy.
- Scaling path: add an explicit persistent/history backend or a bounded retention model before using this store for sustained node workloads.

⚠️ **Chain Scan APIs Are Not Ready for Multi-Worker or Durable Operation:**

- Current capacity: one process-local view of scan jobs and a synthetic tip derived from known jobs.
- Limit: restart safety, distributed workers, and durable user progress are not supported.
- Scaling path: persist scan jobs outside `ChainService` and separate network-derived tip state from placeholder scan progression.

⚠️ **Simulator Scenario 1 Scales Poorly as a Planning Proxy:**

- Current capacity: works as a richly instrumented artifact generator for a narrow scenario.
- Limit: Stage-specific files and report generation logic are too specialized and monolithic to generalize cleanly into more scenarios.
- Scaling path: extract reusable scenario services and artifact contracts before adding more stages or more actors.

## Dependencies at Risk

⚠️ **Vendored Tari Dependency Is Both Critical and Sticky:**

- Risk: `z00z_crypto` depends on vendored Tari crates under `crates/z00z_crypto/tari/`, and the workspace rules mark this subtree read-only.
- Impact: urgent backend fixes or API reductions cannot be applied freely in-tree, while the public `z00z_crypto` surface already exposes many Tari types.
- Migration plan: reduce direct Tari re-exports first, then keep vendored integration behind narrower Z00Z-owned wrapper types.

⚠️ **Wallet Persistence Stack Carries Multiple Native Storage Dependencies:**

- Risk: `z00z_wallets` declares both `sqlx` and `redb`, while active wallet file persistence is implemented in the RedB path.
- Impact: dependency footprint and platform surface area are larger than the currently visible runtime architecture suggests.
- Migration plan: either document the dual-storage plan clearly or remove dormant native storage dependencies from the active phase boundary.

## Missing Critical Features

⚠️ **Chain Scan and Recovery Are Still Placeholder-Level:**

- Problem: path usage, scan jobs, and tip reporting are placeholder behaviors rather than durable chain integration.
- Blocks: credible recovery, background sync, wallet rescan, and realistic network-driven UX.

⚠️ **Checkpoint/Bundle Finalization in Simulator Remains Demo-Oriented:**

- Problem: Stage 6 still labels critical pieces as placeholder-only and non-canonical.
- Blocks: using simulator outputs as a trustworthy pre-implementation oracle for final checkpoint semantics.

⚠️ **Genesis Tooling Is Not Fully Closed Over Final Production Flows:**

- Problem: `assets_allocator_cli` remains an explicit TODO/placeholder, and validator comments still point to unresolved consensus integration steps.
- Blocks: treating genesis tooling as fully finished operational infrastructure.

⚠️ **Wallet Domain Services Outside the Main Wallet Path Are Still Placeholders:**

- Problem: backup, network, storage, key, and tx service modules remain explicitly stubbed or placeholder-oriented.
- Blocks: clean domain decomposition, independent service evolution, and predictable service-level integration planning.

## Test Coverage Gaps

⚠️ **Storage Lacks a Durable Asset-State Backend Test Story:**

- What's not tested: long-lived retention, compaction, and persistence behavior for the asset-state store itself.
- Files: `crates/z00z_storage/src/assets/store.rs`, `crates/z00z_storage/tests/assets/store_api.rs`
- Risk: semantic correctness is well tested in memory, but future persistence or retention changes could break invariants without comparable contract tests.
- Priority: High

⚠️ **Simulator Acceptance Is Strong on Artifacts but Weak on Real External Boundaries:**

- What's not tested: real networked chain integration, restart behavior, and non-debug wallet inspection paths.
- Files: `crates/z00z_simulator/tests/test_pipeline_genesis_tx.rs`, `crates/z00z_simulator/src/scenario_1/stage_4.rs`, `crates/z00z_simulator/src/scenario_1/stage_6.rs`
- Risk: scenario outputs can look consistent while still depending on placeholder runtime seams.
- Priority: High

⚠️ **Wallet Placeholder Helpers Are Easy to Preserve Because Tests Explicitly Tolerate Them:**

- What's not tested: complete elimination of stub helper values from all runtime-facing DTOs and services.
- Files: `crates/z00z_wallets/src/adapters/rpc/types/common.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`, `crates/z00z_wallets/docs/rpc-user-guide.md`
- Risk: future phases may accidentally cement placeholder values or placeholder response shapes into production contracts.
- Priority: Medium

⚠️ **Genesis Validation Still Has Open Follow-Up Markers:**

- What's not tested: the final intended consensus-bound validator wiring implied by TODO markers and placeholder notes.
- Files: `crates/z00z_core/src/genesis/validator.rs`, `crates/z00z_core/bin/genesis/assets_allocator_cli.rs`
- Risk: planners can overestimate the completeness of genesis validation and generation tooling.
- Priority: Medium

---

📌 *Concerns audit: 2026-05-06*
