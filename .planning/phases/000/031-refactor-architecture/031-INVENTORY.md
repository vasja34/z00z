# Phase 031 Wave 0 Inventory

## Objective

This file is the canonical Wave 0 inventory for Phase 031. It freezes every
retirement-sensitive seam before any root-facade narrowing, shim retirement,
or suffix cleanup starts.

## Status Legend

| Field | Meaning |
| --- | --- |
| `public_status` | `default-public`, `public-nondefault`, `internal-only` |
| `lane_status` | `stable`, `provisional`, `migration-only`, `undocumented` |
| `canonical_lane` | `canonical`, `non-canonical`, `mixed`, `unknown` |

## Inventory Table

| Seam ID | Crate | Seam Kind | Owner File | Evidence | public_status | Downstream Caller Pattern | lane_status | Canonical Surviving Facade Or Signature | canonical_lane | Later Plan Allowed To Change It |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| INV-CORE-001 | `z00z_core` | wildcard root export | `crates/z00z_core/src/lib.rs` | Root exports `pub use assets::*;` | default-public | Broad root imports such as `use z00z_core::AssetLeaf`, `AssetWire`, `ChainType`; heavy simulator and wallet adapter use | stable | Curated explicit root exports from `z00z_core::lib` only | non-canonical | `031-02` |
| INV-CORE-002 | `z00z_core` | deep asset module reach-through | `crates/z00z_core/src/lib.rs` and `crates/z00z_core/src/assets/**` | Callers still import `z00z_core::assets::*` directly for `AssetLeaf`, `AssetPkgWire`, registry items, and JSON helpers | default-public | `z00z_storage`, `z00z_wallets`, and `z00z_simulator` rely on direct `assets` submodule paths | stable | Explicit `assets` submodule plus narrowed root aliases proven by caller inventory | mixed | `031-02` |
| INV-CRYPTO-001 | `z00z_crypto` | Tari concrete root leakage | `crates/z00z_crypto/src/lib.rs` | Root publicly re-exports Tari-owned factories, proof services, Ristretto types, `ByteArray`, `SafePassword`, `DiffieHellmanSharedSecret`, and hex helpers | default-public | Wallet, simulator, tests, and examples import directly from `z00z_crypto` root | stable | Stable Z00Z-owned facade in `lib.rs`; vendor passthrough moved to explicit non-default lane | non-canonical | `031-03` |
| INV-CRYPTO-002 | `z00z_crypto` | Tari-backed public type aliases | `crates/z00z_crypto/src/lib.rs` | `Z00ZSchnorrSignature` and `Z00ZCommitmentSignature` are aliases over Tari concrete types | default-public | Simulator/tests consume aliases from root without vendor namespace awareness | stable | Keep only intentionally stable Z00Z aliases or wrappers and demote vendor concrete ownership | mixed | `031-03` |
| INV-CRYPTO-003 | `z00z_crypto` | compatibility-only stealth ECDH lane | `crates/z00z_crypto/src/ecdh_stealth.rs` and `crates/z00z_crypto/src/lib.rs` | Module docs label sender/receiver helpers and trait as compatibility-only or legacy-oriented | default-public via root-compatible exposure | Legacy stealth callers can still bind through public crypto surface | migration-only | Non-legacy callers should bind to canonical stealth and ECDH APIs only | non-canonical | `031-03` and final retirement in `031-10` |
| INV-CRYPTO-004 | `z00z_crypto` | test-only AEAD seam | `crates/z00z_crypto/src/aead_test_only.rs` | Plan context marks nonce-controlled AEAD helpers as blocker if reachable outside test profiles | public-nondefault | Test and compatibility helpers may drift into non-test imports if root stays broad | migration-only | Test-only AEAD remains fully gated and off stable facade | non-canonical | `031-03` |
| INV-CRYPTO-005 | `z00z_crypto` | wildcard submodule export | `crates/z00z_crypto/src/aead.rs` | `pub use super::aead_zkpack::*;` inside AEAD surface | public-nondefault | Internal and downstream callers may receive more AEAD items than intended | provisional | Explicitly named AEAD exports with no wildcard passthrough | non-canonical | `031-03` |
| INV-WLT-001 | `z00z_wallets` | include-assembled service root | `crates/z00z_wallets/src/services/wallet_service.rs` | File is assembled by `include!("wallet_service_types.rs")`, `actions`, `session`, `store`, and tests | default-public via service facade | Root service surface hides reachability, store, session, and placeholder seams behind one flat contract | provisional | Explicit facade over named submodules with visible ownership | non-canonical | `031-05` |
| INV-WLT-002 | `z00z_wallets` | stub-heavy services exposed as stable-looking root | `crates/z00z_wallets/src/lib.rs` | Crate docs admit services and adapters are stub-heavy or partial while root still re-exports service/core surfaces | default-public | Callers bind to `WalletService` and other top-level wallet items as if fully stable | provisional | Curated root split between stable core, services facade, and adapter edge lanes | non-canonical | `031-05` and `031-07` |
| INV-WLT-003 | `z00z_wallets` | persisted identity and auth drift seam | `crates/z00z_wallets/src/services/wallet_service_store_create_unlock_open.rs`, `wallet_paths.rs`, `wallet_service_session_guards.rs` | Phase context marks persisted `(PersistWalletId, network, chain)` identity and `lock_wallet` denial as blockers | default-public mutation paths | Open, unlock, derive, and lock flows can still drift against persisted identity if not centralized | stable security boundary with unresolved proof | Persisted identity owner surface plus auth-bound `lock_wallet` semantics | mixed | `031-06` |
| INV-WLT-004 | `z00z_wallets` | RPC DTO/root facade widening | `crates/z00z_wallets/src/lib.rs` and `crates/z00z_wallets/src/adapters/rpc/types/mod.rs` | Wallet root and adapter edges expose transport-owned DTOs and RPC aliases into broader caller lanes | default-public | Tests, examples, and runtime wiring import RPC-facing contracts through broad wallet surfaces | provisional | Edge-owned RPC type boundary, with root demoted away from DTO ownership | non-canonical | `031-07` |
| INV-WLT-005 | `z00z_wallets` | named compatibility lane `wallet.key.export_public_material_v2` | `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`, `server.rs`, `server_requests.rs`, dispatcher wiring | Explicit RPC method name and implementation remain live and version-suffixed | default-public edge contract | RPC clients and dispatcher wiring bind to versioned method name directly | stable or migration-only, unresolved | Must be explicitly dispositioned as canonical live lane or demoted before suffix retirement | unknown | `031-07`, final disposition in `031-10` |
| INV-WLT-006 | `z00z_wallets` | named compatibility lane `ReceiverCardRecordV1` | `crates/z00z_wallets/src/core/chain/receiver_card_record.rs` plus RPC asset helpers | Type remains used across core chain and RPC conversion helpers | default-public through core chain module | RPC decode and asset helpers still consume `ReceiverCardRecordV1` as named public contract | stable or migration-only, unresolved | Must be explicitly canonical or demoted before suffix retirement | unknown | `031-07`, final disposition in `031-10` |
| INV-STORAGE-001 | `z00z_storage` | backend-mechanics seam under stable crate root | `crates/z00z_storage/src/lib.rs`, `src/checkpoint/**`, `src/assets/store_internal/**` | Public crate is compact, but checkpoint semantics and store internals remain sensitive to proof-binding and backend leakage | default-public crate with internal sub-seams | Wallet core and simulator import storage roots; simulator acceptance intends to avoid `store_internal` | stable crate root, internal mechanics unresolved | Storage root stays stable; checkpoint proof semantics hardened without exposing backend-specific helpers | canonical at crate root, internal seams unresolved | `031-08` |
| INV-STORAGE-002 | `z00z_storage` | proof and attestation compatibility payload seam | `crates/z00z_storage/src/checkpoint/store.rs`, `artifact_final.rs`, `artifact_proof_draft.rs` | Context keeps `cp_proof` semantics as explicit hardening blocker | default-public through checkpoint artifacts | Canonical checkpoint and rehydrate callers rely on store/finalization behavior | stable contract with unresolved semantics | Statement-bound checkpoint identity and explicit proof-binding semantics | mixed | `031-08` |
| INV-SIM-001 | `z00z_simulator` | deep wallet core imports | `crates/z00z_simulator/src/**/*.rs` | Simulator directly imports `z00z_wallets::core::claim`, `core::tx`, `core::address`, `core::hashing`, and `services::WalletService` | internal-only consumer pattern | Scenario stages and tests enter wallet internals rather than only narrow stable facades | undocumented integration-harness allowance | Simulator should enter wallet only through explicitly blessed stable facades after cleanup | non-canonical | `031-09` |
| INV-SIM-002 | `z00z_simulator` | deep storage imports | `crates/z00z_simulator/src/claim_pkg_consumer.rs`, `stage_6_utils/prep_ref.rs`, tests | Simulator imports `z00z_storage::assets::*`, checkpoint decode helpers, and snapshot ids directly | internal-only consumer pattern | Scenario stages and tests bind to storage semantics beyond minimal harness entrypoints | undocumented integration-harness allowance | Stable simulator-facing storage seams only; no `store_internal` use | mixed | `031-09` |
| INV-SIM-003 | `z00z_simulator` | default plaintext secret artifact | `crates/z00z_simulator/src/scenario_1/stage_2.rs` | Stage 2 writes `wlt_secrets_debug.md` under outputs | internal-only runtime artifact | Default scenario execution can emit plaintext wallet secrets | provisional unsafe output | Debug-only gated artifact or removed from default contract | non-canonical | `031-09` |
| INV-SIM-004 | `z00z_simulator` | recursive output reset seam | `crates/z00z_simulator/src/scenario_1/runner.rs` | Runner calls `reset_outputs_dir`, which uses `io::remove_dir_all` on configured output directory | internal-only runtime helper | Scenario runner can recursively remove output directory contents | provisional hardening obligation | Sandbox-validated output root reset only | non-canonical | `031-09` |
| INV-UTILS-001 | `z00z_utils` | broad prelude convenience lane | `crates/z00z_utils/src/lib.rs` and `src/prelude` | Workspace callers use `z00z_utils::prelude::*` and prelude subsets for logger, time, codec, IO, config | default-public | Core/examples/tests consume prelude as convenience root | stable convenience lane with policy drift risk | Explicit admission policy and narrow prelude ownership note | mixed | `031-10` |
| INV-NET-001 | `z00z_networks_rpc` | transport-only root that is reused as network surface | `crates/z00z_networks/rpc/src/lib.rs` and wallet adapter re-exports | RPC root exports `RpcTransport`, dispatcher, local transport, wasm client; wallet adapters re-export them again | default-public | Wallet adapters and simulator stages bind directly to transport interfaces | stable edge lane | RPC remains limited transport facade; higher-level network concerns move to documented future owners | canonical for transport, non-canonical for whole-network ownership | `031-04` |
| INV-NET-002 | `z00z_networks/onionnet` | placeholder overlay namespace | `crates/z00z_networks/onionnet/Cargo.toml`, `README.md`, `src/lib.rs` | Namespace exists as crate-shaped placeholder aligned to Phase 115 | public-nondefault placeholder | No callers yet, but future network work will land here | provisional | OnionNet remains explicit node-owned overlay namespace | canonical placeholder | `031-04` |

## Observations That Gate Later Waves

1. `z00z_core` and `z00z_crypto` can only narrow root facades safely after Wave 0
   because both are imported broadly by `z00z_simulator`, `z00z_wallets`, and
   selected storage or runtime code. Their changes are separable, but only if
   caller migration stays explicit and no new compatibility aliases are added.
2. Wallet work is not one seam. Service assembly, persisted identity or lock
   semantics, and RPC DTO or compatibility lanes have different owners and must
   remain split across `031-05`, `031-06`, and `031-07`.
3. Storage and simulator are blocked by semantics rather than by namespace
   shape alone. Their Wave 3 work must harden artifact meaning, sandboxing, and
   harness entry lanes instead of only moving symbols.
4. Versioned names and migration lanes already present in wallet RPC and chain
   surfaces cannot be retired on sight. They require explicit disposition in the
   final retirement artifact.

## Canonical Plan Ownership Summary

| Area | Owning Plans |
| --- | --- |
| `z00z_core` root narrowing | `031-02` |
| `z00z_crypto` stable or vendor split, AEAD gating | `031-03` |
| `z00z_networks_rpc` vs `onionnet` ownership note | `031-04` |
| Wallet service split and include retirement | `031-05` |
| Wallet persisted identity and lock authorization | `031-06` |
| Wallet RPC DTO demotion and compatibility lane disposition | `031-07` |
| Storage proof-binding and checkpoint semantics | `031-08` |
| Simulator facade-only imports, secret-output gating, reset sandbox | `031-09` |
| Utils admission policy and final retirement proof | `031-10` |
