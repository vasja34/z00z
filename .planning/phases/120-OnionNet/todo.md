## 35. OnionNet Deterministic Control Plane And Packet Discipline

**Goal:**

- Build the deterministic local OnionNet control-plane contracts for registry views, epochs, active-set selection, route planning, packet geometry, AAD, replay durability, and queue policy.
- Prove route determinism, fail-closed selection, packet confidentiality boundaries, replay persistence, and backpressure locally while making no live transport-anonymity claim.

**Source:**

- [OnionNet whitepaper, section 4.1: Public Membership Registry](Z00Z-OnionNet-Whitepaper.md#41-public-membership-registry)
- [OnionNet whitepaper, section 4.2: Deterministic Epoch View](Z00Z-OnionNet-Whitepaper.md#42-deterministic-epoch-view)
- [OnionNet whitepaper, section 4.3: Deterministic Selection Specifics](Z00Z-OnionNet-Whitepaper.md#43-deterministic-selection-specifics)
- [OnionNet whitepaper, section 4.4: Probation, Challenge, And Reserve Discipline](Z00Z-OnionNet-Whitepaper.md#44-probation-challenge-and-reserve-discipline)
- [OnionNet whitepaper, section 5.1: Lane Contract](Z00Z-OnionNet-Whitepaper.md#51-lane-contract)
- [OnionNet whitepaper, section 5.2: Route Workflow](Z00Z-OnionNet-Whitepaper.md#52-route-workflow)
- [OnionNet whitepaper, section 5.5: Witness Retrieval And Route-Intent Privacy](Z00Z-OnionNet-Whitepaper.md#55-witness-retrieval-and-route-intent-privacy)
- [OnionNet whitepaper, section 6.3: Double-Envelope Confidentiality](Z00Z-OnionNet-Whitepaper.md#63-double-envelope-confidentiality)
- [OnionNet whitepaper, section 6.4: AAD Binding, Replay Binding, And Key Separation](Z00Z-OnionNet-Whitepaper.md#64-aad-binding-replay-binding-and-key-separation)
- [OnionNet whitepaper, section 7.1: Replay Durability](Z00Z-OnionNet-Whitepaper.md#71-replay-durability)
- [OnionNet whitepaper, section 7.2: Explicit Backpressure](Z00Z-OnionNet-Whitepaper.md#72-explicit-backpressure)
- [OnionNet whitepaper, section 7.3: Low-Load Contraction And Privacy Floors](Z00Z-OnionNet-Whitepaper.md#73-low-load-contraction-and-privacy-floors)
- [OnionNet whitepaper, section 8.3: Recommended Crypto Direction](Z00Z-OnionNet-Whitepaper.md#83-recommended-crypto-direction)
- [OnionNet whitepaper, section 10.1: Safe Rollout Order](Z00Z-OnionNet-Whitepaper.md#101-safe-rollout-order)
- [OnionNet whitepaper, section 11: Open Contracts And Research Blockers](Z00Z-OnionNet-Whitepaper.md#11-open-contracts-and-research-blockers)
- [OnionNet whitepaper, appendix B: Normative Requirement Summary](Z00Z-OnionNet-Whitepaper.md#appendix-b-normative-requirement-summary)

**Implementation-relevant fragments:**

- Use sections 4.1 through 4.4 for registry state, deterministic epoch views, active-set selection, probation, challenge, reserve, and lifecycle rules.
- Use sections 5.1, 5.2, and 5.5 for lane contracts, route planning, witness-bundle abstraction, and route-intent privacy fixtures.
- Use sections 6.3, 6.4, 7.1, 7.2, and 7.3 for packet envelope, AAD, replay durability, backpressure, and low-load contraction tests.
- Use section 8.3 only as crypto direction behind repo-owned AEAD/domain seams, section 10.1 for local rollout ordering, section 11 as a blocker list, and appendix B as the normative checklist.

**Locality gate:**

- Deterministic membership views, active-set sampling, lane contracts, route construction, packet AAD, replay ledgers, and backpressure logic can be implemented as pure local functions and local storage tests.
- Live OnionNet transport, real node operation, QUIC, messenger relay, and production challenge/slashing stay out of scope.

**Implementation boundary:**

- In scope: local control-plane models, deterministic sampling, route planning, witness bundle abstraction, packet class geometry, double-envelope local crypto, durable replay ledger, queue policy, and simulator adversarial tests.
- Out of scope: live transport, real Sphinx integration unless behind a repo-owned seam, relay incentives, public registry deployment, randomness beacon service, and production slashing.

**Implementation tasks:**

1. **In `z00z_core`, model:**
   - `OnionNodeDescriptor`
   - `OnionNodeState`
   - `RegistryRoot`
   - `PolicyRoot`
   - `EpochSeed`
   - `EpochView`
   - `LaneContract`
   - `RoutePlan`
   - `WitnessBundleRef`
2. Add lifecycle states: `Registered`, `Probation`, `Eligible`, `Active`, `Demoted`, `Revoked`, and `Slashed`.
3. Add deterministic epoch derivation from epoch seed, registry root, policy root, compatibility generation, active set, reserve ordering, lane count, bucket sequence, expiry, and thresholds.
4. Implement weighted deterministic sampling with concentration ceilings, diversity floors, capacity weights, reserve replacement, and fail-closed behavior when constraints cannot be met.
5. In `z00z_wallets`, add a route builder that recomputes epoch view locally and refuses bridge-selected fallback routes.
6. In `z00z_crypto`, add or formalize OnionNet domain constants including `OnionSessionDomain` and `HKDF_INFO_ONIONNET_SESSION`; implement local double-envelope seal/open helpers using approved AEAD APIs.
7. Define packet classes `data`, `cover`, `loop`, and `control` with fixed geometry and size-class validation.
8. Define AAD fields: version, epoch ID, compatibility generation, traffic class, ingress-recipient key ID, ciphertext size class, expiry, and inner replay tag.
9. In `z00z_storage`, add a durable `OnionReplayLedger` that records exact replay tags before side effects; hot caches may exist only as acceleration.
10. In `z00z_simulator`, add local route-construction and adversarial packet scenarios.
11. Add explicit queue/backpressure policy for ingress, decryptor, work-item, and outbound queues.
12. Add low-load contraction behavior with privacy floors and fail-closed thresholds.

**Tests and simulation:**

- Determinism: same registry root, policy root, epoch seed, and compatibility generation produce the same epoch view on repeated runs.
- Constraint failure: insufficient diversity, missing floor, concentration violation, expired epoch, wrong compatibility generation, or revoked node fails closed.
- Route workflow: wallet recomputes epoch view, selects participants, validates witness bundle, and rejects bridge/operator-selected fallback.
- Witness privacy: route intent can use coarse bundle/shard references without revealing a single intended route in the fixture.
- Double envelope: exit unwraps only transport wrapper; ingress recipient decrypts inner envelope; wrong AAD, wrong epoch, wrong traffic class, wrong key ID, wrong size class, or expired packet rejects.
- Replay durability: exact replay tag is persisted before side effects; restart and replay still reject.
- Packet geometry: data/cover/loop/control classes maintain fixed size classes and do not leak dynamic payload length.
- Backpressure: queue overflow rejects or delays explicitly; no silent unbounded queue growth.
- Low load: active-set contraction respects privacy floor or fails closed.

**Done when:**

- OnionNet local control-plane contracts have deterministic tests and adversarial simulator coverage.
- No code path opens a socket, joins a real network, or claims live transport security.
- Crypto stays behind repo-owned domain and AEAD seams.

**Doublecheck:**

- Local condition: satisfied. The package deliberately implements local deterministic control-plane and packet discipline only.
- Developer clarity: satisfied. Models, fields, states, route rules, crypto AAD, and tests are specified.

---

---



## 
