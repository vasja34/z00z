---
phase: 027-crypto-audit-utils
plan: "02"
subsystem: utils
tags: [rust, utils, config, yaml, env, docs]
requires:
  - phase: 027-01
    provides: lifetime-safe secret-memory baseline for the remaining utils audit waves
provides:
  - bounded YAML config loading with explicit oversized, malformed, not-found, and permission-denied classification
  - fail-closed default layered-config constructor with missing-file-only optional downgrade
  - redacted parse-error handling plus explicit rejection of non-UTF-8 env values and non-scalar or null YAML leaves
  - aligned docs, examples, and tests for the same-key-string env precedence contract
affects: [027-03, 027-04, 027-05, 027-06, z00z_utils, z00z_core, z00z_wallets, z00z_storage]
tech-stack:
  added: []
  patterns: [bounded-config-read, fail-closed-layered-config, redacted-parse-surface, serialized-env-tests]
key-files:
  created:
    - .planning/phases/027-crypto-audit-utils/027-02-SUMMARY.md
  modified:
    - crates/z00z_utils/src/config/traits.rs
    - crates/z00z_utils/src/config/env.rs
    - crates/z00z_utils/src/config/yaml.rs
    - crates/z00z_utils/src/config/layered.rs
    - crates/z00z_utils/src/config/mod.rs
    - crates/z00z_utils/src/config/test_config.rs
    - crates/z00z_utils/tests/test_config_integration.rs
    - crates/z00z_utils/examples/config_demo.rs
    - crates/z00z_utils/Z00Z_UTILS_QUICK_REFERENCE.md
    - crates/z00z_utils/Z00Z_UTILS_MODULE_MAP.md
key-decisions:
  - "Keep bounded YAML loading and fail-closed layered construction inside existing z00z_utils seams instead of adding a new parser or config abstraction."
  - "Treat only missing YAML as an allowed optional downgrade and keep malformed, oversized, permission-denied, non-UTF-8, non-scalar, and null values as explicit configuration errors."
  - "Redact raw parse inputs in ConfigError so secret-bearing configuration values do not leak through logs or telemetry on parse failure."
patterns-established:
  - "Config boundaries fail closed: present-but-invalid input must error instead of degrading to caller defaults."
  - "Config tests that mutate global environment state must serialize through a shared lock to avoid parallel flake paths."
requirements-completed: [PH27-CONFIG]
duration: multi-session
completed: 2026-03-29
---

# Phase 027 Plan 02 Summary

📌 **Bounded YAML loading, fail-closed layered config construction, redacted parse errors, and explicit null or non-scalar rejection now define the `z00z_utils` config boundary.**

## Performance

- **Duration:** multi-session
- **Completed:** 2026-03-29T11:20:00Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- ✅ Routed YAML file loading through one bounded read path with explicit not-found, malformed, oversized, and permission-denied classification.
- ✅ Split layered-config construction into a fail-closed default path and an explicit missing-file-only optional YAML downgrade.
- ✅ Closed review-discovered config leaks and downgrade gaps by redacting parse values, rejecting non-UTF-8 env overrides, rejecting non-scalar and null YAML leaves, and aligning docs, examples, and test coverage to the actual same-key-string env contract.

## Task Commits

📌 This execution closed the plan from validated working-tree state.

1. **Task 1: Route YAML file loading through a bounded read and explicit error surface** - not separately committed in this execution
2. **Task 2: Split the layered-config constructors so the permissive path is explicit** - not separately committed in this execution

**Plan metadata:** not committed in this execution; repo-owned git or versioning checkpoint remains deferred.

## Files Created/Modified

- `crates/z00z_utils/src/config/traits.rs` - added redacted parse-value handling so typed config parse errors stop leaking raw secret-bearing values.
- `crates/z00z_utils/src/config/env.rs` - distinguished missing environment values from non-UTF-8 values so invalid env overrides fail explicitly.
- `crates/z00z_utils/src/config/yaml.rs` - moved YAML loading onto bounded reads and made non-scalar and null leaves explicit parse errors instead of silent absence.
- `crates/z00z_utils/src/config/layered.rs` - kept the default constructor fail closed and limited the optional YAML downgrade to missing files only.
- `crates/z00z_utils/src/config/mod.rs` - aligned module docs with the real layered constructor policy and same-key-string env precedence semantics.
- `crates/z00z_utils/src/config/test_config.rs` - extended unit coverage for oversize, malformed, non-UTF-8, non-scalar, null, and redacted-parse cases and serialized env-mutating tests.
- `crates/z00z_utils/tests/test_config_integration.rs` - extended integration coverage for the same error matrix and serialized env-mutating integration tests.
- `crates/z00z_utils/examples/config_demo.rs` - updated the config example to reflect the actual constructor policy and same-key-string env behavior.
- `crates/z00z_utils/Z00Z_UTILS_QUICK_REFERENCE.md` - corrected typed `Option<T>` examples and config guidance so the public quick reference matches the API.
- `crates/z00z_utils/Z00Z_UTILS_MODULE_MAP.md` - corrected config snippet types and aligned the internal reference wording to the final constructor and env policy.

## Decisions Made

- 📌 The default config path remains fail closed; missing YAML is the only tolerated optional downgrade and it must be chosen explicitly.
- 📌 Present-but-invalid config input is treated as an error, not as absence, which now includes non-UTF-8 env values and YAML null or non-scalar leaves.
- 📌 Documentation was aligned to the actual same-key-string env lookup behavior rather than inventing a broader nested env-mapping contract this wave does not implement.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Parse failures leaked raw config values into error surfaces**

- **Found during:** Task 2 final security review
- **Issue:** `ConfigSource::get_typed()` placed the raw input string into parse errors, which could leak secrets through logs or telemetry.
- **Fix:** added redacted parse-value handling in the shared config traits path and regression coverage for secret-bearing parse failures.
- **Files modified:** `crates/z00z_utils/src/config/traits.rs`, `crates/z00z_utils/src/config/test_config.rs`
- **Verification:** `cargo test -p z00z_utils --release --lib`
- **Committed in:** not committed in this execution

**2. [Rule 1 - Bug] Env-mutating config tests could interfere under parallel execution**

- **Found during:** Task 2 review loop
- **Issue:** unit and integration config tests mutated global process environment without a shared serialization guard, creating a real flake risk.
- **Fix:** added an environment mutex and wrapped env-mutating config tests through that shared lock.
- **Files modified:** `crates/z00z_utils/src/config/test_config.rs`, `crates/z00z_utils/tests/test_config_integration.rs`
- **Verification:** `cargo test -p z00z_utils --release --lib`; `cargo test -p z00z_utils --release --test test_config_integration`
- **Committed in:** not committed in this execution

**3. [Rule 1 - Bug] Explicit YAML null still downgraded to missing-key behavior**

- **Found during:** final config security recheck before closeout
- **Issue:** YAML `null` leaves returned `Ok(None)`, preserving a fail-open downgrade path for present-but-invalid values.
- **Fix:** changed null leaves to explicit parse errors and added unit plus integration regressions.
- **Files modified:** `crates/z00z_utils/src/config/yaml.rs`, `crates/z00z_utils/src/config/test_config.rs`, `crates/z00z_utils/tests/test_config_integration.rs`
- **Verification:** `cargo test -p z00z_utils --release --lib`; `cargo test -p z00z_utils --release --test test_config_integration`; `cargo test -p z00z_utils --doc`; `cargo check -p z00z_utils --release --example config_demo`
- **Committed in:** not committed in this execution

---

**Total deviations:** 3 auto-fixed (1 missing critical, 2 bug)
**Impact on plan:** All deviations stayed inside `PH27-CONFIG` and were required to close real review-discovered downgrade, leak, and flake paths before honest plan closure.

## Issues Encountered

- ⚠️ Review uncovered multiple gaps that tests alone had not surfaced yet: raw parse-value leakage, non-serialized env mutation, and explicit YAML null still behaving like absence.
- ⚠️ Public docs and examples initially overstated env override semantics; the final state documents the actual same-key-string contract rather than adding a broader nested env mapping in this wave.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- ✅ `PH27-CONFIG` is now closed with bounded YAML reads, fail-closed layered construction, explicit present-but-invalid errors, and aligned docs/examples.
- ✅ Later Phase 027 waves can rely on the config boundary without carrying silent YAML suppression or parse-value leakage forward.
- ⚠️ `LayeredConfig::new()` still resolves `config.yaml` relative to the current working directory; that operational footgun remains documented residual risk outside this plan’s closed scope.

## Validation Evidence

- ✅ `cargo test -p z00z_utils --release --lib` -> `154 passed; 0 failed`
- ✅ `cargo test -p z00z_utils --release --test test_config_integration` -> `27 passed; 0 failed`
- ✅ `cargo test -p z00z_utils --doc` -> `42 passed; 0 failed`
- ✅ `cargo check -p z00z_utils --release --example config_demo` -> build passed
- ✅ `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` -> passed earlier in the same closeout cycle before the final null-leaf hardening pass
- ✅ `cargo test --release --features test-fast --features wallet_debug_dump` -> passed earlier in the same closeout cycle before the final null-leaf hardening pass
- ✅ Final focused config review closed the last in-scope fail-open case by turning explicit YAML null into a typed parse error

## Self-Check: PASSED

- ✅ Summary artifact created at `.planning/phases/027-crypto-audit-utils/027-02-SUMMARY.md`
- ✅ Validation evidence recorded against the final post-null-fix config tree for the narrow config gate
- ✅ No commit hashes were claimed because git checkpointing was not performed in this execution

---

*Phase: 027-crypto-audit-utils*
*Completed: 2026-03-29*
