<!-- markdownlint-disable MD001 MD022 MD032 MD033 MD047 -->
# Phase 061: Wallet-Refactoring - Context

**Gathered:** 2026-06-23
**Status:** Phase complete; `061-01` through `061-10` are summary-backed
complete and the final one-level wallet tree contract is proven on the live
wallet crate
**Source:** PRD Express Path (`061-TODO.md` plus the referenced wallet docs and
live code anchors)

<domain>
## Phase Boundary

Phase 061 turns `061-TODO.md` into an execution packet for the live
`crates/z00z_wallets/src` tree. The goal is structural only: flatten the wallet
crate so Rust files live only at `src/*.rs` or `src/<domain>/*.rs`, while
preserving the current caller-visible facades and avoiding a second authority
layer.

This is not a cosmetic rename sweep. The phase touches persistence, RPC,
services, key management, receiver flows, and tests at the same time, so the
packet must isolate hub seams before leaf moves:

- preserve `adapters::rpc` while implementation files move into `src/rpc/`;
- preserve `db::redb_wallet_store` while implementation files move into
  `src/redb_store/`;
- preserve `services::WalletService` while its internal shard layout is
  flattened;
- keep shared wallet-store crypto under `src/db/` with neutral
  `wallet_store_crypto*` naming;
- keep `.wlt` labels, schema versions, KDF labels, and hash-domain strings
  unchanged.

### What Phase 061 Delivers

1. A numbered execution packet for the full `061-TODO.md` rename and
   flattening table.
2. A wave order that isolates high-risk facade rewiring from lower-risk leaf
   moves.
3. A live-tree drift contract for stale rows, duplicate candidates, and
   source-anchor-sensitive tests before bulk moves begin.
4. A final closeout lane that proves the wallet crate no longer contains
   multi-level Rust subtrees under `src/`.

### What Phase 061 Does NOT Deliver

- No `.wlt` schema or persistence-format migration.
- No wallet feature addition, RPC product expansion, or UI redesign.
- No new crate-root public modules created only for compatibility.
- No caller-visible `z00z_core` or `core::*` surface rewrite; wallet-side
  imports and crate-root re-exports stay structurally stable unless the TODO
  table says otherwise.
- No second planning root or duplicate `061` directory.
- No edits under `crates/z00z_crypto/tari/**`.

</domain>

<decisions>
## Implementation Decisions

### Authority And Scope

- **D-01:** `.planning/phases/061-Wallet-Refactoring/061-TODO.md` remains the
  canonical Phase 061 rename and flattening authority. This context and the
  numbered plans derive from it and must not replace it.
- **D-02:** The structural target is strict: Rust files may live only at
  `src/*.rs` or `src/<domain>/*.rs`. Depth greater than one directory under a
  domain is Phase 061 debt and must be removed or deliberately deferred in a
  summary-backed exception. Destination names must keep functional prefixes and
  avoid redundant repeated domain words unless a collision or a real semantic
  distinction requires them.
- **D-03:** Caller-visible facades stay stable during the move. Use thin
  `#[path = ...]` or equivalent facade wiring where needed, especially for
  `adapters::rpc`, `db::redb_wallet_store`, and `services::WalletService`.
  Structural waves must not widen into caller-visible API renames beyond the
  TODO table decisions.
- **D-04:** `src/redb_store/` is reserved for the concrete native RedB wallet
  backend only. Shared wallet persistence crypto and schema contracts stay
  under `src/db/` with neutral wallet-store naming.
- **D-05:** Rust module and file renames must not change persisted `.wlt`
  labels, schema versions, KDF labels, or domain-separated strings such as
  `z00z.crypto.redb_wallet_crypto...`.
- **D-15:** Future-only or target design wording in `061-TODO.md` and the
  referenced wallet design corpus is live mandatory Phase 061 scope for the
  current tree. Do not defer those rows behind a second backlog, placeholder
  facade, or alternate module path.

### Source Anchors And Non-Rust Artifacts

- **D-06:** Every moved Rust file must carry its local `mod`, `#[path]`,
  `include!`, `include_str!`, doc anchor, and test anchor updates in the same
  slice. Test-only Rust files must keep the `test_` prefix.
- **D-07:** Live-tree drift is part of scope. `061-TODO.md` contains stale
  assumptions around non-existent service wrapper files; the first slice must
  reconcile those assumptions against the workspace before broad tree moves
  begin.
- **D-08:** Non-Rust artifacts currently living under `src/` are real work, but
  relocation should happen in the owning wave that already touches the
  consuming Rust module. Final closeout sweeps leftovers; it does not invent a
  second config or docs authority. Canonical destinations follow the TODO:
  schemas under `crates/z00z_wallets/schemas/`, key or wallet docs under
  `crates/z00z_wallets/docs/`, and the remaining egui reference bundle under
  `crates/z00z_wallets/docs/`.
- **D-09:** Delete candidates such as `app_settings_tab_2.rs`,
  receipt/scan `storage*.rs`, and stale wrappers may be removed only after
  references are proven absent or replaced.

### Wave Isolation

- **D-10:** Do not mix `db::wallet_store_crypto*` shared-contract renames with
  the `db::redb_wallet_store` facade move. They are separate persistence seams.
- **D-11:** Do not mix `adapters::rpc` support/DTO/logging moves with
  `adapters::rpc::methods` implementation moves. The facade must stabilize
  before the largest RPC churn lands.
- **D-12:** Do not mix `services::WalletService` internal flattening with the
  RPC methods wave. Service wrapper rewiring already has dense `#[path]` and
  source-anchor risk.
- **D-13:** The first execution slice is preflight and anchor freezing, not
  mass rename throughput.
- **D-14:** Every `<verify>` block in this phase follows one mandatory order:
  bootstrap fail-fast first, then slice-specific checks plus
  `cargo test --release` when relevant, then three or more
  `/GSD-Review-Tasks-Execution` YOLO passes until two consecutive runs are
  clean, then `/z00z-git-versioning` for commits.

### the agent's Discretion

- Exact one-level destination filenames inside a numbered slice, provided they
  stay faithful to the `061-TODO.md` table and the five-word naming limit.
- Exact non-`src/` destination folders for non-Rust artifacts, provided the
  move keeps a single authority surface and the owning Rust modules update in
  the same wave.
- Whether stale `061-TODO.md` rows are reconciled directly in the TODO or in
  summary-backed drift notes, provided later slices never execute against known
  false path assumptions.

</decisions>

<canonical_refs>
## Canonical References

### Planning Authority

- `.planning/phases/061-Wallet-Refactoring/061-TODO.md` — canonical rename and
  flattening table plus constraints and verification intent
- `.planning/phases/061-Wallet-Refactoring/061-CONTEXT.md` — derived execution
  context for the numbered packet
- `.planning/ROADMAP.md` — Phase 061 registration, scope fence, and dependency
  note against Phase 060 closeout
- `.github/copilot-instructions.md` — repository-wide execution rules

### Persistence And Wallet Format Constraints

- `crates/z00z_wallets/🔐-разбор-WLT.md` — `.wlt` RedB container semantics,
  object families, and history boundary
- `crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md` — wallet authority,
  history/export separation, and backup/security model
- `crates/z00z_wallets/config/wallet_config.yaml` — embedded wallet default
  config in its canonical non-`src/` home

### Live Facade Anchors

- `crates/z00z_wallets/src/lib.rs` — crate-root module exports
- `crates/z00z_wallets/src/adapters/mod.rs` — `adapters::rpc` facade root
- `crates/z00z_wallets/src/db/mod.rs` — shared db exports and
  `redb_wallet_store` surface
- `crates/z00z_wallets/src/services/mod.rs` — `WalletService` re-export root
- `crates/z00z_wallets/src/services/wallet_service.rs` — current service facade

### Live Anchor-Sensitive Modules

- `crates/z00z_wallets/src/rpc/logging_config.rs` — embedded wallet config
  loading and relative path assumptions
- `crates/z00z_wallets/src/services/wallet_paths.rs` — embedded wallet config
  loading and runtime config merge behavior
- `crates/z00z_wallets/src/redb_store/test_redb_store.rs` — wallet guide and
  source-inspection include anchors
- `crates/z00z_wallets/docs/WALLET-GUIDE.md` — canonical wallet guide
  authority loaded by the RedB-store source-inspection tests
- `docs/tech-papers/TODO-Wallet-idea.md` — external wallet-design source text
  loaded by the RedB store source-inspection tests
- `crates/z00z_core/src/assets/assets_config.yaml` — external asset config
  source text loaded by the same RedB test module
- `crates/z00z_wallets/schemas/redb-schema.yaml` — RedB schema asset in its
  canonical non-`src/` home after the backend move
- `crates/z00z_wallets/src/domains/test_definitions.rs` — domain snapshot
  include anchor
- `crates/z00z_wallets/src/domains/test_hashing.rs` — flat hashing test anchor
- `crates/z00z_wallets/docs/domains_snapshot.txt` — canonical domain
  snapshot asset loaded by definitions tests
- `crates/z00z_wallets/src/services/test_wallet_paths_suite.rs` — flat wallet
  config-path test anchor
- `crates/z00z_wallets/src/egui_views/wallet_tab_staking.rs` — canonical
  staking-tab path after the typo closure
- `crates/z00z_wallets/docs/egui_views.tar.gz` — canonical egui reference
  bundle after the closeout cleanup
- `crates/z00z_wallets/src/security/password.rs` — Bloom filter runtime loader
- `crates/z00z_wallets/bin/gen_password_bloom.rs` — source text and Bloom
  builder path
- `crates/z00z_wallets/config/security/common-passwords.txt` — canonical
  non-`src/` password source corpus
- `crates/z00z_wallets/config/security/password_denylist.bloom` — canonical
  non-`src/` Bloom artifact
- `crates/z00z_wallets/docs/KEYS-DERIVATION.md` — canonical key derivation
  trace doc
- `crates/z00z_wallets/docs/KEYS-Bip44-UserGuide.md` — canonical BIP-44
  policy guide
- `crates/z00z_wallets/docs/KEYS-GUIDE.md` — canonical key call-graph
  comparison note
- `crates/z00z_wallets/docs/KEYS_EXPALNATION.md` — canonical key
  explanation and file-pointer guide
- `crates/z00z_wallets/docs/bip44_derivation.md` — canonical BIP-44
  derivation examples companion

### Test And Source-Anchor Surfaces

- `crates/z00z_wallets/tests/test_common/test_mod.rs`
- `crates/z00z_wallets/tests/test_common/test_range_proof_env.inc`
- `crates/z00z_wallets/tests/test_common/test_rpc_logger.inc`
- `crates/z00z_wallets/tests/test_common/test_wallet_env.inc`
- `crates/z00z_wallets/tests/test_common/test_wallet_env_lock.inc`
- `crates/z00z_wallets/src/services/test_app_service_suite.rs`
- `crates/z00z_wallets/src/services/test_wallet_service.rs`
- `crates/z00z_wallets/src/rpc/test_asset_impl.rs`

</canonical_refs>

<specifics>
## Specific Findings And Constraints

### Coverage And Hotspot Summary

- `061-TODO.md` currently carries `354` rename or delete decisions.
- The largest concentration is under `adapters/rpc/**`; the next major seam is
  `db/redb_wallet_store/**`.
- Deep Rust paths are concentrated in RPC methods, RedB store, receiver, key,
  and service shard trees.
- The phase is refactor-heavy but contract-sensitive: wallet persistence,
  embedded config loading, include-based tests, and facade re-exports all move
  with the tree.

### Live Drift Already Found

- `061-01` repoints the stale D4 service-wrapper rows to the live root seam
  files under `src/services/*.rs`; the nonexistent `src/services/wallet_service/*`
  subtree is not a real implementation target.
- `061-01` records the stale D3 aggregate row as a no-live-file drift note so
  later slices do not reintroduce a phantom `wallet_service_types.rs` lane.
- `061-01` reconciles the stealth rows to the then-live `src/stealth/zkpack/*`
  tree; `061-09` then flattens that subtree into the canonical
  `src/stealth/zkpack.rs` plus `src/stealth/test_zkpack.rs` paths, and the
  removed `facade_zkpack/*` paths remain historical drift only.
- Receipt and scan duplicate candidates need proof before deletion; at least one
  `storage_impl.rs` is not byte-identical to the canonical implementation.
- `061-10` completes the remaining non-Rust relocations: the domain snapshot
  now lives at `crates/z00z_wallets/docs/domains_snapshot.txt`, the egui
  reference bundle lives at `crates/z00z_wallets/docs/egui_views.tar.gz`, and
  no remaining Phase 061 domain or egui non-Rust artifacts stay under `src/`.

### Ordered Plan Routing

| Plan | Primary scope | Why isolated |
| --- | --- | --- |
| `061-01` | preflight, stale-row drift, source-anchor audit, delete-candidate truth | bulk moves are unsafe until the live tree and anchor sites are frozen |
| `061-02` | shared `db` flattening, `wallet_store_crypto*`, `index_codecs`, `storage_backend` | persistence naming and contract strings must stabilize before backend relocation |
| `061-03` | `db::redb_wallet_store` facade move into `src/redb_store/` | highest-risk persistence path-preservation seam |
| `061-04` | `adapters::rpc` support, logging, DTO, dispatcher wiring, wallet-config anchor move | stabilizes the RPC facade before method implementation churn |
| `061-05` | `services::WalletService` internal flattening and service anchors | isolates dense `#[path]` and `include_str!` rewiring from RPC method moves |
| `061-06` | RPC methods and `_rpc_` helper renames | largest single compile-churn cluster inside one facade |
| `061-07` | receiver, persistence, security-vault, duplicate receipt/scan cleanup | local facades and duplicate storage truth after hub seams are stable |
| `061-08` | key tree flattening, BIP/seed/receiver helpers, key docs | include-heavy subtree best isolated from tx and wallet churn |
| `061-09` | tx, claim, stealth, wallet, backup, chain | semantic-heavy leaf move after persistence, RPC, services, receiver, and key stabilize |
| `061-10` | domains, egui, final non-Rust leftovers, tree-wide closeout | closes remaining leafs and proves the final one-level tree contract |

### TODO Coverage Ledger

- **Decision basis**
  - `F1` is executed by the in-domain flattening waves `061-02/05/07/08/09/10`.
  - `F2` is executed by the protected facade moves in `061-03` and `061-04`,
    with `061-06` finishing the `src/rpc/` method subtree.
  - `R1` is owned by the final egui typo correction in `061-10`.
  - `R2` is owned by the WalletService store-name shortening work in `061-05`.
  - `R3` is split between receiver naming cleanup in `061-07` and the claim-tx
    helper split in `061-09`.
  - `R4` is owned by the neutral `wallet_store_crypto*` rename in `061-02`.
  - `R5` is owned by the RPC method-helper rename wave in `061-06`.
  - `D1` is classified in `061-01` and closed in `061-10`.
  - `D2` is classified in `061-01` and closed in `061-07`.
  - `D3` is reconciled in `061-01` and closed in `061-05`.
  - `D4` is reconciled in `061-01` and closed in `061-05`.
- **Constraints**
  - `CON-001`, `CON-002`, `CON-004`, and `CON-010` are enforced by the
    preflight inventory in `061-01`, the naming or flattening waves
    `061-04/06/08/09`, and the final tree audit in `061-10`.
  - `CON-003` is covered by the structural-only phase boundary plus the
    wave-local rename scopes in `061-02` through `061-10`; this packet does not
    authorize cosmetic or parallel-path renames beyond the TODO table.
  - `CON-005` and `CON-006` are owned by facade-preservation waves
    `061-03/04/05/06/10`, with explicit `#[path]` routing for protected seams.
  - `CON-007` is covered by `D-06` and by the test-bearing waves
    `061-01/03/04/06/08/10`.
  - `CON-008` and `CON-009` are covered by the persistence waves
    `061-02/03` and rechecked in `061-09/10`.
- **Implementation notes**
  - `IMP-001` -> `061-04`
  - `IMP-002` -> `061-03`
  - `IMP-003` -> every execution slice that moves Rust files
  - `IMP-004` -> `D-03` plus `061-04/06/09/10`
  - `IMP-005` -> `061-01`, `061-07`, and `061-10`
  - `IMP-006` -> `061-03/04/08/09/10`
  - `IMP-007` and `IMP-008` -> `061-02`
  - `IMP-009` -> `061-06`
  - `IMP-010` -> `061-09`
- **Verification ownership**
  - `VER-001` and the table-vs-tree preflight compare -> `061-01`
  - `VER-002`, `VER-003`, `VER-004`, `VER-005`, `VER-006`, and `VER-007` ->
    `061-10`
  - `VER-008` and `VER-009` -> `061-02`, then rechecked in `061-10`
  - `VER-010` -> `061-06`, then rechecked in `061-10`
  - `VER-011` -> `061-09`, then rechecked in `061-10`
- **Table-generation invariants**
  - `CHECK-001` — `497` Rust source files scanned at the 061-01 live preflight.
  - `CHECK-002` — `354` rename/remove decisions recorded in the TODO table.
  - `CHECK-003` — `0` proposed `new-path` collisions at planner baseline.
  - `CHECK-004` — `0` proposed Rust targets deeper than `src/<dir>/<file>.rs`.
  - `CHECK-005` — `0` proposed module filenames over five words.
  - `CHECK-006` — `0` proposed target conflicts with existing unlisted Rust
    files.
  - `CHECK-007` — `0` listed old paths missing from the workspace after the
    061-01 drift corrections.
  - `CHECK-008` — `0` current nested Rust files uncovered by the table after
    the 061-01 drift corrections.
  - `CHECK-009` — `0` proposed `src/rpc` filenames with redundant `_rpc_`
    qualifiers.
  - `CHECK-010` — `0` ambiguous `claim_helpers` or `claim_tx_helpers` target
    filenames.
  - `061-01` re-runs the live-tree compare and records any drift before edits
    start, and `061-10` re-audits the final tree against the same invariants.

</specifics>

<deferred>
## Deferred Ideas

- None as a second backlog. Public API redesign, `.wlt` schema migration,
  wallet history redesign, new RPC product scope, and vendor subtree edits stay
  out of Phase 061.

</deferred>

<scope_fence>
## Scope Fence

- Reuse `.planning/phases/061-Wallet-Refactoring/` only. Do not create another
  `061` directory.
- Keep work scoped to `crates/z00z_wallets/src` and directly affected local
  docs/tests/assets.
- Preserve existing public facades instead of introducing a second module or
  config authority.
- Do not edit `crates/z00z_crypto/tari/**`.

</scope_fence>

<success_criteria>
## Success Criteria

- The numbered packet covers the full `061-TODO.md` surface without executing
  against stale paths.
- No Rust file remains deeper than `src/<domain>/file.rs` when the phase closes.
- `adapters::rpc`, `db::redb_wallet_store`, and `services::WalletService`
  remain stable compatibility surfaces during the move.
- Persistence labels, schema versions, KDF labels, and wallet-domain strings
  stay unchanged.

</success_criteria>

<risk_summary>
## Risk Summary

- Facade rewiring can silently break downstream imports unless `#[path]`,
  re-exports, and include anchors move together.
- Wallet config, schema, wallet guide, snapshot, and password assets are loaded
  by real Rust code; moving them as "docs" would be false.
- Persistence crypto renames are allowed only at Rust-path level; touching the
  stored label space would create a hidden migration.
- Delete candidates can look trivial but still be latent test or module inputs.

</risk_summary>

---

*Phase: 061-Wallet-Refactoring*
*Context gathered: 2026-06-23 via PRD Express Path*
