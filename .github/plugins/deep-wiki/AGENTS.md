# Deep Wiki Agent Instructions

## Purpose

Deep Wiki is an agent-agnostic repository documentation plugin. It provides
skills, command prompts, and specialist agent prompts for generating
source-cited wikis, repository Q&A, onboarding guides, `llms.txt`, VitePress
sites, and Azure DevOps wiki exports.

## Runtime Surfaces

- Codex plugin hosts: read `.codex-plugin/plugin.json`, then use
  `skills/*/SKILL.md`. If prompt aliases are installed, use
  `$HOME/.codex/prompts/deep-wiki-*.md`, which point back to repository prompt
  files.
- GitHub Copilot: use projected files under `.github/skills/`,
  `.github/agents/`, and `.github/prompts/` when available.
- Claude-style plugin hosts: read `.claude-plugin/plugin.json`, `commands/`,
  `skills/`, and `agents/`.
- This repository copy does not ship Deep Wiki plugin-local hooks. Use only the
  repository-level hook surface under `./.github/hooks/` when workflow behavior
  matters.
- Project-local override surfaces live under
  `.github/deep-wiki-local-overrides/`. Prefer a matching override file over the
  base plugin file when the override exists.
- Archive or cache copies under `.github/gsd-local-patches/`,
  `$HOME/.codex/plugins/cache/`, or `$HOME/.codex/.tmp/` are reference-only
  unless the task explicitly targets those copies.
- Generic agents: read this `AGENTS.md`, then use `skills/*/SKILL.md` and
  `commands/*.md` directly.

## Operating Rules

- Ground every repository claim in source files and line references.
- Trace real implementation paths before writing documentation or answering
  questions.
- Distinguish verified facts from inference.
- Do not overwrite existing project `AGENTS.md` files when using the
  `wiki-agents-md` workflow.
- Keep generated documentation in the user's requested output directory unless
  the command explicitly names another location.
