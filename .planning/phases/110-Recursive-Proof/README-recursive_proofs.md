Если твой выбор это `Plonky3-recursion`, то от тебя требуется не “подключить библиотеку”, а зафиксировать новый proof layer для `z00z`.

Сейчас у тебя активный путь это `Bulletproofs+` для range proofs и wire-level accepted verifier в prover.rs и spend_verification.rs. `Plonky3-recursion` не заменяет это одним патчем. Оно потребует от тебя выбрать, что именно ты доказываешь рекурсивно: скорее всего не приватный spend per tx, а state-transition / checkpoint correctness поверх текущего transaction layer.

**Что это от тебя потребует**

1. Зафиксировать точный statement.
   Нужно письменно определить одну проверяемую функцию перехода:
   `root_old + block_delta/nullifier_delta -> root_new`.
   Без этого recursion stack не к чему прикручивать.

2. Выбрать granularity recursion.
   Ты должен решить:
   - proof на каждый блок,
   - proof на epoch,
   - или aggregation нескольких блоков в один recursive step.
   Это влияет на proof size, latency и сложность интеграции.

3. Принять, что первый этап будет spike, не production rollout.
   По официальному README `Plonky3-recursion` ещё under active development, not audited, not recommended for production use.
   Значит, твой первый commit-level goal должен быть:
   “доказать один recursive state transition end-to-end”, а не “перевести всю сеть”.

4. Согласиться на отдельный crate / subsystem.
   Практически это почти наверняка новый модуль уровня вроде:
   - `z00z_recursive_proofs`
   - или отдельный proof adapter crate
   с собственными types для:
   - public inputs,
   - transition witness,
   - recursion proof object,
   - verifier API.

5. Определить public inputs как канон.
   Тебе надо решить, что будет публично входить в proof:
   - `prev_root`
   - `new_root`
   - epoch/block number
   - chain/domain separator
   - optional digest of block contents
   - optional nullifier-set root
   Это должен быть стабильный контракт, как у тебя уже сделано для accepted public spend contract.

6. Выбрать hash/field stack.
   `Plonky3-recursion` тащит STARK/FRI circuit stack и явно завязан на свою конфигурацию, включая FRI params и circuit builder operations.
   То есть от тебя нужен выбор:
   - поле,
   - hash family,
   - commitment path,
   - recursion parameters.
   Это уже crypto-architecture decision, не просто dependency management.

7. Разделить “что остаётся как есть” и “что уходит в recursion”.
   Самый реалистичный путь для `z00z`:
   - текущий tx/privacy/range-proof слой оставить,
   - recursion добавить на уровень checkpoint/state validity.
   Это сильно проще, чем пытаться немедленно переписать все spends в новый proof backend.

8. Подготовить proving infrastructure.
   Тебе понадобится:
   - prover runtime,
   - benchmark harness,
   - proof serialization,
   - checkpoint artifact integration,
   - verifier entrypoint.
   Сейчас в фазовых заметках recursive checkpoint pipeline ещё roadmap-only, а не live implementation, см. 20-Recursive checkpoint proof.md.

9. Принять operational cost.
   `Plonky3-recursion` это не “дешёвая прозрачность”.
   Ты покупаешь:
   - большую arithmetization complexity,
   - более тяжёлую prover integration surface,
   - incomplete docs,
   - необходимость internal validation before trust.

10. Принять честную rollout strategy.
      Для тебя правильная последовательность выглядит так:
   - Phase A: one-block state-transition proof
   - Phase B: recursive chaining over 2-5 steps
   - Phase C: checkpoint artifact embedding
   - Phase D: benchmark and failure model
   - Phase E: only then discuss production gating

**Коротко: что требуется лично от тебя как от архитектора**

- выбрать exact proof statement;
- выбрать block vs epoch recursion;
- согласиться на spike-first rollout;
- не пытаться сразу заменить текущий spend verifier;
- ==принять, что `Plonky3-recursion` сейчас это R&D-ставка, а не готовый infra commodity.==

**Мой практический совет**

Если выбор уже сделан, твой следующий правильный шаг не “интегрировать Plonky3-recursion везде”, а утвердить очень узкий target:

1. Доказать один `checkpoint transition`.
2. Публичные входы: `prev_root`, `new_root`, `epoch_id`, `delta_commitment`.
3. Без замены текущего `Bulletproofs+` spend path.
4. С recursion chain хотя бы на 3-5 шагов.
5. С измерением proof size, prover time и verifier time.

Это и будет минимальный честный входной билет в твой выбор.

Если хочешь, я следующим сообщением разложу это уже в формате:
1. `что надо решить тебе сегодня`
2. `что надо сделать мне в коде`
3. `какой первый GSD phase открыть под Plonky3-recursion`.
