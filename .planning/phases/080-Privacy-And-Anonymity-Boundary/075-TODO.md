## 5. Privacy And Anonymity Boundary

**Goal:**

- State the factual privacy boundary for local wallet, crypto, package, inbox-helper, and transport-adjacent work.
- Prove local privacy claims match implemented behavior: request-bound receive is preferred, compatibility lanes are bounded, helpers are advisory, packages are sensitive, and transport anonymity is not live.

**Source:**

- [Privacy and anonymity, scope and canonical truth](../.planning/phases/055-privacy-anonimity/10-Privacy-Anonymity.md#101-scope-and-canonical-truth)
- [Privacy and anonymity, current implemented privacy surfaces](../.planning/phases/055-privacy-anonimity/10-Privacy-Anonymity.md#103-current-implemented-privacy-surfaces)
- [Privacy and anonymity, canonical receive model](../.planning/phases/055-privacy-anonimity/10-Privacy-Anonymity.md#105-canonical-receive-model)
- [Privacy and anonymity, implementation direction and backlog](../.planning/phases/055-privacy-anonimity/10-Privacy-Anonymity.md#109-implementation-direction-and-backlog)
- [Privacy and anonymity, non-negotiable privacy rules](../.planning/phases/055-privacy-anonimity/10-Privacy-Anonymity.md#1010-non-negotiable-privacy-rules)

**Implementation-relevant fragments:**

- Use section 10.1 for the canonical privacy truth and concept-drift boundary.
- Use section 10.3 for currently implemented privacy surfaces, not future anonymity claims.
- Use section 10.5 for request-bound receive, compatibility receive, and prefilter boundaries.
- Use sections 10.9 and 10.10 for local backlog and non-negotiable rules around package sensitivity, helpers, logs, exports, and transport non-claims.

**Locality gate:**

- Privacy boundary work is local crypto/wallet/simulator behavior: receive validation, pack handling, package hygiene, selective disclosure, inbox helper scope, and transport-claim guardrails.
- No live anonymity overlay, inbox service, relay network, or external auditor is required.

**Implementation boundary:**

- In scope: state-level unlinkability tests, amount confidentiality tests, receiver identity minimization, request-bound receive, compatibility card/plain receive, `tag16` prefilter limits, package leak risk, inbox helper non-negotiables, and transport privacy as a separate future layer.
- Out of scope: claiming live transport anonymity, treating helper routing as consensus truth, replacing current pack contract prematurely, or turning compatibility receive lanes into preferred privacy lanes.

**Implementation tasks:**

1. Treat this source as the factual privacy boundary for phases `11`, `12`, `17`, `18`, `35`, and `36`.
2. Keep request-bound receive as the preferred privacy lane in wallet behavior and docs.
3. Keep card-only or plain receive as compatibility-only and test that it does not receive privacy claims beyond its actual properties.
4. In `z00z_crypto`, keep owner tag, `tag16`, AEAD, AAD, KDF, and commitment helpers behind approved facades.
5. In `z00z_wallets`, make receive validation enforce chain, request, expiry, identity, amount, version, and ownership checks before persistence.
6. Treat transaction packages, forwarding bundles, exports, backups, and logs as sensitive material.
7. Keep inbox helper records request-bound and off-consensus.
8. Keep OnionNet as future transport namespace unless deterministic local packet work is explicitly scoped as non-live.
9. Add wording and test guards that state privacy does not imply transport anonymity.

**Tests and simulation:**

- Request-bound receive tests for correct request, wrong request, expired request, wrong chain, wrong identity, wrong amount, and replay.
- Compatibility-lane tests proving card/plain receive works where allowed but is labeled lower privacy.
- `tag16` collision tests proving prefilter hits do not imply ownership.
- AEAD/AAD tests for wrong owner key, epoch, asset ID, package digest, nonce, and associated data.
- Package sensitivity tests proving logs, report DTOs, exports, and simulator reports do not expose plaintext secrets.
- Transport wording tests or doc checks preventing local packet discipline from being described as live anonymity.

**Done when:**

- Privacy claims in local plans match implemented local behavior.
- Request-bound receive, compatibility receive, inbox helper scope, and transport privacy boundaries are explicit and tested.
- No privacy section requires a live overlay network or external service.

**Doublecheck:**

- Local condition: satisfied. The work is crypto/wallet validation, local package hygiene, and simulator wording checks.
- Developer clarity: satisfied. Privacy lanes, forbidden claims, and concrete tests are explicit.

## 
