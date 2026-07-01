# 051-03 Summary: API Guardrails And Downstream Semantic Authority

## Objective

Close `051-03-PLAN.md` by making physical backend details non-authoritative at
public and downstream surfaces. Backend-root bytes remain diagnostic or
proof-local only, paired with `AssetStateRoot` and `root_bind`.

## Code Changes

- Documented `ProofBlob::new`, `ProofBlob::backend_root()`,
  `ProofScanOut`, and `ProofScanOut::backend_root()` so backend-root bytes are
  explicitly diagnostic or proof-local, not checkpoint or state authority.
- Documented wallet `AssetClassAuditReport::backend_root` as diagnostic
  proof-local evidence that must stay paired with `semantic_root` and
  `root_bind`.
- Added an authority comment at the wallet audit equality check: backend-root
  equality is consistency-only, while semantic root and root-bind verification
  remain authoritative.
- Added source-shape tests proving public storage exports do not leak `TreeId`,
  `ns_key`, raw JMT proof types, or physical root hash helpers.
- Added downstream guard tests proving validator, wallet, and simulator proof
  consumers continue to use storage-owned proof APIs instead of local backend
  branch semantics.

## Test Coverage

- `test_public_surface_guard` proves the public `z00z_storage::assets` surface
  does not export `TreeId`, `ns_key`, `SparseMerkleProof`, or raw root-hash
  helpers, and that backend-root rustdoc contains diagnostic/proof-local
  authority wording.
- `test_downstream_layout_guard` proves selected validator, wallet, and
  simulator consumers do not import raw `TreeId`, namespace helpers, raw JMT
  proof types, raw branch proof accessors, or `jmt::` implementation details.
- The same downstream guard also freezes the wallet audit shape around
  `semantic_root: AssetStateRoot`, `backend_root: [u8; 32]`,
  `root_bind: [u8; 32]`, and `check_root_bind()`.

## Review Loop

- Task `Add public-surface guardrails`, pass 1 found a brittle exact-source
  assertion in `test_public_surface_guard`; it was fixed by checking the
  required authority words separately after rustfmt line wrapping.
- Task `Add public-surface guardrails`, passes 2 and 3 found no significant
  remaining issues, giving two consecutive clean significant-issue passes.
- Task `Cut validator wallet and simulator consumers to semantic authority`,
  pass 1 confirmed downstream consumers use `ProofBlob::decode`, `chk_blob`,
  `backend_root()`, `root_bind()`, and `check_root_bind()` through storage-owned
  proof surfaces, with no forbidden layout imports in the selected files.
- Task `Cut validator wallet and simulator consumers to semantic authority`,
  passes 2 and 3 found no significant remaining issues, giving two consecutive
  clean significant-issue passes.
- Follow-up doublecheck found no `TODO`, `FIXME`, open checklist markers, or
  stale suffixed claim-source proof spelling in the 051-03 plan/code slice.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  before 051-03 Rust/test changes; the bootstrap script runs its cargo gates
  with `--release`.
- After the test-affecting assertion fix,
  `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` was rerun
  and passed before any broader validation.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump --test test_assets_suite -- --nocapture`
  passed, including `test_public_surface_guard` and
  `test_downstream_layout_guard`.
- A mistaken focused selector using `--test test_assets` failed because that
  test target does not exist; it was corrected to the real release target
  `test_assets_suite` before accepting validation.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed for the workspace; cargo reported the release profile and
  `target/release/deps` test binaries.
- `cargo fmt --check` exited 0; rustfmt printed only the repository's existing
  nightly-only configuration warnings.
- `git diff --check` passed.
- A repository search for the stale suffixed claim-source proof spelling found
  no matches; the live claim-source proof contract spelling remains
  `ClaimSourceProof`.

## Result

`051-03` is summary-backed complete. Public and downstream surfaces now have
source-shape guardrails against physical backend coupling, and backend-root
bytes are documented and tested as diagnostic/proof-local evidence only. The
next execution lane is `051-04-PLAN.md`.
