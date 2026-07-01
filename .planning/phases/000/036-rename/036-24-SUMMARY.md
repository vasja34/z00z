---
phase: 036-rename
plan: 24
type: execute
wave: 13
depends_on: [036-20, 036-23]
status: complete
completed: 2026-04-21
files_modified:
  - crates/z00z_crypto/src/lib.rs
  - crates/z00z_crypto/src/protocol/mod.rs
  - crates/z00z_crypto/src/vendor/mod.rs
  - crates/z00z_crypto/src/aead/mod.rs
  - crates/z00z_crypto/src/hash/mod.rs
  - crates/z00z_crypto/src/kdf/mod.rs
  - crates/z00z_crypto/src/backend/mod.rs
  - crates/z00z_crypto/tests/test_public_surface.rs
  - versions.yaml
---

# Phase 036 Plan 24: Crypto Path-Group Rehome Summary

## Scope

This summary records the execution state for `036-24-PLAN.md`, the self-contained a7 continuation that rehomes the remaining crypto module families into canonical directory-backed module roots.

## Outcome

`036-24` is complete and summary-backed.

The implementation moved the remaining crypto families onto directory-backed module roots under `crates/z00z_crypto/src/protocol/`, `vendor/`, `aead/`, `hash/`, `kdf/`, and `backend/` while keeping the public `z00z_crypto` facade stable.

The current tree now owns those families through `mod.rs` roots and their sibling implementation files. The previous top-level wrapper files for these families were removed as part of the rehome, but the exported module names and public reexports still resolve through the canonical directory roots.

The phase remains separate from `036-20`. This summary does not reopen that partial shim-removal boundary, and it does not claim broader Phase 036 closure.

## Repository Changes

Representative files in the implementation commit include:

- `crates/z00z_crypto/src/lib.rs`
- `crates/z00z_crypto/src/protocol/mod.rs`
- `crates/z00z_crypto/src/vendor/mod.rs`
- `crates/z00z_crypto/src/aead/mod.rs`
- `crates/z00z_crypto/src/hash/mod.rs`
- `crates/z00z_crypto/src/kdf/mod.rs`
- `crates/z00z_crypto/src/backend/mod.rs`
- `crates/z00z_crypto/tests/test_public_surface.rs`
- `crates/z00z_crypto/tests/test_claim_contract.rs`
- `versions.yaml`

## Validation

- `cargo test -p z00z_crypto --release --features test-fast --lib`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`: passed
- `rg -n 'pub mod commitments|pub mod ecdh|pub mod range_proofs|pub mod stealth_bind|pub mod zkpack|pub mod vendor|pub mod expert' crates/z00z_crypto/src`: passed
- `rg -n 'aead/mod|hash/mod|kdf/mod|backend/mod|protocol/mod|vendor/mod' crates/z00z_crypto/src`: passed

## Notes

- The path-group table in `036-a7_crypto-spec.md` is satisfied by the canonical directory roots now present in `crates/z00z_crypto/src/`.
- `036-20-SUMMARY.md` remains the authoritative partial boundary for the separate shim-removal continuation.
- This closeout records the path-group rehome truth only; it does not convert the continuation into a claim about whole-phase completion.
