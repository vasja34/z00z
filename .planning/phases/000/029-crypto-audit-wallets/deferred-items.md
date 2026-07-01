<!-- markdownlint-disable MD003 MD007 MD022 MD031 MD032 MD033 MD041 MD047 -->
---
phase: 029-crypto-audit-wallets
artifact: deferred-items
status: open
updated_at: 2026-03-30
---

## Deferred Items

📌 Out-of-scope verification blocker discovered during `029-01` automated checks:

| Area | Evidence | Why deferred |
| --- | --- | --- |
| `cargo test --release --features test-fast --features wallet_debug_dump` doctests | Fails in `crates/z00z_crypto/tari/**` doctests with duplicate `tari_utilities` trait/version mismatches (`from_hex`, `from_canonical_bytes`, `ByteArray`, `Hex`) | Phase 029 Gate 0 changed only planning artifacts; vendor-adjacent `z00z_crypto/tari` doctest repair is unrelated to the current wallet-audit reconciliation scope |
