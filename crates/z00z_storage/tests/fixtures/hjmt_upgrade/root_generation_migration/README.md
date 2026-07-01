# Batch Proof V1 Root-Generation Migration Corpus

This directory is the Phase 055 root-generation migration evidence home for
the accepted live generation and the explicit unsupported future-generation
reject vector.

- The canonical migration registry lives in `manifest.json`.
- `RGM-G-001` records the accepted live `RootGeneration1` contract.
- `RGM-T-001` records the explicit `BatchRootGenerationMix` rejection for the
  unsupported `RootGeneration0` vector.
- The checked-in bytes are regenerated from the live builder in
  `test_hjmt_batch_proof.rs` and revalidated by the production batch decoder
  before the fixture is accepted.
