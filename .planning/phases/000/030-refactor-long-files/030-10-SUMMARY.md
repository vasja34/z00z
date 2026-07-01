---
phase: 030-refactor-long-files
plan: 10
subsystem: wallets-normalization
tags: [rust, wallets, facade, docs, rustdoc, planning, verification]
requires:
  - phase: 030-01
    provides: stable wallet db facade after redb split
  - phase: 030-05
    provides: stable address facade for caller normalization
  - phase: 030-06
    provides: stable key facade for caller normalization
  - phase: 030-07
    provides: stable service facade for wallet orchestration
  - phase: 030-09
    provides: stable tx and rpc facades before public path cleanup
provides:
  - normalized wallet caller-visible paths onto shallow db, services, core::address, core::key, and core::tx facades
  - synchronized wallet-facing docs and internal walkthrough labels with the final shallow surface
  - recorded caller-inventory evidence plus green wallet and workspace verification on the final source state
affects: [030-11, 030-12, z00z_wallets, wallet-docs, planning]
tech-stack:
  added: []
  patterns: [shallow facade normalization, internal-walkthrough labeling, facade-first rustdoc examples, bootstrap-first release verification]
key-files:
  created:
    - .planning/phases/030-refactor-long-files/030-10-SUMMARY.md
  modified:
    - crates/z00z_wallets/src/lib.rs
    - crates/z00z_wallets/src/db/mod.rs
    - crates/z00z_wallets/src/services/mod.rs
    - crates/z00z_wallets/src/services/wallet_service_types.rs
    - crates/z00z_wallets/src/core/address/mod.rs
    - crates/z00z_wallets/src/core/key/mod.rs
    - crates/z00z_wallets/src/core/key/KEYS-DERIVATION.md
    - crates/z00z_wallets/src/core/tx/mod.rs
    - crates/z00z_wallets/README.md
    - crates/z00z_wallets/docs/rpc-user-guide.md
    - crates/z00z_wallets/examples/ALL-wallet-flows.md
    - .planning/phases/030-refactor-long-files/030-todo.md
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "Hide services::wallet_service as crate-private after the caller audit showed no remaining deep-path consumers."
  - "Expose AddressUsedOracle, Sleeper, and RateLimitPrecheck from the services facade and document them as the stable orchestration helpers."
  - "Frame ALL-wallet-flows.md and KEYS-DERIVATION.md as internal implementation traces so caller guidance remains anchored on shallow facades."
patterns-established:
  - "Wallet normalization closeout: first stabilize split facades, then run one inventory-backed shallow-path wave across code, docs, rustdoc, and planning."
  - "If a split helper becomes public during normalization, add rustdoc immediately so the workspace-wide warning-as-error gate stays green."
requirements-completed: [PH30-NORMALIZE, PH30-VERIFY, PH30-SYNC]
completed: 2026-04-01
---

# Phase 030 Plan 10 Summary

📌 Wallet caller-visible imports now converge on the shallow `db`, `services`, `core::address`, `core::key`, and `core::tx` facades, with docs, rustdoc, and planning evidence aligned to the same surface.

## Performance

- 📌 Started: 2026-04-01T01:05:00Z
- 📌 Completed: 2026-04-01T03:15:51Z
- 📌 Tasks: 2
- 📌 Files modified: 15

## Accomplishments

- 📌 Normalized wallet-owned caller paths away from `redb_wallet_store`, `wallet_service`, `address_manager`, `bip32`, and `key_manager` deep imports onto the shallow facades.
- 📌 Closed the remaining caller-visible service leak by making `services::wallet_service` crate-private while preserving the intended public re-exports on `services`.
- 📌 Synchronized wallet docs with the final public surface, including README facade guidance, RPC wiring guidance, and explicit internal-walkthrough labeling in the long-form example docs.
- 📌 Promoted the service helper types `AddressUsedOracle`, `Sleeper`, and `RateLimitPrecheck` onto the stable facade and added rustdoc so `missing_docs` stays green under the workspace gate.
- 📌 Closed verification on the final source state with clean wallet-targeted tests, rustdoc, repeated review passes, and a successful `full_verify --max-safe-run` summary of `313 planned, 21 skipped, 0 failed`.

## Task Commits

📌 No git commit was created in this closeout. The repository already contains unrelated local changes, and the repo rules require the owned Z00Z git-versioning workflow instead of ad hoc commit commands.

## Files Created/Modified

- `crates/z00z_wallets/src/services/mod.rs` now hides `wallet_service` as an internal split module and keeps the caller-visible surface on facade re-exports.
- `crates/z00z_wallets/src/services/wallet_service_types.rs` now documents the newly public service helper types required by the facade.
- `crates/z00z_wallets/src/core/key/mod.rs` now re-exports the shallow key items required by rustdoc-visible examples, including `VIEW_KEY_ACCOUNT_OFFSET`.
- `crates/z00z_wallets/README.md`, `crates/z00z_wallets/docs/rpc-user-guide.md`, `crates/z00z_wallets/examples/ALL-wallet-flows.md`, and `crates/z00z_wallets/src/core/key/KEYS-DERIVATION.md` now agree on shallow caller entrypoints and clearly mark internal file traces as implementation detail.
- `.planning/phases/030-refactor-long-files/030-todo.md`, `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/REQUIREMENTS.md` now carry the Plan 10 closeout evidence and requirement sync.

## Decisions Made

- 📌 Treat the dedicated caller-normalization wave as complete only after the public `services::wallet_service` deep path is no longer callable from outside the crate.
- 📌 Keep compatibility re-exports for wallet encryption and snapshot-state types on the `services` facade, but document them as a narrow compatibility lane rather than the preferred new-integration surface.
- 📌 Preserve detailed flow documents, but mark them as internal implementation walkthroughs so they do not compete with the public facade contract.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed rustdoc shallow-key example drift**

- 📌 Found during: Task 2 verification (`cargo test -p z00z_wallets --doc --release`)
- 📌 Issue: `bip32.rs` rustdoc examples imported `VIEW_KEY_ACCOUNT_OFFSET` from `core::key`, but the shallow key facade did not re-export it.
- 📌 Fix: re-exported `VIEW_KEY_ACCOUNT_OFFSET` from `crates/z00z_wallets/src/core/key/mod.rs`.
- 📌 Files modified: `crates/z00z_wallets/src/core/key/mod.rs`
- 📌 Verification: `cargo test -p z00z_wallets --doc --release`
- 📌 Committed in: not committed in this closeout

**2. [Rule 2 - Missing Critical] Fixed caller-visible services facade mismatch**

- 📌 Found during: review pass over the post-normalization public surface
- 📌 Issue: `AddressUsedOracle`, `Sleeper`, and `RateLimitPrecheck` were still effectively trapped behind a deep split module even though the plan and README treated them as shallow facade items.
- 📌 Fix: made the helper types public, re-exported them from `services`, then hid `services::wallet_service` as crate-private so the facade became the only caller-visible path.
- 📌 Files modified: `crates/z00z_wallets/src/services/mod.rs`, `crates/z00z_wallets/src/services/wallet_service_types.rs`
- 📌 Verification: focused `cargo check -p z00z_wallets --release`, rustdoc, release wallet suite, and final `full_verify --max-safe-run`
- 📌 Committed in: not committed in this closeout

**3. [Rule 2 - Missing Critical] Added rustdoc for newly public service helper types**

- 📌 Found during: final `full_verify --max-safe-run`
- 📌 Issue: once the helper types became public, the workspace `-D warnings` gate failed on missing docs in `wallet_service_types.rs`.
- 📌 Fix: added concise rustdoc for `AddressUsedOracle`, `Sleeper`, `Sleeper::sleep`, and `RateLimitPrecheck` plus its public fields.
- 📌 Files modified: `crates/z00z_wallets/src/services/wallet_service_types.rs`
- 📌 Verification: `cargo check -p z00z_wallets --release`, `cargo test -p z00z_wallets --doc --release`, and final `full_verify --max-safe-run`
- 📌 Committed in: not committed in this closeout

---

📌 Total deviations: 3 auto-fixed issues.
📌 Impact on plan: All fixes were required to make the normalized shallow facade real, documented, and green under the workspace verification gates.

## Issues Encountered

- 📌 The exact broad `rg` command from the plan still reports expected hits in historical phase artifacts, internal module-owner references, benches, and explicit facade re-export sites. The caller-facing closeout evidence therefore relies on the raw output plus a scoped interpretation rather than on a literal zero-match result for every repository artifact.
- 📌 `full_verify --max-safe-run` initially failed on `missing_docs` after the service helper types became public. That failure was closed in-scope by adding rustdoc and rerunning the full gate to success.

## Caller Inventory Audit

📌 Exact Task 1 audit output:

```text
## TASK1_RG
EXIT=1
```

📌 Exact Task 2 audit output:

```text
## TASK2_RG
.planning/phases/030-refactor-long-files/030-todo.md
356:- `crate::db::redb_wallet_store::{...}` callers move to `crate::db::{...}` for `WltSession`, `ScanStatePayload`, lock checks, and session-backed store helpers.
357:- `crate::services::wallet_service::{...}` callers move to `crate::services::{...}` for `AddressUsedOracle`, `RateLimitPrecheck`, and `Sleeper`.
358:- `crate::core::address::address_manager::{...}` callers move to `crate::core::address::{...}` for cache-size constants.
359:- `crate::core::key::seed::{...}`, `crate::core::key::bip32::{...}`, and `crate::core::key::key_manager::{...}` callers move to `crate::core::key::{...}` for BIP-44, seed, and key-manager surfaces.

.planning/phases/030-refactor-long-files/030-01-PLAN.md
26:    - `crate::db::redb_wallet_store::*` remains the stable caller-visible boundary while the internal store layout changes.
89:  <action>Per D-01 through D-03b, D-06, D-14, and D-15, split `redb_wallet_store.rs` along semantic seams only. Keep the root file as the stable orchestration and compatibility surface, and move tables, codecs, migrations, session, backup, and query helpers into proposed sibling modules only where each extracted file owns one homogeneous responsibility. Do not shard a coherent subsystem merely to hit a line target. Preserve the current external `crate::db::redb_wallet_store::*` contract during this wave. The persisted-wallet KDF, AAD, and envelope contract must stay owned by the existing `crates/z00z_wallets/src/db/redb_wallet_crypto.rs`; do not create a second wallet-store crypto surface or duplicate its constants and helpers.</action>
124:rg -n "crate::db::redb_wallet_store::|redb_wallet_store::" crates/z00z_wallets -g '*.rs'

.planning/phases/030-refactor-long-files/030-01-SUMMARY.md
59:- Split the wallet-store monolith into dedicated tables, codecs, migrations, session, objects, queries, backup, and crypto seam modules without breaking the caller-visible `crate::db::redb_wallet_store::*` boundary.

.planning/phases/030-refactor-long-files/030-10-PLAN.md
92:rg -n "crate::db::redb_wallet_store::|crate::services::wallet_service::|crate::core::address::address_manager::|crate::core::key::(seed|bip32|key_manager)::|crate::core::tx::(state_update|claim_tx|tx_verifier|spending)::" crates/z00z_wallets -g '*.rs'
114:rg -n "redb_wallet_store::|wallet_service::|address_manager::|bip32::|key_manager::|tx_verifier::" crates/z00z_wallets crates/z00z_wallets/README.md .planning/phases/030-refactor-long-files -g '*.rs' -g '*.md'

crates/z00z_wallets/examples/CRYPTO_ARCHITECTURE_EXPLAINED.md
330:use bip32::XPrv;  // Не используется в production

.planning/phases/030-refactor-long-files/030-CONTEXT.md
147:- `crate::db::redb_wallet_store::*` symbols are consumed from `session_service`, `wlt_store`, `core/address`, and `core/wallet`, so internal splitting must preserve that boundary until consumer updates are explicitly scheduled.

crates/z00z_wallets/src/services/mod.rs
25:pub use wallet_service::RateLimitPrecheck;
26:pub use wallet_service::WalletService;
28:pub use wallet_service::{AddressUsedOracle, Sleeper};

crates/z00z_wallets/benches/async_batch_threshold_bench.rs
15:        key::{bip32::Bip44Path, KeyManagerImpl},

crates/z00z_wallets/benches/address_derivation.rs
19:            bip32::{Bip39Seed64, Bip44KeyManager, Bip44Path, Z00Z_BIP44_ASSET},

crates/z00z_wallets/benches/derivation_bench.rs
12:use bip32::ChildNumber;

crates/z00z_wallets/src/db/mod.rs
11:pub use redb_wallet_store::debug_export_wallet;
12:pub use redb_wallet_store::{
16:pub(crate) use redb_wallet_store::{
20:pub(crate) use redb_wallet_store::{

crates/z00z_wallets/src/core/key/bip32_path.rs
72:    Bip32(#[from] bip32::Error),

crates/z00z_wallets/src/core/key/bip32_key_deriver.rs
31:            if matches!(e, bip32::Error::SeedLength) {
119:    /// use bip32::ChildNumber;

crates/z00z_wallets/src/core/tx/tx_assembler.rs
10:    tx_verifier::TxVerifier,
224:        tx_verifier::{TxVerifierResult, VerificationResult},

crates/z00z_wallets/src/core/tx/mod.rs
90:pub use tx_verifier::{
95:pub use tx_verifier::{

crates/z00z_wallets/src/core/key/mod.rs
16:pub use key_manager::{
43:pub(crate) use bip32::{reset_seed_zeroized, seed_zeroized};
44:pub use bip32::{

crates/z00z_wallets/src/core/key/bip32.rs
224:use bip32::{ChildNumber, DerivationPath, XPrv};
1416:        use bip32::Prefix;
1445:        use bip32::Prefix;
1470:        use bip32::Prefix;
1495:        use bip32::Prefix;

crates/z00z_wallets/src/core/tx/claim_tx.rs
19:    output_flow::output_range_ctx_hash, tx_verifier::TX_PACKAGE_KIND,

crates/z00z_wallets/src/core/tx/claim_wire_types.rs
5:use super::tx_verifier::{default_chain_name, default_chain_type};

crates/z00z_wallets/src/core/tx/state_update.rs
18:use super::tx_verifier::TxInputWire;

crates/z00z_wallets/src/core/address/mod.rs
39:pub use address_manager::{

crates/z00z_wallets/src/core/address/address_manager/tests.rs
3:        bip32::Z00Z_BIP44_ASSET, KeyManager, KeyManagerImpl, ReceiverKeys, ReceiverSecret,
EXIT=0
```

📌 Interpretation: the zero-match Task 1 audit confirms no remaining wallet-owned caller imports of the legacy deep paths in `.rs` consumers, while the Task 2 raw output now consists of expected historical plan references, internal facade re-export sites, bench-only references, and one out-of-scope explanatory example file outside the caller-facing closeout surface.

## Known Stubs

📌 None detected in the touched Plan 10 wallet normalization surface.

## User Setup Required

📌 None. This plan changed only Rust module surfaces, docs, and planning artifacts.

## Next Phase Readiness

- 📌 Plan 11 can now normalize core caller-visible paths against the same evidence-backed pattern used here for wallet facades.
- 📌 The wallet crate now presents one coherent caller contract, so later docs and consumer migrations do not need to reason about split implementation files.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_wallets --release --test test_key_manager -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_e2e_public_path -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_app_service_create_wallet -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture`
- 📌 `cargo test -p z00z_wallets --doc --release`
- 📌 `cargo check -p z00z_wallets --release`
- 📌 Three review passes over the wallet shallow-surface closeout, with the final review reporting no remaining technical blocker in the public wallet surface and only planning evidence left before summary creation.
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run` completed successfully with `[summary] planned=313 skipped=21 failed=0` and `exit 0`.

## Self-Check

📌 PASSED: `030-10-SUMMARY.md` exists, `030-todo.md` records the audit requirement as complete, `ROADMAP.md` advances Phase 030 to `10/12 plans executed`, `STATE.md` now points to Plan 11 as next, and `REQUIREMENTS.md` marks `PH30-SYNC` complete.
