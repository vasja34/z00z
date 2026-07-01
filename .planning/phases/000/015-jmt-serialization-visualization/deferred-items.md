# Deferred Items

## 2026-03-23

- Resolved build blocker: the prior `lzma-rust2`/`crc` mismatch is no longer deferred.
  Lockfile resolution was normalized so the `z00z_core -> zip -> lzma-rust2`
  path now builds with `zip v7.0.0` and `crc v3.3.0`, removing the stale
  `crc v2.1.0` API conflict.
- Validation after the fix:
  - `cargo check -p z00z_storage --lib` passes.
  - `cargo test -p z00z_storage --lib -- --nocapture` passes.
  - `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --tests -- --nocapture` passes.
