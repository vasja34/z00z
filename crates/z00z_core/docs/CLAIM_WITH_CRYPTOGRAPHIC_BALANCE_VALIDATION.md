# Claim With Cryptographic Balance Validation

This note records the live intent of the genesis claim-flow coverage.

## Covered Behaviour

- prove that the claimed genesis input commitment opens to the expected amount
  and blinding
- preserve plaintext balance across claim outputs
- preserve Pedersen balance across input and output commitments
- verify output commitments and homomorphic composition before the flow is
  accepted

## Release Verification

```bash
cargo test --release -p z00z_core --test genesis_tests claim_flow -- --nocapture
cargo test --release -p z00z_core --test genesis_tests test_claim_with_cryptographic_balance_validation -- --nocapture
```

## Authority

The executable authority is the live `genesis_tests` target together with the
current `z00z_core::genesis` and wallet claim code. This file is descriptive
only and must not diverge from release-mode test behaviour.
