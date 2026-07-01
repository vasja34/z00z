---
phase: 036-rename
plan: 22
status: completed
updated: 2026-04-21
---

# 036-22 Summary

## 🎯 Scope

This summary records the completed `036-22` execution sweep for `.planning/phases/036-rename/036-22-PLAN.md`, rooted in `.planning/phases/036-rename/036-a5_hashdomain-spec.md`.

## ✅ Outcome

`036-22` is complete. The continuation executed the canonical hash-domain migration as a real same-version byte-contract update across the 27 ledger-owned rows and their direct compile/test fallout.

- REQ-001 through REQ-003 are closed by repository state: every one of the 27 owner rows from `.planning/phases/036-rename/036-a5_hashdomain-spec.md` is implemented and the embedded ledger in `.planning/phases/036-rename/036-22-PLAN.md` is now fully marked `✅`.
- REQ-004 stays satisfied in the narrow sense owned by the ledger: no Tari vendor code changed, all `hash_domain!` call shapes remain intact, and every affected version number remains `1` while the live byte contract is the canonical dotted namespace form.
- One in-scope validation defect was corrected during closeout: `crates/z00z_crypto/tests/zkpack_domain_verification.rs` now again asserts the real Tari `domain_separation_tag("")` contract, which appends `.v1` on top of the declared `*.v1` base domain.

## 🔍 Execution Checkpoint

### ✅ Owner And Runtime Sweep

- The owner-file domain ledger was normalized in `crates/z00z_crypto/src/domains.rs`, including the semantic type renames `KdhDomain -> DhKeyDomain`, `ReqDomain -> PaymentRequestDomain`, and `RcardDomain -> ReceiverCardDomain`.
- The adjacent KDF salt contract was normalized in `crates/z00z_crypto/src/kdf.rs` with the full-word identifiers `KDF_CONSENSUS_SALT`, `KDF_WALLET_SALT`, and `KDF_WALLET_VARIABLE_SALT`.
- Direct runtime consumers are on the canonical names and literals in `crates/z00z_crypto/src/ecdh.rs`, `crates/z00z_wallets/src/core/stealth/ecdh.rs`, `crates/z00z_wallets/src/core/address/stealth_request.rs`, and `crates/z00z_wallets/src/core/address/stealth_card.rs`.
- Crypto-owned mirrors aligned with the row-owned surface in `crates/z00z_crypto/src/hash_domains.rs`, `crates/z00z_crypto/tests/test_hash_policy.rs`, and `crates/z00z_crypto/tests/zkpack_domain_verification.rs`.

### ✅ Row-Owned Scan Proof

- The row-owned scan for stale owner literals and KDF identifiers returned zero hits:
  `rg -n 'hash_domain!\(.*"Z00Z/|\bKDF_CONS_SALT\b|\bKDF_WLT_SALT\b|\bKDF_WLT_VAR_SALT\b' crates/z00z_crypto/src/domains.rs crates/z00z_crypto/src/kdf.rs`
- The direct-consumer row-owned scan for stale renamed symbols and legacy row literals returned zero hits across the consumer and mirror files owned by the plan.

## 🧪 Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`: passed
- `cargo test -p z00z_crypto --release --features test-fast --test zkpack_domain_verification -- --nocapture`: passed
- `cargo test -p z00z_crypto --release --features test-fast --test test_hash_policy -- --nocapture`: passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_kdf -- --nocapture`: passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_ecdh -- --nocapture`: passed
- `cargo test -p z00z_wallets --release --features test-fast --test test_tx_stealth_flow -- --nocapture`: passed

## ⚠️ Broad Spec Discovery Scan

- The broad spec discovery scan was rerun exactly as required:
  `rg -n 'hash_domain!\(|KDF_CONS_SALT|KDF_WLT_SALT|KDF_WLT_VAR_SALT|Z00Z/' crates --glob '*.rs' --glob '!crates/z00z_crypto/tari/**'`
- Broad spec discovery scan result: non-ledger out-of-scope residuals still exist and are recorded here instead of being silently absorbed into `036-22`.
- out-of-scope residual: `crates/z00z_crypto/src/hash_domains.rs` still carries `Z00Z/IDENTITY` compatibility entries.
- out-of-scope residual: `crates/z00z_wallets/src/core/stealth/ecdh.rs` still carries `Z00Z/S_OUT` in the wallet-local `SOutProdDomain` call path.
- out-of-scope residual: `crates/z00z_core/src/assets/leaf.rs` still carries `Z00Z/PERF/ASSET`, `Z00Z/PERF/RSK`, and `Z00Z/PERF/TAG` performance-only literals.
- out-of-scope residual: `crates/z00z_core/src/assets/asset_validation.rs` still carries `Z00Z/ASSET_SECRET`.
- out-of-scope residual: `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl_snapshot.rs` still carries `Z00Z/Wallet/Cache/MAC`.
- out-of-scope residual: `crates/z00z_wallets/src/core/tx/prover.rs`, `crates/z00z_wallets/src/core/tx/spend_rules.rs`, and `crates/z00z_wallets/src/core/tx/output_flow.rs` still carry `Z00Z/SPEND_AUTH_V1`, `Z00Z/BAL`, and `Z00Z/TXPKG_WIRE` respectively.
- out-of-scope residual: `crates/z00z_crypto/tests/test_h2scalar.rs` still carries legacy compatibility test strings `Z00Z/IDENTITY` and `Z00Z/TEST`.
- `crates/z00z_crypto/src/hash_policy.rs` still intentionally contains the `domain.starts_with(b"Z00Z/")` guard and remains part of compatibility policy rather than a missed `036-22` owner row.

## ✅ Acceptance Criteria

- AC-001: satisfied. The owner ledger rows now use the canonical dotted lower-case namespace form.
- AC-002: satisfied. The 27-row ledger still enumerates the executed owner surfaces in source order and is now fully marked complete.
- AC-003: satisfied. The legacy KDF salt identifiers are absent from live non-Tari Rust sources.
- AC-004: satisfied. No Tari vendor file under `crates/z00z_crypto/tari/` was modified.

## 🧱 Boundary

- `036-22` closes only the self-contained hash-domain continuation rooted in `.planning/phases/036-rename/036-a5_hashdomain-spec.md`.
- Phase 036 remains open overall because `036-20-SUMMARY.md` still records the separate partial shim-removal boundary.
- This summary supersedes the earlier blocked checkpoint for `036-22`, but it does not supersede or narrow the still-open `036-20` truth.

## 📌 Canonical Artifact Sync

- `.planning/phases/036-rename/036-22-PLAN.md`
- `.planning/phases/036-rename/036-a5_hashdomain-spec.md`
- `.planning/phases/036-rename/036-22-SUMMARY.md`
<!-- End of 036-22 summary -->
