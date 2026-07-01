---
phase: 042-refactor-wallets
reviewed: 2026-05-03T17:09:43Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - crates/z00z_wallets/src/lib.rs
  - crates/z00z_wallets/src/core/mod.rs
  - crates/z00z_wallets/src/db/mod.rs
  - crates/z00z_wallets/src/services/mod.rs
  - crates/z00z_wallets/src/core/key/seed/seed_mnemonic.rs
  - crates/z00z_wallets/src/core/key/bip/bip32_key_deriver.rs
  - crates/z00z_wallets/src/core/key/bip/bip32_path_validator.rs
  - crates/z00z_wallets/src/core/key/bip/bip32_ristretto_bridge.rs
  - crates/z00z_wallets/src/core/key/bip/mod.rs
  - crates/z00z_wallets/src/core/persistence/mod.rs
  - crates/z00z_wallets/src/core/persistence/tx/mod.rs
findings:
  critical: 0
  warning: 0
  info: 6
  total: 6
status: issues_found
build_gate: pass
---

# Phase 042 Row06 Code Review Report

**Reviewed:** 2026-05-03T17:09:43Z
**Depth:** standard
**Files Reviewed:** 11
**Build Gate:** `cargo test -p z00z_wallets --no-run` — PASS (0 errors, 2 warnings)
**Status:** issues_found

## Summary

All 11 files reviewed. No critical vulnerabilities or logic bugs found. The previous
pass (042-05) surfaced 2 WARNINGs and 4 INFOs; 5 of those 6 findings are fully
resolved. The residual from WR-01 (mnemonic phrase zeroization gap) is partially
addressed: bytes ARE now zeroized via `Hidden<String>` on drop, closing the
security gap, but the `Zeroizing` wrapper added as a fix is rendered a no-op by
`mem::take`, leaving misleading code. Downgraded to INFO.

Four new INFO-level findings reported: one suppressed compiler warning on a dead
re-export, one misleading `compile_error!` message, one undocumented empty-slice
contract in key derivation, and one compiler warning on an unused test constant.

---

## Prior Findings — Resolution Tracking

| Finding | Description | 042-06 Status |
| ------- | ----------- | ------------- |
| WR-01 | `SeedWords::join()` unzeroized intermediate String | Residual → IN-01 below |
| WR-02 | `entropy_from_words` no round-trip tests | FIXED — `test_entropy_roundtrip_all_bip39_sizes` added |
| IN-01 | `// SAFETY:` comment in fully safe code | FIXED — changed to `// Note:` |
| IN-02 | `reject_zero_key` used non-idiomatic `unwrap_u8()` | FIXED — uses `bool::from(key.ct_eq(...))` |
| IN-03 | Redundant `pub use persistence::claim_registry` in storage facade | FIXED — removed; `pub use super::persistence::*` only |
| IN-04 | `validate_seed_phrase` API docs missing zeroization boundary note | FIXED — `# Sensitive Input Boundary` section added |

---

## Info

### IN-01: `SeedWords::join()` — `Zeroizing` wrapper defeated by `mem::take` (residual WR-01)

**File:** `crates/z00z_wallets/src/core/key/seed/seed_mnemonic.rs:72-80`

**Issue:** The `join()` method wraps the phrase buffer in `Zeroizing<String>`, then
immediately extracts the `String` from the wrapper via `core::mem::take(&mut *phrase)`.
After the take, `phrase` holds an empty `String`; its `Zeroizing` drop zeroes nothing
of value. The actual mnemonic bytes end up in `out: String`, then `Hidden::hide(out)`
moves them into `Hidden<String>`. The bytes ARE zeroized correctly when `Hidden` drops,
so the security gap from WR-01 is closed. However, the `Zeroizing` wrapper provides
no additional protection beyond what `Hidden` would provide without it, and the code
pattern misleads readers into believing `phrase` is the secured buffer.

The sibling `join_revealed()` method correctly keeps the phrase in `Zeroizing<String>`
for its full lifetime. The recommended fix from WR-01 (delegate `join` to `join_revealed`)
was not applied.

**Fix:**

```rust
/// Join words with a separator.
///
/// The resulting phrase is wrapped in [`Hidden`] and zeroized on drop.
pub fn join(&self, sep: &str) -> Hidden<String> {
    self.join_revealed(sep, |phrase| Hidden::hide(phrase.to_owned()))
}
```

---

### IN-02: `lib.rs:85` — `compile_error!` message omits `ownership_policy_dual_ok` escape hatch

**File:** `crates/z00z_wallets/src/lib.rs:85`

**Issue:** When both `ownership_policy_keyring` and `ownership_policy_challenge`
features are enabled without `ownership_policy_dual_ok`, the compiler fires:

```text
compile_error!("Enable exactly one: ownership_policy_keyring OR ownership_policy_challenge");
```

The message says "Enable exactly one" but `dual_ok` is the intentional escape hatch
to enable both simultaneously. A developer hitting this error gets no guidance toward
`dual_ok` and may waste time trying to pick one when both are legitimately needed.

**Fix:**

```rust
compile_error!(
    "Enable exactly one: `ownership_policy_keyring` OR `ownership_policy_challenge`. \
     To allow both simultaneously, also enable `ownership_policy_dual_ok`."
);
```

---

### IN-03: `services/mod.rs:30-31` — `#[allow(unused_imports)]` silences dead re-export

**File:** `crates/z00z_wallets/src/services/mod.rs:30`

**Issue:**

```rust
#[allow(unused_imports)]
pub use self::wallet_service::{AddressUsedOracle, Sleeper};
```

The `#[allow(unused_imports)]` suppresses a legitimate compiler warning, hiding that
these are dead re-exports with no callers in the crate. The comment labels them
"Reachability-only integration seams" but provides no citation of which callers rely
on them or when they will be wired. If these are intentional API-stability anchors,
use `#[doc(hidden)]` with an explicit rationale comment; if they are truly unused,
remove them.

**Fix (option A — if keeping for API stability):**

```rust
// API-stability anchor: kept for downstream crates that wire these seams.
// See: <issue/design-doc link>
#[doc(hidden)]
pub use self::wallet_service::{AddressUsedOracle, Sleeper};
```

**Fix (option B — if unused):**

```rust
// Remove the re-export entirely.
```

---

### IN-04: `bip32_key_deriver.rs:132-145` — empty `partial_path` slice silently returns parent clone

**File:** `crates/z00z_wallets/src/core/key/bip/bip32_key_deriver.rs:132`

**Issue:** `derive_from_intermediate(parent, &[])` silently returns `Ok(parent.clone())`.
The loop body is never entered; the caller gets back a clone of the key they passed in.
This is not documented in the function contract. In a BIP-32 context, deriving "zero
steps" is surprising: callers who accidentally pass an empty slice receive the parent
key at a potentially incorrect derivation depth without any error signal. The function
doc says "Partial derivation path" but does not state the empty-slice contract.

**Fix:** Add a contract note to the doc comment:

```rust
/// * `partial_path` - Partial derivation path as slice of ChildNumber.
///   If empty, returns a clone of `parent` unchanged (zero-step derivation).
```

Or, if an empty path is always a caller bug, add a guard:

```rust
if partial_path.is_empty() {
    return Ok(parent.clone());  // documented no-op; callers should verify intent
}
```

---

### IN-05: `assets/mod.rs:6` — missing doc comment on `test_asset_storage_impl_suite` module

**File:** `crates/z00z_wallets/src/core/persistence/assets/mod.rs:6`

**Issue:** The public module `test_asset_storage_impl_suite` has no doc comment.
Under `#![warn(missing_docs)]`, this fires `warning: missing documentation for a module`
in the lib test build. Confirmed by build gate output:

```text
warning: missing documentation for a module
  --> crates/z00z_wallets/src/core/persistence/assets/mod.rs:6:1
   |
6  | pub mod test_asset_storage_impl_suite;
```

**Fix:**

```rust
/// Test suite for [`AssetStorage`] implementations.
pub mod test_asset_storage_impl_suite;
```

---

### IN-06: `test_s5_sender_examples_support.rs:23` — unused constant `ZERO_ROOT`

**File:** `crates/z00z_wallets/tests/test_s5_sender_examples_support.rs:23`

**Issue:** `pub(crate) const ZERO_ROOT: [u8; 32] = [0u8; 32];` is never referenced.
The build gate reports `warning: constant ZERO_ROOT is never used`. Unused test support
constants accumulate as noise and may indicate that a test using this value was removed
or never wired.

**Fix:** Remove the constant, or add a `#[allow(dead_code)]` with a comment explaining
why it is retained:

```rust
// Reserved as a sentinel for future merkle-root absence tests.
#[allow(dead_code)]
pub(crate) const ZERO_ROOT: [u8; 32] = [0u8; 32];
```

---

_Reviewed: 2026-05-03T17:09:43Z_
_Reviewer: gsd-code-reviewer (adversarial, standard depth)_
_Prior pass: 042-05-REVIEW.md_
