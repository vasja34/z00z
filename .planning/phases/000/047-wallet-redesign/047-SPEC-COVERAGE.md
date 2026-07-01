# 047-SPEC-COVERAGE

This file is the explicit section-by-section crosswalk from
`047-wallet-redesign-spec.md` into the Phase 047 planning packet. It exists so
the next executor can prove that the whole specification, not only the
implementation phases near the end, has a concrete owner.

## Coverage Matrix

| Spec lines | Spec section | Required reading / decision | Planned owner |
| --- | --- | --- | --- |
| `10-29` | `## 📌 Scope` | Keep the refactor bounded to `.wlt` wallet objects, receive/scan, tx input selection, simulator Stage 13, and backup/restore while excluding storage-root rewrites and remote ownership authority. | `047-CONTEXT.md` `Phase Boundary`, `Explicit Scope`, and `Explicit Non-Goals`; all plans stay inside this boundary. |
| `30-47` | `## 🎯 Executive Recommendation` | Preserve the central design decision: Snapshot is not the long-term asset center; `.wlt` domain objects are. | `047-CONTEXT.md` `Architectural Choice`; `047-03`, `047-04`, and `047-08`. |
| `48-74` | `## 🔍 Current Code Reality` | Start from the real `.wlt`, JSONL, and Snapshot substrate instead of inventing a new storage model from scratch. | `047-CONTEXT.md` `Current Code Reality Anchors` and `Canonical References`; `047-01`, `047-03`, `047-04`, `047-06`, `047-07`. |
| `75-343` | `### 🔎 Verified Current Symbols And Signatures` | Keep live symbol and signature compatibility in scope while refactoring the authority boundary. | `047-CONTEXT.md` `Verified Current Seams`; per-plan `<interfaces>` blocks in `047-01`, `047-03`, `047-04`, `047-05`, `047-06`, `047-07`, `047-08`. |
| `344-394` | `## ⚠️ Why Snapshot-Owned Assets Are a Weak Design` | Preserve the design rationale: wrong boundary, poor granularity, poor query shape, harder atomicity, harder extension, misleading mental model. | `047-CONTEXT.md` `Architectural Rejection Reasons`; `047-04` and `047-06` cut over the authority; `047-08` removes stale wording. |
| `395-428` | `## 🧭 Gap Coverage From The Two Source Notes` | Keep Phase 046 proof obligations while refusing to treat Snapshot as the target architecture. | `047-CONTEXT.md` `Gap-Coverage Promises`; `047-08` updates the phase-local 046 spec copy and Stage 13 language. |
| `429-466` | `## ⚙️ Requirements The New Design Must Preserve` | Preserve create/open, secrets split, receive authority, spend selection, cancel/reconcile, import, restore, TOFU, compatibility surfaces, and root-helper delegation. | `047-CONTEXT.md` `Preserved Runtime Obligations`; `047-03` through `047-08`. |
| `467-491` | `### ✅ Normative Requirements` | Route `REQ-001` through `REQ-020` into concrete plan owners. | `047-CONTEXT.md` `Spec Coverage Routing`; every `047-0N-PLAN.md` frontmatter `requirements:`. |
| `492-577` | `### 🔧 Wallet Config Cutover` | Expand `wallet_config.yaml`, preserve config priority, fail closed on invalid values, and remove live hardcoded defaults. | `047-03-PLAN.md` Tasks 1-3; `047-07-PLAN.md` for backup/recovery continuation. |
| `578-662` | `## 🧪 Design Options` and `### Native .wlt Domain Object Store` | Choose the native `.wlt` domain-object option and reject hidden-db-inside-Snapshot drift. | `047-CONTEXT.md` `Locked Invariants`; `047-04-PLAN.md`. |
| `663-704` | `## ✅ Recommended .wlt Organization` | Keep the canonical `.wlt` shape with `meta`, `secrets`, `objects`, and explicit indexes. | `047-01-PLAN.md`, `047-02-PLAN.md`, `047-03-PLAN.md`, `047-04-PLAN.md`. |
| `705-742` | `### 🔑 WalletProfile` | Move non-asset Snapshot metadata into `WalletProfilePayload`; no tx history, scan cursor, or `claimed_assets` there. | `047-03-PLAN.md` Task 1. |
| `743-901` | `### 🔑 OwnedAsset` | Add full owned-asset payload shape, invariants, confidentiality rules, and status/source/reference fields. | `047-04-PLAN.md` Tasks 1-3; consumed again by `047-05`, `047-06`, `047-07`. |
| `902-927` | `### 🔑 ScanState` | Keep scan-state separate and wallet-side, with asset inserts plus cursor updates coupled. | `047-05-PLAN.md` Tasks 1-2. |
| `928-998` | `### 🔑 WalletTx And WalletTxEvent` | Keep tx records semantically aligned with owned-asset mutations even while JSONL remains a sidecar plane. | `047-06-PLAN.md` and `047-07-PLAN.md`. |
| `999-1009` | `### 🔑 Keys And Secrets` | Keep secrets in the `secrets` table only; do not overclaim durable seed rotation. | `047-03-PLAN.md`, `047-04-PLAN.md`, `047-07-PLAN.md`. |
| `1012-1026` | `### Flow 1: Wallet Create` | Create wallets through secrets + profile + singleton-object initialization, not Snapshot-as-authority. | `047-03-PLAN.md` Tasks 1-2. |
| `1027-1040` | `### Flow 2: Wallet Open` | Open wallets by loading profile, keys, scan state, and object-backed caches. | `047-03-PLAN.md` Task 1; verified again in `047-04` and `047-08`. |
| `1041-1062` | `### Flow 3: Scan Detects A New Owned Asset` | Persist scan hits as owned assets, keep replay idempotent, and fail closed on conflicting duplicates. | `047-05-PLAN.md` Tasks 1-3. |
| `1063-1080` | `### Flow 4: Alice Builds A Transaction To Bob` | Build from spendable owned assets, reserve them in wallet state, and couple tx status with reservation state. | `047-06-PLAN.md` Task 1. |
| `1081-1092` | `### Flow 5: Alice Cancels Pending Transaction` | Release only assets reserved by that `tx_id`. | `047-06-PLAN.md` Task 2. |
| `1093-1114` | `### Flow 6: Broadcast And Reconcile` | Confirm spent inputs, insert owned outputs/change, and keep asset/tx mutations fail-closed. | `047-06-PLAN.md` Task 2. |
| `1115-1129` | `### Flow 7: Bob Receives Or Imports A Transaction` | Import and receiver-side ownership detection must write owned-asset state and stay on one lifecycle lane. | `047-06-PLAN.md` Task 2; Stage 13 proof in `047-08`. |
| `1130-1163` | `### Flow 8: Backup And Restore` | Restore profile, owned assets, scan state, and tx history via staged promotion with no partial mutation. | `047-07-PLAN.md` Tasks 1-3. |
| `1164-1235` | `## 🧷 Index Strategy` | Add explicit owned-asset index vocabulary, keep indexes derived, and use canonical key/value formats. | `047-01-PLAN.md` Task 1; `047-02-PLAN.md` Tasks 1-3; `047-04-PLAN.md`. |
| `1236-1290` | `## 🧱 Object Kinds And Payload Versions` | Lock new object ids and payload versions in Rust + YAML + debug in one patch. | `047-01-PLAN.md` Tasks 1-2. |
| `1291-1328` | `### 🔧 Required Low-Level Store Helpers` | Add `write_object_by_id(...)` and `read_objects_by_index(...)` as production APIs. | `047-02-PLAN.md` Tasks 1-3. |
| `1329-1449` | `## 🧩 Proposed Interfaces` | Keep callers behind `WalletAssetStore` instead of object-table details. | `047-04-PLAN.md`; supported by `047-02-PLAN.md`. |
| `1450-1491` | `## 🧨 What Should Change In The Existing Phase 046 Spec` | Separate Phase 046 proof truth from Phase 047 target architecture and add Decision 9. | `047-08-PLAN.md` Tasks 1 and 3. |
| `1492-1495` | `## 🛠️ Implementation Plan And Migration Sequence` intro | Preserve the spec’s exact eight-wave cutover order. | `047-CONTEXT.md` `Plan Map` and `Execution Order`; `047-01-PLAN.md` through `047-08-PLAN.md`. |
| `1496-1526` | `### Phase 1: Schema And Payload Groundwork` | Land ids, versions, schema, and debug groundwork before behavior changes. | `047-01-PLAN.md`. |
| `1527-1553` | `### Phase 2: Low-Level Object Upsert And Index API` | Land in-place rewrites and production index/query helpers before owned-asset state exists. | `047-02-PLAN.md`. |
| `1554-1583` | `### Phase 3: Wallet Profile Replacement For Snapshot Metadata` | Replace Snapshot metadata with `WalletProfilePayload`. | `047-03-PLAN.md`. |
| `1584-1624` | `### Phase 4: Owned Asset Store` | Make `OwnedAssetPayload` the live asset authority. | `047-04-PLAN.md`. |
| `1625-1649` | `### Phase 5: Receive And Scan Integration` | Persist receive hits as owned assets and couple them to scan-state progression. | `047-05-PLAN.md`. |
| `1650-1685` | `### Phase 6: Transaction Build, Reservation, Cancel, And Reconcile` | Move the live tx lifecycle to owned-asset authority. | `047-06-PLAN.md`. |
| `1686-1717` | `### Phase 7: Backup, Restore, And Export` | Preserve full wallet-state recovery without Snapshot-owned assets. | `047-07-PLAN.md`. |
| `1718-1744` | `### Phase 8: Simulator And Documentation Cutover` | Rewrite Stage 13 and docs to the new storage truth. | `047-08-PLAN.md`. |
| `1745-1778` | `### 🧭 Mandatory Runtime And Simulator Touchpoints` | Cover the exact runtime files, adapters, and Stage 13 files named by the spec. | `047-CONTEXT.md` `Canonical References`; `047-03` through `047-08` file clusters. |
| `1779-1829` | `## 🧪 Validation Plan` | Preserve unit/RPC/simulator validation obligations, including YAML, reopen, tamper, and replay checks. | `047-CONTEXT.md` `Validation Obligations`; `047-03` through `047-08` test tasks. |
| `1830-1852` | `### 🔄 Existing Test Migration Is Mandatory` | Upgrade existing green suites; new tests alone are insufficient. | `047-CONTEXT.md` `Existing Test Migration Rules` and `Test Artifacts`; `047-08-PLAN.md` Task 2. |
| `1853-1868` | `## ✅ Acceptance Criteria` | Route every acceptance criterion to a concrete plan owner. | `047-CONTEXT.md` `Acceptance Coverage Notes`; `047-03`, `047-04`, `047-05`, `047-06`, `047-07`, `047-08`. |
| `1869-1894` | `## 🧪 Verification Gates` | Keep the exact verify order in every auto task. | `047-CONTEXT.md` `Cross-Cutting Rules`; every `047-0N-PLAN.md` `<verify>` block. |
| `1895-1905` | `## 🚫 What Not To Do` | Preserve all prohibited shapes: simulator-only tables, remote ownership authority, secret duplication, JSONL-only reservations, custom root math, stale `wallet.asset.send_asset` claims, and live Snapshot semantics. | `047-CONTEXT.md` `Explicit Non-Goals` and `Drift Bars`; `047-06-PLAN.md` and `047-08-PLAN.md`. |
| `1906-1934` | `## 📌 Final Specification Position` | End the phase with one final truth: wallet assets are first-class encrypted `.wlt` objects, not a rewritten full Snapshot vector. | `047-CONTEXT.md` `Locked Invariants` and `Architectural Choice`; `047-04`, `047-06`, `047-08`. |

## Completeness Notes

- The eight implementation phases from the spec are preserved 1:1 as
  `047-01-PLAN.md` through `047-08-PLAN.md`.
- `REQ-001` through `REQ-020` are routed in `047-CONTEXT.md` and in the
  numbered plan frontmatter.
- `AC-001` through `AC-013` are routed in `047-CONTEXT.md` and in the owning
  numbered plans.
- The mandatory existing-test migration block is preserved in
  `047-CONTEXT.md` and `047-08-PLAN.md`.
- The verification-gate order is duplicated into every `<task type="auto">`
  verify section, not only into one shared note.
