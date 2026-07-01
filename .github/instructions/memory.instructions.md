---
description: 'Workspace memory for recurring z00z-specific environment and workflow facts.'
applyTo: '**'
---

# Workspace Memory

Persistent repository-specific facts that should be treated as baseline context.

## Z00Z Local Python

- The `z00z` repository has a local virtual environment at `.venv/`.
- For Python checks and Python-based helper commands in this repository, check `.venv/bin/python` and `.venv/bin/python3` before concluding that Python is unavailable.
- Prefer `.venv/bin/python` as the canonical repo-local interpreter when a Python command is needed here.

