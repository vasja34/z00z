# 027-Crypto-Audit-Utils: z00z_utils Deep Cryptographic Audit

**Document:** `027-crypto-audit-utils/utils-audit-m27.md`
**Scope:** `crates/z00z_utils/src/` — all \*.rs implementations
**Excluded:** `z00z_crypto/tari/` vendor code (read-only boundary)
**Date:** 2026-03-26
**Classification:** CRYPTOGRAPHIC CODE REVIEW — Z00Z PHASE 027

---

## Executive Verdict

**`Safe enough`** — no S0/S1 findings. The crate correctly separates RNG
providers, uses `OsRng` for production randomness, and has sound mitigations
for decompression bombs, atomic writes, and permission hardening.

Minor S2/S3 findings and several S4 INFO items are documented below.
All findings are **implementation-quality** issues, not design flaws.

---

## 1. Input Classification

| Category | Value |
| --- | --- |
| Input type | Source code implementation (multiple \*.rs modules) |
| Scope | `z00z_utils` crate — abstraction layer for I/O, serialization, RNG, time, metrics, logging, config, compression, OS hardening |
| Threat model assumed | z00z_utils is a **utility/abstraction layer**, not a cryptographic primitive library. It must: (a) not introduce entropy weaknesses in callers, (b) handle secrets securely in memory, (c) not leak secrets through errors/logs, (d) resist resource exhaustion via bounded reads/decompression |

---

## 2. Security Goals Extracted

| Goal | Status |
| --- | --- |
| RNG: production path uses cryptographically secure `OsRng` | ✅ PASS |
| RNG: deterministic path clearly gated and documented as unsafe for secrets | ✅ PASS |
| Time: abstraction supports deterministic testing without `SystemTime` | ✅ PASS |
| Codec: bounded deserialization prevents memory exhaustion | ✅ PASS |
| Codec: trailing-bytes rejection prevents ambiguous input | ✅ PASS |
| I/O: atomic writes prevent partial-file artifacts | ✅ PASS |
| I/O: private-permission path for wallet material (0o600) | ✅ PASS |
| Compression: bounded decompression prevents bombs | ✅ PASS |
| OS Hardening: mlock best-effort (fails-closed on error) | ✅ PASS |
| Secrets: seed fields use `ZeroizeOnDrop` | ✅ PASS |
| Secrets: Debug impl redacts seed in `DeterministicRngProvider` and `MockRngProvider` | ✅ PASS |
| No hardcoded seeds, no predictable randomness in production | ✅ PASS |
| No `unwrap()` in hot crypto paths | ✅ PASS |

---

## 3. Critical and High Findings (S0/S1)

**None.** No S0 or S1 findings in this crate.

---

## 4. Medium and Low Findings (S2/S3)

### Finding M1 — `io/fs.rs`: `atomic_write_file_private` lacks fsync on non-Unix paths

| Field | Value |
| --- | --- |
| Severity | S2 |
| Component | `crates/z00z_utils/src/io/fs.rs`, `atomic_write_file_private` |
| Problem | The non-Unix (`#[cfg(not(unix))]`) branch delegates to `write_file`, which does **not** call `fsync`. On Windows, a crash after `Write` but before the OS flushes pages can lose data. |
| Impact | Wallet backup snapshots or sensitive data written on Windows may be lost or partially written, with no atomic-guarantee signal to caller. |
| Fix | On the non-Unix path, open with `File::open` + `sync_all()` or use `std::io::Write::flush()` before returning. Alternatively, add a `sync_data()` call in the non-Unix branch, or document that Windows atomic writes are best-effort only. |

```rust
// filepath: crates/z00z_utils/src/io/fs.rs
#[cfg(not(unix))]
{
    write_file(path, data)  // ← no fsync guarantee
}
```

### Finding M2 — `io/fs.rs`: `atomic_write_file_streaming` — identical non-Unix gap

| Field | Value |
| --- | --- |
| Severity | S2 |
| Component | `crates/z00z_utils/src/io/fs.rs`, `atomic_write_file_streaming` |
| Problem | Same as M1: the non-Unix branch has no `fsync` equivalent. |
| Impact | Same as M1, plus caller may assume streaming atomicity is guaranteed. |
| Fix | Same as M1. Apply to both functions or extract a shared helper. |

### Finding L1 — `compression.rs`: LZ4 frame magic constant is little-endian only

| Field | Value |
| --- | --- |
| Severity | S3 |
| Component | `crates/z00z_utils/src/compression.rs` |
| Problem | `LZ4_FRAME_MAGIC_LE` is defined as `[0x04, 0x22, 0x4D, 0x18]` with comment `LE`. LZ4 frame magic is defined in the spec as `0x184D2204` (little-endian representation on little-endian machines). Using this constant directly as a byte array on a big-endian host would produce an incorrect comparison. |
| Impact | `is_lz4_frame` would return incorrect results on big-endian platforms. Since Z00Z compiles for `target_arch = "wasm32"` (little-endian) and standard server targets (all little-endian), this is low risk in practice. |
| Fix | Guard with `#[cfg(target_endian = "little")]` or use `u32::from_le_bytes` to make the conversion explicit and add a compile-time assert for the byte layout. |

### Finding L2 (RESCINDED) — `os_hardening.rs`: `LockedBytes::Drop` was incorrectly suspected

| Field | Value |
| --- | --- |
| Severity | ~~S3~~ **RESCINDED** |
| Problem | Originally suspected that `Drop` was empty — **incorrect**. The `Drop` impl at `os_hardening.rs:199` correctly zeroes the slice via `bytes.zeroize()` and calls `munlock` (Unix) / `VirtualUnlock` (Windows) when `self.active == true`. No fix needed. |

---

## 5. INFO Findings (S4)

### INFO-1 — `compression.rs`: `zstd_decompress_bounded` re-checks after `take(max_output)`

The extra-read check after `take(max_output)` in `zstd_decompress_bounded` is a solid pattern, but the comment says "lower bound" on `actual` in `OutputTooLarge`. This is correct: `read_to_end` may have read fewer bytes than limit if EOF arrived exactly at the boundary. Good.

### INFO-2 — `rng/mock.rs`: `MockRngProvider` compile-error gate is technically correct but fragile

```rust
#[cfg(all(
    not(test),
    not(debug_assertions),
    not(feature = "test-utils"),
    not(feature = "test-fast")
))]
compile_error!("MockRngProvider MUST NOT be compiled in production builds");
```

The gate excludes `debug_assertions` — meaning debug builds with `cargo build` (not `cargo build --release`) will compile `MockRngProvider`. This is intentional per the gate logic, but worth noting: debug builds used in production would have predictable RNG. This is acceptable if Z00Z's threat model only uses release builds in production.

### INFO-3 — `codec/yaml.rs`: Multi-document YAML detection

`serde_yml`'s `Deserializer::from_str` returns an iterator. The check `if docs.next().is_some()` after consuming the first document correctly catches trailing documents. Good.

### INFO-4 — `codec/bincode.rs`: Three fixed size limits (1MB/10MB/100MB)

`deserialize_bounded` only accepts three specific limits matching `LIMIT_1MB_BYTES`, `LIMIT_10MB_BYTES`, `LIMIT_100MB_BYTES`. This is restrictive but prevents arbitrary limit bugs. Caller must use one of the three constants.

### INFO-5 — `time/traits.rs`: `unix_timestamp_micros()` overflow returns `u64::MAX`

In `try_unix_timestamp_micros`, when `as_micros()` overflows `u64`, it returns `u64::MAX` rather than an error. This is **silent** — callers using `unix_timestamp_micros()` (the unwrap variant) get `u64::MAX`. This is unlikely in practice (requires clock to be ~584,000 years after epoch) but could be confusing in tests or forensics. Not a security issue.

### INFO-6 — `io/fs.rs`: `atomic_write_with_context` permission inheritance

When overwriting an existing file, permissions are inherited from the destination via `std::fs::metadata(path)`. If the original file had restrictive permissions and the temp file inherits them, this is fine. However, if the original had **less** restrictive permissions, the new file inherits those weaker permissions temporarily. Since the rename is atomic, this window is extremely small.

### INFO-7 — `os_hardening.rs`: `apply_best_effort` requires explicit consumer call

`apply_best_effort()` is defined and exported but never auto-invoked. See A2 in Open Ambiguities.

---

## 6. Open Ambiguities

| Item | Description |
| --- | --- |
| A1 | No performance benchmarks for compression. `zstd_compress` uses level 0 (implementation-defined default). For wallet backups, this may be acceptable, but the trade-off between compression speed, ratio, and CPU cost should be documented. |
| A2 | `apply_best_effort()` is defined and exported but never auto-invoked at startup. Callers are responsible for invoking it at process initialization. This should be explicitly documented. |

---

## 7. Concrete Fixes

### Fix for M1 + M2 (non-Unix fsync)

For `atomic_write_file_private` and `atomic_write_file_streaming`, add a fallback
fsync-like call in the non-Unix branch, or document the gap explicitly.

---

## 8. Implementation Guidance

| Area | Assessment |
| --- | --- |
| RNG trait separation (`SecureRngProvider` vs `DeterministicRngProvider`) | ✅ Clean separation; no accidental misuse in production |
| ZeroizeOnDrop on seed fields | ✅ Correct |
| Debug impls redact seeds | ✅ Correct |
| Bounded file reads | ✅ `take(max_bytes)` + size pre-check |
| Trailing-bytes rejection in all codecs | ✅ JSON, YAML, Bincode all reject trailing data |
| Atomic writes use temp+rename in same directory | ✅ Mitigates TOCTOU and symlink attacks |
| Private-perm path (0o600) for wallet material | ✅ Good |
| Decompression bomb mitigation | ✅ `take(max_output)` + re-read check |
| OS hardening best-effort fails-closed (no panics) | ✅ Good |

---

## 9. Test Plan

| Test | Coverage |
| --- | --- |
| Trailing-bytes rejection in JSON codec | ✅ Present in `codec/json.rs` tests |
| Trailing-bytes rejection in YAML codec | ✅ Present in `codec/yaml.rs` tests |
| Bounded bincode deserialization | ✅ Partial: covers size limits |
| Decompression bomb: `zstd_decompress_bounded` with max_output=1 | Manual: verify `OutputTooLarge` returned |
| Decompression bomb: `lz4_decompress_bounded` with max_output=1 | Manual: verify `OutputTooLarge` returned |
| `DeterministicRngProvider`: same seed → same sequence | ✅ `test_deterministic_rng_seed_output` |
| `MockRngProvider`: compile_error in production | Manual: verify `cargo build` (non-release) compiles, `cargo build --release` does not |
| `atomic_write_file_private`: permissions 0o600 on Unix | Manual: stat file after call |
| `atomic_write_file_private`: parent dir fsync | Manual: crash-inject before persist |
| `LockedBytes::Drop`: munlock called | Manual: `getrusage(RUSAGE_SELF, &mut r)` before/after drop |

**Missing test coverage:**

- No test for `unix_timestamp_micros()` overflow returning `u64::MAX`
- No integration test for multi-codec fallback (YAML → JSON → Bincode)
- No test for `is_lz4_frame` on a crafted non-LE byte sequence

---

## 10. Confidence Level

| Claim | Confidence | Evidence That Would Change It |
| --- | --- | --- |
| `SystemRngProvider` uses `OsRng` correctly | **HIGH** | Review of `rand` crate source confirms `OsRng` uses OS CSPRNG |
| `DeterministicRngProvider` is never compiled into production binaries | **MEDIUM** | Only if a non-release build is deployed; gate relies on `#[cfg(not(test))]` which fires in debug builds too |
| No decompression bombs possible via public API | **HIGH** | Both bounded functions have explicit limits; `take()` enforced |
| Atomic writes are truly atomic on ext4, XFS, Btrfs | **MEDIUM** | POSIX guarantees rename atomicity on same filesystem; Windows atomic rename has historical issues |
| Secrets are not logged in error messages | **HIGH** | Error messages use `Display` which avoids secret fields |
| `ZeroizeOnDrop` actually zeroes memory | **HIGH** | `zeroize` crate is well-audited |

---

## 11. Final Decision

**`Execution-ready`** — the crate is suitable for use as the ONE SOURCE OF TRUTH
abstraction layer. The two medium findings (M1, M2) and the low finding (L2) are
correctable without design changes. The INFO items are observations, not defects.

**Required before sign-off:**

- [ ] Fix M1 + M2: document or address non-Unix fsync gap in `atomic_write_file_private` and `atomic_write_file_streaming`

**Recommended improvements:**

- [ ] Add test for `unix_timestamp_micros()` overflow returning `u64::MAX`
- [ ] Clarify in docs that `apply_best_effort()` must be called by consumers at startup
- [ ] Consider renaming `lock_bytes_best_effort` → `lock_bytes` with documented best-effort semantics, or add `lock_bytes_exact()` that returns an error on failure
