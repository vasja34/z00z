## 31. Linked Liability MVP: Domain, Commitment, Fraud Proof, Lock Registry

**Goal:**

- Implement a local linked-liability MVP with domain objects, hidden commitments, fraud-proof DTOs, penalty policy, durable locks, quarantine, and unlock conditions.
- Prove that honest use stays private, conflicting local evidence can open a bounded lock case, and locked domains cannot keep spending through wallet or simulator paths.

**Source:**

- [Linked Liability whitepaper, section 3.1: Canonical Liability Objects](Z00Z-Linked-Liability-Whitepaper.md#31-canonical-liability-objects)
- [Linked Liability whitepaper, section 3.2: Attachment To Rights And Packages](Z00Z-Linked-Liability-Whitepaper.md#32-attachment-to-rights-and-packages)
- [Linked Liability whitepaper, section 4.2: Honest Spend And Local Acceptance](Z00Z-Linked-Liability-Whitepaper.md#42-honest-spend-and-local-acceptance)
- [Linked Liability whitepaper, section 5.2: Fraud Proof Extraction](Z00Z-Linked-Liability-Whitepaper.md#52-fraud-proof-extraction)
- [Linked Liability whitepaper, section 5.3: Lock, Slash, Quarantine, And Case Opening](Z00Z-Linked-Liability-Whitepaper.md#53-lock-slash-quarantine-and-case-opening)
- [Linked Liability whitepaper, section 6.1: Bonded Risk Model](Z00Z-Linked-Liability-Whitepaper.md#61-bonded-risk-model)
- [Linked Liability whitepaper, section 6.2: Penalty Policy Design](Z00Z-Linked-Liability-Whitepaper.md#62-penalty-policy-design)
- [Linked Liability whitepaper, section 9.1: Protocol Versus Service Responsibilities](Z00Z-Linked-Liability-Whitepaper.md#91-protocol-versus-service-responsibilities)
- [Linked Liability whitepaper, section 10.2: What Remains Target Architecture](Z00Z-Linked-Liability-Whitepaper.md#102-what-remains-target-architecture)
- [Linked Liability whitepaper, appendix B: Example Flows](Z00Z-Linked-Liability-Whitepaper.md#appendix-b-example-flows)
- [Linked Liability whitepaper, appendix C: Security Property Checklist](Z00Z-Linked-Liability-Whitepaper.md#appendix-c-security-property-checklist)
- [Linked Liability whitepaper, appendix D: Policy Templates](Z00Z-Linked-Liability-Whitepaper.md#appendix-d-policy-templates)

**Implementation-relevant fragments:**

- Use sections 3.1 and 3.2 for the liability object model and attachment points to rights/packages.
- Use sections 4.2, 5.2, and 5.3 for honest spend, local acceptance, conflict extraction, fraud-proof DTOs, and lock/quarantine transitions.
- Use sections 6.1, 6.2, and appendix D for local policy fields such as exposure, cooldown, penalty, and unlock conditions.
- Use section 9.1 to keep protocol and service responsibilities separate, section 10.2 as a non-goal boundary, and appendices B/C for simulator flows and negative tests.

**Locality gate:**

- The MVP can start with local offline payment conflicts, local receipts, local nullifier/replay surfaces, local fraud-proof DTOs, and a local lock registry.
- No production slashing, DAO dispute lane, external legal process, or live network is needed.

**Implementation boundary:**

- In scope: object model, commitment binding, conflict extraction, local lock/quarantine state, policy validation, and simulator fraud scenarios.
- Out of scope: real bond custody, production slashing automation, public reputation systems, manual arbitration portals, and legal case management.

**Implementation tasks:**

1. **In `z00z_core`, add canonical local types:**
   - `LiabilityDomain`
   - `HiddenLiabilityCommitment`
   - `FraudProof`
   - `BondRef`
   - `PenaltyPolicy`
   - `LiabilityAttachment`
2. In `z00z_crypto`, add domain-separated hash/KDF helpers for hidden liability commitments and fraud-proof statement hashes. If full cryptographic reveal logic is not ready, use explicit placeholder proof variants named as local/test or draft.
3. In `z00z_storage`, add a `LockRegistry` model with domain ID, lock status, reason, evidence digest, affected right IDs, created sequence, cooldown/unlock condition, and quarantine scope.
4. In `z00z_wallets`, bind liability attachments to offline payment packages and future right packages without revealing the liability domain during honest operation.
5. Add policy validation for offline limit, max outstanding exposure, penalty tier, cooldown, dispute window, and unlock condition.
6. Add fraud-proof extraction from conflicting local receipts or conflicting checkpoint replay evidence.
7. Add lock enforcement: package creation and local acceptance must fail closed for locked domains where policy says locked domains cannot spend.
8. Add future-right freeze logic scoped to the liability domain, not to unrelated wallet assets.

**Tests and simulation:**

- Honest path: liability-bound offline payment spends and reconciles without revealing liability domain.
- Conflict path: two receipts or packages conflict on spend/replay surfaces and produce one local `FraudProof`.
- Exculpability: fabricated proof, incomplete proof, wrong domain, wrong commitment, and mismatched receipt pair all reject.
- Lock registry: accepted fraud proof creates a lock/quarantine entry; duplicate lock is idempotent.
- Spend while locked: wallet refuses to create or accept a domain-scoped right when the lock registry says locked.
- Scope control: unrelated liability domains and unrelated assets remain spendable.
- Bond policy: aggregate outstanding exposure cannot exceed configured bond policy.
- Unlock: local unlock proof/cooldown fixture clears the lock only when policy allows it.
- Simulator flow: honest offline spend, conflicting spend, fraud proof, lock, attempted future spend rejection, unlock fixture.

**Done when:**

- Offline payment domains have a local linked-liability object model and end-to-end simulator evidence.
- Fraud proofs are explicit about what they reveal and what they do not reveal.
- Lock state is durable in local storage and enforced by wallet policy tests.

**Doublecheck:**

- Local condition: satisfied. The MVP uses local payments, local evidence, and local lock registry only.
- Developer clarity: satisfied. Object list, policy fields, fraud cases, and lock semantics are concrete.

## 
