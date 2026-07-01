# Batch Proof V1 Negative Corpus

This directory is the Phase 055 negative evidence home for `BPB-T-001`
through `BPB-T-008`.

- The canonical case registry lives in `manifest.json`.
- The canonical accepted source bytes used by the tamper cases come directly
  from `../batch_proof_v1_positive/manifest.json`.
- Every case records one mutation point, one expected reject stage, one expected
  verdict, one expected error class, one regeneration command, and one evidence
  pointer.
- There is no second accepted-source registry in this directory; negative
  evidence mutates the Phase 055 positive `BPB-G-*` fixtures in place as the
  single accepted-source authority.
- The checked-in source bytes are cross-checked against the positive fixture
  manifest in `test_hjmt_batch_proof_negative.rs`, then revalidated by the
  production batch decoder and verifier before any tamper mutation is applied.
