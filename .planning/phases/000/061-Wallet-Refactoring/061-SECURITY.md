---
phase: 061
slug: wallet-refactoring
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-24
register_authored_at_plan_time: true
---

# Phase 061 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## 🔒 Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Phase authority -> live wallet tree | `061-TODO.md`, `061-CONTEXT.md`, and the numbered plans define one canonical rename/refactor packet that must stay aligned with the current `crates/z00z_wallets/src` tree. | rename rows, live file paths, plan/slice completion truth |
| Stable public facades -> flattened implementations | `adapters::rpc`, `db::redb_wallet_store`, and `services::WalletService` must keep one caller-visible authority while implementation files move. | module paths, re-exports, service constructors, RPC entry points |
| Non-`src` assets -> include/load anchors | Docs, schema, config, and snapshot assets moved out of `src/` must keep one canonical home and one set of loader/test anchors. | `include!`, `include_str!`, config paths, doc fixtures, snapshot files |
| Persistence strings -> structural refactor | File moves must not mutate `.wlt` labels, schema ids, KDF labels, or domain-separated strings. | persisted labels, domain strings, schema names, wallet metadata |
| Security assets -> runtime consumers | Password denylist/Bloom and wallet docs/domain snapshots must stay synchronized between generators, loaders, and tests. | password corpora, Bloom assets, wallet guide, domain snapshot |
| Cross-crate consumers -> wallet closeout paths | Downstream tests in `z00z_storage` and final path guards in `z00z_wallets` must consume the post-refactor wallet file layout without parallel aliases. | tx proof file path, canonical test paths, final tree audit signals |
| Final tree audit -> status consumers | The closeout audit must prove the one-level wallet tree directly from the live crate, not only from status prose. | nested path checks, removed-path checks, targeted release tests |

---

## 🚨 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Evidence | Status |
|-----------|----------|-----------|-------------|------------|----------|--------|
| T-061-01 | Repudiation | Preflight drift audit | mitigate | Reconcile stale or nonexistent planner rows against the live tree before executing renames. | `061-01-PLAN.md`; `061-01-SUMMARY.md`; `061-TODO.md` | closed |
| T-061-02 | Tampering | Delete-candidate proof | mitigate | Keep duplicate/stale removals behind live-reference proof instead of filename intuition. | `061-01-SUMMARY.md`; `crates/z00z_wallets/tests/test_rename_guards.rs:480-488` | closed |
| T-061-03 | Tampering | Anchor inventory | mitigate | Freeze `include!`/`include_str!`/path-sensitive surfaces before broad moves. | `061-CONTEXT.md`; `crates/z00z_wallets/tests/test_rename_guards.rs:465-512` | closed |
| T-061-04 | Tampering | Shared DB rename | mitigate | Keep structural rename work from mutating persisted label space, schema names, or domain strings. | `061-02-SUMMARY.md`; `crates/z00z_wallets/src/domains/definitions.rs`; `crates/z00z_wallets/src/domains/hashing.rs` | closed |
| T-061-05 | Repudiation | Shared DB seam isolation | mitigate | Isolate shared-crypto rename from backend relocation so one persistence seam changes at a time. | `061-02-PLAN.md`; `061-02-SUMMARY.md`; `crates/z00z_wallets/src/db/mod.rs` | closed |
| T-061-06 | Tampering | Persistence vocabulary | mitigate | Remove stale import/vocabulary survivors so shared DB naming stays singular. | `061-02-SUMMARY.md`; `crates/z00z_wallets/tests/test_rename_guards.rs` | closed |
| T-061-07 | Denial of Service | RedB facade stability | mitigate | Preserve the RedB facade while relocating backend implementation files. | `061-03-SUMMARY.md`; `crates/z00z_wallets/src/db/mod.rs`; `crates/z00z_wallets/src/redb_store/test_redb_store.rs:44-47` | closed |
| T-061-08 | Repudiation | RedB tree constraint | mitigate | Prove the backend subtree flattened to one-level files and left no hidden nested Rust residue. | `061-03-SUMMARY.md`; `find crates/z00z_wallets/src -mindepth 3 -type f -name '*.rs'` returned no output during 2026-06-24 audit | closed |
| T-061-09 | Repudiation | RedB anchor integrity | mitigate | Keep schema/wallet-guide anchors on canonical non-`src` homes after the move. | `crates/z00z_wallets/src/redb_store/test_redb_store.rs:44-47`; `crates/z00z_wallets/docs/WALLET-GUIDE.md` | closed |
| T-061-10 | Tampering | Wallet-config authority | mitigate | Move wallet config once and update every embedded loader so stale defaults cannot survive. | `061-04-SUMMARY.md`; `crates/z00z_wallets/src/services/wallet_paths.rs`; `crates/z00z_wallets/config/wallet_config.yaml` | closed |
| T-061-11 | Elevation of Privilege | RPC surface scope | mitigate | Preserve the `adapters::rpc` compatibility facade instead of introducing a second crate-root API plane. | `061-04-SUMMARY.md`; `crates/z00z_wallets/src/adapters/mod.rs` | closed |
| T-061-12 | Repudiation | RPC wave isolation | mitigate | Keep support/config churn separate from method churn so regressions stay attributable. | `061-04-PLAN.md`; `061-06-PLAN.md`; `061-06-SUMMARY.md` | closed |
| T-061-13 | Denial of Service | Service wrapper integrity | mitigate | Rewire service wrappers atomically so construct/unlock/restore/runtime paths do not silently break. | `061-05-SUMMARY.md`; `crates/z00z_wallets/src/services/mod.rs`; `crates/z00z_wallets/src/services/wallet_service.rs` | closed |
| T-061-14 | Repudiation | Phantom wrapper prevention | mitigate | Close stale service rows instead of recreating nonexistent wrapper layers. | `061-01-SUMMARY.md`; `061-05-SUMMARY.md`; `061-TODO.md` row 306-313 | closed |
| T-061-15 | Repudiation | Service path tests | mitigate | Keep source-anchor/path-sensitive service tests aligned to live file homes. | `crates/z00z_wallets/src/services/wallet_paths.rs:674-675`; `crates/z00z_wallets/tests/test_rename_guards.rs:470-512` | closed |
| T-061-16 | Denial of Service | RPC method behavior | mitigate | Flatten RPC method files without widening or silently breaking wallet RPC behavior. | `061-06-SUMMARY.md`; `crates/z00z_wallets/src/rpc/`; `crates/z00z_wallets/tests/test_rpc_truth.rs` | closed |
| T-061-17 | Tampering | Method helper vocabulary | mitigate | Finish helper renames without leaving split method naming vocabularies behind. | `061-06-SUMMARY.md`; `crates/z00z_wallets/src/rpc/`; `crates/z00z_wallets/tests/test_rename_guards.rs` | closed |
| T-061-18 | Repudiation | RPC support/method boundary | mitigate | Keep support/logging/types work out of the method-flattening slice so audit evidence stays attributable. | `061-04-SUMMARY.md`; `061-06-SUMMARY.md` | closed |
| T-061-19 | Tampering | Duplicate storage truth | mitigate | Remove or retain duplicate storage files only with explicit live-reference proof. | `061-07-SUMMARY.md`; `crates/z00z_wallets/tests/test_rename_guards.rs` | closed |
| T-061-20 | Denial of Service | Security asset paths | mitigate | Move password security assets with generator/loader parity so runtime validation does not drift. | `061-07-SUMMARY.md`; `crates/z00z_wallets/src/security/password.rs`; `crates/z00z_wallets/bin/gen_password_bloom.rs` | closed |
| T-061-21 | Tampering | Receiver facade churn | mitigate | Keep receiver path changes local and atomically wired so service/RPC flows do not inherit stale paths. | `061-07-SUMMARY.md`; `crates/z00z_wallets/src/receiver/`; `crates/z00z_wallets/src/services/` | closed |
| T-061-22 | Tampering | Key include graph | mitigate | Flatten include-heavy key code without leaving latent `.inc.rs` or path bugs behind. | `061-08-SUMMARY.md`; `crates/z00z_wallets/tests/test_rename_guards.rs`; `cargo test --release -p z00z_wallets --test test_rename_guards` on 2026-06-24 | closed |
| T-061-23 | Repudiation | Key doc traceability | mitigate | Move key docs to one flat `docs/*` home and update anchors so code-to-doc traceability survives. | `061-08-SUMMARY.md`; `crates/z00z_wallets/docs/KEYS-DERIVATION.md`; `crates/z00z_wallets/docs/KEYS-Bip44-UserGuide.md` | closed |
| T-061-24 | Repudiation | Key isolation | mitigate | Keep key churn isolated from tx/wallet churn so failures remain attributable. | `061-08-PLAN.md`; `061-09-PLAN.md`; `061-08-SUMMARY.md` | closed |
| T-061-25 | Tampering | Claim/tx helper semantics | mitigate | Split claim/tx helper files without changing claim behavior or leaving ambiguous helper boundaries. | `061-09-SUMMARY.md`; `crates/z00z_wallets/src/tx/claim_tx.rs`; `crates/z00z_wallets/src/tx/claim_tx_verifier_impl.rs`; `crates/z00z_wallets/src/tx/claim_tx_verifier_impl_proof.rs` | closed |
| T-061-26 | Repudiation | Wallet/backup doc authority | mitigate | Move wallet docs out of `src/` and keep a single canonical wallet-guide home. | `crates/z00z_wallets/src/redb_store/test_redb_store.rs:44-47`; `crates/z00z_wallets/docs/WALLET-GUIDE.md`; `crates/z00z_wallets/tests/test_rename_guards.rs:313-316` | closed |
| T-061-27 | Tampering | Persistence string safety | mitigate | Keep string-labeled wallet data semantics unchanged while doing structural cleanup. | `061-09-SUMMARY.md`; `crates/z00z_wallets/src/domains/definitions.rs`; `crates/z00z_wallets/src/domains/hashing.rs` | closed |
| T-061-28 | Repudiation | Final tree completeness | mitigate | Prove the wallet tree has no hidden multi-level Rust files and no orphaned non-Rust artifacts under `src/`. | `find crates/z00z_wallets/src -mindepth 3 -type f -name '*.rs'` returned no output during 2026-06-24 audit; `find crates/z00z_wallets/src -type f ! -name '*.rs'` returned no output during 2026-06-24 audit | closed |
| T-061-29 | Tampering | Split-name residue | mitigate | Close the final typo/placeholder cleanup with grep-backed proof and removed-path guards. | `crates/z00z_wallets/src/egui_views/tab_registry.rs:80-84`; `crates/z00z_wallets/tests/test_rename_guards.rs:470-512`; `cargo test --release -p z00z_wallets --test test_rename_guards` on 2026-06-24 | closed |
| T-061-30 | Repudiation | Final cross-crate audit | mitigate | Re-audit final wallet paths from downstream consumers so earlier slice success cannot mask residual debt. | `crates/z00z_storage/tests/test_live_guardrails.rs:47-50`; `cargo test --release -p z00z_storage --test test_live_guardrails` on 2026-06-24 | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## ✅ Accepted Risks Log

No accepted risks.

---

## 🧾 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-06-24 | 30 | 30 | 0 | Codex `/gsd-secure-phase 061` |

## 🧪 Verification Evidence

- All ten numbered Phase 061 plans contain a parseable `<threat_model>` block,
  so `register_authored_at_plan_time: true` is correct for this phase.
- No `## Threat Flags` sections were present in the available
  `061-01-SUMMARY.md` through `061-09-SUMMARY.md` artifacts.
- `061-SECURITY.md` was created in state B because no prior
  `061-SECURITY.md` existed and the phase folder contained executed plan/slice
  artifacts.
- Direct 2026-06-24 live-tree verification used:
  - `find crates/z00z_wallets/src -mindepth 3 -type f -name '*.rs'`
  - `find crates/z00z_wallets/src -type f ! -name '*.rs'`
  - `cargo test --release -p z00z_wallets --test test_rename_guards`
  - `cargo test --release -p z00z_storage --test test_live_guardrails`
- The current workspace now contains `061-10-SUMMARY.md`, and that summary is
  consistent with the live-tree closeout evidence cited by `STATE.md` and
  `ROADMAP.md`. This audit still treats live file paths, guard tests, and
  targeted release tests as the primary security evidence for final-slice
  threat closure.

## ⚠️ Unregistered Flags

No unregistered `## Threat Flags` were present in the available Phase 061
summary artifacts.

---

## ✅ Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-06-24
