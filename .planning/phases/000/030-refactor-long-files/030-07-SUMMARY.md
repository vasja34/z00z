---
phase: 030-refactor-long-files
plan: 07
subsystem: wallets-service-ui
tags: [rust, wallets, services, egui, facade, seams]
requires:
  - phase: 030-06
    provides: stable include-based wallet split workflow and split-closeout validation pattern
provides:
  - stable wallet-service facade with extracted orchestration seams for state, session, store, and test ownership
  - stable app-shell facade with extracted UI config, state machine, theme, tab registry, loader, and main view seams
  - structural guards and release-style verification for the service and app-shell split
affects: [030-08, 030-09, 030-10, z00z_wallets]
tech-stack:
  added: []
  patterns: [include-based stable facade split, app-shell seam extraction, source-shape split guards, release-style validation closeout]
key-files:
  created:
    - crates/z00z_wallets/src/egui_views/ui_config.rs
    - crates/z00z_wallets/src/egui_views/ui_state_machine.rs
    - crates/z00z_wallets/src/egui_views/ui_theme.rs
    - crates/z00z_wallets/src/egui_views/tab_registry.rs
    - crates/z00z_wallets/src/egui_views/main_view_loaders.rs
    - crates/z00z_wallets/src/egui_views/main_view.rs
    - crates/z00z_wallets/src/services/wallet_service_types.rs
    - crates/z00z_wallets/src/services/wallet_service_actions.rs
    - crates/z00z_wallets/src/services/wallet_service_session.rs
    - crates/z00z_wallets/src/services/wallet_service_store.rs
    - crates/z00z_wallets/src/services/wallet_service_tests.rs
    - crates/z00z_wallets/tests/test_phase30_split.rs
  modified:
    - crates/z00z_wallets/src/egui_views/app_main_view.rs
    - crates/z00z_wallets/src/services/app_service.rs
    - crates/z00z_wallets/src/services/wallet_service.rs
    - reports/full_verify-report-long-running-tests.txt
key-decisions:
  - Keep `wallet_service.rs` and `app_main_view.rs` as shallow stable facades while extracted seam files own the split implementation.
  - Preserve service ownership in the service layer and update guard tests to follow the new facade-plus-store contract instead of the old monolith layout.
  - Preserve pre-existing UI placeholder loaders and config stubs during this structural wave instead of widening scope into RPC/UI behavior changes.
patterns-established:
  - "Service split closeout: facade include guards, targeted anchors, release-style clippy and cargo-check, and the canonical max-safe workspace gate all have to agree before the wave closes."
  - "App shell split: UI shell roots stay shallow while config, state machine, theme, loaders, and render logic move into explicit sibling seams."
requirements-completed: [PH30-SEAMS, PH30-FACADE, PH30-VERIFY]
completed: 2026-03-31
---

# Phase 030 Plan 07 Summary

📌 Wallet service orchestration and the egui app shell were split behind stable
service and UI facades, with targeted anchor tests and the canonical
max-safe workspace gate finishing green.

## Accomplishments

- 📌 Converted `wallet_service.rs` from a mixed monolith into a shallow facade
  that includes dedicated seam files for types, orchestration actions,
  session lifecycle, persistence flows, and test ownership.
- 📌 Converted `app_main_view.rs` into a stable facade that delegates UI
  config, local state machine, theme resolution, tab registry, loader helpers,
  and the main egui view surface to sibling seam files.
- 📌 Added source-shape guards for both stable facades and updated the
  app-service ownership guard so the split contract stays explicit in later
  waves.
- 📌 Closed the wave with green targeted wallet anchors, crate-level compile
  and clippy passes, and a clean canonical
  `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`.

## Task Commits

📌 No git commit was created in this closeout. The repository remains dirty,
and the repo rule requires explicit `z00z-git-versioning` workflow usage for
git fixation instead of ad hoc `git commit`.

## Files Created/Modified

- `crates/z00z_wallets/src/services/wallet_service.rs` - Stayed as the stable
  wallet-service root while orchestration internals moved into sibling seams.
- `crates/z00z_wallets/src/services/wallet_service_types.rs` - Took ownership
  of the wallet-service imports, types, and state holders.
- `crates/z00z_wallets/src/services/wallet_service_actions.rs` - Took
  ownership of the main wallet-service orchestration and reachability block.
- `crates/z00z_wallets/src/services/wallet_service_session.rs` - Took
  ownership of constructors, session lifecycle, auto-lock, and derivation
  helpers.
- `crates/z00z_wallets/src/services/wallet_service_store.rs` - Took ownership
  of persistence, import/export, and wallet creation store flows.
- `crates/z00z_wallets/src/services/app_service.rs` - Updated the service
  ownership guard to validate the new facade-plus-store split instead of the
  old monolith contract.
- `crates/z00z_wallets/src/egui_views/app_main_view.rs` - Stayed as the stable
  app-shell root while UI seams moved into sibling files.
- `crates/z00z_wallets/src/egui_views/ui_config.rs` - Took ownership of local
  app-shell configuration structs and fallback config state.
- `crates/z00z_wallets/src/egui_views/ui_state_machine.rs` - Took ownership of
  the local UI context and state machine helpers.
- `crates/z00z_wallets/src/egui_views/ui_theme.rs` - Took ownership of egui
  theme token parsing and color resolution.
- `crates/z00z_wallets/src/egui_views/tab_registry.rs` - Took ownership of tab
  definitions and tab lookup rules.
- `crates/z00z_wallets/src/egui_views/main_view_loaders.rs` - Took ownership
  of app-shell loader helpers and preserved the placeholder loader behavior.
- `crates/z00z_wallets/src/egui_views/main_view.rs` - Took ownership of the
  main egui window render surface.
- `crates/z00z_wallets/tests/test_phase30_split.rs` - Added stable-facade
  split guards for `wallet_service.rs` and `app_main_view.rs`.

## Decisions Made

- 📌 Preserve include-based facades for the wallet service and app shell in
  this wave so downstream callers and later normalization plans can keep a
  stable surface.
- 📌 Keep service ownership checks aligned to the new `wallet_service_store.rs`
  seam instead of weakening access rules to satisfy the old source layout.
- 📌 Keep UI placeholder loader and config stubs intact during the split,
  because this plan closes structure and ownership, not a new RPC-backed UI.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Restored extracted UI helper behavior after the first split pass drifted from the original shell semantics**

- **Found during:** Task 2 validation after the first `app_main_view.rs` extraction
- **Issue:** The first seam extraction changed the effective behavior of theme
  and loader helpers.
- **Fix:** Recovered the original helper implementations from `HEAD` and
  restored equivalent behavior inside `ui_theme.rs` and
  `main_view_loaders.rs` before continuing.
- **Files modified:** `crates/z00z_wallets/src/egui_views/ui_theme.rs`, `crates/z00z_wallets/src/egui_views/main_view_loaders.rs`
- **Verification:** `cargo check -p z00z_wallets --tests`, targeted wallet
  tests, and the final max-safe gate all passed.
- **Committed in:** not committed in this closeout

**2. [Rule 1 - Bug] Fixed split-boundary parse errors in `wallet_service` seam files**

- **Found during:** Task 1 compile validation after the first mechanical file split
- **Issue:** The initial split left invalid include-file doc comments and an
  unclosed `impl WalletService` boundary across seam files.
- **Fix:** Converted the include-file header comments to plain comments and
  reworked the seam boundaries so each extracted file owns a valid, closed
  `impl WalletService { ... }` block.
- **Files modified:** `crates/z00z_wallets/src/services/wallet_service_types.rs`, `crates/z00z_wallets/src/services/wallet_service_actions.rs`, `crates/z00z_wallets/src/services/wallet_service_session.rs`, `crates/z00z_wallets/src/services/wallet_service_store.rs`, `crates/z00z_wallets/src/services/wallet_service.rs`
- **Verification:** `cargo check -p z00z_wallets --tests` passed after the
  seam-boundary repair.
- **Committed in:** not committed in this closeout

**3. [Rule 1 - Bug] Fixed stricter workspace-gate fallout after the structural split**

- **Found during:** canonical `full_verify.sh --max-safe-run`
- **Issue:** The full workspace gate surfaced formatting drift, an invalid
  `.flatten()` call over `Vec<PathBuf>` in `scan_state_transitions`, a clippy
  duplicate-branch warning in `main_view.rs`, and a stale app-service guard
  that still assumed the old monolithic `wallet_service.rs` layout.
- **Fix:** Normalized formatting, rewrote the loader iteration to use the real
  `read_dir` return type, simplified the duplicated drag-target branch, and
  updated the app-service ownership guard to validate the new facade/store
  contract.
- **Files modified:** `crates/z00z_wallets/src/egui_views/main_view_loaders.rs`, `crates/z00z_wallets/src/egui_views/main_view.rs`, `crates/z00z_wallets/src/services/app_service.rs`, `crates/z00z_wallets/tests/test_phase30_split.rs`
- **Verification:** `cargo check -p z00z_wallets --tests`, `cargo clippy -p z00z_wallets --release --all-targets -- -D warnings`, the targeted wallet anchors, and the final max-safe gate all passed.
- **Committed in:** not committed in this closeout

---

📌 Total deviations: 3 auto-fixed issues
📌 Impact on plan: All fixes stayed inside the wallet service or app-shell
split needed to close `PH30-SEAMS`, `PH30-FACADE`, and `PH30-VERIFY`.

## Known Stubs

- 📌 `crates/z00z_wallets/src/egui_views/main_view_loaders.rs` intentionally
  keeps `load_main_ui_spec()` as a placeholder shell loader with empty wallet
  card data.
- 📌 `crates/z00z_wallets/src/egui_views/ui_config.rs` intentionally keeps
  `WalletConfiguration::stub()` as the fallback app-shell config used by the
  current egui facade.
- 📌 `crates/z00z_wallets/src/egui_views/main_view.rs` intentionally keeps the
  existing `TODO` marker for future RPC-based content rendering.

## User Setup Required

📌 None - no external service configuration or secrets were required for this
plan.

## Next Phase Readiness

- 📌 Plan 08 can now attack genesis-facing long files without re-opening the
  wallet service or app-shell monoliths.
- 📌 Later Phase 030 normalization waves can keep the new service and UI seam
  ownership explicit while moving caller-visible paths and docs.

## Verification

- 📌 `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- 📌 `cargo test -p z00z_wallets --release --test test_wallet_service_errors -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_app_service_create_wallet -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_tx_store_integration -- --nocapture`
- 📌 `cargo test -p z00z_wallets --release --test test_open_wallet_source_discovery -- --nocapture`
- 📌 `cargo test -p z00z_wallets --features test-fast services::app_service::tests::test_wallet_create_no_bypass --lib`
- 📌 `cargo test -p z00z_wallets --release --test test_phase30_split -- --nocapture`
- 📌 `cargo check -p z00z_wallets --tests`
- 📌 `cargo clippy -p z00z_wallets --release --all-targets -- -D warnings`
- 📌 `./.github/skills/z00z-full-verify-gate/scripts/full_verify.sh --max-safe-run`

## Self-Check

📌 PASSED: `030-07-SUMMARY.md` exists, `ROADMAP.md` shows `7/12 plans
executed` with `030-07-PLAN.md` checked off, and `STATE.md` now points to
Phase 030 Plan 08 as the next active slot.
