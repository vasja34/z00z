# 065 Attack Surface TODO

Status: authoritative Phase 065 consolidation  
Last re-verified against live code: 2026-06-30  
Audience: implementers closing the remaining attack-surface backlog

This file replaces the old Phase 065 Markdown reports as the human-readable source of truth. The old Markdown documents may be deleted after this file is accepted. The JSONL catalogs may be archived as evidence, but they are no longer required to understand what must be done.

Superseded documents:
- `readme.md`
- `attack-surfaces-resolve-spec.md`
- `attack-surfaces-TODO.md`
- `attack-surfaces-AUDIT-2.md`
- `attack-surfaces-AUDIT-3.md`
- `attack-surfaces-placeholders.md`
- `attack-surfaces-wallet-simul.md`
- `060-attack-surface-report.md`
- `060-attack-surface-db.jsonl`
- `062-attack-surface-report.md`
- `062-attack-surface-db.jsonl`
- `064-attack-surface-report.md`
- `064-attack-surface-db.jsonl`
- `attack-surface-db.jsonl`
- `attack-surface-crates-report.md`
- `attack-surface-crates.jsonl`
- `attack-surface-crates-symbols.jsonl`
- `attack-surface-crates-inventory.md`
- `attack-surface-crates-security-snapshot.md`
- `z00z-verification-report.md`

## Executive conclusion

The old Phase 065 material mixed three different classes of findings:

- still-live implementation gaps that need code or build-policy changes;
- boundaries that are mostly defended now, but still depend on convention, tests, or naming and therefore must stay sealed by regression gates;
- historical or overbroad claims whose original wording no longer matches the live tree.

This file keeps only the backlog that still makes sense after a fresh repository check. Do not reopen retired findings verbatim unless a new code path reproduces them again.

## Deletion Contract

The purpose of this section is to make deletion of the remaining Phase 065 files explicit rather than implied.

### Files that can be deleted after accepting this TODO

The following files do not contain unique implementation requirements anymore:

- all prior Phase 065 Markdown reports and TODO drafts;
- all phase-local attack-surface JSONL catalogs;
- crate inventory and security snapshot outputs;
- the phase-local symbol inventory JSONL;
- the phase-local verification orchestrator report.

### Why deletion is safe

- Backlog authority now lives here. Every open, sealed, historical, and policy-gate item that still matters is spelled out in this file.
- Finding coverage now lives here. Source finding ids from the legacy catalog, crate scan, and 060/062/064 scans are mapped in this file.
- Gate contracts now live here. The old reports distributed gate semantics across several documents; this file now centralizes canonical inputs, outputs, proofs, and verification obligations.
- The remaining non-TODO files are either raw evidence snapshots, machine-generated inventories, or superseded narrative reports. They may be useful as historical breadcrumbs, but they are not required to know what must be implemented.

### Absorbed non-backlog artifacts

- `attack-surface-crates-inventory.md` is a tool/environment inventory. Its contents are workspace package lists, dependency tree excerpts, and missing-tool observations, not unique remediation tasks.
- `attack-surface-crates-security-snapshot.md` is a fast dependency/advisory snapshot. Its contents are dependency-tree context and tool-availability state, not distinct Phase 065 findings.
- `attack-surface-crates-symbols.jsonl` is a machine-generated symbol inventory. It provides search aid only; it does not define closure work.
- `z00z-verification-report.md` is a single run-local orchestrator report. Its actionable meaning is absorbed into this TODO as closure gates and repository-wide meta-gates. The exact 2026-06-30 run-local gate statuses were:
  - `l0-docs`
  - `l1-alloy`
  - `l1-apalache`
  - `l1-tla`
  - `l2-cryptol`
  - `l2-domain`
  - `l2-proverif`
  - `l2-refinement-map`
  - `l2-saw`
  - `l2-tamarin`
  - `l2-transcript`
  - `l3-kani`
  - `l3-miri`
  - `l3-verify-fast`
  - `l4-adversarial-review`
  - `l4-constant-time`
  - `l4-fuzz`
  - `l4-supply-chain`
  - `l4-unsafe`
- Those exact per-run logs are not needed to understand the attack-surface backlog. What matters is the enduring action: keep closure gated by docs/spec checks, proof/model checks where applicable, implementation verification, fuzz/constant-time/supply-chain review, and adversarial review.

## Verification model used for this consolidation

Every carried item passed the following three checks:

1. the finding existed in a Phase 065 report or legacy attack-surface catalog;
2. live code and current tests were re-checked on 2026-06-30;
3. the finding was classified as one of:
   - `Open`: still needs implementation work;
   - `Seal`: current code is materially defended, but the boundary must stay under explicit regression coverage;
   - `Historical/Narrowed`: the old wording is no longer accurate enough to keep as an active task.

## Legacy CF Numbering Note

The old append-only `CF-*` numbering is not stable across every historical
Phase 065 audit. Some ids were reused in later append-only runs for different
candidate sets. This TODO keeps the current-tree disposition, not a blind
one-to-one replay of every legacy numbering collision.

- Legacy `CF-003` was reused for two different surfaces in old reports:
  - the raw-checkpoint-artifact / compatibility-proof split, now carried by
    `WS-02`;
  - the `wallet.key.rotate_master_key` placeholder-semantics wording, now
    narrowed into `WS-06` instead of kept as a standalone live blocker.
- Legacy `CF-005` referred to the guarded `claim_v1` compatibility lane. It
  does not reproduce as a current production-trusted-path blocker on the live
  tree. Reopen it only if a default or public build depends on that lane
  again.

## Priority order

1. `WS-01` theorem-verified validator acceptance
2. `WS-02` canonical checkpoint persistence and proof semantics
3. `WS-03` release/build hardening for debug and test-only surfaces
4. `WS-04` release-packet truth for simulator draft/debug lanes
5. `WS-05` capability-typed sealing for privileged wallet paths
6. `WS-06` canonical ownership of wallet mutation and restore truth
7. `WS-07` fail-closed construction and operator-visible logging
8. `WS-08` placeholder public RPC and stub DTO cleanup
9. `WS-09` document and source sweep for the few narrowed historical leftovers

## Canonical Gate Inventory: Inputs, Outputs, Verification, Proofs

This section is the cross-cutting contract checklist that was missing from the older Phase 065 material. Every future closure pass must check these gates explicitly instead of reasoning from broad summaries.

### G-01: Rollup Settlement Theorem Gate

Owner:
- `z00z_rollup_node::verify_settlement_theorem()` in `crates/z00z_rollup_node/src/lib.rs`

Canonical inputs:
- `TxPackage`
- `CheckpointArtifact`
- `CheckpointExecInput`
- `CheckpointLink`

Authoritative output:
- `Result<(), SettlementError>` proving one canonical public settlement theorem

Verification and proof obligations:
- `TxPackage` structure verification
- `tx_digest_hex` matches canonical digest
- public spend contract verifies
- artifact exposes `CheckpointStatement::CURRENT`
- `artifact.cp_proof()` equals statement backend payload
- encoded `CheckpointExecInput` derives the same `exec_input_id` carried by statement and link
- `prep_snapshot_id`, `prev_root`, and checkpoint id all agree across artifact, exec input, tx package, and link
- tx package is actually included in the exec input replay set

Current gap:
- `ResolvedBatch` and validator acceptance still do not require this full input bundle on the accepted path

Carried by:
- `WS-01`

### G-02: Validator Checkpoint And Publication Gate

Owner:
- `CheckpointFlow::try_from_resolved()` in `crates/z00z_runtime/validators/src/checkpoint.rs`
- `ValidatorBoundary::verdict_for_batch()` in `crates/z00z_runtime/validators/src/engine.rs`

Canonical inputs:
- `ResolvedBatch { published, ordered, artifact, nullifiers, placement, exec_ticket }`

Authoritative output:
- `Verdict { batch_id, checkpoint_id, publication, kind, reject, object_verdicts }`

Verification and proof obligations:
- batch ids agree across `published` and `ordered`
- checkpoint id derived from artifact equals published checkpoint id
- `published.pub_in` equals `artifact.pub_in()`
- runtime route and ordered route agree
- route binding verifies against route-table digest and publication checkpoint
- publication binding digest is derived from batch id, checkpoint id, route-table digest, and `pub_in`

Current gap:
- this gate verifies publication and checkpoint coherence, but it can still reach `Accepted` without owning the full theorem inputs from `G-01`

Carried by:
- `WS-01`

### G-03: Checkpoint Seal Gate

Owner:
- `CheckpointStore::seal_artifact()` in `crates/z00z_storage/src/checkpoint/store.rs`

Canonical inputs:
- `CheckpointDraft`
- `CheckpointProof`
- `PrepSnapshotId`
- `CheckpointExecInputId`

Authoritative output:
- `CheckpointLink`

Verification and proof obligations:
- statement `prep_snapshot_id` and `exec_input_id` match the supplied ids
- referenced snapshot and exec-input rows already exist
- replay and root checks pass before final artifact persistence
- final artifact is produced from `draft.finalize(proof)`
- resulting link binds artifact, snapshot, and exec-input ids

Current gap:
- raw `save_artifact()` and standalone `save_link()` remain peer-visible neighboring lanes, which keeps canonical-vs-raw misuse risk live

Carried by:
- `WS-02`

### G-04: Checkpoint Link Bind And Codec Gate

Owner:
- `CheckpointLink::new()`
- `encode_link_bin_checked()`
- `decode_link_bin_checked()`
- `encode_link_json_checked()`
- `decode_link_json_checked()`
- all in `crates/z00z_storage/src/checkpoint/link.rs`

Canonical inputs:
- `CheckpointId`
- `PrepSnapshotId`
- `CheckpointExecInputId`

Authoritative output:
- version-checked `CheckpointLink` with a valid `link_bind`

Verification and proof obligations:
- link version is current
- `link_bind` hashes the exact triple `(checkpoint_id, prep_snapshot_id, exec_input_id)`
- encoded and decoded link forms preserve the same bind

Current gap:
- the link object itself is self-consistent, but write-time store rules still need stronger guarantees that the referenced evidence rows already exist and match

Carried by:
- `WS-02`

### G-05: Privileged Session Gate

Owner:
- `verify_session()` and `verify_session_no_touch()` in `crates/z00z_wallets/src/services/wallet_session_runtime_limits.rs`
- representative privileged route: `rotate_master_key_checked()` in `crates/z00z_wallets/src/rpc/key_rpc_server_admin.rs`

Canonical inputs:
- `SessionToken`
- privileged RPC method parameters

Authoritative output:
- typed `WalletResult<VerifiedSession>` / `WalletResult<VerifiedSessionNoTouch>` before any privileged action proceeds

Verification and proof obligations:
- native path validates the session token against the in-memory session manager
- `verify_session()` also refreshes wallet activity
- wasm path fail-closes with explicit typed unsupported-capability errors
- privileged RPCs audit deny/rate-limit/success states around the guard

Current gap:
- the guarantee still depends on every privileged route remembering to call the right guard instead of accepting a typed verified capability

Carried by:
- `WS-05`

### G-06: Wallet Mutation Submission Gate

Owner:
- `submit_local_asset_mutation()` in `crates/z00z_wallets/src/rpc/asset_rpc_support_state.rs`
- `BroadcastImpl` in `crates/z00z_wallets/src/chain/broadcast_impl.rs`

Canonical inputs:
- `wallet_id`
- operation label
- input assets
- output assets

Authoritative output:
- persisted wallet tx id on the canonical local mutation path

Verification and proof obligations:
- local mutation package bytes are built from a deterministic digest seed over wallet, operation, inputs, and outputs
- the package is broadcast through `BroadcastImpl`
- durable tx lifecycle persistence is owned by `TxStorage`
- tx id is derived from the broadcast result rather than invented separately by each RPC method

Current gap:
- canonical truth still emerges from helper composition instead of one sealed mutation executor type

Carried by:
- `WS-06`

### G-07: Atomic Restore Gate

Owner:
- `restore_wallet_pack_atomic()` in `crates/z00z_wallets/src/services/wallet_actions_backup.rs`

Canonical inputs:
- `WalletExportPack`
- password
- optional wallet name override
- `WalletIdentity`
- optional history bytes

Authoritative output:
- `PersistWalletId` for the restored wallet, or a fail-closed rollback error

Verification and proof obligations:
- restore pack is validated before staging
- staged `.wlt` and optional staged history are written first
- existing `.wlt` and history bytes are captured for rollback
- history commit, `.wlt` commit, and publish each have rollback handling
- publish installs restored profile and claimed assets only after file commits succeed

Current gap:
- the flow is careful but still lacks one explicit durable restore journal/marker spanning the full multi-step boundary

Carried by:
- `WS-06`

### G-08: Public Chain Scan And Tip RPC Gate

Owner:
- `ChainScanRpc` in `crates/z00z_wallets/src/rpc/chain_rpc.rs`
- DTOs in `crates/z00z_wallets/src/rpc/chain_types.rs`
- implementation in `crates/z00z_wallets/src/services/chain_service.rs`

Canonical inputs:
- `RuntimeStartScanParams`
- `PersistWalletId`

Authoritative outputs:
- `RuntimeStartScanResponse`
- `RuntimeScanStatus`
- `RuntimeBlockInfo`

Verification and proof obligations today:
- none beyond process-local consistency of the in-memory scan state
- current outputs are synthetic local orchestration objects, not durable or remote-backed chain truth

Current gap:
- public RPC names and DTOs still look authoritative while the underlying source of truth is explicitly process-local placeholder state

Carried by:
- `WS-08`

### G-09: Transaction Receipt And Verification DTO Gate

Owner:
- transaction DTOs in `crates/z00z_wallets/src/rpc/tx_types.rs`
- projection helpers in `crates/z00z_wallets/src/rpc/tx_runtime_state.rs`
- admission/confirmation conversions in `crates/z00z_wallets/src/rpc/tx_rpc_admission.rs`

Canonical inputs:
- `RuntimeConfirmationReceipt`
- `TxConfirmationEvidence`
- durable tx record state

Authoritative outputs:
- `PersistReceiptInfo`
- `RuntimeVerifyTxPkgResponse`
- confirmation/admission receipt DTOs exposed over RPC

Verification and proof obligations today:
- `verified` is carried as a boolean summary
- receipt projection currently stores a `merkle_proof` field that is populated from root-style values rather than from a dedicated proof object
- package verification responses expose lifecycle and validity summaries, but proof-bearing DTO semantics are still mixed with compatibility and placeholder fields

Current gap:
- public tx/receipt DTOs still blur the line between "verified summary", "checkpoint/root evidence", and "actual proof payload", so proof-shaped output cleanup remains part of the active backlog

Carried by:
- `WS-08`

## WS-01: Theorem-Verified Validator Acceptance Must Be Mandatory

Sources:
- `AS-20260627-001`
- `AS-20260630-064-11`
- narrowed remainder of old `CF-006`

Current reality:
- `crates/z00z_rollup_node/src/lib.rs` already contains a real settlement-theorem verifier.
- `crates/z00z_rollup_node/src/da.rs` currently resolves a batch into `ResolvedBatch` without carrying the full theorem input set as validator-owned data.
- `crates/z00z_runtime/validators/src/verdict.rs` and `crates/z00z_runtime/validators/src/engine.rs` still let the validator reason mostly from the artifact, ordered batch, and object verdict surface.
- This leaves a gap between "the theorem can be checked" and "acceptance cannot happen without it."

Required implementation:
- Extend the resolved-batch contract so validator-owned acceptance inputs include the canonical checkpoint link, execution-input identity, and the public theorem material required to prove one coherent publication story.
- Remove any acceptance path that can produce `Accepted` without theorem verification.
- Make DA resolve, validator acceptance, and publication-binding handling consume the same typed structure instead of rebuilding or re-inferring key public inputs in multiple places.
- Ensure publication binding, route-table digest, checkpoint id, and theorem inputs are checked together, not as loosely related post-fact fields.
- Keep missing theorem inputs fail-closed. "Optional" theorem data must not exist on the accepted path.

Required tests:
- Negative tests for wrong checkpoint link, wrong exec input id, wrong prep snapshot id, wrong route digest, detached proof blob, and mismatched publication binding.
- Cross-crate tests that one accepted batch has one theorem story and one publication binding digest.
- Validator tests proving `Accepted` is unreachable when theorem inputs are absent.

Close when:
- validator acceptance cannot compile or run without the theorem input bundle;
- all accepted paths verify theorem, publication binding, and link coherence together;
- artifact-only acceptance no longer exists.

## WS-02: Canonical Checkpoint Persistence Must Be One-Way, And Raw Lanes Must Be Quarantine-Only

Sources:
- `AS-20260630-064-10`
- `AS-20260501-029`
- `AS-20260501-030`
- `CF-002`
- carried parts of `CF-010`

Current reality:
- `crates/z00z_storage/src/checkpoint/store.rs` still exposes both `save_artifact()` and `seal_artifact()`.
- `save_artifact()` is explicitly a raw, noncanonical lane, but it persists the same broad object family as the canonical seal path.
- `save_link()` still writes a link after local uniqueness checks, while snapshot and exec-row coherence are only fully revalidated later on load.
- Compatibility `cp_proof` and draft-proof byte semantics remain weaker than the final attested seal path, even though the persisted canonical path is stronger.

Required implementation:
- Move the raw artifact lane behind a clearly named noncanonical export capability or internal module. Canonical callers must not see it as a peer API.
- Make `save_link()` fail if replay-evidence rows are not already present and coherent at write time.
- Split canonical attested proof types from compatibility or draft proof bytes. Do not keep one ambiguous field name for both.
- Rename compatibility-only proof fields and helper types so downstream code cannot mistake them for a final theorem-bearing checkpoint proof.
- Add provenance or lane markers so downstream code can reject raw-export artifacts when a canonical sealed artifact is required.
- Rewrite tests that currently normalize the raw lane as a canonical persistence pattern. Keep raw-lane tests only where the surface is explicitly noncanonical.

Required tests:
- Negative tests for "link before evidence row exists", mismatched snapshot or exec ids, mismatched statement/proof pairs, and raw artifact reuse on canonical paths.
- Tests proving stage-12 finalization, validator acceptance, and publication export use only the seal path.
- Tests proving a raw-export artifact cannot be loaded as if it were a canonical final artifact.

Close when:
- canonical checkpoint artifacts can only be born through `seal_artifact()`;
- compatibility proof bytes are no longer semantically confusable with canonical proofs;
- raw persistence survives only as an explicitly quarantined export surface.

## WS-03: Debug And Test-Only Surfaces Must Be Impossible In Release-Capable Builds

Sources:
- crate `AS-20260623-001`
- crate `AS-20260623-003`
- `AS-20260501-027`
- `AS-20260501-031`

Current reality:
- `crates/z00z_storage/src/settlement/hjmt_cache.rs` and `crates/z00z_storage/src/settlement/hjmt_scheduler.rs` still expose public corruption and scheduler-test knobs in normal builds.
- `crates/z00z_wallets/src/db/mod.rs` and `crates/z00z_wallets/src/wallet/mod.rs` still re-export `debug_export_wallet` behind `wallet_debug_tools`.
- `crates/z00z_simulator` still contains private plaintext debug artifact lanes for wallet secrets behind `wallet_debug_tools`.
- `test-params-fast` still weakens wallet KDF settings on release-capable build paths.

Required implementation:
- Add workspace-level compile guards so `test-params-fast` cannot be used in release-capable builds outside explicitly internal-only tooling.
- Add workspace-level compile guards so `wallet_debug_tools` cannot be included in public or production-shaped release builds.
- Move `debug_export_wallet` and plaintext debug artifact emission into test-only modules or a separate internal-only tool surface.
- Gate settlement cache corruption and scheduler test controls behind `cfg(test)` or a dedicated internal test-only feature that downstream release crates cannot enable by accident.
- Remove release documentation that normalizes running production-shaped binaries with weakened KDF or secret-export features enabled.

Required tests:
- Expected-failure build checks such as:
  - `cargo check -p z00z_wallets --release --features test-params-fast`
  - `cargo check -p z00z_simulator --release --features wallet_debug_tools`
  - any equivalent workspace matrix that proves these feature combinations are rejected
- Source-audit tests proving release builds do not export settlement corruption hooks or wallet secret debug exporters.

Close when:
- release-capable builds cannot carry weakened KDF settings, plaintext secret export tools, or public cache/scheduler corruption hooks;
- the only remaining debug surfaces are test-only or explicitly internal-only.

## WS-04: Draft And Debug Simulator Paths Must Not Emit Production-Shaped Evidence

Sources:
- `AS-20260630-064-01`
- `AS-20260630-064-07`
- narrowed simulator remainder of old `CF-002`

Current reality:
- `crates/z00z_simulator/src/config.rs` still keeps `Stage6ProofMode::DraftOnly` as a live mode.
- `crates/z00z_simulator/src/scenario_1/stage_12/mod.rs` exits early on `DraftOnly`.
- `crates/z00z_simulator/src/scenario_1/runtime_observability.rs` still synthesizes binding-shaped publication evidence for draft-only status.
- The default public lane is currently guarded against plaintext secret artifact emission, but the draft/debug branches still pressure that boundary.

Required implementation:
- Remove `DraftOnly` from any release or publication-packet contract, or move it into an explicitly debug-only packet schema that cannot be mistaken for final evidence.
- Require publication-binding evidence to reference a real finalized checkpoint id, not a synthetic stand-in derived from batch identity.
- Split public release artifacts from private debug artifacts at the schema and path level, not only by runtime convention.
- Make packet verification reject draft-shaped evidence wherever final publication or checkpoint truth is claimed.

Required tests:
- Simulator tests proving release-mode or public-lane runs reject `DraftOnly`.
- Packet verification tests that reject synthetic checkpoint ids and draft-only publication evidence.
- Anti-regression tests proving the default public lane never emits plaintext wallet-secret artifacts.

Close when:
- draft and debug runs cannot emit production-shaped checkpoint or publication evidence;
- public packet consumers can distinguish debug output from canonical final evidence by type, not by convention.

## WS-05: Privileged Wallet Paths Need Capability Types, Not Only Handler Convention

Sources:
- `AS-20260630-064-03`
- `AS-20260630-064-05`
- `AS-20260630-064-06`

Current reality:
- Sensitive wallet RPC handlers currently use `verify_session(...)` or `verify_session_no_touch(...)`, and the current route set is well covered by tests.
- The security property still depends on each new handler remembering to call the correct guard.
- `crates/z00z_wallets/src/stealth/output.rs` still exposes a raw builder whose own docs say caller-side validation is mandatory.
- wasm and native targets do not always expose the same operational capabilities, which creates policy-truth drift risk even when wasm currently fail-closes.

Required implementation:
- Introduce typed `VerifiedSession` or `PrivilegedSession` capability objects and require secret-export, restore, backup, seed, rotate, and similar privileged services to accept those types instead of raw session tokens.
- Centralize sensitive-method registration so adding a privileged RPC without a guard fails build or CI.
- Rename the raw stealth builder to an explicitly unsafe or unvalidated API shape, or reduce it to an internal/test-oriented surface. The validated builder must become the clearly canonical public path.
- Publish and enforce a capability matrix for native and wasm targets. Unsupported capabilities must return explicit typed errors and must not be described as available.

Required tests:
- Source-audit tests that enumerate all privileged RPC methods and require a capability-typed guard path.
- Negative tests for newly added privileged handlers that omit the guard.
- API tests proving public production code paths do not use the raw stealth builder as an approval decision surface.
- Cross-target tests proving unsupported wasm capabilities fail explicitly.

Close when:
- privileged wallet code cannot compile without a verified capability object;
- the raw builder is visibly noncanonical by API shape;
- target capability truth is explicit and test-enforced.

## WS-06: Wallet Mutation Truth And Restore Truth Need One Canonical Owner

Sources:
- `AS-20260630-064-02`
- `AS-20260630-064-04`
- narrowed remainder of old rotate-master-key wording

Current reality:
- Wallet asset mutation RPCs currently route through a canonical composition based on `LocalNodeSim`, `BroadcastImpl`, and tx storage, but the boundary is still expressed by helper composition rather than one sealed service contract.
- Restore already contains careful rollback logic across staged history, `.wlt`, and in-memory publish, but the flow is still a multi-step durability boundary with several failpoints.
- `wallet.key.rotate_master_key` is no longer just an in-memory placeholder; the remaining concern is policy truth and long-term lifecycle clarity, not missing persisted rewrite behavior.

Required implementation:
- Introduce one canonical mutation executor that owns local mutation submission, broadcast persistence, tx-id truth, and durable lifecycle writes for all asset mutation RPCs.
- Route split, stake, swap, and any other local mutation entrypoints only through that executor.
- Add an explicit restore journal or transaction marker spanning staged tx history, `.wlt` writes, and in-memory publish so crash recovery and retry semantics are unambiguous.
- Keep restore retry/idempotency explicit after failures between file commits and publish.
- Audit the `rotate_master_key` API wording and receipt fields so they describe the implemented persisted rotation contract honestly.

Required tests:
- Failpoint or crash-matrix tests for `history_commit`, `.wlt` commit, publish, rollback failure, and retry.
- Routing tests proving every public asset mutation RPC uses the same executor.
- Tests proving tx storage and broadcast lifecycle remain coherent under repeated or partial failures.

Close when:
- one sealed service owns mutation truth;
- restore has explicit retry semantics across crashes;
- persisted rotation behavior and method wording match each other.

## WS-07: Security Boundary Construction And Logging Must Fail Closed

Sources:
- `AS-20260627-002`
- crate `AS-20260623-002`
- legacy logging and panic cluster:
  - `AS-20260501-002`
  - `AS-20260501-004`
  - `AS-20260501-007`
  - `AS-20260501-011`
  - `AS-20260501-020`
  - `AS-20260501-023`
  - `AS-20260501-024`
  - `AS-20260501-025`
  - `AS-20260501-026`

Current reality:
- `crates/z00z_storage/src/settlement/store.rs` still keeps a panicking `SettlementStore::new()`.
- `crates/z00z_networks/rpc/src/wasm_client.rs` still logs raw RPC params and responses when the logger feature is enabled.
- Several older operator-visible logging findings were line-specific and moved, but they all point to the same real class of risk: the codebase still needs one explicit redaction and fail-closed construction policy, not ad hoc fixes.

Required implementation:
- Deprecate or internalize panicking constructors on security-relevant open/load boundaries. Public callers must use fallible constructors.
- Sweep auth, storage open/load, restore, wallet-key, and proof-verification seams for `unwrap`, `expect`, and `panic!` and replace them with typed errors where the failure crosses a trust boundary.
- Remove raw param/response logging from transport layers. Transport logging may emit method names and already-redacted summaries only.
- Add one sanitization policy for logs and user/operator-visible errors: no plaintext seed phrases, no raw secret material, no wallet ids unless explicitly classified safe, no filesystem paths unless explicitly classified safe, no raw internal storage state dumps.
- Keep a small allowlist instead of broad exceptions.

Required tests:
- Native and wasm logging tests that prove sensitive methods never print raw params or raw responses.
- Source-audit tests for panic patterns in boundary modules.
- Tests that assert high-risk wallet RPC methods use redacted summaries only.

Close when:
- no security-boundary constructor panics on open/load/config failure;
- transport logs never emit raw secret-bearing JSON;
- panic and redaction rules are enforced by CI rather than memory.

## WS-08: Placeholder Public RPC Surfaces And Stub DTOs Must Either Become Real Or Leave The Contract

Sources:
- `CF-004`
- `CF-007`
- `CF-008`
- carried placeholder parts of `CF-010`

Current reality:
- `crates/z00z_wallets/src/services/chain_service.rs` still implements chain scan state and chain tip as process-local synthetic state.
- `crates/z00z_wallets/src/app/app_kernel.rs` still describes these paths as stubs or deterministic placeholders.
- Some DTO and stub-default paths still carry placeholder proof fields such as `range_proof: None` and `merkle_proof: None`.
- The old `V2 memo unsupported` claim was not reproduced as a current trusted-path TODO in this pass, so it should not remain an active generic attack-surface ticket without a fresh feature spec.

Required implementation:
- Either implement durable or remote-backed chain scan and chain tip behavior, or remove or rename these RPCs so they are not production-looking wallet truths.
- Remove verification-free placeholder proof fields from production DTOs unless they are explicitly compatibility wrappers with a documented unsupported state.
- Keep stub-only defaults out of production-facing types.
- If any unsupported memo or placeholder feature is still a product requirement, move it into a dedicated implementation spec instead of leaving it as an ambiguous trusted-path placeholder.

Required tests:
- RPC truth tests proving scan status and chain tip are not synthetic while claiming to be live network truth.
- Schema tests proving placeholder-only proof fields do not appear on production DTOs by default.
- Tests that production docs and route wiring do not describe placeholder paths as completed functionality.

Close when:
- public RPCs are either real or explicitly non-production;
- production DTOs do not advertise placeholder proof semantics as if they were finalized behavior.

## WS-09: Final Source Sweep For Narrowed Historical Leftovers

Sources:
- `AS-20260623-001` from `060-attack-surface-db.jsonl`
- `AS-20260501-021`
- `AS-20260501-022`
- `AS-20260501-028`

Current reality:
- The old rights-config bootstrap shadow surface is mostly closed; the remaining need is a stale-reference sweep so `assets_config.yaml` is not misread as the canonical rights bootstrap path.
- The old placeholder-password finding did not reproduce on the current wallet lifecycle path.
- The old path-leak `eprintln!` and originally cited `.expect()` key-path finding do not survive as live exact claims.

Required implementation:
- Sweep docs, examples, and readmes for stale references to the old compatibility rights bootstrap path.
- Keep the broader WS-07 panic and logging sweeps, but do not keep the exact stale claims as standalone backlog items.
- Remove any remaining planning text that still describes the old claims as live.

Close when:
- no stale human-readable artifact in the repo re-promotes those narrowed claims as current truth.

## Seal-Only Items That Must Keep Regression Coverage

These are not current implementation blockers, but they remain important boundaries and must stay under explicit test coverage:

- Claim-source continuity. `claim_source_contract_for_item()` now checks persisted membership before emitting a claim contract. Keep negative tests for missing or drifted items. Keep `ClaimSourceRoot::new_settlement(...)` documented as a typed root wrapper, not proof of stored authority.
- Object quarantine roundtrip and promotion semantics. Current tests are strong; keep them.
- Object reject-code contract across storage, wallet, and rollup. Current tests are strong; keep them.
- Recovery takeover and resume ownership path. Current code is strongly fail-closed; preserve adversarial tests.
- Default simulator packet secret lane. The default public lane is currently fail-closed; keep anti-regression tests while the real open work stays in `WS-03` and `WS-04`.
- Current persisted `rotate_master_key` flow. Keep wording and receipt-truth tests honest, but do not treat the old "in-memory placeholder only" wording as a live bug.

## Historical Or Narrowed Findings

These findings must not be carried forward verbatim:

- Legacy `CF-006` wording about "still carries no nullifier semantics" is
  obsolete. Current spend verification explicitly documents a delivered
  deterministic nullifier surface. The remaining live gap is validator
  theorem-input closure in `WS-01`.
- Legacy `CF-003` wording about "`wallet.key.rotate_master_key` is only
  placeholder semantics" is obsolete as a standalone bug statement. The
  remaining live concern is narrower: canonical mutation/restore ownership and
  long-term contract truth, which stay carried in `WS-06`.
- Legacy `CF-009` invalid-owner-signature downgrade wording is closed by the
  current fail-closed receive path, which rejects invalid-signature assets
  instead of silently scrubbing them into claimed storage.
- Legacy `CF-011` `V2 memo unsupported` wording was not reproduced as a
  current trusted-path attack-surface ticket in this pass. Reopen only with
  fresh code evidence and a concrete feature contract.

## Permanent Repository-Wide Meta-Gates From The Legacy Catalog

The legacy `attack-surface-db.jsonl` repeatedly surfaced generic security classes. They should no longer live as dozens of separate tickets, but they must remain enforced as CI policy:

- Secret-bearing type hygiene. Ban unreviewed `Debug`, `Serialize`, or `Deserialize` exposure on secret wrappers and key material.
- Constant-time and equality hygiene. Ban direct `==` or ordinary `PartialEq` checks on secret material unless the code routes through an explicit constant-time helper.
- RNG hygiene. Ban non-cryptographic RNG near key generation, nonce generation, salts, proof witnesses, or similar cryptographic inputs.
- Panic hygiene. Ban `unwrap`, `expect`, and `panic!` in auth, storage open/load, restore, seed, key, and proof-verification boundaries.
- Operator-visible logging hygiene. Ban plaintext seeds, raw secret material, internal storage dumps, filesystem paths, and similar boundary leaks unless explicitly allowlisted.

Required implementation:
- Add source-audit tests or lint-style CI checks for each meta-gate.
- Keep allowlists small, path-scoped, and reviewed.
- Fail CI on new violations.

Exact legacy ids absorbed by the meta-gates above:

- Secret-bearing type hygiene:
  - `AS-20260501-001`
  - `AS-20260501-003`
  - `AS-20260501-005`
  - `AS-20260501-006`
  - `AS-20260501-008`
  - `AS-20260501-013`
  - `AS-20260501-014`
  - `AS-20260501-015`
  - `AS-20260501-016`
  - `AS-20260501-019`
- Operator-visible logging and error-boundary hygiene:
  - `AS-20260501-002`
  - `AS-20260501-004`
  - `AS-20260501-023`
  - `AS-20260501-024`
  - `AS-20260501-025`
  - `AS-20260501-026`
- Panic-at-boundary hygiene:
  - `AS-20260501-007`
  - `AS-20260501-011`
  - `AS-20260501-020`
- RNG-near-crypto hygiene:
  - `AS-20260501-009`
  - `AS-20260501-017`
- Equality-on-secret hygiene:
  - `AS-20260501-010`
  - `AS-20260501-012`
  - `AS-20260501-018`

## Active Disposition Map

Only active items remain in this map. Retired `Historical/Narrowed` findings are intentionally excluded.

| Source finding | Disposition | Carried by | Note |
| --- | --- | --- | --- |
| `AS-20260627-001` | Open | `WS-01` | theorem verifier exists but acceptance still needs mandatory typed inputs |
| `AS-20260627-002` | Open | `WS-07` | panicking settlement-store constructor remains live |
| `AS-20260630-064-01` | Open | `WS-04` | draft-only mode still reaches production-shaped evidence |
| `AS-20260630-064-02` | Open | `WS-06` | current path is canonical by composition, not by one sealed owner |
| `AS-20260630-064-03` | Open | `WS-05` | guard coverage is good, but still convention-heavy |
| `AS-20260630-064-04` | Open | `WS-06` | restore is strong but still a multi-step durability boundary |
| `AS-20260630-064-05` | Open | `WS-05` | raw builder remains callable without approval semantics |
| `AS-20260630-064-06` | Seal | `WS-05` | current wasm path fail-closes; remaining task is capability truth and explicit unsupported behavior |
| `AS-20260630-064-07` | Seal | `WS-04` | default lane is guarded; keep anti-regression tests while closing draft/debug truth gaps |
| `AS-20260630-064-08` | Seal | regression only | current quarantine/promotion tests are materially strong |
| `AS-20260630-064-09` | Seal | regression only | current reject-code alignment tests are materially strong |
| `AS-20260630-064-10` | Open | `WS-02` | raw checkpoint save lane remains too near the canonical seal path |
| `AS-20260630-064-11` | Open | `WS-01` | theorem and publication metadata must converge into one validator-owned bundle |
| `AS-20260630-064-12` | Seal | regression only | recovery takeover path is already strongly fail-closed |
| crate `AS-20260623-001` | Open | `WS-03` | test hooks still escape into normal builds |
| crate `AS-20260623-002` | Open | `WS-07` | wasm transport logger still logs raw values |
| crate `AS-20260623-003` | Open | `WS-03` | weakened KDF feature still reaches release-capable builds |
| `AS-20260623-001` from `060-attack-surface-db.jsonl` | Open | `WS-09` | code path is mostly normalized, but stale canonical-rights references still need a final doc sweep |
| `AS-20260501-027` | Open | `WS-03` | debug wallet export still exists behind a feature gate |
| `AS-20260501-029` | Open | `WS-02` | raw link persistence still belongs to the raw/canonical checkpoint boundary |
| `AS-20260501-030` | Open | `WS-02` | compatibility proof semantics still need explicit noncanonical naming |
| `AS-20260501-031` | Open | `WS-03` | plaintext secret debug lane still exists behind a feature gate |
| `AS-20260501-001/003/005/006/008/013/014/015/016/019` | Policy gate | meta-gates | generic secret-bearing type exposure class |
| `AS-20260501-002/004/023/024/025/026` | Policy gate | meta-gates + `WS-07` | generic operator-visible logging and error-boundary class |
| `AS-20260501-007/011/020` | Policy gate | meta-gates + `WS-07` | generic panic-at-boundary class |
| `AS-20260501-009/017` | Policy gate | meta-gates | generic RNG-near-crypto class |
| `AS-20260501-010/012/018` | Policy gate | meta-gates | generic equality-on-secret class |
| `CF-001` claim-source continuity | Seal | regression only | current storage API now checks persisted membership |
| `CF-002` raw/compatibility proof semantics | Open | `WS-02` | still a live naming and authority-boundary problem |
| `CF-004` public chain scan/tip placeholder | Open | `WS-08` | public RPC truth still needs real implementation or removal |
| `CF-007` stub helper coexistence | Open | `WS-08` | still part of the placeholder/stub public contract cleanup |
| `CF-008` placeholder proof fields on DTOs | Open | `WS-08` | production DTOs still need cleanup |
| `CF-010` proofless scaffold/helper surfaces | Open | `WS-02` + `WS-08` | keep only the concrete raw-lane and placeholder-DTO tasks |

## Mandatory Closure Gate Before Marking Phase 065 Closed

Do not mark the phase closed until all `Open` workstreams above are implemented and the following repository gates exist:

- targeted tests in `z00z_storage`, `z00z_wallets`, `z00z_rollup_node`, and `z00z_simulator` for every workstream listed above;
- build-policy checks proving forbidden release-feature combinations fail;
- source-audit checks enforcing the meta-gates;
- negative tests for theorem-input absence, raw checkpoint misuse, unguarded privileged RPCs, draft-only publication evidence, and transport-log leakage;
- one final pass over docs and public API comments so the repo stops describing narrowed historical claims as if they were still live.

Until those gates exist, Phase 065 should be considered reduced and clarified, but not fully closed.
