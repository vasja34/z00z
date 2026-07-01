# Deferred Items

## 2026-04-08

- Unrelated broad-gate blocker: full workspace `cargo test --release --features test-fast --features wallet_debug_dump` can still reach protected vendor or doctest failures under `crates/z00z_crypto/tari/crypto/`. This class is already documented in prior planning artifacts as out of scope for Phase 033 Plan 13 and was not modified by the Plan 13 code or test changes.
- Unrelated broad-gate blocker: the same full workspace gate still fails after Plan 15 on `crates/z00z_crypto/tari/crypto` doctests with duplicate `tari_utilities` trait imports (`ByteArray`/`Hex`) inside protected vendor scope. Plan 15 touched only storage, simulator, wallet test, and control-artifact wording, so this failure remains out of scope and is not evidence of a Plan 15 regression.
