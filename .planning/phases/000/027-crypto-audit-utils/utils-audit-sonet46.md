# 🔐 Crypto-Security Audit — `z00z_utils` Crate

| Field | Value |
|---|---|
| **Report ID** | `027-utils-audit-sonet46` |
| **Auditor** | GitHub Copilot (Claude Sonnet 4.6) |
| **Scope** | `crates/z00z_utils/src/**/*.rs` — all 47 implementation files |
| **Excluded** | `tari/` vendor subtree, external crates, test-only files (noted where relevant) |
| **Date** | 2025-07-15 |
| **Rust Edition** | 2021, `rand 0.8`, `bincode 2.0.1`, `zstd 0.13`, `lz4 1` |

---

## 📌 Executive Verdict

> **RISKY BUT SALVAGEABLE** — No critical S0/S1 flaw exists. Two medium S2 findings can trigger silent security degradation in production (clock fallback to timestamp=0, and permission-preservation failure on file writes). Six S3/S4 gaps are defense-in-depth weaknesses that increase attack surface without being directly exploitable on their own. The positive control set is strong: atomic writes, `ZeroizeOnDrop`, bounded decompression, symlink rejection, 0o600 mode enforcement on sensitive paths, and a meaningful OS-hardening layer.
>
> ⭐ `atomic_write_file_private` is well-implemented (0o600 + `sync_all` + parent dir sync). It should be the canonical write path for all key material.
>
> 🚨 Two changes are required before production use of any nonce, expiry, or file-permission-sensitive path.

---

## ⚙️ Scope & Input Classification

| Module | Sensitive input type | Notes |
|---|---|---|
| `rng/` | Seed material, 32-byte keys | DeterministicRng seed, mock SHA-256 expansion |
| `os_hardening` | Process-level memory layout | mlock, prctl, LockedBytes |
| `time/` | System clock, timestamps | Anti-replay, nonce, expiry inputs |
| `io/fs.rs` | File contents, permissions | Wallet snapshots, keys, configs |
| `codec/` | Wire/disk binary buffers | Deserialization bomb vectors |
| `logger/` | Log strings, user data | Log injection, ANSI escape |
| `config/` | Env vars, YAML keys | Path traversal in key lookup, env exposure |
| `compression/` | Arbitrary byte streams | Decompression bomb |

---

## 🎯 Security Goals & Threat Model

1. **G1** — Key material MUST NOT leak via debug output, log files, or file permissions.
2. **G2** — Time-based anti-replay MUST NOT silently admit replays when the clock is misconfigured.
3. **G3** — File operations on sensitive paths MUST maintain least-privilege permissions atomically.
4. **G4** — Deserialization and decompression MUST be bounded to prevent DoS bombs.
5. **G5** — Log state MUST NOT be corruptible by attacker-supplied strings.
6. **G6** — RNG interfaces MUST communicate unambiguously whether output is unpredictable.
7. **G7** — Memory locking primitives MUST zeroize before unlock (defense against swap residue).

---

## ⚠️ Findings Summary

| ID | Sev | Component | Title |
|---|---|---|---|
| F-01 | **S2** | `time/traits.rs` | `unix_timestamp()` silent 0 fallback violates G2 |
| F-02 | **S2** | `io/fs.rs` | `write_file()` permission preservation silently discarded |
| F-03 | **S3** | `os_hardening.rs` | `LockedBytes` exposes raw address in fmt output |
| F-04 | **S3** | `logger/file_logger.rs` | `sanitize_message` omits ANSI escape sequences |
| F-05 | **S3** | `config/env.rs` | No allowlist on `EnvConfig` — arbitrary env var readable |
| F-06 | **S3** | `rng/traits.rs` | `DeterministicRngProvider` requires `CryptoRng` bound — semantic mismatch |
| F-07 | **S4** | `rng/mock.rs` | `with_u64_seed` caps entropy at 2^64 despite SHA-256 output |
| F-08 | **S4** | `rng/` | `MockRngProvider` (StdRng) vs `DeterministicRngProvider` (ChaCha20) inconsistency |
| F-09 | **S4** | `codec/yaml.rs` | `TrailingBytes` error reports `consumed: 0` |
| F-10 | **S4** | `time/format.rs` | `format_system_time_local()` mixes local timezone with UTC log stream |
| F-11 | **S4** | `logger/structured.rs` | `encode_event` failure substitutes silent sentinel token |
| F-12 | **S4** | `io/fs.rs` | `write_file()` (non-private) has no `fsync`/`sync_all` |
| F-13 | **S4** | `os_hardening.rs` | `munlock` failure silently dropped in `Drop` impl |

---

## 🔑 Detailed Findings

---

### ⭐ F-01 · S2 — `unix_timestamp()` Silent 0 on Clock Error

**Component:** `src/time/traits.rs`

**Code (reconstructed):**
```rust
fn unix_timestamp(&self) -> u64 {
    self.try_unix_timestamp().unwrap_or(0)
    // ^ same pattern for unix_timestamp_millis() and unix_timestamp_micros()
}
```

**Problem:** When `SystemTime::now()` precedes UNIX_EPOCH (misconfigured NTP, VM snapshot restore, negative time drift), `unix_timestamp()` silently returns `0`. There is no compile-time or lint-time signal preventing callers from using this method in security-critical paths. Callers in the rest of the workspace that use this value for:
- nonce construction (`timestamp || random`)
- expiry comparison (`if now > expiry { reject }`)
- anti-replay window (`if timestamp + window < now { reject }`)

…will receive `0`, causing nonce collisions or bypassing replay-window checks.

The doc comment says "prefer `try_unix_timestamp()` in security-critical code" but this is advisory text, not enforced.

**Severity Justification:** Not directly exploitable without a co-occurring clock-rollback event, but a misconfigured VM or restored snapshot causes silent degradation with no error propagation.

**Fix:**
```rust
// Option A: change return type (recommended for critical paths)
fn unix_timestamp(&self) -> Result<u64, TimeError>;

// Option B: add a separate panicking variant and document the fallback clearly
fn unix_timestamp_or_panic(&self) -> u64 {
    self.try_unix_timestamp().expect("system clock is before Unix epoch")
}

// Option C: cfg-gated debug_assert (minimal change)
fn unix_timestamp(&self) -> u64 {
    let ts = self.try_unix_timestamp().unwrap_or(0);
    debug_assert!(ts > 0, "unix_timestamp returned 0 — check system clock");
    ts
}
```

**Recommended:** Adopt Option A for `z00z_core` callers. Keep the `unwrap_or(0)` only for non-security log/metric usage with an explicit method rename to `unix_timestamp_lossy()`.

---

### ⭐ F-02 · S2 — `write_file()` Permission Preservation Silently Discarded

**Component:** `src/io/fs.rs`

**Code:**
```rust
if let Ok(meta) = std::fs::metadata(path) {
    let _ = temp.as_file().set_permissions(meta.permissions()); // silently ignored
}
```

**Problem:** When overwriting an existing file, the code attempts to copy the original file's permissions onto the temp file before renaming. If `set_permissions` fails (insufficient privilege, unsupported filesystem), the error is silently discarded via `let _ = ...`. The temp file retains its default filesystem permissions (typically umask-derived, e.g., 0o644). After `persist()` (rename), the overwritten file now has relaxed permissions.

This is the **non-private** variant (`write_file`). Callers using `write_file()` for any file that began life as restrictive (e.g., config files written by a previous `atomic_write_file_private()`) may inadvertently expose content on the next write cycle.

**The private variant** (`atomic_write_file_private`) is correctly implemented: it explicitly sets 0o600, calls `sync_all()`, and propagates errors on permission failure. **`write_file()` should not exist as a drop-in replacement without this awareness.**

**Fix:**
```rust
// Propagate the permission error instead of swallowing it:
if let Ok(meta) = std::fs::metadata(path) {
    temp.as_file().set_permissions(meta.permissions())
        .map_err(IoError::Io)?;
}
```

If backward compatibility requires silent best-effort, rename the method to `write_file_best_effort_perms()` and add a `#[deprecated]` pointing to `atomic_write_file_private` for sensitive files.

---

### 💥 F-03 · S3 — `LockedBytes` Leaks Raw Memory Address in `fmt`

**Component:** `src/os_hardening.rs`

**Code:**
```rust
// inferred from struct fields seen — Debug fmt would expose addr
struct LockedBytes {
    addr: usize,  // raw pointer stored as usize
    len: usize,
    active: bool,
}
```

**Problem:** If `LockedBytes` (or any wrapper that derives/implements `Debug`) prints the `addr` field, the virtual address of mlock'd memory is emitted into logs or error messages. This is an ASLR information leak: an attacker who can observe logs or panic output learns the heap address of key material, allowing targeted heap-spray or return-oriented programming if combined with a memory-corruption bug.

The struct is already security-sensitive (it holds locked memory containing secrets). Its debug representation should be opaque.

**Fix:**
```rust
impl fmt::Debug for LockedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LockedBytes")
            .field("len", &self.len)
            .field("active", &self.active)
            // addr intentionally omitted — ASLR protection
            .finish()
    }
}
```

---

### 🚨 F-04 · S3 — `sanitize_message` Missing ANSI Escape Sequences

**Component:** `src/logger/file_logger.rs`, `src/logger/rotating_file_logger.rs`

**Code:**
```rust
fn sanitize_message(msg: &str) -> String {
    msg.replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\0', "")
    // Missing: '\x1b' (ESC) and other C0/C1 control characters
}
```

**Problem:** ANSI escape sequences (`\x1b[...`) are not stripped. An attacker who controls a log message (e.g., through a user-supplied transaction memo, peer-reported error, or network message) can inject terminal control codes into the log file. When a human views the log in a terminal (e.g., `tail -f app.log`), the injected sequences can:
- Clear the terminal or hide subsequent log lines
- Re-color output to mask anomalous entries
- Exfiltrate terminal clipboard on some terminals (OSC52)
- Change terminal title or tab name (social engineering)

This is a **log injection** attack (OWASP A03:2021 Injection).

**Fix:**
```rust
fn sanitize_message(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len());
    for ch in msg.chars() {
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\0' => {}  // drop null bytes
            // Drop all C0 control chars except printable escapes already handled
            c if (c as u32) < 0x20 || c == '\x7f' => {
                // Replace with unicode escape notation
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}
```

For production: consider using the `strip-ansi-escapes` crate (MIT) or `console::strip_ansi_codes`.

---

### ⚠️ F-05 · S3 — `EnvConfig` Has No Allowlist

**Component:** `src/config/env.rs`, `src/config/layered.rs`

**Problem:** `EnvConfig::get(key)` retrieves environment variables by name. The `LayeredConfig` resolves keys using ENV overrides first. If a config consumer uses user-supplied or config-file-derived key strings as lookup keys against a `LayeredConfig`, an attacker who controls the key name can read the value of any environment variable including:
- `LD_PRELOAD` / `LD_LIBRARY_PATH` — dynamic linker paths
- `HOME`, `PATH` — user environment
- `DATABASE_URL`, `SECRET_KEY` — injected CI/CD secrets

**Scenario:** A YAML config file says:
```yaml
plugin_config_key: "LD_PRELOAD"
```
If the application evaluates `config.get(&yaml_value("plugin_config_key"))` by first resolving the YAML value to `"LD_PRELOAD"` and then calling `env_config.get("LD_PRELOAD")`, secrets leak.

**Fix options:**
1. **Prefix allowlist:** only permit vars with a project-specific prefix (e.g., `Z00Z_`):
   ```rust
   const ENV_PREFIX: &str = "Z00Z_";
   pub fn get(&self, key: &str) -> Option<String> {
       let env_key = format!("{}{}", ENV_PREFIX, key.to_uppercase().replace('.', "_"));
       std::env::var(&env_key).ok()
     }
   ```
2. **Explicit allowlist:** take a `HashSet<&'static str>` of permitted keys at construction time.
3. **Document the risk** and require callers to normalize keys before lookup.

---

### ⚠️ F-06 · S3 — `DeterministicRngProvider` Requires `CryptoRng` Bound

**Component:** `src/rng/traits.rs`

**Code (inferred):**
```rust
pub trait DeterministicRngProvider {
    type Rng: RngCore + CryptoRng + Send;  // CryptoRng bound on deterministic trait
    fn rng(&self) -> Self::Rng;
}
```

**Problem:** The `CryptoRng` marker trait from `rand` explicitly signals "output is cryptographically unpredictable." Applying it to `DeterministicRngProvider` is semantically incorrect: the output IS predictable — that is the entire point of the deterministic variant (reproducible genesis, test harnesses). A caller who sees `Rng: CryptoRng` may legitimately conclude it is safe to use for key generation, nonce production, or signature randomness. This would be a serious mistake.

**Note:** ChaCha20 does satisfy `CryptoRng` mathematically (it is an approved CSPRNG when seeded properly), but using it with a known/fixed seed defeats the unpredictability requirement that `CryptoRng` communicates.

**Fix:**
```rust
pub trait DeterministicRngProvider {
    // Remove CryptoRng from the bound — deterministic output is NOT unpredictable
    type Rng: RngCore + Send;
    fn rng(&self) -> Self::Rng;
}
```

Add a doc comment:
```rust
/// WARNING: Output is fully deterministic from the seed.
/// Do NOT use for key generation, nonces, or any security-critical randomness.
/// Use `SecureRngProvider` for cryptographically unpredictable output.
```

---

### 🐞 F-07 · S4 — `with_u64_seed` Effective Entropy Limited to 2^64

**Component:** `src/rng/mock.rs`

**Code:**
```rust
pub fn with_u64_seed(seed: u64) -> Self {
    let mut seed_bytes = [0u8; 32];
    seed_bytes[..8].copy_from_slice(&seed.to_le_bytes()); // u64 in first 8 bytes
    let hash = Sha256::digest(seed_bytes);                 // rest 24 bytes are zero
    Self { seed: hash.into() }
}
```

**Problem:** Although SHA-256 produces 256-bit output, the input is always `u64 || 0x00...00` (8 bytes of seed + 24 zero bytes). The effective seed space is 2^64. Two collision implications:
1. An exhaustive search of the 2^64 seed space can recover original u64 from SHA-256 digest via preimage (for small u64 values in tests, practically trivial).
2. No domain separation label (e.g., `"z00z_mock_rng_v1" || seed`) means if SHA-256 is used elsewhere with similar patterns, cross-context confusion is possible.

**Severity:** S4 because this method is test-only (the `compile_error!` guard prevents non-test use). Not exploitable in production.

**Improvement (optional for test code quality):**
```rust
pub fn with_u64_seed(seed: u64) -> Self {
    const DOMAIN: &[u8] = b"z00z_mock_rng_v1:";
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    hasher.update(seed.to_le_bytes());
    Self { seed: hasher.finalize().into() }
}
```

---

### 🐞 F-08 · S4 — MockRngProvider (StdRng) vs DeterministicRngProvider (ChaCha20) Inconsistency

**Component:** `src/rng/mock.rs`, `src/rng/deterministic.rs`

**Problem:** Two "deterministic" RNG providers use different backing algorithms:
- `MockRngProvider` → `StdRng` (platform-dependent algorithm, currently HC-128 on 32-bit or ChaCha20 on 64-bit in `rand 0.8`)
- `DeterministicRngProvider` → `ChaCha20Rng` (explicit, stable)

`StdRng`'s algorithm is **not guaranteed to be stable across rand versions** (as documented by the `rand` crate). This means `MockRngProvider::with_u64_seed(42)` may produce different byte sequences after a `rand` crate upgrade, breaking reproducibility of tests that rely on specific output values.

**Fix:** Change `MockRngProvider` to use `ChaCha20Rng` for consistent behavior:
```rust
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

impl DeterministicRng for MockRngProvider {
    type Rng = ChaCha20Rng;
    fn rng(&self) -> ChaCha20Rng {
        ChaCha20Rng::from_seed(self.seed)
    }
}
```

---

### 🐞 F-09 · S4 — YAML `TrailingBytes` Error Reports `consumed: 0`

**Component:** `src/codec/yaml.rs`

**Problem:** When the YAML codec detects a second document (multi-document YAML attack vector), it returns a `TrailingBytes` error. The `consumed` field is reported as `0` rather than the actual byte offset where the second document begins. This makes forensic debugging of malformed payloads harder and may confuse log parsers that expect non-zero `consumed` values for trailing-byte errors.

**Fix (informational):** Track the byte count of the first document and populate `consumed` correctly, or document that `consumed` is always 0 for YAML multi-document detection.

---

### 🐞 F-10 · S4 — Local-Timezone Format Function Mixes Into UTC Log Stream

**Component:** `src/time/format.rs`

**Problem:** `format_system_time_local()` formats timestamps in the local timezone. If any log consumer (file reader, ELK/Grafana, SIEM) expects all log timestamps to be UTC, mixing in local-timezone entries causes ordering errors and event-correlation failures. In cloud deployments where the server timezone differs from the operator's timezone, timestamps may appear to go backward or forward unexpectedly.

**Fix:** Deprecate `format_system_time_local()` for any log-facing use. Use only `format_system_time_utc()` in the logger components. If local time is needed (UI display), perform the conversion at the presentation layer, not at the log-write layer.

---

### 🐞 F-11 · S4 — Structured Logger Silently Substitutes Missing Events

**Component:** `src/logger/structured.rs`

**Code (paraphrased):**
```rust
fn encode_event(event: &dyn LogEvent) -> Option<String> {
    // On serde failure → returns None
    // Caller substitutes {"event":"logger.serialize_error"}
}
```

**Problem:** When structured event serialization fails, a sentinel JSON object `{"event":"logger.serialize_error"}` is written instead of an error. Consumers parsing structured logs will see anomalous entries without sufficient context to identify which event type failed or why. Security-relevant events (audit trail entries, transaction events) could be silently dropped without triggering alerts.

**Fix:** Log the event type name alongside the serialize error, and consider a fallback plain-text representation. Consider making serialization failures emit a log to the standard error channel for operator visibility.

---

### 🐞 F-12 · S4 — `write_file()` Has No `sync_all`/`fsync`

**Component:** `src/io/fs.rs`

**Problem:** The non-private `write_file()` function performs atomic rename via `NamedTempFile::persist()` but does not call `file.sync_all()` or `file.flush()` before the rename (only `temp.flush()` is called, which flushes the Rust `BufWriter` layer but not the OS page cache). On a system crash or power loss after `persist()` returns but before the OS writes to disk, the new file may contain stale or zero-byte data.

For non-sensitive files (configs, logs), this is low risk. For any file that a higher-level caller accidentally routes through `write_file()` instead of `atomic_write_file_private()`, it violates durability expectations.

**Fix (optional):** Add `temp.as_file().sync_all()?;` before `temp.persist(path)` in `write_file()`, mirroring `atomic_write_file_private`.

**Note:** The streaming variant `atomic_write_file_streaming` appears to handle sync correctly (based on code structure observed).

---

### 🐞 F-13 · S4 — `munlock` Failure Silently Dropped in `LockedBytes::Drop`

**Component:** `src/os_hardening.rs`

**Code:**
```rust
if unsafe { munlock(ptr, self.len) } != 0 {
    let _ = HardeningError::Munlock(std::io::Error::last_os_error());
    // ^ error constructed but immediately discarded
}
```

**Problem:** The `HardeningError` is constructed but the `let _` binding immediately drops it without recording or acting on the failure. If `munlock` fails (e.g., the kernel already swapped out the pages), the caller has no way to know, and the memory may have been paged out while it was supposed to be locked. For a higher-severity protocol: log the error to a best-effort channel or increment a counter.

**Fix:**
```rust
if unsafe { munlock(ptr, self.len) } != 0 {
    // In Drop we cannot propagate errors, but we can emit a best-effort log
    eprintln!("[z00z_utils::os_hardening] munlock failed: {}",
        std::io::Error::last_os_error());
}
```

---

## ✅ Positive Controls (What Is Done Well)

| Control | Location | Assessment |
|---|---|---|
| `atomic_write_file_private` with 0o600, `sync_all`, parent dir sync | `io/fs.rs` | ⭐ Excellent — full crash-safety for key material |
| `ZeroizeOnDrop` on all RNG seed fields | `rng/mock.rs`, `rng/deterministic.rs` | ✅ Correct — key material zeroed on eviction |
| `compile_error!` guard on `MockRngProvider` | `rng/mock.rs` | ✅ Prevents test-only types from reaching production |
| Symlink rejection before file open | `io/fs.rs`, `logger/file_logger.rs` | ✅ TOCTOU symlink attack prevention |
| Bounded deserialization on Bincode, JSON, YAML | `codec/*.rs` | ✅ Decompression/deserialization bomb prevented |
| Bounded decompression with extra-byte probe | `compression.rs` | ✅ Correct bomb detection pattern |
| LZ4 magic number check | `compression.rs` | ✅ Format confusion prevention |
| `LockedBytes::drop` zeroizes BEFORE `munlock` | `os_hardening.rs` | ✅ Correct ordering — memory cleared before unlocking |
| `setrlimit(RLIMIT_CORE, 0)` + `PR_SET_DUMPABLE` | `os_hardening.rs` | ✅ Core dump and ptrace-attach hardening |
| MockRngProvider debug prints `<redacted>` for seed | `rng/mock.rs` | ✅ Seed not leaked in debug output |
| YAML multi-document rejection | `codec/yaml.rs` | ✅ Prevents multi-doc parsing confusion attacks |
| JSON trailing bytes detection via stream iterator | `codec/json.rs` | ✅ Prevents extra-field injection |
| `read_file_bounded` with `take(max+1)` overflow check | `io/fs.rs` | ✅ File-size bomb prevention |
| 0o600 enforcement on FileLogger Unix open | `logger/file_logger.rs` | ✅ Log file confidentiality protected |

---

## 🔑 Implementation Guidance

### Priority 1 (Required — S2 fixes)

1. **`time/traits.rs`** — Rename `unix_timestamp()` to `unix_timestamp_lossy()` or change return to `Result<u64, TimeError>`. Any nonce/expiry/anti-replay caller in `z00z_core` MUST call `try_unix_timestamp()` and handle the error explicitly.

2. **`io/fs.rs`** — Change `let _ = temp.as_file().set_permissions(...)` to propagate the error:
   ```rust
   temp.as_file().set_permissions(meta.permissions()).map_err(IoError::Io)?;
   ```

### Priority 2 (Recommended — S3 fixes)

3. **`os_hardening.rs`** — Suppress `addr` field in `LockedBytes::Debug`.

4. **`logger/*.rs`** — Extend `sanitize_message` to strip all C0 control characters (`< 0x20` and `0x7f`) and known ANSI sequences.

5. **`config/env.rs`** — Add a `Z00Z_` namespace prefix requirement or an explicit allowlist to `EnvConfig`.

6. **`rng/traits.rs`** — Remove `CryptoRng` from `DeterministicRngProvider::Rng` bound and add a prominent doc-level warning.

### Priority 3 (Optional — S4 improvements)

7. **`rng/mock.rs`** — Switch to `ChaCha20Rng` for stable cross-version reproducibility.

8. **`rng/mock.rs`** — Add domain label in `with_u64_seed` SHA-256 input.

9. **`io/fs.rs`** — Add `sync_all` to `write_file()` for durability parity with the private variant.

---

## 🧪 Test Plan

| Test ID | Coverage Target | Approach |
|---|---|---|
| T-01 | F-01: clock fallback | `MockTimeProvider` set before epoch → verify `try_unix_timestamp()` returns `Err`, `unix_timestamp_lossy()` returns 0 |
| T-02 | F-02: permission preservation fail | Create 0o600 file, cause `set_permissions` to fail (e.g., read-only mount mock), verify existing permissions preserved or error returned |
| T-03 | F-03: addr hidden in Debug | Create `LockedBytes` instance → `format!("{:?}", guard)` → assert does not contain hex address |
| T-04 | F-04: ANSI injection | `sanitize_message("\x1b[31mred\x1b[0m")` → assert no `\x1b` in output |
| T-05 | F-05: env var allowlist | `EnvConfig::get("PATH")` with allowlist enabled → returns `None` |
| T-06 | F-06: CryptoRng removed | Compile-time trait bound check via `cargo check` after removing bound |
| T-07 | F-07: seed uniqueness | `MockRngProvider::with_u64_seed(0)` and `with_u64_seed(1)` → assert different first bytes |
| T-08 | F-08: ChaCha20 stability | After algorithm change, `with_u64_seed(42)` first 32 output bytes match reference vector |
| T-09 | `atomic_write_file_private` 0o600 | Write file, stat it, assert mode 0o600 on Unix |
| T-10 | Decompression bomb | `zstd_decompress_bounded(bomb, 1024)` → `Err(OutputTooLarge)` |
| T-11 | Symlink rejection | Create symlink at log path → `FileLogger::new` returns `Err` |
| T-12 | ZeroizeOnDrop | Drop `DeterministicRngProvider` holding known seed → assert seed bytes zero via unsafe inspection (test-utils only) |

---

## ❓ Open Ambiguities

1. **`atomic_write_file_streaming` on non-Unix:** Falls back to `write_file` on Windows (non-Unix path). Verify that the streaming fallback on Windows also enforces appropriate ACL permissions for sensitive files. Not verified in this audit (no Windows code path available).

2. **`LockedBytes` lifetime soundness:** The struct stores `addr: usize` (a raw pointer). The drop impl reconstructs a `*mut u8` slice. This is sound IF the underlying buffer lives longer than `LockedBytes`, but this is enforced only by the caller (a raw pointer cannot encode lifetime in Rust). A formal review of all `lock_bytes_best_effort` call sites in `z00z_wallets` and `z00z_storage` is outside this audit scope but is recommended.

3. **`compression.rs` streaming variants:** `zstd_encode_to_writer` and `zstd_decode_to_writer_bounded` were not fully reviewed for size-limit enforcement. The streaming API accepts an `impl Write` output — verify that the `max_output` bound is respected in the streaming path with the same `take + extra-byte probe` pattern used in the buffered path.

4. **`prometheus` feature flag:** The `prometheus` optional dependency is declared in `Cargo.toml` but `metrics/noop.rs` serves as the default implementation. It is unclear whether enabling the `prometheus` feature introduces any additional attack surface (HTTP exposition, bind addr configuration). Not in scope for this audit but should be reviewed before enabling in production.

---

## 💯 Confidence Assessment

| Finding | Confidence | Basis |
|---|---|---|
| F-01 (clock fallback) | **High (95%)** | Source read + trait contract analysis |
| F-02 (permission silent drop) | **High (95%)** | `let _ = ...` pattern directly observed |
| F-03 (addr in fmt) | **Medium (75%)** | Struct fields inferred; custom fmt may already suppress it — verify `impl Debug` manually |
| F-04 (ANSI sanitize) | **High (95%)** | `sanitize_message` body read in full |
| F-05 (env allowlist) | **Medium (80%)** | `EnvConfig::get` body not fully shown; inferred from module structure and layered priority logic |
| F-06 (CryptoRng bound) | **High (90%)** | Trait definition directly read |
| F-07–F-08 (mock RNG) | **High (95%)** | `mock.rs` read in full |
| F-09–F-13 (S4 items) | **Medium–High (70–90%)** | Based on source analysis; some require runtime verification |

---

## 📌 Dependency Versions (Cargo.toml Snapshot)

```toml
rand        = "0.8"       # StdRng algorithm not stable — see F-08
rand_chacha = "0.3"
sha2        = "0.10"
zeroize     = "1.7"
bincode     = "2.0.1"     # Size-limit API changed significantly from v1 — review migration risk
serde_yml   = "0.0.12"    # Not serde_yaml — different crate, verify trait compatibility
tempfile    = "3.10"
zstd        = "0.13"
lz4         = "1"
lz4_flex    = "0.11"      # Two lz4 crates present — verify only one used per path
libc        = "0.2"
rustix      = "1"         # Modern alternative to libc for mm/process — good choice
```

**Notable:** `serde_yml = "0.0.12"` — this is the community fork of the archived `serde_yaml` library. It is not the original `serde_yaml` crate. This addition is intentional (as the crate re-exports `YamlValue` and `from_yaml_value` adapters) but warrants tracking for upstream security patches.

Two LZ4 crates (`lz4` and `lz4_flex`) are present as dependencies. Verify that `compression.rs` consistently uses one of them — mixing implementations can produce output one decoder cannot read.

---

## 🎯 Conclusion

`z00z_utils` is a security-conscious utility crate with strong fundamentals: no critical flaws, correct use of atomic writes, proper zeroization, bounded I/O, and meaningful OS hardening. The two S2 findings (clock fallback and permission preservation) are fixable in a single PR without API breakage. The S3 set (ANSI log injection, ASLR address leak, env allowlist, misleading CryptoRng bound) represents defense-in-depth hardening that should precede production deployment of any wallet or node component that depends on this crate.

> 🏁 **Recommended action:** Address F-01 and F-02 as blocking PRs. Schedule F-03 through F-06 for the next hardening sprint. Treat F-07 through F-13 as technical debt issues.
