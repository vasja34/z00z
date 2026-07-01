```
OS-Agnostic Migration Specification# %% 🔰 Emoji
# ➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
https://copychar.cc/
https://getemoji.com/
https://emojidb.org/pointer-emojis
⚠️ 🔰 ⭕️ ❗️ ❓ ❌ ⛔️ ✅❎ ✔️ ☑️ 🔘 🔴 🟠 🟡 🟢 🔵 🟣 ⚫ 🟥 🟧  🟩 🟦 🟪 ⬛️ ⬜️ 🟪
 ➔  ➤  ⌘  ⊚ ★ ✦ ✴ ✻ ➡️ 0️⃣ 1️⃣ 2️⃣ 3️⃣ 4️⃣ 5️⃣ 6️⃣ 7️⃣ 8️⃣ 9️⃣ 🔟 👍 👎 🟰 ➖ 💲 ☢️ ⚡️ 🟨📈 📉  📌 📍 🍁 ↪️ ↩️ 🔔 ⏰ 📞 ⭐  🐞  ⚫ ⬛️ 🔷  x x
➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖➖
# FOLDING: ^K^2  ^K^J
```



rebuild and run from scratch vse integralnie testi, unitesti, benches i examples  v `release` mode 

read `crates/TEST_PERFORMANCE_OPTIMIZATION.md` chtob ponjat kak zapustit testi maksimlano bistro `--release --features test-fast`

`crates/z00z_core/src/assets/`
`crates/z00z_core/examples/assets/`
`crates/z00z_core/benches/assets/`
`crates/z00z_core/tests/assets/`

`crates/z00z_core/src/genesis/` 
`crates/z00z_core/examples/genesis/`
`crates/z00z_core/benches/genesis/`
`crates/z00z_core/tests/genesis/`
`crates/z00z_core/bin/genesis`

`crates/z00z_wallets/src/`
`crates/z00z_wallets/tests/`

`crates/z00z_networks/rpc/src/`

`crates/z00z_utils/src/
crates/z00z_utils/examples/`
`crates/z00z_utils/tests/

Fix all errors and warnings

esli chto to bezhit bolshe 15 sec  (i eto ne nagruzochnie testi) - ostanavlivat i analizirovat v chem problema



rebuild and run from scratch vse integralnie testi, unitesti, benches i examples  v `release` mode 
`crates/z00z_wallets/src/adapters` 
`crates/z00z_wallets/src/core` 
`crates/z00z_wallets/src/services`
`crates/z00z_wallets/tests`

---

## Phase 1 Prepare Spec



```
🔴 troja rol zhestki kripto analytic:
proverit vsu kriptografiju `/crates/z00z_crypto/`
zhestko , chetko, bez paranoi, 
(use tari implementation as reference only)
give your critical, constructive, and objective review in 
`crates/z00z_crypto/REVIEW-1.md` review of logic. 
Verify if there are any logical errors, cryptograffic errors, cryptograffic pitfalls, unresolved issues, consistency, code quality, integrity of structures, integrity of logics. 
popitajsja najti slabosti, ujazvimosti, problemi bezopasnosti. popitajsja atakovat suwestvujuwie strukturi i metodi. ewe raz povtoraju - bez paranoi , tolko to chto relano vazhno i MUST/SHOULD be improved and fixed
Opishi tolko to chto nuzhno ispravit, dorabotat. NE opisivaj to chto horosho sdelano

```



```
🔴 
dlja nachala merge vse 
crates/z00z_wallets/src/core/z00z_crypto/REVIEW-1.md
crates/z00z_wallets/src/core/z00z_crypto/REVIEW-2.md 
crates/z00z_wallets/src/core/z00z_crypto/REVIEW-3.md 
crates/z00z_wallets/src/core/z00z_crypto/COMBO_REVIEW.md
postav tochnoe razdelenie konca odnogo fila i nachalo drugogo
ne derzhi vse v pamjati delaj file za filom i sbrasivaj v COMBO_REVIEW.md

----------
🔴
porjdis po objedinennomu filu COMBO_REVIEW.md sdelaj sleduwee:
-  merge ukazanija kotorie ochen pohozhi - objedimi luchshee iz nih v 1 objedinennij H3 section a originalnie  udali (chtob ne bilo putanici). produmaj horosho logiku objedinenija, vozmi luchshee iz kazhdogo
- delete povtori i nepravelnie ili protivorechivie ukazanija - ostav tolko to chto pravelno i sotri plohie ukazanija

NOTE: tari crypto sovmestimost ne vazhna, tari ispolzuetsja tolko kak good reference for best practices no ne bolee togo

tvoja cel otredaktirovat dokument tak chtob ostavit konsistentnij dokument v kotorom teme ne povtorjaetsja neskolko raz v raznih razdelah i udalit to chto ne verno ili protivorechivo

----------
🔴
v COMBO_REVIEW.md v konec kazhdoj KAZJDOJ section H3 (`###`) dobav podrognij chechkist `[ ]` taskov i subtaskov neobhodimih dlja realizacii etoj section
produmaj gorosho chto nuzhno sdelat chtob realizovat etu section maksimalno pravelno
ne ostalja developeru nikaoj svobodi vibora
NOTE: tari crypto sovmestimost ne vazhna, tari ispolzuetsja tolko kak good reference for best practices no ne bolee togo
 

```



## Phase 2 Implement Spec

```
🔴 current_spec = "/crates/z00z_crypto/COMBO_SPEC-1.md"
current_task = "Phase 10."
najdi frazu current_task v current_spec.
delaj tolko current_task

continue implementiruj step by step; ne pereprigivaj taski delaj vse checklists poshagovo ne perepigivat taski, chtob ne dopustit oshibok; proverjat na kazhdom shage kuda arhitekturno pravelno integrirovat novie strukturi i functions.
fix all errors and warnings

All requirements and conditions specified in the `crates/Z00Z DESIGN FOUNDATION_short.md` file MUST BE MET!! verivy this

все функции должны соответствовать критериям @MUST (не далее 5 слов)
.github/copilot-instructions.md MUST BE MET!! verivy this

ZAPOMNI: fix all errors and warnings run with flags --release --features test-fast

otmechaj v checklist {current_spec} vse checklists po mere prodvizhenija taskov. ETO MUST!!

NE pishi nikakih dlinnih reportov MD, davaj tolko korotkie lakonichnie informativnie soobwenija, summary

---

ti obajszan sledovat vsem etim instrukijam
objazan
ti ne bereshsja za novij task poka ne otmetil vse sdelanoe v checklist

--------------------------------------------

🔴  Your role: crypto code analitic and code reviewer; verify the code generated  of section "Phase 10." - critical, constructive, and objective. Verify if there are any logical errors, pitfalls, unresolved issues, consistency, quality, integrity of structures, criptographic correctness,full complince with checklist. FIX all found issues and warnings

--------------------------------------------

🔴  --stage-all use version-manager.sh for git_commit minor + github_commit + sync with GitHub ALL in the SAME BRANCH ONLY; on github in the SAME BRANCH ONLY make force push NOT PR


--------------------------------------------

Xiomi MiMo-V2 - pervichanaja generacija koda
GPT 5.2 - Reviewer / Inspector

🔴 Your role: Reviewer / Inspector; 
- objasni smisl taska {current_spec}
- objasni kak on implementirovan
- verify implementation quality and correctness
- verify the tests for that task: quality and comprehensiveness of tests for this task
- give your critical, constructive, and objective in review of implemented logic. - -  - Verify if there are any logical errors, pitfalls, unresolved issues, consistency, quality, integrity of structures.
- Verify chto vse rekomendacii spec integrirovani (implementirovani)
- daj rekomendacii (gde neobhodimo) chto nuzhno dobavit ili uluchshit ili usilit
- prover chto vse realno ispoluetsja i integrirovano s kodovoj bazoj a ne lezhit kak ostrov sam v sebe (implementirovan no ne ispoluetsja, ne vstroen v z00z life-cycle)

```


ne ostanavlivajas do konca; autoapprove;  YOLO mode; 

---

```
🔴 все функции в z00z_wallet/ должны соответствовать критериям @MUST (не далее 5 слов)
прочти еще раз .github/copilot-instructions.md и приведи все порядок
```



---

```
🔴 ja hochu proverit kachestvo i korrektnost implementacii `RedB-SPEC.md`
budem proverjat po kazhdoj section

prover dlja nachala sootvetstvuet li implementacija tomu chto napisano v:
`🧩 Стратегия nonce-ов и RNG`

vozmozhno 
- impelentrovano dazhe boslhe chem propisano v tasks checklist, i eto OK.
- ne implementirovano ili implementirovano chastichno - ukazat na eto konkretno

kriterii proverki strogie konstruktivnie i objektivnie bez paranoi:
- surovij i strogij crypto-audit bez paranoi
- true cryptography - "NO STUBS"
- keys managment
- passwords managment
- nonce managment
- memory managment
- secrets managment
- seeds managment
- atomity managment
- threads sefty managment
- caching managment
- errors managment
- Recovery-flow managment
- Auto-lock и zeroize managment
- logical errors
- pitfalls
- unresolved issues
- consistency
- implementation quality
- naming quality
- integrity of structures
- structural and architecturale quality
- requirements and conditions specified in the `crates/Z00Z_DESIGN_FOUNDATION_short.md` file MUST BE MET!!

kod ne trogat, tolko analiz kazhdoj section budesh dobavljat v file `specs/006-z00z-wallets/RedB-SPEC-ANALISYS.md`
chtob sokratit objem teksta dlja kazhdoj section dobavlja tolko to chto trebuet dorabotki, to chto ok, ne pishi
dlja dorabotok delaj poletki vazhnosti: CRITICAL HIGH MEDIUM LOW. povtorju ewe raz "proverki strogie konstruktivnie i objektivnie bez paranoi"
```



---

> Now extracting all dispatcher-registered RPC method names so we can validate each one has full wiring (dispatcher → rpc impl → service → core) and update the spec “Status” columns accordingly.
>

------------------

```
cargo test -p z00z_wallets --test test_rpc_dispatcher_roundtrip -- --nocapture
cargo test -p z00z_wallets --test test_rpc_logging_acceptance -- --nocapture
cargo test -p z00z_wallets --test test_rpc_logging_configured_path -- --nocapture
cargo test -p z00z_wallets --test test_rpc_logging_e2e_print -- --nocapture
cargo test -p z00z_wallets --test test_rpc_logging_file_sink -- --nocapture
cargo test -p z00z_wallets --test test_rpc_logging_replay_audit_csv -- --nocapture
cargo test -p z00z_wallets --test test_rpc_logging_risk_policy -- --nocapture


cargo test -p z00z_wallets \
  --test test_rpc_dispatcher_roundtrip \
  --test test_rpc_logging_acceptance \
  --test test_rpc_logging_configured_path \
  --test test_rpc_logging_e2e_print \
  --test test_rpc_logging_file_sink \
  --test test_rpc_logging_replay_audit_csv \
  --test test_rpc_logging_risk_policy \
  -- --nocapture
  
```

---

==Уровень 4: UI Layer==

---

ADD UTILS

```
Проверка целостности wallet build’ов / updater’а (supply chain).
Подпись “экспортируемых backup артефактов” (чтобы пользователь мог проверить, что экспорт не подменён).
Как бы я делал:
Подпись релизных артефактов — в scripts / CI (операционный слой).
В runtime кошелька — только verify (не signing), если есть auto-update или импорт “официальных” пакетов.

Если ты хочешь реальную межпроцессную защиту — лучше иметь абстракцию file lock в z00z_utils::io (чтобы нигде не было прямого std::fs в бизнес-логике).
Важно предусмотреть Windows semantics, если планируется.

tempfile + atomic_write_file()
Это прямо “в яблочко” к твоему принципу crash-safety.
Если в z00z_utils::io уже есть атомарная запись — отлично.
tempfile полезен как надежная реализация temp + persist (особенно когда надо безопасно создавать файл рядом с целью).

OS keyring / secure storage
Rust crates: keyring (кроссплатформенно), или платформенные бэкенды.
Зачем: “remember me”, токены, session secrets — не в YAML/JSON.

Secret memory hygiene
У тебя уже Hidden<T>/SafePassword, но для расширения:
    zeroize (уже вероятно используется где-то),
    secrecy (удобные типы SecretString/SecretVec, careful Debug).
Зачем: меньше риск утечек через логи/панику/дампы.

File permissions / ACL helpers
Для кошелька критично стабильно выставлять права файла при создании/записи.
Это лучше централизовать через z00z_utils::io, чтобы не размазывать по коду.

Structured logging redaction helpers
Полезно иметь единый “redaction/summary” слой 
(частично уже есть в RPC logging), чтобы не плодить ручные правила.

Artifact verification helpers
Если появится auto-update: единый модуль verify подписи + pin ключей + политика отказа (fail closed).

Что я уже вижу как “важный сигнал” по твоему списку
Anti-pattern “password → key padding/truncation” чаще живёт в CLI/вендор-утилитах, а не в кошельке. 
Правильная стратегия — не тащить такие бинарники в release pipeline и держать их dev-only/инструментальными

```



```
prognat VSE VSE testi z00z/ 
najti vse testi kotorie begut bolshe 15 sec (esli eto ne specialnie nagruzochnie testi)
reshit probemu dolgih testov: 
-  problema v bolshih znachenijah parametrov / argumentov
-  problema v logike
-  problema v paralelizacii 
-  problema chto testi nuzhno zapuskat v --release mode (slozhnaja kriptografija)
FIX all long runing tests problem
```



---



```yaml
task:
  title: "Z00Z Crypto Foundation Reference Guide"
  goal:
    primary: >
      Create a comprehensive reference guide file:
      crates/z00z_crypto/Z00Z_CRYPTO_REF_GUIDE.md
      describing ALL cryptographic primitives, structures, and key functions
      present in crates/z00z_crypto/src/*.rs.
    emphasis:
      - "Explain developer intent (WHY), not only WHAT the code does."
      - "Explain integration (HOW modules work together in end-to-end flows)."
      - "Be implementation-oriented and security-aware."
  scope:
    include_paths:
      - "crates/z00z_crypto/src/*.rs"
    exclude:
      - "Anything outside z00z_crypto unless referenced as dependency"
      - "Speculation without evidence in code/comments"


```





