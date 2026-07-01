# Wiki — Agent Instructions

## Overview

Generated VitePress documentation site for the Z00Z workspace. It packages source-cited architecture pages, onboarding guides, and LLM-facing summaries.

## Build & Run

- Install dependencies: `npm install`
- Start local docs server: `npm run dev`
- Build static site: `npm run build`
- Preview built site: `npm run preview`

## Wiki Structure

- `index.md` — Developer-facing landing page and documentation map
- `onboarding/` — Audience-specific guides for contributors, staff engineers, executives, and product managers
- `01-getting-started/` — Setup, verification, and workspace orientation
- `02-architecture/` through `07-networking-and-observability/` — Deep technical pages
- `catalogue.json` — Documentation structure used for the VitePress sidebar
- `llms.txt` — LLM-friendly summary with wiki-relative links
- `llms-full.txt` — Inlined full-content LLM context
- `.vitepress/config.mts` — VitePress config and Mermaid setup
- `.vitepress/theme/` — Custom theme, Mermaid color fixups, and zoom behavior

## Content Conventions

- Keep all repository-facing technical content in English
- Preserve VitePress frontmatter on every page
- Use source citations with GitHub links and line numbers
- Keep Mermaid diagrams dark-mode compatible and follow each diagram with a `<!-- Sources: ... -->` comment
- Prefer tables for structured information and keep relative links between pages intact

## Boundaries

- ✅ **Always do:** Update `catalogue.json` when adding new sections, keep citations tied to real files, run `npm run build` after structural changes
- ⚠️ **Ask first:** Changing the theme direction, removing sections, or replacing the citation format
- 🚫 **Never do:** Delete generated pages casually, strip source citations, or turn light-mode-only Mermaid styling into the default

## Documentation

- Wiki root: `./`
- LLM context: `llms.txt` and `llms-full.txt`
- Public copies for deployment: `.vitepress/public/llms.txt` and `.vitepress/public/llms-full.txt`
- Onboarding hub: `onboarding/index.md`
