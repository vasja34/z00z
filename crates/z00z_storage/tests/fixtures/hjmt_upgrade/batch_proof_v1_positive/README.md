# Batch Proof V1 Positive Corpus

This directory is the Phase 055 positive evidence home for `BPB-G-001`
through `BPB-G-005`.

- The canonical accepted case registry lives in `manifest.json`.
- Every case is generated from the live storage-owned batch builder and the
  live canonical encoder.
- Every case records one accepted proof family, one path shape, one accepted
  semantic root, exact canonical bytes, witness and reference counts, one
  regeneration command, and one evidence pointer.
- The checked-in bytes are cross-checked against the live builder in
  `test_hjmt_batch_proof.rs` and then revalidated by the production batch
  decoder and verifier before the fixture is accepted.
