---
phase: 030-refactor-long-files
artifact: todo
status: summary-backed-historical
updated: 2026-04-04
source: .planning/temp/very-long-files.md, 030-24-SUMMARY.md, 030-25-SUMMARY.md
---

<!-- markdownlint-disable MD041 MD012 -->

## Phase 030 Closeout State

📌 Phase 030 is closed through plans `030-01` through `030-25`, the live non-test
Rust residue above 400 lines is `0`, and the canonical repo-native
`full_verify --max-safe-run` gate closed green on `2026-04-03`.

📌 This file is retained as the historical seam map and planning baseline that
guided the refactor waves. It is no longer the live pending-work checklist for
Phase 030.

📌 The authoritative closeout evidence now lives in:

- `.planning/phases/030-refactor-long-files/030-length_stat.md`
- `.planning/phases/030-refactor-long-files/030-24-SUMMARY.md`
- `.planning/phases/030-refactor-long-files/030-25-SUMMARY.md`
- `.planning/phases/030-refactor-long-files/030-VALIDATION.md`

📌 The seam inventory below remains useful as a design map, but its entries
must be read as historical planning notes rather than open debt.

## Historical Planning Scope

📌 This file turns the long-file analysis into one actionable refactoring backlog for every Rust file listed in `.planning/temp/very-long-files.md`.

📌 Default split rule: keep the current top-level file as a thin facade whenever that preserves public imports, reduces churn, and lets internal modules move in smaller reviewable steps.

📌 Default execution order inside each file: extract pure types and helpers first, then stateful logic, then trait or service impl blocks, and only last shrink the facade.

📌 Size goal for this backlog: every resulting Rust file, including the facade, must close below 1000 lines, with a preferred extracted-module band of 250-800 lines, a preferred facade band of 80-220 lines, and a normal facade closeout target below 300 lines. A facade may remain at `300+` lines when it is still cohesive, preserves compatibility, and stays clearly bounded to rustdoc, re-export, orchestration, or another non-mixed root responsibility instead of collapsing back into a mixed-concern soup.

📌 These line bands are heuristics, not a hard slicing contract. Do not split a structurally homogeneous responsibility into brittle fragments only to hit a target number of modules or lines.

📌 Split target filenames listed below are proposed seam names, not verified existing modules. The planner may rename them if the seam intent, protected-surface rules, and caller-visible stability rules stay intact.

📌 `Release-style gate` in this backlog means `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`.

## Size Guardrails

📌 Apply these size rules before calling any split complete.

1. Files in the 3000-4999 line range should target at least one facade plus six extracted modules.
2. Files in the 5000-6999 line range should target at least one facade plus eight extracted modules.
3. Files in the 7000-plus line range should target at least one facade plus ten extracted modules.
4. If any planned module is likely to land above 900 lines, add another seam before implementation starts.
5. If any resulting module still lands at 1000 lines or more after extraction, the split is not done.
6. If a proposed split produces many tiny files that still represent one concept and must be read together, the seam design is wrong even when the numeric limits look good.

## Size Table

📌 `Target files` counts the facade plus extracted modules from the current split plan.

📌 `Target size band` is the projected first-pass extracted-module range for the current seam design, not a guarantee for every individual module.

| Source file | Source lines | Target files | Target size band |
| --- | ---: | ---: | --- |
| `crates/z00z_wallets/src/db/redb_wallet_store.rs` | 8885 | 11 | `700-900` |
| `crates/z00z_wallets/src/services/wallet_service.rs` | 8414 | 11 | `700-900` |
| `crates/z00z_wallets/src/core/address/address_manager.rs` | 5284 | 9 | `400-700` |
| `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs` | 4229 | 7 | `400-700` |
| `crates/z00z_wallets/src/core/address/z00z_address.rs` | 4184 | 8 | `400-700` |
| `crates/z00z_wallets/src/core/key/seed.rs` | 3965 | 7 | `400-700` |
| `crates/z00z_wallets/src/core/key/bip32.rs` | 3314 | 7 | `400-700` |
| `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs` | 2949 | 7 | `400-700` |
| `crates/z00z_core/src/assets/assets.rs` | 2856 | 7 | `400-700` |
| `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs` | 2580 | 6 | `400-700` |
| `crates/z00z_wallets/src/core/key/key_manager.rs` | 2448 | 6 | `400-700` |
| `crates/z00z_core/src/assets/registry.rs` | 2129 | 4 | `400-700` |
| `crates/z00z_core/src/genesis/genesis.rs` | 2051 | 5 | `400-700` |
| `crates/z00z_crypto/src/aead.rs` | 1758 | 5 | `400-700` |
| `crates/z00z_wallets/src/core/tx/state_update.rs` | 1673 | 6 | `200-400` |
| `crates/z00z_wallets/src/egui_views/app_main_view.rs` | 1580 | 6 | `200-400` |
| `crates/z00z_crypto/src/kdf.rs` | 1519 | 5 | `200-400` |
| `crates/z00z_wallets/src/core/wallet/wallet.rs` | 1513 | 6 | `200-400` |
| `crates/z00z_crypto/src/hash.rs` | 1418 | 5 | `200-400` |
| `crates/z00z_crypto/src/types.rs` | 1386 | 4 | `400-700` |
| `crates/z00z_wallets/src/core/tx/claim_tx.rs` | 1351 | 6 | `200-400` |
| `crates/z00z_crypto/src/backend_tari.rs` | 1262 | 5 | `200-400` |
| `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` | 1260 | 7 | `120-200` |
| `crates/z00z_wallets/src/core/tx/tx_verifier.rs` | 1178 | 5 | `200-400` |
| `crates/z00z_wallets/src/core/tx/spending.rs` | 1144 | 5 | `200-400` |
| `crates/z00z_utils/src/io/fs.rs` | 1121 | 6 | `200-400` |
| `crates/z00z_core/src/assets/nonce.rs` | 1083 | 4 | `200-400` |
| `crates/z00z_core/src/genesis/validator.rs` | 1065 | 4 | `200-400` |
| `crates/z00z_wallets/src/services/app_service.rs` | 1043 | 6 | `120-200` |
| `crates/z00z_core/src/assets/definition.rs` | 1036 | 3 | `400-700` |

## Tier 1

📌 These files have the highest navigation and review cost and should be split first.

### `crates/z00z_wallets/src/db/redb_wallet_store.rs`

- Split into `redb_wallet_store_tables.rs`, `redb_wallet_store_codecs.rs`, `redb_wallet_store_compression.rs`, `redb_wallet_store_migrations.rs`, `redb_wallet_store_crypto.rs`, `redb_wallet_store_asset_objects.rs`, `redb_wallet_store_tx_objects.rs`, `redb_wallet_store_session.rs`, `redb_wallet_store_backup.rs`, `redb_wallet_store_queries.rs`
- Keep `redb_wallet_store.rs` as facade plus top-level store orchestration
- Primary seam: table or codec wiring, compression, migrations, crypto, object persistence, session lifecycle, backup flow, and query helpers must not stay in one or two giant files

### `crates/z00z_wallets/src/services/wallet_service.rs`

- Split into `wallet_service_state_store.rs`, `wallet_service_session.rs`, `wallet_service_limits.rs`, `wallet_service_receive_flow.rs`, `wallet_service_send_flow.rs`, `wallet_service_recovery.rs`, `wallet_service_derivers.rs`, `wallet_service_reachability.rs`, `wallet_service_background.rs`, `wallet_service_events.rs`
- Keep `wallet_service.rs` as service facade and orchestration layer
- Primary seam: in-memory state maps, session lifecycle, send and receive flows, recovery helpers, limits, background jobs, event fan-out, address-deriver handles, and reachability stubs must be isolated

### `crates/z00z_wallets/src/core/address/address_manager.rs`

- Split into `address_manager_trait.rs`, `address_manager_cache.rs`, `cache_metrics.rs`, `eviction_listener.rs`, `rate_limiter_bucket.rs`, `address_manager_expiry.rs`, `address_manager_config.rs`, `address_manager_impl.rs`
- Keep `address_manager.rs` as facade for public traits and re-exports
- Primary seam: trait surface, cache store, cache metrics, eviction listeners, expiry policy, config, and token-bucket logic should not evolve in one file

### `crates/z00z_wallets/src/adapters/rpc/methods/asset_impl.rs`

- Split into `asset_rpc_caches.rs`, `asset_rpc_rate_limits.rs`, `asset_rpc_stakes.rs`, `asset_rpc_balance.rs`, `asset_rpc_registry.rs`, `asset_rpc_history.rs`
- Keep `asset_impl.rs` as RPC facade
- Primary seam: cache TTL logic, send throttling, stake or quarantine tracking, balance shaping, registry lookups, and asset-history responses are separate responsibilities

### `crates/z00z_wallets/src/core/address/z00z_address.rs`

- Split into `z00z_address_codec.rs`, `z00z_address_features.rs`, `z00z_address_validation.rs`, `z00z_address_normalize.rs`, `z00z_address_parts.rs`, `z00z_single_address.rs`, `z00z_dual_address.rs`
- Keep `z00z_address.rs` as facade and shared re-export surface
- Primary seam: Bech32 codec, normalization, structural address parts, validation, and single-versus-dual address types should be independent

### `crates/z00z_wallets/src/core/key/seed.rs`

- Split into `seed_entropy_validation.rs`, `seed_entropy_source.rs`, `seed_mnemonic.rs`, `seed_cipher.rs`, `seed_kdf_params.rs`, `seed_backup_format.rs`
- Keep `seed.rs` as facade around the seed domain
- Primary seam: entropy heuristics, entropy source handling, mnemonic logic, encrypted seed container logic, backup format handling, and Argon2 policy presets should not be coupled

### `crates/z00z_wallets/src/core/key/bip32.rs`

- Split into `bip32_constants.rs`, `bip32_path.rs`, `bip32_path_validator.rs`, `bip32_child_key.rs`, `bip32_key_deriver.rs`, `bip32_ristretto_bridge.rs`
- Keep `bip32.rs` as facade and top-level documentation entry
- Primary seam: path parsing, path validation, child-key derivation, CKD flow, and the Ristretto bridge should be separate

### `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`

- Split into `tx_preparation_core.rs`, `input_selection_scan.rs`, `output_construction.rs`, `tx_validation_gates.rs`, `wallet_state_capture.rs`, `stage4_reporting.rs`
- Keep `stage_4_utils/mod.rs` as the stage-4 caller facade while `tx_lane_impl.rs` remains the internal execution seam
- Primary seam: orchestration, input discovery, output building, validation, state capture, and reporting should be reviewable independently

### `crates/z00z_core/src/assets/assets.rs`

- Split into `asset_error.rs`, `asset_class.rs`, `asset_serde_helpers.rs`, `asset_metadata.rs`, `asset_validation.rs`, `asset_ownership.rs`
- Keep `assets.rs` as facade and top-level asset orchestration entry
- Primary seam: the asset error system, asset-class taxonomy, serde helpers, and domain logic should not all live together

### `crates/z00z_wallets/src/adapters/rpc/methods/tx_impl.rs`

- Split into `tx_rpc_rate_limits.rs`, `tx_rpc_idempotency.rs`, `tx_rpc_storage.rs`, `tx_rpc_broadcast.rs`, `tx_rpc_impl.rs`
- Keep `tx_impl.rs` only if it remains a thin trait-impl facade; otherwise rename facade to `tx_rpc_impl.rs`
- Primary seam: RPC transport methods should not own rate limiting, persistence, idempotency caching, and broadcast retry policy in one file

## Tier 2

📌 These files are still oversized enough to justify targeted splits after Tier 1 stabilizes.

### `crates/z00z_wallets/src/core/key/key_manager.rs`

- Split into `key_cache.rs`, `key_derivation.rs`, `key_state.rs`, `key_persistence.rs`, `key_manager_impl.rs`
- Keep `key_manager.rs` as facade and trait or type re-export point
- Primary seam: TTL cache logic, derivation, persistent restore, and state metadata should evolve separately

### `crates/z00z_core/src/assets/registry.rs`

- Split into `registry_core.rs`, `registry_snapshot.rs`, `registry_config.rs`
- Keep `registry.rs` as facade with aliases and global registry exports
- Primary seam: snapshot sync, registry mutation, and config loading should not share one monolith

### `crates/z00z_core/src/genesis/genesis.rs`

- Split into `chain_type.rs`, `genesis_accumulator.rs`, `genesis_seed.rs`, `genesis_derivation.rs`
- Keep `genesis.rs` as orchestration facade
- Primary seam: network typing, seed policy, typed asset accumulation, and deterministic derivation helpers are naturally distinct

### `crates/z00z_crypto/src/aead.rs`

- Split into `aead_error.rs`, `aead_primitives.rs`, `aead_envelope.rs`, `aead_aad.rs`
- Keep `aead.rs` as facade and public export surface
- Primary seam: low-level XChaCha operations, envelope format, and AAD construction should be isolated

### `crates/z00z_wallets/src/core/tx/state_update.rs`

- Split into `state_witness.rs`, `state_resolved_input.rs`, `state_checkpoint.rs`, `state_errors.rs`, `state_traits.rs`
- Keep `state_update.rs` as transition facade
- Primary seam: witness model, resolved-input model, checkpoint assembly, and error mapping are separate axes of change

### `crates/z00z_wallets/src/egui_views/app_main_view.rs`

- Split into `tab_registry.rs`, `ui_config.rs`, `ui_state_machine.rs`, `ui_theme.rs`, `main_view.rs`
- Optionally add `tab_wallet.rs`, `tab_settings.rs`, `tab_network.rs` if the tab shell remains large after the first pass
- Primary seam: tab definitions, YAML-backed UI config, state machine transitions, and theme parsing should not be mixed into the eframe app loop

### `crates/z00z_crypto/src/kdf.rs`

- Split into `secret_bytes.rs`, `argon2_params.rs`, `argon2_kdf.rs`, `hkdf_kdf.rs`
- Keep `kdf.rs` as facade and policy documentation entry
- Primary seam: secret byte wrappers, Argon2 tuning policy, Argon2 execution, and HKDF expansion should move independently

### `crates/z00z_wallets/src/core/wallet/wallet.rs`

- Split into `chain_id.rs`, `wallet_identity.rs`, `wallet_kernel.rs`, `wallet_record.rs`, `wallet_entity.rs`
- Keep `wallet.rs` as facade if public API churn must stay low
- Primary seam: identity model, chain typing, wallet metadata, and the generic wallet container are separate concerns

### `crates/z00z_crypto/src/hash.rs`

- Split into `domain_separation.rs`, `blake2_hash.rs`, `sha256_hash.rs`, `hmac_sha256.rs`
- Keep `hash.rs` as facade and compatibility entry
- Primary seam: domain format policy should not be buried inside one file with all concrete hash algorithms

### `crates/z00z_crypto/src/types.rs`

- Split into `protocol_constants.rs`, `crypto_constants.rs`, `scalar_type.rs`
- Keep `types.rs` as facade for type aliases and public export surface
- Primary seam: scalar wrapper implementation should not live in the same file as every protocol constant

## Tier 3

📌 These files still deserve splitting, but the payoff is slightly lower or the seam is more straightforward.

### `crates/z00z_wallets/src/core/tx/claim_tx.rs`

- Split into `claim_wire_types.rs`, `claim_errors.rs`, `claim_verifier.rs`, `claim_helpers.rs`, `claim_auth.rs`
- Keep `claim_tx.rs` as facade for the claim-transaction domain
- Primary seam: wire types and verifier logic should be physically separate

### `crates/z00z_crypto/src/backend_tari.rs`

- Split into `backend_init.rs`, `backend_commitment.rs`, `backend_range_proofs.rs`, `backend_batch.rs`
- Keep `backend_tari.rs` as trait-impl facade
- Primary seam: service initialization, single-proof operations, commitment helpers, and batch verification should not share one file

### `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs`

- Split into `prep_snapshot_loader.rs`, `fragment_construction.rs`, `exec_input_builder.rs`, `bridge_output_router.rs`, `demo_checkpoint_agg.rs`, `stage6_logging.rs`
- Keep `stage_6_utils/mod.rs` as the stage-6 caller facade while `bundle_lane_impl.rs` remains the internal execution seam
- Primary seam: snapshot loading, fragment building, checkpoint aggregation, and logging are naturally separate

### `crates/z00z_wallets/src/core/tx/tx_verifier.rs`

- Split into `tx_wire_types.rs`, `tx_digest.rs`, `tx_errors.rs`, `tx_verifier.rs`
- Keep `tx_verifier.rs` only if it stays as the final verifier facade
- Primary seam: digest creation and wire-model definitions should not be trapped inside the verifier file

### `crates/z00z_wallets/src/core/tx/spending.rs`

- Split into `spend_rules.rs`, `spend_verification.rs`, `spend_errors.rs`, `spend_test_helpers.rs`
- Keep `spending.rs` as facade and public entrypoint for spending checks
- Primary seam: declarative spend rules and the verification engine should not be coupled to test-only helpers

### `crates/z00z_utils/src/io/fs.rs`

- Split into `atomic_write.rs`, `file_read.rs`, `json_io.rs`, `yaml_io.rs`, `bincode_io.rs`
- Keep `fs.rs` as facade and generic codec entrypoint
- Primary seam: atomic write semantics and per-codec I/O helpers are already distinct responsibilities

### `crates/z00z_core/src/assets/nonce.rs`

- Split into `nonce_type.rs`, `nonce_counter.rs`, `nonce_derivation.rs`
- Keep `nonce.rs` as facade and policy documentation point
- Primary seam: nonce storage contract and nonce derivation strategies must not remain tightly coupled

### `crates/z00z_core/src/genesis/validator.rs`

- Split into `genesis_error.rs`, `genesis_verification.rs`, `genesis_config_validate.rs`
- Keep `validator.rs` as facade and orchestration entry
- Primary seam: batch proof validation and config-schema validation are independent domains

### `crates/z00z_wallets/src/services/app_service.rs`

- Split into `app_wallet_lifecycle.rs`, `app_chain_network.rs`, `app_seed_password.rs`, `app_kernel.rs`, `app_service_impl.rs`
- Keep `app_service.rs` as top-level service facade if public imports depend on it
- Primary seam: wallet lifecycle operations, chain switching, and seed or password helpers should move separately

### `crates/z00z_core/src/assets/definition.rs`

- Split into `definition_id.rs`, `definition_validate.rs`
- Keep `definition.rs` for the struct and constructors
- Primary seam: definition ID derivation and validation policy are already separable from the data model itself

## Protected Seams

📌 These seams are not ordinary layout cleanup. Split them without fragmenting the caller-visible security or boundary contract.

### Crypto Ownership Seams

- `crates/z00z_crypto/src/hash.rs`, `crates/z00z_crypto/src/kdf.rs`, and `crates/z00z_crypto/src/aead.rs` must preserve one canonical owner for domain tags, transcript framing helpers, AAD builders, and KDF info constants.
- Do not create parallel public entrypoints for the same crypto operation during a split wave.
- Keep `crates/z00z_crypto/src/lib.rs` as the stable external facade while internals move.

### Genesis Alias Seams

- Before cleaning up deep paths around `crates/z00z_core/src/genesis/genesis.rs`, normalize consumers to existing shallow aliases such as `z00z_core::genesis::ChainType`.
- Do not combine the first internal split of `genesis.rs` with deep-path public normalization in the same wave.

### Wallet Session And Store Seams

- Treat `WltSession`, `ScanStatePayload`, `is_lock_held_local`, session readers, and session-backed store helpers in `crates/z00z_wallets/src/db/redb_wallet_store.rs` as boundary-sensitive symbols.
- Keep caller-visible contracts stable for `session_service`, `wlt_store`, `core/address`, and `core/wallet` while the internal store split is in progress.

## Verification Anchors

📌 Every protected-seam or normalization wave must name exact verification anchors in the plan and execution notes instead of only saying `run crate tests`.

### Wallet Store And Service Waves

- Anchor store-boundary waves with `crates/z00z_wallets/tests/test_redb_wlt_open.rs`.
- Anchor wallet-source and snapshot waves with `crates/z00z_wallets/tests/test_open_wallet_source_discovery.rs`.
- Anchor consumer-facing tx-store behavior with `crates/z00z_wallets/tests/test_tx_store_integration.rs`.

### Genesis And Asset Waves

- Anchor genesis splits with `crates/z00z_core/tests/genesis/test_genesis.rs` and `crates/z00z_core/tests/genesis/test_reproducibility.rs`.
- Anchor asset splits with `crates/z00z_core/tests/assets/test_assets.rs` and `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs`.

### Crypto Ownership Waves

- Anchor crypto ownership seams with `crates/z00z_crypto/tests/test_hash_policy.rs` and `crates/z00z_crypto/tests/test_domain_separation.rs`.
- When wallet-facing derivation or envelope behavior can change indirectly, also anchor `crates/z00z_wallets/tests/test_kdf.rs` and relevant doc-test surfaces such as `crates/z00z_crypto/src/aead.rs`.

### Cross-Crate Smoke Waves

- When a wave changes aliases, public paths, or simulator-facing structure, include at least one cross-crate smoke anchor such as `crates/z00z_simulator/tests/test_genesis_integration.rs` or `crates/z00z_simulator/tests/test_stage4_split.rs`.

## Sequence Rules

📌 Apply these rollout rules to every split.

1. Extract type definitions and pure helper functions first.
2. Add private modules and re-export from the current top-level file.
3. Move stateful logic and impl blocks only after the helper seams are stable.
4. Preserve caller-visible public paths by default during split waves, especially on compatibility-sensitive surfaces.
5. Run targeted crate tests after each file split instead of batching many files into one risky change.
6. Do not combine the first structural split and external path normalization in the same wave for protected seams.
7. Normalize consumers to existing shallow aliases before moving or renaming deep modules such as `genesis::ChainType`.
8. Do not parallelize two waves that touch the same facade root, the same public re-export surface, or the same boundary-sensitive symbols.
9. When deep imports are already widespread, allow an alias-only consumer rewrite as a dedicated prerequisite wave before the final crate-level normalization subwave.
10. Prefer one coherent extracted module over several small shard files when the logic still forms one homogeneous responsibility.

## Rollback Triggers

📌 If one of these triggers fires, stop widening the wave and fall back to the last stable path-preserving seam.

1. A split creates duplicate crypto entrypoints, duplicate domain-tag owners, or multiple transcript or AAD builder surfaces.
2. A split forces caller-visible path churn across more than one crate before the planner has a full grep-backed caller inventory.
3. A moved module introduces circular imports, hidden boundary breakage, or a new module that still projects above 900 lines.
4. A protected seam wave fails bootstrap, targeted tests, or the mandatory same-wave release-style gate.
5. A seam design turns one coherent subsystem into many small files with no strong responsibility boundary between them.

## First Five Candidates

📌 If only five refactors start in the next wave, use this preferred order unless a dependency-first split inside the same crate requires a local reorder.

1. `crates/z00z_wallets/src/db/redb_wallet_store.rs`
2. `crates/z00z_wallets/src/services/wallet_service.rs`
3. `crates/z00z_wallets/src/core/address/address_manager.rs`
4. `crates/z00z_core/src/assets/assets.rs`
5. `crates/z00z_core/src/genesis/genesis.rs`

## Done Criteria

📌 A file is considered successfully split only when all conditions below are true.

## 030-10 Caller Inventory

📌 `030-10` is the dedicated wallet normalization wave that rewrites wallet-owned callers onto shallow facades after the structural split waves stabilized.

- The database facade is the canonical import lane for `WltSession`, `ScanStatePayload`, lock checks, and session-backed store helpers.
- The services facade is the canonical import lane for `AddressUsedOracle`, `RateLimitPrecheck`, and `Sleeper`.
- The address facade is the canonical import lane for cache-size constants and address-manager surface types.
- The key facade is the canonical import lane for BIP-44, seed, and key-manager surfaces.
- The transaction facade remains the canonical entrypoint for verifier and spend-pipeline surface types.

- Record the exact `rg` audit output for the wallet normalization wave in `030-10-SUMMARY.md`.

- The current top-level file is normally below 300 lines, or a documented facade exception explains why a larger cohesive facade, including `500+` lines, is still the cleaner non-soup result.
- No resulting file, including every extracted module, is 1000 lines or more.
- Any module projected above 900 lines is split again before the task closes.
- The split matches responsibility seams instead of arbitrary line-count slicing.
- The result preserves structural homogeneity instead of replacing one monolith with a pile of shard-like files.
- Caller-visible imports remain stable during structural split waves, or a dedicated normalization subwave documents the full caller update and closes with the required gates.
- Any dedicated normalization subwave closes only after grep-backed caller inventory plus synchronized code, tests, rustdoc, and planning-reference updates.
- Protected seams still have one canonical owner surface for crypto helpers, boundary-sensitive wallet session types, and genesis alias paths.
- Tests for the owning crate still pass.
- Protected seam waves also pass bootstrap, targeted crate tests, the same-wave release-style gate, and a grep audit for legacy deep imports, rustdoc paths, and stale planning references.
- The execution notes for the wave name the exact verification anchors used and the concrete commands run for local, consumer-facing, and cross-crate verification.
- The new module names explain responsibility without abbreviations.

