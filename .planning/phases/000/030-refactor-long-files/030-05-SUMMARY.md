---
phase: 030-refactor-long-files
plan: 05
subsystem: wallets-address
tags: [rust, wallets, address, facade, seams]
requires:
  - phase: 030-01
    provides: stable facade-first split pattern and protected validation workflow
provides:
  - stable wallet address-manager facade with extracted cache, expiry, config, and rate-limit seams
  - stable wallet address-format facade with extracted codec, validation, normalization, and single or dual address seams
  - structural and semantic regression coverage aligned to the new seam owners
affects: [030-06, 030-09, 030-10, z00z_wallets, z00z_simulator]
tech-stack:
  added: []
  patterns: [include-based stable facade split, bounded batch preflight with atomic reserve, seam-owner structural regression gating]
key-files:
  created: []
  modified:
    - crates/z00z_wallets/src/core/address/address_manager.rs
    - crates/z00z_wallets/src/core/address/address_manager/address_manager_impl.rs
    - crates/z00z_wallets/src/core/address/address_manager/address_manager_trait.rs
    - crates/z00z_wallets/src/core/address/address_manager/eviction_listener.rs
    - crates/z00z_wallets/src/core/address/address_manager/rate_limiter_bucket.rs
    - crates/z00z_wallets/src/core/address/address_manager/tests.rs
    - crates/z00z_wallets/src/core/address/z00z_address.rs
    - crates/z00z_wallets/src/core/address/z00z_address/tests.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_address_codec.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_address_features.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_address_normalize.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_address_validation.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address.rs
    - crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address.rs
key-decisions:
  - Keep `address_manager.rs` and `z00z_address.rs` as stable include-based facades while sibling seam files own the extracted logic.
  - Preserve fast oversized-batch rejection, but reserve rate-limit tokens atomically inside bounded batches so partial cache side effects cannot occur.
  - Treat repeated review-loop findings as in-scope correctness work and close the plan only after two consecutive clean review passes plus a fresh max-safe gate.
patterns-established:
  - "Stable facade split: caller-visible address roots stay shallow and include sibling seam owners instead of widening the public API."
  - "Bounded batch preflight: reject oversize batches before scan work, then reserve rate-limit tokens once for the actual bounded derivation set."
requirements-completed: [PH30-SEAMS, PH30-FACADE, PH30-VERIFY]
duration: 6h 00m
completed: 2026-03-31
---

# Phase 030 Plan 05 Summary

📌 Stable wallet address facades were preserved while cache, expiry,
rate-limit, codec, validation, normalization, and single or dual address
logic moved into coherent seam files with clean targeted and max-safe
verification.

## Performance

- 📌 Duration: 6h 00m
- 📌 Started: 2026-03-31T13:09:00Z
- 📌 Completed: 2026-03-31T19:09:22Z
- 📌 Tasks: 2
- 📌 Files modified: 15

## Accomplishments

- 📌 Split `address_manager.rs` into stable trait, cache, eviction,
  rate-limit, config, and implementation seams without changing the
  caller-visible address facade.
- 📌 Split `z00z_address.rs` into stable feature, validation, codec,
  normalization, single-address, dual-address, and v2 helper seams while
  keeping one public address-format root.
- 📌 Closed the seam split with repeated review-task passes, new semantic
  regressions for batch rate limiting and v2 version validation, a green
  wallet release suite, and a fresh clean max-safe workspace report.

## Task Commits

📌 No git commit was created in this closeout because the repository contains
unrelated dirty files and the current repo workflow requires explicit
version-manager git operations only on request.

## Files Created/Modified

- `crates/z00z_wallets/src/core/address/address_manager.rs` - Kept as the
  stable address-manager facade over extracted seam owners.
- `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl.rs`
  - Centralized bounded batch preflight and atomic rate-limit reservation.
- `crates/z00z_wallets/src/core/address/address_manager/tests.rs` - Added
  regressions for depleted-bucket atomicity and timing-safe batch charging.
- `crates/z00z_wallets/src/core/address/z00z_address.rs` - Kept as the stable
  address-format facade over extracted seam owners.
- `crates/z00z_wallets/src/core/address/z00z_address/tests.rs` - Updated
  source-shape ownership checks and added the v2 wrong-version regression.
- `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`
  - Added fail-closed version gating for v2 helper payload decode.

## Decisions Made

- 📌 Preserve stable root files and move only homogeneous address-domain
  ownership into sibling seams.
- 📌 Keep anti-DoS fast rejection on total batch size, and only do refined
  bounded accounting after the batch is known to be within burst limits.
- 📌 Treat split-aware tests as the source of truth for seam ownership rather
  than reintroducing compatibility clutter into production code.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Repaired extracted address-manager seam structure**

- **Found during:** Task 1 validation
- **Issue:** The extracted manager implementation lost a required field,
  derive metadata, and async-trait ownership, which broke compilation after
  the split.
- **Fix:** Restored the missing `metrics` field, `Debug` coverage, public doc
  ownership, and `#[async_trait]` placement across the manager seams.
- **Files modified:** `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl.rs`, `crates/z00z_wallets/src/core/address/address_manager/address_manager_trait.rs`, `crates/z00z_wallets/src/core/address/address_manager/rate_limiter_bucket.rs`, `crates/z00z_wallets/src/core/address/address_manager/eviction_listener.rs`
- **Verification:** Re-ran targeted address-manager tests, the wallet release
  suite, and the max-safe workspace gate.
- **Committed in:** not committed in this closeout

**2. [Rule 1 - Bug] Repaired extracted address-format seam boundaries**

- **Found during:** Task 2 validation
- **Issue:** The split left orphaned derives and doc comments, duplicate
  imports, a truncated dual-address seam, and duplicated tail code in the v2
  helper seam.
- **Fix:** Restored the missing derives, moved misplaced docs to the correct
  owners, repaired the dual-address tail, and removed duplicate seam content.
- **Files modified:** `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_codec.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_features.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_normalize.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_validation.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_dual_address.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_single_address.rs`
- **Verification:** Re-ran targeted address-format tests, the wallet release
  suite, and the max-safe workspace gate.
- **Committed in:** not committed in this closeout

**3. [Rule 1 - Bug] Updated structural regressions to the new seam owners**

- **Found during:** Wider release-style validation
- **Issue:** Source-shape tests still assumed the old monolith literal file
  layout instead of the new facade-plus-seam ownership.
- **Fix:** Updated split-aware tests to assert the facade include edges and the
  true seam owner for `ADDRESS_VERSION`, plus corrected the test include path.
- **Files modified:** `crates/z00z_wallets/src/core/address/z00z_address/tests.rs`
- **Verification:** Re-ran the targeted address tests and the wallet release
  suite.
- **Committed in:** not committed in this closeout

**4. [Rule 1 - Bug] Fixed façade formatting fallout surfaced by max-safe**

- **Found during:** `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- **Issue:** The stable facade files were missing final newlines, which caused
  `cargo fmt --check` to fail in the workspace gate.
- **Fix:** Added the missing EOF newlines to both address root facade files.
- **Files modified:** `crates/z00z_wallets/src/core/address/address_manager.rs`, `crates/z00z_wallets/src/core/address/z00z_address.rs`
- **Verification:** Re-ran the max-safe workspace gate to a clean summary.
- **Committed in:** not committed in this closeout

**5. [Rule 1 - Bug] Hardened batch rate-limit and v2 decoder semantics after review-loop findings**

- **Found during:** Review-loop passes 2 through 4
- **Issue:** Bounded batch derivation could partially mutate cache state on
  depleted buckets, timing-safe mode undercharged expensive cached batches,
  and the v2 decode helper accepted unsupported versions.
- **Fix:** Added bounded batch preflight with atomic reserve, restored fast
  oversize rejection, aligned timing-safe charging with real work, added a
  fail-closed v2 version gate, and covered all three cases with focused
  regressions.
- **Files modified:** `crates/z00z_wallets/src/core/address/address_manager/address_manager_impl.rs`, `crates/z00z_wallets/src/core/address/address_manager/tests.rs`, `crates/z00z_wallets/src/core/address/z00z_address/z00z_address_parts.rs`, `crates/z00z_wallets/src/core/address/z00z_address/tests.rs`
- **Verification:** Re-ran focused regression tests, the wallet release suite,
  and a fresh max-safe workspace gate.
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 5 auto-fixed bugs
📌 Impact on plan: All fixes stayed inside the address split and validation
closure scope required for `PH30-SEAMS`, `PH30-FACADE`, and `PH30-VERIFY`.

## Issues Encountered

- 📌 Wider validation exposed multiple extraction-boundary mistakes that were
  not obvious from the initial compile surface.
- 📌 Review-loop passes surfaced semantic rate-limit edge cases that required a
  second correctness pass after the seam split was already green on basic
  anchors.
- 📌 The repository remained dirty outside this plan, so closeout stays
  documentation-only until an explicit version-manager git request is given.

## User Setup Required

📌 None - no external service configuration or secrets were required for this
plan.

## Next Phase Readiness

- 📌 Later Phase 030 wallet key, service, transaction, and caller-normalization
  waves can build on smaller address facades without reopening the address
  ownership split.
- 📌 Address-domain structural tests now encode the truthful seam owners, which
  reduces pressure to keep legacy monolith assumptions alive in later waves.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_wallets --release --test test_addr_rate_limit_integration -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_address_manager_create_card -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_address_manager_scan_integration -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release dual_new_accepts_many_pairs -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release uri_rejects_non_ascii -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release bech32m_detects_data_corruption -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_rate_limit_batch_too_large -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_rate_limit_batch_is_atomic -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_rate_limit_batch_counts_timing_safe_paths -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release test_decode_v2_rejects_wrong_version -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --features test-fast --features wallet_debug_dump -- --nocapture`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`
- 📌 Review-loop completed with six passes total and two consecutive clean runs
  at the end.

## Self-Check

📌 PASSED for summary creation, planning-state sync intent, targeted regression
coverage, two consecutive clean review passes, and the fresh clean max-safe
verification summary (`planned=312 skipped=21 failed=0`).

📌 Git closeout intentionally left undone because no explicit commit or push
request was given and the repository contains unrelated dirty files that must
stay outside any version-manager flow.

---
*Phase: 030-refactor-long-files*
*Completed: 2026-03-31*
