---
phase: 061-Wallet-Refactoring
plan: 061-07
status: complete
completed_at: 2026-06-23
next_plan: 061-08
summary_artifact_for: .planning/phases/061-Wallet-Refactoring/061-07-PLAN.md
---

# 061-07 Summary: Receiver, Persistence, And Security Vault Flattening

## Completed Scope

`061-07` is complete for the receiver, persistence, and security-vault
flattening slice.

This slice flattened the owned local trees into one-level domain files, kept
the public facades readable, and moved the password denylist assets out of
`src/`:

- The receiver subtree is now one-level under
  `crates/z00z_wallets/src/receiver/*.rs`.
- The persistence subtree is now one-level under
  `crates/z00z_wallets/src/persistence/*.rs`.
- The security vault subtree is now flat under
  `crates/z00z_wallets/src/security/{vault.rs,vault_*.rs}`.
- `claim_own.rs` was renamed to the canonical
  `stealth_ownership_check.rs` lane.
- `nfc_utils.rs` was renamed to the canonical `nfc_ndef.rs` lane.
- The password source corpus and Bloom artifact moved from
  `src/security/` to `crates/z00z_wallets/config/security/`, and both
  `security/password.rs` and `bin/gen_password_bloom.rs` now point at that
  non-`src/` home.
- The duplicate receipt and scan storage shims were removed only after
  reference-backed proof that no live module declarations, docs, or tests still
  depended on them. Because the duplicate pairs were not byte-identical, the
  removal decision stayed anchored to live-reference proof plus `061-TODO.md`
  authority rather than a content-equality shortcut.
- Path-sensitive docs, rename guards, and the live Phase 061 context or plan
  packet were updated atomically to the flattened tree.

## Files Changed

- `crates/z00z_wallets/src/receiver/*.rs`
- `crates/z00z_wallets/src/persistence/*.rs`
- `crates/z00z_wallets/src/security/password.rs`
- `crates/z00z_wallets/src/security/vault.rs`
- `crates/z00z_wallets/src/security/vault_*.rs`
- `crates/z00z_wallets/config/security/common-passwords.txt`
- `crates/z00z_wallets/config/security/password_denylist.bloom`
- `crates/z00z_wallets/bin/gen_password_bloom.rs`
- `crates/z00z_wallets/tests/test_rename_guards.rs`
- `crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md`
- `crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md`
- `.planning/phases/061-Wallet-Refactoring/061-CONTEXT.md`
- `.planning/phases/061-Wallet-Refactoring/061-07-PLAN.md`

Retired old paths in this slice:

- `crates/z00z_wallets/src/receiver/card/**`
- `crates/z00z_wallets/src/receiver/manager/**`
- `crates/z00z_wallets/src/receiver/ownership/**`
- `crates/z00z_wallets/src/receiver/request/**`
- `crates/z00z_wallets/src/receiver/scan/**`
- `crates/z00z_wallets/src/persistence/assets/**`
- `crates/z00z_wallets/src/persistence/receipts/**`
- `crates/z00z_wallets/src/persistence/scans/**`
- `crates/z00z_wallets/src/persistence/tx/**`
- `crates/z00z_wallets/src/security/vault/**`
- `crates/z00z_wallets/src/security/common-passwords.txt`
- `crates/z00z_wallets/src/security/password_denylist.bloom`

## Boundary Kept

- The top-level `receiver`, `persistence`, and `security::vault` facades remain
  the canonical local entrypoints.
- This slice did not reopen wallet-service, RPC-method, or key-tree ownership
  beyond the path updates needed to follow the flattened files.
- Receiver scan and ownership behavior stayed on the same runtime path; the
  work was structural and naming-boundary cleanup, not a semantic redesign.
- Receipt or scan duplicate removal was guarded by live-reference proof rather
  than inferred from matching filenames.

## Review Loop

Manual fallback for `.github/prompts/gsd-review-tasks-execution.prompt.md` was
used because the slash prompt is not a callable tool in this environment.

- Pass 1 audited live code and docs for stale nested receiver or persistence
  paths, stale `security/vault/` file references, and the removed
  `claim_own` / `nfc_utils` names. The live tree was clean and the flat-file
  inventory had no nested receiver or persistence Rust survivors.
- Pass 2 audited the rename-guard and password-asset surfaces. The tests now
  assert the new canonical `receiver/*.rs`, `persistence/*.rs`,
  `security/vault.rs`, and `config/security/*` paths, and the runtime loader
  plus generator both point at the same new asset home.
- Pass 3 audited the path-attr rewiring and final flat-root truth. The flat
  module roots, receiver/persistence tree depth, and touched-file whitespace
  were all clean.

Two consecutive clean review passes were achieved on passes 2 and 3.

## Validation

- Mandatory bootstrap gate passed before broader validation:
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh`
- `cargo check --release -p z00z_wallets --all-targets --all-features` passed.
- `cargo test --release -p z00z_wallets --all-targets --all-features` passed.
- `git diff --check -- crates/z00z_wallets/src/receiver crates/z00z_wallets/src/persistence crates/z00z_wallets/src/security crates/z00z_wallets/bin/gen_password_bloom.rs crates/z00z_wallets/tests/test_rename_guards.rs crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md 'crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md' .planning/phases/061-Wallet-Refactoring/061-CONTEXT.md .planning/phases/061-Wallet-Refactoring/061-07-PLAN.md`
  is clean.
- `find crates/z00z_wallets/src \( -type f -path '*/receiver/*/*' -o -type f -path '*/persistence/*/*/*' \) | sort`
  returned no live nested receiver or persistence Rust files.
- `find crates/z00z_wallets/config/security -maxdepth 1 -type f | sort`
  confirmed the canonical password asset home.
- `rg -n "claim_own|nfc_utils|src/security/password_denylist|src/security/common-passwords|src/security/vault/|src/receiver/(card|manager|ownership|request|scan)/|src/persistence/(assets|receipts|scans|tx)/" crates/z00z_wallets/src crates/z00z_wallets/bin crates/z00z_wallets/src/key/bip/docs/KEYS_EXPALNATION.md 'crates/z00z_wallets/🔐-разбор-кошелька-Z00Z.md' -g '*.rs' -g '*.md'`
  returned no live-code or live-doc matches.

## Result

`061-07` is complete. Phase 061 advances to `061-08-PLAN.md` for the key-tree
flattening, BIP anchor preservation, and key-doc synchronization wave while
keeping one canonical local path per receiver, persistence, and security-vault
behavior.
