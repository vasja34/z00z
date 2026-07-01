# Phase 042: Refactor Wallets - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

## Phase Boundary

Phase 042 is a structural refactor of `crates/z00z_wallets/src/core`, `crates/z00z_wallets/src/db`, and `crates/z00z_wallets/src/services`.

In-scope:

- Folder and module decomposition into cohesive subdomains.
- `mod.rs` rewiring and compatibility re-exports during migration.
- Unit test relocation and naming normalization (`test_*`).
- Integration-path stability for existing callers.

Out-of-scope:

- New protocol behavior.
- Cryptographic primitive changes.
- Semantics changes in wallet, proof, or storage flows.
- Vendor code edits.

This context is implementation-authoritative for planning and execution in Phase 042.

## Implementation Decisions

### Locked Scope And Source Authority

- **D-01:** The three canonical source specs are authoritative and must be executed without scope reduction:
  - `042-core-refactore-spec.md`
  - `042-db-refactore-spec.md`
  - `042-services-refactore-spec.md`
- **D-02:** The refactor is structural only; no behavior-level changes are allowed unless required to preserve compile/runtime compatibility during path rewiring.
- **D-03:** Every existing file in each targeted tree must have an explicit placement decision (`keep` or `move-<domain>`) per the source ledgers.
- **D-16:** All rows from each source spec's exhaustive file placement ledger are mandatory execution scope. No row may be dropped, merged away, or deferred inside Phase 042 without an explicit blocker entry.

### Mandatory Transfer From Core Spec

- **D-04:** Core refactor must preserve and apply all mandatory elements from `042-core-refactore-spec.md`:
  - Scope target: `crates/z00z_wallets/src/core`.
  - Objectives: reduce cross-domain coupling, make ownership boundaries explicit, split mixed-responsibility folders.
  - Test policy: colocated unit tests, `test_*` naming, no nested unit-test folders under `core/*`, integration/E2E under `crates/z00z_wallets/tests`.
  - Decision method: declaration signals + intra-core dependency affinity + `mod.rs` constraints; placement is content-driven, not filename-driven.
  - Target tree is mandatory as the migration destination contract.
  - Pros/cons taxonomy (`keep`, `move-<domain>`) is mandatory for each file placement decision.
  - Exhaustive placement ledger is mandatory and non-optional.

### Mandatory Transfer From DB Spec

- **D-05:** DB refactor must preserve and apply all mandatory elements from `042-db-refactore-spec.md`:
  - Scope target: `crates/z00z_wallets/src/db`.
  - Objectives: separate backend/store internals, codecs, crypto, and IO boundaries.
  - Test policy: colocated unit tests, `test_*` naming, convergence into `db/redb/tests`, integration/E2E under `crates/z00z_wallets/tests`.
  - Decision method: declaration signals + intra-db dependency affinity + `db/mod.rs` and Redb composition constraints.
  - Target tree is mandatory (`backends`, `codecs`, `crypto`, `io`, `redb/{migrations,schema,store,tests}`).
  - Pros/cons taxonomy and exhaustive file placement ledger are mandatory.

### Mandatory Transfer From Services Spec

- **D-06:** Services refactor must preserve and apply all mandatory elements from `042-services-refactore-spec.md`:
  - Scope target: `crates/z00z_wallets/src/services`.
  - Objectives: isolate app orchestration, runtime adapters, seed lane, and wallet subdomains.
  - Test policy: colocated unit tests, `test_*` naming, convergence into `services/app/tests` and `services/wallet/tests`, integration/E2E under `crates/z00z_wallets/tests`.
  - Decision method: declaration signals + intra-services dependency affinity + `services/mod.rs` and wallet-service composition constraints.
  - Target tree is mandatory (`app`, `runtime`, `seed`, `wallet/{actions,paths,session,store,tests,types}`).
  - Pros/cons taxonomy and exhaustive file placement ledger are mandatory.

### Execution Order, Dependency Safety, And Parallelization Rules

- **D-07:** Execution order is strict to avoid import-path breakage:
  1. Build compatibility facades (`mod.rs`/re-export seams) for destination domains.
  2. Move low-risk leaf files per ledger.
  3. Rewire internal imports to destination paths.
  4. Move tests and update test module references.
  5. Remove temporary compatibility shims only after green gates.
- **D-08:** Parallelization is allowed only for disjoint file ownership. Any shared `mod.rs`, shared re-export surface, or shared trait/type boundary forces serial execution.
- **D-09:** No planning/execution step may declare completion without passing ordered validation gates.

### API, Trait, And Integration Boundaries

- **D-10:** Existing public facades in `core/mod.rs`, `db/mod.rs`, and `services/mod.rs` must remain stable during migration unless an explicitly documented compatibility shim exists.
- **D-11:** Any unresolved path target from the specs must be labeled `proposed` until verified in-tree; it must not be presented as landed fact.
- **D-12:** Existing crate boundaries remain unchanged (`z00z_wallets` internal refactor only). No new duplicate abstraction lanes are allowed when existing structures can be extended.

### Cryptographic And Security Invariants (From Mandatory Review)

- **D-13:** Refactor must preserve domain separation and transcript/path binding semantics by preventing accidental import drift between `core` crypto-adjacent modules and service adapters.
- **D-14:** Secret-bearing paths (key/session/store/backup flows) must preserve zeroization and fail-closed error behavior; structural moves must not weaken cleanup/error propagation.
- **D-15:** No debug/test helper path may become reachable from production public facades after module rewiring.

### the agent's Discretion

- Task slicing into plan waves and per-wave file batches.
- Temporary module aliases naming, as long as they are removed before closeout.
- Exact rustfmt-friendly import ordering and local module organization details.

## Specific Ideas

- Keep migration evidence explicit: every moved file must be traceable to one ledger row.
- Keep compatibility facades minimal and temporary.
- Preserve current release behavior while reducing structural coupling.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 042 Source Specs

- `.planning/phases/042-refactor-wallets/042-core-refactore-spec.md` — authoritative scope/objectives/tree/ledger for `src/core`.
- `.planning/phases/042-refactor-wallets/042-db-refactore-spec.md` — authoritative scope/objectives/tree/ledger for `src/db`.
- `.planning/phases/042-refactor-wallets/042-services-refactore-spec.md` — authoritative scope/objectives/tree/ledger for `src/services`.

### Phase State And Roadmap

- `.planning/ROADMAP.md` — Phase 042 boundary and status lane.
- `.planning/STATE.md` — active state pointer and continuity context.

### Current Integration Anchors

- `crates/z00z_wallets/src/core/mod.rs` — current core facade and boundary exports.
- `crates/z00z_wallets/src/db/mod.rs` — current DB facade/re-export seam.
- `crates/z00z_wallets/src/services/mod.rs` — current services facade/re-export seam.

### Architecture And Security Baseline

- `.github/requirements/Z00Z_DESIGN_FOUNDATION.md` — one-source-of-truth abstractions, trait DI, and boundary rules.
- `.github/copilot-instructions.md` — repository operational constraints (including protected directories and safety rules).

## Existing Code Insights

### Reusable Assets

- `core/mod.rs`, `db/mod.rs`, and `services/mod.rs` already provide public seams that can carry compatibility re-exports during staged moves.
- Existing `test_*` suites already cover significant behavior and should be preserved while relocating.

### Established Patterns

- Module-level facade pattern with re-exported subdomains.
- `WalletError`/`WalletResult` propagation and fail-closed flow expectations.
- Strict crate boundary discipline and no vendor-directory edits.

### Integration Points

- Import-path rewiring for all moved files must align with `mod.rs` exports.
- Service layer references to core/db boundaries must stay consistent during intermediate waves.
- Test support path updates are required when files are relocated.

### Validation Gates (Ordered)

1. `cargo fmt`
2. `cargo clippy --all-targets --all-features` (zero warnings for touched areas)
3. `cargo test -p z00z_wallets --no-run`
4. Targeted suite runs for touched domains (`core`, `db`, `services` test subsets)
5. If public API surfaces changed, run `cargo doc --no-deps`

A wave cannot close unless gates 1-4 are green for that wave.

### Rollback And Blockers

- If a move breaks import resolution or public facade continuity, rollback to previous compatibility shim state in the same wave before proceeding.
- If a ledger row target is unverifiable in current tree, mark it as `proposed` and block closure until resolved.
- If cryptographic/security invariants are at risk (secret lifetime, fail-closed semantics, domain separation), stop and reopen wave with explicit fix tasks.

## Deferred Ideas

- Any behavior expansion beyond structural refactor (new wallet features, protocol behavior changes, or API redesign) is deferred to future phases.

---

*Phase: 042-refactor-wallets*
*Context gathered: 2026-05-02*
