---
title: Phase 018 Research
phase: "018"
phase_name: "a-b-c"
date_created: 2026-03-24
last_updated: 2026-03-24
owner: GitHub Copilot
status: Planned
tags:
  - research
  - scenario_1
  - storage
  - jmt
  - wallet
---

## Phase 018: 018-A-B-C - Research

**Researched:** 2026-03-24
**Domain:** Scenario 1 canonical storage, JMT proof, wallet receive, and
checkpoint publication flow
**Confidence:** HIGH

## Summary

Phase 018 should not introduce a new stack. The correct implementation path is
to finish adopting the stack that already exists in the repository:
`z00z_storage` owns canonical state, witness, snapshot, checkpoint draft, and
final artifact semantics; `z00z_simulator` should consume those surfaces
directly; `z00z_wallets` should remain responsible for ownership detection and
wallet persistence, not for inventing a second ledger proof model.

The current gaps are not random defects. They all come from three structural
breaks in the Scenario 1 path. First, Stage 4 builds its prep root from the
selected input subset, not from the full claim-published store, so `claim_post`
and `pre_tx` do not form one canonical chain. Second, Stage 6 and Stage 7 stop
at bridge and draft artifacts and never drive a wallet update from the committed
post-apply store, so Charlie never changes. Third, wallet-side verification is
still leaf-oriented and report-oriented, while storage already supports
proof-validated inclusion via `ProofBlob` and `chk_blob(...)`.

A fourth execution gap is still material for acceptance: the scenario already
has transaction-local balance gates and wallet before/after artifacts, but it
does not yet enforce one explicit global wallet-balance invariant over the
refreshed post-apply wallet evidence. That leaves the artifact story weaker
than the codebase can support.

The smallest successful Phase 018 closes those three breaks. Implement Stage 4
continuity from the claim-backed store, add one post-apply JMT wallet scan path
that verifies inclusion before ownership detection, wire that path to Charlie
wallet artifacts, and run Stage 8 in a finalized acceptance mode so the final
checkpoint artifact, link, and audit paths are provable in outputs.

Phase 018 should also add one wallet-balance invariant gate over the refreshed
wallet evidence so the simulator proves not only tx-local balance correctness,
but also coherent wallet totals after the JMT-driven update.

**Primary recommendation:** Implement one storage-backed post-apply wallet scan
helper in `z00z_simulator`, move Stage 4 prep continuity to the full
claim-backed store root, and make Stage 8 finalized mode part of the acceptance
path.

## Project Constraints (from copilot-instructions.md)

- All code, comments, documentation, and technical content must be in English.
- Do not modify `crates/z00z_crypto/tari/`.
- Keep storage-owned boundaries intact. Canonical state, roots, witness bytes,
  checkpoint drafts, and final checkpoint artifacts must stay in
  `z00z_storage`.
- Do not bypass project abstractions in core crates. Use `z00z_utils` for I/O,
  codec, and time abstractions where applicable.
- Prefer typed Rust errors. Do not introduce `unwrap()` or `expect()` into
  production paths.
- Group imports from the same crate in one `use` statement.
- Do not introduce identifiers longer than 5 words.
- Use safe file operations. Do not use destructive delete commands.
- End user-facing work cycles with `scripts/play_tone.sh` when feasible.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
| ------- | ------- | ------- | ------------ |
| `z00z_storage::assets` | workspace | Canonical asset state, path lookup, proof generation | Already owns `AssetStore`, `AssetPath`, `AssetLookup`, `proof_blob`, and `chk_blob(...)`. |
| `z00z_storage::snapshot` | workspace | Canonical pre-state snapshot and replay | Already validates witness bytes and preserves `PrepSnapshot` semantics. |
| `z00z_storage::checkpoint` | workspace | Canonical exec input, draft apply, final artifact, link, audit | Already owns `build_cp_draft`, `seal_artifact`, `save_exec_input`, and `save_audit`. |
| `z00z_simulator::scenario_1` | workspace | Stage orchestration and artifact generation | Already defines Stage 3 through Stage 8 seams and output contracts. |
| `z00z_wallets` stealth receive surfaces | workspace | Ownership detection and wallet persistence | Already owns `receiver_scan_leaf`, `receiver_scan_report`, `StealthOutputScanner`, and `recv_route(...)`. |

### Supporting

| Library | Version | Purpose | When to Use |
| ------- | ------- | ------- | ----------- |
| `z00z_utils::codec::JsonCodec` | workspace | Deterministic artifact encoding | For scenario artifacts and summaries. |
| `z00z_utils::io` | workspace | File and directory operations | For report, summary, and artifact path persistence. |
| `serde` | workspace | Typed JSON serialization | For stable report payloads and stage artifacts. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| ---------- | --------- | -------- |
| `AssetStore::load(...)` + `list(...)` + `proof_blob(...)` | Parse `jmt_artifact` files directly | Worse. The artifact is observational output, while the store API already preserves canonical lookup and proof semantics. |
| `build_cp_draft(...)` + `CheckpointFsStore` | Simulator-local digest or ad hoc root aggregation | Worse. This would duplicate canonical checkpoint semantics and drift from storage-owned roots. |
| `StealthOutputScanner` after proof-validated store enumeration | Fragment-only or tx-output-only receive checks | Worse. That proves ownership of a detached leaf, not inclusion in committed state. |

**Installation:** No new dependency should be added for Phase 018.

**Version verification:** Not applicable. Phase 018 should use the existing
workspace crates and current simulator/storage surfaces.

## Architecture Patterns

### Recommended Project Structure

```text
crates/z00z_simulator/src/scenario_1/
├── stage_3.rs           # Claim publication into storage and claim_post export
├── stage_4.rs           # Tx preparation and prep snapshot continuity
├── stage_5.rs           # Leaf/report-only wallet receive checks
├── stage_6.rs           # Bridge fragments + exec_input
├── stage_7.rs           # Canonical storage-backed apply
├── stage_8.rs           # Final checkpoint publication
├── storage_view.rs      # claim_post / pre_tx / post_tx observational exports
└── <new helper>.rs      # Post-apply JMT wallet scan and Charlie proof path

crates/z00z_storage/src/
├── assets/proof.rs      # `ProofBlob` and `chk_blob(...)`
├── assets/store.rs      # `AssetStore`, lookup, list
├── assets/store_internal/proof_help.rs
├── snapshot/store.rs    # Snapshot witness validation
└── checkpoint/store.rs  # Draft/artifact/link/audit persistence
```

### Pattern 1: Storage-Owned Transition Spine

**What:** The canonical path is claim publication into `AssetStore`, prep
snapshot from storage-owned witness data, exec input build, storage-backed draft
apply, then final artifact sealing.

**When to use:** For every proof-bearing transition that is supposed to survive
reload, RedB reopen, and checkpoint publication.

**Use this pattern:**

- Publish claim outputs into `AssetStore` in Stage 3.
- Build Stage 4 prep inputs from canonical storage paths and witness blobs.
- Use `build_exec_input(...)` in Stage 6 only as the bridge artifact.
- Use `build_cp_draft(...)` in Stage 7 for the canonical apply.
- Use `seal_artifact(...)` in Stage 8 for the final publication surface.

### Pattern 2: Proof Before Ownership Detection

**What:** For JMT-backed wallet discovery, first prove that a leaf belongs to
the committed store root, then run stealth ownership detection on that proven
leaf.

**When to use:** When the user asks for wallet-side verification of JMT
inclusion proofs, or when a wallet update must be justified by canonical state
rather than detached tx artifacts.

**Use this pattern:**

- Load the persisted `AssetStore` from `outputs/storage/post_tx`.
- Enumerate committed `StoreItem`s via `list(...)`.
- Build one `ProofBlob` per `AssetPath`.
- Verify it with `chk_blob(...)`.
- Only after that, call `StealthOutputScanner::scan_leaf(...)` or the wallet
  persistence surface.

### Pattern 3: Draft And Final Surfaces Stay Separate

**What:** Stage 7 owns the draft-state checkpoint summary. Stage 8 owns the
final artifact, link, and audit surfaces.

**When to use:** Always. Draft and final checkpoint surfaces are different
contracts and must not be collapsed into one artifact.

**Use this pattern:**

- Keep `checkpoint_s7.json` draft-derived.
- Keep `checkpoint_s8.json` finalization-derived.
- Add explicit artifact/link/audit file paths only after successful Stage 8
  finalization.

### Pattern 4: Observational View Export, Not Canonical Logic

**What:** `claim_post`, `pre_tx`, and `post_tx` under `storage_view.rs` are
inspection mirrors. They are useful for validation and artifact evidence, but
they are not a substitute for the canonical store and checkpoint contracts.

**When to use:** For acceptance evidence, test reports, and reload-parity
inspection.

### Anti-Patterns to Avoid

- **Do not parse `jmt_artifact` directly for wallet ownership.** Load the
  persisted `AssetStore` and use store-owned proof APIs.
- **Do not treat `claim_post`, `pre_tx`, and `post_tx` summaries as one chain
  unless root continuity is explicitly enforced.** They are separate views today.
- **Do not update Charlie from fragment JSON alone.** Fragments are bridge
  artifacts, not wallet receive proof.
- **Do not use `tx_digest_hex` as a checkpoint proof source.** The existing
  finalized path already uses `pkg.tx.proof` bytes, not `tx_digest_hex`.
- **Do not keep `PassProof` as the only tx-proof gate for the wallet-proof
  story.** It only checks that proof bytes are non-empty.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| ------- | ----------- | ----------- | --- |
| JMT inclusion verification | Custom digest comparisons or manual JSON witness checks | `AssetStore::proof_blob(...)` + `chk_blob(...)` | The storage layer already validates semantic path binding and backend branch proofs. |
| Canonical post-apply root calculation | Simulator-local root aggregation | `build_cp_draft(...)` | The checkpoint crate already applies validated inputs against canonical state. |
| Final checkpoint ids and link binding | Manual checkpoint id strings and file names | `seal_artifact(...)`, `save_link(...)`, `save_audit(...)` | The checkpoint store already enforces id and replay-link binding. |
| Wallet change reports | New one-off diff format | Existing Stage 4 `wallets_state_before/after/diff/report` artifacts | The scenario already has a wallet evidence format. Extend it rather than replacing it. |
| JMT store traversal order | Ad hoc directory traversal over exported files | `AssetStore::list(...)` and `AssetLookup` | Canonical order and path semantics already live in storage. |

**Key insight:** The expensive work is already implemented. The missing work is
adoption and continuity, not new cryptographic plumbing.

## Common Pitfalls

### Pitfall 1: Subset Root Masquerading As Chain Root

**What goes wrong:** `pre_tx` looks like a canonical pre-state root, but Stage 4
currently computes it from the selected input subset.

**Why it happens:** `build_prep_file(...)`, `prep_store(...)`, and `prep_root(...)`
assemble a temporary store from selected rows only.

**How to avoid:** Rebase Stage 4 prep continuity on the full claim-backed store
root, then derive selected path witnesses from that store.

**Warning signs:** `claim_post.view_check_root_hex != pre_tx.source_check_root_hex`.

### Pitfall 2: Ownership Detection Without Inclusion Proof

**What goes wrong:** The wallet proves a leaf is mine but not that the leaf is
actually committed in canonical post-apply state.

**Why it happens:** Stage 3 and Stage 5 use `scan_leaf(...)` and
`receiver_scan_leaf(...)` on detached asset or output material.

**How to avoid:** Run ownership detection only after `chk_blob(...)` succeeds on
the leaf from the persisted post-apply store.

**Warning signs:** Wallet report changes exist, but there is no `ProofBlob`
verification step tied to the store root.

### Pitfall 3: Treating Observational Exports As Consensus Anchors

**What goes wrong:** `claim_post` is mistaken for a canonical chain anchor even
though Stage 3 marks the flow as `wallet_only_intermediate`.

**Why it happens:** The exported view is real, but it is still an observational
mirror of that stage.

**How to avoid:** Add explicit continuity from claim-backed live store to Stage 4
prep root, then record the continuity in one ledger-path artifact.

**Warning signs:** Separate `claim_post`, `pre_tx`, and `post_tx` roots with no
single continuity artifact.

### Pitfall 4: Placeholder Tx Proof Gates

**What goes wrong:** The flow appears proof-checked, but `PassProof` only rejects
empty proof bytes.

**Why it happens:** The checkpoint apply path accepts `TxProofChk`, and the
simulator currently passes a minimal implementation.

**How to avoid:** Keep using checkpoint-owned proof bytes, but add explicit test
coverage for the relationship between Stage 6 exec input, Stage 7 draft, and
Stage 8 final artifact.

**Warning signs:** The flow passes even when only placeholder proof semantics
are exercised.

### Pitfall 5: Draft-Only Acceptance Runs

**What goes wrong:** The final checkpoint artifact path appears missing even when
the code is correct.

**Why it happens:** The current scenario run is in `draft_only` mode.

**How to avoid:** Keep draft-only as a gate test, but add one finalized
acceptance path with `OpaqueTest` proof mode.

**Warning signs:** `checkpoint_s8.json` has `status = draft_only` and
`checkpoint_id_hex = null`.

## Gap Breakdown

| Gap ID | Gap | Repository Evidence | Likely Files / Modules | Minimal Implementation Seam | Confidence |
| ------ | --- | ------------------- | ---------------------- | --------------------------- | ---------- |
| `GAP-01` | Stage 6/7/8 path does not update Charlie wallet runtime | `stage_6.rs` logs `wallet_skip`; `scenario_design.yaml` explicitly forbids `charlie_after_s6.json`; current wallet report shows Charlie unchanged | `crates/z00z_simulator/src/scenario_1/stage_6.rs`, `stage_7.rs`, wallet report helpers | Add one post-apply wallet scan helper after Stage 7 apply, then persist Charlie receive/update artifacts and regenerate wallet evidence files | HIGH |
| `GAP-02` | No full Alice -> Bob -> Charlie asset path | Stage 4 proves Alice -> Bob; Stage 6 creates Alice -> Charlie fragments only; no Charlie receive artifact exists | `stage_4.rs`, `stage_6.rs`, `stage_7.rs` | Reuse Stage 7 post-apply store to discover Charlie-owned outputs and persist Charlie-side receive evidence | HIGH |
| `GAP-03` | Wallet-side verification of JMT inclusion proofs is not in the scenario path | Storage has `proof_blob(...)` and `chk_blob(...)`; scenario grep shows only `scan_leaf` / `receiver_scan_leaf` ownership paths | `z00z_storage/src/assets/proof.rs`, `proof_help.rs`, `stage_5.rs`, new simulator helper | Enumerate `AssetStore` rows from `post_tx`, verify `ProofBlob`, then scan ownership on the proven leaf | HIGH |
| `GAP-04` | Outputs do not contain final sealed checkpoint artifact path | Current `checkpoint_s8.json` is `draft_only`; Stage 8 writes artifact/link/audit only in `OpaqueTest` mode | `stage_8.rs`, `test_stage6_checkpoint_final_gate.rs` | Add finalized acceptance run and record artifact/link/audit paths in Stage 8 summary and report | HIGH |
| `GAP-05` | Tx proof verification in Stage 6/7 is partial or opaque | `PassProof::verify_tx(...)` only checks non-empty proof bytes | `stage_6.rs`, `z00z_storage/src/checkpoint/build.rs` | Keep checkpoint crate ownership, but add explicit tests that the same proof bytes flow from tx package to exec input to final artifact | MEDIUM |
| `GAP-06` | Claim and regular tx do not form one canonical chain path | `claim_post`, `pre_tx`, and `post_tx` are separate exports; `pre_tx` root differs from `claim_post` root | `stage_3.rs`, `stage_4.rs`, `storage_view.rs` | Rebuild Stage 4 prep continuity from the full claim-backed store root instead of a selected-input subset store | HIGH |
| `GAP-07` | Lifecycle transitions are not proven to be driven by wallet JMT scan | `wallets_pending.json` and `wallets_confirmed.json` exist, but they are generated in Stage 4 before a committed post-apply scan | `stage_4.rs`, `stage_7.rs`, wallet report helpers | Add one Stage 7 or Stage 8 JMT-driven receive report that explains which rows were discovered from committed post-tx state | HIGH |
| `GAP-08` | Difference between leaf scan and JMT scan is not explicit | Stage 3 and Stage 5 scan detached leaves; no scenario artifact explains the distinction | `stage_3.rs`, `stage_5.rs`, new report file | Write one explicit scan report artifact and document the distinction in acceptance output | HIGH |
| `GAP-09` | JMT-by-wallet scanning via stealth fields and `tag16` is not found | No scenario path iterates the committed store and filters by stealth ownership; current scan paths are detached-leaf only | New simulator helper, `stage_7.rs` | Implement store enumeration -> proof verification -> stealth ownership detection pipeline | HIGH |
| `GAP-10` | Stage 8 draft-only run blocks proof of final checkpoint artifact and audit path | Existing output and tests already distinguish draft-only and opaque finalization | `stage_8.rs`, existing final gate tests | Keep draft-only tests, add one opaque finalized scenario acceptance and path assertions | HIGH |
| `GAP-11` | No explicit global wallet-balance invariant gate exists over the refreshed wallet evidence | Stage 4 has tx-local balance gates plus wallet before/after artifacts, but there is no one acceptance gate proving coherent wallet totals after the committed post-apply update | `stage_4.rs`, `stage_4_utils/reports_diff.rs`, `stage_7.rs`, wallet report helpers | Reuse the existing wallet evidence surfaces to add one explicit post-apply invariant gate across before/after or confirmed rows | MEDIUM |

## Minimal Proof Path

The smallest end-to-end artifact path that closes the requested proof story is:

1. **Stage 3 canonical claim publication**
   - Publish claim packages into one `AssetStore` with `publish_claims_store(...)`.
   - Export `outputs/storage/claim_post` from that live store.

2. **Stage 4 continuity from the full claim-backed store**
   - Stop deriving `prep.prev_root_hex` from a temporary store built only from
     selected inputs.
   - Derive selected witness rows from the full claim-backed store root, so
     Stage 4 continuity can reference the same canonical claim state.

3. **Stage 6 bridge only**
   - Keep fragments and exec input persistence.
   - Do not attempt wallet mutation here.

4. **Stage 7 canonical apply plus JMT wallet scan**
   - Apply the draft with `build_cp_draft(...)`.
   - Load `outputs/storage/post_tx/asset_state.redb` through `AssetStore::load(...)`.
   - Enumerate committed store rows.
   - For each row, verify inclusion with `proof_blob(...)` and `chk_blob(...)`.
   - Run `StealthOutputScanner::scan_leaf(...)` only on proof-validated rows.
   - Persist Charlie scan and receive artifacts.

5. **Stage 7 wallet evidence refresh**
   - Write one Charlie-focused artifact such as
     `outputs/transactions/charlie_jmt_scan.json`.
   - Regenerate wallet before/after/diff/report artifacts so Charlie changes are
     visible in the standard report surface.
   - Run one wallet-balance invariant gate over the refreshed evidence so the
     post-apply wallet totals are proven coherently, not only described.

6. **Stage 8 finalized acceptance path**
   - Run finalization in `OpaqueTest` mode for at least one acceptance path.
   - Record `checkpoint_id_hex` and concrete artifact/link/audit paths.
   - Export one ledger-path artifact tying together claim root, prep root,
     post-apply root, draft id, and checkpoint id.

## Execution Gates

Phase 018 is not safe to execute out of order.

1. Repair Stage 4 continuity before treating `pre_tx` as part of one canonical
  claim-plus-regular path.
2. Keep Stage 6 bridge-only; do not attach wallet mutation here.
3. Run the new wallet scan only after Stage 7 canonical apply and persisted
  `post_tx` state exist.
4. Refresh wallet evidence only after committed-store proof verification passes.
5. Run the wallet-balance invariant gate before accepting Charlie wallet output
  as complete.
6. Run Stage 8 finalization only after continuity, JMT proof verification,
  Charlie wallet refresh, and wallet invariant gates all pass.

If a gate fails, keep the draft or bridge artifacts for debugging, but do not
promote the run to accepted wallet evidence or finalized checkpoint output.

## Clarifications

### Scanning By Leaf Or Asset

This is what Stage 3 and Stage 5 do today.

- The caller already has an `Asset` or `AssetLeaf`.
- The wallet runs `receiver_scan_leaf(...)`, `receiver_scan_report(...)`, or
  `StealthOutputScanner::scan_leaf(...)`.
- The result answers: "Does this leaf belong to this wallet?"

This does **not** answer: "Is this leaf committed in the canonical store root
that the scenario is claiming to use?"

### Scanning By JMT

This is what Phase 018 must add for the proof-complete path.

- Load the committed `AssetStore` from the persisted post-apply state.
- Enumerate canonical `AssetPath`s through storage-owned APIs.
- Build a `ProofBlob` for each path.
- Verify it with `chk_blob(...)` against the committed root.
- Only then run stealth ownership detection on the proven leaf.

This answers both questions:

- "Is this committed in the canonical ledger state?"
- "Is it mine?"

### Why `tag16` And Stealth Fields Are Not A JMT Scan By Themselves

`tag16`, `owner_tag`, `r_pub`, and `enc_pack` are ownership hints carried by the
leaf. They are sufficient for ownership detection once the leaf is already in
hand. They are not a ledger traversal mechanism. A JMT scan still needs a
canonical source of committed leaves and inclusion verification.

### Why The Current `pre_tx` View Is Not One Canonical Chain Anchor

`stage_4.rs` currently builds `PrepFile` and `prep_root(...)` from selected rows
only. That is enough for the spend-witness gate, but it is not enough to prove
continuity from the full claim-published store. This is the root cause of the
missing merged claim-plus-regular chain path.

## Code Examples

Verified repository patterns that should be reused directly:

### Canonical Inclusion Verification From A Persisted Store

```rust
// Source: crates/z00z_storage/src/assets/store_internal/proof_help.rs
// Source: crates/z00z_storage/src/assets/proof.rs

let page = store.list(AssetListReq::all(10_000))?;
for item in page.items() {
    let path = item.path();
    let blob = store.proof_blob(&path)?;
    let bytes = blob.encode()?;
    let proof_item = blob.item();

    chk_blob(
        &bytes,
        proof_item.root(),
        &path,
        proof_item.def_leaf(),
        proof_item.ser_leaf(),
        item.leaf(),
    )?;
}
```

### Canonical Draft Apply

```rust
// Source: crates/z00z_simulator/src/scenario_1/stage_7.rs

let link = draft_link(snap_id, exec_id)?;
let draft = build_cp_draft(
    stage_id as u64,
    snap_id,
    &prep,
    &replay,
    &link,
    &exec,
    &PassProof,
    &NoSpent,
)?;
let draft_id = checkpoint_store.save_draft(&draft)?;
```

### Final Artifact Publication

```rust
// Source: crates/z00z_simulator/src/scenario_1/stage_8.rs

let proof = build_cp_proof(&draft, &pkg)?;
let link = checkpoint_store.seal_artifact(&draft, proof, snap_id, exec_id)?;
checkpoint_store.save_audit(&audit)?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
| ------------ | ---------------- | ------------ | ------ |
| Detached leaf scan proves receive | Proof-validated store scan should prove receive | Storage proof surfaces already exist; scenario adoption is incomplete | Closes the wallet-side inclusion gap. |
| Stage 4 prep root from selected subset | Stage 4 should derive continuity from full claim-backed store root | Needed now for Phase 018 | Closes the merged chain-path gap. |
| Draft-only Stage 8 as the only observed path | Keep draft-only for negative gates, add one finalized acceptance path | Existing code already supports `OpaqueTest` | Makes final artifact, link, and audit path provable. |
| Placeholder `PassProof` gate | Keep checkpoint-owned proof bytes and add explicit propagation tests | Current code still placeholder-heavy here | Avoids false confidence about tx proof coverage. |

**Deprecated or insufficient for this phase:**

- Fragment-only Charlie artifacts as a substitute for wallet update evidence.
- `scan_leaf(...)` on detached assets as a substitute for JMT-backed receive.
- Treating `claim_post` as a canonical chain anchor without Stage 4 root
  continuity.

## Plan Skeleton

1. **Repair root continuity in Stage 4**
   - Update `crates/z00z_simulator/src/scenario_1/stage_4.rs` so prep witness
     derivation comes from the full claim-backed live store, not from a
     selected-input subset store.
   - Extend `storage_view.rs` or a new ledger-path artifact to record continuity
     from `claim_post` into the Stage 4 prep root.

2. **Add one post-apply JMT wallet scan helper**
   - Create one focused helper module under
     `crates/z00z_simulator/src/scenario_1/`.
   - Input: persisted `post_tx` store root, actor keys, actor wallet id.
   - Output: proof-validated owned rows plus machine-readable report artifact.

3. **Wire Charlie proof path into Stage 7**
   - Call the helper after `export_post_tx_view(...)` in
     `crates/z00z_simulator/src/scenario_1/stage_7.rs`.
   - Persist Charlie receive/update artifacts and refresh wallet evidence files.

4. **Make the chain path explicit**
   - Write one canonical ledger-path artifact, for example
     `outputs/storage/ledger_path.json`, with claim root, prep root, post-apply
     root, draft id, exec input id, and final checkpoint id when present.

5. **Expose final checkpoint publication paths**
   - Extend `crates/z00z_simulator/src/scenario_1/stage_8.rs` to record
     artifact/link/audit paths in `checkpoint_s8.json` and any top-level report
     when finalization succeeds.

6. **Add one wallet-balance invariant gate**
   - Reuse the current wallet before/after/diff or confirmed rows to prove
     coherent totals after the JMT-driven update.
   - Keep this gate scenario-level; do not replace the existing tx-local balance
     checks in Stage 4.

7. **Add Phase 018 test coverage**
   - Add one test for Charlie full path after Stage 7 apply.
   - Add one test for proof-validated JMT wallet scan.
   - Add one test for Stage 4 continuity from claim-backed root.
   - Add one test for the wallet-balance invariant gate over refreshed wallet
     evidence.
   - Reuse existing Stage 8 final gate tests for finalized path assertions.

## Resolved By Context

1. **Charlie wallet update shape**
   - Resolved by `018-CONTEXT.md` D-07 and D-08.
   - Phase 018 must perform a real post-apply Charlie wallet runtime update and
     expose that change through the standard wallet evidence surface.

2. **Tx proof hardening depth**
   - Resolved by `018-CONTEXT.md` D-10 through D-12.
   - Phase 018 must strengthen proof propagation and binding coverage across
     Stage 6, Stage 7, and Stage 8, but must not widen cryptographic scope
     unless a failing requirement proves that necessary.

## Environment Availability

This phase is a code-and-artifact change inside the existing Rust workspace.
No new external service is required beyond the current Cargo toolchain and the
existing simulator runtime.

## Validation Architecture

### Test Framework

| Property | Value |
| -------- | ----- |
| Framework | Rust `cargo test` in the workspace |
| Config file | `Cargo.toml` workspace manifests |
| Quick run command | `cargo test -p z00z_simulator --test test_stage6_checkpoint_final_gate --release --features test-fast --features wallet_debug_dump` |
| Full suite command | `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| ------ | -------- | --------- | ----------------- | ------------ |
| `GAP-01` | Charlie wallet changes after canonical apply | integration | `cargo test -p z00z_simulator --test test_phase18_charlie_path --release --features test-fast --features wallet_debug_dump` | ❌ Wave 0 |
| `GAP-03` | Wallet-side JMT proof verification runs before ownership detection | integration | `cargo test -p z00z_simulator --test test_phase18_jmt_wallet_scan --release --features test-fast --features wallet_debug_dump` | ❌ Wave 0 |
| `GAP-04` | Finalized Stage 8 exports artifact/link/audit paths | integration | `cargo test -p z00z_simulator --test test_stage6_checkpoint_final_gate --release --features test-fast --features wallet_debug_dump` | ✅ |
| `GAP-06` | Claim-backed root continuity reaches Stage 4 prep | integration | `cargo test -p z00z_simulator --test test_phase18_chain_continuity --release --features test-fast --features wallet_debug_dump` | ❌ Wave 0 |
| `GAP-07` | Lifecycle evidence is driven by post-apply JMT scan | integration | `cargo test -p z00z_simulator --test test_phase18_wallet_lifecycle_from_jmt --release --features test-fast --features wallet_debug_dump` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p z00z_simulator --test test_stage6_checkpoint_final_gate --release --features test-fast --features wallet_debug_dump`
- **Per wave merge:** `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- **Phase gate:** `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`

### Wave 0 Gaps

- [ ] `crates/z00z_simulator/tests/test_phase18_charlie_path.rs` — prove
  Charlie wallet update after Stage 7.
- [ ] `crates/z00z_simulator/tests/test_phase18_jmt_wallet_scan.rs` — prove
  store enumeration, `ProofBlob`, `chk_blob(...)`, and ownership detection.
- [ ] `crates/z00z_simulator/tests/test_phase18_chain_continuity.rs` — prove
  `claim_post -> prep -> post_tx -> checkpoint` continuity artifact.
- [ ] `crates/z00z_simulator/tests/test_phase18_wallet_lifecycle_from_jmt.rs`
  — prove lifecycle reporting is driven by the JMT-backed scan path.

## Sources

### Primary (HIGH confidence)

- Repository: `.planning/phases/018-a-b-c/todo.md` — explicit gap inventory and
  observed outputs.
- Repository: `crates/z00z_simulator/src/scenario_1/stage_3.rs` — claim
  publication and `claim_post` export.
- Repository: `crates/z00z_simulator/src/scenario_1/stage_4.rs` — prep file,
  subset-root construction, canonical snapshot build, wallet evidence output.
- Repository: `crates/z00z_simulator/src/scenario_1/stage_5.rs` — detached
  leaf ownership scan path.
- Repository: `crates/z00z_simulator/src/scenario_1/stage_6.rs` — bridge
  semantics, `wallet_skip`, exec input build, placeholder tx proof check.
- Repository: `crates/z00z_simulator/src/scenario_1/stage_7.rs` — canonical
  storage-backed draft apply and `post_tx` export.
- Repository: `crates/z00z_simulator/src/scenario_1/stage_8.rs` — finalization
  gating and artifact publication.
- Repository: `crates/z00z_simulator/src/scenario_1/storage_view.rs` —
  `claim_post`, `pre_tx`, `post_tx` observational exports.
- Repository: `crates/z00z_storage/src/assets/proof.rs` — `ProofBlob` and
  `chk_blob(...)`.
- Repository: `crates/z00z_storage/src/assets/store_internal/proof_help.rs` —
  `proof_blob(...)` generation.
- Repository: `crates/z00z_storage/src/checkpoint/build.rs` — draft apply and
  `TxProofChk` seam.
- Repository: `crates/z00z_storage/src/checkpoint/store.rs` — artifact/link/audit
  store semantics.
- Repository: `crates/z00z_simulator/tests/test_stage6_checkpoint_final_gate.rs`
  — finalized versus draft-only checkpoint coverage.
- Repository artifacts:
  `crates/z00z_simulator/src/scenario_1/outputs/storage/claim_post/summary.json`,
  `pre_tx/summary.json`, `post_tx/summary.json`, and
  `outputs/transactions/checkpoint_s8.json`.

### Secondary (MEDIUM confidence)

- `crates/z00z_storage/src/assets/README.MD` — internal design guidance for
  proof validation and search surface.
- `crates/z00z_storage/src/assets/root-types.md` — internal explanation of the
  proof chain and root semantics.

### Tertiary (LOW confidence)

- None. No critical claims in this document depend on external or unverified
  sources.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — the stack is already fixed by repository ownership
  boundaries.
- Architecture: HIGH — root causes are visible in current stage code and
  current artifacts.
- Pitfalls: HIGH — every listed pitfall is backed by repository evidence.

**Research date:** 2026-03-24
**Valid until:** 2026-04-23
