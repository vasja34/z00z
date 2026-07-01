# 036-a4 Shim Removal Spec

## 🎯 Goal

Remove compatibility shims from Phase 036 and converge every caller onto the canonical owners that already exist in the repository.

This spec is a full-delete pass, not a compatibility-retention pass. If a surface already has a canonical owner, callers should move there and the shim should be deleted. If a surface does not yet have a verified canonical replacement, it belongs in the open-gap section instead of being silently treated as resolved.

## 📦 Scope

In scope:

- explicit backward-compatible constructors
- deprecated wrappers
- compatibility-only facades
- stateless compatibility output lanes
- simulator-only entropy adapters
- compatibility roots and legacy address codecs

Out of scope:

- Tari vendor code under `crates/z00z_crypto/tari/`
- generated artifacts and `target/`
- logging text, comments, and docs that do not own behavior
- Phase 1 placeholder RPC methods that do not yet have a verified canonical replacement

## 🧭 Repository-Backed Baseline

The canonical owner split is already documented in the source tree:

- `crates/z00z_utils/src/time/traits.rs` says `try_unix_timestamp*()` is the canonical production contract and `compat_unix_timestamp*()` is the explicit compatibility surface.
- `crates/z00z_crypto/src/ecdh.rs` is the canonical owner for point-based stealth ECDH and the `derive_dh_key(&Z00ZRistrettoPoint)` formula chain.
- `crates/z00z_wallets/src/core/stealth/mod.rs` says public callers should go through `core::ecdh`, `core::kdf`, and `core::scan`.
- `crates/z00z_wallets/src/core/tx/mod.rs` says `core::tx::builder::*` and `core::tx::output_flow::*` are no longer part of the public caller surface.
- `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` says `recv_range(...)` is the preferred request-aware receive lane.
- `crates/z00z_storage/src/assets/README.MD` and `crates/z00z_storage/src/assets/types_record.rs` say `AssetStateRoot` is the canonical storage state commitment, `CheckRoot` is the checkpoint-facing root, and `CompatRoot` is the compatibility root for older migration paths.
- `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` says `SeqSecureRngProvider` is a simulator-only injection shim and must not be treated as production entropy.

## 🧩 Canonical Target Map

| Compatibility surface | Current owner | Canonical path | Deletion action | Comments |
| --- | --- | --- | --- | --- |
| Deprecated unix timestamp wrappers | `crates/z00z_utils/src/time/traits.rs` | `try_unix_timestamp*()` and `compat_unix_timestamp*()` on `TimeProvider` | Delete `unix_timestamp*()` wrappers and update callers to the explicit canonical or compatibility entrypoint they actually need. |  |
| Bytes-oriented stealth ECDH compatibility layer | `crates/z00z_crypto/src/ecdh_stealth.rs` | `z00z_crypto::ecdh::{generate_ephemeral_keypair, compute_stealth_dh_sender, recover_stealth_dh_receiver, derive_dh_key, validate_stealth_point}` | Delete the compatibility-only byte wrapper and route new callers to the point-based crypto API. |  |
| Backward-compatible point constructor | `crates/z00z_crypto/src/types.rs` | `Z00ZRistrettoPoint::try_from_bytes([u8; 32])` | Delete `from_canonical_bytes(&[u8])` call sites and switch them to the fixed-size canonical constructor. |  |
| Backward-compatible commitment constructor | `crates/z00z_crypto/src/types.rs` | `PedersenCommitment::from_canonical_bytes(bytes)` followed by `Z00ZCommitment::from_commitment(...)` | Delete `Z00ZCommitment::from_canonical_bytes()` and decode commitments at the call site before wrapping them. |  |
| Backward-compatible stealth key decoder alias | `crates/z00z_wallets/src/core/stealth/encoding.rs` | `decode_public_key()` | Delete `decode_ristretto_pk()` and keep `decode_public_key()` as the single decoder name. |  |
| Wallet-facing tag and leaf-ad duplicates | `crates/z00z_wallets/src/core/stealth/tag.rs` | `z00z_crypto::stealth_bind::{compute_tag16, compute_leaf_ad}` | Delete the wallet duplicate formulas and keep only the request-bound helpers that have distinct semantics. |  |
| Stateless stealth output compatibility lanes | `crates/z00z_wallets/src/core/stealth/output.rs` | `crates/z00z_wallets/src/core/stealth/output_build.rs::{build_output_ctx, build_output_ctx_with_serial, build_output_ctx_with_r}` and the public `build_tx_stealth_output*` / `build_tx_stealth_output_validated` / `build_card_stealth_output_validated` surfaces | Delete `build_stealth_sender_leaf()`, `build_stealth_bundle()`, and `build_stealth_bundle_with_rng()` as compatibility-only lanes. |  |
| Fail-closed tx compatibility path | `crates/z00z_wallets/src/core/tx/output_flow.rs` | `crates/z00z_wallets/src/core/tx/tx_digest.rs::build_tx_package_digest` and `crates/z00z_wallets/src/core/tx/tx_verifier.rs::TxVerifier` | Delete the compatibility path once callers move to the canonical digest and verification modules. |  |
| Deprecated tx builder shims | `crates/z00z_wallets/src/core/tx/mod.rs` | `crate::core::stealth::{build_stealth_leaf, build_stealth_leaf_with_blind, build_stealth_leaf_with_rng}` | Delete the test-only forwarding layer that still points at the stealth-owned leaf builders. |  |
| Backward-compatible address derivation helper | `crates/z00z_wallets/src/core/address/address_manager/address_manager_trait.rs` | `derive_wallet_keys()`, `create_dual_address()`, `derive_batch()`, `scan_checkpoint_with_requests()`, `scan_range_with_requests()` | Delete `derive_address()` once callers move to the richer manager API. |  |
| Single-asset receive fallback | `crates/z00z_wallets/src/services/wallet_service_actions_receive.rs` | `recv_range()` | Delete `recv_one()` if the compatibility receive lane is no longer required; otherwise keep it explicitly documented as fallback-only. |  |
| Legacy single/dual address compatibility types | `crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address.rs` and `crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address.rs` | `AddressManager::derive_wallet_keys()` and `AddressManager::create_dual_address()` | Delete new callers to the legacy address codec structs; prefer manager-owned derivation and Bech32 transport edges. |  |
| Compatibility storage root wrapper | `crates/z00z_storage/src/assets/types_record.rs` | `AssetStateRoot`, `CheckRoot`, and `RootApi` | Delete `CompatRoot` only after the migration proof no longer needs the older flat-row compatibility decode path. |  |
| Simulator-local deterministic RNG adapter | `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` | `WalletService::create_service_custom_output_directory(...)` with a caller-owned `SecureRngProvider` boundary, or `WalletService::with_output_dir(...)` for the default production path | Delete `SeqSecureRngProvider` from simulator transport once reproducibility is handled by a named test fixture instead of a simulator-owned entropy shim. |  |

## ⚠️ Open Gaps And Watchpoints

These items are compatibility-shaped, but they are not clean alias shims and should not be collapsed into the table above without a separate proof-backed deletion decision:

- The Phase 1 asset RPC methods in `crates/z00z_wallets/src/core/wallet/wallet_entity_asset_api.rs` are stub reachability methods, not alias wrappers. They still need a separate implementation decision before they can be treated as canonical production paths.
- The old address codec structs in `crates/z00z_wallets/src/core/address/z00z_address/` are still used as transport-visible compatibility types. If the phase wants to delete them entirely, the follow-up must define the replacement transport story first.
- The `Z00ZCommitment` and `Z00ZRistrettoPoint` backward-compatible constructors are used broadly across crypto, tx, and address code. Their deletion needs a deliberate call-site sweep so the repo does not end up with scattered partial migrations.

## 🧪 Validation

The shim inventory is complete when a workspace scan only returns canonical owners, explicit compatibility surfaces that are still intentionally retained, or the open gaps above.

Useful verification scan:

```bash
rg -n "compat_unix_timestamp|unix_timestamp\b|decode_ristretto_pk|build_stealth_sender_leaf|build_stealth_bundle|build_stealth_bundle_with_rng|build_output_leaf|build_output_with_blind|build_output_with_rng|recv_one\b|derive_address\b|CompatRoot\b|SeqSecureRngProvider|from_canonical_bytes\b" crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'
```

Expected result:

- remaining `unix_timestamp*` hits are only the explicit compatibility helpers or tests that still prove the transition
- `decode_ristretto_pk()` is gone
- stateless stealth output compatibility lanes are gone
- tx test-only shims are gone
- `recv_one()` is either gone or explicitly documented as fallback-only
- `CompatRoot` is either gone or still justified by a migration proof
- `SeqSecureRngProvider` is either gone or confined to a test-only fixture boundary

## ✅ Notes For The Next Phase

- Treat this document as the source of truth for shim deletion candidates in Phase 036.
- Do not invent new compatibility layers to replace the ones listed here.
- If a surface is not in the table and is not one of the open gaps, it is not a shim target for this slice.
