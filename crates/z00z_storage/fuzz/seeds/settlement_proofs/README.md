# Settlement fuzz seeds

This seed corpus is for `settlement_proofs`.

Scope:
- malformed settlement paths
- malformed settlement proof envelopes
- right-leaf family drift
- voucher-leaf family drift
- fee-envelope drift
- occupancy-evidence bytes
- epoch drift
- root drift
- policy-transition replay/tamper lanes

Entry points are storage-owned only:
- `SettlementLeaf::decode`
- `BincodeCodec.deserialize::<SettlementPath>`
- `BincodeCodec.deserialize::<FeeEnvelope>`
- `BincodeCodec.deserialize::<ProofBlob>`
- `SettlementStore::validate_settlement_proof_blob`
- `SettlementStore::validate_settlement_nonexistence_proof_blob`
- `SettlementStore::validate_split_proof`
- `SettlementStore::validate_merge_proof`
- `SettlementStore::validate_policy_transition_proof`

Dispatch contract:
- first byte `% 8` selects the branch
- remaining bytes are fed to the matching decoder/verifier lane

Seed file coverage:
- `00_settlement_leaf.seed` -> terminal leaf codec
- `01_settlement_path.seed` -> settlement path decode
- `02_fee_envelope.seed` -> fee envelope decode
- `03_proof_envelope.seed` -> proof envelope decode + generic validation
- `04_occupancy.seed` -> occupancy evidence decode
- `05_policy_transition.seed` -> policy transition verifier
- `06_split.seed` -> split verifier
- `07_merge.seed` -> merge verifier
