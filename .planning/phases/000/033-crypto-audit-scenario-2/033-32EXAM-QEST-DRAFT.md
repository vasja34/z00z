## Quest

📌 Claim register with final ratings.

1. `C3` Passive leaf or JMT observers get no direct receiver identifier from the leaf payload.
   PLAUSIBLE.
   Reason: the leaf format is privacy-oriented, but this is not a formal metadata-proof.

2. does the repository  implement a real stealth receive path and real Bulletproofs+ range-proof verification.

3. The repository does not justify the full claim as originally stated.

4. 🚨 The three most important conclusions are these.

   1. Alice does know sender-side output secret material in the current implementation.
   2. The spend and validator path is not yet backed by a complete public ZK verifier.
   3. The design documents themselves support sender knowledge of `s_out`; the unresolved issue is not sender ignorance, but the missing finished public proof boundary.

5. `C4` Alice does not know the asset secret.
   DISPUTED.
   Reason: sender code derives `k_dh` and `s_out` during output construction.

6. `C5` Alice still cannot spend without Bob's receiver secret.
   DISPUTED.
   Reason: local ownership logic depends on `receiver_secret`, but the report cannot promote this to an end-to-end proof-backed guarantee while the public spend boundary remains incomplete.

7. `C6` Validator can trustlessly verify spend correctness with a real ZK verifier.
   DISPUTED.
   Reason: the current spend gate is structural and placeholder-like, not a full public proof verifier.

8. `C8` Checkpoint or JMT publish is already backed by authoritative proof verification.
   DISPUTED.
   Reason: the storage or checkpoint path enforces non-empty proof bytes, but not full cryptographic proof validation.

9. The theft-resistance story must therefore come from an additional receiver-secret requirement at spend time, not from sender ignorance.

10. 🚨 The ideas-document sentence `Alice still cannot steal because Spend-TxProof requires receiver_secret` matches the intended ownership model, but it is stronger than the live proof boundary. 

11. receiver-secret-gated spend authorization is part of both the intended design and the local rule logic, but the repository does not yet justify the stronger end-to-end claim that Alice is cryptographically excluded by a finished public spend proof.

12. The validator-facing spend or checkpoint trust model is not yet cryptographically complete. the strong claim that an untrusted validator can fully verify privacy-preserving spend correctness from a real proof object alone ? repository does not yet prove that every accepted JMT state transition must satisfy it under an untrusted validator or aggregator.

13. today the aggregator is constrained by membership and batch-shape checks, but the report cannot honestly say the aggregator is prevented from theft specifically because it lacks Bob's secrets. That anti-theft statement remains blocked on a real proof verifier.

14. this is not a demonstrated break by itself, but it is exactly the sort of boundary ambiguity that blocks strong formal claims and makes proof, wire, and runtime drift more likely.

15. this is not evidence that sender should be ignorant of `s_out`; both document and code allow sender knowledge. The real issue is design drift in `s_out` derivation semantics, which should be frozen canonically before stronger proof claims are made.

16. spending should additionally require receiver-secret knowledge.

17. Storage and checkpoint code accepts non-empty proof bytes rather than verifying a real recursive proof transcript at the final boundary.

18. Repository memory records that current checkpoint artifacts accept non-empty `cp_proof` and that storage persists synthetic proof bytes.

19. `crates/z00z_simulator/src/scenario_1/stage_6_utils/bundle_lane_impl.rs` contains explicit placeholder comments for checkpoint aggregation and digest construction.

20.  Does `receiver_secret + s_in` actually guarantee that Alice cannot steal after the tx is inserted into JMT?

21. The authoritative checkpoint boundary does not yet enforce a real Bob-secret-gated spend proof.

22. Spend or checkpoint authorization is not yet backed by a finished authoritative proof verifier. not yet supported as a full trustless-proof claim.

23. JMT publish.
    Goal: the aggregator should publish without learning the recipient and without requiring trust for correctness.
    Implemented: a privacy-oriented leaf payload exists.
    Gap: checkpoint proof path is not yet authoritative; placeholder proof semantics remain.
    Verdict: FAIL FOR TRUSTLESSNESS.

24. Spend.
    Goal: spend should require hidden witness and preserve privacy.
    Implemented: rule logic exists for owner-tag, asset-id, balance, and range conditions.
    Gap: no complete public spend ZK verifier exists yet, so Bob's local recovery of `s_in` does not automatically become an authoritative chain-level exclusion of Alice.
    Verdict: FAIL FOR FULL ZK CLAIM.

25. Make checkpoint proof verification authoritative at the storage boundary

    📌 Required outcome: `cp_proof` and `tx_proof` are cryptographically validated, not only checked for non-empty bytes.

    📌 Minimum work.

    1. Replace synthetic proof bytes in storage or checkpoint paths.
    2. Verify proof objects during finalize and load operations.
    3. Reject artifacts that do not match canonical proof transcripts.

26. ### Harden receiver identity binding

    📌 Required outcome: sender cannot be tricked into encrypting to Mallory under Alice's directory entry.

    📌 Minimum work.

    1. Add TOFU plus pinning or signed directory binding for `ReceiverCard`.
    2. Treat request or card validation as mandatory, not just caller discipline.
    3. Freeze rotation behavior and mismatch handling.

27. ### Make request-bound tag derivation the normal privacy path

    📌 Required outcome: targeted scan spam becomes materially harder.

    📌 Minimum work.

    1. Prefer payment-request mode for normal sends.
    2. Bind `tag16` and optionally `k_dh` derivation to `req_id`.
    3. Add tests that show card-bound and request-bound behavior diverge in the intended way.

    ---

    

28. # HOW IT MUST BE DONE

    **Executive Verdict**

    Risky but salvageable.

    Короткий ответ: нет, это работает не так, что stealth-address Bob “вкладывает” внутрь монеты знание `receiver_secret`. Правильнее так: Alice строит монету по публичной карточке Bob, Bob потом узнаёт “это моё” и расшифровывает содержимое, но право траты в замысле должно проверяться отдельным условием: нужен не только `s_in`/`s_out`, но и секрет Bob `receiver_secret`. То есть stealth-address отвечает за распознавание и доставку, а не сам по себе за окончательное право траты.

    **Что Здесь Происходит**

    Это mixed review: дизайн + код + threat model.

    Цель безопасности здесь такая:

    - Alice может создать монету для Bob без handshake.
    - Наблюдатель не должен понять, кому она принадлежит.
    - Bob должен уметь её распознать и открыть.
    - Alice не должна уметь её потратить после публикации в JMT.

    В текущем замысле системы это достигается не через “Alice не знает `s_out`”, а через “Alice знает `s_out`, но не знает `receiver_secret` Bob”.

    **Строго По Шагам**

    1. Bob публикует не свой секрет, а публичную карточку:
       - `owner_handle = H(receiver_secret)`
       - `view_pk = f(receiver_secret) * G`

    2. Alice берёт эту карточку и строит output:
       - выбирает/получает `R_pub`
       - вычисляет общий секрет `k_dh`
       - формирует `owner_tag`
       - шифрует pack
       - в текущем коде получает `s_out` детерминированно из `k_dh + r_pub + serial`

    3. Bob при скане leaf:
       - по своему `receiver_secret` восстанавливает `view_sk`
       - по `R_pub` получает тот же `k_dh`
       - проверяет `owner_tag`
       - расшифровывает pack
       - достаёт `s_out`

    4. При трате, по intended logic, нужно доказать две вещи:
       - я знаю `s_in`, которое даёт `asset_id`
       - я знаю `receiver_secret`, которое даёт правильные `owner_handle` и `view_sk`

    Именно второе условие должно выбрасывать Alice.

    **Главная Мысль**

    `receiver_secret` не “зашивается” в монету так, чтобы Alice его узнала. Он используется как скрытый второй фактор владения.

    Аналогия, но точная:

    - `s_out` это как номер ячейки и ключ от внешней дверцы.
    - `receiver_secret` это как отпечаток пальца для внутреннего замка.
    - Alice может знать номер ячейки и даже внешний ключ.
    - Но если внутренний замок реально проверяется, без отпечатка Bob она не откроет ячейку до конца.

    Проблема текущего репозитория не в том, что эта логика неверная. Проблема в том, что “охранник у двери” пока не полностью реализован как настоящий публичный proof verifier.

    **Почему У Вас Возникает Разрыв Логики**

    Потому что интуитивно кажется так:

    - если Alice знает `s_out`,
    - а `asset_id = H(s_out)`,
    - значит Alice уже знает “секрет монеты”,
    - значит она должна мочь украсть.

    Это было бы правдой, если бы право траты определялось только `s_out`.

    Но в текущем B3-замысле право траты определяется не `s_out` alone, а парой:

    $$
    \text{spend authority} = (\text{receiver\_secret}, \text{s\_in})
    $$

    То есть:

    - `s_in` привязывает тебя к конкретной монете,
    - `receiver_secret` привязывает тебя к конкретному владельцу.

    Если убрать `receiver_secret`, Alice действительно сможет красть.
    Если `receiver_secret` реально обязателен в verifier, Alice не сможет красть, даже зная `s_out`.

    **Где Это Видно В Коде**

    Локальная логика траты это реально моделирует в spending.rs.

    Распознавание и извлечение секрета Bob после скана идёт через stealth_scan_support.rs.

    Построение sender-side output и derivation `s_out` видно в output.rs и ecdh.rs.

    А вот место, где становится видно, почему у вас остаётся недоверие: proof boundary всё ещё structural/placeholder-like в witness_gate.rs.

    **Самый Важный Ответ На Ваш Вопрос**

    Если протокол корректен, Alice должна быть отрезана уже на момент первой допустимой траты, потому что:

    - монета уже Bob-owned по правилу,
    - а validator принимает spend только если доказано знание `receiver_secret`.

    **То Есть Два Разных Дизайна**

    Текущий intended design:

     - Alice знает `s_out`
     - Bob тоже узнаёт `s_out`
     - но только Bob знает `receiver_secret`
     - spend requires both
     - re-encode не нужен

    **Критические Findings**

    | Field     | Content                                                      |
    | --------- | ------------------------------------------------------------ |
    | Severity  | S1                                                           |
    | Component | B3 ownership model                                           |
    | Problem   | Анти-кража зависит не от сокрытия `s_out`, а от обязательности `receiver_secret` в spend-verifier |
    | Impact    | Если verifier не требует `receiver_secret`, Alice сможет тратить, зная sender-side material |
    | Fix       | Сделать authoritative public verifier, который жёстко биндует `receiver_secret`, `s_in`, `owner_tag`, `asset_id`, `prev_root`, outputs |

    | Field     | Content                                                      |
    | --------- | ------------------------------------------------------------ |
    | Severity  | S1                                                           |
    | Component | JMT/checkpoint boundary                                      |
    | Problem   | Репозиторий пока не доказывает end-to-end, что все принятые state transitions реально enforceят B3 |
    | Impact    | Intended logic есть, но нет полной chain-level криптографической гарантии |
    | Fix       | Довести proof boundary до настоящего verifier, а не structural checks / non-empty proof bytes |

    | Field     | Content                                                      |
    | --------- | ------------------------------------------------------------ |
    | Severity  | S2                                                           |
    | Component | `s_out` semantics                                            |
    | Problem   | Документы говорят `random32`, код говорит `derive(k_dh, r_pub, serial)` |
    | Impact    | Это путает reasoning, но не меняет сам факт, что Alice знает `s_out` в обеих моделях |
    | Fix       | Канонизировать одну модель и переписать docs/code в одно состояние |

    **Ответ В Одной Фразе**

    Alice не должна быть отрезана от монеты потому, что она “не знает `s_out`”. Она должна быть отрезана потому, что даже зная `s_out`, она не знает `receiver_secret` Bob, а verifier должен требовать оба секрета вместе.

    **Final Decision**

    Blocked:

    1. Нет завершённого authoritative public spend verifier на границе принятия state transition.
    2. Не зафиксирована одна каноническая семантика `s_out`.
    3. Поэтому intended anti-theft logic понятна, но end-to-end доказанной гарантией в текущем live stack ещё не является.

    ---

    ---

    

29. # MORE EXPLANATIONS

    **Executive Verdict**

    Risky but salvageable.

    Да, понимать это нужно так: монета Bob связана не с тем, что внутри нее “лежит” секрет Bob, а с тем, что ее публичные поля согласованы только с секретом Bob. Это как замок, который не хранит ключ внутри себя, но открывается только ключом с правильной нарезкой.

    **Что Именно Связывает Монету С Bob**

    В текущей логике есть две разные вещи.

    1. Распознавание монеты.
       Bob по своей карточке и своему секрету понимает: это монета для меня, я могу ее открыть.

    2. Право траты.
       Чтобы потратить монету, мало знать секрет монеты. Нужно еще показать правильный секрет владельца.

    Это видно в output.rs и spending.rs.

    Логика там такая:

    $$
    owner\_handle_B = H(RID \parallel receiver\_secret_B)
    $$

    $$
    view\_sk_B = H2Scalar(VIEW \parallel receiver\_secret_B)
    $$

    Дальше из монеты берется ее публичный эпhemeral ключ $R_{pub}$, и при трате проверка считает:

    $$
    k_{in} = DH(view\_sk_B, R_{pub})
    $$

    Потом проверяется, что owner tag монеты совпадает с тем, что должен получиться именно для Bob:

    $$
    owner\_tag = H(TAG \parallel owner\_handle_B \parallel k_{in})
    $$

    И отдельно проверяется, что секрет монеты действительно соответствует ее asset id:

    $$
    asset\_id = H(ASSET \parallel s_{in})
    $$

    То есть spend-rule в замысле требует сразу две вещи:

    $$
    spend\ authority = (receiver\_secret,\ s_{in})
    $$

    **Почему Секрет Alice Не Подходит**

    Потому что если Alice подставит свой собственный receiver secret, получится другой owner handle и другой view key.

    Значит:

    - из того же самого $R_{pub}$ она получит другой $k_{in}$,
    - из него получится другой owner tag,
    - и проверка owner tag не пройдет.

    Проще говоря:

    - Alice может знать секрет монеты,
    - но монета “помечена” как принадлежащая тому, чей receiver secret воспроизводит правильную пару owner handle плюс view key,
    - для монеты Bob это секрет Bob, не Alice.

    **Самая Простая Аналогия**

    Монета устроена как сейф с двумя условиями.

    Первое условие:
    нужно знать номер сейфа. Это роль секрета монеты, то есть $s_{in}$.

    Второе условие:
    нужно приложить правильный отпечаток владельца. Это роль receiver secret.

    Alice знает номер сейфа, потому что сама его создала.
    Но ее отпечаток не совпадает с тем, под который сейф был выписан.
    Поэтому сейф должен не открыться.

    **Где Возникает Ваш Разрыв Логики**

    Он нормальный. Он появляется из вопроса:

    “Если Alice знает $s_{out}$, почему она не может потратить?”

    Ответ:
    потому что в этой схеме $s_{out}$ сам по себе не является полным правом собственности.

    Если бы правило было такое:

    $$
    spend\ authority = s_{in}
    $$

    тогда Alice действительно могла бы украсть.

    Но в текущем B3-замысле правило другое:

    $$
    spend\ authority = (receiver\_secret,\ s_{in})
    $$

    Именно поэтому receiver secret и важен.

    **Что Делает Stealth Address И Чего Он Не Делает**

    Stealth address:

    - помогает Alice создать output для Bob,
    - помогает Bob потом узнать “это мое”,
    - помогает Bob расшифровать содержимое.

    Но stealth address сам по себе не является доказательством права траты.

    Право траты появляется только там, где verifier требует receiver secret как часть witness.

    **Значит Ли Это, Что Все Уже Надежно**

    Вот здесь самое важное ограничение.

    На уровне intended logic ответ такой:
    да, Alice не должна мочь тратить монету Bob, потому что у нее нет receiver secret Bob.

    На уровне текущего репозитория как окончательной chain-level гарантии ответ слабее:
    это пока не доведено до конца.

    Причина в том, что публичная proof boundary еще не завершена. Локальная логика ownership есть в spending.rs, но финальная verifier boundary все еще structural или placeholder-like в witness_gate.rs.

    Именно поэтому формулировка должна быть строгой:

    - как криптографическая логика дизайна: да, Bob’s receiver secret должен отсекать Alice;
    - как уже завершенная гарантия live stack: пока нет, потому что финальный verifier не доведен.

    **Пример Совсем По-Простому**

    Пусть:

    - Alice отправляет монету Bob,
    - Alice знает секрет монеты,
    - Bob знает секрет монеты после расшифровки,
    - но только Bob знает свой receiver secret.

    Тогда:

    - Alice пытается потратить,
    - verifier спрашивает: покажи секрет монеты и секрет владельца,
    - Alice показывает секрет монеты, но секрет владельца показать не может,
    - spend должен быть отвергнут.

    Если verifier вдруг проверяет только секрет монеты, тогда Alice украдет.
    В этом и состоит текущий риск незавершенной proof boundary.

    **Ключевой Итог**

    Вы правильно понимаете главное:

    не монета хранит внутри себя receiver secret Bob.
    Правильнее сказать так:
    монета построена так, что только Bob’s receiver secret согласуется с ее owner-tag и stealth-структурой при spend-проверке.

    То есть связь монеты с Bob идет через проверяемое соответствие, а не через явное хранение секрета внутри монеты.

    **Final Decision**

    Blocked.

    1. Криптологическая логика B3 понятна и внутренне связна: Alice должна отсекаться через отсутствие receiver secret Bob.
    2. Но текущая кодовая база еще не дает полной end-to-end гарантии этого на границе принятия state transition.
    3. Поэтому правильный ответ на ваш вопрос: “почему Alice не может тратить?” — потому что должен проверяться receiver secret Bob, а не потому что Alice не знает секрет монеты.

    Если хотите, я могу следующим сообщением нарисовать это как мини-схему в формате:
    Alice знает что,
    Bob знает что,
    что проверяет validator,
    и на каком именно шаге Alice должна ломаться.

    ---

    ---

    

30. ## Critical And High Findings

    | Severity | Component                                                    | Problem                                                      | Impact                                                       | Exploit path                                                 | Fix                                                          |
    | -------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
    | S0       | `crates/z00z_crypto/src/claim/prover.rs`, `crates/z00z_crypto/src/claim/verifier.rs`, `crates/z00z_simulator/src/scenario_1/stage_3.rs` | `prove_genesis_claim` and `verify_genesis_claim` implement a deterministic placeholder digest over the public statement, not a proof tied to a secret witness. `ClaimAuthoritySig::from_statement` is likewise forgeable from public data. | Any party able to assemble a statement can generate a passing proof and authority signature without possessing secret authority material or a real witness. This is direct proof forgery and invalid authorization. | Build any desired `GenesisClaimStatement`, derive `proof_bytes(statement_hash)` and `sig_bytes(statement_hash)`, then submit through the normal claim verifier path. | Replace the placeholder claim proof and signature with a real authorization model. At minimum, require a real signing key for claim authority and an authenticated genesis-membership witness. If the design remains temporary, isolate it behind a verifier mode that cannot be enabled outside simulator tests. |
    | S0       | `crates/z00z_simulator/src/scenario_1/stage_3.rs`, `crates/z00z_wallets/src/core/tx/claim_tx.rs` | The claim statement binds `genesis_root` to `ZERO_ROOT`, and wallet verification explicitly documents it as a transitional non-authoritative root. | Claim soundness is broken because the verifier does not authenticate the claimed source against a real genesis commitment set. A forged or arbitrary source commitment can be represented as if it came from genesis. | Construct a claim package for an arbitrary source commitment and asset id, then rely on the placeholder proof and zero-root statement binding to satisfy verification. | Bind claim statements to a real authenticated genesis root or equivalent commitment root, and require verifier-side membership validation against that root before any claim is accepted. |
    | S1       | `crates/z00z_simulator/src/scenario_1/stage_6.rs`, `crates/z00z_simulator/src/scenario_1/stage_7.rs` | Checkpoint construction uses `PassProof` and `NoSpent` placeholders when building the checkpoint draft. | The checkpoint pipeline can accept state transitions without a real proof check or spent-set validation, undermining double-spend resistance and state integrity in the simulator's finalization flow. | Feed duplicate or invalid fragment inputs into the checkpoint flow and rely on the stubbed verifier path to produce a valid-looking draft and follow-on artifact chain. | Replace placeholders with the real proof verifier and real spent-index checks, or make the draft builder refuse execution unless an explicit simulator-only test mode is proven impossible to ship. |
    | S1       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs`, `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs`, `crates/z00z_simulator/src/scenario_1/stage_3_utils/post_claim.rs` | Scenario artifacts store passwords, seed phrases, and receiver secrets in plaintext, including a Markdown dump and long-lived `String` fields. | Anyone with filesystem access, CI artifact access, or accidental repository exposure can recover wallet material and fully compromise scenario wallets. | Run the scenario with debug export enabled, read the generated secrets artifact, unlock or import wallets, and spend funds or decrypt owned outputs. | Remove plaintext secret dumps from the default workflow. Use `Hidden<T>` or `SecretBytes` wrappers, zeroizing containers, and explicit one-shot encrypted export flows only. If debug export is unavoidable, gate it behind a separate test-only binary and write to a guaranteed throwaway location with loud runtime confirmation. |

    These are structural findings, not style concerns. The first two are enough on their own to block any claim that the scenario exercises a sound claim protocol.

    **Confidence:** high for all four findings. The relevant code paths are direct, short, and unambiguous.

    ## Medium And Low Findings

    | Severity | Component                                                    | Problem                                                      | Impact                                                       | Exploit path                                                 | Fix                                                          |
    | -------- | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
    | S2       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/transport.rs` | `SeqSecureRngProvider` derives seeds by XORing a fixed seed with a counter-based mixer, then feeds them into `StdRng`. | This is not a cryptographically defensible randomness construction for key material or unlinkability-sensitive flows, even if labeled simulator-only. The pattern is likely to be copied. | Predict future RNG streams from the small seed and public counter schedule, then reproduce ephemeral choices across runs. | Delete the custom RNG wrapper. Use `MockRngProvider` for deterministic tests and `CryptoRng`-backed sources for any key or stealth output generation. |
    | S2       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/actors.rs` | Actor passwords and mock seeds are fixed, human-readable, and low entropy. | The scenario normalizes weak credential habits and makes exported artifacts trivially reusable by anyone who knows the defaults. | Read default actor names, derive matching passwords from source, unlock exported wallets, and recover state. | Generate per-run test credentials, or derive them from a single test seed with a KDF and role-separated labels. |
    | S2       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/flow.rs` | `std::env::set_var` mutates global wallet network and chain settings during runtime assembly. | Concurrent scenario execution can cross-contaminate wallet configuration and silently misbind artifacts to the wrong chain or network context. | Run multiple scenario flows in the same process or runtime, interleave initialization, and observe configuration bleed-through. | Pass network and chain settings through explicit configuration objects instead of process-global environment mutation. |
    | S3       | `crates/z00z_simulator/src/scenario_1/stage_2_utils/artifacts.rs` | Sensitive values live in ordinary `String` fields without zeroization. | Secret exposure surface is larger than necessary and survives longer in heap memory. | Memory dump, panic dump, or later debug logging can reveal material that should have been erased after use. | Replace with zeroizing secret wrappers and avoid cloning or formatting secrets. |
    | S3       | `crates/z00z_simulator/src/scenario_1/Readme.md`             | The scenario is described as a happy-path baseline without a prominent disclaimer that claim proof and authority semantics are placeholders. | Readers may over-trust the scenario and propagate broken constructions into adjacent code or docs. | Developers use the simulator as a reference implementation rather than a test harness. | Add an explicit cryptography status section listing placeholder components and banned reuse surfaces. |
    | S4       | `crates/z00z_simulator/src/scenario_1/stage_7.rs`, `crates/z00z_simulator/src/scenario_1/jmt_wallet_scan.rs` | Several parts of the pipeline are otherwise disciplined: proof validation precedes ownership detection, state transitions are checkpointed, and report artifacts are emitted consistently. | This reduces operational ambiguity but does not compensate for the structural proof and root-binding failures. | None.                                                        | Preserve this sequencing while replacing placeholder crypto. |

    Positive observations worth preserving after remediation:

    - BLAKE3 domain separation for claim output leaf hashes and owner-binding hashes is explicit and distinct.
    - Nullifier derivation is chain-bound in the scenario flow instead of being a bare claim identifier.
    - Range-proof generation for claim outputs is tied to the derived blinding value rather than an obvious dummy scalar.
    - Post-transaction JMT scanning validates proof data before ownership checks, which is the right direction for avoiding semantic false positives.

    **Confidence:** high for the S2 findings, medium-high for the S3/S4 observations.

    ---

    ---

31. The following questions prevent stronger positive conclusions even after reading the full scenario module:

    1. Who owns the long-term authority key that should authorize genesis claims, and where is that trust anchor anchored in chain state or configuration?
    2. Is `scenario_1` expected to remain permanently simulator-only, or is it a migration path toward a production claim flow?
    3. What exact spent-set invariant should `stage_6` enforce, and which storage component is the intended source of truth for it?
    4. What are the maximum allowed amounts and asset classes for the stage-3 range-proof statements, and are those limits consensus-bound anywhere outside the simulator?
    5. Is there an approved artifact-handling policy for `wallet_debug_dump`, including retention, cleanup, CI publishing, and accidental commit prevention?

    **Confidence:** high that these are real blockers, because each ambiguity maps to a missing trust anchor or enforcement rule.

    ---

    ---

    

32. ## Concrete Fixes

    **Fix set A: replace placeholder claim authorization.**

    1. Introduce a real genesis claim authority keypair and bind its public key in configuration or chain state.
    2. Sign the canonical claim statement with a real signature scheme already present in `z00z_crypto`, not a statement-hash wrapper.
    3. Make the verifier reject any placeholder mode unless compiled into an isolated simulator-only test target.

    **Fix set B: authenticate genesis membership.**

    1. Replace `ZERO_ROOT` with a real genesis commitment root.
    2. Add membership witnesses or authenticated inclusion data for the source asset.
    3. Ensure the claim statement binds asset id, source commitment, chain id, scenario or ruleset version, and the authenticated genesis root.

    **Fix set C: remove checkpoint integrity placeholders.**

    1. Replace `PassProof` with the real proof validation result used by storage finalization.
    2. Replace `NoSpent` with an actual spent-set lookup against the checkpoint source of truth.
    3. Add a fail-closed path so draft creation aborts if either proof validation or spent-set validation is unavailable.

    **Fix set D: harden secret lifecycle.**

    1. Delete plaintext secret Markdown export from the default path.
    2. Use `Hidden<T>`, `SecretBytes`, or equivalent zeroizing wrappers for passwords, seed phrases, and receiver secrets.
    3. Keep any debug export behind a separate feature and separate executable with runtime confirmation, artifact cleanup, and explicit non-production labeling.

    **Fix set E: fix simulator randomness and configuration handling.**

    1. Remove `SeqSecureRngProvider` and use existing deterministic test RNG abstractions already provided by the workspace.
    2. Derive actor credentials from a scenario seed plus domain-separated labels instead of hard-coded literals.
    3. Stop mutating process-global environment variables during runtime assembly.

    **Confidence:** high that these changes address the identified issues without requiring vendor edits under `z00z_crypto/tari/`.

    

