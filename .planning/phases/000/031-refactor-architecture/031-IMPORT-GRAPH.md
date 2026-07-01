# Phase 031 Wave 0 Import Graph

## Objective

This file records the per-crate import and caller graph needed by Gate G-00.
It answers two questions:

1. Which reviewed crate roots are currently acting as broad caller entrypoints?
2. Which later plans can move in parallel without creating shared-caller drift?

## Reviewed Root Summary

### `z00z_core`

- Direct root re-exports:
  - `pub mod assets`
  - `pub mod domains`
  - `pub mod genesis`
  - `pub mod hashing`
  - `pub use assets::*`
  - `pub use genesis::{... ChainType ...}`
- Major inward callers:
  - `z00z_storage` imports `z00z_core::assets::*` and root aliases
  - `z00z_wallets` RPC adapters import `Asset`, `AssetClass`, registry ids, and
    asset-wire types
  - `z00z_simulator` heavily imports `Asset`, `AssetWire`, `AssetLeaf`,
    `AssetDefinitionRegistry`, `ChainType`, and genesis helpers
  - runtime aggregators also consume root aliases such as `AssetLeaf`
- Public surface hot paths:
  - asset DTOs and leaf/wire types
  - registry ids and global registry access
  - genesis helper and chain-type access
- Blocking ambiguity:
  - The broad `pub use assets::*` root makes it impossible to tell whether a
    caller intentionally wants stable root aliases or is depending on internal
    asset namespace spillover.
- Phase 031 implication:
  - `031-02` must replace wildcard root exposure with a curated facade and
    migrate real callers directly, not by preserving another alias lane.

### `z00z_crypto`

- Direct root re-exports:
  - Z00Z-owned helpers, domains, hash/KDF utilities, AEAD surface, proof
    helpers, framing helpers
  - Tari passthroughs from `tari_crypto::*`
  - Tari-backed aliases such as `Z00ZSchnorrSignature` and
    `Z00ZCommitmentSignature`
- Major inward callers:
  - `z00z_wallets` imports `SafePassword`, `Hidden`, AEAD helpers, hash helpers,
    and scalar or commitment-related contracts from the root
  - `z00z_storage` imports hash domains, `hash_zk`, and claim-proof versions
  - `z00z_simulator` imports root crypto contracts for Stage 2, Stage 3, Stage 4,
    and bundle lanes
  - `z00z_rollup_node`, benches, tests, and examples also bind directly to root
    exports
- Public surface hot paths:
  - `SafePassword`, `Hidden`, scalar or point wrappers
  - proof generation and verification helpers
  - domain tags and framing helpers
  - vendor concrete factories and services leaking through root
- Blocking ambiguity:
  - The root currently mixes stable Z00Z contracts with expert and vendor
    passthroughs, so a caller can appear root-stable while actually coupling to
    Tari semantics.
- Phase 031 implication:
  - `031-03` must separate stable facade from vendor lane without changing
    domain-tag ownership or canonical framing semantics.

### `z00z_wallets`

- Direct root re-exports:
  - root modules `core`, `adapters`, `services`, `wasm`, `db`, `egui_views`
  - root-facing wallet service, sender/stealth flows, and multiple adapter-edge
    contracts
- Major inward callers:
  - `z00z_simulator` is the dominant consumer and imports wallet root, wallet
    core, wallet services, and adapter RPC types directly
  - runtime aggregators consume `core::tx` types
  - benches, examples, and tests consume both root and deep module lanes
- Public surface hot paths:
  - `core::tx`
  - `core::claim`
  - `core::address`
  - `services::WalletService`
  - `adapters::rpc::types::*`
- Blocking ambiguity:
  - Wallet root simultaneously exposes stable-looking domain APIs,
    reachability-only helper lanes, service assembly, RPC DTOs, and compatibility
    names like `export_public_material_v2` or `ReceiverCardRecordV1`.
- Phase 031 implication:
  - Wallet work cannot be treated as one root cleanup. Service split,
    persisted-identity/auth hardening, and RPC DTO/root demotion need separate
    plans because they affect overlapping but not identical caller sets.

### `z00z_storage`

- Direct root re-exports:
  - `assets`
  - `checkpoint`
  - `error`
  - `serialization`
  - `snapshot`
- Major inward callers:
  - `z00z_wallets::core::tx` imports asset proofs, checkpoint types, and
    checkpoint state update machinery
  - `z00z_simulator` imports asset roots, checkpoint decode helpers, and
    snapshot ids
  - runtime crates consume storage-facing asset and verdict types
- Public surface hot paths:
  - asset proof and root types
  - checkpoint artifact and finalization semantics
  - snapshot persistence and replay lanes
- Blocking ambiguity:
  - The crate root is not too broad, but canonical proof meaning vs backend or
    compatibility payload handling is still unresolved in checkpoint internals.
- Phase 031 implication:
  - `031-08` is semantic hardening, not namespace surgery.

### `z00z_simulator`

- Direct root re-exports:
  - scenario harness and stage-facing types from `src/lib.rs`
  - scenario runner entrypoints exposed to tests and examples
- Major inward callers:
  - simulator tests and examples consume root contracts directly
  - no reviewed non-simulator crate depends heavily on simulator as production
    dependency, which confirms integration-harness role
- Public surface hot paths:
  - `scenario_1::runner`
  - `ScenarioCfg`
  - `StageResult`
  - Stage 3 and Stage 6 helper exports used by tests
- Blocking ambiguity:
  - The simulator root is not the main issue; the issue is that scenario stages
    directly import deep wallet and storage internals, and Stage 2 emits
    plaintext secret artifacts by default.
- Phase 031 implication:
  - `031-09` must tighten harness policy and stable-facade entry rules rather
    than attempt to shrink simulator into a minimal facade only.

### `z00z_utils`

- Direct root re-exports:
  - modules for codec, compression, config, io, logger, metrics,
    os_hardening, rng, and time
  - broad `prelude` convenience lane
- Major inward callers:
  - `z00z_core` imports logger, metrics, time, and IO helpers through prelude
  - `z00z_storage` uses codec and IO helpers directly
  - `z00z_crypto` uses RNG and logger helpers
  - `z00z_simulator` uses IO, codec, RNG, and logger utilities
- Public surface hot paths:
  - prelude convenience imports
  - codec and IO helpers
  - RNG traits and providers
  - time providers
- Blocking ambiguity:
  - The crate is intentionally cross-cutting, but without a README-level
    admission policy it can continue to absorb unrelated ownership by inertia.
- Phase 031 implication:
  - `031-10` needs written boundary policy, not code movement first.

### `z00z_networks_rpc`

- Direct root re-exports:
  - transport trait
  - dispatcher
  - local transport helpers
  - wasm client
  - rpc error surface
- Major inward callers:
  - wallet adapter modules and tests import `RpcTransport`, `RpcDispatcher`,
    and `LocalRpcTransport`
  - simulator Stage 2, Stage 3, Stage 4, Stage 5, and Stage 11 flows import
    RPC transport directly
- Public surface hot paths:
  - `RpcTransport`
  - `RpcDispatcher`
  - local transport test wiring
- Blocking ambiguity:
  - The crate is transport-scoped, but its current reuse pattern can let callers
    treat it as the whole network layer unless peer identity, auth, retry, and
    connection-lifecycle ownership are documented elsewhere.
- Phase 031 implication:
  - `031-04` must make the limited transport contract explicit and reserve
    `onionnet` as the future overlay owner.

## Parallelism Decisions

### Wave 1: `031-02` and `031-03`

These two plans can proceed in parallel after Wave 0.

- Why parallelism is acceptable:
  - `031-02` narrows `z00z_core` root ownership.
  - `031-03` narrows `z00z_crypto` stable vs vendor ownership.
  - Their caller overlap is real (`z00z_wallets`, `z00z_simulator`, and some
    storage/runtime callers), but the actual symbols differ enough that the
    migration work can remain crate-local if each plan uses the Wave 0 caller map.
- Required caution:
  - Neither plan may introduce fresh compatibility aliases to “unblock” the
    other.
  - Simulator and wallet caller updates must stay explicit in each plan summary.

### Wave 2: wallet plans `031-05`, `031-06`, `031-07`

These plans should not be treated as freely parallel.

- `031-05` must land first because it turns the include-assembled service root
  into explicit module ownership.
- `031-06` then hardens persisted-identity and `lock_wallet` authorization at
  the service/session layer.
- `031-07` finally narrows RPC DTO ownership, root exports, and named
  compatibility lanes.

The same downstream callers span all three plans: simulator runtime, wallet RPC
wiring, service tests, and wallet root imports. That is shared-caller drift by
default, so the sequence must stay explicit.

## Gate G-00 Conclusion

Gate G-00 is satisfied for execution purposes when later plans rely on the
following Wave 0 truths:

1. `z00z_core` and `z00z_crypto` are broad stable roots with evidence-backed
   inward callers, so Wave 1 can narrow them only through explicit caller
   migration.
2. `z00z_wallets` is not one cleanup seam; it contains at least three distinct
   ownership problems with overlapping callers.
3. `z00z_storage` is namespace-stable but semantics-sensitive.
4. `z00z_simulator` is the integration harness and therefore the main deep
   consumer that must be constrained by stable-facade policy, output secrecy,
   and sandboxed reset behavior.
5. `z00z_utils` and `z00z_networks_rpc` both require clarified ownership notes
  to prevent future scope drift.
