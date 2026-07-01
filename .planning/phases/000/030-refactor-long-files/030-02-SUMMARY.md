---
phase: 030-refactor-long-files
plan: 02
subsystem: database
tags: [assets, registry, nonce, definition, wire, facade, refactor]
requires:
  - phase: 030-01
    provides: wallet-store semantic seam split patterns and repo-managed closeout flow
provides:
  - semantic asset-domain seams behind the stable `z00z_core::assets` facade
  - split registry, nonce, and definition internals without caller-path drift
  - hardened asset wire confidentiality and compatibility regression anchors
affects: [phase-030-03, phase-030-asset-followups, asset-wire, genesis, wallet-import]
tech-stack:
  added: []
  patterns: [semantic seam extraction, facade-preserving split, backward-compatible signature framing, transport secret scrubbing]
key-files:
  created:
    - crates/z00z_core/src/assets/asset_class.rs
    - crates/z00z_core/src/assets/asset_error.rs
    - crates/z00z_core/src/assets/asset_metadata.rs
    - crates/z00z_core/src/assets/asset_ownership.rs
    - crates/z00z_core/src/assets/asset_validation.rs
    - crates/z00z_core/src/assets/registry_core.rs
    - crates/z00z_core/src/assets/registry_config.rs
    - crates/z00z_core/src/assets/registry_snapshot.rs
    - crates/z00z_core/src/assets/nonce_type.rs
    - crates/z00z_core/src/assets/nonce_counter.rs
    - crates/z00z_core/src/assets/nonce_derivation.rs
    - crates/z00z_core/src/assets/definition_id.rs
    - crates/z00z_core/src/assets/definition_validate.rs
  modified:
    - crates/z00z_core/src/assets/assets.rs
    - crates/z00z_core/src/assets/registry.rs
    - crates/z00z_core/src/assets/nonce.rs
    - crates/z00z_core/src/assets/definition.rs
    - crates/z00z_core/src/assets/mod.rs
    - crates/z00z_core/src/assets/wire.rs
    - crates/z00z_core/src/assets/test_wire.rs
    - crates/z00z_core/tests/assets/test_metadata_limits.rs
    - crates/z00z_core/tests/assets/test_wire_format_snapshots.rs
    - docs/code-review/2026-03-31-phase-030-plan-02-asset-domain-split-review.md
key-decisions:
  - Keep `assets.rs`, `registry.rs`, `nonce.rs`, and `definition.rs` as stable facades while moving homogeneous logic into sibling seams.
  - Preserve canonical asset identity and metadata hashing behavior; harden coverage instead of changing production framing rules without migration.
  - Keep owner-signature verification backward-compatible by accepting the new framed message first and the legacy message second.
  - Treat `AssetWire` as a public transport boundary that must never export runtime `secret` material.
patterns-established:
  - Facade-preserving asset splits: public types stay rooted in the existing asset module surface while validation, ownership, config, and snapshot code move behind internal seams.
  - Compatibility-first hardening: add negative regression tests and legacy verify fallback before changing any signature- or wire-bearing contract.
requirements-completed: [PH30-SEAMS, PH30-VERIFY]
duration: multi-session
completed: 2026-03-31
---

# Phase 030 Plan 02 Summary

## Outcome

Asset, registry, nonce, and definition monoliths split into semantic seams behind the stable asset facade, with wire secrecy and compatibility anchors kept green.

## Performance

- **Duration:** multi-session
- **Started:** 2026-03-31T06:40:21Z
- **Completed:** 2026-03-31T00:00:00Z
- **Tasks:** 2
- **Files modified:** 23

## Accomplishments

- Split the asset-domain root files into responsibility-focused seams for errors, classes, metadata, validation, ownership, registry core/config/snapshot, nonce policy, and definition identity/validation while preserving the current `z00z_core::assets` facade surface.
- Restored compile and contract stability after the split, including registry visibility/path fixes, nonce and definition re-export stability, and compatibility-preserving owner-signature framing with legacy verification fallback.
- Closed the final confidentiality finding by making `AssetWire::from_asset()` scrub runtime `secret` material and by expanding regression coverage for metadata integrity, stealth-wire roundtrip behavior, and secret rejection on import.

## Task Commits

This plan is prepared for one consolidated repository-managed version-manager commit after artifact creation. Per-task hashes were not materialized during execution because the repo workflow requires the final staged file set to flow through `version-manager.sh`.

## Files Created/Modified

- `crates/z00z_core/src/assets/assets.rs` - Stable asset facade with debug redaction hardening and legacy owner-signature regression coverage.
- `crates/z00z_core/src/assets/asset_class.rs` - Canonical asset-class taxonomy extracted from the mixed asset root.
- `crates/z00z_core/src/assets/asset_error.rs` - Unified asset error surface extracted behind the existing facade.
- `crates/z00z_core/src/assets/asset_metadata.rs` - Metadata hashing and verification helpers isolated into one seam.
- `crates/z00z_core/src/assets/asset_ownership.rs` - Owner-signature framing, signing, and backward-compatible verification logic.
- `crates/z00z_core/src/assets/asset_validation.rs` - Asset validation, amount checks, stealth consistency, and verification helpers.
- `crates/z00z_core/src/assets/registry.rs` - Stable registry facade over extracted core, config, and snapshot seams.
- `crates/z00z_core/src/assets/registry_core.rs` - Registry mutation, lookup, version, and batch insert behavior.
- `crates/z00z_core/src/assets/registry_config.rs` - Registry YAML/config-loading seam.
- `crates/z00z_core/src/assets/registry_snapshot.rs` - Snapshot export/import integrity and shared-snapshot helpers.
- `crates/z00z_core/src/assets/nonce.rs` - Stable nonce facade over split nonce internals.
- `crates/z00z_core/src/assets/nonce_type.rs` - Nonce alias and timestamp helpers.
- `crates/z00z_core/src/assets/nonce_counter.rs` - Persistent nonce-counter logic.
- `crates/z00z_core/src/assets/nonce_derivation.rs` - Deterministic and minimal nonce derivation helpers.
- `crates/z00z_core/src/assets/definition.rs` - Stable `AssetDefinition` facade over identity and validation seams.
- `crates/z00z_core/src/assets/definition_id.rs` - Canonical definition-id derivation and validation.
- `crates/z00z_core/src/assets/definition_validate.rs` - Definition field-validation seam.
- `crates/z00z_core/src/assets/mod.rs` - Public asset re-export surface updated for the extracted seams.
- `crates/z00z_core/src/assets/wire.rs` - Asset transport DTO now scrubs runtime `secret` material on export.
- `crates/z00z_core/src/assets/test_wire.rs` - Unit regression coverage for secret scrubbing and import boundary checks.
- `crates/z00z_core/tests/assets/test_metadata_limits.rs` - Real metadata-hash contract coverage.
- `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs` - Expanded stealth/wire regression coverage and explicit secret rejection coverage.
- `docs/code-review/2026-03-31-phase-030-plan-02-asset-domain-split-review.md` - Recorded review findings and hardening follow-up context for this wave.

## Decisions Made

- Keep the old asset root file names as compatibility facades instead of replacing them with numeric or shard-style files.
- Refuse canonical asset-identity drift: when review surfaced concerns around metadata hashing, keep production hashing stable and strengthen tests rather than silently changing the protocol contract.
- Preserve verification compatibility for already-signed assets by checking the new owner-signature message first and the legacy message second.
- Treat wire-secret export as a hard boundary violation and fix it in the DTO conversion path rather than only documenting the import restriction.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired registry seam wiring after the initial extraction**

- **Found during:** Task 1
- **Issue:** Partial extraction left missing imports, duplicate helpers, premature module declarations, and a too-narrow `insert_prechecked` visibility that broke `genesis` callers and compile.
- **Fix:** Removed duplicated logic from `assets.rs`, corrected module wiring, restored `insert_prechecked` to `pub(crate)`, and stabilized the extracted registry seams.
- **Files modified:** `crates/z00z_core/src/assets/assets.rs`, `crates/z00z_core/src/assets/registry.rs`, `crates/z00z_core/src/assets/registry_core.rs`, `crates/z00z_core/src/assets/registry_config.rs`, `crates/z00z_core/src/assets/registry_snapshot.rs`, `crates/z00z_core/src/assets/mod.rs`
- **Verification:** `cargo check -p z00z_core --lib`
- **Committed in:** Pending consolidated version-manager commit

**2. [Rule 1 - Bug] Corrected verification anchors to the real cargo target layout**

- **Found during:** Task 1 verification
- **Issue:** The plan referenced nonexistent `test_assets` and `test_wire_format_snapshots` cargo targets, which would have made verification evidence misleading.
- **Fix:** Mapped the phase to the real target surface: `assets_tests`, the `wire_format_snapshots` filter inside that target, `asset_signature_domain`, `stealth_consistency`, and the release-style `z00z_core` `test-fast` gate.
- **Files modified:** none
- **Verification:** `cargo test -p z00z_core --release --test assets_tests wire_format_snapshots -- --nocapture`, `cargo test -p z00z_core --release owner_signature -- --nocapture`, `cargo test -p z00z_core --release test_from_asset_scrubs_secret -- --nocapture`, `cargo test -p z00z_core --release --features test-fast -- --nocapture`
- **Committed in:** Pending consolidated version-manager commit

**3. [Rule 1 - Bug] Hardened owner-signature framing without breaking legacy signatures**

- **Found during:** Task 2 review passes
- **Issue:** The canonical owner-signature message did not frame the variable-length `range_proof` slot strongly enough for the new seam layout and review requirements.
- **Fix:** Added an explicit framed owner-message path for current signatures, preserved legacy message generation, and made verification accept legacy signatures as a fallback.
- **Files modified:** `crates/z00z_core/src/assets/asset_ownership.rs`, `crates/z00z_core/src/assets/assets.rs`
- **Verification:** `cargo test -p z00z_core --release owner_signature -- --nocapture`, `cargo test -p z00z_core --release legacy_owner_signature -- --nocapture`, `cargo clippy -p z00z_core --release --all-targets -- -D warnings`
- **Committed in:** Pending consolidated version-manager commit

**4. [Rule 2 - Missing Critical] Closed the asset-wire confidentiality leak on export**

- **Found during:** Final security review
- **Issue:** `AssetWire::from_asset()` still copied runtime `secret` material into the public transport DTO even though import paths rejected it.
- **Fix:** Scrubbed `secret` on wire export and added regression coverage proving both export scrubbing and import rejection.
- **Files modified:** `crates/z00z_core/src/assets/wire.rs`, `crates/z00z_core/src/assets/test_wire.rs`, `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs`
- **Verification:** `cargo test -p z00z_core --release test_from_asset_scrubs_secret -- --nocapture`, `cargo test -p z00z_core --release --test assets_tests wire_format_snapshots -- --nocapture`, final read-only security review
- **Committed in:** Pending consolidated version-manager commit

**5. [Rule 3 - Blocking] Fixed in-scope fmt/clippy fallout detected by the max-safe gate**

- **Found during:** Final phase validation
- **Issue:** `full_verify.sh --max-safe-run` first flagged formatting drift in touched asset files and then flagged a clippy issue in the legacy-signature fallback branch.
- **Fix:** Applied `rustfmt` to the in-scope asset files and simplified the legacy verify path into one boolean decision to satisfy clippy without changing behavior.
- **Files modified:** `crates/z00z_core/src/assets/assets.rs`, `crates/z00z_core/src/assets/asset_ownership.rs`, `crates/z00z_core/src/assets/test_wire.rs`, `crates/z00z_core/tests/assets/test_metadata_limits.rs`, `crates/z00z_core/tests/assets/test_wire_format_snapshots.rs`
- **Verification:** `cargo clippy -p z00z_core --release --all-targets -- -D warnings`
- **Committed in:** Pending consolidated version-manager commit

---

**Total deviations:** 5 auto-fixed (2 bug fixes, 1 missing critical fix, 2 blocking/verifier fixes)
**Impact on plan:** All deviations were required either to keep the seam split compiling and verifiable or to preserve confidentiality and compatibility guarantees. No unrelated product scope was added.

## Issues Encountered

- `.planning/STATE.md` drifted back to `030-01` execution state during the wave and needs explicit resynchronization during closeout.
- Running bootstrap and `cargo test -p z00z_core --release --features test-fast` in parallel produced a transient rustdoc race on shared build artifacts; the serial rerun passed cleanly and the issue was procedural, not code-level.
- The repo worktree already contained unrelated deletions and wallet-side formatting changes, so this plan must close on a carefully staged asset-domain file set only.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The asset domain now has clear internal seams and a stable external facade, so later Phase 030 waves can normalize deeper caller paths without redoing the structural split.
- Registry, nonce, and definition logic are now isolated enough for follow-up caller/path cleanup or further focused audits.
- Repository-managed commit and push are still pending because the repo workflow requires an explicit staged closeout and version-manager run.

## Deferred Issues

- None added in this wave. Existing Phase 030 deferred items remain in `deferred-items.md`.

## Self-Check: PENDING

- Summary created; final existence and commit checks should be refreshed after planning-state sync and the repository-managed closeout commit.

---
*Phase: 030-refactor-long-files*
*Completed: 2026-03-31*
