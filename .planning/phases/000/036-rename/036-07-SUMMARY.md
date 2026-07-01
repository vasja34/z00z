# 036-07 Summary

## Scope

This summary records the completion state for `036-07-PLAN.md`, covering task
`036-04 Rename Current Single-Version Internal And Persisted Identifiers Without Changing Encoded Values`.

## Outcome

Plan 07 is closed for the active Phase 036 slice.

The Step 3 single-version identifier wave is now complete only on the raw rows
owned by the canonical spec: `36-43`, `45-47`, and `52`. The Rust identifier
surface now uses unsuffixed current names, while the live AAD framing bytes,
schema-key literals, stored numeric versions, receipt schema byte, and fee
model literal remain unchanged. In particular, `z00z.aead.v1\0`,
`wallet.integrity.v1`, `fee-weight-v1`, and version byte `1` were preserved.

## Repository Changes

- `crates/z00z_crypto/src/aead_aad.rs` now exposes `build_aad` as the single
  current helper for the existing v1-framed AAD contract without changing the
  framing bytes.
- `crates/z00z_wallets/src/db/index_codecs.rs`,
  `crates/z00z_wallets/src/db/index_codecs_tx_time.rs`, and
  `crates/z00z_wallets/src/db/index_codecs_body.rs` now use unsuffixed current
  index-key version identifiers while preserving the encoded version bytes.
- `crates/z00z_wallets/src/db/schema_keys.rs`,
  `crates/z00z_wallets/src/wasm/schema_keys.rs`,
  `crates/z00z_wallets/src/wasm/mod.rs`,
  `crates/z00z_wallets/src/db/redb_wallet_store.rs`,
  `crates/z00z_wallets/src/db/redb_wallet_store_crypto_ops.rs`, and
  `crates/z00z_wallets/src/db/wallet_validate.rs` now use
  `META_WALLET_INTEGRITY` while preserving the literal
  `wallet.integrity.v1` across native and wasm mirrors.
- `crates/z00z_wallets/src/core/key/seed_cipher_container.rs`,
  `crates/z00z_wallets/src/core/key/seed_cipher_container_crypto.rs`,
  `crates/z00z_wallets/src/core/key/seed_cipher_persistence.rs`,
  `crates/z00z_wallets/src/core/key/seed_cipher_basic_tests.rs`,
  `crates/z00z_wallets/src/core/key/seed_cipher_metadata_tests.rs`, and
  `crates/z00z_wallets/tests/test_key_manager.rs` now use `VERSION` and
  `AAD_VERSION` while preserving the persisted container byte values.
- `crates/z00z_wallets/src/db/redb_wallet_store_codecs.rs`,
  `crates/z00z_wallets/src/db/tests/redb_wallet_store.rs`,
  `crates/z00z_wallets/src/core/claim/claim_receipt.rs`,
  `crates/z00z_wallets/src/core/tx/fee_estimator.rs`,
  `crates/z00z_wallets/src/core/tx/fee_estimator_tests.rs`,
  `crates/z00z_wallets/tests/test_tx_fee.rs`,
  `crates/z00z_wallets/src/adapters/rpc/methods/tx_rpc_storage.rs`,
  `crates/z00z_wallets/src/core/security/password.rs`, and
  `crates/z00z_wallets/src/core/security/password_checks.rs` now use
  unsuffixed current identifiers while preserving the same schema bytes,
  decode shapes, and literal payload contracts.
- `.planning/phases/036-rename/036-TODO-2.md` now truthfully marks the Step 3
  checklist and coverage row complete after the bounded review and validation
  loop.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test --release --features test-fast --features wallet_debug_dump`:
  failed outside Plan 07 scope in read-only vendor
  `crates/z00z_crypto/tari/crypto/` doctests because multiple
  `tari_utilities` versions break `tari_crypto --doc`
- `cargo test -p z00z_wallets --release --features test-fast --test test_redb_wlt_open`:
  passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_fee`:
  passed
- `rg -n --glob '!*.backup' "build_aad_v1|TX_TIME_KEY_VERSION_V1|SEMANTIC_KEY_VERSION_V1|META_WALLET_INTEGRITY_V1|VERSION_V1|AAD_VERSION_V1|OBJECT_PAYLOAD_HEADER_VERSION_V1|CLAIM_SCHEMA_V1|FEE_WGT_VER_V1|TxStoreMetaV1|DENYLIST_BLOOM_V1|wallet\.integrity\.v1|fee-weight-v1" crates/z00z_crypto/src crates/z00z_wallets/src crates/z00z_wallets/tests`:
  passed, with old Step 3 internal spellings absent from live source and test
  files and the preserved literals retained only at their current values
- editor diagnostics on all Plan 07 modified Rust files and the updated TODO
  section: clean

## Review Loop

The review loop closed truthfully in three passes:

1. review pass 1 found no significant issues in the bounded Step 3 slice after
   implementation reconciliation had already resolved the rename-collision
   fallout locally
2. review pass 2 found no significant issues
3. review pass 3 found no significant issues, making passes 2 and 3 the two
   consecutive clean review runs required by the plan verify gate

The exact runtime commands for `/crypto-architect`, `/security-audit`, and
`/doublecheck` were not directly available in this environment, so the review
evidence used the repo-local best-effort path: canonical spec rereads,
symbol-usage inspection, residue scans, editor diagnostics, and targeted test
reruns.

## Current Boundary

This summary closes only Plan 07 of Phase 036. It does not claim execution of
the later Step 4 through Step 7 hold, cleanup, or validation waves now queued
under `036-08-PLAN.md` through `036-10-PLAN.md`.
