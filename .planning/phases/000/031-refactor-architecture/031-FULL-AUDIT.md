# Phase 031 Full Audit

## Scope

- Audit timestamp: `2026-04-05T01:28:44Z`
- Execution mode: manual fallback for all four mandatory audit passes in this session. The named audit skills were available as instruction sources, but not as executable audit tools, so the rerun used evidence-backed crate inspection, targeted searches, plan-driven invariant checks, and release-style validation commands.
- Phase source of truth:
  - [031-CONTEXT.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-CONTEXT.md)
  - [031-INVENTORY.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-INVENTORY.md)
  - [031-IMPORT-GRAPH.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md)
  - [031-VALIDATION.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-VALIDATION.md)
  - [031-VERIFICATION.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-VERIFICATION.md)
  - [031-05-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-05-PLAN.md)
- Final target crate list derived only from the Phase 031 artifacts:
  - `z00z_core`
  - `z00z_crypto`
  - `z00z_wallets`
  - `z00z_storage`
  - `z00z_simulator`
  - `z00z_utils`
  - `z00z_networks_rpc`
  - `z00z_networks/onionnet`

## Initial Audit Results

| Crate | `crypto-architect` | `security-audit` | `spec-to-code-compliance` | `z00z-design-foundation-compliance` | Result |
| --- | --- | --- | --- | --- | --- |
| `z00z_core` | manual fallback | manual fallback | manual fallback | manual fallback | No new actionable Phase 031 rerun findings in the phase-touched asset/domain files. |
| `z00z_crypto` | manual fallback | manual fallback | manual fallback | manual fallback | Actionable documentation drift in user-facing facade guidance. |
| `z00z_wallets` | manual fallback | manual fallback | manual fallback | manual fallback | Actionable canonical entrypoint seam drift in `wallet_service.rs`. |
| `z00z_storage` | manual fallback | manual fallback | manual fallback | manual fallback | No new actionable Phase 031 rerun findings in the phase-touched proof/checkpoint/store files. |
| `z00z_simulator` | manual fallback | manual fallback | manual fallback | manual fallback | No new actionable Phase 031 rerun findings in the phase-touched scenario/template runners. |
| `z00z_utils` | manual fallback | manual fallback | manual fallback | manual fallback | No new actionable Phase 031 rerun findings from the phase scope. |
| `z00z_networks_rpc` | manual fallback | manual fallback | manual fallback | manual fallback | No new actionable Phase 031 rerun findings from the phase scope. |
| `z00z_networks/onionnet` | manual fallback | manual fallback | manual fallback | manual fallback | No new actionable Phase 031 rerun findings from the phase scope. |

### Actionable Finding 1: Canonical Wallet Service Entrypoint Still Used Top-Level `include!` Assembly

- Crate: `z00z_wallets`
- Phase link: `G-03` and `D-28` from [031-CONTEXT.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-CONTEXT.md)
- Evidence before fix:
  - [wallet_service.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service.rs) still assembled the canonical service entrypoint with top-level `include!("wallet_service_types_core.rs")`, `include!("wallet_service_types_reachability.rs")`, and `include!("wallet_service_types_state.rs")`.
  - [test_phase30_split.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_phase30_split.rs) was still anchored to the legacy flat assembly shape.
- Why it mattered:
  - The public-looking service root still masked placeholder-heavy and reachability-only seams behind a stable-looking facade.
  - The phase plan explicitly required the canonical entrypoint to become a truthful explicit seam map.

### Actionable Finding 2: `z00z_crypto` User-Facing Docs Still Taught Legacy Facade Imports

- Crate: `z00z_crypto`
- Evidence before fix:
  - [Tari-Crypto-Components-Cookbook.md](/home/vadim/Projects/z00z/.github/requirements/Tari-Crypto-Components-Cookbook.md) still showed user-facing examples that relied on old root or direct Tari import guidance rather than the Phase 031 root-facade split.
  - [Tari-Crypto-Integration-Z00Z.md](/home/vadim/Projects/z00z/.github/requirements/Tari-Crypto-Integration-Z00Z.md) still contained a confidential-transaction example importing `PedersenCommitmentFactory`, `HomomorphicCommitmentFactory`, `BulletproofsPlusService`, and `RistrettoSecretKey` from the root facade.
- Why it mattered:
  - The codebase had already moved phase-touched callers to the split facade lanes.
  - The docs would have trained future changes back toward the deprecated import style.

### Informational Findings That Were Not Actionable For This Phase Rerun

- `crates/z00z_wallets/docs/tari/*.md` still contains direct `tari_crypto::*` and `tari_utilities::*` examples, but those files are explicit Tari reference docs, not Phase 031 application-facing facade guidance.
- Internal `include!` seams still exist under `z00z_wallets/src/services/`, but this rerun only treated the canonical entrypoint rule from [031-05-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-05-PLAN.md) as actionable. Sub-split helper includes were not elevated by the phase scope to a new architectural finding.

## Fixes Applied

### Wallet Service Seam Fixes

- Reworked [wallet_service.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service.rs) so the canonical entrypoint now declares explicit sibling modules with `#[path = ...] mod ...;` and re-exports only the intended shallow facade types.
- Updated [test_phase30_split.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/tests/test_phase30_split.rs) to enforce the new seam map and to forbid the legacy top-level `include!` assembly for the `wallet_service_types_*` files.
- Fixed the visibility fallout introduced by the explicit module split in:
  - [wallet_service_types_core.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service_types_core.rs)
  - [wallet_service_types_state.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service_types_state.rs)
  - [wallet_service_session_derivation_recovery.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service_session_derivation_recovery.rs)
  - [wallet_service_session_construction.rs](/home/vadim/Projects/z00z/crates/z00z_wallets/src/services/wallet_service_session_construction.rs)
- The remediation stayed narrow: sibling access was restored with `pub(super)` and explicit imports rather than expanding public API surface.

### `z00z_crypto` Facade-Documentation Fixes

- Updated [Tari-Crypto-Components-Cookbook.md](/home/vadim/Projects/z00z/.github/requirements/Tari-Crypto-Components-Cookbook.md) to state the facade rule explicitly and to keep user-facing examples on the root and `expert` facade paths.
- Updated [Tari-Crypto-Integration-Z00Z.md](/home/vadim/Projects/z00z/.github/requirements/Tari-Crypto-Integration-Z00Z.md) so the confidential-transaction example uses:
  - `z00z_crypto::expert::keys::RistrettoSecretKey`
  - `z00z_crypto::{BulletproofsPlusService, HomomorphicCommitmentFactory, PedersenCommitmentFactory}`
  - `z00z_crypto::Hidden`

## Re-Audit Results

### Targeted Evidence Checks

| Check | Command / Method | Result |
| --- | --- | --- |
| Canonical wallet service entrypoint no longer uses top-level `include!` assembly | `rg -n '^include!\(' crates/z00z_wallets/src/services/wallet_service.rs` | Clean: no matches. |
| Wallet seam regression guard | `cargo test -p z00z_wallets --release --test test_phase30_split -- --nocapture` | Passed. |
| Wallet error-surface guard | `cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture` | Passed. |
| Required release-style wallet validation from [031-05-PLAN.md](/home/vadim/Projects/z00z/.planning/phases/031-refactor-architecture/031-05-PLAN.md) | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture` | Passed end to end, including the updated seam guards and wallet service error tests. |
| Remaining legacy root-import guidance in `z00z_crypto` Markdown | regex search over `crates/z00z_crypto/**/*.md` for Phase 031 forbidden root imports | Clean after the cookbook and integration-guide fixes. |
| Direct Tari references in Phase 031 target crates | regex search over target crates for `tari_crypto::` and `tari_utilities::` | Only informational matches remained under `crates/z00z_wallets/docs/tari/*.md` reference docs. |

### Re-Audit Conclusion By Crate

| Crate | Re-audit result |
| --- | --- |
| `z00z_core` | Clean for the rerun invariants checked here. |
| `z00z_crypto` | Clean after the two documentation fixes. |
| `z00z_wallets` | Clean for the canonical entrypoint seam requirement after the module split and regression-test update. |
| `z00z_storage` | Clean for the rerun invariants checked here. |
| `z00z_simulator` | Clean for the rerun invariants checked here. |
| `z00z_utils` | Clean for the rerun invariants checked here. |
| `z00z_networks_rpc` | Clean for the rerun invariants checked here. |
| `z00z_networks/onionnet` | Clean for the rerun invariants checked here. |

## Doublecheck Results

- Doublecheck mode: manual fallback.
- Method:
  - re-read the phase scope artifacts,
  - re-checked the changed wallet-service seam files and the two `z00z_crypto` docs,
  - re-ran the mandatory wallet release-style validation commands,
  - re-ran the targeted searches for top-level `include!`, legacy root-facade drift, and direct Tari leakage.
- Remaining actionable issues: none found.
- Remaining informational issues:
  - [Tari-Crypto-Integration-Z00Z.md](/home/vadim/Projects/z00z/.github/requirements/Tari-Crypto-Integration-Z00Z.md) still has pre-existing Markdown lint noise unrelated to the Phase 031 facade fix.
  - [Tari-Crypto-Components-Cookbook.md](/home/vadim/Projects/z00z/.github/requirements/Tari-Crypto-Components-Cookbook.md) still has broader Markdown spacing lint noise, but the Phase 031 architectural doc-drift examples are now aligned.

## Final Status

- Phase 031 rerun target list was derived from the phase directory only.
- All four mandatory audit passes were executed as manual fallbacks and recorded here.
- All actionable rerun findings discovered in this pass were fixed directly in code or documentation.
- Re-audit evidence is green for the wallet-service seam requirement and the `z00z_crypto` facade-guidance drift.
- No remaining actionable findings were left open by this rerun.

## Audit Rerun 2

- Audit timestamp: `2026-04-05T01:36:03Z`
- Trigger: explicit `/gsd-audit-4 phase_dir = 031-refactor-architecture` rerun after the previous audit closure and version-manager sync.
- Audit method: manual fallback again, using the Phase 031 planning corpus as the only scope source and re-running the Phase 031 wallet-service validation commands on the current `HEAD`.
- Final target crate list re-confirmed from the phase artifacts only:
  - `z00z_core`
  - `z00z_crypto`
  - `z00z_wallets`
  - `z00z_storage`
  - `z00z_simulator`
  - `z00z_utils`
  - `z00z_networks_rpc`
  - `z00z_networks/onionnet`

### Current-Head Evidence

| Check | Command / Method | Result |
| --- | --- | --- |
| Canonical wallet service split guard | `cargo test -p z00z_wallets --release --test test_phase30_split -- --nocapture` | Passed on current `HEAD`. |
| Wallet service error guard | `cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture` | Passed on current `HEAD`. |
| Required Phase 031 release-style wallet validation | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture` | Passed on current `HEAD`, including wallet tests and doctests. |
| Canonical wallet service root no longer uses top-level `include!` assembly | `rg -n '^include!\(' crates/z00z_wallets/src/services/wallet_service.rs` | Clean: no matches. |
| Legacy `z00z_crypto` root-import guidance in phase-relevant docs | regex search for `z00z_crypto::{PedersenCommitmentFactory,BulletproofsPlusService,RistrettoSecretKey,CommitmentSignature,DiffieHellmanSharedSecret}` under `crates/z00z_crypto/**/*.md` | Clean: no matches. |
| `z00z_core` wildcard asset root export | `rg -n 'pub use assets::\\*' crates/z00z_core/src/lib.rs` | Clean: no matches. |

### Rerun Findings

- No new actionable findings were produced by this explicit rerun.
- The previously fixed `z00z_wallets` canonical service-root seam remains in the explicit-module shape required by Phase 031.
- The previously fixed `z00z_crypto` facade documentation remains aligned to the root-facade and `expert` split.

### Informational Notes

- `crates/z00z_wallets/src/services/wallet_service_types.rs` still contains internal `include!` assembly for helper sub-splits, but this remains consistent with the earlier audit disposition: the Phase 031 actionable requirement targeted the canonical `wallet_service.rs` entrypoint, not every internal helper seam.
- Broad compatibility and version-suffix grep passes still surface explicit live contracts and bounded compatibility surfaces across storage, utils, and simulator code. Nothing in this rerun changed their prior Phase 031 disposition into a new actionable finding.

### Rerun 2 Conclusion

- The explicit `/gsd-audit-4` rerun confirms the Phase 031 audit remains closed on the current `HEAD`.
- No new code or documentation fixes were required in this rerun.
- `031-FULL-AUDIT.md` now contains append-only evidence for both the original rerun and this follow-up rerun.
