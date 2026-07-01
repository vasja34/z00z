# 027 — Cryptographic Audit: `z00z_utils` Crate

**Auditor:** GLM-5 Crypto Architect  
**Date:** 2026-03-26  
**Scope:** `crates/z00z_utils/src/**/*.rs` (excluding `tari/` vendor)  
**Branch:** `z00z-simul`  
**Classification:** Implementation review — utility/infrastructure layer

---

## Executive Verdict

**`Safe enough`** — no S0 or S1 cryptographic findings. The crate is a well-structured
abstraction layer with correct security boundaries, proper secret handling, and
defense-in-depth against common attack vectors. Several S2/S3 improvements are
recommended but none are blocking.

---

## 1. Input Classification

**Type:** Implementation review — Rust source code for a utility/infrastructure crate.  
**Layers reviewed:** RNG, OS hardening, serialization/codec, file I/O, compression, time, config, logger, metrics.  
**Crypto-relevant modules:** `rng/`, `os_hardening.rs`, `codec/`, `io/fs.rs`, `compression.rs`.

---

## 2. Scope and Threat Model

### Security Goals

| Goal | Relevance to z00z_utils |
|------|------------------------|
| Confidentiality | RNG entropy quality; secret zeroization; file permissions for wallet data |
| Integrity | Atomic writes; trailing-byte rejection in deserialization; decompression bomb prevention |
| Authenticity | Not directly applicable (no signatures in this crate) |
| Availability | Bounded reads/writes; decompression output limits; file size limits |

### Assets Protected

- **RNG output** — used upstream for keys, nonces, commitments, range proofs
- **Seed material** — `MockRngProvider::seed`, `DeterministicRngProvider::seed`
- **Wallet files on disk** — written via `atomic_write_file_private` with mode 0o600
- **In-memory secrets** — `LockedBytes` guard prevents swapping to disk
- **Process memory** — core dump disable, non-dumpable flag

### Adversaries Considered

| Adversary | Mitigated? |
|-----------|-----------|
| Local attacker reading core dumps | ✅ `disable_core_dumps()` + `PR_SET_DUMPABLE` |
| Local attacker reading swapped memory | ✅ `mlock`/`VirtualLock` via `LockedBytes` |
| Remote attacker sending malformed files | ✅ Bounded reads, trailing-byte rejection, decompression limits |
| Symlink attack on log/wallet files | ✅ `ensure_no_symlink()` in `FileLogger` and `RotatingFileLogger` |
| TOCTOU on temp file creation | ✅ `NamedTempFile::new_in` with random name in same directory |
| Denial-of-service via decompression bomb | ✅ `zstd_decompress_bounded` / `lz4_decompress_bounded` with strict limits |

### Trust Boundaries

- `z00z_utils` is the **lowest layer** — all other crates depend on it
- No network-facing code; all I/O is local filesystem
- RNG is the most security-critical component (feeds `z00z_crypto` and `z00z_core`)

---

## 3. Findings

### S0 — CRITICAL

*None found.*

### S1 — HIGH

*None found.*

### S2 — MEDIUM

| # | Component | Problem | Impact | Fix |
|---|-----------|---------|--------|-----|
| S2-1 | `rng/mock.rs` — `MockRngProvider::with_u64_seed` | Seed expansion uses SHA-256 but the input is only 8 bytes of entropy padded with 24 zero bytes. While SHA-256 prevents trivial seed recovery, the effective entropy space is $2^{64}$, which is below the 128-bit minimum for cryptographic seed material. | If `MockRngProvider` is accidentally used in production (despite `compile_error!` guard), keys derived from a u64 seed have only 64 bits of entropy — vulnerable to brute force. | Document that `with_u64_seed` is test-only and must never be used for any value that contributes to key material. Consider adding a `#[deprecated]` annotation or renaming to `with_test_seed`. The `compile_error!` guard already prevents production compilation, which is the primary mitigation. |
| S2-2 | `rng/deterministic.rs` — `DeterministicRngProvider` | The `seed()` accessor is gated behind `#[cfg(any(test, feature = "test-utils"))]`, but the `Debug` impl redacts the seed. However, `Clone` is derived, meaning the seed can be extracted via `Clone` + `rng()` → compare output. More importantly, `DeterministicRngProvider` implements `DeterministicRng` (not `SecureRngProvider`), which is correct, but the trait-level distinction could be bypassed if a caller uses `impl DeterministicRng` where `impl SecureRngProvider` was intended. | Type-system confusion could lead to deterministic RNG being used where unpredictable RNG is required. The trait names provide good guidance but are not compiler-enforced barriers. | Consider adding a `#[doc(hidden)]` marker or a lint check that verifies `SecureRngProvider` is used in specific modules. The current design is adequate given the clear documentation, but a static analysis rule would add defense-in-depth. |
| S2-3 | `os_hardening.rs` — `LockedBytes::Drop` | The `Drop` impl uses `unsafe { std::slice::from_raw_parts_mut(self.addr as *mut u8, self.len) }` followed by `bytes.zeroize()`. This reconstructs a raw slice from a stored pointer. If the original buffer is dropped before `LockedBytes`, this is use-after-free. The current API design (returning `Option<LockedBytes>` tied to the caller's buffer) makes this difficult but not impossible in unsafe code. | Memory safety violation if `LockedBytes` outlives the backing buffer. In practice, Rust's borrow checker prevents this in safe code, but the `unsafe` block in `Drop` means the invariant must be maintained manually. | Add a lifetime parameter to `LockedBytes<'a>` that ties it to the backing buffer. This would make the invariant compiler-enforced. Alternatively, store a reference-counted handle to the buffer. |
| S2-4 | `codec/yaml.rs` — `YamlCodec::deserialize` | YAML deserialization does not enforce a size limit. Unlike `BincodeCodec` (which has `deserialize_bounded`) and `JsonCodec` (which uses `serde_json::Deserializer::from_slice`), `YamlCodec` passes the full byte slice to `serde_yml::Deserializer::from_str`. A malicious YAML file could cause excessive memory allocation during parsing. | Denial-of-service via YAML bomb (e.g., deeply nested anchors/aliases that expand exponentially). | Add a bounded variant `deserialize_bounded` similar to `BincodeCodec`, or wrap the YAML string in a `Read::take(max_bytes)` adapter before parsing. |
| S2-5 | `config/yaml.rs` — `YamlConfig::from_file` | Uses `std::fs::read_to_string` directly instead of `crate::io::read_file_bounded`. This bypasses the 10 MB default file size limit enforced by the I/O abstraction layer. | A malicious or corrupted YAML config file could consume unbounded memory. | Replace `std::fs::read_to_string(path.as_ref())?` with `crate::io::read_to_string(path.as_ref())?` which uses `read_file_bounded` internally. |

### S3 — LOW

| # | Component | Problem | Impact | Fix |
|---|-----------|---------|--------|-----|
| S3-1 | `rng/traits.rs` — `SecureRngProvider` | The trait requires `Rng: RngCore + CryptoRng + Send` but does not require `Clone`. `OsRng` is `Clone` (each clone is independent), but if a future implementation uses a stateful CSPRNG, the lack of `Clone` could cause issues with the "call `rng()` once per operation" pattern documented in `MockRngProvider`. | Minor API inconsistency; no current security impact. | Consider adding `Clone` bound or documenting the expected cloning semantics. |
| S3-2 | `os_hardening.rs` — `apply_best_effort` | Core dump disabling and `PR_SET_DUMPABLE` are applied at process level but not verified after application. An attacker with `CAP_SYS_PTRACE` could re-enable core dumps after hardening. | Hardening can be bypassed by privileged local attacker. | This is acceptable for best-effort hardening. Document the assumption that the process runs without excessive capabilities. |
| S3-3 | `io/fs.rs` — `write_file` | The non-private `write_file` does not set restrictive permissions on Unix. Files are created with the process umask, which may be world-readable. | Wallet data written via `write_file` instead of `atomic_write_file_private` could be readable by other users. | Document that `write_file` is for non-sensitive data only. Consider adding a `write_file_private` alias or a lint. |
| S3-4 | `compression.rs` — `lz4_compress` | LZ4 compression level is not explicitly configured (uses `EncoderBuilder::new()` defaults). The default level (0 = fast) may produce larger outputs than expected. | No security impact; minor efficiency concern. | Consider documenting the compression level or making it configurable. |
| S3-5 | `logger/file_logger.rs` — `sanitize_message` | Null bytes are stripped from log messages but other control characters (e.g., terminal escape sequences) are not. Log injection via crafted log messages could manipulate terminal output. | Log injection in terminal-based log viewers. | Extend sanitization to strip or escape ANSI escape sequences and other control characters. |
| S3-6 | `time/traits.rs` — `unix_timestamp_micros` | Returns `u64::MAX` when the microsecond value overflows `u64`. This sentinel value could be confused with a legitimate far-future timestamp. | Logic errors in timestamp comparisons. | Return `Result<u64, TimeError>` instead of clamping to `u64::MAX`, or document the sentinel behavior prominently. |

### S4 — INFO

| # | Component | Observation |
|---|-----------|-------------|
| S4-1 | `rng/system.rs` | `SystemRngProvider` returns a new `OsRng` per call. This is correct — each `OsRng` is an independent handle to the OS CSPRNG. No state is shared or reused. |
| S4-2 | `codec/json.rs` | Trailing-byte rejection is correctly implemented using `serde_json::Deserializer::into_iter` and checking for additional values. This prevents ambiguous parsing. |
| S4-3 | `codec/bincode.rs` | The `deserialize_bounded` method correctly enforces size limits at both the input level and the bincode configuration level. The tiered limit approach (1MB/10MB/100MB) is pragmatic. |
| S4-4 | `io/fs.rs` | `atomic_write_file_private` correctly uses `sync_all()` on both the file and parent directory for durability. This prevents data loss on crash. |
| S4-5 | `os_hardening.rs` | The `compile_error!` guard on `MockRngProvider` is an excellent defense-in-depth measure preventing accidental production use. |
| S4-6 | `rng/ext.rs` | `RngCoreExt` is a thin wrapper that avoids leaking `rand::RngCore` into downstream crates. Good abstraction hygiene. |
| S4-7 | `compression.rs` | Decompression bomb prevention via `take(max_output)` + extra-byte check is correctly implemented for both Zstd and LZ4. |
| S4-8 | `Cargo.toml` | Dependencies are appropriate: `rand 0.8`, `rand_chacha 0.3`, `sha2 0.10`, `zeroize 1.7`, `serde_yml 0.0.12`. All are well-maintained crates from the RustCrypto ecosystem. |

---

## 4. Composition Review

### RNG → Crypto Pipeline

The RNG abstraction correctly separates `SecureRngProvider` (production) from `DeterministicRngProvider` (testing/genesis). The type system enforces this at the trait level:

```
SecureRngProvider → OsRng (unpredictable, per-call independent)
DeterministicRngProvider → ChaCha20Rng / StdRng (deterministic, seed-derived)
```

**Verified:** `SecureRngProvider` is only implemented by `SystemRngProvider`.  
**Verified:** `DeterministicRngProvider` is implemented by `MockRngProvider` and `DeterministicRngProvider`.  
**Verified:** `MockRngProvider` has `compile_error!` for non-test, non-debug builds.

### Serialization → Canonical Encoding

All three codecs (JSON, YAML, Bincode) reject trailing bytes after deserialization. This prevents ambiguous encodings that could be exploited in consensus-critical contexts.

**Verified:** `JsonCodec` uses `into_iter::<T>()` and checks `stream.next().is_some()`.  
**Verified:** `YamlCodec` uses `Deserializer::from_str` and checks `docs.next().is_some()`.  
**Verified:** `BincodeCodec` checks `len != bytes.len()` after `decode_from_slice`.

### File I/O → Atomicity

All write operations use `NamedTempFile::new_in` (random name in same directory) + `persist()`. This prevents TOCTOU attacks from predictable temp paths.

**Verified:** `write_file`, `save_json`, `save_yaml`, `save_bincode`, `atomic_write_file_private`, `atomic_write_file_streaming` all use this pattern.

### OS Hardening → Secret Lifecycle

The hardening module provides:
1. Core dump disable (`RLIMIT_CORE = 0`)
2. Non-dumpable process (`PR_SET_DUMPABLE = 0`)
3. Memory locking (`mlock`/`VirtualLock`)
4. Zeroization on drop (`zeroize::Zeroize` in `LockedBytes::Drop`)

**Gap:** `LockedBytes` does not have a lifetime parameter (see S2-3).

---

## 5. Dependency Audit

| Crate | Version | Status | Notes |
|-------|---------|--------|-------|
| `rand` | 0.8 | ✅ Good | Standard Rust RNG; `CryptoRng` trait used correctly |
| `rand_chacha` | 0.3 | ✅ Good | ChaCha20 CSPRNG; audited construction |
| `sha2` | 0.10 | ✅ Good | RustCrypto SHA-256; used for seed expansion |
| `zeroize` | 1.7 | ✅ Good | Secure memory zeroing; `ZeroizeOnDrop` derived |
| `serde_json` | 1.0 | ✅ Good | Standard JSON serialization |
| `serde_yml` | 0.0.12 | ⚠️ Check | Fork of `serde_yaml`; verify maintenance status |
| `bincode` | 2.0.1 | ✅ Good | v2.0 with bounded deserialization |
| `zstd` | 0.13 | ✅ Good | Zstd compression; well-maintained |
| `lz4` | 1 | ✅ Good | LZ4 frame compression |
| `tempfile` | 3.10 | ✅ Good | Secure temp file creation |
| `libc` | 0.2 | ✅ Good | POSIX bindings for hardening |
| `chrono` | 0.4 | ✅ Good | Timestamp formatting only |

**Recommendation:** Verify `serde_yml 0.0.12` maintenance status. This is a fork of the unmaintained `serde_yaml` crate. Ensure it receives security updates.

---

## 6. Test Coverage Assessment

### Positive Tests (Verified Present)

- ✅ `test_system_rng_provider_generates` — verifies non-deterministic output
- ✅ `test_mock_rng_provider_deterministic` — verifies deterministic output
- ✅ `test_mock_rng_provider_different` — verifies seed sensitivity
- ✅ `test_mock_rng_provider_thread` — verifies thread safety
- ✅ `test_rng_provider_send_sync` — verifies trait bounds
- ✅ `test_json_trailing_data_rejected` — trailing JSON rejection
- ✅ `test_json_trailing_garbage_rejected` — garbage after JSON rejection
- ✅ `test_yaml_multi_doc_rejected` — multi-document YAML rejection
- ✅ `test_bincode_codec_round_trip` — bincode serialization round-trip
- ✅ `test_lock_bytes_zero_on_drop` — memory zeroization verification
- ✅ `test_disable_core_dumps` — hardening API contract
- ✅ `test_memory_lock` — mlock API contract

### Missing Tests (Recommended)

| Test | Priority | Description |
|------|----------|-------------|
| `test_json_empty_input_rejected` | S3 | Verify empty input returns `CodecError::Json("empty JSON input")` |
| `test_yaml_empty_input_rejected` | S3 | Verify empty input returns `CodecError::Yaml("empty YAML input")` |
| `test_bincode_oversized_input_rejected` | S3 | Verify input > 10 MB returns `DeserializeSizeLimitExceeded` |
| `test_decompression_bomb_rejected` | S2 | Verify Zstd/LZ4 decompression with small compressed input but large output limit is enforced |
| `test_locked_bytes_empty_slice` | S4 | Verify `lock_bytes_best_effort(&mut [])` returns `None` |
| `test_file_logger_symlink_rejected` | S3 | Verify symlink log paths are rejected |
| `test_atomic_write_preserves_permissions` | S3 | Verify existing file permissions are preserved on overwrite |

### Fuzzing Targets (Recommended)

| Target | Priority | Description |
|--------|----------|-------------|
| `JsonCodec::deserialize` | S3 | Fuzz with arbitrary bytes to verify no panics |
| `YamlCodec::deserialize` | S3 | Fuzz with arbitrary bytes to verify no panics |
| `BincodeCodec::deserialize` | S3 | Fuzz with arbitrary bytes to verify no panics |
| `lz4_decompress_bounded` | S2 | Fuzz with arbitrary bytes to verify bomb prevention |
| `zstd_decompress_bounded` | S2 | Fuzz with arbitrary bytes to verify bomb prevention |

---

## 7. Confidence Assessment

| Claim | Confidence | Evidence That Would Change It |
|-------|-----------|-------------------------------|
| RNG entropy quality (production) | **High (95%)** | `OsRng` delegates to OS CSPRNG (`getrandom` syscall on Linux). Would decrease if custom entropy mixing were added without audit. |
| Deterministic RNG reproducibility | **High (95%)** | ChaCha20 is a standard stream cipher; same seed → same output is mathematically guaranteed. |
| Secret zeroization | **Medium (80%)** | `zeroize::Zeroize` is applied in `LockedBytes::Drop` and `ZeroizeOnDrop` is derived on providers. Would increase with `mlock` + `zeroize` integration test under Valgrind/ASAN. |
| Atomic write safety | **High (90%)** | `NamedTempFile` + `persist()` is a well-established pattern. Would decrease if custom temp file logic were introduced. |
| Decompression bomb prevention | **High (90%)** | `take(max_output)` + extra-byte check is correct. Would decrease if streaming decompression were added without limits. |
| Serialization canonical enforcement | **High (90%)** | Trailing-byte rejection is implemented for all three codecs. Would decrease if new codecs were added without this check. |

---

## 8. Open Ambiguities

| # | Question | Owner | Impact |
|---|----------|-------|--------|
| OA-1 | Is `serde_yml 0.0.12` actively maintained? Who is the maintainer? | Build/infra team | S2 if unmaintained — YAML parsing vulnerabilities |
| OA-2 | Should `YamlConfig::from_file` use bounded I/O? (see S2-5) | z00z_utils maintainer | S2 DoS vector |
| OA-3 | Is `LockedBytes` intended to be used across async boundaries? If so, the lack of lifetime parameter is more concerning. | z00z_utils maintainer | S2 if used in async context |
| OA-4 | What is the maximum expected file size for wallet backup containers? This determines appropriate `max_output` for decompression. | z00z_wallets team | S2 if limit is too generous |

---

## 9. Concrete Fixes

### Fix S2-1: Document `with_u64_seed` as test-only

```rust
/// ⚠️ **TEST-ONLY:** Expands a u64 to a 256-bit seed via SHA-256.
/// Effective entropy is 64 bits — MUST NOT be used for key material.
/// Use `with_seed([u8; 32])` with a full-entropy seed for any security-relevant purpose.
#[cfg(any(test, feature = "test-utils", feature = "test-fast"))]
pub fn with_u64_seed(seed: u64) -> Self {
```

### Fix S2-3: Add lifetime to `LockedBytes`

```rust
pub struct LockedBytes<'a> {
    _marker: std::marker::PhantomData<&'a mut [u8]>,
    addr: usize,
    len: usize,
    active: bool,
}
```

### Fix S2-4: Add bounded YAML deserialization

```rust
impl YamlCodec {
    pub fn deserialize_bounded<T: DeserializeOwned>(
        &self,
        bytes: &[u8],
        max_bytes: u64,
    ) -> Result<T, CodecError> {
        if bytes.len() as u64 > max_bytes {
            return Err(CodecError::DeserializeSizeLimitExceeded {
                size: bytes.len(),
                limit: max_bytes,
            });
        }
        self.deserialize(bytes)
    }
}
```

### Fix S2-5: Use bounded I/O in `YamlConfig::from_file`

```rust
pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
    let content = crate::io::read_to_string(path.as_ref())?;
    // ...
}
```

---

## 10. Final Decision

**`Execution-ready`** with the following conditions:

1. ✅ No S0 or S1 findings — proceed with current implementation
2. ⚠️ S2-5 (`YamlConfig::from_file` unbounded read) should be fixed before processing untrusted YAML config files from external sources
3. ⚠️ S2-3 (`LockedBytes` lifetime) should be addressed if `LockedBytes` is used across async boundaries or stored in structs
4. 📋 S2-4 (YAML deserialization bounds) should be added for defense-in-depth
5. 📋 Verify `serde_yml` maintenance status (OA-1)

All other findings are S3/S4 and can be addressed incrementally.

---

## Appendix A: Files Reviewed

| File | Lines | Crypto-Relevant |
|------|-------|----------------|
| `lib.rs` | 55 | Module structure |
| `rng/mod.rs` | 25 | RNG facade |
| `rng/traits.rs` | 55 | **HIGH** — trait definitions |
| `rng/system.rs` | 35 | **HIGH** — production CSPRNG |
| `rng/mock.rs` | 130 | **HIGH** — test RNG |
| `rng/deterministic.rs` | 120 | **HIGH** — genesis RNG |
| `rng/ext.rs` | 15 | Abstraction helper |
| `rng/test_rng.rs` | 100 | Test coverage |
| `os_hardening.rs` | 200 | **HIGH** — memory locking, core dumps |
| `codec/mod.rs` | 15 | Codec facade |
| `codec/traits.rs` | 80 | Codec trait + error types |
| `codec/json.rs` | 120 | JSON serialization |
| `codec/bincode.rs` | 130 | Binary serialization |
| `codec/yaml.rs` | 100 | YAML serialization |
| `io/mod.rs` | 30 | I/O facade |
| `io/fs.rs` | 350 | **MEDIUM** — file operations |
| `io/error.rs` | 30 | Error types |
| `compression.rs` | 200 | **MEDIUM** — decompression limits |
| `time/traits.rs` | 70 | Time abstraction |
| `time/system.rs` | 40 | System time |
| `time/mock.rs` | 80 | Mock time |
| `time/format.rs` | 70 | Timestamp formatting |
| `time/mod.rs` | 30 | Time facade |
| `config/mod.rs` | 40 | Config facade |
| `config/traits.rs` | 60 | Config trait |
| `config/env.rs` | 25 | ENV config |
| `config/yaml.rs` | 60 | YAML config |
| `config/layered.rs` | 70 | Layered config |
| `logger/traits.rs` | 30 | Logger trait |
| `logger/structured.rs` | 50 | Structured logging |
| `logger/file_logger.rs` | 80 | File logger |
| `logger/rotating_file_logger.rs` | 100 | Rotating logger |
| `metrics/traits.rs` | 30 | Metrics trait |

**Total:** ~2,500 lines of Rust source code reviewed.

---

## Appendix B: Severity Distribution

```
S0 (CRITICAL):    0
S1 (HIGH):        0
S2 (MEDIUM):      5
S3 (LOW):         6
S4 (INFO):        8
```

---

*Report generated by GLM-5 Crypto Architect following the Crypto Architect skill specification.*
