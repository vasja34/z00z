## 7. Cross-Crate Test Matrix

**Goal:**

- Turn the local implementation plan into a crate-owned verification map across `z00z_core`, `z00z_crypto`, `z00z_storage`, `z00z_wallets`, and `z00z_simulator`.
- Prove that each behavior has lower-level rule tests before simulator evidence is accepted, and that simulator scenarios demonstrate integration instead of defining business rules.

**Source:**

- [Main whitepaper, section 12.1: What Is Already Live](Z00Z-Main-Whitepaper.md#121-what-is-already-live)
- [Main whitepaper, section 12.2: What Is Still Target Architecture](Z00Z-Main-Whitepaper.md#122-what-is-still-target-architecture)
- [Main whitepaper, section 12.3: Proposed Expansion Path](Z00Z-Main-Whitepaper.md#123-proposed-expansion-path)
- [Linked Liability whitepaper, section 10.3: Proposed Expansion Path](Z00Z-Linked-Liability-Whitepaper.md#103-proposed-expansion-path)
- [OnionNet whitepaper, section 10.1: Safe Rollout Order](Z00Z-OnionNet-Whitepaper.md#101-safe-rollout-order)
- [Agentic whitepaper, section 8.2: Dependency Stack](Z00Z-Agentic-Offline-Machine-Economy-Whitepaper.md#82-dependency-stack)

**Implementation-relevant fragments:**

- Use main sections 12.1, 12.2, and 12.3 to distinguish current live surfaces, target architecture, and expansion candidates before assigning tests.
- Use linked-liability section 10.3, OnionNet section 10.1, and agentic section 8.2 only as dependency-order inputs for matrix rows.
- Do not treat these source fragments as proof that a test already exists; they only justify where crate-local and simulator tests must be added.

**Locality gate:**

- The matrix is a local verification plan. It names tests that developers can implement and run without external services.

**Implementation boundary:**

- In scope: test ownership and minimum acceptance scenarios per package.
- Out of scope: replacing detailed test plans inside each crate, or claiming tests already exist before implementation.

**Implementation tasks:**

1. Add or update tests in the crate that owns each rule.
2. Add simulator scenarios only after lower-level unit tests define the rule.
3. Add fixtures under existing test fixture patterns rather than ad hoc files.
4. Use deterministic RNG/time providers where existing code supports them.
5. Keep external-looking fixtures clearly named as local/mock/test.

**Tests and simulation:**

Use the matrix below as the minimum local verification map. Each row must have crate-local rule tests before any cross-crate simulator scenario is treated as evidence. Simulator scenarios should demonstrate integration behavior, not define the rule by themselves.

- Add crate-local rule tests for every package row before simulator evidence is accepted.
- Add simulator scenarios only for behavior that crosses crate boundaries or needs restart, replay, tamper, or local publication evidence.
- Record which lower-level tests each simulator scenario depends on.
- Keep all external-looking fixtures explicitly labeled as local, mock, or test data.

**Minimum matrix:**

| Package                                      | `z00z_core`                                                | `z00z_crypto`                                   | `z00z_storage`                                | `z00z_wallets`                                               | `z00z_simulator`                                            |
| -------------------------------------------- | ---------------------------------------------------------- | ----------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------ | ----------------------------------------------------------- |
| Core settlement hardening                    | asset object validation, policy metadata, golden encodings | commitment/range/AEAD facade tests              | path/root/claim-root/replay tests             | tx/claim package pre-broadcast tests                         | claim publish and checkpoint continuity                     |
| Wallet receive/scan/import authority         | request-binding metadata if core-owned                     | request/signature crypto helpers through facade | tx evidence and final spent-state persistence | receive taxonomy, scan orchestration, import gates, and nullifier transition | offline handoff, report-only/import parity, and restart     |
| Wallet history/asset convergence             | asset-facing metadata contracts if core-owned              | digest helpers if needed                        | accepted-status and tx evidence persistence   | tx-history and `wallet.asset.*` agreement                    | history projection and restart drift                        |
| Privacy/selective disclosure                 | public leaf fields and metadata limits                     | owner tag, tag16, AEAD/AAD tests                | no secret persistence fixtures                | reveal/redaction/audit package tests                         | selective audit scenario                                    |
| Multi-asset families                         | family metadata and trust-tier validation                  | hash/digest helpers if needed                   | registry snapshot and derived lookup tests    | display/non-equivalence checks                               | local family accounting scenario                            |
| Local adapters/mock DA                       | adapter DTOs and replay keys                               | attestation digest helpers if needed            | external event/release replay registries      | wallet adapter import boundaries                             | local bridge and DA mock scenarios                          |
| Voucher/payroll/useful-work                  | policy-shaped right models                                 | claim proof digest helpers if needed            | claim/replay persistence                      | policy and audit checks                                      | voucher, payroll, B2B, useful-work                          |
| Linked liability                             | liability object model and policies                        | hidden commitment/proof statement hashes        | lock registry and evidence persistence        | lock enforcement and conflict extraction                     | fraud, lock, unlock scenario                                |
| Fee envelopes/rights wallet                  | fee envelope and right catalogs                            | sponsor auth digest/signature helpers           | fee pool state if persisted                   | inventory and mandate enforcement                            | agent/machine fee execution                                 |
| Machine capabilities                         | machine right models                                       | receipt digest helpers                          | spent cache and receipt persistence           | machine wallet policy                                        | charging, access, relay, compute                            |
| Agentic rights                               | agent right models                                         | receipt/escrow digest helpers                   | escrow/payout state if persisted              | agent wallet policy                                          | tool purchase, escrow, payout                               |
| OnionNet local control plane                 | registry, epoch, lane, route types                         | OnionSessionDomain, AEAD helpers                | replay ledger                                 | route builder and fail-closed policy                         | deterministic route and packet scenarios                    |
| Publication/recovery and checkpoint evidence | publication DTOs if core-owned                             | digest helpers if needed                        | artifact/link/evidence store                  | confirmation evidence consumption                            | mock DA, theorem, tamper, and restart faults                |
| Evaluation/proof-size/storage guardrails     | benchmark metadata schema if core-owned                    | crypto bench metadata                           | storage metrics and serialization checks      | wallet metrics and scan-authority guardrails                 | scenario metrics, proof-size sidecar, and report validation |

**Done when:**

- Each package has at least one crate-local test and, where it crosses crate boundaries, at least one simulator scenario.
- Every simulator scenario states which lower-level rule tests it depends on.
- Test names and fixture names use local/mock/test labels where appropriate.

**Doublecheck:**

- Local condition: satisfied. The matrix contains only local test and simulator work.
- Developer clarity: satisfied. Ownership by crate and scenario coverage are explicit.

## 
