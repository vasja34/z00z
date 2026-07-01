# Genesis Test Layout

This note documents the canonical Phase 063 genesis test layout.

## Canonical Roots

- integration target: `cargo test --release -p z00z_core --test genesis_tests`
- root harness: `tests/test_genesis.rs`
- module map: `tests/test_genesis_mod.rs`
- shared helpers: `tests/test_genesis_helpers.rs`
- owned fixtures directory: `tests/fixtures/`

All owned genesis test files live directly under `tests/` with
`test_genesis_<behavior>.rs` names. No owned nested genesis subtree remains.

## Coverage Shape

The root module map owns:

- reproducibility and determinism checks
- config parsing and validation
- integration and settlement-corpus flows
- claim flow, range proofs, and batch verification
- manifest, rights, policies, and vouchers regression coverage

Standalone root-level test crates keep the manifest-layout and guardrail
checks:

- `test_genesis_manifest_goldens.rs`
- `test_genesis_manifest_refs.rs`
- `test_live_guardrails.rs`

## Release Commands

```bash
cargo test --release -p z00z_core --test genesis_tests
cargo test --release -p z00z_core --test genesis_tests genesis_rights -- --nocapture
cargo test --release -p z00z_core --test test_genesis_manifest_goldens -- --nocapture
```

## Guardrails

- `tests/fixtures/` is the only owned subdirectory allowed under `tests/`.
- New genesis integration tests should use `test_genesis_<behavior>.rs`.
- Add new module wiring in `test_genesis_mod.rs` unless the file is
  intentionally a standalone root-level test crate.
