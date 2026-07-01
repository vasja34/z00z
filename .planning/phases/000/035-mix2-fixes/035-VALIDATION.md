---
phase: 35
slug: mix2-fixes
status: partial
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-13
---

# Phase 035 - Validation Strategy

> Reconstructed Nyquist validation contract for the completed Phase 035 execution set.

---

## Test Infrastructure

| Property | Value |
| -------- | ----- |
| **Framework** | Rust `cargo test` integration and unit suites plus repo bootstrap gate |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` |
| **Full suite command** | `cargo test --release --features test-fast --features wallet_debug_dump` |
| **Estimated runtime** | not profiled |

## Sampling Rate

- After every task commit: run `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- After every plan wave: run `cargo test --release --features test-fast --features wallet_debug_dump`
- Before `/gsd-verify-work`: full suite must be green
- Max feedback latency: bounded by the quick bootstrap gate

## Evidence Snapshot

- 2026-04-13 quick gate: `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` ended with `=== BOOTSTRAP COMPLETE ===`
- 2026-04-13 targeted runtime refresh:
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_misuse_gate`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_record_gate`
  - `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_asset_pack_v2_memo`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage3_nullifier_store`
  - `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline`
- All targeted commands above completed green in this validation pass

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| ------- | ---- | ---- | ----------- | ---------- | --------------- | --------- | ----------------- | ----------- | ------ |
| 035-01 | 01 | deferred | Canonical Deferred-Intake Freeze | — | no historical deferred import is reopened implicitly | doc review | — | ✅ | ⚠️ manual |
| 035-02 | 01 | deferred | Live Phase-Source Binding | — | six-source authority stays fixed to Phase 035 artifacts | doc review | — | ✅ | ⚠️ manual |
| 035-03 | 02 | deferred | Historical Triage Lock-In | — | historical deferred sources remain excluded unless re-authorized | doc review | — | ✅ | ⚠️ manual |
| 035-04 | 02 | deferred | Optional keep_path Sidecar Gate | — | optional storage sidecar cannot become semantic closure | doc review | — | ✅ | ⚠️ manual |
| 035-05 | 02 | deferred | Phase Closeout Honesty Rules | — | closeout wording cannot overclaim inherited scope | doc review | — | ✅ | ⚠️ manual |
| 035-06 | 03 | deferred | Deferred-Consistency Validation Wave | T-035-07 | deferred intake controls remain enforced | integration | `cargo test -p z00z_storage --release --test test_search_api && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-07 | 03 | deferred | Optional Sidecar Validation Gate | T-035-08 | active keep_path refactor remains housekeeping-only | doc + integration | — | ✅ | ⚠️ manual |
| 035-08 | 04 | suffix | Suffix Authority Freeze | — | suffix lane stays on canonical authority only | doc review | — | ✅ | ⚠️ manual |
| 035-09 | 04 | suffix | Declaration-Backed Inventory Lock-In | — | raw inventory cannot drive live rename execution | doc review | — | ✅ | ⚠️ manual |
| 035-10 | 05 | suffix | Production-Head Cleanup Target | — | only curated production-current suffix rows may hand off | doc review | — | ✅ | ⚠️ manual |
| 035-11 | 05 | suffix | Filename And Exclusion Hygiene | — | filename-only and excluded rows stay out of execution scope | doc review | — | ✅ | ⚠️ manual |
| 035-12 | 05 | suffix | Curated Rename And Retirement Handoff | — | suffix lane hands off only curated declaration-backed rows | doc review | — | ✅ | ⚠️ manual |
| 035-13 | 06 | suffix | Suffix Inventory Validation Wave | — | curated suffix families stay separated from corrected rows and exceptions | doc + bootstrap | — | ✅ | ⚠️ manual |
| 035-14 | 06 | suffix | Suffix Cleanup Readiness Gate | — | suffix cleanup remains planning-only and bounded | doc review | — | ✅ | ⚠️ manual |
| 035-15 | 07 | garbage | Garbage Classification Freeze | — | garbage, keep-set, and source-drift lanes remain distinct | doc review | — | ✅ | ⚠️ manual |
| 035-16 | 07 | garbage | Hard-Garbage Removal Cluster | — | only audited source-shell removals land in the first deletion wave | integration | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` | ✅ | ⚠️ manual |
| 035-17 | 08 | garbage | Debug-Dump Retirement Review | — | debug trio remains deferred as one simulator-backed cluster | doc review | — | ✅ | ⚠️ manual |
| 035-18 | 08 | garbage | Compatibility And Migration Keep-Set Freeze | — | compatibility and migration keep-set stays explicit and frozen | doc review | — | ✅ | ⚠️ manual |
| 035-19 | 08 | garbage | Current-Path-Only Source Drift Handoff | — | stronger current-path-only target cannot drive deletions until source updates | doc review | — | ✅ | ⚠️ manual |
| 035-20 | 09 | garbage | Garbage-Filter Validation Wave | — | garbage lane closeout remains narrow and reviewed | integration | `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` | ✅ | ⚠️ manual |
| 035-21 | 09 | garbage | Current-Path Closure Gate | — | debug and compatibility lanes cannot be silently collapsed into deletion scope | doc review | — | ✅ | ⚠️ manual |
| 035-22 | 10 | sender | Sender Seam Freeze | — | sender ownership stays wallet-local under stealth seam | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_misuse_gate` | ✅ | ✅ green |
| 035-23 | 10 | sender | Canonical Helper And Approval Extension | — | validated sender helpers enforce approval boundaries | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_misuse_gate` | ✅ | ✅ green |
| 035-24 | 10 | sender | Validated Card-Only Entrypoint | — | card-only validated lane fails closed on non-pinned cards | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_misuse_gate` | ✅ | ✅ green |
| 035-25 | 11 | sender | Legacy Builder Adapter Convergence | — | legacy builder callers remain adapters over canonical helper/formula ownership | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-26 | 11 | sender | Replayable Bundle Adapter Convergence | — | replayable bundle callers stay on canonical sender seam | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-27 | 11 | sender | Stealth Export And Unit Coverage | — | public sender surfaces preserve raw and validated seam split | unit + integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-28 | 12 | sender | Downstream Adapter Regression Sweep | T-035-28 | Stage 3 callers cannot drift back to legacy sender construction | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_stage3_nullifier_store && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline` | ✅ | ✅ green |
| 035-29 | 12 | sender | Documentation Correction Wave | — | sender docs match live helper ownership and approval split | integration + doc recheck | `cargo test -p z00z_wallets --lib core::stealth::output::tests::test_public_serial_aware_sender_seam_rejects_wrong_serial_validation_ctx -- --exact` | ✅ | ✅ green |
| 035-30 | 13 | sender | Sender Workflow Validation Wave | — | sender behavior is validated on wallet and simulator surfaces | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_misuse_gate && cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_claim_tx_pipeline` | ✅ | ✅ green |
| 035-31 | 13 | sender | Sender Workflow Acceptance Gate | — | sender lane closes only on validated card and request semantics | source acceptance | — | ✅ | ⚠️ manual |
| 035-32 | 14 | stealth | Stealth Scope Freeze | — | stealth additions remain fenced to approved Phase 035 slice | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-33 | 14 | stealth | Receiver-Secret Exposure Inventory | — | receiver secret no longer leaks through public debug accessors | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-34 | 14 | stealth | Receiver-Secret Narrowing Seam | — | simulator compatibility lane reconstructs receiver secret locally | integration | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-35 | 15 | stealth | Stealth Derivation Vector Freeze | — | card-bound and request-bound derivation families stay frozen | integration | `cargo test -p z00z_wallets --test test_stealth_kdf_vectors --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-36 | 15 | stealth | Derivation Drift Regression Sweep | — | derivation families cannot silently collapse or swap argument order | integration | `cargo test -p z00z_wallets --test test_stealth_kdf_vectors --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-37 | 15 | stealth | V2 Memo Contract Definition | — | V2 memo decoding remains bounded and version-aware | unit | `cargo test -p z00z_core test_v2_memo --release --features test-fast && cargo test -p z00z_core test_decode_asset_pack --release --features test-fast` | ✅ | ✅ green |
| 035-38 | 16 | stealth | V2 Memo Receive-Path Enablement | — | live receive path can classify V2 memo packs without leaking memo data | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_asset_pack_v2_memo` | ✅ | ✅ green |
| 035-39 | 16 | stealth | Stealth Additions Validation Wave | T-035-39 | approved stealth additions validate without widening memo or secret scope | integration | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_asset_pack_v2_memo && cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_s5_record_gate` | ✅ | ✅ green |
| 035-40 | 16 | stealth | Stealth Additions Acceptance Gate | T-035-40 | stealth lane closes only on approved additions and out-of-scope fence | source acceptance | — | ✅ | ⚠️ manual |
| 035-41 | 17 | rename | Rename Scope Freeze | — | curated rename authority stays frozen to approved rows only | static review | — | ✅ | ⚠️ manual |
| 035-42 | 17 | rename | Live Rename Manifest And Lane Split | — | file-first and signature-after lanes remain separated | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |
| 035-43 | 17 | rename | File Rename Wave A - Test And Support Files | — | approved Wave A file moves land without widening rename scope | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |
| 035-44 | 18 | rename | File Rename Wave B - Wallet DB And Egui Canonical Files | — | canonical wallet DB and egui filenames stay live | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |
| 035-45 | 18 | rename | Signature Rename Wave A - Module, Path, And Include Mirrors | — | module and include mirrors resolve through canonical filenames | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |
| 035-46 | 18 | rename | Signature Rename Wave B - Types, Functions, And Methods | — | curated declaration renames stay on approved spellings only | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |
| 035-47 | 19 | rename | Cross-File Reference Sweep And No-Change Guard | — | bounded sweep removes curated old-name residue while preserving no-change rows | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |
| 035-48 | 19 | rename | Rename Validation Wave | — | curated rename acceptance is backed by live helper spelling and no-change guards | integration + suite | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards && cargo test --release --features test-fast --features wallet_debug_dump` | ✅ | ✅ green |
| 035-49 | 19 | rename | Rename Acceptance Gate | — | final rename lane closes only on curated authority surface | integration + guard | `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump --test test_phase035_rename_guards` | ✅ | ✅ green |

Status legend: ⬜ pending, ✅ green, ❌ red, ⚠️ manual.

## Wave 0 Requirements

- [x] `crates/z00z_wallets/tests/test_phase035_rename_guards.rs` - curated rename acceptance guards for the final Phase 035 rename slice
- [ ] Remaining boundary and acceptance assertions are manual-only unless the project wants dedicated planning-artifact guard tests

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| -------- | ----------- | ---------- | ----------------- |
| Deferred authority freeze and historical triage honesty | `035-01..035-05`, `035-07` | repository policy and scope-boundary wording are enforced through planning artifacts, not runtime APIs | Read `035-CONTEXT.md`, `035-TODO.md`, `035-01-SUMMARY.md`, `035-03-SUMMARY.md`; confirm no historical deferred import is implied and `keep_path(...)` remains optional housekeeping only |
| Suffix authority, inventory lock, and cleanup readiness | `035-08..035-14` | these tasks validate planning authority, curated handoff rules, and bounded readiness language | Read `035-2-suffixes.md`, `035-a6-renames.md`, `035-04-SUMMARY.md`, `035-05-SUMMARY.md`, `035-06-SUMMARY.md`; confirm corrected rows and filename-only rows remain excluded from execution authority |
| Garbage governance and current-path boundary | `035-15..035-21` | closure depends on scoped interpretation of delete lanes, deferred debug cluster, and compatibility keep-set wording | Read `035-3-garbage-filter.md`, `035-07-SUMMARY.md`, `035-08-SUMMARY.md`, `035-09-SUMMARY.md`; confirm the debug trio remains deferred and the compatibility keep-set is still frozen |
| Sender acceptance boundary | `035-31` | acceptance claims rely on source-level seam and approval-boundary review beyond behavioral tests | Read `035-13-SUMMARY.md` and the cited sender files; confirm the validated card-only and request-bound lanes remain the only accepted approval surfaces |
| Stealth acceptance boundary | `035-40` | acceptance depends on preserving the explicit out-of-scope list, not just passing receive-path tests | Read `035-16-SUMMARY.md`, `035-a5-fix-spec.md`, and `test_asset_pack_v2_memo.rs`; confirm no routing, proof, or broader memo semantics were smuggled into closure |
| Rename scope freeze | `035-41` | the execution-authority fence still lives in the curated planning table rather than a stable runtime contract | Read `035-a6-renames.md`, `035-17-SUMMARY.md`, and `035-19-SUMMARY.md`; confirm only curated rows were executed and the raw matrix stayed inventory-only |

## Validation Sign-Off

- [x] Existing test infrastructure detected and reused
- [x] Acceptance-critical sender and stealth runtime tests rerun green
- [x] Curated rename acceptance guards added and green
- [ ] All tasks have fully automated verification
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] No watch-mode flags
- [ ] `nyquist_compliant: true` set in frontmatter

Approval: partial, manual review outstanding
