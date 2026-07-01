## 053-16 Summary

Implemented the mixed settlement regression corpus for Phase 053 live HJMT scope.

### Delivered

- Added a deterministic mixed asset/right fixture in
  `crates/z00z_storage/tests/fixtures/test_settlement_corpus_fixture.json`.
- Added shared test support and an independent settlement-root oracle in
  `crates/z00z_storage/tests/test_settlement_corpus_support.inc`.
- Added golden corpus coverage in `crates/z00z_storage/tests/test_golden_corpus.rs` for:
  - mixed live asset/right operations;
  - inclusion, deletion, and non-existence proofs;
  - legacy envelope-less proof rejection without state drift;
  - adaptive split, merge, policy-transition, and reload validation.
- Added property coverage in `crates/z00z_storage/tests/test_property_corpus.rs` for:
  - generated mixed sequences against the independent oracle;
  - operation reordering invariants;
  - reject-path root/row preservation;
  - fee replay and missing-fee rejection;
  - malformed proof bytes rejecting without panic;
  - split/merge/policy-transition terminal-set preservation;
  - reload idempotence.
- Added fuzz harness artifacts:
  - `crates/z00z_storage/fuzz/Cargo.toml`
  - `crates/z00z_storage/fuzz/fuzz_targets/settlement_proofs.rs`
  - `crates/z00z_storage/fuzz/seeds/settlement_proofs/README.md`
  - `crates/z00z_storage/fuzz/seeds/settlement_proofs/*.seed`
- Added cross-crate corpus checks:
  - `crates/z00z_core/tests/genesis/test_settlement_corpus.rs`
  - `crates/z00z_simulator/tests/test_scenario_settlement.rs`

### Fixes During Review

- Fixed the reject-path property setup so the "present asset" lane actually seeds the mixed fixture before asserting non-existence rejection.
- Fixed property fee helpers to avoid zeroed or colliding fee-envelope fields under wrapped marks, preventing false `FailurePolicyMix` and `ReplayMix` failures in generated tests.
- Relaxed the policy-transition property assertion to the live contract: epoch monotonicity and next-policy binding are required, but semantic root delta is not.

### Verification

- `./.github/skills/smart-tests-bootstrap/scripts/bootstrap_tests.sh` — passed on final state.
- `cargo test -p z00z_storage --release --features test-fast --test test_golden_corpus --test test_property_corpus --test test_fuzz_seeds -- --nocapture` — passed.
- `cargo test -p z00z_core --release --features test-fast --test genesis_tests test_settlement_corpus -- --nocapture` — passed.
- `cargo test -p z00z_simulator --release --features test-fast --features wallet_debug_dump --test test_scenario_settlement -- --nocapture` — passed.
- `cargo check --manifest-path crates/z00z_storage/fuzz/Cargo.toml --release` — passed.
- `cargo test --release --features test-fast --features wallet_debug_dump` — passed on the final tree.

### Review Loop

- Review pass 1: storage corpus/property/fuzz surface reviewed after targeted fixes; no remaining in-scope warnings or compile issues.
- Review pass 2: legacy rejection, fuzz dispatch lanes, and core/simulator corpus handoff reviewed; no significant in-scope issues found.
- Review pass 3: broad verification and checklist reconciliation reviewed; no significant in-scope issues found.

### Next

- `053-17-PLAN.md` was read as the next owner plan.
