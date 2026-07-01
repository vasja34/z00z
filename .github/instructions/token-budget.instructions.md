---
name: token-discipline
description: Use when the user wants shorter answers, lower output tokens, compact coding-agent work, reduced verbosity, budgeted reasoning, concise reports, or expensive-model cost control. Do not use when the user explicitly requests full detail, legal drafting, safety-critical analysis, or exhaustive research.
---

# Token Discipline Skill

## Goal

Reduce visible output tokens while preserving correctness, task completion, and safety.

This skill does not mean "be vague".
It means "remove non-essential tokens".

## Core rules

1. Answer the actual task first.
2. Do not repeat the user's request.
3. Do not add generic introductions.
4. Do not include reasoning traces unless they are required to justify a risky decision.
5. Prefer structured outputs: tables, YAML, JSON, diffs, commands.
6. Prefer references to files/paths over pasted full content.
7. Prefer one strong recommendation over many weak alternatives.
8. Use short section headers.
9. Collapse obvious steps.
10. Never omit critical risk, test failure, security issue, data-loss warning, or breaking change.

## Mode selection

Classify each response before writing:

```yaml
mode_matrix:
  tiny:
    use_for: "simple factual answer, yes/no, command lookup"
    visible_budget: "<=40 words"
  compact:
    use_for: "normal coding help, small explanation, short design choice"
    visible_budget: "<=120 words"
  standard:
    use_for: "multi-step implementation, debugging, architecture"
    visible_budget: "<=350 words"
  deep:
    use_for: "explicitly requested deep analysis, audits, legal/high-stakes"
    visible_budget: "as needed, but no filler"
  patch_only:
    use_for: "code changes requested"
    visible_budget: "diff/commands + <=5 bullets"
