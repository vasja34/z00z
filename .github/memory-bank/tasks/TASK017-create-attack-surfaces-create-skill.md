# TASK017 - Create attack-surfaces-create Skill

**Status:** Completed
**Added:** 2026-05-01
**Updated:** 2026-05-01

## Original Request

Create a workspace skill named `attack-surfaces-create` for finding security,
cryptography, and threat attack surfaces in one module, one crate, or the
entire repository. The skill must use SSoT "String Seed of Thought" to run
multiple diversified static-analysis attempts, reject weak ideas, perform a
pro-con audit and verification pass, keep only one strong candidate per run,
and append only verified findings to an attack inventory database.

## Thought Process

The request was not for a generic security-audit clone. The core requirement
was autonomous seeded exploration with hard skepticism gates: a diversified
search phase, strict rejection of weak hypotheses, and append-only persistence
only for findings that survive code-backed verification. The design therefore
needed both a reusable repository skill surface and a deterministic local tool,
so the implementation became a compact skill package with one Python scanner,
one standard report contract, and one reference file that freezes taxonomy,
rejection criteria, and defensive expectations.

## Implementation Plan

- Review local skill-builder and agent-skill rules plus adjacent security and
  crypto audit skills for repository conventions
- Design the workflow around seeded variant generation, skeptical scoring,
  pro-con audit, and single-candidate admission control
- Add `.github/skills/attack-surfaces-create/` with `SKILL.md`, `REFERENCE.md`,
  `FORMS.md`, and `scripts/ssot_attack_surface_scan.py`
- Validate the skill, smoke-test the scanner, and update memory-bank
  continuity files

## Progress Tracking

**Overall Status:** Completed - 100%

### Subtasks

| ID | Description | Status | Updated | Notes |
| --- | --- | --- | --- | --- |
| 17.1 | Inspect repository skill rules and adjacent patterns | Complete | 2026-05-01 | Reviewed memory-bank files, `agent-skills.instructions.md`, skill builder rules, and adjacent security skills |
| 17.2 | Design the seeded attack-surface workflow | Complete | 2026-05-01 | Locked the SSoT axes, skepticism gates, one-candidate rule, and append-only DB contract |
| 17.3 | Create the new skill files and scanner | Complete | 2026-05-01 | Added `SKILL.md`, `REFERENCE.md`, `FORMS.md`, and the Python scanner script |
| 17.4 | Validate and smoke-test the skill | Complete | 2026-05-01 | Skill validator passed cleanly, Python syntax passed, and a self-scan returned a truthful `no-candidate` result |
| 17.5 | Sync continuity docs | Complete | 2026-05-01 | Updated `activeContext.md`, `progress.md`, and `tasks/_index.md` |

## Progress Log

### 2026-05-01

- Interpreted the request as a repository-local attack-surface discovery skill
  with stronger admission control than a normal audit helper: seeded variation,
  skeptical rejection, one strong candidate only, and append-only persistence
- Added a new skill package at `.github/skills/attack-surfaces-create/` with a
  repository-discoverable description, workflow instructions, taxonomy and
  rejection guidance, a standard attack-surface card, and a JSONL database
  entry contract
- Implemented `scripts/ssot_attack_surface_scan.py` as a stdlib-only Python
  scanner that varies threat model, cryptographic primitive, failure scenario,
  implementation constraint, and adversarial angle per variant seed, then
  applies rule-based evidence collection, a skeptical score, and a verification
  gate before admitting a finding
- Preserved the user requirement that only one strong candidate can be written
  per run by selecting the highest-scoring verified candidate and refusing to
  persist rejected or unverifiable attempts
- Ran
  `python .github/skills/skill-builder/scripts/validate-skill.py .github/skills/attack-surfaces-create`
  and got a clean pass
- Ran
  `python -m py_compile .github/skills/attack-surfaces-create/scripts/ssot_attack_surface_scan.py`
  to confirm the scanner syntax
- Ran a bounded smoke test against the skill directory itself and got a truthful
  `no-candidate` result written to `reports/attack-surfaces/skill-self-scan.md`,
  confirming the rejection path works without inventing a weak finding
