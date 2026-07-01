---
phase: 042-refactor-wallets
spec_id: 042-z00z-address-remove
status: draft
created: 2026-05-05
updated: 2026-05-05
owner: Z00Z Wallets
scope: stealth-only wallet and RPC migration
---

# Phase 042 z00z_address Removal Specification

This specification defines the removal path for the legacy `z00z_address` concept from the wallet crate when the product target is stealth-only and does not preserve backward compatibility.

The core decision is explicit: `z00z_address` is not a stealth primitive and must not remain a core wallet concept. It can be deleted only after all live callers are migrated to receiver-key, receiver-card, payment-request, and scan/output stealth surfaces.

## 🎯 1. Purpose And Scope

### ✅ Purpose

- Remove the old address/API layer as a production wallet concept.
- Move wallet-session derivation and recovery onto key derivation plus stealth receiver material.
- Rebind RPC derive, validate, label, and list flows away from legacy address strings.
- Preserve deterministic wallet recovery by keeping seed/BIP32/BIP44/path derivation where it feeds stealth receiver material.
- Delete `z00z_address` types, tests, docs, and public re-exports after all callers are migrated.

### ✅ In Scope

- `crates/z00z_wallets/src/address/z00z_address/**`
- `crates/z00z_wallets/src/address/z00z_address.tar.gz`
- `crates/z00z_wallets/src/address/mod.rs`
- `crates/z00z_wallets/src/key/mod.rs`
- `crates/z00z_wallets/src/lib.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_seed_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_snapshot.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_rotation.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_core.rs`
- `crates/z00z_wallets/src/services/wallet/types/wallet_service_types_state.rs`
- `crates/z00z_wallets/src/services/wallet/actions/wallet_service_actions_rpc.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/key.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_derive.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/support_tail.rs`
- `crates/z00z_wallets/src/wallet/snapshot/**`
- `crates/z00z_wallets/src/services/wallet/store/**` paths that create, restore, import, export, or snapshot `AddressDeriverState`.
- Tests, docs, and backup DTOs that still encode address-specific state.

### ❌ Out Of Scope

- Editing `crates/z00z_crypto/tari/**`.
- Replacing BIP32/BIP44 derivation itself.
- Redesigning stealth cryptographic primitives.
- Introducing legacy compatibility shims after the final removal gate.
- Preserving old `Z00ZSingleAddress` or `Z00ZDualAddress` wire schemas.

## 🔑 2. Definitions

| Term | Meaning |
| --- | --- |
| Legacy address | `Z00ZSingleAddress`, `Z00ZDualAddress`, `Z00ZAddressFeatures`, Bech32/URI/QR address encoding, and related strict parsers under `address/z00z_address`. |
| Stealth receiver material | Receiver-side keys and public metadata derived from wallet seed/path material, including `ReceiverKeys`, receiver card fields, receiver-card records, and payment requests. |
| Receiver card | Signed receiver publication material exported by `ReceiverKeys::export_receiver_card()` and serialized through `ReceiverCardRecord`. |
| Payment request | Compact stealth request payload created and validated through `PaymentRequest`/`ValidatedRequest` surfaces. |
| Derivation state | Wallet-session state that tracks deterministic key/path progress for recovery. It must remain key-derivation-owned, not address-owned. |
| Address label | Current label map keyed by legacy address string. This must become a receiver label keyed by a stealth receiver identifier. |
| No backward compatibility | Old address strings and old RPC response shapes do not need to continue working after the cutover. One-time migration may exist, but runtime compatibility must not remain. |

## 📌 3. Current Live Surfaces

The live tree currently proves `z00z_address` is not dead code. It is not stealth-core, but it is still used by service/RPC/public facades.

| Surface | Current role | Removal action |
| --- | --- | --- |
| `address/z00z_address/**` | Implements legacy single/dual address types, Bech32 codecs, feature flags, serde, transport, and tests. | Delete after caller migration. |
| `address/z00z_address.tar.gz` | Stale archive sibling of the legacy address module in the active source tree. | Classify explicitly during cleanup; delete it or move it to an approved archive location outside production source before closeout. |
| `address/mod.rs` | Re-exports `Z00ZAddressError`, `Z00ZAddressFeatures`, `Z00ZDualAddress`, `Z00ZSingleAddress`. | Remove re-exports and module. |
| `key/mod.rs` | Re-exports `Z00ZAddressFeatures` and `Z00ZSingleAddress` through key facade. | Replace with stealth receiver exports only. |
| `lib.rs` | Re-exports legacy address types from crate root. | Remove from public crate facade. |
| `wallet_service_session_derivation.rs` | Builds `AddressManagerImpl<KeyManagerImpl>`, derives public keys and dual addresses by BIP44 path, and persists address counters. | Replace with receiver derivation state built directly from key derivation. |
| `wallet_service_session_derivation_recovery.rs` | Included from `wallet_service_session_derivation.rs`; performs gap-limit scans using derived address public keys, lists cached addresses, exports account public material through the address deriver, and rebuilds cached address derivation state during rotation. | Replace with receiver/path discovery over stealth receiver identifiers and receiver-owned cached material. |
| `wallet_service_session_seed_derivation.rs`, `wallet_service_session_snapshot.rs`, `wallet_service_session_rotation.rs` | Source-residue files with overlapping address-derivation code that are not directly included by `wallet_service_session.rs` in the current module wiring. | Delete or reconcile during migration so source-shape gates do not leave an unused address model behind. |
| `wallet_service_types_core.rs` | Owns `address_labels`, `wallet_address_derivers`, and address-oriented type aliases. | Rename and reshape to receiver labels/receiver derivers. |
| `wallet_service_types_state.rs` | Defines `WalletAddressDeriverState` with `AddressManagerImpl<KeyManagerImpl>`. | Replace with `WalletReceiverDeriverState` or equivalent key/receiver-owned state. |
| `wallet_service_actions_rpc.rs` | Provides placeholder `list_addresses`, `validate_address`, and `label_address` service calls; some comments mark them `[NOT_IN_USE]`, but the dispatcher and service path still keep the address concept reachable. | Remove or replace with receiver/card/request methods. |
| `server_derive.rs` | `wallet.key.derive_key` returns `address`; `wallet.key.derive_dual_address` returns dual address; receiver card response still has an `address` display field generated from `format_address(&decoded.owner_handle)`. | Replace derived-address outputs with stealth receiver material; remove dual-address method. |
| `server_admin.rs` | Validates `Z00ZSingleAddress::parse_strict`, labels legacy addresses, and checks cached address list membership. | Replace with receiver-card/payment-request validation and receiver-label ownership checks. |
| `support.rs` and `support_tail.rs` | Convert cached BIP44 public keys into `Z00ZSingleAddress` strings for list responses and build paginated address DTOs. | Replace list entries with receiver identifiers and stealth public material. |
| `adapters/rpc/types/key.rs` | Defines `RuntimeDeriveKeyResponse`, `RuntimeDeriveDualAddressResponse`, `RuntimeAddressFilter`, `PersistAddressInfo`, `RuntimeListAddressesResponse`, `RuntimeValidateAddressResponse`, `RuntimeLabelAddressResponse`; several legacy comments already say `[NOT_IN_USE]`, but trait, dispatcher, implementation, and tests still require migration. | Replace address DTOs with receiver DTOs; delete legacy DTOs. |
| `wallet_dispatcher_wiring.rs` and `wallet_dispatcher_wiring_register.rs` | Register `wallet.key.derive_key`, `wallet.key.derive_dual_address`, `wallet.key.list_addresses`, `wallet.key.validate_address`, and `wallet.key.label_address`. | Register receiver replacements and delete old address method strings. |
| `WalletPersistenceState`, `WalletExportPack`, and wallet store flows | Persist, restore, export, and import `AddressDeriverState`; active session snapshot persistence also writes optional `.addr_cache` payloads from `AddressManagerImpl::export_cache`. | Rename or version snapshot/export fields around receiver derivation counters and remove `.addr_cache` address-manager payloads. |

## 🧭 4. Target Architecture

### ✅ Stealth-Only Receive Model

The wallet receive surface must be centered on these concepts:

- Seed/session unlock provides secret wallet material.
- BIP32/BIP44/path derivation remains the deterministic recovery substrate.
- Derived path material feeds receiver keys, receiver cards, payment requests, and scan output ownership.
- Public API returns receiver/card/request identifiers, not legacy Bech32 address strings.
- Labels attach to receiver-card or payment-request identifiers, not `Z00ZSingleAddress` strings.

### ✅ Blessed Receiver Surfaces

| Need | Target surface |
| --- | --- |
| Static wallet receiver identity | `WalletService::receiver_keys(...)` and `ReceiverKeys::export_receiver_card()` |
| Signed receiver publication | `ReceiverCard`, `ReceiverCardRecord`, `card_compact`, `registry_entry_id`, `card_epoch` |
| Payment/invoice-like receiver payload | `PaymentRequest`, `RequestParams`, `RuntimeCreatePaymentRequestResponse` |
| Request validation | `RuntimeValidatePaymentRequestResponse`, `ValidatedRequest`, `ValidationOutcome` |
| Output discovery | `StealthOutputScanner`, `WalletStealthOutput`, `ReceiveReport`, `ReceiveStatus` |
| Deterministic recovery | BIP39 seed -> BIP32/BIP44 path -> receiver key material and counters |

### ❌ Forbidden Target State

- No production call site may construct `Z00ZSingleAddress` or `Z00ZDualAddress`.
- No production RPC response may expose a legacy `address` field derived from `z00z_address`.
- No production validation path may parse legacy address strings.
- No key/session state may own an `AddressManagerImpl` only to produce legacy addresses.
- No final public facade may re-export `Z00ZAddressFeatures`, `Z00ZSingleAddress`, or `Z00ZDualAddress`.

## 📋 5. Requirements

| ID | Requirement |
| --- | --- |
| ZAR-001 | THE SYSTEM SHALL remove `z00z_address` as a production wallet concept after all callers are migrated. |
| ZAR-002 | THE SYSTEM SHALL keep deterministic seed/path recovery on key derivation, not on the legacy address layer. |
| ZAR-003 | THE SYSTEM SHALL replace wallet-session address derivation state with receiver derivation state. |
| ZAR-004 | THE SYSTEM SHALL replace RPC derive responses that return legacy address strings with stealth receiver material. |
| ZAR-005 | THE SYSTEM SHALL remove or rename RPC list/validate/label address methods so they operate on receiver cards, payment requests, or receiver identifiers. |
| ZAR-006 | THE SYSTEM SHALL migrate labels from address-string keys to receiver identifiers or deliberately drop them under the no-backward-compatibility policy. |
| ZAR-007 | THE SYSTEM SHALL update tests so stealth receiver/card/request behavior replaces legacy address codec behavior. |
| ZAR-008 | THE SYSTEM SHALL remove docs that teach `Z00ZSingleAddress` or `Z00ZDualAddress` as active wallet concepts. |
| ZAR-009 | THE SYSTEM SHALL fail closed when a legacy address string is submitted after the cutover. |
| ZAR-010 | THE SYSTEM SHALL keep BIP32/BIP44 modules if deterministic wallet recovery remains required. |

## 🧩 6. Replacement Contracts

### ✅ Receiver Derivation State

Replace the current address-owned state:

```text
WalletAddressDeriverState {
  address_manager: AddressManagerImpl<KeyManagerImpl>,
  next_payment_index: u32,
  next_change_index: u32,
}
```

with a receiver-owned state:

```text
WalletReceiverDeriverState {
  key_manager: KeyManagerImpl or dedicated BIP44 receiver deriver,
  next_external_index: u32,
  next_internal_index: u32,
  cached_receivers: path -> receiver public material,
}
```

The exact Rust names can differ, but the ownership rule is mandatory: derivation state may own key/path derivation and receiver public material; it must not own or depend on `z00z_address` types.

### ✅ Receiver Identifier

A receiver identifier must be deterministic, stable, and non-secret. Preferred options in order:

1. `registry_entry_id` from `ReceiverCardRecord` for published cards.
2. `owner_handle` hex for wallet-local receiver identity when card publication metadata is not needed.
3. `req_id` for payment-request-specific labels.
4. A domain-separated hash over canonical receiver-card or request bytes when a shorter local ID is required.

The chosen identifier must be documented in the implementation summary before labels are migrated.

### ✅ New RPC DTO Shape

New DTOs should be added before old DTOs are deleted. Recommended shape:

```text
RuntimeDeriveReceiverResponse {
  path: String,
  owner_handle: String,
  view_key: String,
  identity_key: Option<String>,
  receiver_id: String,
  card_compact: Option<String>,
  request_compact: Option<String>,
}

PersistReceiverInfo {
  receiver_id: String,
  path: String,
  owner_handle: String,
  view_key: String,
  used: bool,
  internal: bool,
  label: Option<String>,
  index: u32,
}
```

If a public field is named `address` in a receiver-card response only as display text, rename it to `owner_handle_display` or delete it. The canonical machine-readable fields are `owner_handle`, `card_compact`, and `registry_entry_id`.

## 🧱 7. Step-By-Step Removal Plan

### ✅ Step 0: Freeze The Inventory

1. Run a production-source inventory before edits:
   - `rg "z00z_address|Z00ZSingleAddress|Z00ZDualAddress|Z00ZAddressFeatures" crates/z00z_wallets/src crates/z00z_wallets/tests`
   - `rg "derive_dual_address|validate_address|label_address|list_addresses|PersistAddressInfo|RuntimeAddressFilter" crates/z00z_wallets/src crates/z00z_wallets/tests`
2. Split matches into these buckets:
   - Legacy implementation to delete.
   - Public facade/re-export to remove.
   - Wallet-session caller to migrate.
   - RPC caller to migrate.
   - Tests to delete or rewrite.
   - Historical docs/planning references to leave or update.
3. Add the inventory to the implementation summary when this spec is executed.

### ✅ Step 1: Define Receiver DTOs And API Names

1. Add receiver-oriented DTOs in `adapters/rpc/types/key.rs`.
2. Add receiver method names in `adapters/rpc/methods/key.rs`.
3. Keep old methods only until the same plan wave can migrate callers.
4. Recommended replacements:
   - `wallet.key.derive_key` -> `wallet.key.derive_receiver`.
   - `wallet.key.derive_dual_address` -> delete; use `wallet.key.get_receiver_card` or `wallet.key.create_payment_request`.
   - `wallet.key.list_addresses` -> `wallet.key.list_receivers`.
   - `wallet.key.validate_address` -> `wallet.key.validate_receiver_card` and/or existing `wallet.key.validate_payment_request`.
   - `wallet.key.label_address` -> `wallet.key.label_receiver`.
5. Because the product has no backward compatibility requirement, remove the old methods after internal callers and tests switch.

### ✅ Step 2: Rewrite Wallet-Session Derivation

1. In `wallet_service_types_state.rs`, replace `WalletAddressDeriverState` with receiver-owned derivation state.
2. In `wallet_service_types_core.rs`, rename state holders:
   - `WalletAddressDeriverHandle` -> receiver deriver handle.
   - `wallet_address_derivers` -> receiver derivers.
   - `wallet_address_deriver_counters` -> receiver derivation counters.
   - `address_labels` -> receiver labels.
3. In `wallet_service_session_derivation.rs`, replace `create_address_deriver_state(...)` with a function that initializes key/path derivation only.
4. Replace `get_create_wallet_address_deriver(...)` with receiver deriver creation from unlocked `seed_bip39`, persisted chain identity, and counters.
5. Replace `derive_public_key_for_path(...)` with a receiver-specific function that returns receiver material needed by the new DTOs.
6. Delete `derive_dual_address_for_path(...)` after RPC no longer calls it.
7. Preserve counter updates for external/payment and internal/change chains.
8. Keep persistence through `persist_snapshot_for_open_session(...)`; do not drop recovery determinism.
9. Audit `wallet_service_session_seed_derivation.rs`, `wallet_service_session_snapshot.rs`, and `wallet_service_session_rotation.rs`; delete them if they remain unwired duplicates, or migrate any intentionally live code before final source-shape gates run.

### ✅ Step 3: Rewrite Recovery And Gap Scan

1. In the active `wallet_service_session_derivation.rs` plus its included `wallet_service_session_derivation_recovery.rs`, replace address public-key cache scans with receiver identifier scans.
2. Change `AddressUsedOracle` from `(Bip44Path, [u8; 32])` to `(Bip44Path, ReceiverId or receiver public material)`.
3. Replace `list_cached_addresses(...)` with `list_cached_receivers(...)`.
4. Ensure gap-limit reconciliation updates the same counters as active derivation.
5. If current production cannot determine receiver usage by receiver id yet, make the oracle contract explicit and fail closed instead of silently marking everything unused.
6. Replace address-cache persistence (`.addr_cache` and `AddressManagerImpl::export_cache`) with receiver-cache persistence, or remove the cache if receiver discovery is recomputed deterministically.

### ✅ Step 4: Rebind RPC Derive Paths

1. In `server_derive.rs`, change `derive_key_impl(...)` so it returns receiver material without `Z00ZSingleAddress::from_public_key(...)`.
2. Delete `derive_dual_address_impl(...)` or replace it with receiver-card/payment-request derivation.
3. Keep `get_receiver_card_impl(...)` as the preferred receive surface, but remove or rename the display-only `address` field that is currently generated from `format_address(&decoded.owner_handle)`.
4. In `server.rs`, wire new RPC trait methods and remove old address methods after DTO migration.
5. Update `wallet_dispatcher_wiring.rs` and `wallet_dispatcher_wiring_register.rs`, where the current method strings are registered.

### ✅ Step 5: Rebind RPC Validate And Label Paths

1. In `server_admin.rs`, delete `Z00ZSingleAddress::parse_strict(...)` validation.
2. Implement validation for these stealth inputs:
   - receiver card compact record,
   - payment request compact payload,
   - receiver id if label-only local lookup is needed.
3. Replace `label_address_impl(...)` with label-by-receiver-id.
4. Replace cached-address membership checks with cached receiver/path membership checks.
5. Keep label validation rules (`non-empty`, max length) unless a new product rule changes them.
6. In `support.rs` and `support_tail.rs`, replace `prepare_addrs(...)`, `reuse_counts(...)`, `lookup_address_label(...)`, `addr_page(...)`, and `list_addrs_page(...)` with receiver equivalents.

### ✅ Step 6: Migrate Persistence And Backup State

1. Inspect `WalletPersistenceState`, `WalletExportPack`, backup importer/exporter tests, and snapshot serialization for `AddressDeriverState` and address label fields.
2. Choose one no-backward-compatibility path:
   - Epoch bump: reject old wallet snapshots/backups and require fresh stealth-only wallet state.
   - One-shot converter: read old state once, convert counters and labels, then write only stealth-only state.
3. Do not keep runtime old-schema compatibility after migration.
4. Preserve counter semantics so deterministic recovery does not reuse or skip receiver paths accidentally.
5. If labels are migrated, map old `address -> label` only when a deterministic receiver id can be proven for the same path. Otherwise drop labels and record the loss as intentional under no backward compatibility.

### ✅ Step 7: Remove Public Facades

1. Remove `pub mod z00z_address` from `address/mod.rs`.
2. Remove legacy exports from `address/mod.rs`:
   - `Z00ZAddressError`
   - `Z00ZAddressFeatures`
   - `Z00ZDualAddress`
   - `Z00ZSingleAddress`
3. Remove legacy exports from `key/mod.rs`.
4. Remove crate-root legacy exports from `lib.rs`.
5. Delete `address/z00z_address/**` only after production source has no live imports.
6. Delete address-manager methods that only construct `Z00ZDualAddress` if no other live behavior uses them.

### ✅ Step 8: Rewrite Tests

1. Delete pure legacy address codec tests when the implementation is deleted.
2. Rewrite RPC key tests around receiver DTOs:
   - derive receiver returns stable receiver id and key material,
   - invalid receiver card/request fails closed,
   - label receiver requires session and owned receiver id,
   - list receivers paginates and filters without address strings.
3. Add source-shape guards that fail if production Rust reintroduces `Z00ZSingleAddress`, `Z00ZDualAddress`, `Z00ZAddressFeatures`, or `z00z_address`.
4. Update backup/import/export tests to assert receiver derivation counters, not address counters.
5. Preserve release-style simulator checks if wallet RPC flows are used in Scenario 1.

### ✅ Step 9: Update Docs

1. Delete or archive active user docs that describe `Z00ZSingleAddress` or `Z00ZDualAddress` as wallet concepts.
2. Update key/BIP docs to say BIP32/BIP44 feed stealth receiver derivation, not addresses.
3. Update RPC docs and examples:
   - receiver card for reusable receive identity,
   - payment request for invoice/request-bound receive,
   - scan output reports for received asset discovery.
4. Update planning docs to mark `z00z_address` as removed from production, not merely moved.

### ✅ Step 10: Final Deletion Gate

The implementation may delete `address/z00z_address/**` only when all checks pass:

- No production Rust match for legacy names:
  - `Z00ZSingleAddress`
  - `Z00ZDualAddress`
  - `Z00ZAddressFeatures`
  - `z00z_address`
- No active RPC method name exposes legacy address flow.
- No public crate facade exports legacy address types.
- Wallet snapshot/backup structs no longer expose address-specific state names.
- No stale sibling archive such as `crates/z00z_wallets/src/address/z00z_address.tar.gz` remains in production source unless it is moved to a deliberately documented archive location outside active Rust source.
- Tests prove receiver derivation and recovery remain deterministic.
- Docs no longer teach legacy addresses as active API.

## 🧪 8. Validation Strategy

### ✅ Focused Validation

Run after each migration wave:

```bash
cargo fmt
cargo test -p z00z_wallets --lib --tests --no-run
cargo test -p z00z_wallets --test test_rpc_key_derive_e2e -- --test-threads=1 --nocapture
cargo test -p z00z_wallets --test test_rpc_types_serialization -- --nocapture
cargo test -p z00z_wallets --test test_rpc_wiring_spec_a -- --nocapture
cargo test -p z00z_wallets wallet_service_tests --lib -- --nocapture
cargo test -p z00z_wallets --test test_receiver_card_record -- --nocapture
cargo test -p z00z_wallets --test test_stealth_request -- --nocapture
```

These are live targets or live inner-module filters in the current tree. Replace them only when the implementation moves or deletes the corresponding surfaces; do not invent success if a target name has drifted.

### ✅ Release-Style Validation

Run before closeout:

```bash
cargo test -p z00z_wallets --release --features test-fast --all-targets
cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump
cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump
```

### ✅ Source-Shape Gates

Add or run a source-shape guard equivalent to:

```bash
rg "Z00ZSingleAddress|Z00ZDualAddress|Z00ZAddressFeatures|z00z_address" crates/z00z_wallets/src crates/z00z_wallets/tests --glob '!**/test_phase042_source_shape*.rs'
rg "derive_dual_address|validate_address|label_address|list_addresses|PersistAddressInfo|RuntimeAddressFilter" crates/z00z_wallets/src/adapters/rpc crates/z00z_wallets/tests --glob '!**/test_phase042_source_shape*.rs'
test ! -e crates/z00z_wallets/src/address/z00z_address.tar.gz
```

The final gate may allow historical `.planning` references and the guard test file that contains the forbidden strings by design, but production Rust and active public docs must be clean.

## 🚨 9. Risks And Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Derivation counters drift during rename from address to receiver. | Recovery can skip or reuse receive paths. | Preserve counter semantics and add deterministic recovery tests. |
| Labels cannot be mapped from legacy address strings to receiver ids. | User-visible labels may be lost. | Choose epoch bump or one-shot converter explicitly; do not silently fake mappings. |
| RPC methods remain address-shaped after internals migrate. | Public API keeps legacy concept alive. | Delete/rename methods and DTOs in the same migration chain. |
| `AddressManagerImpl` remains as hidden dependency. | Legacy address layer survives under a new name. | Source-shape guard for address manager methods that construct `Z00ZDualAddress`. |
| BIP32/BIP44 gets removed accidentally. | Deterministic wallet recovery breaks. | Treat BIP32/BIP44 as key derivation substrate, not legacy address logic. |
| Receiver card display field named `address` causes concept drift. | API still teaches users an address model. | Rename to `owner_handle_display` or remove. |
| Old docs stay active. | Future agents reintroduce address layer. | Delete/update active docs and add wording guard if needed. |

## ✅ 10. Acceptance Criteria

| ID | Criteria |
| --- | --- |
| AC-001 | WHEN wallet receiver material is derived by path, THE SYSTEM SHALL derive it from seed/key/path state without using `z00z_address`. |
| AC-002 | WHEN RPC derive is called after migration, THE SYSTEM SHALL return receiver/card/request material and SHALL NOT return a legacy address string. |
| AC-003 | WHEN validation receives a legacy address string after cutover, THE SYSTEM SHALL reject it as unsupported or unknown input. |
| AC-004 | WHEN a receiver card or payment request is validated, THE SYSTEM SHALL use the stealth validation path and return fail-closed errors on malformed data. |
| AC-005 | WHEN a label is assigned, THE SYSTEM SHALL bind it to a receiver id, card record id, owner handle, or request id, not to a legacy address string. |
| AC-006 | WHEN wallet recovery reconciles derivation progress, THE SYSTEM SHALL update receiver counters with the same deterministic safety as the old address counters. |
| AC-007 | WHEN production source is scanned, THE SYSTEM SHALL contain no live `Z00ZSingleAddress`, `Z00ZDualAddress`, `Z00ZAddressFeatures`, or `z00z_address` imports. |
| AC-008 | WHEN public crate exports are inspected, THE SYSTEM SHALL expose stealth receiver/card/request surfaces and SHALL NOT expose legacy address types. |
| AC-009 | WHEN tests run, THE SYSTEM SHALL prove receiver derivation, receiver validation, receiver labels, and recovery counters without relying on legacy address tests. |
| AC-010 | WHEN docs are inspected, active wallet/RPC docs SHALL describe receiver cards, payment requests, and scan outputs as the receive model. |

## 🧾 11. Execution Order Summary

1. Freeze inventory.
2. Add receiver DTOs and method names.
3. Rewrite session derivation state.
4. Rewrite recovery and gap scan.
5. Rebind RPC derive.
6. Rebind RPC validate/list/label.
7. Migrate or intentionally invalidate address-labeled data.
8. Remove public facades.
9. Delete implementation and legacy tests.
10. Run final source-shape and release gates.
11. Write summary with inventory, migration decision, validation evidence, and remaining blockers.

## 📚 12. Related Artifacts

- `.planning/phases/042-refactor-wallets/042-CONTEXT.md`
- `.planning/phases/042-refactor-wallets/042-core-refactore-spec.md`
- `.planning/phases/042-refactor-wallets/042-services-refactore-spec.md`
- `crates/z00z_wallets/src/address/mod.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation.rs`
- `crates/z00z_wallets/src/services/wallet/session/wallet_service_session_derivation_recovery.rs`
- `crates/z00z_wallets/src/wallet/snapshot/snapshot_types.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_support.rs`
- `crates/z00z_wallets/src/services/wallet/store/wallet_service_store_persistence_pack_snapshot.rs`
- `crates/z00z_wallets/src/adapters/rpc/wallet_dispatcher_wiring_register.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_derive.rs`
- `crates/z00z_wallets/src/adapters/rpc/methods/key_impl/server_admin.rs`
- `crates/z00z_wallets/src/adapters/rpc/types/key.rs`
- `crates/z00z_wallets/tests/test_rpc_key_derive_e2e.rs`
- `crates/z00z_wallets/tests/test_rpc_types_serialization.rs`
- `crates/z00z_wallets/tests/test_rpc_wiring_spec_a.rs`
- `crates/z00z_wallets/tests/test_receiver_card_record.rs`
- `crates/z00z_wallets/tests/test_stealth_request.rs`

## ⚠️ 13. Planning Note

The original Phase 042 context says the phase is structural-only. This document is intentionally different: it is a behavior and public-API migration spec for a stealth-only product. It must be executed only after accepting that no backward compatibility is required for legacy address APIs.
