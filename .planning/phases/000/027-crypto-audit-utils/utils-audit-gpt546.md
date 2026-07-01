---
post_title: "Crypto Audit: z00z_utils GPT-5.4.6"
author1: "GitHub Copilot"
post_slug: "utils-audit-gpt546"
microsoft_alias: "copilot"
featured_image: "none"
categories:
  - "engineering"
tags:
  - "crypto"
  - "audit"
  - "rust"
  - "z00z_utils"
ai_note: "AI-assisted source-only audit of z00z_utils Rust implementation"
summary: "Deep cryptography-adjacent audit of z00z_utils focusing on RNG, time, codec, compression, I/O, logging, config, and OS hardening surfaces."
post_date: "2026-03-26"
---

# Crypto Audit: z00z_utils

## Executive Verdict

🚨 Verdict: `Risky but salvageable`.

🚨 Final decision: `Blocked` until the memory-locking API in
`crates/z00z_utils/src/os_hardening.rs` is made lifetime-safe. The rest of the
crate is mostly sound utility infrastructure with good separation of secure and
deterministic RNG, bounded decompression, bounded binary deserialization, and
atomic file-write helpers, but there are still medium-severity boundary issues
around config loading and fail-soft convenience APIs.

## Input Type And Scope

📌 Input type: Rust source-code audit in implementation-review mode.

📌 Scope: `crates/z00z_utils/src/**/*.rs` only.

📌 Exclusions: all non-Rust documents, all other crates, and all vendor code.

📌 Reviewed module families:

- `codec/` for serialization, trailing-data rejection, and bounded decoding
- `compression.rs` for decompression-bomb resistance and streaming helpers
- `config/` for trusted/untrusted config boundaries and fallback semantics
- `io/` for atomic write, private write, bounded read, and destructive wrappers
- `logger/` for filesystem safety and message handling
- `os_hardening.rs` for `unsafe`, memory locking, zeroization, and process hardening
- `rng/` for secure vs deterministic randomness separation
- `time/` for fail-open/fail-closed timestamp behavior

📌 Method: targeted full reads of the security-sensitive modules plus a pattern
scan across all Rust files for `unsafe`, panic-style behavior, direct
filesystem access, infallible time helpers, symlink handling, and RNG usage.

## Security Goals Extracted From Code

📌 The crate appears to aim for these security properties:

| Goal | Status | Notes |
| --- | --- | --- |
| Clear separation between production CSPRNG and deterministic test/genesis RNG | ✅ | `SecureRngProvider` vs `DeterministicRngProvider` is explicit |
| Bounded deserialization and decompression of untrusted bytes | ✅ | Bincode size limits, JSON/YAML trailing-data rejection, bounded Zstd/LZ4 decode |
| Atomic and durable writes for sensitive files | ✅ | `atomic_write_file_private` and streaming variant use temp file + rename + `sync_all` |
| Best-effort OS hardening and memory locking for secret bytes | ⚠️ | Good intent, but `LockedBytes` API is lifetime-unsound |
| Fail-closed behavior in security-sensitive helpers | ⚠️ | Mixed: some APIs are explicit `Result`, others silently fall back |
| Secret-safe logging and file permissions | ⚠️ | File loggers enforce `0600` and reject symlink leaf paths, but generic loggers do not provide redaction policy |
| One-source-of-truth wrappers around file I/O, time, config, and RNG | ✅ | Most boundary logic is centralized in this crate |

## Threat Model Summary

📌 Relevant adversaries for `z00z_utils` are not protocol attackers directly,
but boundary attackers who exploit utility misuse:

| Adversary | Capability | Relevant Surface |
| --- | --- | --- |
| Local attacker controlling files or paths | Can race path handling, plant oversized config files, or tamper with persisted state | `io/`, `logger/`, `config/` |
| Malicious input supplier | Can feed oversized compressed or serialized payloads | `codec/`, `compression/` |
| Maintainer or downstream caller | Can accidentally pick fail-soft helpers in security-sensitive code | `time/`, `config/`, `logger/` |
| Same-process misuse | Can hold guards longer than backing buffers or misuse deterministic RNG | `os_hardening.rs`, `rng/` |

📌 Trust boundaries extracted from code:

- Downstream crates are expected to treat `SystemRngProvider` as production-safe
  and deterministic providers as non-production.
- Downstream crates are expected to use `try_unix_timestamp*()` in
  security-sensitive code, not the convenience zero-on-error wrappers.
- `z00z_utils::io` is intended to be the only low-level file I/O surface.
- `os_hardening.rs` is intended to be the one-source-of-truth for memory
  locking and dump hardening.

## Critical And High Findings

### S1-01: `LockedBytes` Is Lifetime-Unsound And Can Cause UB From Safe Callers

📌 Component: `crates/z00z_utils/src/os_hardening.rs` —
`lock_bytes_best_effort()` and `impl Drop for LockedBytes`.

| Field | Content |
| --- | --- |
| Severity | S1 |
| Problem | `lock_bytes_best_effort(bytes: &mut [u8]) -> Option<LockedBytes>` returns a guard that stores only `addr: usize` and `len: usize`. The guard does not carry a lifetime tying it to the borrowed slice. In `Drop`, it reconstructs `&mut [u8]` from the raw address and length via `std::slice::from_raw_parts_mut(...)` and then zeroizes/unlocks that memory. A safe caller can let the original buffer drop before the guard drops, leaving the guard with a dangling pointer. |
| Impact | This is a Rust soundness violation at the API boundary. Safe downstream code can trigger use-after-free, memory corruption, zeroization of unrelated reallocated memory, or crashes. In a secret-handling utility crate, that also undermines the stated goal of reliable secret wiping. |
| Fix | Make the guard lifetime-bound to the borrowed memory. A safe design is `pub struct LockedBytes<'a> { ptr: NonNull<u8>, len: usize, active: bool, _marker: PhantomData<&'a mut [u8]> }` and `pub fn lock_bytes_best_effort<'a>(bytes: &'a mut [u8]) -> Option<LockedBytes<'a>>`. Also stop exposing raw addresses in `Debug`. |

📌 Confidence: High.

📌 Evidence that would change confidence: none. The unsoundness follows
directly from the public signature and the raw-pointer reconstruction in `Drop`.

## Medium Findings

### S2-01: `LayeredConfig::new()` Fails Open On YAML Errors

📌 Component: `crates/z00z_utils/src/config/layered.rs` — `LayeredConfig::new()`.

| Field | Content |
| --- | --- |
| Severity | S2 |
| Problem | `LayeredConfig::new()` loads `config.yaml` using `YamlConfig::from_file("config.yaml").ok()`. Any YAML parse error, UTF-8 issue, permission problem, or transient I/O failure is silently downgraded to `yaml: None`, and the process continues as if no YAML configuration existed. |
| Impact | Security-relevant configuration can be silently dropped. If downstream code relies on YAML for wallet paths, network policy, limits, or logging destinations, a malformed or inaccessible config file becomes a fail-open path rather than a startup error. |
| Fix | Only ignore `NotFound`; propagate every other error. For example, change `new()` to return `Result<Self, ConfigError>`, or add a dedicated constructor that is explicitly `best_effort_if_missing()`. |

📌 Confidence: High.

### S2-02: `YamlConfig::from_file()` Bypasses The Crate’s Bounded I/O Boundary

📌 Component: `crates/z00z_utils/src/config/yaml.rs` — `YamlConfig::from_file()`.

| Field | Content |
| --- | --- |
| Severity | S2 |
| Problem | `YamlConfig::from_file()` uses `std::fs::read_to_string()` directly, reading the entire file into memory without size limits. This bypasses the crate’s own bounded file-read infrastructure in `z00z_utils::io`, which otherwise exists specifically to reduce memory-exhaustion risk on untrusted inputs. |
| Impact | An oversized config file, named pipe, or unusual filesystem object can force unbounded allocation and parsing work. This is not a remote cryptographic break, but it is exactly the kind of boundary bypass that the crate otherwise tries to prevent. |
| Fix | Reuse `z00z_utils::io::read_file_bounded()` or `read_to_string()` with an explicit max-size policy, then parse YAML from the bounded buffer. Expose a max-size-aware constructor if needed. |

📌 Confidence: High.

### S2-03: Log File Symlink Protection Is Partial, Not Race-Hardened

📌 Component: `crates/z00z_utils/src/logger/file_logger.rs` and
`crates/z00z_utils/src/logger/rotating_file_logger.rs`.

| Field | Content |
| --- | --- |
| Severity | S2 |
| Problem | Both file loggers reject symlinks on the final log path via `symlink_metadata()`, which is good, but the protection is not atomic and does not cover symlinked parent directories. There is a TOCTOU window between `ensure_no_symlink(path)` and `OpenOptions::open(path)`, and the parent path is auto-created or traversed without hardening. |
| Impact | A local attacker who controls the directory tree can potentially redirect logs or abuse log creation in a location that the caller did not intend. If logs ever contain sensitive data, that becomes a confidentiality issue. |
| Fix | On Unix, prefer `openat`-style no-follow semantics or equivalent `O_NOFOLLOW` handling on the final component, and treat the log directory as a trusted precondition or validate it separately. |

📌 Confidence: Medium.

## Low And Informational Findings

### S3-01: Infallible `TimeProvider` Helpers Collapse Clock Errors To Zero

📌 Component: `crates/z00z_utils/src/time/traits.rs`.

| Field | Content |
| --- | --- |
| Severity | S3 |
| Problem | `unix_timestamp()`, `unix_timestamp_millis()`, and `unix_timestamp_micros()` call their `try_*` variants and then `unwrap_or(0)`. The docs warn against using them in security-critical paths, but the convenience surface still makes silent zero timestamps the easiest API. |
| Impact | Downstream callers can accidentally turn clock failure into sentinel `0`, which is a dangerous value for nonce derivation, expiry checks, ordering, or anti-replay logic. The crate documents the risk, but the API still encourages fail-soft usage. |
| Fix | Keep the `try_*` methods as the primary API and consider renaming the infallible variants to `*_or_zero`, or de-emphasize them in docs/examples used by security-sensitive crates. |

📌 Confidence: High.

### S3-02: `LockedBytes` Debug Output Leaks Raw Addresses

📌 Component: `crates/z00z_utils/src/os_hardening.rs` — `impl Debug for LockedBytes`.

| Field | Content |
| --- | --- |
| Severity | S3 |
| Problem | The `Debug` implementation prints `addr`, `len`, and `active`. Even if the lifetime-unsoundness is fixed, exposing raw addresses through logs or diagnostics is unnecessary and weakens ASLR hygiene. |
| Impact | Minor information leak. On its own this is not a compromise, but a secret-handling utility type should not expose memory addresses casually. |
| Fix | Redact the address entirely and log only high-level state such as `len` and `active`. |

📌 Confidence: High.

### S4-01: Structured Logging Falls Back To A Generic Event On Serialization Failure

📌 Component: `crates/z00z_utils/src/logger/structured.rs`.

📌 `StructuredLogger::log_event()` collapses serialization failure to the fixed
JSON string `{"event":"logger.serialize_error"}`. This is acceptable as a
fail-safe logging fallback, but it hides the original event type and makes audit
trails less precise.

📌 Fix: optionally include a stable event type name or a counter metric while
still avoiding raw serialization errors in logs.

## Positive Security Properties

✅ `rng/` cleanly separates secure OS randomness from deterministic RNG, and
`MockRngProvider` is compile-blocked in non-test, non-debug, non-test-feature
production builds.

✅ `codec/json.rs` and `codec/yaml.rs` reject trailing documents/data instead of
accepting a valid prefix and ignoring the rest.

✅ `codec/bincode.rs` enforces explicit size caps and rejects trailing bytes,
which is the correct shape for untrusted binary payloads.

✅ `compression.rs` implements bounded decompression and includes bomb-rejection
tests for both Zstd and LZ4.

✅ `io/fs.rs` uses temp-file-in-same-directory writes, which is the right atomic
pattern, and the private-write helpers add Unix `0600` permissions and `sync_all`
for file plus parent directory durability.

✅ File loggers sanitize newline, carriage return, and NUL characters and set
private file mode `0600` on Unix.

✅ The crate keeps `unsafe` localized to `os_hardening.rs` instead of spreading
it across generic utility modules.

## Open Ambiguities

❓ It is not visible from `z00z_utils` alone whether `lock_bytes_best_effort()` is
actually used in long-lived production secret containers or only in tightly
scoped helper code. The soundness issue exists regardless, but exploitability
depends on usage.

❓ It is unclear whether `config.yaml` is treated as a trusted local artifact or
an import boundary that may be attacker-influenced. That affects the severity of
the unbounded YAML read and fail-open layered loading.

❓ It is unclear whether downstream crates already forbid convenience
`unix_timestamp*()` helpers in consensus- or crypto-sensitive paths by policy.

## Concrete Fixes

### Fix 1: Make Memory Lock Guard Lifetime-Safe

🚩 Replace raw-address ownership-free guard design with a lifetime-bound guard.

```rust
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct LockedBytes<'a> {
    ptr: NonNull<u8>,
    len: usize,
    active: bool,
    _marker: PhantomData<&'a mut [u8]>,
}

pub fn lock_bytes_best_effort<'a>(bytes: &'a mut [u8]) -> Option<LockedBytes<'a>> {
    // lock pages, then return guard tied to `bytes`
}
```

🚩 This change is the main blocker because it removes safe-code-triggered UB.

### Fix 2: Make Layered Config Loading Fail Closed Except For Missing Files

🚩 Replace silent `.ok()` on YAML load with explicit handling:

- `NotFound` => no YAML source
- any other error => constructor error

### Fix 3: Route YAML File Reads Through Bounded I/O

🚩 Add a bounded constructor such as:

```rust
pub fn from_file_bounded(path: impl AsRef<Path>, max_bytes: u64) -> Result<Self, ConfigError>
```

🚩 Then implement `from_file()` in terms of a conservative default max size.

### Fix 4: Harden File Logger Path Opening

🚩 Treat the parent log directory as trusted or explicitly validate it.

🚩 On Unix, use no-follow path semantics for the final open where possible.

### Fix 5: Demote Or Rename Zero-On-Error Time Helpers

🚩 Keep `try_unix_timestamp*()` as the advertised API for production and
security-sensitive code.

🚩 Consider renaming convenience wrappers to make their fallback semantics
explicit.

## Test Plan

📌 Required validation before sign-off:

1. Add a compile-time or borrow-check regression test proving a `LockedBytes`
   guard cannot outlive the backing buffer once the API is fixed.
2. Run Miri against the `os_hardening` tests after the lifetime fix to validate
   that no safe-code UB remains in the guard path.
3. Add a test that `LayeredConfig::new()` (or its replacement) propagates YAML
   parse errors and permission errors but still tolerates an absent file.
4. Add bounded-config tests for oversized YAML inputs and special-file style
   reads.
5. Add log-path hardening tests that cover symlinked parent directories or at
   least document that parent directories are trusted prerequisites.
6. Add downstream policy tests or lint guidance ensuring security-critical code
   uses `try_unix_timestamp*()` instead of the zero-on-error wrappers.

## Confidence

📌 S1 lifetime-unsoundness finding: High confidence.

📌 S2 fail-open layered config finding: High confidence.

📌 S2 unbounded YAML file read finding: High confidence.

📌 S2 log-path race/symlink-hardening gap: Medium confidence because exact
impact depends on local attacker control over the directory tree.

📌 S3 zero-on-error time helpers: High confidence on the API behavior, medium
confidence on real exploitability without a downstream call-path review.

## Final Decision

🚨 `Blocked`.

🚨 Owner actions required:

- `z00z_utils` maintainer: fix `LockedBytes` soundness and remove raw-pointer
  lifetime escape.
- `z00z_utils` maintainer: make layered config loading fail closed for malformed
  or unreadable YAML.
- `z00z_utils` maintainer: route YAML file loading through the bounded I/O
  boundary already present in the crate.

🚨 After those items are resolved, the crate is likely to move to
`Safe enough`, because the remaining issues are hardening and misuse-resistance
concerns rather than structural cryptographic failures.
