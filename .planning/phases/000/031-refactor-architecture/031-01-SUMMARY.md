---
phase: 031-refactor-architecture
plan: "01"
subsystem: architecture
tags: [inventory, import-graph, facade, caller-proof, phase-031]
requires:
  - phase: 030-refactor-long-files
    provides: zero long-file residue and stable facade split baseline
provides:
  - canonical Wave 0 seam inventory for reviewed Phase 031 crates
  - Gate G-00 import graph with per-crate caller and hot-path evidence
  - wave ordering proof for parallel core/crypto cleanup and serialized wallet cleanup
affects: [031-core, 031-crypto, 031-wallet, 031-storage, 031-simulator, 031-closeout]
tech-stack:
  added: []
  patterns: [inventory-first refactor gating, evidence-backed caller proof, plan-owned seam retirement]
key-files:
  created:
    - .planning/phases/031-refactor-architecture/031-INVENTORY.md
    - .planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md
    - .planning/phases/031-refactor-architecture/031-01-SUMMARY.md
  modified:
    - .planning/STATE.md
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md
key-decisions:
  - "Treat Wave 0 artifacts as a mandatory proof gate before any facade narrowing or suffix retirement."
  - "Allow `031-02` and `031-03` to proceed in parallel only because the import graph isolates their caller overlap."
  - "Keep wallet cleanup split across service, identity/auth, and RPC lanes instead of collapsing it into one plan."
patterns-established:
  - "Wave 0 inventory first: document seams before retiring them."
  - "Import-graph-backed sequencing: use caller proof to justify parallel or serialized cleanup."
requirements-completed: [PH31-INV]
duration: multi-session
completed: 2026-04-04
---

# Phase 031 Plan 01: Wave 0 Inventory Summary

## Overview

Wave 0 seam inventory and Gate G-00 import graph for evidence-backed Phase 031 refactor sequencing.

## Performance

- **Duration:** multi-session
- **Started:** 2026-04-04T13:33:36Z
- **Completed:** 2026-04-04T13:45:58Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created the canonical Phase 031 seam inventory covering wildcard exports, versioned or compat lanes, Tari leakage, wallet service assembly, storage semantics, simulator deep imports, and utils/network boundary drift.
- Created the Gate G-00 import graph with per-crate caller and hot-path summaries for `z00z_core`, `z00z_crypto`, `z00z_wallets`, `z00z_storage`, `z00z_simulator`, `z00z_utils`, and `z00z_networks_rpc`.
- Proved that Wave 1 `z00z_core` and `z00z_crypto` cleanup may run in parallel, while wallet plans `031-05` through `031-07` must stay serialized to avoid shared-caller drift.

## Task Commits

Each task was committed atomically:

1. **Task 1: Produce the canonical Phase 031 seam inventory** - `11164e3a` (chore)
2. **Task 2: Produce the per-crate import-graph proof required by Gate G-00** - `3898a9b2` (chore)
3. **Plan metadata:** pending final docs commit

## Files Created/Modified

- `.planning/phases/031-refactor-architecture/031-INVENTORY.md` - Authoritative Wave 0 seam inventory and later-plan ownership map.
- `.planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md` - Per-crate caller graph and sequencing proof for Gate G-00.
- `.planning/phases/031-refactor-architecture/031-01-SUMMARY.md` - Closeout summary for the Wave 0 plan.
- `.planning/STATE.md` - Advances active Phase 031 position from plan 01 to plan 02.
- `.planning/ROADMAP.md` - Marks `031-01` summary-backed and updates Phase 031 execution status.
- `.planning/REQUIREMENTS.md` - Marks `PH31-INV` complete.

## Decisions Made

- Wave 0 remains a non-optional proof gate; no later plan may rediscover or reinterpret seam scope ad hoc.
- `z00z_core` and `z00z_crypto` can move in parallel only under explicit caller-proof constraints from `031-IMPORT-GRAPH.md`.
- Wallet cleanup stays split into service, identity/auth, and RPC ownership waves because the same callers overlap different seam classes.

## Deviations from Plan

None - plan executed exactly as written.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` completed green; the captured log ends with `=== BOOTSTRAP COMPLETE ===`.
- `cargo test -p z00z_crypto --release --test test_hash_policy -- --nocapture` passed (`6 passed; 0 failed`).
- `cargo test -p z00z_core --release --test assets_tests -- --nocapture` completed with exit code `0`.
- Search guards confirmed the documented seam and caller pressure, including the direct simulator import `use z00z_wallets::services::WalletService;` in `crates/z00z_simulator/src/context.rs`.

## Issues Encountered

- Interactive terminal sessions occasionally closed on high-volume `rg` and `cargo test` commands. Verification was stabilized with background execution, captured logs, and workspace-native search results without changing scope or outputs.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 031 can advance to `031-02` and `031-03` in parallel, using the Wave 0 artifacts as mandatory caller-proof references.
- Wallet boundary work must remain serialized beginning with `031-05` because Wave 0 confirmed overlapping downstream callers across service, identity/auth, and RPC lanes.

## Self-Check: PASSED

- Verified file presence for `.planning/phases/031-refactor-architecture/031-INVENTORY.md`, `.planning/phases/031-refactor-architecture/031-IMPORT-GRAPH.md`, and `.planning/phases/031-refactor-architecture/031-01-SUMMARY.md`.
- Verified task commits `11164e3a` and `3898a9b2` exist in git history.

---
*Phase: 031-refactor-architecture*
*Completed: 2026-04-04*
