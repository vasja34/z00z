# Crypto Architect Audit Report — `z00z_utils` Crate

**Date:** 2026-03-26  
**Auditor:** GitHub Copilot (Crypto Architect skill)  
**Scope:** `crates/z00z_utils/src/**/*.rs` — all Rust implementation files  
**Exclusions:** `z00z_crypto/tari/` vendor code, non-Rust documentation  
**Model:** Xiaomi MiMo-V2-Pro  

---

## Executive Verdict

**`Risky but salvageable`** — 2 S1 findings remain after retraction of 1 false positive; concrete fixes exist and are documented below.

The crate is a well-structured utility abstraction layer that largely follows the ONE SOURCE OF TRUTH principle. However, several cryptographic and security-relevant issues require remediation before production deployment.

---

## 1. Input Type & Scope

**Classification:** Implementation review mode — pure Rust source code across 48 files.

**Modules audited:**
- `rng/` — RNG provider abstraction (7 files)
- `codec/` — Serialization/deserialization (6 files)
- `io/` — File I/O with atomic writes (3 files)
- `time/` — Time provider abstraction (7 files)
- `logger/` — Logging abstraction (11 files)
- `metrics/` — Metrics sink (3 files)
- `config/` — Configuration sources (5 files)
- `compression.rs` — Zstd/LZ4 compression
- `os_hardening.rs` — OS-level hardening (mlock, prctl)
- `lib.rs` — Crate root and prelude

---

## 2. Security Goals

| Goal | Status | Notes |
|------|--------|-------|
| Secret lifecycle (zeroization) | ⚠️ Partial | RNG seeds zeroized; `LockedBytes` has Drop issue |
| Constant-time discipline | ✅ N/A | No secret comparison in this crate |
| Randomness quality | ✅ Good | `OsRng` for production, ChaCha20 for deterministic |
| Atomic file writes | ✅ Good | tempfile + rename pattern |
| Decompression bomb protection | ✅ Good | Bounded decompression with size limits |
| Symlink attack mitigation | ✅ Good | `ensure_no_symlink` in loggers and I/O |
| File permission hardening | ✅ Good | `0o600` on Unix for log files |
| Secret redaction in Debug | ✅ Good | RNG seeds show `<redacted>` |
| Compile-time production guard | ⚠️ Partial | `MockRngProvider` has guard; `DeterministicRngProvider` does not |

---

## 3. Threat Model Summary

**Assets protected:**
- RNG seeds (deterministic providers)
- Log files (sensitive operational data)
- Configuration values (may contain secrets)
- Compressed wallet backups

**Adversaries:**
- Local attacker with filesystem access (symlink, TOCTOU)
- Attacker supplying malformed compressed data (decompression bomb)
- Attacker observing log output (secret leakage)
- Attacker exploiting deterministic RNG in production (predictability)

**Trust boundaries:**
- Filesystem (atomic writes, permissions, symlink checks)
- OS entropy source (`OsRng`)
- Serialization layer (trailing bytes, size limits)

---

## 4. Critical & High Findings (S0/S1)

### ~~S1: `LockedBytes::drop` Does Not Call `munlock`~~ — RETRACTED

**Component:** `os_hardening.rs` — `LockedBytes` struct  
**Status:** RETRACTED — initial read was truncated at line 200; full source (lines 200–230) confirms `Drop` is correctly implemented.

The `Drop` implementation:
- Calls `bytes.zeroize()` to clear locked memory before unlock.
- Calls `munlock` on Unix and `VirtualUnlock` on Windows when `self.active == true`.
- Handles errors silently (best-effort pattern, consistent with design).
- Sets `self.active = false` after cleanup.

**Verdict:** ✅ Correct implementation. No remediation needed.

---

### S1: `DeterministicRngProvider` Has No Compile-Time Production Guard

**Component:** `rng/deterministic.rs`  
**Problem:** `MockRngProvider` has a `compile_error!` guard preventing production compilation, but `DeterministicRngProvider` does not. While `DeterministicRngProvider` is intended for genesis/testing, nothing prevents its accidental use in production code paths for nonce or key generation.  
**Impact:** Predictable randomness in production — key extraction, nonce reuse.  
**Fix:** Add a feature gate or compile-time guard:

```rust
#[cfg(all(
    not(test),
    not(debug_assertions),
    not(feature = "test-utils"),
    not(feature = "test-fast"),
    not(feature = "deterministic-rng"),
))]
compile_error!(
    "DeterministicRngProvider MUST NOT be compiled in production builds. \
     Enable feature 'deterministic-rng' for genesis-only use."
);
```

Alternatively, gate the entire module behind `#[cfg(any(test, feature = "test-utils", feature = "test-fast", feature = "deterministic-rng"))]`.

**Confidence:** High — direct observation of missing guard.

---

### S1: `logger/macros.rs` Uses `serde_json` Directly — Violates ONE SOURCE OF TRUTH

**Component:** `logger/macros.rs`  
**Problem:** The logging macros (`log_info!`, `log_warn!`, etc.) directly invoke `::serde_json::json!()`. This bypasses the `z00z_utils::codec` abstraction layer, violating the ONE SOURCE OF TRUTH principle stated in the project's copilot instructions. If `serde_json` is ever replaced or feature-gated, these macros break silently.  
**Impact:** Architectural violation; maintenance risk; potential for divergent serialization behavior.  
**Fix:** Replace `::serde_json::json!` with a macro-local helper that delegates to `crate::codec::JsonCodec`, or document the exception with a rationale (macros cannot use trait objects at compile time). If the exception is intentional, add a `// ONE_SOURCE_OF_TRUTH exception: macros require compile-time JSON construction` comment.

**Confidence:** Medium — this may be an intentional pragmatic choice for macro ergonomics.

---

## 5. Medium & Low Findings (S2/S3/S4)

### S2: `codec/mod.rs` Re-exports `serde_json::{json, Value}` Directly

**Component:** `codec/mod.rs` line 12  
**Problem:** `pub use serde_json::{json, Value};` leaks the `serde_json` dependency into the public API. Downstream crates can bypass the `Codec` abstraction entirely.  
**Impact:** Erosion of the abstraction boundary; harder to swap serialization backends.  
**Fix:** Remove the re-export or wrap `Value` in a newtype (e.g., `pub struct JsonValue(pub serde_json::Value)`). At minimum, add a deprecation note.

**Confidence:** Medium — may be intentional for ergonomics.

---

### S2: `YamlConfig::from_file` Uses `std::fs::read_to_string` Directly

**Component:** `config/yaml.rs` line 24  
**Problem:** `std::fs::read_to_string(path.as_ref())` bypasses the `z00z_utils::io` abstraction (no bounded read, no atomic semantics). A malicious YAML file could exhaust memory.  
**Impact:** Resource exhaustion via unbounded file read.  
**Fix:** Use `crate::io::read_file_bounded` with a reasonable limit (e.g., 1 MB for config files):

```rust
pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
    let bytes = crate::io::read_file_bounded(path, 1024 * 1024)
        .map_err(|e| ConfigError::Io(e.into()))?;
    let content = String::from_utf8(bytes)
        .map_err(|e| ConfigError::Yaml(format!("invalid UTF-8: {e}")))?;
    let data = serde_yml::from_str(&content).map_err(|e| ConfigError::Yaml(e.to_string()))?;
    Ok(Self { data })
}
```

**Confidence:** High — direct observation.

---

### S2: `RotatingFileLogger::write_log` Discards Level Information

**Component:** `logger/rotating_file_logger.rs` — `write_log` method  
**Problem:** The `write_log` method receives a `level` parameter but immediately discards it (`let _ = level;`). Log lines written by `RotatingFileLogger` contain only the message, not the log level.  
**Impact:** Operational blindspot — cannot distinguish ERROR from INFO in rotated logs.  
**Fix:** Include the level in the formatted log line:

```rust
fn write_log(&self, level: &str, message: &str) {
    let msg = sanitize_message(message);
    let log_line = format!("[{}] {}\n", level, msg);
    // ... rest unchanged
}
```

**Confidence:** High — direct observation of `let _ = level;`.

---

### S3: `BincodeCodec::deserialize_bounded` Only Supports Three Fixed Sizes

**Component:** `codec/bincode.rs` — `deserialize_bounded` method  
**Problem:** The match on `limit` only handles 1 MB, 10 MB, and 100 MB. Any other value returns an error. This is inflexible and surprising for callers.  
**Impact:** API usability issue; callers must know the exact allowed values.  
**Fix:** Use a single `with_limit::<N>()` call with the actual `max_bytes` value, or use `bincode::config::standard().with_limit::<{ usize::MAX }>()` and rely on the pre-check for enforcement.

**Confidence:** Low — this is a design choice, not a security flaw.

---

### S3: `MockRngProvider::rng()` Returns Same Sequence Per Call

**Component:** `rng/mock.rs`  
**Problem:** Each call to `rng()` creates a new `StdRng` from the same seed. The documentation warns about this, but the API is still error-prone. A caller who does `provider.rng().next_u64()` twice gets the same value.  
**Impact:** Test flakiness or false positives if used incorrectly.  
**Fix:** Consider adding a counter-based seed derivation (e.g., `seed = H(seed || counter)`) so each `rng()` call returns a different but deterministic sequence. This would be a breaking API change.

**Confidence:** Low — documented behavior, mitigated by warnings.

---

### S4: `logger/structured.rs` Uses `erased_serde` for Dynamic Serialization

**Component:** `logger/structured.rs`  
**Problem:** The `LogEvent` trait uses `erased_serde::serialize_trait_object!` for dynamic dispatch. This is correct but adds a dependency. If `erased_serde` has a vulnerability, it affects the logging layer.  
**Impact:** Transitive dependency risk.  
**Fix:** No action needed — `erased_serde` is well-maintained. Document the dependency in the crate's `Cargo.toml` comments.

**Confidence:** Low — standard practice.

---

### S4: `time/format.rs` Uses `chrono` for Timestamp Formatting

**Component:** `time/format.rs`  
**Problem:** The `chrono` crate has had historical CVEs (e.g., CVE-2020-26235, CVE-2020-35872). While the current version may be patched, the dependency adds attack surface for a formatting-only use case.  
**Impact:** Transitive dependency risk.  
**Fix:** Consider replacing with `time` crate or manual formatting if `chrono` is not used elsewhere. Low priority.

**Confidence:** Low — `chrono` is widely used and currently maintained.

---

## 6. Open Ambiguities

| # | Ambiguity | Impact on Confidence | Evidence Needed |
|---|-----------|---------------------|-----------------|
| 1 | Whether `DeterministicRngProvider` is used in production code paths | Determines severity of S1 | `grep -rn "DeterministicRngProvider" crates/z00z_core/ crates/z00z_wallets/` |
| 2 | Whether `serde_json` re-export is intentional for macro ergonomics | Determines severity of S2 | Project design discussion or ADR |
| 3 | `YamlConfig` usage scope — is it ever used with untrusted input? | Determines severity of S2 | `grep -rn "YamlConfig::from_file" crates/` |

---

## 7. Concrete Fixes (Summary)

| # | Severity | File | Fix |
|---|----------|------|-----|
| 1 | S1 | `rng/deterministic.rs` | Add `compile_error!` guard or feature gate for production builds |
| 2 | S1 | `logger/macros.rs` | Document ONE_SOURCE_OF_TRUTH exception or refactor to use codec abstraction |
| 3 | S2 | `codec/mod.rs` | Remove or deprecate `serde_json::{json, Value}` re-export |
| 4 | S2 | `config/yaml.rs` | Use `read_file_bounded` instead of `std::fs::read_to_string` |
| 5 | S2 | `logger/rotating_file_logger.rs` | Include log level in formatted output |
| 6 | S3 | `codec/bincode.rs` | Generalize `deserialize_bounded` to accept arbitrary limits |

---

## 8. Implementation Guidance

### What Is Done Well

- **Atomic writes:** `tempfile::NamedTempFile` + `persist()` prevents partial writes and TOCTOU races.
- **Symlink checks:** `ensure_no_symlink` in `FileLogger`, `RotatingFileLogger`, and compression functions.
- **File permissions:** `0o600` enforced on Unix for log files.
- **Bounded reads:** `read_file_bounded` prevents memory exhaustion from large files.
- **Bounded decompression:** `zstd_decompress_bounded` and `lz4_decompress_bounded` prevent decompression bombs.
- **Secret redaction:** `Debug` impls for RNG providers redact seeds.
- **Trait-based abstraction:** Clean separation of `SecureRngProvider` vs `DeterministicRngProvider`.
- **Compile-time guard:** `MockRngProvider` has `compile_error!` for production builds.
- **Memory locking:** `LockedBytes` correctly calls `munlock`/`VirtualUnlock` on drop and zeroizes locked memory.

### What Needs Improvement

- Add production guard to `DeterministicRngProvider`.
- Use bounded reads in `YamlConfig::from_file`.
- Include log level in `RotatingFileLogger` output.
- Document or fix the `serde_json` macro exception.

---

## 9. Test Plan

### Positive Tests (Already Present)
- ✅ RNG determinism tests (`test_deterministic_rng_seed_output`, `test_mock_rng_provider_deterministic`)
- ✅ Codec round-trip tests for JSON, YAML, Bincode
- ✅ Trailing bytes rejection tests
- ✅ Atomic write tests
- ✅ Rotating file logger rotation tests
- ✅ Mock time provider advancement tests

### Missing Tests (Recommended)
- `DeterministicRngProvider` compile-time guard — verify it fails in release mode
- `YamlConfig::from_file` with oversized file — verify bounded read rejection
- `RotatingFileLogger` output format — verify level is included
- `lz4_decompress_bounded` with non-LZ4 input — verify format rejection
- `zstd_decompress_bounded` with truncated input — verify error handling

### Adversarial Tests (Recommended)
- Symlink attack on log file path
- Decompression bomb (1 GB compressed → 1 MB limit)
- Concurrent `MockTimeProvider` access from multiple threads
- `BincodeCodec::deserialize_bounded` with exact boundary values

---

## 10. Confidence Level

| Claim | Confidence | Evidence That Would Change It |
|-------|------------|-------------------------------|
| `LockedBytes::drop` is correctly implemented | **High** | Verified by reading full `os_hardening.rs` (lines 200–230) |
| `DeterministicRngProvider` lacks production guard | **High** | Verified by direct code inspection |
| `serde_json` re-export violates abstraction | **Medium** | Project ADR may document intentional exception |
| `YamlConfig` uses unbounded read | **High** | Verified by direct code inspection |
| `RotatingFileLogger` discards level | **High** | Verified by `let _ = level;` |
| `chrono` dependency is a risk | **Low** | Standard practice; no action needed |

---

## 11. Final Decision

**Status:** `Blocked — 2 open decisions`

| # | Decision | Owner | Required Evidence |
|---|----------|-------|-------------------|
| 1 | Is `DeterministicRngProvider` used in production code paths? | Project lead | `grep` across `z00z_core` and `z00z_wallets` |
| 2 | Is `serde_json` macro re-export an intentional exception? | Project lead | ADR or design discussion |

Once these decisions are resolved, all S1 findings have concrete fixes and the crate can be signed off for production use.

---

*Report generated by GitHub Copilot using Crypto Architect skill v1.0*  
*Xiaomi MiMo-V2-Pro*
