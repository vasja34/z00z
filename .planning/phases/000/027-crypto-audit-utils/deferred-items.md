# Deferred Items

## 2026-03-29

- The earlier `z00z_simulator` acceptance blocker is resolved: `cargo test -p z00z_simulator --release --features test-fast --test test_claim_acceptance -- --nocapture` is now green after Stage 2 RPC-log parsing was updated for persisted prefixed logger lines.
- Stage 4 simulator integration fallout from the same persisted logger format is resolved: `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage4_bob_flow --test test_stage4_selection -- --nocapture` is now green after the shared Stage 4 RPC-log reader was updated.
- The later `z00z_wallets` release-gate fallout is resolved: stale wallet fixtures and file-backed RPC-log readers were aligned to current contracts, and `cargo test --release --features test-fast --features wallet_debug_dump` now passes.
- Phase 027 has no active deferred blocker remaining.
