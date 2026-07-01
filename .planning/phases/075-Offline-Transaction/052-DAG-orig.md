# 050 DAG And Offline Transaction Spec

Status: implementation-oriented synthesis

Date: 2026-04-08

## 🎯 Purpose

This document extracts the implementation-relevant ideas from the historical ECC
planning corpus around `Offline DAG & Transaction Packages` and rewrites them
 against the current Z00Z codebase.

It is not a copy of the older planning text. It is a repo-aligned synthesis
that separates four things explicitly:

1. the live contracts already implemented today,
2. the future DAG ideas that still make sense,
3. the stale signatures and concept drift that must not come back unchanged,
4. the recommended implementation direction if Phase 050 later grows beyond the
   current single-package flow.

When this document conflicts with older `Z00Z-ECC-*` notes, the current codebase
and the current Phase 050 authoritative spec win.

## 📚 Source Corpus

This synthesis was derived from the following repository artifacts:

- `.planning/temp/Z00Z-ECC-IDEAS.md`
- `.planning/temp/Z00Z-ECC-SPEC_part1.md`
- `.planning/temp/Z00Z-ECC-SPEC_part2.md`
- `.planning/phases/050-offline-tx/050-Offline-Tx-Spec.md`
- `.planning/phases/050-offline-tx/050-TODO.md`
- wallet, RPC, simulator, and storage code that already define the live package,
  verification, and checkpoint handoff surfaces.

## 🔑 Canonical Concepts

Use the following vocabulary going forward. Historical names that do not map to
these concepts directly are drift and must be translated before reuse.

### ✅ Portable Transaction Package

The current canonical offline-capable transaction node is:

- `TxPackage`
- carrying `TxWire`
- digested by `build_tx_package_digest`
- locally verified by `TxVerifierImpl`

This is the only live package contract for regular offline-capable transaction
exchange in the repository today.

### ✅ Receiver Routing Surface

The current receiver-routing surface is split into two live contracts:

- `ReceiverCard` for direct signed routing input,
- `ReceiverCardRecordV1` for the canonical published receiver-card record.

The historical idea of embedding a separate `receiver_view` object inside each
portable package does not match the current implementation surface.

### ✅ Public Spend Contract

The current public spend contract is:

- `SpendProofWire`
- `SpendAuthWire`
- `build_public_spend_contract(...)`
- `verify_tx_public_spend_contract(...)`

This is already a real accepted-path public verifier boundary. It is still
narrower than a fully closed global spend theorem because the current public
statement does not yet carry full nullifier semantics.

### ✅ Output Construction Contract

The sender-side confidential output pipeline is already implemented through:

- `create_output_bundle(...)`
- `bind_output_wire(...)`
- `decode_output_pack(...)`
- `verify_self_decrypt(...)`

Any DAG extension must reuse this output contract instead of introducing a new
parallel output format.

### ✅ Checkpoint Handoff Boundary

The current checkpoint-side truth is not a standalone offline DAG backend.
It is a package-coupled continuity boundary built around:

- `CheckpointPackageProofVerifier`
- `verify_stage7_handoff(...)`
- package revalidation against the original stage-4 package contract
- exact proof, input-ref, and bridged-output continuity checks

The repository explicitly denies stronger wording than this. Checkpoint
acceptance is currently package-coupled continuity, not standalone authoritative
publish-proof closure.

## ⚙️ Live Implementation Baseline

The following capabilities already exist and should be treated as the starting
point for any future Offline DAG work.

### ✅ What Is Already Implemented

1. `TxPackage` is a real portable package format with canonical digest framing.
2. `TxInputWire` is reference-only. Inputs are identified by canonical
   `asset_id_hex` plus `serial_id`; full global membership is intentionally kept
   outside the local tx package.
3. `TxOutputWire` already carries the confidential output bridge surface through
   `AssetPkgWire` and the bound stealth fields.
4. `wallet.tx.verify_transaction_package` already provides a delayed-connectivity
   verification path that:
   - parses and verifies the package,
   - scans owned outputs only after local validity succeeds,
   - reports `import_ready` only when local validity and accepted status both
     hold.
5. `is_import_ready(status)` already centralizes the current import gate.
6. The simulator already contains a current-stack checkpoint handoff flow that
   revalidates the package contract before checkpoint draft creation.

### ⚠️ What Is Only Partially Implemented

1. The current public spend contract is live, but it still leaves nullifier
   semantics open at the public verifier boundary.
2. Checkpoint apply already closes several tamper and replay classes, but only
   on the accepted package-coupled current-stack path.
3. The storage and simulator layers already use proof bytes as verifier-bound
   compatibility payload for checkpoint promotion, but that does not make them a
   standalone recursive or publish-proof backend.

### 🚫 What Is Not Implemented Today

1. A separate `BundlePackage_v1` or any live bundle DAG container.
2. A generic ancestor resolver or topological-sort bundle validator.
3. An `offline_chain.rs`, `dag_storage.rs`, `offline_service.rs`, or live
   `dag_validator.rs` module.
4. A second offline import pipeline distinct from the current verify and import
   surfaces.
5. Hardware-backed reservation state, TEE vouchers, slash artifacts, or redeem
   economics.

## 🧭 Extracted Future DAG Ideas Worth Preserving

The historical documents contain several ideas that remain useful, but only if
they are translated onto the current contracts.

### 📦 Idea 1: Treat Each `TxPackage` As A DAG Node

The most reusable idea is not the historical type names. It is the graph model:

- a child package depends on a parent package when it spends an output that the
  parent package creates,
- a publishable set therefore needs ancestor closure,
- a submitter may need to transport more than one package when the child spends
  an output that never reached chain state yet.

This idea survives intact, but it must reuse current `TxPackage` nodes instead
of inventing a second transaction payload family.

### 📦 Idea 2: Minimal Ancestor Closure

The historical DAG notes correctly identify the key publication rule:

- if an input does not exist in the anchored base state,
- the publisher must provide the ancestor package that created that input,
- and must recursively include all required parents until every unresolved input
  becomes either state-anchored or ancestor-provided.

This remains the cleanest conceptual rule for any later offline bundle design.

### 📦 Idea 3: Topological Apply In A Working Window

Another valid preserved idea is aggregator-side working-window execution:

- build a dependency graph,
- reject cycles,
- topologically order accepted packages,
- treat outputs created earlier in the ordered set as available inputs for later
  packages,
- emit one resulting state transition artifact.

This idea is still good, but it must be layered on top of the current package
and checkpoint contracts rather than bypassing them.

### 📦 Idea 4: Conflict Detection Remains Necessary

The historical branch-conflict examples remain valid:

- if two offline children spend the same unresolved parent output,
- publication can accept at most one branch,
- conflict policy must be explicit rather than accidental.

The useful preserved design question is therefore not whether conflicts exist.
It is whether a future bundle layer should use atomic rejection or best-effort
subset application.

### 📦 Idea 5: Privacy Boundary Must Stay Honest

The old notes are correct about one important privacy distinction:

- state unlinkability is not the same thing as transport privacy,
- if a portable package or future bundle leaks, routing and output linkage can
  leak outside the state tree.

That is still true in the current repository and should remain explicit in every
future offline transport or bundle design.

### 📦 Idea 6: Publication Rights Must Follow Package Possession

One operational rule in the historical notes is worth preserving explicitly:

- a later holder of the child package must be able to publish the chosen branch,
- publication must not require the original intermediary to remain online,
- the publishable artifact therefore has to carry the selected branch plus its
  minimal ancestor closure.

This is the core handoff property that makes offline forwarding useful instead
of merely locally composable.

### 📦 Idea 7: Minimal Branch Packaging Must Exclude Unrelated Siblings

The old DAG examples also preserve an important minimization rule:

- include the chosen branch,
- include the transitive ancestors needed to resolve its unresolved inputs,
- exclude unrelated sibling transactions and unrelated change paths that are not
  needed to publish that branch.

This is stronger than merely saying "compute ancestor closure". It means future
bundle assembly must avoid over-inclusion and must not drag unrelated offline
history into the submitted artifact.

### 📦 Idea 8: Bundle-Level Operational Invariants Still Matter

The historical `bundle_package_v1` sketch contains several operational rules
that remain useful even though the concrete type name is drift:

- all bundled package nodes should share one anchored base root,
- deduplication should key off the canonical package digest,
- dependencies should express which parent-created output a child spends,
- aggregator processing should run in a working window that makes earlier
  package outputs available to later package inputs,
- the result of that window should feed the checkpoint builder through ordered
  operations and resulting deltas rather than through a proof-only shortcut.

These are still valuable design constraints for any future wrapper around live
`TxPackage` nodes.

### 📦 Idea 9: Transport Metadata Must Stay Optional And Non-Consensus

The historical notes mix together two different concerns:

- consensus-critical package contents,
- transport-only helpers such as outer encryption, holder submit hints,
  deadlines, or receiver quick-check fields.

The important preserved idea is not the exact historical field layout. It is the
separation rule:

- transport privacy and handoff metadata may exist,
- but they must remain optional,
- and they must not change canonical digest or local transaction validity.

Any future transport envelope must therefore stay outside the canonical live
package digest path.

## 🚨 Concept Drift And Stale Signatures

The historical ECC planning texts use names and fields that no longer match the
live repository. These ideas should not be reintroduced verbatim.

### ❌ Drift 1: `TxPackage_v1` As A Separate Future Transaction Format

Historical drift:

- `TxPackage_v1`
- `TxBody_v1`
- `receiver_view`
- package-local fields that do not exist in the current `TxPackage`

Canonical return:

- keep `TxPackage` and `TxWire` as the regular package contract,
- add any later DAG wrapper around them instead of replacing them.

### ❌ Drift 2: `BundlePackage_v1` As If It Already Exists

Historical drift:

- `BundlePackage_v1`
- `DependencyEdge`
- explicit `submission_policy` and `deps` structs as if already wired into the
  repo.

Canonical return:

- treat bundle support as future orchestration around existing `TxPackage`
  nodes,
- do not document a bundle container as implemented until real Rust types,
  verifiers, and persistence paths exist.

### ❌ Drift 3: Input Witnesses And Nullifiers Inside Portable Package Inputs

Historical drift:

- input specs that inline `nullifier` and `witness` into the portable package.

Canonical return:

- keep `TxInputWire` reference-only,
- keep global membership and spent-state logic in the checkpoint or pre-state
  resolution path,
- keep the public-contract caveat explicit that full nullifier semantics are not
  yet closed in the current public spend contract.

### ❌ Drift 4: Receiver View Embedded In Package

Historical drift:

- package-owned `receiver_view.cards` and output indexes tied to that embedded
  array.

Canonical return:

- routing is already modeled by `ReceiverCard` and `ReceiverCardRecordV1`,
- package verification and owned-output scanning currently operate without an
  embedded `receiver_view` layer,
- any future transport hint layer must justify itself separately and not replace
  the current routing contracts.

### ❌ Drift 5: Standalone Offline DAG Modules

Historical drift names:

- `offline_chain.rs`
- `dag_storage.rs`
- `offline_service.rs`
- `dag_validator.rs`

Canonical return:

- extend existing wallet, RPC, simulator, and checkpoint seams first,
- only introduce new modules when the bundle semantics are real enough that the
  existing authority surface cannot carry them cleanly.

### ❌ Drift 6: Stronger Checkpoint Claims Than The Code Delivers

Historical temptation:

- describe checkpoint artifacts as an authoritative standalone proof backend,
- describe compatibility payload bytes as if they were full publication proof.

Canonical return:

- keep checkpoint wording package-coupled,
- preserve the explicit repository boundary that integrity checks are real but
  authoritative publish-proof closure does not yet exist.

## 🏗️ Recommended Implementation Direction

If Phase 050 later grows into real Offline DAG support, the direction below is
the repo-compatible path.

### 1. Reuse `TxPackage` As The Only Transaction Node Format

Do not build a second regular transaction payload.

Future DAG work should treat each existing `TxPackage` as a graph node and add a
wrapper only for:

- node ordering,
- ancestor inclusion,
- deduplication metadata,
- submission policy.

### 2. Keep One Verification Ladder

Future DAG admission should still flow through the current ladder:

1. parse package,
2. run `TxVerifierImpl`,
3. require `verify_tx_public_spend_contract(...)` whenever spend containers are
   present,
4. only after package-local acceptance use any ancestor or window logic.

This avoids creating a special offline verifier that drifts from the regular
package path.

### 3. Add Ancestor Resolution As A Wrapper Concern

Ancestor logic should be an orchestration layer around existing packages, not a
replacement for them.

Useful future interfaces would therefore look like:

- compute required ancestors for one `TxPackage`,
- topologically order a set of accepted packages,
- resolve whether an input is state-anchored or ancestor-provided,
- reject unresolved inputs or cycles before checkpoint build,
- assemble only the selected branch and its minimal required ancestors.

### 3.1 Preserve Package-Possession Publishability

Future DAG design should keep the historical handoff guarantee explicit:

- a downstream holder with the child package plus required ancestors should be
  able to publish,
- the design should not assume the original sender or intermediary is still
  reachable,
- and publication authority should follow artifact possession rather than live
  session continuity.

This is one of the few offline-DAG properties that is directly product-shaping
and should be written down before code starts.

### 3.2 Keep Bundle Invariants Explicit

If a future wrapper is introduced around multiple `TxPackage` nodes, document
the invariants directly instead of leaving them implicit in code:

1. all nodes in the bundle share one base anchor,
2. deduplication uses canonical package digest,
3. dependencies identify which parent-created output is consumed,
4. working-window apply produces ordered operations and resulting deltas for the
   checkpoint builder,
5. transport helpers stay outside canonical digest semantics.

### 4. Keep Checkpoint Promotion Honest

Any future DAG bundle apply path must preserve the current checkpoint honesty
rules:

- package revalidation stays mandatory,
- proof or exec material alone stays insufficient,
- stage-11-style continuity checks remain package-coupled until a stronger
  backend truly exists.

### 5. Preserve Verify Versus Import Separation

The current repository already distinguishes:

- local package validity,
- wallet-owned output discovery,
- import readiness.

Any DAG extension must keep those as separate surfaces. A future bundle path
must not turn verification or scanning into implicit import authority.

## 📋 Implementation Checklist Derived From The Historical DAG Ideas

The following backlog themes are relevant if the repository later promotes DAG
support from planning to code:

1. Define a bundle wrapper around existing `TxPackage` nodes only.
2. Define canonical ancestor closure rules against current `TxInputWire`
   semantics.
3. Define minimal-branch packaging rules so unrelated sibling or change paths do
  not get dragged into publication artifacts.
4. Decide whether package possession alone is sufficient submit authority for a
  chosen branch and document the exact holder-submit rule.
5. Decide and document conflict policy: atomic bundle versus best-effort.
6. Implement cycle rejection, topological ordering, and digest-based
  deduplication.
7. Reuse current package verification before any window apply step.
8. Reuse current checkpoint handoff semantics rather than inventing proof-only
   shortcut lanes.
9. Specify how working-window apply yields ordered operations and resulting
  deltas for checkpoint construction.
10. Document transport privacy and optional transport metadata separately from
   state unlinkability and canonical digest semantics.
11. Keep stale signatures and stronger-than-code wording out of new APIs.

## 🔎 Verification Summary Merged Into This Spec

The verification conclusions that were previously tracked in the standalone
Phase 050 audit artifact are now merged into this document so the DAG synthesis
and its audit trail stay in one place.

### ✅ Audit Verdict

The historical `Offline DAG & Transaction Packages` source corpus is covered in
this document at the level of implementation-relevant ideas.

No critical DAG idea from the reviewed historical corpus remains missing after
the verification pass.

No hidden semantic contradiction was found between this document and
`050-Offline-Tx-Spec.md`.

The role split remains intentional and correct:

- `050-Offline-Tx-Spec.md` is the authoritative current-state specification.
- this document is the future-facing DAG synthesis constrained by current
  repository contracts.

### 📚 Coverage Map

| ID | Verified DAG topic | Covered in this document | Relation to `050-Offline-Tx-Spec.md` |
| --- | --- | --- | --- |
| S1 | DAG node model and publish graph closure | `Idea 1`, `Idea 2` | Current-state spec intentionally excludes DAG settlement as live behavior |
| S2 | Minimal ancestor closure for unresolved inputs | `Idea 2`, `2. Keep One Verification Ladder`, `3. Add Ancestor Resolution As A Wrapper Concern` | Current-state spec correctly says local verification alone does not prove ancestor resolution |
| S3 | Topological apply in a working window | `Idea 3`, `3.2 Keep Bundle Invariants Explicit` | Current-state spec correctly treats this as future-only |
| S4 | Explicit conflict policy for competing branches | `Idea 4`, implementation checklist item 5 | Current-state spec correctly warns against partial DAG shipping without full validator semantics |
| S5 | Privacy honesty when package or bundle bytes leak | `Idea 5` | Current-state spec already states that `TxPackage` remains sensitive material |
| S6 | Publication rights follow package possession | `Idea 6`, `3.1 Preserve Package-Possession Publishability` | Current-state spec correctly does not claim this as current runtime behavior |
| S7 | Minimal branch packaging excludes unrelated siblings | `Idea 7`, implementation checklist item 3 | Current-state spec correctly omits this because multi-package branch publication is not live |
| S8 | Bundle invariants: shared anchor, dedupe, dependency semantics, working-window deltas | `Idea 8`, `3.2 Keep Bundle Invariants Explicit`, implementation checklist items 6 and 9 | Current-state spec keeps checkpoint wording honest and does not overclaim live bundle semantics |
| S9 | Transport metadata stays optional and non-consensus | `Idea 9`, `3.2 Keep Bundle Invariants Explicit` | Current-state spec already protects digest truth from transport-only metadata |

### 🛑 Verified Exclusions And Drift Guardrails

The verification pass also confirmed that the following historical shapes are
correctly treated as drift and must remain excluded unless real code lands:

- embedded `receiver_view` package structure,
- `TxPackage_v1` as a second regular package type,
- `BundlePackage_v1` as if already implemented,
- inline `nullifier` or `witness` payloads inside portable package inputs,
- standalone offline DAG modules such as `offline_chain.rs`,
  `dag_storage.rs`, `offline_service.rs`, or `dag_validator.rs`.

### 📌 Maintenance Rule

If future DAG code lands, update the two Phase 050 specs in lockstep while
preserving the role split:

1. update `050-Offline-Tx-Spec.md` only when the repository implements new
  current-state behavior,
2. update this document when future-facing DAG design constraints are refined,
3. re-run a dedicated audit when bundle types, ancestor logic, checkpoint apply
  semantics, or transport metadata become real code.

## 🛑 Non-Goals For This Document

This document does not claim that the repository already implements:

- full offline cash finality,
- standalone bundle settlement,
- final recursive checkpoint proofs,
- TEE-backed reserve logic,
- slash-backed recovery economics,
- actor-independent authoritative publication proofs.

Those remain future concepts unless code, tests, and persistence paths land.

## ⭐ Bottom Line

The core idea worth preserving from the old ECC notes is simple:

- offline-capable transfer can eventually grow from a single portable package
  flow into a graph of dependent packages,
- but the graph must be expressed on top of the current `TxPackage`,
  `ReceiverCard`, output-flow, spend-verification, and package-coupled
  checkpoint surfaces,
- not by reviving obsolete type names, module names, or stronger claims than the
  repository currently proves.

In short: preserve the DAG idea, preserve ancestor closure, preserve topo apply,
preserve privacy honesty, but return all of them to the current canonical
contracts before implementation starts.
