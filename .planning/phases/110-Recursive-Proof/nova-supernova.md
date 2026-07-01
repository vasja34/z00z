## 23. Recursive-Proof Spike Boundary

**Goal:**

- Bound recursive-proof work to one exact local checkpoint or state-transition statement before any backend integration.
- Prove the statement, public inputs, verifier contract, failure model, and benchmark metadata are clear while current spend verification remains unchanged.

**Source:**

- [Recursive proof README, exact statement and spike-first rollout](../.planning/phases/075-recursive-proofs/README-recursive_proofs.md)

**Implementation-relevant fragments:**

- Use README numbered requirements 1, 2, 3, 5, 7, 8, and 10 for the one code-facing statement, granularity choice, spike-first rollout, public inputs, unchanged spend layer, verifier/proof object, benchmark harness, and rollout order.
- Use the final practical-advice list for the first local target: one checkpoint transition, public inputs, no spend-verifier replacement, 3-5 step chain, and proof-size/prover/verifier timing.

**Locality gate:**

- The spike boundary is local architecture and proof-adapter work: define a statement, public inputs, witness shape, verifier entrypoint, benchmark harness, and failure model.
- No production recursive backend, audited prover, live network, or testnet is required.

**Implementation boundary:**

- In scope: choosing one checkpoint/state-transition statement, defining public inputs, choosing block or epoch granularity for local tests, proof serialization, verifier API, benchmark metadata, and failure modes.
- Out of scope: integrating Plonky3 or any other backend as live protocol truth, replacing current transaction privacy/range-proof layer, or migrating the network.

**Implementation tasks:**

1. Write the exact statement in code-facing terms: previous root, delta or block digest, new root, epoch/block ID, domain separator, and optional nullifier-set root.
2. Define a public input struct and serialization contract in a proof-facing module.
3. Define a transition witness fixture for local tests and simulator spikes.
4. Define block, epoch, or aggregation granularity for the first local spike and record why the other choices are not selected yet.
5. Add proof serialization and verifier entrypoint contracts that can be backed by a mock adapter first.
6. Add benchmark harness metadata fields before collecting proof-size or timing data.
7. Add failure model entries for wrong root, wrong delta, wrong proof object, wrong chain link, and unsupported backend.
8. Keep current spend verification explicitly unchanged.

**Tests and simulation:**

- Statement construction tests for required fields and deterministic digest.
- Public input serialization tests for version, length, and domain separation.
- Mock verifier tests for valid statement, wrong root, wrong delta, wrong proof object, unsupported backend, and chain-link mismatch.
- Benchmark metadata validation before any measurement is accepted.
- Simulator 3-step spike using the mock adapter and local transition witness.

**Done when:**

- The recursive proof spike has one exact local target and a verifier contract.
- Measurement and failure-model scaffolding exists before backend integration.
- The current spend verifier remains unchanged and explicitly outside the spike.

**Doublecheck:**

- Local condition: satisfied. This is a local proof-boundary spike with mockable adapters.
- Developer clarity: satisfied. The statement, inputs, verifier contract, and non-goals are explicit.

## 24. Nova Or SuperNova State-Transition Note

**Goal:**

- Translate the Nova/SuperNova note into backend-neutral local state-transition semantics around `root_old`, update witnesses, `root_new`, and prior-proof output binding.
- Prove the model can be tested as local transition and chain-binding logic without selecting Nova/SuperNova as production dependency.

**Source:**

- [Nova or SuperNova note, state-transition proof model](../.planning/phases/075-recursive-proofs/z00z-recursive-proofs.md)

**Implementation-relevant fragments:**

- Use the headings "What you have" and "What the Nova/SuperNova scheme does" for `root_old`, update witness, `root_new`, and prior-proof output binding.
- Use "Proof size" and backend-selection discussion only as measurement metadata and research notes; do not select Nova/SuperNova as production dependency here.

**Locality gate:**

- The actionable part is a local IVC-style transition model over state roots and update witnesses.
- No Nova/SuperNova production integration, trusted network, external consensus, or live DA is needed.

**Implementation boundary:**

- In scope: modeling `root_old`, update witness, `root_new`, prior-proof output binding, local chain verification, proof-size expectation capture, and backend-selection notes.
- Out of scope: treating Nova/SuperNova as selected production dependency, claiming audited security, changing consensus, or discarding raw transaction availability requirements.

**Implementation tasks:**

1. Translate the note into a local state-transition interface where `apply_updates(root_old, updates) == root_new`.
2. Add prior-proof output binding so `prev_proof.output_root == root_old` is part of the local verification model.
3. Define update witness cases for insert, spend/nullifier mark, create output, delete if supported, and no-op or empty delta.
4. Add local tree fixtures with Merkle or JMT-style paths that are sufficient for proof-shape testing.
5. Add a backend-neutral adapter label for Nova/SuperNova research notes without importing it as protocol dependency.
6. Add measurement placeholders for expected proof size, prover time, verifier time, and CPU-only feasibility.
7. Document data availability responsibility as local fixture retention, not network-level guarantee.

**Tests and simulation:**

- Transition tests for valid update set, wrong witness path, wrong old root, wrong new root, duplicate update, and empty delta.
- Chain-binding tests for valid prior output, mismatched prior output, skipped prior proof, and reordered proof.
- Backend-neutral serialization tests proving Nova/SuperNova labels remain research metadata unless a real adapter is enabled.
- Simulator state-chain test over 3-5 transitions using local fixtures.
- Data-retention test proving raw update fixtures are available for local failure reproduction.

**Done when:**

- The Nova/SuperNova note is captured as backend-neutral local transition semantics.
- Local tests verify root transition and prior-proof binding.
- No production dependency or consensus claim is introduced.

**Doublecheck:**

- Local condition: satisfied. The work is local transition modeling and simulator chaining.
- Developer clarity: satisfied. Translation from note to code-facing tasks and tests is explicit.

## 25. Recursive State Proof Research Register

**Goal:**

- Mine the broad recursive state proof register for local failure cases, proof-chain assumptions, data-retention needs, and measurement questions.
- Prove every extracted item is labeled as local implementation, local test, measurement, future architecture, research-only, or out of scope.

**Source:**

- [Recursive state proof research register](../.planning/phases/075-recursive-proofs/11_Z00Z_Recursive_StateProof.md)

**Implementation-relevant fragments:**

- Use the register sections on incomplete chain verification, historical nullifier checks, state reconstruction attacks, what must be stored, and fraud-proof cost for local failure cases, proof-chain assumptions, data-retention needs, and measurement questions.
- Treat post-quantum, weak-subjectivity, fraud-economics, broad recursive-epoch policy, and link-tag sketches as research-only unless a later accepted design promotes them.

**Locality gate:**

- The register is broad research material. Only local proof statements, failure cases, measurement questions, and simulator fixtures should be extracted.
- No speculative proof system, post-quantum design, fraud-economics model, weak-subjectivity policy, or live checkpoint protocol change is required.

**Implementation boundary:**

- In scope: extracting local failure cases around recursive epochs, compressed state, nullifier history, fraud handling, chain verification, proof-size assumptions, and data-retention requirements.
- Out of scope: implementing all research sketches, declaring post-quantum readiness, adopting weak subjectivity policy, changing fraud economics, or treating link-tag sketches as current protocol truth.

**Implementation tasks:**

1. Create a research extraction checklist that labels each item as local implementation, local test, measurement question, future architecture, or out of scope.
2. Extract failure cases for incomplete chain verification, wrong genesis or initial state, repeated secret use, aggregate proof mismatch, stale proof, and fraud-proof inconsistency.
3. Extract local nullifier-history cases that can be tested without live consensus.
4. Extract data-retention assumptions into simulator fixture requirements.
5. Extract proof-size and historical fraud-proof cost questions into phase `27` measurement metadata.
6. Mark post-quantum, lattice, weak-subjectivity, fraud-economics, and broad recursive-epoch policy sections as research-only unless a later design accepts them.
7. Add doc guards so research claims cannot drift into current protocol sections.
8. Link any actionable extracted item to phases `20`, `26`, or `27`.

**Tests and simulation:**

- Checklist test or documentation review proving every extracted research item has a status label.
- Local failure-case tests for wrong genesis/root, broken proof chain, stale proof, repeated nullifier, and inconsistent fraud evidence where current fixtures support it.
- Measurement mapping test proving proof-size questions feed phase `27` only.
- Drift check proving research-only post-quantum, weak-subjectivity, and fraud-economics text is not described as implemented protocol.
- Simulator fixture retention test proving historical failure reproduction has the necessary local raw artifacts.

**Done when:**

- The research register has been mined for local tasks without importing speculative claims into current architecture.
- Actionable recursive proof work maps to phases `20`, `26`, and `27`.
- Research-only material is explicitly labeled and cannot be mistaken for implemented protocol.

**Doublecheck:**

- Local condition: satisfied. Only local tests, measurement questions, and simulator fixtures are extracted.
- Developer clarity: satisfied. Extraction statuses, failure cases, and non-goals are explicit.

## 26. Recursive-Proof Statement Spike And Proof-Size Guardrails

**Goal:**

- Define one exact local recursive-proof target around checkpoint or state-transition correctness, with typed public inputs, witnesses, proof-object codecs, verifier adapters, and measurement metadata.
- Prove a short deterministic proof chain locally while keeping checkpoint admission, spend verification, and public proof envelopes unchanged.

**Source:**

- [Recursive proof README, statement and spike-first requirements](../.planning/phases/075-recursive-proofs/README-recursive_proofs.md)
- [Nova or SuperNova note, state-transition proof model](../.planning/phases/075-recursive-proofs/z00z-recursive-proofs.md)
- [Recursive state proof research register](../.planning/phases/075-recursive-proofs/11_Z00Z_Recursive_StateProof.md)
- [Main whitepaper, appendix C: Benchmarks And Evaluation Boundary](Z00Z-Main-Whitepaper.md#appendix-c-benchmarks-and-evaluation-boundary)

**Implementation-relevant fragments:**

- Use the recursive proof README numbered requirements 1, 3, 5, 7, 8, and 10 for the statement, spike boundary, public inputs, unchanged spend layer, proving infrastructure, and rollout order.
- Use the Nova/SuperNova note headings "What you have", "What the Nova/SuperNova scheme does", and "Important nuance" for `root_old`, update witnesses, `root_new`, prior-proof binding, and nullifier-set retention.
- Use the research register only to mine local failure cases and data-retention questions from sections on incomplete chain verification, historical nullifier checks, what must be stored, and fraud-proof cost notes; do not import speculative proof systems as implementation truth.
- Use main appendix C to label proof-size and timing results as local evidence, not production security or throughput claims.

**Locality gate:**

- This is a local proof-facing spike around checkpoint/state-transition statements, proof-object codecs, verifier adapters, deterministic witnesses, and simulator measurement.
- No production recursive prover, live consensus admission, external DA, audited recursive backend, or testnet rollout is required.

**Implementation boundary:**

- In scope: one exact statement such as `root_old + delta_commitment -> root_new`, public input typing, witness fixtures, local proof adapter trait, mock or experimental proof object, serialization, verifier entrypoint, 3-5 step local chaining, proof-size/prover-time/verifier-time measurement, and guard text that this is spike evidence only.
- Out of scope: replacing the current spend verifier, replacing Bulletproofs-plus range proofs, claiming production recursive security, changing canonical checkpoint admission, introducing a live proof backend as protocol truth, or changing public proof envelopes without a separate accepted design.

**Implementation tasks:**

1. In `z00z_core`, define a bounded `RecursiveStateStatement` or equivalent experimental type with previous root, new root, epoch or block ID, chain/domain separator, delta commitment, optional nullifier-set root, and statement version.
2. In `z00z_storage`, bind the statement to existing checkpoint artifacts without making the recursive proof authoritative for checkpoint admission.
3. In `z00z_crypto`, add only domain constants, digest helpers, and adapter-facing wrappers needed to hash public inputs; do not introduce unaudited cryptography as a live verifier.
4. Add a proof adapter trait with `prove_step`, `verify_step`, and `verify_chain` semantics, and provide a deterministic local/mock implementation first.
5. Define a transition witness fixture that applies insert, update, delete, and nullifier-mark operations to a local tree model and produces the claimed new root.
6. In `z00z_simulator`, build a 3-5 step chain where each step verifies that the prior output root equals the next input root.
7. Add proof-object codecs with strict version, length, statement digest, backend label, and public-input digest checks.
8. Record proof size, proving time, verifying time, witness size, and chain length as local measurement metadata.
9. Keep storage proof and scan measurements as an optional sidecar that cannot change storage authority or wallet scan ownership.
10. Add documentation guards in generated measurement artifacts that this is local spike evidence and not a production security claim.

**Tests and simulation:**

- Statement tests for valid transition, wrong previous root, wrong new root, wrong epoch or block ID, wrong chain/domain separator, wrong delta commitment, and wrong nullifier root.
- Codec tests for unsupported version, malformed proof bytes, wrong backend label, wrong public-input digest, missing proof bytes, oversized proof bytes, and statement/proof mismatch.
- Chain tests for valid 3-step and 5-step chains, broken prior-output link, skipped step, repeated step, and reordered step.
- Storage binding tests proving recursive proof artifacts can be attached as local evidence while checkpoint admission still uses the existing checkpoint verifier.
- Simulator spike run that builds a local state chain, emits measurement metadata, and rejects a tampered intermediate proof object.
- Storage-measurement sidecar tests proving measurement code does not alter canonical encoding, wallet scan ownership, or checkpoint root authority.

**Done when:**

- One exact recursive state statement is typed, serialized, verified locally, and demonstrated over a short deterministic chain.
- Proof-size and timing metadata is emitted with explicit local-spike scope.
- Checkpoint admission remains owned by the existing storage/checkpoint verifier.
- No test or simulator run needs a live prover service, testnet, external DA, or production recursive backend.

**Doublecheck:**

- Local condition: satisfied. The spike uses local statements, local fixtures, local proof adapters, and simulator measurement only.
- Developer clarity: satisfied. Statement fields, crate ownership, forbidden protocol changes, and rejection cases are explicit.

## 
