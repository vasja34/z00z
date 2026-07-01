# 051-02 Summary: Root Taxonomy And Proof Envelope

## Objective

Close `051-02-PLAN.md` by making root roles explicit and by turning the
compatibility proof envelope into a fail-closed version boundary.

## Code Changes

- Added `COMPAT_PROOF_ENVELOPE_VERSION` as the public compatibility envelope
  version and routed the root-binding version through it.
- Added `CompatProofFamily` and `check_compat_proof_family(...)` so the current
  compatibility verifier accepts inclusion proofs only and rejects deletion or
  non-existence proof families explicitly.
- Added `ProofChkErr::UnsupportedProofFamily` and mapped it through snapshot
  witness preparation as a witness mismatch.
- Documented the compatibility envelope as v1 inclusion-only; bucket policy,
  bucket proof, deletion, and non-existence semantics remain future forest work
  behind the verifier boundary.
- Added `ProofBlob` and `ProofScanOut` accessors for envelope version and proof
  family.

## Test Coverage

- `test_root_taxonomy_guard` proves that `AssetStateRoot` converts to
  `CheckRoot` through the typed path and `TxDigest::to_check()` rejects with
  `RootErr::TxRootMix`.
- `test_root_taxonomy_guard` also keeps future-only root and asset vocabulary
  out of the live storage and core asset exports for this phase.
- `test_compat_proof_guard` proves the compatibility envelope version, inclusion
  proof family, unsupported deletion/non-existence families, fail-closed
  rejection of unexpected bucket metadata, and unsupported envelope version
  rejection.
- `test_backend_root_mix` proves a raw backend root cannot stand in for the
  checkpoint-facing semantic root context.

## Review Loop

- Manual `/GSD-Review-Tasks-Execution` fallback pass 1 found style drift in
  long test identifiers; the tests were renamed to the shorter names listed
  above.
- Review passes 2 and 3 found no significant remaining issues in the 051-02 code
  or tests, giving two consecutive clean significant-issue passes.
- The follow-up doublecheck verified source coverage against `051-02-PLAN.md`,
  `051-CONTEXT.md`, `051-TEST-SPEC.md`, and `051-TESTS-TASKS.md`.

## Validation

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` passed
  first as the mandatory fail-fast gate.
- `cargo test -p z00z_storage --release --features test-fast --features wallet_debug_dump -- --nocapture`
  passed, including the new 051-02 tests.
- `cargo test --release --features test-fast --features wallet_debug_dump`
  passed for the workspace; cargo reported the release profile and
  `target/release/deps` test binaries.
- `cargo fmt --check` exited 0; rustfmt printed only the repository's existing
  nightly-only configuration warnings.
- `git diff --check` passed after trimming trailing whitespace from planning
  text.
- A repository search for the stale suffixed claim-source proof spelling found
  no matches; the live claim-source proof contract spelling remains
  `ClaimSourceProof`.

## Result

`051-02` is summary-backed complete. Root taxonomy and compatibility proof
envelope v1 behavior are now explicit, test-backed, fail-closed, and ready for
`051-03-PLAN.md` downstream guardrail execution.
