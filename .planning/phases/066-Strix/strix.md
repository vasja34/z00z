> https://github.com/usestrix/strix 
>
> https://github.com/0x4m4/hexstrike-ai
> Я хочу вообще выбросить MCP. То есть я хочу сделать всё на уровне локальных skills. Дай мне полную программу того, как это сделать на базе двух этих библиотек. То есть нужно завязать skills, может быть, сделать некую иерархию. Один skill вызывает другой. То есть либо это какие-то tools, которые могут бежать параллельно, в смысле. То есть посмотри эти репозитории и дай мне конкретную раскладку. Что взять, как взять, как это всё переупаковать в локальный penetration test, который я смогу запускать для своего проекта. Я вообще не хочу связываться ни с MCP серверами, ни с API keys.



Да. **MCP можно полностью выбросить.**
Правильная архитектура для тебя:

```text
Codex subscription = мозг
Codex local SKILL.md = методология / workflow / правила
local scripts = запуск tools
local binaries = semgrep, trivy, gitleaks, cargo-audit, nmap, nuclei, ffuf...
Strix = источник playbooks
HexStrike = источник списка tools и CLI-паттернов
```

То есть **не запускать Strix**, **не запускать HexStrike server**, **не поднимать MCP**, **не использовать API keys**.

------

# 1. Главный вывод

**Strix надо разобрать на skills.
HexStrike надо разобрать на список локальных tools.**

У Strix уже есть структура skills: `/vulnerabilities`, `/frameworks`, `/technologies`, `/protocols`, `/tooling`, `/cloud`, `/reconnaissance`, `/custom`, плюс notable source-aware skills `source_aware_whitebox` и `source_aware_sast`. Это почти готовая база для Codex skills. ([GitHub](https://github.com/usestrix/strix/tree/main/strix/skills))

HexStrike, наоборот, сделан как MCP framework: AI agent → MCP → HexStrike server → 150+ security tools. В нём полезен **tool inventory**, но не сам MCP/server layer. README прямо показывает core tools: `nmap`, `nuclei`, `gobuster`, `ffuf`, `httpx`, `katana`, `trivy`, cloud/container tools и т.д. ([GitHub](https://github.com/0x4m4/hexstrike-ai))

Codex skills официально устроены как директория с `SKILL.md` плюс optional `scripts/`, `references/`, `assets/`. Поэтому твой вариант должен быть именно таким: **skills + scripts**, без MCP. ([OpenAI Developers](https://developers.openai.com/codex/skills))

------

# 2. Важное ограничение: skill не “вызывает” другой skill как функция

В Codex нет нормальной модели типа:

```text
skill_A.call(skill_B)
```

Вместо этого делается так:

```text
orchestrator skill
  → говорит Codex: используй/прочитай child skill X
  → запускает scripts
  → просит subagents параллельно проверить разные зоны
```

Codex умеет активировать skills явно или неявно, по `description`, а также умеет запускать subagents, но subagents запускаются только когда ты явно просишь это сделать. ([OpenAI Developers](https://developers.openai.com/codex/skills))

Поэтому правильный дизайн: **один главный coordinator-skill + несколько специализированных child-skills + один параллельный runner-script**.

------

# 3. Рекомендуемая структура в твоём проекте

Для Z00Z/Rust/blockchain/privacy проекта я бы сделал так:

```text
z00z/
  AGENTS.md

  .agents/
    skills/
      z00z-local-pentest/
        SKILL.md
        references/
          scope-template.yaml
          report-template.md
          severity-rubric.md

      z00z-source-aware-sast/
        SKILL.md
        scripts/
          run_source_sast.sh
          summarize_sast.py
        references/
          semgrep.md
          ast-grep.md
          source-map.md
        rules/
          semgrep/
            z00z-rust-security.yml

      z00z-rust-security/
        SKILL.md
        scripts/
          run_rust_security.sh
          summarize_rust_security.py
        references/
          rust-unsafe-checklist.md
          cargo-supply-chain.md

      z00z-secrets-supply-chain/
        SKILL.md
        scripts/
          run_secrets_supply_chain.sh

      z00z-local-dast/
        SKILL.md
        scripts/
          run_local_dast.sh
        references/
          nmap.md
          nuclei.md
          ffuf.md

      z00z-crypto-protocol-audit/
        SKILL.md
        references/
          z00z-privacy-boundary.md
          z00z-double-spend-checklist.md
          z00z-checkpoint-da-checklist.md
          z00z-stealth-output-checklist.md

      z00z-security-report/
        SKILL.md
        scripts/
          build_report.py
        references/
          report-template.md

  scripts/
    security/
      install_local_security_tools.sh
      run_full_local_pentest.sh
      run_parallel_static.sh
      run_local_targets_check.sh

  .security/
    scope.yaml
    allowed-targets.txt
    excluded-paths.txt

  .security-artifacts/
    2026-06-30_...
```

Почему repo-local `.agents/skills`, а не global? Потому что Codex docs говорят, что skills можно хранить в repo `.agents/skills` или в `$HOME/.agents/skills`; repo-local лучше для Z00Z-специфичных правил. ([OpenAI Developers](https://developers.openai.com/codex/skills))

------

# 4. Что взять из Strix

## Брать напрямую

| Strix source                                         | Что взять                                                    | Куда положить                                            |
| ---------------------------------------------------- | ------------------------------------------------------------ | -------------------------------------------------------- |
| `strix/skills/coordination/source_aware_whitebox.md` | Главная white-box методология: source map → static triage → dynamic validation → evidence-only reporting | `z00z-local-pentest/references/source-aware-whitebox.md` |
| `strix/skills/custom/source_aware_sast.md`           | Semgrep + ast-grep + gitleaks + trufflehog + trivy fs workflow | `z00z-source-aware-sast/SKILL.md` + scripts              |
| `strix/skills/tooling/semgrep.md`                    | Semgrep flags, `--metrics=off`, JSON/SARIF output, scoped rules | `z00z-source-aware-sast/references/semgrep.md`           |
| `strix/skills/tooling/nmap.md`                       | bounded nmap scanning, two-pass scan, timeouts               | `z00z-local-dast/references/nmap.md`                     |
| `strix/skills/tooling/nuclei.md`                     | bounded nuclei scan, `-ni`, rate limits, severity filters    | `z00z-local-dast/references/nuclei.md`                   |
| vulnerability/framework/protocol skills              | Выбирать только релевантные: auth, access control, race, API, GraphQL/WebSocket, injection, SSRF | `references/` внутри child skills                        |

Strix `source_aware_sast` уже содержит почти нужный baseline: `semgrep`, генерация AST target list, `sg`, `gitleaks`, `trufflehog`, `trivy fs --offline-scan`, плюс правило “scanner output is not final truth”. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/strix/skills/custom/source_aware_sast.md))

Strix `source_aware_whitebox` правильно формулирует главный принцип: static findings — это гипотезы, а vulnerability report должен быть evidence-driven. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/strix/skills/coordination/source_aware_whitebox.md))

## Не брать из Strix

Не брать:

```text
strix runtime
strix agents graph
strix Docker orchestration
strix CLI
openai-agents / litellm layer
LLM provider config
```

Причина: Strix как приложение завязан на `openai-agents[litellm]`, Docker, Caido SDK и свой runtime. Это видно в `pyproject.toml`. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/pyproject.toml))

То есть Strix — **не запускаем**.
Из него делаем **source library для skills**.

------

# 5. Что взять из HexStrike

## Брать

Из HexStrike бери только:

```text
tool inventory
CLI categories
safe defaults idea
timeouts
structured output idea
process/artifact discipline
```

Полезные tools из HexStrike для твоего локального варианта:

```text
SAST / repo:
semgrep
trivy fs
gitleaks
trufflehog
ast-grep / sg
tree-sitter

Rust:
cargo audit
cargo deny
cargo geiger
cargo clippy
cargo test
cargo nextest
cargo miri selectively
cargo fuzz selectively

Local DAST:
nmap
nuclei
httpx
katana
ffuf
gobuster
nikto cautiously

IaC / containers:
trivy config
checkov
terrascan
docker-bench-security, если реально используешь Docker
```

HexStrike README показывает эти категории: network/recon, web app security, cloud/container, binary/reverse, OSINT и т.д. ([GitHub](https://github.com/0x4m4/hexstrike-ai))

## Не брать

Не брать в auto-skill:

```text
MCP server
HexStrike Flask/API server
execute_command wrapper
file create/modify/delete wrappers
payload generator
metasploit
hydra
john/hashcat
pacu exploitation
shodan/censys/osint API tools
anything requiring external API keys
wide internet recon
```

В самом HexStrike wrapper есть функции для arbitrary command execution, file create/modify/delete, payload generation, Metasploit, Hydra, John и т.д. Это не нужно и опасно для твоей цели “локальный controlled pentest”. ([GitHub](https://raw.githubusercontent.com/0x4m4/hexstrike-ai/master/hexstrike_mcp.py))

------

# 6. Минимальный набор skills

Не делай 50 skills. Codex docs предупреждают: список skills занимает limited context budget; если skills слишком много, Codex может сокращать descriptions или не показать часть списка. ([OpenAI Developers](https://developers.openai.com/codex/skills))

Я бы сделал **7 главных skills**.

## 1. `z00z-local-pentest`

Главный orchestrator.

Задача: принять scope, запустить SAST, Rust checks, secrets, local DAST, crypto/protocol audit, потом собрать report.

```md
---
name: z00z-local-pentest
description: Run an authorized local-only security assessment for the current owned repository using repo-local scripts, source-aware SAST, Rust checks, secrets/supply-chain checks, local DAST, and evidence-based reporting. No MCP, no API keys, no public target scanning.
---

Use this skill only for a repository, local service, devnet, testnet, or staging target owned by the user or explicitly authorized.

Hard boundaries:
- Do not use MCP.
- Do not use external LLM API keys.
- Do not run Strix CLI.
- Do not run HexStrike server.
- Do not scan public targets unless they are explicitly listed in .security/scope.yaml.
- Do not run password brute force, credential attacks, persistence, malware, destructive exploitation, or data exfiltration.
- Prefer local static analysis before dynamic probing.
- Treat scanner findings as hypotheses until validated by code trace, test, reproduction, or controlled local evidence.

Workflow:
1. Read AGENTS.md and .security/scope.yaml.
2. Create .security-artifacts/<timestamp>/.
3. Build a source map.
4. Run z00z-source-aware-sast.
5. Run z00z-rust-security.
6. Run z00z-secrets-supply-chain.
7. If .security/scope.yaml contains local_urls or localhost ports, run z00z-local-dast.
8. Run z00z-crypto-protocol-audit for Z00Z-specific logic.
9. Deduplicate findings.
10. Produce final report using z00z-security-report.

Parallelization:
- For static scans, prefer scripts/security/run_parallel_static.sh.
- For conceptual audits, spawn subagents only when the user explicitly asks for parallel agents.
- Keep each subagent bounded to one domain: Rust memory/unsafe, crypto/privacy, storage/checkpoints, RPC/API, wallet/keys, dependencies/secrets.

Output:
- Executive summary.
- Findings table.
- Evidence per finding.
- Affected files/modules.
- Reproduction or validation method.
- Severity and confidence.
- Fix plan.
- Tests to add.
- False positives / uncertain items.
```

## 2. `z00z-source-aware-sast`

Это адаптация Strix `source_aware_sast`.

```md
---
name: z00z-source-aware-sast
description: Run local source-aware static security triage using Semgrep, ast-grep, tree-sitter, gitleaks, trufflehog, and trivy fs with outputs stored under .security-artifacts. Use for source-heavy code security review.
---

Goal:
Build static hypotheses from source code, not final vulnerability claims.

Required tools:
- semgrep
- sg / ast-grep
- tree-sitter where available
- gitleaks
- trufflehog
- trivy

Rules:
- Always write machine-readable outputs into .security-artifacts/<timestamp>/sast/.
- Always use --metrics=off for Semgrep.
- Prefer local vendored Semgrep rules when offline/reproducibility matters.
- Run secrets and supply-chain checks separately from code pattern checks.
- Do not report scanner output as a vulnerability until it is traced and validated.
- For Z00Z, prioritize wallet, key handling, proof verification, checkpoint/delta logic, RPC handlers, aggregator boundaries, storage, and serialization/deserialization.

Default command:
Run scripts/security/run_parallel_static.sh or .agents/skills/z00z-source-aware-sast/scripts/run_source_sast.sh.
```

## 3. `z00z-rust-security`

Rust-specific.

```md
---
name: z00z-rust-security
description: Run local Rust security checks for the current repository: cargo audit, cargo deny, cargo geiger, clippy, tests, unsafe review, panic/unwrap review, serialization boundary review, and crypto-sensitive invariants.
---

Focus areas:
- unsafe blocks
- panic/unwrap/expect in consensus, wallet, crypto, storage, RPC
- dependency advisories
- duplicate crates and risky features
- serde/bincode/postcard/deserialization boundaries
- filesystem writes
- key material lifetime
- zeroization
- randomness
- timing-sensitive crypto
- proof verification paths
- checkpoint/state transition invariants

Commands:
- cargo audit
- cargo deny check
- cargo geiger
- cargo clippy --workspace --all-targets --all-features
- cargo test --workspace
- cargo nextest run, if installed
- cargo miri test selectively for critical crates
```

## 4. `z00z-secrets-supply-chain`

```md
---
name: z00z-secrets-supply-chain
description: Run local secret scanning and supply-chain checks using gitleaks, trufflehog filesystem, trivy fs/config, cargo audit, and cargo deny. No external APIs.
---

Rules:
- Do not upload code or secrets anywhere.
- Do not use SaaS scanners.
- Store outputs under .security-artifacts/<timestamp>/secrets-supply-chain/.
- Redact secrets in final report.
- Treat test keys and fixtures separately from real secrets.
```

## 5. `z00z-local-dast`

```md
---
name: z00z-local-dast
description: Run bounded local-only DAST against localhost/devnet/staging targets listed in .security/scope.yaml using nmap, nuclei, httpx, katana, ffuf, and gobuster. No public scanning by default.
---

Hard scope:
- Allowed targets must come from .security/scope.yaml.
- Default allowed host: 127.0.0.1 only.
- Do not use -p- full port scans unless explicitly requested.
- Do not run brute-force credential attacks.
- Do not use OAST/interactsh unless explicitly authorized.
- Always rate-limit dynamic scans.

Default sequence:
1. Confirm target is local or explicitly scoped.
2. Run small nmap discovery.
3. Run service enrichment only on discovered ports.
4. Run httpx/katana only for scoped local URLs.
5. Run nuclei with severity/tag/template bounds and -ni.
6. Run ffuf/gobuster only with small wordlists and rate limits.
7. Save all outputs under .security-artifacts/<timestamp>/dast/.
```

## 6. `z00z-crypto-protocol-audit`

Это Z00Z-specific. Именно это даст value, которого нет в Strix/HexStrike.

```md
---
name: z00z-crypto-protocol-audit
description: Audit Z00Z-specific blockchain, privacy, wallet, checkpoint, stealth-output, rights/assets/voucher, and DA/reconciliation logic. Use for protocol-level security, not generic web pentest.
---

Focus:
- double-spend paths
- checkpoint delta validation
- created_delta/spent_delta consistency
- no permanent nullifier table assumptions
- wallet coin file integrity
- stealth output ownership tag
- ECDH receiver scan logic
- proof verification boundaries
- aggregation and shard failover
- storage replay/corruption
- offline transfer reconciliation
- voucher expiry/fallback
- assets/rights/policies as terminal HJMT leaves
- RPC trust boundaries
- serialization/deserialization of proofs, deltas, commitments, wallet files

Method:
1. Identify invariants.
2. Map code enforcing each invariant.
3. Search bypass paths.
4. Write adversarial scenarios.
5. Create or propose regression tests.
6. Report only with file/function evidence.
```

## 7. `z00z-security-report`

```md
---
name: z00z-security-report
description: Convert local security artifacts into a deduplicated evidence-based report with severity, confidence, affected code, reproduction, fix plan, and tests.
---

Report format:
- Scope
- Tools run
- Findings summary
- Critical/high findings first
- Evidence
- Reproduction/validation
- Affected files
- Root cause
- Fix recommendation
- Regression tests
- False positives
- Open questions
```

------

# 7. Local scope file

Создай:

```yaml
# .security/scope.yaml

project: z00z
mode: local-only

allowed_paths:
  - "."
  - "crates/"
  - "src_rs/"
  - "src_py/"
  - "tests/"
  - "benches/"

excluded_paths:
  - ".git/"
  - "target/"
  - "node_modules/"
  - ".security-artifacts/"
  - "vendor/"
  - "tmp/"

allowed_hosts:
  - "127.0.0.1"
  - "localhost"

allowed_urls:
  - "http://127.0.0.1:3000"
  - "http://127.0.0.1:8080"
  - "http://127.0.0.1:9944"

forbidden:
  - "public internet scanning"
  - "credential brute force"
  - "password cracking"
  - "metasploit"
  - "hydra"
  - "john"
  - "hashcat"
  - "pacu"
  - "data exfiltration"
  - "destructive tests"

evidence_required: true
report_static_findings_as_hypotheses: true
```

------

# 8. Главный runner без MCP

`scripts/security/run_full_local_pentest.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

TS="$(date +%Y-%m-%d_%H-%M-%S)"
ART="$ROOT/.security-artifacts/$TS"
mkdir -p "$ART"/{sast,rust,secrets,dast,report,logs}

echo "$ART" > "$ROOT/.security-artifacts/LATEST"

echo "[z00z-pentest] artifacts: $ART"

echo "[1/5] source-aware static scan"
bash "$ROOT/.agents/skills/z00z-source-aware-sast/scripts/run_source_sast.sh" "$ART/sast" \
  > "$ART/logs/source_sast.log" 2>&1 || true

echo "[2/5] Rust security scan"
bash "$ROOT/.agents/skills/z00z-rust-security/scripts/run_rust_security.sh" "$ART/rust" \
  > "$ART/logs/rust_security.log" 2>&1 || true

echo "[3/5] secrets / supply-chain scan"
bash "$ROOT/.agents/skills/z00z-secrets-supply-chain/scripts/run_secrets_supply_chain.sh" "$ART/secrets" \
  > "$ART/logs/secrets_supply_chain.log" 2>&1 || true

echo "[4/5] local DAST, only if scope has local targets"
bash "$ROOT/.agents/skills/z00z-local-dast/scripts/run_local_dast.sh" "$ART/dast" \
  > "$ART/logs/local_dast.log" 2>&1 || true

echo "[5/5] build preliminary report"
python3 "$ROOT/.agents/skills/z00z-security-report/scripts/build_report.py" "$ART" \
  > "$ART/report/security-report.md" 2> "$ART/logs/report.log" || true

echo "[done] $ART/report/security-report.md"
```

------

# 9. Параллельный static runner

`scripts/security/run_parallel_static.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ART="${1:-.security-artifacts/manual-static}"
mkdir -p "$ART"/{semgrep,ast,secrets,trivy,rust,logs}

run_bg() {
  local name="$1"
  shift
  echo "[parallel] start $name"
  (
    set +e
    "$@"
    echo $? > "$ART/logs/$name.exit"
  ) > "$ART/logs/$name.out" 2> "$ART/logs/$name.err" &
}

run_bg semgrep bash .agents/skills/z00z-source-aware-sast/scripts/run_semgrep.sh "$ART/semgrep"
run_bg astgrep bash .agents/skills/z00z-source-aware-sast/scripts/run_astgrep.sh "$ART/ast"
run_bg gitleaks bash .agents/skills/z00z-secrets-supply-chain/scripts/run_gitleaks.sh "$ART/secrets"
run_bg trufflehog bash .agents/skills/z00z-secrets-supply-chain/scripts/run_trufflehog.sh "$ART/secrets"
run_bg trivy bash .agents/skills/z00z-secrets-supply-chain/scripts/run_trivy_fs.sh "$ART/trivy"
run_bg rust bash .agents/skills/z00z-rust-security/scripts/run_rust_security.sh "$ART/rust"

wait

echo "[parallel] complete"
find "$ART/logs" -name "*.exit" -maxdepth 1 -print -exec cat {} \;
```

Это даёт реальную параллельность без MCP и без subagents.

------

# 10. Source-aware SAST script

`.agents/skills/z00z-source-aware-sast/scripts/run_source_sast.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ART="${1:-.security-artifacts/source-aware-sast}"
mkdir -p "$ART"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

EXCLUDES=(
  --exclude target
  --exclude .git
  --exclude .security-artifacts
  --exclude node_modules
)

echo "[sast] semgrep"
if command -v semgrep >/dev/null 2>&1; then
  if [ -d ".agents/skills/z00z-source-aware-sast/rules/semgrep" ]; then
    semgrep scan \
      --config ".agents/skills/z00z-source-aware-sast/rules/semgrep" \
      --metrics=off \
      --json \
      --output "$ART/semgrep-local.json" \
      --quiet \
      --jobs 4 \
      --timeout 20 \
      "${EXCLUDES[@]}" \
      . || true
  fi

  semgrep scan \
    --config p/default \
    --config p/secrets \
    --metrics=off \
    --json \
    --output "$ART/semgrep-default.json" \
    --quiet \
    --jobs 4 \
    --timeout 20 \
    "${EXCLUDES[@]}" \
    . || true
else
  echo "semgrep not installed" > "$ART/semgrep.missing"
fi

echo "[sast] derive ast-grep targets"
python3 - <<'PY'
import json
from pathlib import Path

art = Path(".security-artifacts/LATEST")
if art.exists():
    latest = Path(art.read_text().strip())
else:
    latest = Path(".security-artifacts/manual")

sast = latest / "sast"
sast.mkdir(parents=True, exist_ok=True)

semgrep_json = Path("'"$ART"'") / "semgrep-default.json"
targets_file = Path("'"$ART"'") / "sg-targets.txt"

try:
    data = json.loads(semgrep_json.read_text(encoding="utf-8"))
except Exception:
    targets_file.write_text("", encoding="utf-8")
    raise SystemExit(0)

scanned = data.get("paths", {}).get("scanned") or []
if not scanned:
    scanned = sorted({
        r.get("path")
        for r in data.get("results", [])
        if isinstance(r, dict) and isinstance(r.get("path"), str)
    })

bounded = [p for p in scanned if p][:4000]
targets_file.write_text("".join(f"{p}\n" for p in bounded), encoding="utf-8")
print(f"sg-targets: {len(bounded)}")
PY

echo "[sast] ast-grep"
if command -v sg >/dev/null 2>&1; then
  xargs -r -n 200 sg run --pattern '$F($$$ARGS)' --json=stream \
    < "$ART/sg-targets.txt" \
    > "$ART/ast-grep.json" 2> "$ART/ast-grep.log" || true
else
  echo "sg not installed" > "$ART/ast-grep.missing"
fi

echo "[sast] tree-sitter quick parse"
if command -v tree-sitter >/dev/null 2>&1; then
  find . \
    -path './target' -prune -o \
    -path './.git' -prune -o \
    -name '*.rs' -print \
    | head -500 \
    | xargs -r tree-sitter parse -q \
    > "$ART/tree-sitter-rust.log" 2>&1 || true
else
  echo "tree-sitter not installed" > "$ART/tree-sitter.missing"
fi

echo "[sast] done"
```

Это практически прямая адаптация Strix SAST workflow, только под repo-local artifacts и без `/workspace/.strix-source-aware`. Strix baseline использует именно Semgrep, AST targets, `sg`, `gitleaks`, `trufflehog`, `trivy fs`. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/strix/skills/custom/source_aware_sast.md))

------

# 11. Rust security script

`.agents/skills/z00z-rust-security/scripts/run_rust_security.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ART="${1:-.security-artifacts/rust-security}"
mkdir -p "$ART"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

run() {
  local name="$1"
  shift
  echo "[rust] $name"
  if "$@" > "$ART/$name.out" 2> "$ART/$name.err"; then
    echo 0 > "$ART/$name.exit"
  else
    echo $? > "$ART/$name.exit"
  fi
}

if [ -f Cargo.toml ]; then
  command -v cargo >/dev/null 2>&1 || { echo "cargo missing" > "$ART/cargo.missing"; exit 0; }

  if cargo audit --version >/dev/null 2>&1; then
    run cargo-audit cargo audit --json
  else
    echo "cargo-audit missing" > "$ART/cargo-audit.missing"
  fi

  if cargo deny --version >/dev/null 2>&1; then
    run cargo-deny cargo deny check --format json
  else
    echo "cargo-deny missing" > "$ART/cargo-deny.missing"
  fi

  if cargo geiger --version >/dev/null 2>&1; then
    run cargo-geiger cargo geiger --all-features --output-format GitHubMarkdown
  else
    echo "cargo-geiger missing" > "$ART/cargo-geiger.missing"
  fi

  run cargo-clippy cargo clippy --workspace --all-targets --all-features --message-format=json
  run cargo-test cargo test --workspace --all-features

  if cargo nextest --version >/dev/null 2>&1; then
    run cargo-nextest cargo nextest run --workspace --all-features
  fi
else
  echo "No Cargo.toml found" > "$ART/no-rust-project"
fi
```

------

# 12. Secrets / supply-chain script

`.agents/skills/z00z-secrets-supply-chain/scripts/run_secrets_supply_chain.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ART="${1:-.security-artifacts/secrets-supply-chain}"
mkdir -p "$ART"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

echo "[secrets] gitleaks"
if command -v gitleaks >/dev/null 2>&1; then
  gitleaks detect \
    --source . \
    --report-format json \
    --report-path "$ART/gitleaks.json" \
    --no-banner || true
else
  echo "gitleaks missing" > "$ART/gitleaks.missing"
fi

echo "[secrets] trufflehog filesystem"
if command -v trufflehog >/dev/null 2>&1; then
  trufflehog filesystem \
    --no-update \
    --json \
    --no-verification \
    . > "$ART/trufflehog.jsonl" 2> "$ART/trufflehog.err" || true
else
  echo "trufflehog missing" > "$ART/trufflehog.missing"
fi

echo "[supply-chain] trivy fs"
if command -v trivy >/dev/null 2>&1; then
  trivy fs \
    --scanners vuln,misconfig \
    --timeout 30m \
    --offline-scan \
    --format json \
    --output "$ART/trivy-fs.json" \
    . || true
else
  echo "trivy missing" > "$ART/trivy.missing"
fi
```

------

# 13. Local DAST script

`.agents/skills/z00z-local-dast/scripts/run_local_dast.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

ART="${1:-.security-artifacts/local-dast}"
mkdir -p "$ART"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

SCOPE=".security/scope.yaml"

if [ ! -f "$SCOPE" ]; then
  echo "No .security/scope.yaml; skipping DAST" > "$ART/skipped"
  exit 0
fi

TARGETS="$ART/targets.txt"
grep -Eo 'https?://(127\.0\.0\.1|localhost):[0-9]+' "$SCOPE" | sort -u > "$TARGETS" || true

if [ ! -s "$TARGETS" ]; then
  echo "No local allowed_urls found; skipping DAST" > "$ART/skipped"
  exit 0
fi

echo "[dast] local targets:"
cat "$TARGETS"

echo "[dast] nmap localhost quick"
if command -v nmap >/dev/null 2>&1; then
  nmap -n -Pn \
    -p 22,80,443,3000,5000,8000,8080,8443,9944 \
    --open \
    -T3 \
    --max-retries 1 \
    --host-timeout 90s \
    -oA "$ART/nmap-local-quick" \
    127.0.0.1 || true
else
  echo "nmap missing" > "$ART/nmap.missing"
fi

echo "[dast] nuclei local bounded"
if command -v nuclei >/dev/null 2>&1; then
  nuclei \
    -l "$TARGETS" \
    -as \
    -s critical,high,medium \
    -ni \
    -rl 20 \
    -c 10 \
    -bs 10 \
    -timeout 10 \
    -retries 1 \
    -silent \
    -j \
    -o "$ART/nuclei-local.jsonl" || true
else
  echo "nuclei missing" > "$ART/nuclei.missing"
fi

echo "[dast] ffuf small local discovery"
if command -v ffuf >/dev/null 2>&1 && [ -f /usr/share/wordlists/dirb/common.txt ]; then
  while read -r url; do
    safe_name="$(echo "$url" | sed 's#[/:]#_#g')"
    ffuf \
      -u "$url/FUZZ" \
      -w /usr/share/wordlists/dirb/common.txt \
      -mc 200,204,301,302,307,401,403 \
      -rate 20 \
      -timeout 10 \
      -of json \
      -o "$ART/ffuf-$safe_name.json" || true
  done < "$TARGETS"
else
  echo "ffuf or wordlist missing" > "$ART/ffuf.missing"
fi
```

Strix Nmap playbook recommends bounded scans, explicit target scope, timeouts, and two-pass scanning. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/strix/skills/tooling/nmap.md))
Strix Nuclei playbook recommends scoped template selection, explicit rate/concurrency limits, JSONL output, and `-ni` where OAST/interactsh is not expected. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/strix/skills/tooling/nuclei.md))

------

# 14. AGENTS.md для проекта

В repo root:

```md
# AGENTS.md

## Security workflow rules

This repository uses local Codex skills for authorized local security assessment.

Hard rules:
- Do not use MCP.
- Do not run Strix CLI.
- Do not run HexStrike server.
- Do not use external LLM API keys.
- Do not scan public targets unless explicitly listed in .security/scope.yaml.
- Dynamic testing is limited to localhost/devnet/staging targets listed in scope.
- Do not run credential brute force, password cracking, persistence, destructive exploitation, malware, or data exfiltration.
- Prefer evidence-based reporting.
- Scanner output alone is not a vulnerability report.

Default security command:
bash scripts/security/run_full_local_pentest.sh

Artifacts:
.security-artifacts/<timestamp>/

Primary skill:
$z00z-local-pentest
```

Codex reads `AGENTS.md` before work and layers global/project instructions, so this is the right place for permanent “no MCP/no API/no public scan” constraints. ([OpenAI Developers](https://developers.openai.com/codex/guides/agents-md))

------

# 15. Как запускать в Codex

Из корня проекта:

```bash
codex
```

Потом в Codex:

```text
Use $z00z-local-pentest.
Run a local-only authorized penetration test for this repository.
Do not use MCP.
Do not use API keys.
Use only local scripts and local tools.
Use .security/scope.yaml.
Run static scans first, then local DAST only for localhost targets.
Return an evidence-based report and proposed fixes.
```

Для параллельного conceptual audit:

```text
Use $z00z-local-pentest.
Spawn parallel subagents for:
1. Rust unsafe/panic/deserialization review
2. Wallet/key/privacy boundary review
3. Checkpoint/delta/double-spend logic review
4. RPC/API/local DAST review
5. Dependency/secrets/supply-chain review

Do not use MCP or API keys. Use local scripts only. Wait for all subagents and merge findings.
```

Codex docs say subagents are useful for parallel codebase exploration and multi-step review, but Codex spawns them only when explicitly asked. ([OpenAI Developers](https://developers.openai.com/codex/subagents))

------

# 16. Как физически вытащить из репозиториев

Создай vendor area:

```bash
mkdir -p third_party/security-playbooks
cd third_party/security-playbooks

git clone https://github.com/usestrix/strix.git
git clone https://github.com/0x4m4/hexstrike-ai.git
```

Потом:

```bash
mkdir -p ../../.agents/skills/_imported_strix
cp -R strix/strix/skills/* ../../.agents/skills/_imported_strix/
```

Но не оставляй всё как active skills. Лучше переложить в references:

```bash
mkdir -p ../../.agents/skills/z00z-source-aware-sast/references/strix
cp strix/strix/skills/custom/source_aware_sast.md \
  ../../.agents/skills/z00z-source-aware-sast/references/strix/

cp strix/strix/skills/coordination/source_aware_whitebox.md \
  ../../.agents/skills/z00z-local-pentest/references/

cp strix/strix/skills/tooling/semgrep.md \
  ../../.agents/skills/z00z-source-aware-sast/references/

cp strix/strix/skills/tooling/nmap.md \
  ../../.agents/skills/z00z-local-dast/references/

cp strix/strix/skills/tooling/nuclei.md \
  ../../.agents/skills/z00z-local-dast/references/
```

HexStrike extract:

```bash
mkdir -p ../../.agents/skills/z00z-local-dast/references/hexstrike

cp hexstrike-ai/README.md \
  ../../.agents/skills/z00z-local-dast/references/hexstrike/README.md

cp hexstrike-ai/hexstrike_mcp.py \
  ../../.agents/skills/z00z-local-dast/references/hexstrike/hexstrike_mcp_reference_only.py
```

Добавь сверху в copied HexStrike file:

```text
REFERENCE ONLY. Do not run this file. Do not start MCP. Do not start HexStrike server.
Use only to inspect tool names and CLI patterns.
```

------

# 17. Install local tools

Минимальный install script:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "[install] system packages"
sudo apt update
sudo apt install -y \
  nmap \
  jq \
  ripgrep \
  fd-find \
  git \
  curl \
  python3 \
  python3-pip \
  pipx \
  golang-go \
  build-essential \
  pkg-config \
  libssl-dev

echo "[install] Python tools"
if command -v uv >/dev/null 2>&1; then
  uv tool install semgrep || true
else
  pipx install semgrep || true
fi

echo "[install] Rust cargo tools"
cargo install cargo-audit || true
cargo install cargo-deny || true
cargo install cargo-geiger || true
cargo install cargo-nextest --locked || true
cargo install cargo-fuzz || true

echo "[install] Go security tools"
go install github.com/gitleaks/gitleaks/v8@latest || true
go install github.com/projectdiscovery/nuclei/v3/cmd/nuclei@latest || true
go install github.com/projectdiscovery/httpx/cmd/httpx@latest || true
go install github.com/projectdiscovery/katana/cmd/katana@latest || true
go install github.com/ffuf/ffuf/v2@latest || true
go install github.com/ast-grep/ast-grep/cmd/sg@latest || true

echo "[install] trivy"
if ! command -v trivy >/dev/null 2>&1; then
  echo "Install trivy using your preferred distro method or Aqua official installer."
fi

echo "[install] check PATH"
for t in semgrep sg gitleaks trufflehog trivy cargo-audit cargo-deny cargo-geiger nmap nuclei httpx katana ffuf; do
  if command -v "$t" >/dev/null 2>&1; then
    echo "OK $t: $(command -v "$t")"
  else
    echo "MISSING $t"
  fi
done
```

Некоторые tools, особенно `trivy`, `nuclei templates`, `cargo audit DB`, требуют локальных баз/шаблонов. Для “без API keys” это нормально, но для “полностью offline” нужно заранее vendor/cache базы и запускать offline flags. Strix SAST уже использует `trivy fs --offline-scan`, что хорошо для твоего режима. ([GitHub](https://raw.githubusercontent.com/usestrix/strix/main/strix/skills/custom/source_aware_sast.md))

------

# 18. Что получится на практике

После запуска ты получишь:

```text
.security-artifacts/
  2026-06-30_19-xx-xx/
    sast/
      semgrep-default.json
      semgrep-local.json
      ast-grep.json
      tree-sitter-rust.log

    rust/
      cargo-audit.out
      cargo-deny.out
      cargo-geiger.out
      cargo-clippy.out
      cargo-test.out

    secrets/
      gitleaks.json
      trufflehog.jsonl
      trivy-fs.json

    dast/
      nmap-local-quick.xml
      nuclei-local.jsonl
      ffuf-*.json

    report/
      security-report.md

    logs/
      source_sast.log
      rust_security.log
      secrets_supply_chain.log
      local_dast.log
```

Codex потом читает эти artifacts, делает triage, дедупликацию, связывает scanner finding с конкретным файлом/функцией и предлагает fix/test.

------

# 19. Что я бы НЕ делал

Не делал бы:

```text
- Не запускал бы Strix.
- Не запускал бы HexStrike.
- Не поднимал бы MCP.
- Не импортировал бы все 150+ tools.
- Не давал бы Codex arbitrary command wrapper.
- Не делал бы auto-run для hydra/john/hashcat/metasploit/sqlmap.
- Не делал бы public recon из skill.
- Не смешивал бы generic pentest и Z00Z protocol audit в один огромный prompt.
```

Strix требует LLM API key в quickstart, а ты как раз хочешь без API. ([GitHub](https://github.com/usestrix/strix))
HexStrike сам предупреждает, что AI agents получают мощный system access и что это нужно запускать в isolated environments / dedicated testing VMs. ([GitHub](https://github.com/0x4m4/hexstrike-ai))

------

# 20. Финальная recommended программа

Мой вариант:

```text
Phase 1 — Extract
- Clone Strix and HexStrike.
- Copy Strix source-aware and tooling skills into references.
- Copy HexStrike README/tool inventory only as reference.
- Do not execute Strix or HexStrike.

Phase 2 — Repackage
- Create 7 Codex skills:
  1. z00z-local-pentest
  2. z00z-source-aware-sast
  3. z00z-rust-security
  4. z00z-secrets-supply-chain
  5. z00z-local-dast
  6. z00z-crypto-protocol-audit
  7. z00z-security-report

Phase 3 — Local tools
- Install local binaries.
- Vendor/copy local rules where reproducibility matters.
- Use no API keys.
- Use no MCP.

Phase 4 — Scripts
- Put actual execution in bash/python scripts.
- Let Codex skills describe when/how to run scripts.
- Use parallel bash runner for static scans.

Phase 5 — Codex run
- Invoke $z00z-local-pentest from project root.
- Codex runs scripts, reads artifacts, validates findings, proposes patches/tests.

Phase 6 — Iteration
- Add Z00Z-specific Semgrep rules.
- Add crypto/protocol checklists.
- Add regression tests for every confirmed finding.
```

**Итог:** да, это будет работать в Codex. Самый правильный путь — **Strix skills → Codex skills**, **HexStrike tools → local CLI scripts**, **никакого MCP**, **никаких API keys**, **никакого чужого agent runtime**.