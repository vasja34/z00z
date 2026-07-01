# 041 Full Threats Audit

<!-- markdownlint-disable MD047 -->

Scope: phase 041 spend-proof and adjacent claim, digest, and verifier surfaces. This audit keeps only scenarios that survived a pros and cons filter and could matter at CRITICAL, HIGH, or MEDIUM severity. Trivial malformed-input cases, already-covered reject cases, and replay-only helpers are intentionally excluded.

## Filtered-Out Candidates

### 1. ==CRITICAL - Final tx admission can be bypassed at the local verifier layer==

Attack idea: TxVerifierImpl::verify only checks local structure, balance, digest, signatures, and range proofs. It does not run the public spend contract. If any caller treats this as final admission, a package that is locally valid but missing spend proof or spend auth can enter the system.

Pseudo-code:

1. Build tx bytes that pass local structure, balance, signature, and range-proof checks.
2. Leave spend proof or spend auth missing, stale, or otherwise non-final.
3. Call TxVerifierImpl::verify(bytes).
4. If the caller accepts valid = true without verify_full_tx_package, the spend layer never runs.

Evidence:

- [crates/z00z_wallets/src/core/tx/tx_verifier.rs](../../../crates/z00z_wallets/src/core/tx/tx_verifier.rs#L103)
- [crates/z00z_wallets/src/core/tx/tx_verifier.rs](../../../crates/z00z_wallets/src/core/tx/tx_verifier.rs#L111)
- [crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs](../../../crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs#L164)
- [crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs](../../../crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs#L182)

Current defense:

- The canonical runtime path uses verify_full_tx_package in [crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs](../../../crates/z00z_wallets/src/adapters/rpc/methods/tx_impl_server_lifecycle.rs#L187).
- The simulator stage-4 gate also uses verify_full_tx_package in [crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs](../../../crates/z00z_simulator/src/scenario_1/stage_4_utils/tx_validation_gates.rs#L186).

Verdict: Critical if a production caller wires the wrong verifier. The repo currently appears to use the full verifier in the canonical paths, so this is a high-value seam rather than a confirmed live bypass.

### 2. ==HIGH - Raw claim verification is unsafe without persisted-store rebinding==

Attack idea: ClaimTxVerifierImpl::verify validates internal claim-package consistency, but the authoritative consumer must also rebind the claim source proof to the live AssetStore contract. Without that second step, a helper-owned or synthetic tree can look valid from the verifier's point of view.

Pseudo-code:

1. Build a claim package whose claim source proof and statement agree with each other.
2. Use a synthetic or stale store root that is internally self-consistent.
3. Call ClaimTxVerifierImpl::verify(bytes).
4. If the caller skips AssetStore::claim_source_contract_for_item, the package can look valid even though it was not rebound to the persisted store.

Evidence:

- [crates/z00z_wallets/src/core/tx/claim_tx.rs](../../../crates/z00z_wallets/src/core/tx/claim_tx.rs#L227)
- [crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs](../../../crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs#L142)
- [crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs](../../../crates/z00z_wallets/src/core/tx/claim_tx_verifier_impl_proof.rs#L277)
- [crates/z00z_simulator/src/claim_pkg_consumer.rs](../../../crates/z00z_simulator/src/claim_pkg_consumer.rs#L210)
- [crates/z00z_simulator/src/claim_pkg_consumer.rs](../../../crates/z00z_simulator/src/claim_pkg_consumer.rs#L278)

Current defense:

- The simulator consumer compares the carried proof against the persisted store contract before accepting a claim package.
- This closes the production path, but not the raw verifier API by itself.

Verdict: High-risk API seam. It is safe in the current consumer flow, but unsafe as a standalone admission gate.

### 3. ==HIGH - tx_digest_hex is stable under auth drift, so digest-only trust is unsafe==

Attack idea: build_tx_package_digest normalizes regular txs by zeroing auth and spend blobs before hashing. That means auth drift does not change the digest. Any cache, dedupe path, or persistence layer that uses tx_digest_hex as the full identity of a package can be tricked into associating the tampered package with the original one.

Pseudo-code:

1. Start with a canonical tx package.
2. Change auth.spend_sig_hex or receiver_card_compact.
3. Recompute nothing.
4. Observe that tx_digest_hex stays the same.
5. If an upstream consumer trusts digest equality instead of re-verifying the payload, the tampered package can be misclassified.

Evidence:

- [crates/z00z_wallets/src/core/tx/tx_digest.rs](../../../crates/z00z_wallets/src/core/tx/tx_digest.rs#L22)
- [crates/z00z_wallets/tests/test_spend_statement.rs](../../../crates/z00z_wallets/tests/test_spend_statement.rs#L210)
- [crates/z00z_wallets/tests/test_spend_statement.rs](../../../crates/z00z_wallets/tests/test_spend_statement.rs#L232)
- [crates/z00z_simulator/tests/test_checkpoint_acceptance.rs](../../../crates/z00z_simulator/tests/test_checkpoint_acceptance.rs#L524)

Current defense:

- Stage-11 and full-package admission re-read the payload and reject digest tamper.
- That protection only works for consumers that actually re-run the verifier and do not trust the digest alone.

Verdict: High severity as a second-order integrity risk. The digest is an identity root for routing and storage, not a substitute for verification.

### 4. ==MEDIUM - The public spend contract is not a full tx verifier==

Attack idea: verify_tx_public_spend_contract checks the statement-bound envelope only. It cannot re-establish the witness-only theorem and it does not replace local structure, balance, signature, or range-proof validation. If a caller uses this helper alone as the final gate, malformed or unbalanced packages can slip through.

Pseudo-code:

1. Build a tx package whose public spend envelope is self-consistent.
2. Leave the rest of the tx outside the full verifier's expectations.
3. Call verify_tx_public_spend_contract(pkg).
4. If the caller treats that result as complete admission, the rest of the pipeline is skipped.

Evidence:

- [crates/z00z_wallets/src/core/tx/tx_verifier.rs](../../../crates/z00z_wallets/src/core/tx/tx_verifier.rs#L103)
- [crates/z00z_wallets/src/core/tx/tx_verifier.rs](../../../crates/z00z_wallets/src/core/tx/tx_verifier.rs#L111)
- [crates/z00z_wallets/src/core/tx/tx_verifier.rs](../../../crates/z00z_wallets/src/core/tx/tx_verifier.rs#L121)
- [crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs](../../../crates/z00z_wallets/src/core/tx/test_tx_verifier_suite.rs#L186)

Current defense:

- Canonical admission composes local verification plus the public spend contract in verify_full_tx_package.
- The simulator and RPC paths already use the combined entry point.

Verdict: Medium because this is mainly a caller-misuse hazard, but the API split is real and the boundary is intentionally narrower than a theorem proof.



