<!-- markdownlint-disable MD001 MD024 MD025 MD035 MD060 -->

# Z00Z Verification Orchestrator

> Я знаю, что есть математические языки, которые могут формализовать код и логику того, что происходит, и доказать как бы математическое комплаенс, соответствие или доказательство того, что система работает, или наоборот, доказать, что система неполна. Я имею в виду не в смысле Геделя неполна, а в смысле того, что есть дырки в логике и в коде. Я не помню точно, как называются эти системы, я бы хотел, чтобы ты мне напомнил.

Ты, скорее всего, имеешь в виду общий класс под названием **Formal Methods / Formal Verification** — формальные методы и формальная верификация.

Это не один язык, а целая группа инструментов, которые позволяют **записать систему как математическую модель**, задать **инварианты / правила / safety properties**, а потом либо доказать, что они всегда выполняются, либо найти **counterexample** — пример сценария, где логика ломается.

Главные названия, которые тебе нужно вспомнить:

### 1. **Formal Specification Languages**

Языки для описания логики системы до кода.

Примеры:

| Инструмент | Для чего хорош |
| --- | --- |
| **TLA+** | распределённые системы, консенсус, state machines, протоколы, ledger logic |
| **PlusCal** | алгоритмический синтаксис поверх TLA+ |
| **Alloy** | поиск дыр в моделях: ownership, rights, vouchers, constraints |
| **Z notation / B-Method / Event-B** | классические математические спецификации систем |

Для Z00Z это особенно важно: **TLA+** хорошо подходит для checkpoint/state transition/aggregator/validator логики, а **Alloy** — для проверки моделей `Asset / Rights / Voucher / Policy`, чтобы найти невозможные или опасные состояния. TLA+ используется как язык формальной спецификации и верификации сложных software/hardware/distributed-систем, а Alloy прямо позиционируется как язык моделирования, который может находить “holes” в security mechanisms. ([GitHub](https://github.com/tlaplus/awesome-tlaplus/blob/master/README.md?utm_source=chatgpt.com))

------

### 2. **Model Checkers**

Это инструменты, которые перебирают возможные состояния системы и ищут нарушение правил.

Примеры:

| Инструмент         | Проверяет                                     |
| ------------------ | --------------------------------------------- |
| **TLC**            | TLA+ модели                                   |
| **Apalache**       | symbolic model checking для TLA+ через SMT/Z3 |
| **SPIN / Promela** | concurrent / multithreaded / protocol logic   |
| **NuSMV / nuXmv**  | finite-state systems                          |

То есть ты пишешь:
“Никогда не должно быть состояния, где один и тот же coin/voucher потрачен дважды” — и model checker пытается найти сценарий, где это всё-таки возможно.

Apalache, например, анализирует TLA+ спецификации через bounded model checking, symbolic execution и inductive invariants, используя SMT-логику вроде Z3. ([Apalache](https://apalache-mc.org/?utm_source=chatgpt.com)) SPIN — классический open-source инструмент для формальной проверки многопоточных приложений и протоколов. ([spinroot.com](https://spinroot.com/?utm_source=chatgpt.com))

------

### 3. **Theorem Provers / Proof Assistants**

Это более тяжёлая артиллерия: ты не просто перебираешь состояния, а строишь **математическое доказательство**.

Примеры:

| Инструмент       | Для чего                                                     |
| ---------------- | ------------------------------------------------------------ |
| **Coq / Rocq**   | доказательства корректности программ, математики, протоколов |
| **Lean**         | математика, proof engineering, формально проверенный код     |
| **Isabelle/HOL** | сильная система для формальных доказательств                 |
| **Agda**         | dependent types, доказательства через типы                   |
| **ACL2**         | логика + Lisp-подобная система для hardware/software proofs  |

Rocq/Coq описывает себя как interactive theorem prover / proof assistant для формальных спецификаций и доказательства, что программы соответствуют спецификациям. ([Rocq](https://rocq-prover.org/?utm_source=chatgpt.com)) Lean — open-source язык и proof assistant для формально проверенного кода. ([Lean Language](https://lean-lang.org/?utm_source=chatgpt.com)) Isabelle тоже прямо описывается как proof assistant для выражения математических формул и доказательства их в логическом исчислении. ([Isabelle TUM](https://isabelle.in.tum.de/?utm_source=chatgpt.com))

------

### 4. **Verification-Aware Programming Languages**

Это языки, где спецификация и доказательство встроены прямо в код.

Примеры:

| Инструмент       | Что делает                                           |
| ---------------- | ---------------------------------------------------- |
| **Dafny**        | код + contracts + автоматическая проверка            |
| **F\***          | proof-oriented language, dependent types, SMT        |
| **Why3 / WhyML** | deductive program verification                       |
| **SPARK/Ada**    | safety-critical systems                              |
| **Viper**        | intermediate verification language                   |
| **F\* / Low\***  | особенно интересно для crypto/security-critical кода |

Dafny — verification-aware язык с native support для спецификаций и static program verifier. ([Dafny](https://dafny.org/?utm_source=chatgpt.com)) F* — proof-oriented язык, сочетающий dependent types, SMT automation и интерактивные доказательства. ([fstar-lang.org](https://fstar-lang.org/?utm_source=chatgpt.com)) Why3 — платформа deductive program verification, которая генерирует verification conditions и отдаёт их SMT/theorem provers. ([why3.org](https://why3.org/?utm_source=chatgpt.com))

------

### 5. **Code-Level Verification для Rust**

Поскольку Z00Z у тебя на Rust, тебе особенно важны:

| Инструмент  | Назначение                                     |
| ----------- | ---------------------------------------------- |
| **Kani**    | model checking Rust-кода, bounded verification |
| **Prusti**  | формальная верификация Rust contracts          |
| **Verus**   | SMT-based verification для Rust                |
| **Creusot** | deductive verification Rust через Why3         |
| **MIRAI**   | static analysis / abstract interpretation      |
| **Loom**    | проверка concurrency interleavings в Rust      |
| **Miri**    | UB / unsafe / interpreter-level checks         |

Kani официально описан как bit-precise model checker для Rust, полезный для проверки safety и correctness. ([GitHub](https://github.com/model-checking/kani?utm_source=chatgpt.com)) Prusti — verifier для Rust, который позволяет формально доказывать отсутствие багов и корректность code contracts. ([GitHub](https://github.com/viperproject/prusti-dev?utm_source=chatgpt.com))

------

### Ключевые термины, которые ты искал

Тебе нужно искать по таким словам:

**formal methods**
**formal verification**
**formal specification**
**model checking**
**theorem proving**
**proof assistant**
**SMT solver**
**deductive verification**
**program verification**
**software model checking**
**symbolic execution**
**abstract interpretation**
**Hoare logic**
**separation logic**
**temporal logic**
**invariants**
**refinement types**
**dependent types**
**correctness proof**
**counterexample generation**

------

### В твоём контексте Z00Z я бы разложил так

**1. Архитектура / ledger / checkpoint / validator logic**
→ **TLA+ + TLC/Apalache**

Проверять:

```text
- input cannot be spent twice
- checkpoint transition preserves supply
- spent_delta and created_delta are consistent
- validator never accepts invalid root transition
- aggregator cannot reorder state into invalid state
```

**2. Asset / Rights / Voucher / Policy model**
→ **Alloy**

Проверять:

```text
- rights не существуют без asset
- voucher cannot redeem twice
- non-transferable right cannot be moved
- policy cannot create dead asset
- receiver refusal does not lock value forever
```

**3. Rust implementation**
→ **Kani / Prusti / Verus / Creusot**

Проверять:

```text
- serialization/deserialization roundtrip
- no panic on malformed input
- state transition function preserves invariants
- verifier rejects invalid proof objects
- no arithmetic overflow in amount logic
```

**4. Криптографическая математика**
→ **Coq/Rocq, Lean, Isabelle, F\***

Проверять:

```text
- proof protocol soundness assumptions
- commitment equations
- transcript binding
- Fiat–Shamir domain separation
- no replay/malleability under formal assumptions
```

------

Главная фраза, которую ты искал: **formal verification / formal methods**.

А конкретные системы, которые тебе стоит запомнить первыми: **TLA+, Alloy, Coq/Rocq, Lean, Isabelle/HOL, Dafny, F\*, Why3, Kani, Prusti, Verus**.

------

------

> Собери мне тогда, пожалуйста, полный технологический стек, как я должен с ним работать, как я должен верифицировать свой код. Может быть, даже проверка документов, специфика перед тем или после того, как код имплементирован. Меня главным образом интересует поиск дырок, проблем, каких-то уязвимостей в системе ZOOS, начиная от криптографии и до конкретной правильной реализации. То есть мне нужен такой стек, который бы бежал параллельно развитию кода. То есть это должен быть некий скиллз в моем проекте, который бы на постоянной основе валидировали, верифицировали все, что сделано, плюс новые куски, которые добавляются.

Да. Для Z00Z я бы строил это не как “один инструмент для формальной проверки”, а как **Z00Z Continuous Verification Stack** — постоянный verification/security gate, который идёт рядом с разработкой кода.

Главная идея:

```text
Spec / Whitepaper / Threat Model
        ↓
Formal Models: TLA+ / Alloy / Tamarin
        ↓
Executable Invariants / Property Tests
        ↓
Rust Verification: Kani / Verus / Miri / Loom
        ↓
Fuzzing / Differential Tests / Crypto Tests
        ↓
Supply-chain / unsafe / dependency audit
        ↓
CI gate + security report per PR
```

------

## 1. Базовое архитектурное решение

Для Z00Z я бы разделил проверки на **5 уровней**:

| Уровень                               | Что проверяет                                                | Главные инструменты                                          |
| ------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **L0 — Documents & Spec Consistency** | не противоречат ли whitepaper/spec/code друг другу           | Markdown specs, YAML invariants, traceability matrix         |
| **L1 — Protocol Logic**               | возможны ли double-spend, invalid checkpoint, locked voucher, broken rights | TLA+, Apalache, Alloy                                        |
| **L2 — Crypto Protocols**             | replay, malleability, secrecy, transcript binding, stealth delivery | Tamarin, ProVerif, hacspec, EasyCrypt                        |
| **L3 — Rust Implementation**          | panic, overflow, invalid state transition, UB, concurrency bugs | Kani, Verus, Creusot, Prusti, Miri, Loom                     |
| **L4 — Security Engineering**         | fuzzing, dependencies, unsafe code, constant-time, CI hardening | proptest, cargo-fuzz, cargo-audit, cargo-deny, cargo-vet, dudect |

TLA+/TLC хорошо подходит для конечных моделей state machines и distributed protocols; TLC — explicit-state model checker для TLA+, а Apalache добавляет symbolic/bounded/invariant checking через SMT. ([docs.tlapl.us](https://docs.tlapl.us/using%3Atlc%3Astart?utm_source=chatgpt.com)) Alloy особенно полезен именно для поиска “дыр” в моделях объектов и security mechanisms, что хорошо ложится на `Asset / Rights / Voucher / Policy`. ([alloytools.org](https://alloytools.org/?utm_source=chatgpt.com))

------

## 2. Что именно проверять в Z00Z

### A. Ledger / JMT / Checkpoint / Aggregator

Это самый важный слой.

Проверять:

```text
- один input/output не может быть потрачен дважды;
- checkpoint transition сохраняет корректность root;
- spent_delta и created_delta согласованы;
- validator не принимает transition без proof;
- aggregator не может скрыть spent input;
- prev_root → new_root всегда объясняется typed delta;
- no output creation without valid proof payload;
- no spend after checkpoint finalization;
- replay старого TxProof невозможен;
- supply conservation выполняется на уровне commitments / denominations.
```

Инструменты:

```text
TLA+ / Apalache:
  specs/tla/Z00Z_Checkpoint.tla
  specs/tla/Z00Z_Aggregator.tla
  specs/tla/Z00Z_OfflineBundle.tla

Kani / Verus:
  crates/z00z-state/src/transition.rs
  crates/z00z-validator/src/checkpoint_verify.rs

proptest:
  tests/properties/state_transition_props.rs

cargo-fuzz:
  fuzz/fuzz_targets/checkpoint_blob.rs
```

Для этого слоя **TLA+ важнее Coq/Lean**. Тебе не нужно сразу доказывать всю математику. Сначала нужно заставить model checker найти контрпримеры: double-spend, неправильный порядок deltas, voucher deadlock, invalid root transition.

------

### B. Asset / Rights / Voucher / Policy

Здесь лучше всего использовать **Alloy**.

Почему: Alloy хорошо ищет невозможные или неожиданные комбинации объектов. Для твоей текущей архитектуры это критично, потому что у тебя есть `Asset`, `Rights`, `Voucher`, `Policy`, и риск именно в том, что логика формально “кажется красивой”, но даёт мёртвые состояния.

Проверять:

```text
- Asset не существует без валидного ownership/control path;
- Rights не могут жить отдельно от Asset, если ты выбрал rights-bound-to-asset;
- Voucher cannot redeem twice;
- Voucher cannot permanently lock value without reclaim/expiry path;
- Non-transferable right cannot be transferred through wrapper/voucher;
- Receiver refusal does not destroy or freeze sender value forever;
- Policy cannot create asset that nobody can ever spend;
- Unspendable right cannot be converted into spendable by state transition;
- Mutable policy cannot mutate immutable asset.
```

Рекомендуемый файл:

```text
specs/alloy/z00z_asset_rights_voucher.als
```

Alloy должен стать твоим **первым фильтром для новых идей**. Перед тем как кодить новую модель voucher/right/action/policy, сначала прогонять её в Alloy.

------

### C. Stealth Address / Inbox / Offline Delivery

Здесь нужны **Tamarin** и частично **ProVerif**.

Проверять:

```text
- attacker cannot derive receiver secret from public delivery data;
- replay inbox message cannot create second spend;
- sender cannot force receiver into accepting toxic voucher;
- aggregator cannot forge notification;
- owner_tag/tag16 не раскрывает identity сверх выбранной модели;
- PaymentRequest_v1 нельзя переиспользовать вне intended context;
- Fiat–Shamir transcript binds all required fields;
- domain separation не пересекается между proof types.
```

Tamarin прямо предназначен для symbolic verification и attack finding в security protocols. ProVerif также используется для автоматической проверки криптографических протоколов и моделирования криптографических примитивов через symbolic rules. ([tamarin-prover.com](https://tamarin-prover.com/?utm_source=chatgpt.com))

Рекомендуемые файлы:

```text
specs/tamarin/z00z_stealth_delivery.spthy
specs/proverif/z00z_inbox_replay.pv
specs/crypto/domain_separation_registry.yaml
specs/crypto/transcript_binding.md
```

------

### D. Crypto implementation

Здесь правило жёсткое:

> **Не писать свои cryptographic primitives, если можно взять хорошо проверенную библиотеку.**

Для Z00Z можно писать protocol glue, transcript binding, commitments layout, serialization, proof object validation — но не свои curve arithmetic, hash primitive, RNG, scalar multiplication.

Что проверять:

```text
- all transcript labels unique;
- all Fiat–Shamir challenges bind protocol name/version/root/input/output/proof type;
- no ambiguous serialization;
- no unchecked decompression of points;
- no secret-dependent branching;
- no panic on malformed proof;
- no accept-by-default parsing;
- no accidental reuse of randomness;
- no RNG fallback;
- no domain label collision between TxProof / CheckpointProof / VoucherProof.
```

Инструменты:

```text
hacspec       — executable/verifiable crypto specifications in Rust subset
EasyCrypt     — game-based crypto security proofs
dudect        — statistical constant-time leakage detection
subtle        — constant-time helper traits for Rust crypto code
HACL*/EverCrypt — verified crypto implementation reference where applicable
```

hacspec — это functional subset of Rust для succinct, executable, verifiable crypto specifications. EasyCrypt используется для machine-checked cryptographic proofs в computational model. HACL*/EverCrypt — high-assurance crypto library family; HACL* code is verified for memory safety, functional correctness and secret independence. ([hacspec](https://hacspec.org/?utm_source=chatgpt.com)) dudect измеряет разные классы входов и статистически ищет timing leakage; `subtle` даёт best-effort constant-time utilities для Rust, но сам подчёркивает, что side channels зависят от всей deployed-системы, включая hardware. ([GitHub](https://github.com/oreparaz/dudect?utm_source=chatgpt.com))

------

## 3. Rust stack для постоянной проверки

Я бы выбрал такой стек.

### Fast gate — на каждый commit / PR

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --workspace --doc
cargo audit
cargo deny check
cargo geiger --all-features
```

`cargo-nextest` даёт более быстрый test runner с per-test isolation и CI support. `cargo-audit` проверяет `Cargo.lock` против RustSec advisories; `cargo-deny` проверяет advisories, licenses, banned crates, duplicate versions and sources. ([Nexte](https://nexte.st/?utm_source=chatgpt.com)) `cargo-geiger` показывает usage of `unsafe` в твоём crate и dependencies. ([GitHub](https://github.com/geiger-rs/cargo-geiger?utm_source=chatgpt.com))

------

### Medium gate — на каждый важный PR

```bash
cargo miri test -p z00z-crypto
cargo miri test -p z00z-state
cargo kani -p z00z-state
cargo kani -p z00z-validator
cargo test --workspace --features proptest-heavy
cargo fuzz run checkpoint_blob -- -max_total_time=300
cargo fuzz run tx_decode -- -max_total_time=300
```

Kani — Rust model checker для safety/correctness; Verus — verifier для functional correctness of low-level systems code; Creusot — deductive verifier для Rust через Why3; Prusti — Rust verifier для proving absence of bugs and correctness of contracts. ([model-checking.github.io](https://model-checking.github.io/kani/?utm_source=chatgpt.com)) Miri ловит Undefined Behavior в Rust tests, особенно важно для любого `unsafe`. ([GitHub](https://github.com/rust-lang/miri/?utm_source=chatgpt.com))

------

### Deep gate — nightly / release gate

```bash
apalache check specs/tla/Z00Z_Checkpoint.tla
apalache check specs/tla/Z00Z_OfflineBundle.tla

java -jar alloy.jar specs/alloy/z00z_asset_rights_voucher.als

tamarin-prover specs/tamarin/z00z_stealth_delivery.spthy

cargo fuzz run checkpoint_blob -- -max_total_time=7200
cargo fuzz run proof_decode -- -max_total_time=7200
cargo fuzz run wallet_bundle -- -max_total_time=7200

cargo vet check
cargo semver-checks check-release
```

`cargo-fuzz` is the recommended Rust fuzz testing tool and invokes libFuzzer; `proptest` generates inputs automatically and shrinks failing cases to minimal counterexamples. ([rust-fuzz.github.io](https://rust-fuzz.github.io/book/cargo-fuzz.html?utm_source=chatgpt.com)) `cargo-vet` helps ensure third-party Rust dependencies have been audited by a trusted entity. ([GitHub](https://github.com/mozilla/cargo-vet?utm_source=chatgpt.com))

------

## 4. Как это должно лежать в проекте

Я бы сделал такую структуру:

```text
z00z/
  specs/
    whitepapers/
      Z00Z-Main-Whitepaper.md
      Z00Z-ECC-StealthAddress.md
      Z00Z-Rights-Assets-Vouchers.md

    threat-model/
      threat-model.md
      attacker-capabilities.md
      known-non-goals.md
      privacy-failures.md

    invariants/
      z00z_global_invariants.yaml
      checkpoint_invariants.yaml
      voucher_invariants.yaml
      crypto_invariants.yaml

    tla/
      Z00Z_Checkpoint.tla
      Z00Z_Aggregator.tla
      Z00Z_OfflineBundle.tla

    alloy/
      z00z_asset_rights_voucher.als
      z00z_policy_model.als

    tamarin/
      z00z_stealth_delivery.spthy
      z00z_payment_request.spthy

    crypto/
      domain_separation_registry.yaml
      transcript_binding.md
      proof_objects_schema.yaml

  crates/
    z00z-crypto/
    z00z-state/
    z00z-wallet/
    z00z-validator/
    z00z-aggregator/
    z00z-proofs/

  tests/
    properties/
      state_transition_props.rs
      voucher_props.rs
      serialization_roundtrip_props.rs

    integration/
      checkpoint_pipeline.rs
      offline_bundle_pipeline.rs

  fuzz/
    fuzz_targets/
      tx_decode.rs
      checkpoint_blob.rs
      proof_decode.rs
      wallet_bundle.rs

  verification/
    kani/
      state_transition_harness.rs
      checkpoint_harness.rs

    verus/
      amount_model.rs
      delta_model.rs

    reports/
      latest-verification-report.md

  skills/
    z00z-verification-gate/
      SKILL.md
      prompts/
        spec_critic.md
        crypto_critic.md
        rust_critic.md
        invariant_extractor.md
        attack_surface_generator.md
      scripts/
        verify-p0.sh
        verify-p1.sh
        verify-nightly.sh
        extract-invariants.py
        check-traceability.py
```

------

## 5. Главный принцип: spec-before-code

Для Z00Z я бы ввёл правило:

> **Новый кусок логики нельзя кодить напрямую. Сначала он должен появиться как spec + invariant + attack surface.**

Например, ты хочешь добавить новый тип `Voucher`.

До кода должны появиться:

```text
1. specs/whitepapers/Z00Z-Rights-Assets-Vouchers.md
2. specs/invariants/voucher_invariants.yaml
3. specs/alloy/z00z_asset_rights_voucher.als
4. tests/properties/voucher_props.rs
5. fuzz/fuzz_targets/voucher_decode.rs
6. threat-model update
```

И только потом:

```text
crates/z00z-state/src/voucher.rs
crates/z00z-validator/src/voucher_verify.rs
```

------

## 6. Traceability matrix — обязательная вещь

Нужно связать документы, формальные модели, тесты и код.

Пример:

```yaml
REQ-Z00Z-CHECKPOINT-001:
  statement: "A checkpoint transition must consume each input at most once."
  spec: specs/whitepapers/Z00Z-Main-Whitepaper.md#checkpoint-transition
  tla_model: specs/tla/Z00Z_Checkpoint.tla
  rust_code:
    - crates/z00z-state/src/transition.rs
    - crates/z00z-validator/src/checkpoint_verify.rs
  property_tests:
    - tests/properties/state_transition_props.rs
  kani_harness:
    - verification/kani/state_transition_harness.rs
  fuzz_targets:
    - fuzz/fuzz_targets/checkpoint_blob.rs
  threat:
    - THREAT-DOUBLE-SPEND-001
  status: enforced
```

Это решает твою проблему: **документ говорит одно, код делает другое**. Любое изменение в коде должно ссылаться на invariant ID.

------

## 7. Что должен делать `z00z-verification-gate` skill

Это должен быть не просто prompt. Это должен быть **агентный gate**, который запускает проверки и пишет отчёт.

### Skill должен делать 7 вещей

```text
1. Read changed files in PR / git diff.
2. Detect affected domain:
   - crypto
   - state transition
   - wallet
   - validator
   - aggregator
   - voucher/right/policy
   - serialization
   - concurrency
   - documentation only

3. Extract or update invariants.
4. Generate attack hypotheses using SSoT / adversarial prompts.
5. Map each hypothesis to:
   - spec
   - code
   - test
   - formal model
   - reproducible check

6. Run machine checks.
7. Produce final report:
   - passed
   - failed
   - unknown / not verified
   - requires human cryptographer review
```

Важно: LLM/skill не должен быть “верификатором”. Он должен быть **attack-surface generator + orchestrator**. Истина — это артефакты: TLA counterexample, Alloy instance, failing proptest seed, fuzz crash, Kani failure, Miri UB report, cargo-audit finding.

------

## 8. Категории проверок для Z00Z

Я бы зафиксировал такие security gates.

### Gate 1 — State correctness

```text
- checkpoint root transition valid;
- spent_delta exactly matches consumed inputs;
- created_delta exactly matches new outputs;
- no duplicate input inside same checkpoint;
- no input spent after finalization;
- invalid proof cannot alter state;
- validator is deterministic;
- aggregator cannot produce accepted but unverifiable checkpoint.
```

Инструменты:

```text
TLA+ / Apalache
Kani
proptest
cargo-fuzz
```

------

### Gate 2 — Ownership / rights correctness

```text
- no orphan rights;
- no orphan asset;
- no invisible transferable wrapper around non-transferable right;
- no dead voucher;
- no voucher double redemption;
- no hidden policy escalation;
- no mutable policy over immutable asset.
```

Инструменты:

```text
Alloy
proptest
Kani
manual design review
```

------

### Gate 3 — Crypto protocol safety

```text
- transcript binding complete;
- domain separation complete;
- no replay across proof types;
- no ambiguous encoding;
- no malleable proof object;
- stealth delivery does not leak receiver identity beyond chosen leakage model;
- inbox notification cannot become spend authority;
- PaymentRequest_v1 cannot be replayed into another context.
```

Инструменты:

```text
Tamarin
ProVerif
hacspec
EasyCrypt for selected constructions
dudect for timing checks
```

------

### Gate 4 — Rust implementation safety

```text
- no panic on untrusted input;
- no unwrap/expect in validator path;
- no arithmetic overflow;
- no unchecked deserialization;
- no unsafe without local proof comment;
- no nondeterminism in consensus/validator code;
- no time/random/network dependency inside deterministic verifier.
```

Инструменты:

```text
clippy
Miri
Kani
Verus / Creusot for core pure functions
cargo-fuzz
proptest
```

------

### Gate 5 — Dependency / supply chain safety

```text
- no vulnerable crates;
- no duplicate crypto crates unless approved;
- no unexpected licenses;
- no git dependencies without pinning;
- no unreviewed unsafe-heavy dependencies;
- no dependency update without cargo-vet review.
```

Инструменты:

```text
cargo-audit
cargo-deny
cargo-vet
cargo-geiger
cargo-semver-checks
```

------

## 9. Что не надо делать сразу

Я бы **не начинал** с Coq/Lean для всей системы. Это слишком тяжёлый путь.

Правильный порядок:

```text
P0:
  Rust tests + proptest + fuzz + cargo-audit + cargo-deny + Miri + Kani

P1:
  TLA+ for checkpoint/aggregator/offline bundle
  Alloy for rights/voucher/policy

P2:
  Tamarin/ProVerif for stealth/inbox/payment request
  Verus/Creusot for critical Rust functions

P3:
  EasyCrypt / Coq / Lean only for selected crypto constructions
```

Coq/Lean/Isabelle нужны позже, когда у тебя уже стабилизировалась математика. На раннем этапе больше пользы дадут **counterexample tools**: TLA+, Apalache, Alloy, fuzzing, Kani.

------

## 10. Минимальный рабочий стек, который я бы поставил первым

Если коротко, для Z00Z я бы выбрал **обязательный initial stack**:

```text
Formal protocol:
  - TLA+
  - Apalache
  - Alloy

Crypto protocol:
  - Tamarin
  - ProVerif
  - hacspec
  - dudect

Rust verification:
  - Kani
  - Verus
  - Miri
  - Loom

Testing:
  - cargo-nextest
  - proptest
  - cargo-fuzz

Supply chain:
  - cargo-audit
  - cargo-deny
  - cargo-vet
  - cargo-geiger

Project discipline:
  - specs/invariants/*.yaml
  - traceability matrix
  - threat-model.md
  - domain_separation_registry.yaml
  - z00z-verification-gate skill
```

------

## 11. Самое важное правило для Z00Z

Я бы зафиксировал это как внутренний закон проекта:

```text
No invariant — no code.
No threat model — no protocol.
No fuzz target — no parser.
No Kani/proptest harness — no state transition.
No domain label — no Fiat–Shamir challenge.
No Alloy model — no new rights/voucher/policy object.
No TLA+ model — no checkpoint/aggregator/offline settlement change.
No human crypto review — no new cryptographic construction.
```

Вот это и будет правильный стек: не “один формальный язык”, а **постоянная система доказательства, атаки и регрессии**, которая параллельно развитию Z00Z ищет дырки в документах, протоколе, криптографии и Rust-реализации.

---

---

> Ты считаешь, что можно это всё организовать в некие skills и скрипты, которые будут запускаться параллельно с развитием кода, то я бы хотел, чтобы ты более точно сформулировал эти skills, и я бы хотел, чтобы ты, я имею в виду на всех уровнях, которые ты описал, там 5 уровней, и я бы хотел, чтобы ты мне сказал, что нужно установить в системе все ли эти инструменты, они написаны на Rust, нужно ли мне установить какие-то дополнительные конкретные инструменты и откуда их брать. Понятно, что предпочтение для Rust, но если нет другого выхода, значит, нужно установить те системы, на которых оно работает.

Да. Я бы организовал это как **Z00Z Continuous Verification Skills**: не один большой “магический skill”, а набор независимых skills + scripts, которые запускаются локально, в CI, на PR и nightly.

Главная мысль: **LLM-skill не является доказателем**. Он должен быть **orchestrator + attack-surface generator + report writer**. Реальные проверки должны делать Kani, TLA+, Alloy, Tamarin, fuzzing, Miri, cargo-audit и т.д.

------

# 1. Сначала ответ: всё ли это Rust?

Нет.

| Уровень                               | Инструменты                                            | Rust?                         |
| ------------------------------------- | ------------------------------------------------------ | ----------------------------- |
| L0 — docs/spec gate                   | mdBook, lychee, taplo, markdownlint, custom Python     | частично Rust                 |
| L1 — protocol/model checking          | TLA+, TLC, Apalache, Alloy                             | в основном Java/JVM           |
| L2 — crypto protocol                  | Tamarin, ProVerif, hacspec, EasyCrypt, dudect          | смешано: Rust/C/OCaml/Haskell |
| L3 — Rust implementation verification | Kani, Miri, Loom, Verus, Creusot, proptest, cargo-fuzz | в основном Rust               |
| L4 — supply-chain/security            | cargo-audit, cargo-deny, cargo-vet, cargo-geiger       | Rust                          |

Для Z00Z я бы **сразу ставил L0, L1, L3, L4**, а L2 ставил постепенно: сначала Tamarin/ProVerif для stealth/inbox/payment request, потом EasyCrypt/hacspec только для реально стабилизированной криптографии.

------

# 2. Что поставить в систему

## 2.1. Базовые системные зависимости

Для Linux я бы начал так:

```bash
sudo apt update

sudo apt install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  clang \
  lld \
  cmake \
  git \
  curl \
  unzip \
  openjdk-17-jdk \
  graphviz \
  opam \
  z3 \
  nodejs \
  npm
```

Почему Java 17: TLA+ tools требуют Java 11+, а Alloy 6 требует recent JVM / Java 17+, поэтому OpenJDK 17 закрывает оба случая. ([GitHub](https://github.com/tlaplus/tlaplus?utm_source=chatgpt.com))

------

## 2.2. Rust toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

rustup update
rustup component add rustfmt clippy rust-src

rustup toolchain install nightly
rustup +nightly component add miri rust-src llvm-tools-preview
```

Rust официально ставится через `rustup`, а компоненты вроде `clippy`, `rustfmt`, `rust-src` управляются через `rustup component`. Miri ставится на nightly через `rustup +nightly component add miri` и запускается как `cargo miri test`. ([Rust](https://rust-lang.org/install.html?utm_source=chatgpt.com))

------

## 2.3. Rust verification / testing / security tools

```bash
for tool in \
  cargo-nextest \
  cargo-audit \
  cargo-deny \
  cargo-vet \
  cargo-geiger \
  cargo-fuzz \
  cargo-llvm-cov \
  just \
  bacon \
  watchexec-cli \
  mdbook \
  lychee \
  taplo-cli
do
  cargo install --locked "$tool"
done

cargo install --locked kani-verifier
cargo kani setup
```

`cargo-nextest` — быстрый test runner для Rust; `cargo-fuzz` — стандартный wrapper вокруг libFuzzer для Rust; `cargo-audit` проверяет `Cargo.lock` по RustSec; `cargo-deny` проверяет advisories, licenses, bans, duplicate dependencies and sources; `cargo-vet` нужен для audit trail по third-party dependencies; `cargo-geiger` показывает unsafe usage в crate и dependencies. ([Nexte](https://nexte.st/?utm_source=chatgpt.com))

Kani ставится отдельно: официальная инструкция — `cargo install --locked kani-verifier`, затем `cargo kani setup`. ([model-checking.github.io](https://model-checking.github.io/kani/install-guide.html))

------

## 2.4. Formal protocol tools: TLA+, Apalache, Alloy

Я бы не ставил их через хаотичные system packages. Лучше держать в проекте:

```text
tools/
  formal/
    tla/
      tla2tools.jar
    apalache/
      apalache
    alloy/
      org.alloytools.alloy.dist.jar
```

Смысл: **pin exact versions**. Иначе через полгода CI начнёт вести себя иначе.

TLA+ можно использовать через официальный VS Code extension, который поддерживает TLC model checker, или через `tla2tools.jar` из releases; Apalache — symbolic model checker для TLA+, который проверяет bounded executions и inductive invariants через SMT/Z3; Alloy распространяется как runnable jar и используется для поиска моделей/counterexamples в relational specifications. ([docs.tlapl.us](https://docs.tlapl.us/using%3Avscode%3Astart?utm_source=chatgpt.com))

------

## 2.5. Crypto protocol tools: Tamarin, ProVerif, EasyCrypt, hacspec, dudect

```bash
opam init
eval "$(opam env)"
opam update

opam install proverif
opam install why3
opam install easycrypt
```

ProVerif официально можно ставить через OPAM; Why3 проще всего ставится через `opam install why3`; EasyCrypt тоже официально поддерживает `opam install easycrypt`. ([bblanche.gitlabpages.inria.fr](https://bblanche.gitlabpages.inria.fr/proverif/README?utm_source=chatgpt.com))

Tamarin я бы ставил отдельно — через Homebrew/Nix/Arch package или binary release. Официальная документация прямо говорит, что на macOS/Linux самый простой вариант — Homebrew, также есть Arch/Nix и binaries. ([tamarin-prover.com](https://tamarin-prover.com/manual/master/book/002_installation.html?utm_source=chatgpt.com))

```bash
# Если используешь Homebrew / Linuxbrew:
brew install tamarin-prover/tap/tamarin-prover

# Если Arch:
sudo pacman -S tamarin-prover

# Если Nix:
nix profile install nixpkgs#tamarin-prover
```

`hacspec` — это Rust-like functional subset для executable/verifiable crypto specifications; `dudect` — C-based timing leakage detection tool, которому нужен C compiler; HACL*/EverCrypt — high-assurance verified crypto reference family, но я бы использовал это как reference/interop source, а не как обязательный первый gate. ([hacspec](https://hacspec.org/?utm_source=chatgpt.com))

------

# 3. Какие skills я бы сделал

Я бы сделал **6 skills**: 5 по уровням + 1 orchestrator.

```text
skills/
  z00z-verification-orchestrator/
    SKILL.md

  z00z-spec-gate/
    SKILL.md
    scripts/
      check-docs.sh
      check-traceability.py
      extract-invariants.py

  z00z-protocol-model-gate/
    SKILL.md
    scripts/
      run-tla.sh
      run-apalache.sh
      run-alloy.sh

  z00z-crypto-protocol-gate/
    SKILL.md
    scripts/
      run-tamarin.sh
      run-proverif.sh
      check-domain-separation.py
      check-transcript-binding.py
      run-dudect.sh

  z00z-rust-implementation-gate/
    SKILL.md
    scripts/
      verify-fast.sh
      verify-kani.sh
      verify-miri.sh
      verify-loom.sh
      verify-verus.sh

  z00z-fuzz-security-gate/
    SKILL.md
    scripts/
      run-fuzz-short.sh
      run-fuzz-nightly.sh
      minimize-crash.sh

  z00z-supply-chain-gate/
    SKILL.md
    scripts/
      audit.sh
      deny.sh
      vet.sh
      unsafe-report.sh
```

------

# 4. Skill 0 — `z00z-verification-orchestrator`

Это главный skill. Он не проверяет сам, а решает, какие gates запускать.

## Назначение

```text
Purpose:
  Analyze git diff, classify touched files, select relevant verification gates,
  run scripts, collect reports, and produce final verification status.

Inputs:
  - git diff
  - changed files
  - specs/invariants/*.yaml
  - Cargo.toml / Cargo.lock
  - formal models
  - previous verification reports

Outputs:
  - verification/reports/latest.md
  - verification/reports/latest.json
  - PASS / FAIL / UNKNOWN / NEEDS_HUMAN_CRYPTO_REVIEW
```

## Логика запуска

```text
If changed files include:
  specs/**/*.md or specs/**/*.yaml:
    run z00z-spec-gate

  specs/tla/** or crates/z00z-state/** or crates/z00z-validator/**:
    run z00z-protocol-model-gate
    run z00z-rust-implementation-gate

  specs/alloy/** or crates/z00z-rights/** or voucher/policy code:
    run z00z-protocol-model-gate

  crates/z00z-crypto/** or proof/transcript/domain files:
    run z00z-crypto-protocol-gate
    run z00z-rust-implementation-gate
    run z00z-fuzz-security-gate

  Cargo.toml or Cargo.lock:
    run z00z-supply-chain-gate

  parsers/deserializers/network inputs:
    run z00z-fuzz-security-gate
```

------

# 5. Skill 1 — `z00z-spec-gate`

Это gate для документов, whitepapers, invariants, threat model.

## Что он ловит

```text
- документ говорит одно, код делает другое;
- новый code path не привязан к invariant ID;
- новый protocol object не описан в spec;
- rights/voucher/policy добавлены без threat model;
- Fiat–Shamir challenge добавлен без domain label;
- proof object описан в коде, но отсутствует в proof_objects_schema.yaml;
- markdown/spec broken links;
- YAML/TOML schema errors.
```

## Инструменты

```text
Rust:
  mdbook
  lychee
  taplo-cli

Node optional:
  markdownlint-cli2

Custom:
  check-traceability.py
  extract-invariants.py
```

`mdBook` ставится через Cargo и используется для Markdown documentation books; `lychee` — Rust link checker; Taplo — TOML toolkit/linter/formatter; markdownlint-cli2 — Node-based Markdown/CommonMark linter. ([Rust Language](https://rust-lang.github.io/mdBook/guide/installation.html?utm_source=chatgpt.com))

## Скрипт

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "[spec-gate] markdown/doc build"
mdbook build specs/book || true

echo "[spec-gate] broken links"
lychee specs/**/*.md README.md || true

echo "[spec-gate] TOML lint"
taplo fmt --check Cargo.toml crates/*/Cargo.toml

echo "[spec-gate] traceability"
python3 skills/z00z-spec-gate/scripts/check-traceability.py

echo "[spec-gate] invariant extraction"
python3 skills/z00z-spec-gate/scripts/extract-invariants.py
```

------

# 6. Skill 2 — `z00z-protocol-model-gate`

Это самый важный formal layer для Z00Z.

## Что он проверяет

```text
Checkpoint / Aggregator / JMT:
  - no double spend
  - no invalid root transition
  - no created_delta without valid input/proof
  - no missing spent_delta
  - deterministic validator acceptance
  - aggregator cannot produce accepted but unverifiable checkpoint

Rights / Assets / Voucher:
  - no orphan right
  - no orphan asset
  - no dead voucher
  - no double redemption
  - non-transferable right cannot be transferred via wrapper
  - receiver refusal cannot lock value forever
```

## Инструменты

```text
TLA+ / TLC:
  checkpoint, aggregator, offline bundle state machines

Apalache:
  symbolic bounded model checking
  inductive invariants

Alloy:
  asset / rights / voucher / policy object model
```

TLA+ tools/TLC идут через Java; Apalache — symbolic checker для TLA+, использующий SMT/Z3; Alloy — open-source language/analyzer для software modeling и поиска дыр в security mechanisms. ([GitHub](https://github.com/tlaplus/tlaplus?utm_source=chatgpt.com))

## Файлы

```text
specs/tla/
  Z00Z_Checkpoint.tla
  Z00Z_Aggregator.tla
  Z00Z_OfflineBundle.tla

specs/alloy/
  z00z_asset_rights_voucher.als
  z00z_policy_model.als
```

## Скрипты

```bash
# scripts/run-tla.sh
#!/usr/bin/env bash
set -euo pipefail

java -cp tools/formal_verification/tla/tla2tools.jar tlc2.TLC \
  -config specs/tla/Z00Z_Checkpoint.cfg \
  specs/tla/Z00Z_Checkpoint.tla
# scripts/run-apalache.sh
#!/usr/bin/env bash
set -euo pipefail

tools/formal_verification/apalache/bin/apalache-mc check \
  specs/tla/Z00Z_Checkpoint.tla
# scripts/run-alloy.sh
#!/usr/bin/env bash
set -euo pipefail

java -jar tools/formal_verification/alloy/org.alloytools.alloy.dist.jar \
  specs/alloy/z00z_asset_rights_voucher.als
```

По Alloy я бы сделал два режима: GUI/manual для проектирования модели и headless/CI через jar/API/CLI wrapper. Сам jar официально runnable; для серьёзного CI лучше добавить маленький Java/Kotlin/Python wrapper, который запускает все `check` commands и возвращает non-zero exit code при counterexample. ([alloytools.org](https://alloytools.org/download.html?utm_source=chatgpt.com))

------

# 7. Skill 3 — `z00z-crypto-protocol-gate`

Это отдельный security gate. Его нельзя смешивать с обычными Rust tests.

## Что он проверяет

```text
Stealth / Inbox / PaymentRequest:
  - attacker cannot derive receiver secret
  - aggregator cannot forge spend authority
  - inbox notification cannot become ownership proof
  - old PaymentRequest cannot be replayed into another context
  - tag16/owner_tag leakage stays inside chosen leakage model

Transcript / Proofs:
  - TxProof and CheckpointProof cannot share domain
  - challenge binds protocol version, root, input, output, proof type
  - no missing field in Fiat-Shamir transcript
  - no malleable proof object
  - no ambiguous serialization

Implementation:
  - no secret-dependent branch in critical crypto path
  - no timing leakage on selected secret-dependent operations
```

## Инструменты

```text
Tamarin:
  stateful security protocols, temporal claims, attack finding

ProVerif:
  fast symbolic secrecy/authentication checks

hacspec:
  executable crypto specs in Rust-like subset

EasyCrypt:
  later-stage game-based crypto proofs

dudect:
  constant-time leakage detection
```

Tamarin прямо предназначен для symbolic verification/proving и falsification/attack finding в security protocols; ProVerif используется для automated symbolic protocol verification в Dolev-Yao model; EasyCrypt — OPAM-based proof system для cryptographic proofs; hacspec — executable/verifiable Rust-like crypto specification language; dudect статистически ищет timing leakage. ([tamarin-prover.com](https://tamarin-prover.com/?utm_source=chatgpt.com))

## Файлы

```text
specs/tamarin/
  z00z_stealth_delivery.spthy
  z00z_payment_request.spthy
  z00z_inbox_notify.spthy

specs/proverif/
  z00z_stealth_delivery.pv
  z00z_payment_request_replay.pv

specs/crypto/
  domain_separation_registry.yaml
  transcript_binding.md
  proof_objects_schema.yaml
```

## Скрипты

```bash
# scripts/check-domain-separation.py
# Проверяет, что каждый proof/challenge/domain имеет уникальный label.
# scripts/check-transcript-binding.py
# Проверяет, что каждый proof type bind-ит обязательные поля:
# protocol_id, version, root, input_refs, output_refs, asset_id, rights_id, policy_id.
# scripts/run-tamarin.sh
#!/usr/bin/env bash
set -euo pipefail

tamarin-prover --prove specs/tamarin/z00z_stealth_delivery.spthy
tamarin-prover --prove specs/tamarin/z00z_payment_request.spthy
# scripts/run-proverif.sh
#!/usr/bin/env bash
set -euo pipefail

proverif specs/proverif/z00z_stealth_delivery.pv
proverif specs/proverif/z00z_payment_request_replay.pv
```

------

# 8. Skill 4 — `z00z-rust-implementation-gate`

Это gate для actual Rust code.

## Что он ловит

```text
- panic on malformed input
- unwrap/expect in validator path
- arithmetic overflow
- invalid state transition
- nondeterministic validator behavior
- unsafe usage
- UB in unsafe code
- concurrency interleaving bugs
- proof verifier accepts malformed object
```

## Инструменты

```text
Always:
  cargo fmt
  cargo clippy
  cargo nextest
  Miri
  Kani
  proptest

For concurrency:
  Loom

For deeper formal verification:
  Verus or Creusot, not both at first
```

Loom прогоняет concurrent tests много раз, перебирая возможные interleavings under C11 memory model; Miri исполняет Rust tests в interpreter и ловит UB; Kani — Rust verifier/model checker; Verus проверяет functional correctness of Rust code; Creusot переводит Rust в Why3/Coma и проверяет panics/overflows/assertion failures plus annotated correctness. ([GitHub](https://github.com/tokio-rs/loom?utm_source=chatgpt.com))

## Мой выбор для Z00Z

Я бы сделал так:

```text
Use immediately:
  - Kani
  - Miri
  - Loom
  - proptest
  - cargo-fuzz

Use selectively:
  - Verus for pure critical kernels:
      amount arithmetic
      delta application
      supply preservation
      deterministic validation
  - Creusot as alternative to Verus, not together initially

Do not prioritize now:
  - Prusti, unless you specifically want VSCode-assisted experiments.
```

Prusti я бы не ставил как основной gate, потому что это prototype verifier; полезен для экспериментов, но для Z00Z я бы основную ставку делал на Kani + Verus/Creusot selectively. ([GitHub](https://github.com/viperproject/prusti-dev?utm_source=chatgpt.com))

## Скрипты

```bash
# scripts/verify-fast.sh
#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
# scripts/verify-miri.sh
#!/usr/bin/env bash
set -euo pipefail

cargo +nightly miri test -p z00z-crypto
cargo +nightly miri test -p z00z-state
cargo +nightly miri test -p z00z-validator
# scripts/verify-kani.sh
#!/usr/bin/env bash
set -euo pipefail

cargo kani -p z00z-state
cargo kani -p z00z-validator
cargo kani -p z00z-crypto
# scripts/verify-loom.sh
#!/usr/bin/env bash
set -euo pipefail

RUSTFLAGS="--cfg loom" cargo test -p z00z-aggregator loom_
RUSTFLAGS="--cfg loom" cargo test -p z00z-validator loom_
```

------

# 9. Skill 5 — `z00z-fuzz-security-gate`

Это gate для всего, что принимает bytes извне.

## Обязательные fuzz targets для Z00Z

```text
fuzz/fuzz_targets/
  tx_decode.rs
  proof_decode.rs
  checkpoint_blob.rs
  wallet_bundle.rs
  inbox_message.rs
  voucher_decode.rs
  payment_request.rs
  jmt_delta.rs
```

## Что он ловит

```text
- parser panic
- deserialization confusion
- accepted malformed proof object
- OOM / pathological input
- infinite loop
- ambiguous encoding
- invalid object accepted after roundtrip
```

`cargo-fuzz` официально описан как recommended Rust fuzz testing tool; он использует libFuzzer и требует nightly/compiler sanitizer support on supported Unix-like targets. ([rust-fuzz.github.io](https://rust-fuzz.github.io/book/cargo-fuzz.html?utm_source=chatgpt.com))

## Скрипты

```bash
# scripts/run-fuzz-short.sh
#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fuzz run tx_decode -- -max_total_time=120
cargo +nightly fuzz run proof_decode -- -max_total_time=120
cargo +nightly fuzz run checkpoint_blob -- -max_total_time=120
# scripts/run-fuzz-nightly.sh
#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fuzz run tx_decode -- -max_total_time=7200
cargo +nightly fuzz run proof_decode -- -max_total_time=7200
cargo +nightly fuzz run checkpoint_blob -- -max_total_time=7200
cargo +nightly fuzz run wallet_bundle -- -max_total_time=7200
cargo +nightly fuzz run voucher_decode -- -max_total_time=7200
```

------

# 10. Skill 6 — `z00z-supply-chain-gate`

Это gate для dependencies и unsafe.

## Что он ловит

```text
- vulnerable crate
- yanked crate
- duplicate crypto implementations
- unexpected license
- git dependency without pin
- unsafe-heavy dependency
- dependency drift
- unreviewed transitive dependency
```

## Скрипт

```bash
# scripts/audit.sh
#!/usr/bin/env bash
set -euo pipefail

cargo audit
cargo deny check
cargo vet check
cargo geiger --all-features
cargo tree -d
```

`cargo-audit` работает с RustSec advisory database; `cargo-deny` дополнительно проверяет licenses, sources, duplicate/banned crates; `cargo-vet` фиксирует trusted audits of third-party dependencies; `cargo-geiger` даёт статистику unsafe usage. ([rustsec.org](https://rustsec.org/?utm_source=chatgpt.com))

------

# 11. Как запускать это параллельно разработке

Я бы поставил `just` как главный command runner и `bacon`/`watchexec` для постоянного background mode. `just` — Rust command runner; `bacon` designed to run alongside editor and show Rust warnings/errors/tests; `watchexec` watches files and reruns commands on changes. ([GitHub](https://github.com/casey/just?utm_source=chatgpt.com))

## `justfile`

```make
p0:
    bash skills/z00z-rust-implementation-gate/scripts/verify-fast.sh
    bash skills/z00z-supply-chain-gate/scripts/audit.sh
    bash skills/z00z-spec-gate/scripts/check-docs.sh

p1:
    just p0
    bash skills/z00z-rust-implementation-gate/scripts/verify-kani.sh
    bash skills/z00z-rust-implementation-gate/scripts/verify-miri.sh
    bash skills/z00z-fuzz-security-gate/scripts/run-fuzz-short.sh

models:
    bash skills/z00z-protocol-model-gate/scripts/run-tla.sh
    bash skills/z00z-protocol-model-gate/scripts/run-apalache.sh
    bash skills/z00z-protocol-model-gate/scripts/run-alloy.sh

crypto:
    bash skills/z00z-crypto-protocol-gate/scripts/check-domain-separation.py
    bash skills/z00z-crypto-protocol-gate/scripts/check-transcript-binding.py
    bash skills/z00z-crypto-protocol-gate/scripts/run-proverif.sh
    bash skills/z00z-crypto-protocol-gate/scripts/run-tamarin.sh

nightly:
    just p1
    just models
    just crypto
    bash skills/z00z-fuzz-security-gate/scripts/run-fuzz-nightly.sh

release:
    just nightly
    cargo llvm-cov --workspace --all-features
    cargo semver-checks check-release
```

## В реальной работе

```bash
# постоянно рядом с VS Code
bacon

# перед commit
just p0

# перед PR
just p1

# если менял checkpoint/aggregator/state machine
just models

# если менял stealth/inbox/crypto/proofs
just crypto

# ночью / CI
just nightly
```

------

# 12. Какие проверки привязать к Z00Z-модулям

| Z00Z area                     | Mandatory gate                                |
| ----------------------------- | --------------------------------------------- |
| `z00z-state`                  | TLA+, Kani, proptest                          |
| `z00z-validator`              | TLA+, Kani, Miri, fuzz                        |
| `z00z-aggregator`             | TLA+, Loom, proptest                          |
| `z00z-crypto`                 | Miri, Kani, dudect, transcript/domain checker |
| `z00z-wallet`                 | fuzz, proptest, Miri                          |
| `stealth/inbox`               | Tamarin, ProVerif, fuzz                       |
| `asset/rights/voucher/policy` | Alloy, proptest, Kani                         |
| `parsers/deserializers`       | cargo-fuzz, proptest, Miri                    |
| `Cargo.lock`                  | cargo-audit, cargo-deny, cargo-vet            |
| `whitepapers/specs`           | spec-gate, traceability, invariant extraction |

------

# 13. Самое важное правило для code review

Каждый security-critical кусок кода должен иметь ссылку на invariant.

Например:

```rust
// ZINV: CHECKPOINT-001
// Every consumed input must appear exactly once in spent_delta.
pub fn apply_checkpoint_delta(...) -> Result<NewRoot, VerifyError> {
    ...
}
```

А в `specs/invariants/checkpoint_invariants.yaml`:

```yaml
CHECKPOINT-001:
  statement: "Every consumed input must appear exactly once in spent_delta."
  spec: "specs/whitepapers/Z00Z-Main-Whitepaper.md#checkpoint-transition"
  tla: "specs/tla/Z00Z_Checkpoint.tla"
  rust:
    - "crates/z00z-state/src/transition.rs"
    - "crates/z00z-validator/src/checkpoint_verify.rs"
  tests:
    - "tests/properties/state_transition_props.rs"
  kani:
    - "verification/kani/state_transition_harness.rs"
  fuzz:
    - "fuzz/fuzz_targets/checkpoint_blob.rs"
  status: "enforced"
```

`check-traceability.py` должен падать, если изменён critical file, но рядом нет `ZINV:` или invariant не существует.

------

# 14. Что я бы установил прямо сейчас

Мой практический выбор для первого этапа:

```text
Install immediately:
  - Rust stable + nightly
  - clippy / rustfmt / miri
  - cargo-nextest
  - cargo-fuzz
  - cargo-audit
  - cargo-deny
  - cargo-vet
  - cargo-geiger
  - Kani
  - mdbook
  - lychee
  - taplo-cli
  - just
  - bacon
  - TLA+ tools
  - Apalache
  - Alloy

Install second wave:
  - Tamarin
  - ProVerif
  - dudect
  - Verus

Install later / selectively:
  - Creusot / Why3
  - EasyCrypt
  - hacspec/hax
  - HACL*/EverCrypt reference integration
  - Coq/Lean/Isabelle only if a specific proof demands it
```

И самое важное: **не пытаться формально доказать весь Z00Z сразу**. Сначала надо построить continuous bug-finding machine: TLA+/Alloy для логических дыр, Kani/Miri/proptest/fuzz для реализации, Tamarin/ProVerif для crypto protocol, cargo-audit/deny/vet/geiger для dependency/security hygiene. Это даст максимальный security return на ранней стадии Z00Z.
