# llm-wiki in Codex

Verified in this workspace on 2026-06-26.

This note explains the difference between these two surfaces:

- `Wiki Manager [$wiki:wiki](/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/SKILL.md)`
- `LLM Wiki [@wiki](plugin://wiki@llm-wiki)`

## 🎯 Short Answer

- `@wiki` is the normal user-facing Codex entry point. Use this in chat.
- `$wiki:wiki` is the underlying skill identifier exposed by the plugin.
- `plugin://wiki@llm-wiki` is a Codex UI/plugin link, not a shell command.
- The `SKILL.md` file is the instruction source that Codex loads when the skill is activated.

These are not two different wiki systems. `@wiki` is the explicit user-facing way to activate the same underlying `wiki:wiki` skill.

If you only want to use llm-wiki, start with `@wiki`.

## 🧩 What Each Name Means

| Surface | What it is | Source of truth | How to use it |
|---|---|---|---|
| `LLM Wiki` | Plugin display name | `plugin.json` | Seen in Codex plugin UI |
| `@wiki` | Explicit plugin invocation | Codex plugin runtime | Type it in chat: `@wiki ...` |
| `Wiki Manager` | Skill display name | `skills/wiki/agents/openai.yaml` | Mostly UI/debug metadata |
| `wiki:wiki` | Plugin skill id | Codex skill registry | Advanced/debug forcing of the skill |
| `SKILL.md` | The actual instructions | Plugin cache / plugin repo | Read it for behavior details; do not "run" it directly in chat |

## ▶️ Which One To Launch

### ✅ Normal usage

Use `@wiki`.

Examples:

```text
@wiki compile --local
@wiki lint --local --fix
@wiki ingest docs/Z00Z-Main-Whitepaper.md --local --type papers
@wiki "Use the local .wiki only and summarize the main whitepaper themes."
```

### ✅ Natural-language usage

You can also ask for wiki work without `@wiki`.

This works because the skill policy allows implicit invocation:

```yaml
policy:
  allow_implicit_invocation: true
```

Practical meaning: if the request clearly looks like wiki ingestion, compile, lint, audit, or query work, Codex may auto-activate `wiki:wiki`.

### ⚙️ Advanced/debug usage

Use `$wiki:wiki` only when you want to force the skill by name or debug skill routing.

For day-to-day work, prefer `@wiki`.

### ❌ What not to use in Codex

Do not use Claude-style `/wiki:*` slash commands in Codex.

The Codex skill explicitly says that Codex plugins do not register custom `/wiki:*` commands. In Codex, use:

- `@wiki ...`
- natural language that triggers the wiki skill

## ⚙️ Install And Enable

This repo already includes a local installer script:

```bash
./scripts/install_nvk_llm_wiki.sh
```

Useful checks after install:

```bash
codex plugin marketplace list | rg '^llm-wiki\\b'
codex plugin list | rg '^wiki@llm-wiki\\b'
```

Expected current state in this workspace:

```text
llm-wiki        /home/vadim/.cache/z00z/llm-wiki
wiki@llm-wiki  installed, enabled  0.12.0
```

## 🔍 Launch Probes

These probes were run in this workspace and are safe to reuse.

### 1. Marketplace is registered

```bash
codex plugin marketplace list | rg '^llm-wiki\\b'
```

### 2. Plugin is installed and enabled

```bash
codex plugin list | rg '^wiki@llm-wiki\\b'
```

### 3. Codex exposes the plugin skill

```bash
codex -C "$PWD" debug prompt-input '@wiki test' | rg 'wiki:wiki|r3/wiki/0\\.12\\.0/skills/wiki/SKILL\\.md'
```

This probe confirms that a Codex prompt with `@wiki` exposes the `wiki:wiki` skill from the llm-wiki plugin cache.

> [!NOTE]
> In this workspace on 2026-06-26, the upstream helper `verify-codex-plugin.sh --scope user` returned a false negative because it mis-parsed `codex debug prompt-input` output. The direct probes above are the reliable checks that were actually observed to pass here.

## 📚 Local Wiki In This Repo

This repository already has a project-local wiki at `.wiki/`.

Current verified state:

- `docs/*Whitepaper.md`: 15 source files
- `.wiki/raw/papers/`: 15 ingested paper files plus `_index.md`
- `.wiki/wiki/`: 7 compiled wiki articles

Representative source files:

- `docs/Z00Z-Main-Whitepaper.md`
- `docs/Z00Z-Privacy-Threat-Model-Whitepaper.md`
- `docs/Z00Z-Tokenomics-Incentives-Whitepaper.md`

Representative raw outputs already present:

- `.wiki/raw/papers/2026-06-26-main.md`
- `.wiki/raw/papers/2026-06-26-privacy-threat-model.md`
- `.wiki/raw/papers/2026-06-26-tokenomics.md`

Representative compiled outputs already present:

- `.wiki/wiki/references/whitepaper-corpus.md`
- `.wiki/wiki/topics/protocol-core.md`
- `.wiki/wiki/topics/governance-and-economics.md`

## 🧠 Mental Model

For a new user, the simplest mental model is:

1. `ingest` takes an external source and writes one normalized raw file into `.wiki/raw/`
2. `compile` reads `.wiki/raw/` and synthesizes fewer, better-connected articles into `.wiki/wiki/`
3. `lint` checks whether the wiki is structurally healthy, and `--fix` repairs mechanical issues

For this repo, the whitepaper flow looks like this:

```text
docs/Z00Z-Main-Whitepaper.md
  -> @wiki ingest ... --local --type papers
  -> .wiki/raw/papers/2026-06-26-main.md
  -> @wiki compile --local
  -> .wiki/wiki/topics/*.md and/or .wiki/wiki/references/*.md
```

Important consequence: `compile` does not read `docs/` directly. It reads the already-ingested files under `.wiki/raw/`.

## 📍 What `--local` Means

`--local` tells llm-wiki to use the project-local wiki rooted at `<repo>/.wiki/`.

In this repo, that means:

- target wiki root: `.wiki/`
- raw sources go under `.wiki/raw/`
- compiled articles go under `.wiki/wiki/`
- indexes and logs are updated inside `.wiki/`

Why use it even though this repo already has `.wiki/`?

- it makes your intent explicit
- it prevents accidental work against a global hub wiki
- it is the highest-priority wiki selector in the resolution order

For this repository, `--local` is the right default for whitepaper work.

## 🛠️ What `--fix` Means

`--fix` belongs to `lint`, not to `ingest` or `compile`.

Without `--fix`, `lint` is report-only.

With `--fix`, `lint` is allowed to auto-repair clear mechanical issues such as:

- missing or stale `_index.md` entries
- broken counts and dates in indexes
- safe frontmatter alias rewrites
- canonical placement moves such as a `type: papers` file being moved back into `raw/papers/`
- missing indexes inside existing optional layers
- safe default `volatility: warm` when that field is absent

`--fix` does **not** mean "improve content". It does not:

- rewrite claims for correctness
- invent missing rationale
- create arbitrary project structure
- rewrite article prose just because it could be better

## ⚙️ Command Breakdown For New Users

### `@wiki ingest <source> --local`

Purpose:

- bring one external source into `.wiki/raw/`
- normalize its frontmatter and storage location
- preserve the source as immutable raw material for later compilation

Typical whitepaper example:

```text
@wiki ingest docs/Z00Z-Main-Whitepaper.md --local --type papers
```

What happens:

1. llm-wiki reads `docs/Z00Z-Main-Whitepaper.md`
2. it classifies the source as a `papers` raw source
3. it writes one normalized raw file under `.wiki/raw/papers/`
4. it updates raw indexes and the master wiki index

Expected output shape:

```text
.wiki/raw/papers/YYYY-MM-DD-main.md
```

Important beginner point:

- `ingest` is fundamentally a single-source command
- it accepts one URL, one file path, or one quoted text block per call
- for many files, either run `ingest` repeatedly or use `--inbox`

Relevant flags:

| Flag | Meaning | When to use it |
|---|---|---|
| `--local` | target `<repo>/.wiki/` | default in this repo |
| `--type papers` | force raw type to `papers` | good for whitepapers and PDFs |
| `--title "..."` | override the detected title | when source title is poor or noisy |
| `--inbox` | process everything in `.wiki/inbox/` | native batch mode |
| `--keep` | keep originals when using `--inbox` | when inbox files must remain |
| `--new-topic <name>` | create a new topic wiki and ingest there | for a new standalone wiki topic |
| `--project <slug>` | tag the ingested source to a project | when using project-scoped outputs |
| `--include-archived` | allow ingest into archived topic wiki | advanced/archive-only workflows |

When to use `ingest`:

- the source is not in `.wiki/raw/` yet
- you want the source preserved before synthesis
- you are adding new material from `docs/`, a URL, a PDF, or quoted notes

When **not** to use `ingest`:

- when the content is already present in `.wiki/raw/`
- when you actually want to synthesize articles from existing raw sources; that is `compile`

### `@wiki compile --local`

Purpose:

- transform raw sources into synthesized wiki articles
- update existing articles or create new concept/topic/reference pages
- cross-link the resulting knowledge base

Typical whitepaper example:

```text
@wiki compile --local
```

What `compile` reads:

- `.wiki/raw/_index.md`
- raw source files under `.wiki/raw/articles|papers|repos|notes|data/`
- existing compiled articles under `.wiki/wiki/`

What `compile` writes:

- `.wiki/wiki/concepts/*.md`
- `.wiki/wiki/topics/*.md`
- `.wiki/wiki/references/*.md`
- updated wiki indexes and log entries

What it means in this repo:

- the 15 whitepaper raw files are the input
- the 7 compiled wiki articles are the synthesized output
- multiple whitepapers can contribute to one compiled article

By default, `compile` is incremental:

- it looks at the last compile date
- it processes only sources ingested after that date

Relevant flags:

| Flag | Meaning | When to use it |
|---|---|---|
| `--local` | compile the local `.wiki/` | default in this repo |
| `--full` | re-read and rewrite from all raw sources | when you want a full rebuild |
| `--source <path>` | compile one raw file only | when testing one new source |
| `--topic <name>` | create or update one specific topic article | when you know the target topic |
| `--include-archived` | compile an archived target wiki explicitly | archive maintenance only |

Good examples:

```text
@wiki compile --local
@wiki compile --local --full
@wiki compile --local --source .wiki/raw/papers/2026-06-26-main.md
```

When to use `compile`:

- after new raw sources were ingested
- after adding multiple new whitepapers to `.wiki/raw/papers/`
- after you want the wiki articles refreshed from raw evidence

### `@wiki lint --local --fix`

Purpose:

- run structural and consistency checks on the wiki
- detect broken links, stale indexes, wrong file placement, frontmatter issues, and similar health problems
- optionally repair the mechanical subset of those issues

Typical whitepaper example:

```text
@wiki lint --local --fix
```

What `lint` checks:

- required structure and `_index.md` files
- frontmatter validity
- index consistency
- link integrity
- source coverage
- canonical placement of files based on frontmatter
- freshness and optional deeper checks

Whitepaper-specific example:

- if `.wiki/raw/notes/2026-06-26-main.md` had frontmatter `type: papers`, `lint --local --fix` should move it to `.wiki/raw/papers/2026-06-26-main.md`
- if `.wiki/wiki/_index.md` counts do not match actual article files, `lint --local --fix` should repair the index metadata

Relevant flags:

| Flag | Meaning | When to use it |
|---|---|---|
| `--local` | lint the local `.wiki/` | default in this repo |
| `--fix` | auto-repair unambiguous mechanical issues | best after large ingest/compile changes |
| `--deep` | add fact-checking and deeper review | slower, more investigative pass |
| `--include-archived` | include archived topic wikis in structural maintenance | archive maintenance |
| `--archived-only` | lint only archived topic wikis | archive-only maintenance |

Good examples:

```text
@wiki lint --local
@wiki lint --local --fix
@wiki lint --local --deep
```

When to use `lint`:

- after ingestion or compilation
- when the wiki feels inconsistent
- before trusting derived indexes
- before or after a larger batch import

## 📦 Whitepaper Workflow In This Repo

If you are starting from the whitepapers under `docs/`, the beginner workflow is:

### 1. Ingest one whitepaper

```text
@wiki ingest docs/Z00Z-Main-Whitepaper.md --local --type papers
```

### 2. Ingest more whitepapers

Repeat `ingest` for additional files such as:

```text
@wiki ingest docs/Z00Z-Privacy-Threat-Model-Whitepaper.md --local --type papers
@wiki ingest docs/Z00Z-Tokenomics-Incentives-Whitepaper.md --local --type papers
```

### 3. Compile the raw papers into wiki articles

```text
@wiki compile --local
```

### 4. Lint and repair the structure

```text
@wiki lint --local --fix
```

### 5. Query or audit the result

```text
@wiki "Use the local .wiki only and summarize the whitepaper corpus."
@wiki audit --local
```

## 📥 Batch Ingestion Note

For a new user, this is the most common confusion:

- `ingest` is one source per call
- `compile` is many raw sources into fewer synthesized articles

If you want a native batch ingest flow inside llm-wiki, use the inbox mode:

```text
@wiki ingest --local --inbox
```

Use that after placing files into `.wiki/inbox/`.

If your whitepapers are already ingested into `.wiki/raw/papers/`, do **not** ingest them again. Start with:

```text
@wiki compile --local
@wiki lint --local --fix
```

## 📎 Evidence Used For This Note

- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/.codex-plugin/plugin.json`
- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/SKILL.md`
- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/agents/openai.yaml`
- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/references/compilation.md`
- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/references/linting.md`
- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/references/ingestion.md`
- `/home/vadim/.codex/plugins/cache/llm-wiki/wiki/0.12.0/skills/wiki/references/wiki-structure.md`
- `/home/vadim/.cache/z00z/llm-wiki/claude-plugin/commands/compile.md`
- `/home/vadim/.cache/z00z/llm-wiki/claude-plugin/commands/lint.md`
- `/home/vadim/.cache/z00z/llm-wiki/claude-plugin/commands/ingest.md`
- `/home/vadim/.cache/z00z/llm-wiki/README.md`
- local runtime probes from `codex plugin list`, `codex plugin marketplace list`, and `codex debug prompt-input`
