---
phase: 052
slug: 052-hjmt-backend
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-29
updated: 2026-05-29
---

# Phase 052 - Security

Per-phase security contract: threat register, accepted risks, and audit trail.

## 🧭 Audit Basis

- Threat register derived from all 34 bullets inside the `<threat_model>`
  blocks in `052-01-PLAN.md` through `052-11-PLAN.md`.
- `052-*-SUMMARY.md` files did not contain a separate `Threat Flags`
  section, so this audit used workspace code, tests, summaries, and closeout
  artifacts directly.
- Plans `052-08` through `052-11` are future-only candidate plans; their
  threats are closed here only when the candidate remains non-live and guarded
  from current runtime authority.

## 🛡️ Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Storage facade to backend mode router | `AssetStore` selects compatibility, forest, or dual-verify behind one semantic facade. | Asset paths, semantic roots, proof requests, checkpoint-attested ops |
| Durable forest internals | `ForestStore`, commit journal, path index, and RedB persistence own physical layout and crash recovery. | Physical tree roots, journal rows, replay digests, path rows |
| Proof and checkpoint verifier seam | `ProofBlob`, `chk_blob`, `CheckRoot`, checkpoint draft/artifact contracts, and reload validation bind verifier-visible evidence to semantic roots. | Root-bound proof bytes, bucket metadata, checkpoint rows, prior/next roots |
| Downstream consumers | Wallet, validator, runtime, and simulator consume storage-owned semantic evidence only. | Proof blobs, semantic roots, checkpoint ids, audit records |
| Deferred protocol candidates | Adaptive buckets, occupancy counters, generalized settlement roots, `RightLeaf`, and `FeeEnvelope` stay future-only until a later phase promotes them. | No live runtime authority crossing in Phase 052 |

## 🔐 Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-052-01 | Tampering | Backend mode router | mitigate | Compatibility stays default through `AssetBackendMode::default()` / `from_env_or_default()`, unknown modes reject in `AssetStore::try_new_with_backend_mode`, and tests `test_backend_mode_defaults_to_compatibility` plus `test_backend_mode_parsing_and_forest_boundaries` cover mode confusion. | closed |
| T-052-02 | Tampering | Bucket policy metadata | mitigate | `BucketPolicy` validation, deterministic hashing, and decode rejection are covered by `test_fixed_bucket_policy_is_deterministic_and_versioned` and `test_bucket_policy_decode_rejects_invalid_or_mismatched_policy`. | closed |
| T-052-03 | Information Disclosure | Public asset facade | mitigate | Public facade shape is checked by `test_facade_hides_tree_shape`, and downstream source-shape guardrails block `BucketId`, `BucketPolicy`, `BucketRootLeaf`, and other layout authority in `test_downstream_sources_do_not_import_physical_forest_layout`. | closed |
| T-052-04 | Integrity | Forest runtime completeness | mitigate | Forest path is no longer a placeholder lane for in-scope operations: `AssetStore` routes to `forest_*` and `dual_*` methods in `store.rs`, and live forest behavior is exercised by `test_phase052_forest_backend`, `test_phase052_forest_proofs`, `test_phase052_recovery`, and `052-07-SUMMARY.md`. | closed |
| T-052-05 | Spoofing | Forest tree ownership | mitigate | Physical forest state uses crate-private `ForestTreeId` and `ForestStore` ownership; `test_forest_physical_layout_stays_private` proves no public `ForestTreeId` or planner type leaks through `assets::mod`. | closed |
| T-052-06 | Integrity | Forest batch planner | mitigate | Planner rejects duplicate and missing workloads before mutation; `test_forest_rejects_duplicate_and_missing_delete_without_state_drift` proves root and item state preservation after reject. | closed |
| T-052-07 | Information Disclosure | Storage layout boundary | mitigate | Physical layout remains storage-private through `pub(crate)` tree identities and the downstream/layout guardrails in `test_forest_physical_layout_stays_private` and `test_downstream_sources_do_not_import_physical_forest_layout`. | closed |
| T-052-08 | Integrity | Forest commit journal | mitigate | `ForestCommitJournalEntry` and `ForestCommitStatus::{Prepared,ChildrenCommitted,ParentsCommitted,RootPublished}` enforce child-before-parent durability; crash and replay cases are covered by `test_phase052_recovery.rs`. | closed |
| T-052-09 | Tampering | Reload and path-index rebuild | mitigate | `forest_validate_reload`, `forest_rehydrate`, and path-index rebuild stay root-bound; reload coverage includes `test_forest_search_reload_index`, `test_phase052_recovery`, and `test_redb_rehydrate`. | closed |
| T-052-10 | Tampering | Checkpoint reload metadata | mitigate | Rehydrate path rejects state-root and flat-root drift before acceptance, and checkpoint root binding rejects wrong semantic or backend-root contexts in `test_checkpoint_root_binding`. | closed |
| T-052-11 | Spoofing | Forest proof envelope | mitigate | Forest proofs are versioned through `FOREST_PROOF_ENVELOPE_VERSION` and validated by storage-owned `chk_blob`; reject matrix coverage lives in `test_phase052_forest_proofs`. | closed |
| T-052-12 | Spoofing | Absence proof family | mitigate | `check_forest_proof_family()` allows only inclusion; deletion and non-existence stay explicit unsupported fail-closed paths, covered by `test_phase052_forest_proofs` and `test_phase051_golden_corpus`. | closed |
| T-052-13 | Integrity | Benchmark and proof-size evidence | mitigate | Proof-size numbers are recorded only as implementation evidence in `crates/z00z_storage/benches/assets/assets_benches.md` and `052-06-SUMMARY.md`; they are not treated as protocol constants. | closed |
| T-052-14 | Integrity | Dual-verify oracle | mitigate | `dual_verify.rs` makes compatibility-vs-forest drift fatal, and corpus coverage in `test_phase051_golden_corpus` plus whitebox dual-reload tests prevents local-only false green. | closed |
| T-052-15 | Information Disclosure | Downstream consumers | mitigate | Validators, wallets, and simulator stages are source-guarded against physical layout imports by `test_phase052_guardrails.rs`; simulator semantic usage is asserted by `test_scenario_1_stages_remain_semantic_storage_clients`. | closed |
| T-052-16 | Elevation of Privilege | Checkpoint authority seam | mitigate | Backend-root bytes stay diagnostic only; `test_checkpoint_root_binding` and `test_wallet_backend_root_stays_diagnostic` prove semantic roots remain verifier authority. | closed |
| T-052-17 | Release Safety | Rollout default | mitigate | Compatibility remains default unless `Z00Z_ASSET_BACKEND_MODE` explicitly selects forest or dual-verify, and summary evidence in `052-06-SUMMARY.md` records the required validation gates. | closed |
| T-052-18 | Integrity | Benchmark harness | mitigate | Benchmark evidence lives in the landed harness and output files, not ad hoc claims; `052-06-SUMMARY.md` enumerates exact commands and artifacts for async insert/delete, proof timing, recovery, and proof-size runs. | closed |
| T-052-19 | Integrity | Closeout ledger | mitigate | Deferred work is explicitly recorded in `052-TODO.md`, `052-CONTEXT.md`, `052-TEST-SPEC.md`, `052-TESTS-TASKS.md`, and `052-SUMMARY.md`; `052-07-SUMMARY.md` audits that closeout cannot hide follow-up scope. | closed |
| T-052-20 | Integrity | Green-state audit | mitigate | `052-07-SUMMARY.md` audits plans `052-01` through `052-06` against executed evidence before follow-up promotion, closing the risk of a green summary over unfinished fixed-bucket work. | closed |
| T-052-21 | Scope Drift | Phase boundary guardrails | mitigate | `test_future_exports_blocked` and `test_no_occupancy_proof_fields` prove future-only root, rights, fee, migration-proof, and occupancy symbols were not exported into live storage code. | closed |
| T-052-22 | Spoofing | Planning-only candidates | mitigate | Plans `052-08` through `052-11` and their summaries mark candidate work as non-live, and `052-SUMMARY.md` repeats those guardrails at phase closeout. | closed |
| T-052-23 | Privacy | Adaptive bucket candidate | mitigate | No adaptive split runtime shipped; `052-08-SUMMARY.md` keeps adaptive behavior future-only pending privacy review and later phase promotion. | closed |
| T-052-24 | Tampering | Migration proof candidate | mitigate | No live split, merge, epoch, or migration proof placeholder was added; `052-08-SUMMARY.md` keeps old/new root and epoch binding requirements as future duties only. | closed |
| T-052-25 | Integrity | Adaptive recovery candidate | mitigate | No adaptive crash/replay runtime exists in Phase 052, so split/merge fork-risk stays blocked behind a future phase and cannot affect current commits. | closed |
| T-052-26 | Privacy | Occupancy metadata gate | mitigate | Proof-visible counters remain absent from live proof/record types; `test_no_occupancy_proof_fields` blocks `bucket_occupancy`, `occupancy_count`, and related fields. | closed |
| T-052-27 | Tampering | Occupancy version gate | mitigate | Because no occupancy field exists in the proof envelope, no counter can drift into verifier input without a later design/version update; this is guarded by `test_no_occupancy_proof_fields` and `052-09-SUMMARY.md`. | closed |
| T-052-28 | Elevation of Privilege | Occupancy business meaning | mitigate | Downstream code cannot treat bucket activity as authority because counters are not exported and layout authority is blocked by `test_phase052_guardrails.rs`. | closed |
| T-052-29 | Integrity | Settlement-root migration candidate | mitigate | `AssetStateRoot` remains the live oracle, and `test_future_exports_blocked` plus `052-10-SUMMARY.md` confirm `SettlementStateRoot` is not a live Phase 052 export. | closed |
| T-052-30 | Spoofing | Root-generation gate | mitigate | No old/new settlement-root coexistence runtime exists in Phase 052; `052-10-SUMMARY.md` keeps generation binding and checkpoint migration as future-only requirements. | closed |
| T-052-31 | Elevation of Privilege | Root vocabulary boundary | mitigate | No generalized settlement root authority was introduced; checkpoint, proof, wallet, validator, and simulator stay semantic-root bound per `052-10-SUMMARY.md` and source guardrails. | closed |
| T-052-32 | Scope Drift | `RightLeaf` candidate | mitigate | `test_future_exports_blocked` proves `RightLeaf` is absent from live storage exports, and `052-11-SUMMARY.md` keeps it as a future protocol candidate only. | closed |
| T-052-33 | Scope Drift | `FeeEnvelope` candidate | mitigate | `test_future_exports_blocked` proves `FeeEnvelope` is absent from live storage exports, and `052-11-SUMMARY.md` keeps fee support separate from current ownership semantics. | closed |
| T-052-34 | Integrity | Future rights and checkpoint widening | mitigate | `052-11-SUMMARY.md` records proof/checkpoint widening as future-only protocol work; no live Phase 052 proof or checkpoint contract references `RightLeaf` or `FeeEnvelope`. | closed |

Status values: `open`, `closed`
Disposition values: `mitigate`, `accept`, `transfer`

## 📓 Accepted Risks Log

No accepted risks.

## 🧪 Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-29 | 34 | 34 | 0 | Codex / `gsd-secure-phase` |

## ✅ Sign-Off

- [x] All threats have a disposition (`mitigate` / `accept` / `transfer`)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

Approval: verified 2026-05-29
