---
phase: 031-refactor-architecture
plan: "04"
subsystem: networking
tags: [rust, rpc, onionnet, transport, overlay, boundary]
requires:
  - phase: 031-01
    provides: Wave 0 inventory and import-graph proof for the Phase 031 boundary cleanup.
provides:
  - Explicit transport-only ownership contract for `z00z_networks_rpc`.
  - Named deferred seams for peer identity, authentication, retry policy, and connection lifecycle.
  - A documented `onionnet` placeholder crate boundary aligned to the Phase 115 module map.
affects: [031-05, 031-07, 115-onionnet, z00z_networks_rpc, onionnet]
tech-stack:
  added: []
  patterns: [transport-only rpc seam, crate-shaped overlay placeholder boundary]
key-files:
  created: []
  modified:
    [crates/z00z_networks/rpc/Cargo.toml, crates/z00z_networks/rpc/src/lib.rs, crates/z00z_networks/rpc/src/transport.rs, crates/z00z_networks/rpc/src/dispatcher.rs, crates/z00z_networks/rpc/src/local_transport.rs, crates/z00z_networks/rpc/src/error.rs, crates/z00z_networks/onionnet/Cargo.toml, crates/z00z_networks/onionnet/README.md, crates/z00z_networks/onionnet/src/lib.rs]
key-decisions:
  - "Keep `z00z_networks_rpc` limited to transport and dispatch concerns, and name peer identity, authentication, retry policy, and connection lifecycle as external seams instead of growing the trait surface."
  - "Treat the existing `onionnet` crate as the canonical placeholder boundary and align its docs to Phase 115 rather than moving or renaming the namespace."
patterns-established:
  - "Generic transport crates document adjacent higher-level seams explicitly instead of silently absorbing auth or lifecycle policy later."
  - "Future overlay work lands behind a crate-shaped placeholder seam with the final module map reserved in place."
requirements-completed: [PH31-NET]
duration: 1m 15s
completed: 2026-04-04
---

# Phase 031 Plan 04: Network Boundary Clarification Summary

**Transport-only `z00z_networks_rpc` contract with an explicit `onionnet` node-overlay placeholder aligned to the Phase 115 module map**

## Performance

- **Duration:** 1m 15s
- **Started:** 2026-04-04T14:55:07Z
- **Completed:** 2026-04-04T14:56:22Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Narrowed `z00z_networks_rpc` documentation and module contracts to transport, request dispatch, and local testing helpers only.
- Named peer identity, authentication, retry policy, and connection lifecycle as explicitly external seams so later node or wallet work does not smuggle them into the generic RPC crate.
- Aligned the existing `onionnet` placeholder crate to the Phase 115 ownership model and reserved module map without inventing fake runtime behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Narrow the RPC crate to transport and dispatch concerns only** - `0010f234`, `b089ac8b` (test, feat)
2. **Task 2: Write the OnionNet ownership note without inventing a fake implementation** - `83bf10ba` (feat)

## Files Created/Modified

- `crates/z00z_networks/rpc/Cargo.toml` - fixed the direct `z00z_utils` dependency so the RPC crate can compile the codec-based default path.
- `crates/z00z_networks/rpc/src/lib.rs` - documented the transport-only crate contract and added source-shape tests for the boundary wording.
- `crates/z00z_networks/rpc/src/transport.rs` - clarified that peer identity, authentication, retry policy, and connection lifecycle stay outside the transport trait.
- `crates/z00z_networks/rpc/src/dispatcher.rs` - documented dispatcher ownership as method routing only.
- `crates/z00z_networks/rpc/src/local_transport.rs` - documented the in-process adapter as a local testing helper rather than a remote network policy owner.
- `crates/z00z_networks/rpc/src/error.rs` - clarified that the generic error surface is transport-facing and higher-level policy is mapped in by owning adapters.
- `crates/z00z_networks/onionnet/Cargo.toml` - tightened placeholder crate metadata for the overlay namespace.
- `crates/z00z_networks/onionnet/README.md` - documented OnionNet as a node-owned privacy overlay and crate-shaped placeholder seam.
- `crates/z00z_networks/onionnet/src/lib.rs` - clarified the overlay role while preserving the reserved Phase 115 module map in code.

## Decisions Made

- Preserved a narrow `RpcTransport` surface instead of expanding the trait with speculative peer identity, auth, retry, or lifecycle hooks.
- Reused the already-present `onionnet` crate as the canonical placeholder seam because the repository had already adopted the Phase 115 namespace shape.
- Used source-shape tests in `z00z_networks_rpc` to keep the limited ownership wording from drifting after later networking work lands.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed the missing direct `z00z_utils` dependency in `z00z_networks_rpc`**

- **Found during:** Task 1 (Narrow the RPC crate to transport and dispatch concerns only)
- **Issue:** `z00z_networks_rpc` already depended on `z00z_utils::codec::Value` across the default code path, but `Cargo.toml` still declared `z00z_utils` as an optional dependency, so `cargo test -p z00z_networks_rpc --release` failed before the boundary review could even run.
- **Fix:** Made `z00z_utils` a normal dependency and left the `logger` feature as a reserved no-op gate for future transport-local integrations.
- **Files modified:** `crates/z00z_networks/rpc/Cargo.toml`
- **Verification:** `cargo test -p z00z_networks_rpc --release -- --nocapture`
- **Committed in:** `0010f234`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The fix was required just to compile and validate the RPC boundary work; no additional runtime scope was introduced.

## Issues Encountered

- The repository already contained an `onionnet` placeholder crate, so Task 2 became an alignment and documentation pass rather than a brand-new crate creation step.
- The executor environment did not provide a direct `/GSD-Review-Tasks-Execution` runner, so the review loop was completed through three explicit manual passes over source-shape tests, grep guards, and plan-required validation commands.

## Review Passes

- **Pass 1:** TDD review on `z00z_networks_rpc` boundary wording via failing source-shape tests, then green re-run with `cargo test -p z00z_networks_rpc --release -- --nocapture`.
- **Pass 2:** Plan-required network boundary validation via `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`, `rg -n "peer|auth|retry|connection|transport|dispatcher|local transport|rpc" crates/z00z_networks/rpc/src -g '*.rs'`, and `cargo test -p z00z_networks_rpc --release -- --nocapture`.
- **Pass 3:** OnionNet ownership validation via `rg -n "OnionNet|overlay|RPC|transport|peer identity|privacy|placeholder|interface|transport_quic|sphinx_path|bridge_api|telemetry" crates/z00z_networks/onionnet crates/z00z_networks/rpc -g '*.md' -g '*.rs' -g 'Cargo.toml'` and `cargo test -p onionnet --release -- --nocapture`.

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `z00z_networks_rpc` now states one limited transport contract, so later wallet and simulator cleanup can depend on it without assuming hidden auth or lifecycle ownership.
- `onionnet` already reserves the Phase 115 namespace shape in-place, so later overlay implementation can fill real modules without a crate reshuffle.

## Known Stubs

None.

## Self-Check: PASSED

- Found `.planning/phases/031-refactor-architecture/031-04-SUMMARY.md`
- Found commit `0010f234`
- Found commit `b089ac8b`
- Found commit `83bf10ba`
