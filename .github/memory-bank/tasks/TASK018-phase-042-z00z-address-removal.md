# TASK018 - Phase 042 z00z_address Removal

**Status:** Completed  
**Added:** 2026-05-05  
**Updated:** 2026-05-05

## Original Request

Execute the Phase 042 stealth-only removal of the legacy `z00z_address` surface from the wallet crate. The work must remain no-backcompat, start from the verified source spec and execute plan, and keep `bootstrap_tests.sh` as the first validation gate.

## Thought Process

The repository audit confirmed that `z00z_address` was still live through wallet RPC routes, session derivation, recovery, snapshot/export/import flows, public facades, tests, docs, and the archive sibling. The safe path was a receiver-oriented migration in waves, not a broad speculative rename.

Wave 1 established the control artifacts, validated the bootstrap gate, and proved the first receiver-card field rename. The final migration then physically moved the live receive stack from `src/address/` to `src/receiver/`, removed the `core::receiver` alias shim, and cleaned active API/schema/policy terminology while preserving the deterministic BIP32/BIP44 recovery substrate and frozen cryptographic domain labels.

## Implementation Plan

- Migrate receiver-facing RPC routes and method names away from legacy address vocabulary.
- Update session derivation, recovery, snapshot, and persistence surfaces to the receiver model.
- Remove stale public facades, tests, docs, and archive artifacts after live callers move.
- Add and enforce source-shape guards so legacy address shapes do not return.

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

| ID | Description | Status | Updated | Notes |
| ---- | ------------- | -------- | -------- | ----- |
| 1.1 | Validate bootstrap and control artifacts | Complete | 2026-05-05 | `bootstrap_tests.sh` passed; source spec, plan, coverage ledger, and wave log are in sync. |
| 1.2 | Remove receiver-card address-shaped output | Complete | 2026-05-05 | `RuntimeGetReceiverCardResponse.address` is now `owner_handle_display`; focused tests passed. |
| 1.3 | Migrate receiver RPC routes and names | Complete | 2026-05-05 | The active derive/list/validate/label routes are receiver-native, `key/mod.rs` and `lib.rs` no longer leak `Z00Z*` address exports, and `core.rs` directly re-exports `crate::receiver`. |
| 1.4 | Remove legacy session and persistence surfaces | Complete | 2026-05-05 | The active runtime lane uses `ReceiverManagerImpl`, `receiver_manager`, `.receiver_cache`, receiver-native snapshot/session naming, and receiver-native DB schema/index terminology. |
| 1.5 | Delete stale docs and archive artifacts | Complete | 2026-05-05 | The live receive stack is physically under `src/receiver/`; `src/address/` is absent; stale active docs/tests were updated; the old address conversion helper binary is absent; remaining address hits are classified frozen domains, BIP44 vocabulary, external Tari refs, or generic non-receiver docs. |

## Progress Log

### 2026-05-05 Final Physical Receiver Migration

- Physically renamed `crates/z00z_wallets/src/address/` to `crates/z00z_wallets/src/receiver/`.
- Removed the compatibility-style `core::receiver` shim over `crate::address`; `src/core.rs` now directly re-exports `crate::receiver`.
- Updated `src/lib.rs` to declare `pub mod receiver;` and removed the active `address` module declaration.
- Renamed active receiver/API/schema/policy terminology including `format_receiver_handle`, `return_receiver`, `receiver_mode`, `allowed_recipients`, `ReceiverByKind`, `INDEX_RECEIVER_BY_KIND_TABLE`, `index_receiver_by_kind`, and `index.receiver_by_kind`.
- Rechecked receiver and stealth cryptographic flow for card/request signatures, request-bound scan context, owner tags, tag16 filtering, KDF binding, direct scan, sender output self-validation, and constant-time tag comparisons.
- Ran `cargo fmt -p z00z_wallets`.
- Ran `cargo check -p z00z_wallets --all-targets --features test-params-fast` successfully.
- Ran strict generated-dir-excluded residue scans and classified the only remaining `address` hits as frozen domain labels, BIP44 derivation vocabulary, external Tari reference docs, or generic non-receiver docs.
- Ran `cargo test -p z00z_wallets --release --features test-params-fast --features wallet_debug_tools`; the release suite passed through final doc-tests with `32 passed`, `0 failed`, and `10 ignored`.

### 2026-05-05 Continued Runtime And Source Cleanup

- Verified the Phase 042 source spec and execute plan.
- Ran `bootstrap_tests.sh` first and confirmed the bootstrap gate passed.
- Created the Phase 042 coverage ledger and wave log.
- Renamed the receiver-card response display field to `owner_handle_display`.
- Added the receiver-oriented `wallet.key.list_receivers` alias.
- Added the receiver-oriented validation and label aliases.
- Switched the receiver validation alias to compact card validation and verified it.
- Validated the receiver-card slice with focused wallet tests.
- Updated the memory-bank dashboard and progress note to reflect the active Phase 042 state.

### 2026-05-05

- Rebound the active wallet runtime path to `ReceiverManagerImpl`, `receiver_manager`, and `.receiver_cache` vocabulary while keeping the compile gate green.
- Removed the crate-root and key-module `Z00Z*` address re-export leaks.
- Renamed the public/backing address facade to `stealth_address`.
- Renamed the backing `address/z00z_address/` directory to `address/stealth_address/`.
- Renamed the internal `z00z_address_*` helper files and their source-shape assertions to stealth-native filenames.
- Verified that `crates/z00z_wallets/src/**/*.rs` no longer contains `z00z_address` routing literals.
- Narrowed the honest remaining blocker set to the internal `AddressManagerImpl` / `Z00Z*` type family plus tests/docs cleanup.
