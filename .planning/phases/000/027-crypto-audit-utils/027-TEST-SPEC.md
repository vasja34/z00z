---
phase: 027-crypto-audit-utils
artifact: test-spec
status: verification-backed
source: context-fusion-research-plans-and-live-code-seams
updated: 2026-03-29
---

# Phase 027 Test Spec

## Purpose

📌 This document defines the unit, Rust integration, and end-to-end acceptance
coverage required for Phase 027.

📌 It is intended to be directly usable by another engineer or agent without
guessing scenario boundaries, state transitions, failure oracles, or which
existing test seam should own a given assertion.

📌 Phase 027 is a `z00z_utils` security-hardening phase. Its end-to-end proof is
not browser automation. The required E2E signal is Rust integration coverage
plus targeted downstream release-style validation that proves memory-locking,
config fail-closed behavior, time-policy migration, deterministic-RNG
guardrails, persisted logger behavior, file-write semantics, and JSON-boundary
ownership under realistic success and failure inputs.

## Workflow Status

✅ Strict fallback conditions no longer apply because
`.planning/phases/027-crypto-audit-utils/` now contains summary artifacts for
plans `027-01` through `027-06` and a phase-local
`027-VERIFICATION.md` artifact.

📌 This test spec is now verification-backed and uses these inputs as the
current source of truth:

- `.planning/phases/027-crypto-audit-utils/027-FUSION.md`
- `.planning/phases/027-crypto-audit-utils/027-CONTEXT.md`
- `.planning/phases/027-crypto-audit-utils/027-RESEARCH.md`
- `.planning/phases/027-crypto-audit-utils/027-01-PLAN.md`
- `.planning/phases/027-crypto-audit-utils/027-02-PLAN.md`
- `.planning/phases/027-crypto-audit-utils/027-03-PLAN.md`
- `.planning/phases/027-crypto-audit-utils/027-04-PLAN.md`
- `.planning/phases/027-crypto-audit-utils/027-05-PLAN.md`
- `.planning/phases/027-crypto-audit-utils/027-06-PLAN.md`
- `.planning/phases/027-crypto-audit-utils/027-01-SUMMARY.md`
- `.planning/phases/027-crypto-audit-utils/027-02-SUMMARY.md`
- `.planning/phases/027-crypto-audit-utils/027-03-SUMMARY.md`
- `.planning/phases/027-crypto-audit-utils/027-04-SUMMARY.md`
- `.planning/phases/027-crypto-audit-utils/027-05-SUMMARY.md`
- `.planning/phases/027-crypto-audit-utils/027-06-SUMMARY.md`
- `.planning/phases/027-crypto-audit-utils/027-VERIFICATION.md`
- `.planning/REQUIREMENTS.md`
- Existing test anchors in `crates/z00z_utils/tests/`,
  `crates/z00z_wallets/tests/`, `crates/z00z_core/tests/`, and
  `crates/z00z_simulator/tests/`

📌 This document remains the phase-local test contract for Phase 027 and is now
backed by executed verification evidence.

## Classification

### TDD And Integration Targets

- `crates/z00z_utils/src/os_hardening.rs`
  because Phase 027 must prove one lifetime-safe `LockedBytes<'a>` contract,
  zeroize-before-unlock behavior, and safe debug surface.
- `crates/z00z_utils/src/config/yaml.rs`
  because YAML loading must become bounded and explicitly fail closed.
- `crates/z00z_utils/src/config/layered.rs`
  because the default layered-config constructor must stop silently dropping
  malformed or unreadable YAML.
- `crates/z00z_utils/src/time/traits.rs`
  because the fallible-versus-compatibility time policy is defined here.
- `crates/z00z_utils/tests/test_time_policy_micros.rs`
  because Phase 027 must keep one repository guard seam for forbidden lossy
  time usage.
- `crates/z00z_wallets/src/**`, `crates/z00z_core/src/**`,
  `crates/z00z_storage/src/**`, and `crates/z00z_simulator/src/**`
  because Phase 027 must prove that security-critical time consumers migrate or
  are explicitly classified.
- `crates/z00z_utils/src/rng/traits.rs` and
  `crates/z00z_utils/src/rng/deterministic.rs`
  because deterministic RNG semantics and guardrails live here.
- `crates/z00z_core/src/genesis/**`,
  `crates/z00z_core/bin/assets/assets_generation_cli.rs`,
  `crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_lane_impl.rs`, and
  `crates/z00z_crypto/src/types.rs`
  because Phase 027 must distinguish approved reproducibility domains from
  non-allowlisted deterministic seams.
- `crates/z00z_utils/src/logger/mod.rs`,
  `crates/z00z_utils/src/logger/file_logger.rs`,
  `crates/z00z_utils/src/logger/rotating_file_logger.rs`,
  `crates/z00z_utils/src/logger/structured.rs`, and
  `crates/z00z_utils/src/logger/macros.rs`
  because persisted logger behavior, serialization-failure signaling, and the
  sanctioned JSON boundary all converge there.
- `crates/z00z_utils/src/io/fs.rs`
  because generic versus private atomic write semantics and permission-copy
  failures must be explicit.
- `crates/z00z_utils/src/codec/mod.rs`
  because the narrow JSON compatibility exception must be deliberate and owned.

### E2E Browser Targets

- None.

📌 Phase 027 end-to-end proof must remain in Rust integration tests and
release-style crate commands because the phase hardens library boundaries,
downstream consumers, and file or process behavior rather than browser flows.

### Skip Targets

- Planning markdown files themselves
  because they are specification inputs, not executable logic.
- Vendor code under `crates/z00z_crypto/tari/`
  because Phase 027 must preserve the vendor boundary.
- UI-only wallet examples and browser surfaces
  unless they are explicitly used as downstream time-policy consumers.

## Existing Test Structure

📌 Phase 027 already has one direct integration seam per core `z00z_utils`
subdomain:

- `crates/z00z_utils/tests/test_os_hardening_integration.rs`
- `crates/z00z_utils/tests/test_config_integration.rs`
- `crates/z00z_utils/tests/test_time_policy_micros.rs`
- `crates/z00z_utils/tests/test_logger_integration.rs`
- `crates/z00z_utils/tests/test_io_integration.rs`
- `crates/z00z_utils/tests/test_codec_integration.rs`

📌 The repository already has realistic downstream anchors that should be
reused before creating new files:

- `crates/z00z_wallets/tests/test_addr_rate_limit_integration.rs`
- `crates/z00z_wallets/tests/test_key_manager.rs`
- `crates/z00z_wallets/tests/test_stealth_request.rs`
- `crates/z00z_wallets/tests/test_rpc_logging_acceptance.rs`
- `crates/z00z_core/tests/genesis/test_reproducibility.rs`
- `crates/z00z_core/tests/genesis/test_genesis.rs`
- `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

📌 Proposed new files are acceptable only when extending one of the anchors
above would blur ownership across unrelated assertions.

## Canonical Test Commands

📌 Every implementation wave should keep the same top-level validation order:

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo test -p z00z_utils --release`
- `cargo test -p z00z_core --test genesis_tests -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_addr_rate_limit_integration -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_key_manager -- --nocapture`
- `cargo test -p z00z_wallets --release --test test_stealth_request -- --nocapture`
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump`
- `cargo run --release -p z00z_simulator --bin scenario_1 --features wallet_debug_dump`
- `cargo test --release --features test-fast --features wallet_debug_dump`

📌 The memlock blocker also requires the Miri-oriented command from the phase
plans:

- `cargo +nightly miri test -p z00z_utils --test test_os_hardening_integration`

📌 The Phase 027 time-policy closure scan must also be reproducible as one exact
repository command, not rederived from prose during review. The minimum
accepted form for this phase is:

- `rg -n '\.unix_timestamp(_millis|_micros)?\(|duration_since\(SystemTime::UNIX_EPOCH\).*as_micros\(' crates --glob 'crates/**/src/**/*.rs' --glob 'crates/**/bin/**/*.rs' --glob '!crates/z00z_crypto/tari/**' --glob '!crates/z00z_utils/src/time/**' --glob '!**/tests/**' --glob '!**/examples/**' --glob '!**/benches/**' --glob '!**/fuzz/**'`

📌 The Phase 027 release-style simulator commands above are authoritative for
this spec and must remain exact because the user requires those forms when
Scenario 1 is part of the phase gate.

## Plan-To-Proof Coverage Map

| Plan | Must Be Proven | Primary Seams | Primary Test Ownership |
| ---- | ---- | ---- | ---- |
| `027-01` | Safe callers cannot outlive the locked slice and the guard zeroizes before unlock without leaking backing addresses | `os_hardening.rs` | `test_os_hardening_integration.rs` + `os_hardening.rs` unit seam + Miri |
| `027-02` | YAML loading is bounded, layered config fails closed by default, and only `NotFound` may degrade through the explicit optional path | `yaml.rs`, `layered.rs`, config docs and examples | `test_config_integration.rs` + `config_demo.rs` |
| `027-03` | Fallible time APIs become the blessed path, every direct production `.unix_timestamp*()` caller is classified, and no security-critical caller closes on lossy wrappers | `time/traits.rs` + downstream wallet/core/storage/simulator callers | `test_time_policy_micros.rs` + wallet/core/simulator anchors |
| `027-04` | Deterministic RNG stays reproducible in approved genesis and simulator domains while unapproved production domains become harder or impossible to use | `rng/traits.rs`, `rng/deterministic.rs`, genesis and simulator callers, `z00z_crypto/src/types.rs` | `test_reproducibility.rs`, `test_genesis.rs`, simulator release gate |
| `027-05` | Persisted logs sanitize injection, rotated logs preserve severity, structured-logger serialization failures stay explicit, and file writes surface permission-copy failures | logger modules + `io/fs.rs` | `test_logger_integration.rs` + `test_io_integration.rs` |
| `027-06` | Logger macros route through an owned JSON boundary and the narrow `serde_json` exception is explicit rather than accidental | `codec/mod.rs`, `logger/macros.rs` | `test_codec_integration.rs` + `test_logger_integration.rs` |

## Critical Workflow Journeys

📌 Another engineer should treat the following as the canonical Phase 027
workflow journeys.

1. Secret-memory journey:
   mutable secret slice -> `lock_bytes_best_effort` -> guard lifetime ties to
   slice borrow -> drop zeroizes -> unlock happens last.
2. YAML-config journey:
   YAML path -> bounded file read -> UTF-8 validation -> YAML parse -> explicit
   error classification -> default or optional `LayeredConfig` path.
3. Time-policy journey:
   `TimeProvider` helper -> downstream caller classification ->
   security-critical callers move to `try_unix_timestamp*` -> remaining
   compatibility users are explicit and justified.
4. Deterministic-RNG journey:
   deterministic provider export -> build or feature guard -> approved genesis
   or simulator opt-in -> reproducible output remains stable -> non-approved
   production seam is blocked, migrated, or explicitly allowlisted.
5. Persisted logger journey:
   input message -> shared sanitization -> severity-tagged persisted line ->
   structured payload serialize or explicit `logger.serialize_error` fallback.
6. File-write journey:
   generic write helper or private atomic helper -> temp-file write -> optional
   permission copy -> explicit success or explicit failure -> documented
   durability class.
7. JSON-boundary journey:
   logger macro -> owned `z00z_utils` JSON helper or re-export -> structured
   payload string -> logger sink without direct `::serde_json::json!()` drift.

## Security And Crypto Invariants

📌 Phase 027 does not introduce new commitments, Merkle roots, or signature
schemes. The critical security invariants for this phase are utility-boundary
invariants:

- secret bytes must not be unlockable through a dangling safe-code drop path;
- secret buffers must be zeroized before unlock;
- nonce, expiry, anti-replay, and ordering flows must not depend on implicit
  zero-fallback time values;
- deterministic RNG must remain reproducibility-only and must not masquerade as
  approved unpredictable entropy;
- seeded genesis and simulator reproducibility must stay stable under the new
  guardrails;
- persisted log files must not accept escape-sequence or control-byte injection
  as trustworthy text;
- structured logger serialization failure must remain explicit through the fixed
  `logger.serialize_error` sentinel contract;
- generic writes must not silently hide permission-copy failure;
- logger macros must not bypass the owned JSON boundary.

## Required End-To-End Behaviors

| Behavior | Requirement | Primary Path | Assertions | Pass Signal | Fail Signal |
| ---- | ---- | ---- | ---- | ---- | ---- |
| Secret lock guard is lifetime-safe | `PH27-MEMLOCK` | `lock_bytes_best_effort(&mut [u8]) -> LockedBytes<'a> -> Drop` | guard type is lifetime-bound; debug output does not reveal raw addresses; empty slice remains non-panicking; Miri exercises safe-code path | tests pass and Miri does not report UB | guard can outlive slice, debug leaks raw address, or Miri exposes dangling-drop behavior |
| YAML read path is bounded and typed | `PH27-CONFIG` | `YamlConfig::from_file -> read_file_bounded -> UTF-8 -> YAML parse` | oversized input exceeds named limit by at least one byte; malformed YAML, permission denial, and missing file are distinguishable | success path preserves dotted lookup and failure matrix is explicit | YAML path still uses unbounded read or collapses failures into one generic default |
| Default layered config fails closed | `PH27-CONFIG` | `LayeredConfig` default or optional constructor split | default path rejects malformed and permission-denied YAML; only explicit optional path may downgrade `NotFound`; env override still wins | constructor behavior matches docs and examples | `new()` or equivalent still swallows malformed YAML or docs keep fail-open wording |
| Security-critical time consumers fail closed | `PH27-TIME` | `try_unix_timestamp*` in nonce, expiry, policy, ordering, and rate-limit flows | direct production callers are classified; security-critical callers use `try_*`; typed failure path is observable | no security-critical caller remains on ambiguous lossy helper | rate limits, expiry, anti-replay, or nonce derivation still accept zero-fallback behavior |
| Remaining lossy time use is explicit and non-security | `PH27-TIME` | compatibility helper or justified direct caller retained after scan | every retained caller has non-security rationale in summary and does not gate canonical ordering or rejection logic | full classification table closes Gate C | a direct caller remains unclassified or keeps security semantics under lossy behavior |
| Approved deterministic reproducibility stays stable | `PH27-RNG` | genesis or simulator deterministic provider path | same seed produces same deterministic outputs; different seed diverges; approved callers still compile under guardrail | reproducibility tests and simulator release gate stay green | guardrail breaks approved reproducibility or silently changes deterministic vectors |
| Unapproved deterministic seam is blocked or explicit | `PH27-RNG` | non-allowlisted deterministic consumer such as `z00z_crypto/src/types.rs` | seam is migrated, guarded, allowlisted with rationale, or blocked with explicit failure | no non-allowlisted direct or aliased deterministic seam remains implicit | production deterministic use still compiles or remains undocumented by summary |
| Persisted logs sanitize and preserve severity | `PH27-LOGGER` | `sanitize_message -> FileLogger/RotatingFileLogger -> disk line` | ANSI escapes and control bytes are neutralized; rotated line still includes severity marker; symlink or trusted-parent policy remains explicit | persisted output is machine-checkably sanitized and still severity-readable | dangerous bytes remain intact or rotation drops severity |
| Structured serialization failure remains explicit | `PH27-LOGGER` | `StructuredLogger::log_event -> serialization error -> fallback payload` | forced serialization failure emits payload containing `logger.serialize_error`; fallback does not look like a normal success event | regression test proves sentinel contract exactly | fallback payload silently degrades or sentinel string changes |
| Generic write failure is explicit | `PH27-IO` | `write_file` overwrite path with existing metadata | permission-copy failure surfaces to caller; stronger private path remains strongest secret path | overwrite-permission regression fails closed and semantics are documented | permission-copy failure is ignored or hidden |
| JSON boundary is owned and narrow | `PH27-JSON` | logger macros -> `codec/mod.rs` JSON surface | no direct `::serde_json::json!()` remains in logger macros; codec compatibility surface stays narrow and documented | logger integration and source-scan checks pass | logger macros bypass codec boundary or policy remains implicit |

## Critical Integration Paths

📌 Another engineer should treat these as the canonical integration paths for
Phase 027. If a new test does not anchor to one of these paths, it is probably
secondary regression coverage rather than phase-closing proof.

1. `lock_bytes_best_effort(...) -> LockedBytes<'a> -> Drop`
2. `YamlConfig::from_file(...) -> bounded read -> parse -> LayeredConfig default or optional path`
3. `TimeProvider::try_unix_timestamp* -> z00z_core/src/assets/nonce.rs`
4. `TimeProvider::* -> wallet policy, rate limit, address-manager, request-expiry, storage stamp, simulator artifact callers`
5. `DeterministicRngProvider or DeterministicRng -> Cargo guard or feature -> approved genesis and simulator callers`
6. `StructuredLogger or file logger -> sanitize_message -> persisted output`
7. `write_file or atomic_write_file_private or atomic_write_file_streaming -> permission or durability outcome`
8. `logger macros -> codec::json or codec::Value compatibility surface -> payload string`

## Scenario Oracle Rules

📌 Every scenario in this spec must have a machine-checkable pass or fail
oracle.

1. A scenario passes only when it proves both behavior and invariant.
2. A rejection scenario passes only when the rejection is explicit and no
   silent fallback or partial-success artifact remains accepted.
3. A memlock scenario passes only when the public API shape prevents safe-code
   lifetime drift and Miri-oriented validation stays clean.
4. A config scenario passes only when `NotFound` is the sole allowed downgrade
   and malformed, oversized, and permission-denied inputs remain explicit
   failures.
5. A time-policy scenario passes only when the caller classification is named,
   the retained compatibility cases are non-security, and the failure path is
   observable rather than replaced with timestamp `0`.
6. A deterministic-RNG scenario passes only when reproducibility is preserved
   for approved domains and any other deterministic seam is either blocked,
   migrated, or explicitly allowlisted.
7. A persisted-logger scenario passes only when the on-disk line is sanitized,
   severity-bearing, and the serialization-failure sentinel remains exact.
8. A file-write scenario passes only when permission-copy failure is surfaced
   to the caller and the helper's durability or secrecy class is explicit.
9. A JSON-boundary scenario passes only when the source scan confirms no direct
   `serde_json` macro route remains in logger macros.

## Test Files To Add Or Extend

### 1. Extend `crates/z00z_utils/tests/test_os_hardening_integration.rs`

📌 This file must own the executable proof for `PH27-MEMLOCK`.

Tests to implement or tighten:

- `lock_guard_binds_to_slice_borrow`
  demonstrates that the public API shape instantiates a lifetime-bound guard
  through safe code.
  Assertions: guard creation compiles for borrowed slices; no helper exposes an
  owned or detached guard path.
  Pass: the test compiles and exercises the public entrypoint only.
  Fail: the test requires private internals or a detached guard shape.
- `lock_guard_debug_redacts_backing_address`
  demonstrates that debug output is still usable without exposing raw address
  material.
  Assertions: debug output contains guard state but no raw pointer or integer
  address field.
- `lock_guard_zeroizes_before_unlock`
  demonstrates the required drop order.
  Assertions: zeroization happens before unlock path is observed or reported.
  Ownership: this ordering oracle may live in the crate-local `os_hardening.rs`
  unit seam when the public integration path does not expose unlock observation
  directly; the integration seam still owns the public API shape and Miri path.
- `lock_empty_slice_is_non_panicking`
  demonstrates that empty inputs stay best-effort and safe.
- `miri_public_guard_path`
  demonstrates that the same public safe-code path used above is the one driven
  by the Miri command.

### 2. Extend `crates/z00z_utils/tests/test_config_integration.rs`

📌 This file must own the YAML and layered-config error matrix for
`PH27-CONFIG`.

Tests to implement or tighten:

- `yaml_bounded_success_preserves_dotted_lookup`
  demonstrates successful bounded load plus normal dotted-key behavior.
- `yaml_missing_file_is_typed_not_found`
  demonstrates that missing YAML is still explicit on the raw YAML path.
- `yaml_oversized_input_is_rejected`
  demonstrates that a fixture exceeding the named YAML limit by one byte fails
  deterministically.
  Assertions: error class distinguishes oversize from parse failure.
- `yaml_permission_denied_is_not_downgraded`
  demonstrates that unreadable YAML does not silently disappear from the
  default path.
- `layered_config_default_fails_closed`
  demonstrates that the default constructor rejects malformed or unreadable
  YAML.
- `layered_config_optional_path_allows_only_not_found`
  demonstrates the one allowed downgrade.
- `env_override_survives_constructor_split`
  demonstrates that YAML hardening does not invert environment precedence.

### 3. Extend `crates/z00z_utils/examples/config_demo.rs`

📌 This example must demonstrate the successful config workflow rather than old
fail-open convenience wording.

Example behaviors to demonstrate:

- explicit fail-closed constructor for normal production use;
- explicit optional constructor for missing-file-only workflows;
- one valid YAML plus environment override example;
- one documented malformed-YAML example that states the expected failure.

Pass condition: the shortest documented path is the fail-closed constructor.

### 4. Extend `crates/z00z_utils/tests/test_time_policy_micros.rs`

📌 This file must remain the repository guardrail for `PH27-TIME`.

Tests to implement or tighten:

- `security_critical_callers_must_use_try_time`
  demonstrates that blessed nonce or anti-replay paths stay on `try_*`.
- `compatibility_helpers_are_named_and_allowlisted`
  demonstrates that any retained compatibility helper is explicit and not the
  shortest blessed path.
- `repository_scan_has_no_unclassified_direct_time_callers`
  demonstrates the phase summary closure rule.
  Assertions: scan covers production `src/**` and `bin/**` Rust files under the
  declared perimeter through the recorded exact `rg` command above; any
  retained direct caller is in the explicit allowlist with non-security
  rationale.
- `rounded_micros_and_direct_epoch_patterns_stay_forbidden`
  demonstrates that old policy regressions remain rejected.

### 5. Extend downstream wallet and storage anchors for time policy

📌 These tests must prove that `PH27-TIME` survives real consumer workflows and
not just source scans.

Files to extend:

- `crates/z00z_wallets/tests/test_addr_rate_limit_integration.rs`
  to prove rate-limit windows and purge logic do not depend on hidden zero
  fallback.
- `crates/z00z_wallets/tests/test_key_manager.rs`
  to prove cache or derivation timestamps remain explicit under failure.
- `crates/z00z_wallets/tests/test_stealth_request.rs`
  to prove expiry logic is either fail-closed or explicitly classified as a
  retained compatibility path.

If those three files cannot absorb the full scope cleanly, add one focused new
file named `crates/z00z_wallets/tests/test_time_policy_phase027.rs` and use it
only for the remaining unowned Phase 027 assertions.

### 6. Extend `crates/z00z_core/tests/genesis/test_reproducibility.rs`

📌 This file must prove the approved deterministic-RNG domain for genesis.

Tests to implement or tighten:

- `same_seed_same_genesis_output`
  demonstrates reproducibility is preserved under the guardrail.
- `different_seed_changes_genesis_output`
  demonstrates the reproducibility surface still distinguishes seeds.
- `approved_genesis_path_remains_explicit`
  demonstrates that the approved deterministic path remains clearly separate
  from secure entropy.

Assertions must cover at least canonical emitted identity, reproducible derived
randomness, or stable artifact bytes.

### 7. Extend `crates/z00z_simulator/tests/test_scenario1_unified_gate.rs`

📌 This file must own the release-style E2E proof that Phase 027 does not break
the approved simulator reproducibility domain.

Tests to implement or tighten:

- `scenario1_release_gate_survives_time_policy_change`
  demonstrates that approved non-security simulator timestamps remain workable.
- `scenario1_release_gate_survives_deterministic_guardrail`
  demonstrates that approved deterministic simulator reproducibility remains
  available.

Pass condition: both exact user-preferred release commands stay green.

### 8. Classify the non-allowlisted deterministic seam anchored by

`crates/z00z_crypto/src/types.rs`

📌 This seam must not remain implicit.

Required proof:

- either a new or extended test demonstrates the seam is migrated away from the
  deterministic contract;
- or a compile-time or feature-gated failure demonstrates the seam is blocked;
- or an explicit allowlist decision is recorded in the phase summary with a
  rationale.

Pass condition: `random_deterministic(provider: &impl z00z_utils::rng::DeterministicRng)`
is no longer an undocumented production edge.

### 9. Extend `crates/z00z_utils/tests/test_logger_integration.rs`

📌 This file must own `PH27-LOGGER` and the logger half of `PH27-JSON`.

Tests to implement or tighten:

- `persisted_log_sanitizes_escape_and_control_bytes`
  demonstrates that newline-only sanitization is no longer the full policy.
- `rotating_logger_preserves_severity_prefix`
  demonstrates that disk output keeps the severity token after rotation.
- `structured_logger_serialize_error_is_explicit`
  demonstrates the fixed `logger.serialize_error` sentinel contract.
- `logger_macros_route_through_owned_json_boundary`
  demonstrates that macro output still works after removing direct
  `::serde_json::json!()` use.
- `logger_symlink_or_parent_trust_policy_is_explicit`
  demonstrates that the current path policy is either enforced or clearly
  documented in output or code comments.

### 10. Extend `crates/z00z_utils/tests/test_io_integration.rs`

📌 This file must own `PH27-IO`.

Tests to implement or tighten:

- `write_file_propagates_permission_copy_failure`
  demonstrates that overwrite-with-metadata failure is visible to callers.
  Assertions: the failure path is reproduced deterministically in CI either by
  a stable platform-gated filesystem setup or by one crate-private
  permission-propagation seam documented in the summary.
- `private_atomic_write_remains_strongest_secret_path`
  demonstrates that the private helper still owns the strongest secrecy and
  durability contract.
- `generic_write_durability_class_is_explicit`
  demonstrates that Unix versus non-Unix behavior is documented or encoded and
  test-visible where practical.

### 11. Extend `crates/z00z_utils/tests/test_codec_integration.rs`

📌 This file must own the codec side of `PH27-JSON`.

Tests to implement or tighten:

- `json_compatibility_surface_is_intentional`
  demonstrates that `json` and `Value` are an owned compatibility policy, not
  accidental drift.
- `logger_macro_boundary_remains_narrow`
  demonstrates that the sanctioned JSON surface did not widen beyond the
  recorded exception.

## Realistic Examples To Demonstrate Successful Execution

📌 Examples are required for Phase 027 because the phase changes recommended
library usage, not only rejection behavior.

1. `crates/z00z_utils/examples/config_demo.rs`
   must show one valid YAML plus env-override success path and one explicit
   optional missing-file path.
   Demonstrates: what production config code should do now.
2. `crates/z00z_utils/examples/time_provider_demo.rs`
   must show `try_unix_timestamp*` first and any compatibility helper second.
   Demonstrates: what the blessed production time contract looks like.
3. `crates/z00z_core/bin/assets/assets_generation_cli.rs`
   must remain a realistic reproducibility example where the same seed yields
   the same generated output and a different seed changes it.
   Demonstrates: the approved deterministic genesis-tooling domain.
4. `crates/z00z_utils/tests/test_logger_integration.rs`
   must contain one realistic structured-event example and one forced
   serialization-failure example.
   Demonstrates: normal logger behavior versus explicit failure handling.

## Measurable Phase Success Conditions

📌 Phase 027 test coverage is sufficient only when all conditions below are
true.

1. Gate A is proven by `test_os_hardening_integration.rs` plus the Miri command.
  The zeroize-before-unlock ordering oracle may be satisfied by a crate-local
  `os_hardening.rs` unit seam when public integration coverage cannot observe
  unlock timing directly.
2. Gate B is proven by a four-case YAML matrix: `NotFound`, malformed,
   oversized, and permission-denied.
3. Gate C is proven by a downstream consumer table that covers all direct
   production `.unix_timestamp*()` callers in the declared perimeter.
  The proof must preserve the exact repository scan command in the summary.
4. Gate D is proven by the absence of security-critical lossy time consumers at
   phase close.
5. Gates E and E1 are proven by release-green approved deterministic domains
   plus explicit treatment of any non-allowlisted deterministic seam.
6. Gates F and G are proven by persisted logger and write-helper integration
  tests, not by docs alone, with a deterministic CI proof path for
  permission-copy failure.
7. Gate H is proven by logger-macro tests and the direct source scan that shows
   no remaining direct `serde_json` macro route in `logger/macros.rs`.
8. The exact release-style simulator commands still pass after Waves 03 and 04.

## Completion Rule For This Spec

📌 This artifact is complete when another engineer can implement Phase 027 test
coverage without guessing:

- which existing files should own the assertions;
- which success paths demonstrate the intended workflow;
- which negative paths must fail closed;
- which downstream consumers must be classified or migrated;
- which exact oracles make each scenario pass or fail.

📌 If implementation discovers a missing test seam that is not covered here,
the engineer must extend this spec or record the deviation explicitly in the
phase summary instead of silently improvising new acceptance rules.
