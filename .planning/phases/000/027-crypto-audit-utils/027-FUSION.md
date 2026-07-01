---
post_title: "027 Utils Audit Fusion"
author1: "GitHub Copilot"
post_slug: "027-utils-audit-fusion"
microsoft_alias: "copilot"
featured_image: "none"
categories:
  - "engineering"
tags:
  - "crypto"
  - "audit"
  - "rust"
  - "z00z_utils"
  - "fusion"
ai_note: "AI-assisted fusion of five source-only audit reports into one canonical markdown document."
summary: "Canonical fused audit for z00z_utils covering shared findings, disputed findings, positive controls, and a prioritized remediation path."
post_date: "2026-03-26"
---

<!-- markdownlint-disable MD041 -->

## Fusion Intent

📌 This document fuses five independent `z00z_utils` audit drafts into one
canonical reading surface organized by topic instead of source-file order.

📌 Input set:

- `.planning/phases/027-crypto-audit-utils/utils-audit-glm5.md`
- `.planning/phases/027-crypto-audit-utils/utils-audit-gpt546.md`
- `.planning/phases/027-crypto-audit-utils/utils-audit-m27.md`
- `.planning/phases/027-crypto-audit-utils/utils-audit-mimov2.md`
- `.planning/phases/027-crypto-audit-utils/utils-audit-sonet46.md`

📌 Full section-level coverage, provision coverage, duplicate handling, and
conflict tracking are recorded in `FUSION.audit.md`.

## [UF-01] Canonical Verdict

🚨 Canonical verdict: `Risky but salvageable`.

🚨 Canonical release decision: `Blocked for security sign-off` until the
`LockedBytes` memory-lock guard in `os_hardening.rs` is made lifetime-safe and
the YAML/config boundary stops failing open.

📌 Why this is the fused outcome:

- Two reports classify the crate as broadly safe enough or execution-ready.
- Three reports raise stronger boundary concerns around `LockedBytes`, config
  loading, deterministic RNG misuse risk, and logger/file hardening.
- The highest-confidence multi-report issue is not primitive cryptography, but a
  Rust soundness and boundary-enforcement failure in secret handling.

📌 The fusion therefore keeps the strongest defensible position: the crate has a
solid foundation, but it should not be treated as fully signed off while a
safe-code-triggered memory-unsound API and several fail-soft boundary helpers
remain in the public surface.

## [UF-02] Scope And Threat Model

📌 All five source reports agree that `z00z_utils` is not a cryptographic
primitive crate. It is the utility and abstraction layer that feeds the rest of
the workspace with RNG, time, file I/O, config, logging, codec, compression,
and process-hardening behavior.

📌 The main security goals extracted across the source reports are:

- keep production randomness separate from deterministic test or genesis RNG
- avoid secret leakage through debug output, logs, file permissions, or swap
- provide bounded parsing and bounded decompression for untrusted bytes
- preserve atomicity and durability for sensitive writes
- centralize security-relevant wrappers so downstream crates do not bypass them
- avoid silent fallback behavior in clock, config, and filesystem helpers when
  those helpers are used in security-sensitive flows

📌 The shared adversary model is mostly local or boundary-driven rather than
protocol-driven:

- a local attacker controlling files, directories, symlinks, or log paths
- an attacker supplying malformed compressed or serialized payloads
- a maintainer or downstream caller accidentally choosing convenience helpers in
  a security-sensitive path
- a process-local misuse case where deterministic RNG or memory-locking helpers
  are used outside their intended contract

📌 The fused trust model is also consistent:

- `SystemRngProvider` is expected to be production-safe
- deterministic RNG providers are expected to be non-production tools
- `z00z_utils::io` is expected to be the one-source-of-truth file boundary
- `os_hardening.rs` is expected to be the one-source-of-truth secret-memory and
  dump-hardening boundary

## [UF-03] Canonical Finding Map

### [UF-03.1] `LockedBytes` soundness is the strongest confirmed blocker

🚨 The most serious fused finding is the `LockedBytes` API shape in
`os_hardening.rs`.

📌 Strongest shared interpretation:

- one report classifies it as a true S1 blocker because the guard stores only an
  address and length and reconstructs a mutable slice in `Drop`
- one report classifies the same issue as medium severity but still recommends a
  lifetime-parameterized guard as the correct fix
- another report retracts a separate false positive about `munlock`, but that
  retraction does not invalidate the lifetime-unsoundness finding
- one additional report independently flags the same type as leaking raw memory
  addresses in debug output

📌 Canonical finding:

- the current public shape does not encode the borrow lifetime of the backing
  slice into the returned guard type
- that allows safe callers to create a dangling-pointer drop scenario if the
  backing buffer drops before the guard drops
- the existing `Drop` implementation may then zeroize or unlock unrelated memory

📌 Canonical remediation:

- solve this inside `crates/z00z_utils/src/os_hardening.rs`; no new crate is
  required because the current codebase already has `zeroize`, `rustix`, and
  `windows-sys` for the OS-facing parts
- change the guard shape to `LockedBytes<'a>` and change
  `lock_bytes_best_effort` to return `Option<LockedBytes<'a>>` bound to the
  backing slice lifetime
- store `NonNull<u8>` plus `PhantomData<&'a mut [u8]>` instead of a plain
  address integer so the type system, not just comments, enforces the drop
  ordering contract
- keep the existing zeroize-before-unlock order, but remove raw addresses from
  `Debug`
- validate with unit tests plus Miri so the fix proves that a safe caller can
  no longer produce a dangling-pointer drop path

### [UF-03.2] YAML and layered-config handling are too fail-soft

📌 This is the clearest multi-report medium-severity cluster after the
`LockedBytes` issue.

📌 Shared findings:

- `YamlConfig::from_file()` reads with `std::fs::read_to_string()` instead of
  the crate's bounded I/O layer
- one report additionally shows that `LayeredConfig::new()` silently downgrades
  YAML load failure to `None` by calling `.ok()`
- one report recommends adding bounded YAML deserialization explicitly, not just
  bounded file reads

📌 Canonical finding:

- the crate already has a bounded-I/O design, but the YAML config path bypasses
  it
- malformed, oversized, or unreadable YAML can therefore become either a memory
  pressure issue or a fail-open configuration drop

📌 Canonical remediation:

- reuse the existing bounded-read path in `crates/z00z_utils/src/io/fs.rs`
  instead of adding a new parsing stack; the missing piece is policy wiring,
  not a missing dependency
- introduce an explicit config-size constant and load bytes through
  `read_file_bounded`, then perform UTF-8 validation before calling
  `serde_yml`
- treat `NotFound` as the only allowed best-effort downgrade and surface every
  other YAML read or parse error
- replace `LayeredConfig::new()` fail-open behavior with either
  `Result<Self, ConfigError>` or a clearly named best-effort constructor so the
  permissive path is never the implicit default
- verify with oversized-file, malformed-YAML, permission-denied, and
  missing-file tests

### [UF-03.3] Time helper fallback semantics are unsafe by default

📌 Two reports converge on the same theme, but with different severity:

- one labels zero-on-error timestamp helpers as a medium-severity production
  risk
- one labels them low severity but still calls them a dangerous convenience
  surface

📌 Canonical finding:

- `unix_timestamp()`, `unix_timestamp_millis()`, and
  `unix_timestamp_micros()` collapse clock failure to `0`
- the API docs warn callers away from that behavior, but the shortest and
  easiest method names still produce fail-soft output
- that is exactly the wrong default if downstream code uses timestamps for
  nonce generation, ordering, expiry, or anti-replay decisions

📌 Related lower-priority time notes preserved from the source set:

- one report notes the `u64::MAX` overflow sentinel for microsecond timestamps
- one report flags local-time formatting as a logging-forensics inconsistency

📌 Canonical remediation:

- keep `try_unix_timestamp*()` as the production API because the crate already
  has the correct fallible surface; this is an API-default problem, not a
  missing capability problem
- rename the lossy wrappers to something explicit such as
  `unix_timestamp_lossy_or_zero()` or deprecate them from security-sensitive
  examples
- audit actual call sites in `z00z_core`, `z00z_wallets`, and related crates so
  severity is based on usage, not only on API shape
- add tests and guidance that any nonce, expiry, ordering, or anti-replay code
  must use the fallible path

### [UF-03.4] File-write semantics need clearer durability and permission rules

📌 The source reports agree that the private atomic write path is strong, but
they disagree on how close the generic helpers are to that standard.

📌 Shared positives:

- temp-file-in-same-directory writes are correct
- `atomic_write_file_private` is the right canonical path for secret material
- parent-directory sync and `0600` enforcement on Unix are real strengths

📌 Preserved concerns:

- one report flags that `write_file()` can silently discard permission-copy
  errors when overwriting an existing file
- one report flags that generic `write_file()` lacks `sync_all()` durability
  parity with the private helper
- one report flags Windows or non-Unix `fsync` gaps in
  `atomic_write_file_private` and `atomic_write_file_streaming`

📌 Canonical finding:

- the private path is good, but the generic write helpers still leave room for
  silent weakening of durability or permission expectations
- downstream crates should not assume all atomic-write helpers are equally safe
  for secrets or crash-sensitive state

📌 Canonical remediation:

- treat `atomic_write_file_private` as the canonical secret-bearing write path
  because it already provides the strongest semantics in the crate today
- make `write_file()` propagate permission-copy failures instead of silently
  swallowing them
- either extend generic writes with an explicit durable variant or document that
  generic writes are not the crash-safe, secret-safe contract
- close the non-Unix durability gap in the existing helpers or document it as
  best-effort only; current `tempfile` plus existing file-sync logic are enough
  to implement this without a new crate
- verify with overwrite-permission, crash-safety, and platform-specific
  durability tests

### [UF-03.5] Logger hardening is good, but not complete

📌 Multiple reports point to the same logger cluster:

- log file symlink rejection exists and is a real positive control
- file mode `0600` on Unix is a real positive control
- newline, carriage return, and NUL sanitization exist

📌 The preserved weaknesses are:

- `sanitize_message` does not strip ANSI escape sequences or broader control
  characters
- rotating log output drops the log level in at least one implementation path
- one report says symlink hardening only protects the final component and leaves
  a TOCTOU window or trusted-parent assumption
- structured logging falls back to a generic serialization-error event, which is
  safe but weakens audit precision

📌 Canonical remediation:

- keep the existing newline, carriage-return, and NUL sanitization, but extend
  it to cover ANSI escape sequences and broader control-byte injection
- restore log-level prefixes in rotating log output so operational severity does
  not disappear on disk
- on Unix, prefer using the already-present `rustix` dependency if the project
  decides to harden final-component open semantics beyond a best-effort
  symlink check; otherwise document the trusted-parent-directory assumption
- if the team wants full ANSI coverage rather than a minimal local filter, the
  only external crate that looks justified is `strip-ansi-escapes`; everything
  else in this logger cluster is solvable inside the current codebase
- verify with ESC-sequence, control-byte, symlink, and rotation-format tests

### [UF-03.6] Deterministic RNG separation is conceptually strong, but still easy to misuse

📌 The reports agree on the strongest positive point: `SystemRngProvider` and the
deterministic providers are conceptually separate, and `MockRngProvider` already
has a compile-time guard for production builds.

📌 The preserved concerns are more subtle:

- one report argues `DeterministicRngProvider` should also have a production
  guard or dedicated feature gate
- one report argues the trait bound using `CryptoRng` on a deterministic provider
  is semantically misleading even if the underlying algorithm is sound
- one report notes `MockRngProvider::rng()` restarts the same sequence on every
  call and is therefore easy to misuse in tests
- one report notes `with_u64_seed()` has only `2^64` effective entropy and
  should stay clearly test-only
- one report notes `MockRngProvider` and `DeterministicRngProvider` use different
  backing algorithms, which weakens reproducibility guarantees across upgrades

📌 Canonical finding:

- production CSPRNG separation is a strength
- deterministic helper ergonomics still make accidental misuse plausible enough
  that extra guardrails are justified

📌 Canonical remediation:

- reuse the existing `MockRngProvider` compile-time guard pattern or feature-gate
  deterministic providers behind an explicit genesis/test capability
- tighten naming and docs so deterministic output is never mistaken for
  unpredictable output even if the underlying algorithm is cryptographically
  strong as a stream cipher
- consider unifying `MockRngProvider` and `DeterministicRngProvider` on one
  deterministic backend so reproducibility rules stay stable across upgrades
- keep this local to the current RNG abstraction; no new crate is required
  because `rand`, `rand_chacha`, and `sha2` already cover the implementation
  space

### [UF-03.7] Abstraction-boundary drift exists around JSON and codec exports

📌 This theme comes mostly from one source report, but it is concrete enough to
preserve.

📌 Preserved issues:

- logger macros call `serde_json::json!()` directly
- `codec/mod.rs` re-exports `serde_json::{json, Value}` directly

📌 Canonical interpretation:

- this is not a direct cryptographic break
- it is a real pressure point against the crate's stated abstraction goals and
  the repository's one-source-of-truth rules

📌 Canonical remediation:

- make an explicit repository policy choice rather than leaving the drift
  implicit
- option A: document `serde_json::json!()` as a narrow macro-level exception
  while removing or reducing the public re-export pressure on downstream crates
- option B: remove the public `json` and `Value` re-exports and force
  structured payload creation back through `JsonCodec` or an internal wrapper
- no new crate is needed; this is an architecture decision inside the current
  abstraction boundary

### [UF-03.8] Lower-priority platform and dependency notes are worth keeping

📌 These issues are not blockers, but they are retained because they add unique
engineering value:

- LZ4 magic check is endian-fragile in theory, though irrelevant on current
  target architectures
- `apply_best_effort()` being opt-in should be documented clearly for consumers
- `serde_yml` maintenance status should be tracked
- `chrono` and `erased_serde` are dependency-surface notes rather than defects
- the fixed 1 MB, 10 MB, and 100 MB bincode limits are restrictive by design and
  may be a usability tradeoff rather than a security bug

## [UF-04] Confirmed Positive Controls

✅ The strongest area of agreement across all five source reports is the crate's
baseline defensive design.

✅ Cross-report positives preserved in the fusion:

- secure and deterministic RNG are explicitly separated
- `MockRngProvider` has a real production-build compile guard
- RNG seed debug output is redacted
- atomic write paths use temp-file plus rename in the same directory
- `atomic_write_file_private` enforces `0600` on Unix and syncs file plus parent
  directory
- JSON, YAML, and bincode codecs reject trailing content instead of silently
  accepting a valid prefix
- decompression is bounded and includes bomb-detection behavior
- symlink leaf checks exist on sensitive log and file paths
- `unsafe` is localized mostly to `os_hardening.rs` instead of being spread
  across utility modules
- zeroization-on-drop is already present on RNG seed fields and memory-lock code

📌 The fusion keeps these strengths explicit because they explain why the crate
is salvageable even though the final sign-off remains blocked.

## [UF-05] Open Conflicts And Ambiguities

📌 The source reports do not fully agree on severity, sign-off status, or which
items count as hard blockers.

📌 Main preserved conflicts:

- some reports say `Safe enough` or `Execution-ready`; others say `Blocked`
- one report retracts a false positive about `munlock` while another correctly
  escalates the different lifetime-unsoundness issue
- one report treats deterministic-provider production gating as an S1 issue,
  while others do not raise it at all
- one report treats direct `serde_json` use in macros as an architectural issue
  serious enough for S1, while others treat it as a non-security abstraction
  decision or do not mention it
- the severity of zero-on-error time helpers ranges from low to medium across
  the reports

📌 Main preserved ambiguities:

- whether `lock_bytes_best_effort()` is heavily used in long-lived secret paths
- whether `config.yaml` is always trusted local input or ever an attacker-shaped
  boundary
- whether deterministic RNG providers are actually reachable in production code
- whether the `serde_json` macro and re-export usage is a deliberate policy
  exception

📌 The fusion resolves these disagreements conservatively by elevating only the
best-supported blocker and keeping the rest as explicit conflict or ambiguity
items rather than forcing false consensus.

## [UF-06] Prioritized Remediation Roadmap

📌 The original fusion had enough findings to justify a verdict, but not enough
implementation detail to be execution-ready. The roadmap below upgrades the
document into a solution document by stating, for each major issue, where the
fix comes from and what concrete path should be used.

### [UF-06.1] Solution Source Matrix

| Issue | Recommended source of solution | New crate required? | Execution path |
| --- | --- | --- | --- |
| `LockedBytes` lifetime unsoundness | Existing codebase plus current deps (`zeroize`, `rustix`, `windows-sys`) | No | Rebuild the guard type in `os_hardening.rs` around lifetime-bound ownership semantics. |
| YAML bounded load and fail-open layered config | Existing codebase (`read_file_bounded`, current config modules) | No | Rewire `YamlConfig` and `LayeredConfig` to use bounded reads and explicit error policy. |
| Lossy zero-fallback time helpers | Existing codebase (`try_unix_timestamp*`) | No | Promote fallible APIs and demote lossy wrappers. |
| Generic write durability and permission semantics | Existing codebase (`atomic_write_file_private`, current temp-file logic) | No | Separate generic writes from secret-safe durable writes and stop swallowing permission failures. |
| Logger ANSI/control-byte sanitization | Existing codebase for minimum fix; optional crates.io for full ANSI stripping | Optional: `strip-ansi-escapes` | Patch format behavior locally and only add one focused dependency if full ANSI parsing is required. |
| Deterministic RNG misuse resistance | Existing codebase and current RNG deps | No | Extend current guardrails, naming, and backend policy without changing the primitive set. |
| `serde_json` abstraction drift | Existing codebase and current repo policy | No | Make the exception-or-removal decision explicit and enforce it in public API. |

### [UF-06.2] Execution-Ready Changes

📌 Priority 1, sign-off blockers:

1. Make `LockedBytes` lifetime-safe in `crates/z00z_utils/src/os_hardening.rs`.
  Use `NonNull<u8>`, `PhantomData<&'a mut [u8]>`, and the existing zeroize and
  unlock flow. Acceptance gate: compile-time lifetime enforcement plus Miri.
2. Route YAML config loading through bounded I/O in
  `crates/z00z_utils/src/config/yaml.rs`.
  Use the existing bounded read layer with a dedicated config size constant.
  Acceptance gate: oversized YAML fails before parse.
3. Stop silent fail-open YAML downgrade in
  `crates/z00z_utils/src/config/layered.rs`.
  Only `NotFound` may degrade to `None`; every other error must surface.
  Acceptance gate: malformed YAML and permission failure are no longer ignored.

📌 Priority 2, hardening issues with real operational impact:

1. Make `write_file()` propagate permission-copy failure and clearly document
  that secret-bearing writes must use `atomic_write_file_private`.
2. Close or document the non-Unix durability gap in the existing write helpers.
3. Rename, deprecate, or demote the zero-on-error time helpers so the lossy
  behavior is visible at the call site.
4. Strip ANSI/control-byte sequences from logger messages and restore log-level
  prefixes in rotating files.
5. Decide whether the project wants the minimal in-tree sanitizer or the
  stronger crates.io-backed ANSI stripper.

📌 Priority 3, misuse-resistance and architecture hygiene:

1. Reuse the `MockRngProvider` guard pattern or feature-gate deterministic
  providers explicitly for test/genesis-only use.
2. Revisit `CryptoRng` semantics on deterministic traits and align the docs with
  the intended misuse boundary.
3. Decide whether direct `serde_json` macro and re-export usage is a deliberate
  exception or a boundary violation to remove.
4. Consider stabilizing the deterministic RNG backend used across helper types.
5. Track `serde_yml` maintenance and other dependency-surface notes separately
  from the sign-off blockers.

### [UF-06.3] External Crate Decision

📌 The crypto-architect follow-up does not justify a broad dependency expansion.

📌 The only external crate that currently looks justified is
`strip-ansi-escapes`, and only for the logger sanitization problem if the team
wants complete ANSI-sequence coverage instead of a smaller local filter.

📌 Every other major issue in this fusion is best solved inside the current
codebase using already-present abstractions and dependencies.

## [UF-07] Consolidated Test Plan

📌 The fused reports converge on this minimum test set:

- prove a `LockedBytes` guard cannot outlive its backing buffer after the API is
  fixed
- run Miri or equivalent UB-oriented validation against `os_hardening` after the
  lifetime fix
- verify oversized YAML config input is rejected by the bounded path
- verify layered config loading tolerates missing YAML but does not ignore parse
  or permission errors
- verify file logger and rotating logger reject symlink targets in the supported
  threat model
- verify `sanitize_message` strips ANSI and control-byte injection
- verify the rotating logger output contains the log level
- verify timestamp helper policy in downstream security-sensitive crates
- verify non-Unix atomic-write semantics or document the gap explicitly

📌 Lower-priority follow-up tests preserved from the source set:

- empty-input and oversize-input codec tests
- decompression bomb tests for Zstd and LZ4
- deterministic RNG reproducibility vectors
- `unix_timestamp_micros()` overflow/sentinel tests

## [UF-08] Confidence And Sign-Off

📌 Highest-confidence findings:

- `LockedBytes` lifetime unsoundness
- `YamlConfig::from_file()` bypass of bounded I/O
- fail-open YAML downgrade in layered config loading
- zero-on-error timestamp helper behavior

📌 Medium-confidence findings that remain worth keeping:

- log path race hardening beyond final symlink checks
- deterministic-provider production-gating requirement
- the security weight of direct `serde_json` use in macros and re-exports

🚨 Fused sign-off decision: `Blocked pending boundary fixes`.

📌 This does not mean the crate is broadly broken. It means the strongest
defensible reading of the combined source set is:

- the foundational design is sound
- the public utility surface is still too permissive in a few places
- one secret-memory API shape is unsound enough to stop a clean sign-off

📌 The source drafts should therefore be treated as superseded by this fused
document for reading purposes, but not as deletion-safe replacements until the
audit artifact and external verification gate are accepted.
