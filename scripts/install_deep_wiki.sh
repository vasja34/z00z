#!/usr/bin/env bash
# install_deep_wiki.sh
#
# Install or refresh the Microsoft Deep Wiki plugin directly from upstream,
# reapply durable local overrides, and project it into Codex, GitHub Copilot,
# and generic agent-compatible surfaces.
#
# Default source:
#   https://github.com/microsoft/skills.git
#   .github/plugins/deep-wiki
#
# Quick start:
#   ./scripts/install_deep_wiki.sh
#
# Common usage:
#   ./scripts/install_deep_wiki.sh --skip-codex-install
#   ./scripts/install_deep_wiki.sh --ref main --dry-run
#   ./scripts/install_deep_wiki.sh --with-global
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_NAME
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
DEFAULT_PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_PROJECT_ROOT
readonly DEFAULT_REPO_URL="https://github.com/microsoft/skills.git"
readonly DEFAULT_REF="main"
readonly DEFAULT_UPSTREAM_PATH=".github/plugins/deep-wiki"
readonly DEFAULT_PLUGIN_DIR="${DEFAULT_PROJECT_ROOT}/.github/plugins/deep-wiki"
readonly DEFAULT_MARKETPLACE_FILE="${DEFAULT_PROJECT_ROOT}/.agents/plugins/marketplace.json"
readonly DEFAULT_OVERLAY_DIR="${DEFAULT_PROJECT_ROOT}/.github/deep-wiki-local-overrides"
readonly PLUGIN_NAME="deep-wiki"
INSTALL_STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
readonly INSTALL_STAMP

PROJECT_ROOT="$DEFAULT_PROJECT_ROOT"
REPO_URL="$DEFAULT_REPO_URL"
REF="$DEFAULT_REF"
UPSTREAM_PATH="$DEFAULT_UPSTREAM_PATH"
PLUGIN_DIR="$DEFAULT_PLUGIN_DIR"
MARKETPLACE_FILE="$DEFAULT_MARKETPLACE_FILE"
MARKETPLACE_NAME=""
MARKETPLACE_ROOT=""
MARKETPLACE_PLUGIN_PATH=""
OVERLAY_DIR="$DEFAULT_OVERLAY_DIR"
PROJECT_PLUGIN_DIR=""
BACKUP_ROOT=""

DRY_RUN=0
KEEP_TEMP=0
SKIP_OVERLAYS=0
REFRESH_OVERLAYS=0
SKIP_PROJECT=0
SKIP_GLOBAL=1
SKIP_CODEX_INSTALL=0
VERIFY=1

TEMP_DIR=""
SOURCE_PLUGIN_DIR=""
PREPARED_PLUGIN_DIR=""

usage() {
  cat <<EOF
Usage: ${SCRIPT_NAME} [OPTIONS]

Install Microsoft Deep Wiki from upstream and project it into agent-agnostic
surfaces:
  - Shared plugin source for Copilot + Codex: .github/plugins/deep-wiki
  - Codex repo marketplace: .agents/plugins/marketplace.json
  - Repo Codex skills: .agents/skills/wiki-*
  - Project Copilot surface: .github/plugins, .github/skills, .github/agents,
    .github/prompts
  - Codex prompt hints: ~/.codex/prompts/deep-wiki-*.md -> .github/prompts/*
  - Optional global mirrors: ~/.codex/skills/wiki-*, ~/.agents/skills/wiki-*,
    ~/.agents/commands/deep-wiki

Notes:
  - Codex can install this plugin from the same project-local source tree that
    Copilot uses.
  - Codex plugin installs expose skills. Codex custom prompt hints are
    surfaced through ~/.codex/prompts symlinks that point back to repo prompt
    files and are invoked as /prompts:deep-wiki-ask.

Options:
  --project-root <dir>       Project root for repo-local projections.
                             Default: ${DEFAULT_PROJECT_ROOT}
  --plugin-dir <dir>         Canonical local plugin source directory.
                             Default: ${DEFAULT_PLUGIN_DIR}
  --marketplace-file <file>  Codex repo marketplace file.
                             Default: ${DEFAULT_MARKETPLACE_FILE}
  --marketplace-name <name>  Marketplace identifier for Codex.
                             Default: <project-name>-local
  --overlay-dir <dir>        Durable local override directory re-applied after
                             each upstream refresh.
                             Default: ${DEFAULT_OVERLAY_DIR}
  --repo-url <url>           Git repository URL.
                             Default: ${DEFAULT_REPO_URL}
  --ref <ref>                Branch, tag, or ref to fetch.
                             Default: ${DEFAULT_REF}
  --upstream-path <path>     Plugin path inside the repository.
                             Default: ${DEFAULT_UPSTREAM_PATH}
  --refresh-overlays         Rebuild the overlay directory from the current
                             installed plugin diff before applying overrides.
  --skip-overlays            Disable capture and reapply of local overrides.
  --skip-project             Do not write project-local .github/.agents/.codex projections.
  --with-global              Also write global ~/.codex and ~/.agents mirrors.
  --skip-global              Legacy alias for local-only install (default).
  --skip-codex-install       Do not run 'codex plugin add'.
  --no-verify                Skip final verification checks.
  --keep-temp                Keep the temporary sparse checkout.
  --dry-run                  Print the plan and exit without changing files.
  -h, --help                 Show this help.

Examples:
  ${SCRIPT_NAME}
  ${SCRIPT_NAME} --with-global
  ${SCRIPT_NAME} --project-root "\$PWD" --ref main
EOF
}

log() {
  printf '[install_deep_wiki] %s\n' "$*" >&2
}

die() {
  printf '[install_deep_wiki] ERROR: %s\n' "$*" >&2
  exit 1
}

run() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '[dry-run] ' >&2
    printf '%q ' "$@" >&2
    printf '\n' >&2
    return 0
  fi

  "$@"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"
}

normalize_path() {
  local path="$1"

  if [[ "$path" == /* ]]; then
    printf '%s' "$path"
  else
    printf '%s/%s' "$PWD" "$path"
  fi
}

slugify() {
  local text="$1"
  local slug

  slug="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')"
  printf '%s' "${slug:-repo}"
}

default_marketplace_name() {
  local project_root="$1"

  printf '%s-local' "$(slugify "$(basename "$project_root")")"
}

same_path() {
  [[ "$(realpath -m "$1")" == "$(realpath -m "$2")" ]]
}

marketplace_root_from_file() {
  local marketplace_file="$1"

  case "$marketplace_file" in
    */.agents/plugins/marketplace.json)
      printf '%s' "${marketplace_file%/.agents/plugins/marketplace.json}"
      ;;
    *)
      die "Marketplace file must end with '.agents/plugins/marketplace.json': $marketplace_file"
      ;;
  esac
}

compute_marketplace_plugin_path() {
  local marketplace_root="$1"
  local plugin_dir="$2"
  local relative_path

  relative_path="$(realpath -m --relative-to "$marketplace_root" "$plugin_dir")"
  [[ "$relative_path" != ".." && "$relative_path" != ../* ]] \
    || die "Plugin directory must stay inside the marketplace root: $plugin_dir"

  printf './%s' "$relative_path"
}

safe_dispose_dir() {
  local path="$1"

  [[ -n "$path" && -d "$path" ]] || return 0

  if command -v trash-put >/dev/null 2>&1; then
    if trash-put "$path" >/dev/null 2>&1; then
      return 0
    fi
  fi

  if command -v gio >/dev/null 2>&1; then
    if gio trash "$path" >/dev/null 2>&1; then
      return 0
    fi
  fi

  if command -v python3 >/dev/null 2>&1; then
    python3 - "$path" <<'PY'
import os
import shutil
import sys

path = sys.argv[1]

if os.path.islink(path) or os.path.isfile(path):
    os.unlink(path)
elif os.path.isdir(path):
    shutil.rmtree(path, ignore_errors=True)
PY
    [[ ! -e "$path" && ! -L "$path" ]] && return 0
  fi

  die "Failed to dispose temporary directory: $path"
}

cleanup() {
  if [[ "$KEEP_TEMP" -eq 1 ]]; then
    [[ -n "$TEMP_DIR" ]] && log "Temporary directory kept: $TEMP_DIR"
    return 0
  fi

  [[ "$DRY_RUN" -eq 1 ]] && return 0
  safe_dispose_dir "$TEMP_DIR"
}

backup_path_if_exists() {
  local target="$1"
  local backup="${BACKUP_ROOT}${target}"

  if [[ ! -e "$target" && ! -L "$target" ]]; then
    return 0
  fi

  [[ ! -e "$backup" && ! -L "$backup" ]] || die "Backup path already exists: $backup"

  log "Backing up $target to $backup"
  run mkdir -p "$(dirname "$backup")"
  run mv "$target" "$backup"
}

copy_dir_fresh() {
  local source_dir="$1"
  local target_dir="$2"

  [[ -d "$source_dir" ]] || die "Source directory not found: $source_dir"

  if [[ -d "$target_dir" && ! -L "$target_dir" ]]; then
    if diff -qr "$source_dir" "$target_dir" >/dev/null 2>&1; then
      log "Directory already current: $target_dir"
      return 0
    fi
  fi

  backup_path_if_exists "$target_dir"
  run mkdir -p "$(dirname "$target_dir")"
  log "Installing directory: $target_dir"
  run cp -a "$source_dir" "$target_dir"
}

count_dir_files() {
  local dir="$1"

  if [[ ! -d "$dir" ]]; then
    printf '0'
    return 0
  fi

  find "$dir" -type f | wc -l | awk '{ print $1 }'
}

list_relative_files() {
  local dir="$1"

  [[ -d "$dir" ]] || return 0
  (
    cd "$dir"
    find . -type f | sed 's#^\./##' | sort
  )
}

write_text_file() {
  local target_file="$1"
  local target_dir
  local temp_file

  if [[ "$DRY_RUN" -eq 1 ]]; then
    cat >/dev/null
    log "Would write file: $target_file"
    return 0
  fi

  target_dir="$(dirname "$target_file")"
  mkdir -p "$target_dir"
  temp_file="${target_file}.${INSTALL_STAMP}.tmp"

  cat >"$temp_file"

  if [[ -f "$target_file" ]] && cmp -s "$temp_file" "$target_file"; then
    rm -f "$temp_file"
    log "File already current: $target_file"
    return 0
  fi

  backup_path_if_exists "$target_file"
  mv "$temp_file" "$target_file"
}

extract_frontmatter_description() {
  local file_path="$1"

  awk '
    NR == 1 && $0 == "---" { in_fm = 1; next }
    in_fm && $0 == "---" { exit }
    in_fm && /^description:/ {
      sub(/^description:[[:space:]]*/, "")
      print
      exit
    }
  ' "$file_path"
}

extract_markdown_body() {
  local file_path="$1"

  awk '
    NR == 1 && $0 == "---" { in_fm = 1; next }
    in_fm && $0 == "---" { in_fm = 0; next }
    !in_fm { print }
  ' "$file_path"
}

title_from_slug() {
  local slug="$1"
  local part
  local output=""

  IFS='-' read -r -a parts <<<"$slug"
  for part in "${parts[@]}"; do
    [[ -n "$part" ]] || continue
    output+="${part^} "
  done

  printf '%s' "${output% }"
}

json_string() {
  jq -Rn --arg value "$1" '$value'
}

print_plan() {
  cat <<EOF
Deep Wiki install plan:
  repo:             $REPO_URL
  ref:              $REF
  upstream path:    $UPSTREAM_PATH
  plugin dir:       $PLUGIN_DIR
  overlay dir:      $OVERLAY_DIR
  marketplace file: $MARKETPLACE_FILE
  marketplace name: $MARKETPLACE_NAME
  marketplace root: $MARKETPLACE_ROOT
  source.path:      $MARKETPLACE_PLUGIN_PATH
  backup root:      $BACKUP_ROOT
  project root:     $PROJECT_ROOT
  overlays:         $([[ "$SKIP_OVERLAYS" -eq 1 ]] && printf 'skip' || printf 'apply')
  overlay refresh:  $([[ "$REFRESH_OVERLAYS" -eq 1 ]] && printf 'rebuild' || printf 'reuse-or-seed')
  project surface:  $([[ "$SKIP_PROJECT" -eq 1 ]] && printf 'skip' || printf 'write')
  global surface:   $([[ "$SKIP_GLOBAL" -eq 1 ]] && printf 'skip' || printf 'write')
  codex install:    $([[ "$SKIP_CODEX_INSTALL" -eq 1 ]] && printf 'skip' || printf 'run')
EOF
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --project-root)
        [[ $# -ge 2 ]] || die "Missing value for --project-root"
        PROJECT_ROOT="$2"
        shift 2
        ;;
      --plugin-dir)
        [[ $# -ge 2 ]] || die "Missing value for --plugin-dir"
        PLUGIN_DIR="$2"
        shift 2
        ;;
      --marketplace-file)
        [[ $# -ge 2 ]] || die "Missing value for --marketplace-file"
        MARKETPLACE_FILE="$2"
        shift 2
        ;;
      --marketplace-name)
        [[ $# -ge 2 ]] || die "Missing value for --marketplace-name"
        MARKETPLACE_NAME="$2"
        shift 2
        ;;
      --overlay-dir)
        [[ $# -ge 2 ]] || die "Missing value for --overlay-dir"
        OVERLAY_DIR="$2"
        shift 2
        ;;
      --repo-url)
        [[ $# -ge 2 ]] || die "Missing value for --repo-url"
        REPO_URL="$2"
        shift 2
        ;;
      --ref)
        [[ $# -ge 2 ]] || die "Missing value for --ref"
        REF="$2"
        shift 2
        ;;
      --upstream-path)
        [[ $# -ge 2 ]] || die "Missing value for --upstream-path"
        UPSTREAM_PATH="$2"
        shift 2
        ;;
      --refresh-overlays)
        REFRESH_OVERLAYS=1
        shift
        ;;
      --skip-overlays)
        SKIP_OVERLAYS=1
        shift
        ;;
      --skip-project)
        SKIP_PROJECT=1
        shift
        ;;
      --skip-global)
        SKIP_GLOBAL=1
        shift
        ;;
      --with-global)
        SKIP_GLOBAL=0
        shift
        ;;
      --skip-codex-install)
        SKIP_CODEX_INSTALL=1
        shift
        ;;
      --no-verify)
        VERIFY=0
        shift
        ;;
      --keep-temp)
        KEEP_TEMP=1
        shift
        ;;
      --dry-run)
        DRY_RUN=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "Unknown option: $1"
        ;;
    esac
  done
}

normalize_inputs() {
  PROJECT_ROOT="$(normalize_path "$PROJECT_ROOT")"
  PLUGIN_DIR="$(normalize_path "$PLUGIN_DIR")"
  MARKETPLACE_FILE="$(normalize_path "$MARKETPLACE_FILE")"
  OVERLAY_DIR="$(normalize_path "$OVERLAY_DIR")"
  PROJECT_PLUGIN_DIR="$PROJECT_ROOT/.github/plugins/$PLUGIN_NAME"
  MARKETPLACE_ROOT="$(marketplace_root_from_file "$MARKETPLACE_FILE")"
  MARKETPLACE_NAME="${MARKETPLACE_NAME:-$(default_marketplace_name "$PROJECT_ROOT")}"
  MARKETPLACE_PLUGIN_PATH="$(compute_marketplace_plugin_path "$MARKETPLACE_ROOT" "$PLUGIN_DIR")"
  BACKUP_ROOT="$PROJECT_ROOT/.agents/.install-backups/$PLUGIN_NAME/$INSTALL_STAMP"

  local overlay_relative
  overlay_relative="$(realpath -m --relative-to "$PLUGIN_DIR" "$OVERLAY_DIR")"
  if [[ "$overlay_relative" == "." || ( "$overlay_relative" != ".." && "$overlay_relative" != ../* ) ]]; then
    die "Overlay directory must live outside the plugin directory: $OVERLAY_DIR"
  fi
}

fetch_upstream_plugin() {
  TEMP_DIR="$(mktemp -d)"
  local checkout_dir="${TEMP_DIR}/skills"

  log "Fetching $REPO_URL at ref '$REF'"
  run git clone --filter=blob:none --no-checkout "$REPO_URL" "$checkout_dir"
  run git -C "$checkout_dir" sparse-checkout init --no-cone
  run git -C "$checkout_dir" sparse-checkout set "$UPSTREAM_PATH"
  run git -C "$checkout_dir" fetch --depth 1 origin "$REF"
  run git -C "$checkout_dir" checkout --detach FETCH_HEAD

  SOURCE_PLUGIN_DIR="${checkout_dir}/${UPSTREAM_PATH}"
  [[ -d "$SOURCE_PLUGIN_DIR" ]] || die "Plugin path not found in upstream checkout: $UPSTREAM_PATH"
}

prepare_plugin_copy() {
  local version
  local codex_version
  local source_revision

  PREPARED_PLUGIN_DIR="${TEMP_DIR}/${PLUGIN_NAME}-prepared"
  run cp -a "$SOURCE_PLUGIN_DIR" "$PREPARED_PLUGIN_DIR"

  version="$(jq -r '.version // "2.0.0"' "$PREPARED_PLUGIN_DIR/.claude-plugin/plugin.json")"
  source_revision="$(git -C "$SOURCE_PLUGIN_DIR" rev-parse --short=12 HEAD 2>/dev/null || printf '%s' "$INSTALL_STAMP")"
  codex_version="${version%%+*}+codex.${source_revision}"

  write_codex_manifest "$PREPARED_PLUGIN_DIR" "$codex_version"
  write_portable_agents_file "$PREPARED_PLUGIN_DIR"
  rewrite_readme_header "$PREPARED_PLUGIN_DIR"
}

capture_overlay_snapshot() {
  local current_dir="$1"
  local baseline_dir="$2"
  local overlay_dir="$3"
  local staged_dir="${TEMP_DIR}/overlay.${INSTALL_STAMP}"
  local overlay_count=0
  local relative_path=""
  local current_file=""
  local baseline_file=""
  local target_file=""

  [[ -d "$current_dir" ]] || {
    log "Current plugin directory not found for overlay capture: $current_dir"
    return 0
  }

  mkdir -p "$staged_dir"

  while IFS= read -r relative_path; do
    [[ -n "$relative_path" ]] || continue

    current_file="${current_dir}/${relative_path}"
    baseline_file="${baseline_dir}/${relative_path}"

    if [[ -f "$baseline_file" ]] && cmp -s "$current_file" "$baseline_file"; then
      continue
    fi

    target_file="${staged_dir}/${relative_path}"
    run mkdir -p "$(dirname "$target_file")"
    run cp -a "$current_file" "$target_file"
    overlay_count=$((overlay_count + 1))
  done < <(list_relative_files "$current_dir")

  if [[ "$overlay_count" -eq 0 ]]; then
    log "No local deep-wiki overrides detected; overlay directory left unchanged"
    return 0
  fi

  if [[ -d "$overlay_dir" ]] && diff -qr "$staged_dir" "$overlay_dir" >/dev/null 2>&1; then
    log "Overlay directory already current: $overlay_dir"
    return 0
  fi

  backup_path_if_exists "$overlay_dir"
  run mkdir -p "$(dirname "$overlay_dir")"
  log "Writing overlay directory: $overlay_dir"
  run cp -a "$staged_dir" "$overlay_dir"
}

capture_local_overrides() {
  [[ "$SKIP_OVERLAYS" -eq 0 ]] || return 0

  if [[ "$REFRESH_OVERLAYS" -eq 0 && -d "$OVERLAY_DIR" ]]; then
    log "Using existing overlay directory: $OVERLAY_DIR"
    return 0
  fi

  if [[ "$REFRESH_OVERLAYS" -eq 1 ]]; then
    log "Refreshing local overrides from $PLUGIN_DIR"
  else
    log "Seeding local overrides from $PLUGIN_DIR"
  fi

  capture_overlay_snapshot "$PLUGIN_DIR" "$PREPARED_PLUGIN_DIR" "$OVERLAY_DIR"
}

apply_overlay_dir() {
  [[ "$SKIP_OVERLAYS" -eq 0 ]] || return 0

  local overlay_files
  overlay_files="$(count_dir_files "$OVERLAY_DIR")"
  if [[ "$overlay_files" -eq 0 ]]; then
    log "No overlay files to apply from $OVERLAY_DIR"
    return 0
  fi

  log "Applying ${overlay_files} overlay file(s) from $OVERLAY_DIR"
  run cp -a "$OVERLAY_DIR"/. "$PREPARED_PLUGIN_DIR"/
}

write_codex_manifest() {
  local plugin_dir="$1"
  local codex_version="$2"

  mkdir -p "$plugin_dir/.codex-plugin"

  write_text_file "$plugin_dir/.codex-plugin/plugin.json" <<EOF
{
  "name": "deep-wiki",
  "version": "$(printf '%s' "$codex_version")",
  "description": "Agent-agnostic wiki generator for code repositories with source citations, Mermaid-rich pages, onboarding guides, and VitePress packaging.",
  "author": {
    "name": "Microsoft / thegovind"
  },
  "homepage": "https://github.com/microsoft/skills/tree/main/.github/plugins/deep-wiki",
  "repository": "https://github.com/microsoft/skills",
  "license": "MIT",
  "keywords": [
    "deep-wiki",
    "wiki",
    "documentation",
    "codebase-analysis",
    "mermaid",
    "vitepress",
    "onboarding",
    "agent-agnostic"
  ],
  "skills": "./skills/",
  "interface": {
    "displayName": "Deep Wiki",
    "shortDescription": "Generate source-cited repository wikis, onboarding docs, Q&A, and VitePress sites.",
    "longDescription": "Deep Wiki provides portable repository documentation workflows for Codex, GitHub Copilot, Claude-style plugin hosts, and agents that read SKILL.md, command markdown, or AGENTS.md files.",
    "developerName": "Microsoft / thegovind",
    "category": "Developer Tools",
    "capabilities": [
      "Read",
      "Write"
    ],
    "defaultPrompt": [
      "Generate a source-cited wiki for this repository.",
      "Research how this subsystem works with file citations.",
      "Create onboarding docs and llms.txt for this project."
    ],
    "brandColor": "#2563EB"
  }
}
EOF
}

write_portable_agents_file() {
  local plugin_dir="$1"

  write_text_file "$plugin_dir/AGENTS.md" <<'EOF'
# Deep Wiki Agent Instructions

## Purpose

Deep Wiki is an agent-agnostic repository documentation plugin. It provides
skills, command prompts, and specialist agent prompts for generating
source-cited wikis, repository Q&A, onboarding guides, `llms.txt`, VitePress
sites, and Azure DevOps wiki exports.

## Runtime Surfaces

- Codex plugin hosts: read `.codex-plugin/plugin.json`, then use `skills/*/SKILL.md`.
- GitHub Copilot: use projected files under `.github/skills/`,
  `.github/agents/`, and `.github/prompts/` when available.
- Claude-style plugin hosts: read `.claude-plugin/plugin.json`, `commands/`,
  `skills/`, and `agents/`.
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
EOF
}

rewrite_readme_header() {
  local plugin_dir="$1"
  local readme_file="${plugin_dir}/README.md"
  local body_file="${TEMP_DIR}/README.body.${INSTALL_STAMP}.tmp"
  local new_readme="${TEMP_DIR}/README.${INSTALL_STAMP}.tmp"

  [[ -f "$readme_file" ]] || return 0
  tail -n +2 "$readme_file" >"$body_file"

  {
    cat <<'EOF'
# Deep Wiki

**AI-powered, agent-agnostic wiki generator for code repositories.**

This installed copy includes Codex, GitHub Copilot, Claude-style, and generic
agent surfaces. The canonical upstream source remains
`https://github.com/microsoft/skills/tree/main/.github/plugins/deep-wiki`.

## Agent-Agnostic Installation

- Shared project-local plugin source: `.github/plugins/deep-wiki`
- Codex repo marketplace: `.agents/plugins/marketplace.json`
- Codex plugin hosts: `.codex-plugin/plugin.json` and `skills/*/SKILL.md`.
- GitHub Copilot: projected `.github/skills`, `.github/agents`, and
  `.github/prompts` files.
- Claude-style plugin hosts: `.claude-plugin/plugin.json`, `commands/`,
  `skills/`, and `agents/`.
- Generic agents: `AGENTS.md`, `skills/*/SKILL.md`, `commands/*.md`, and
  `agents/*.md`.

## Codex Notes

- Codex loads Deep Wiki from the repo-local marketplace and plugin source path.
- Codex plugin installs expose skills.
- Codex custom prompt hints are provided by per-user `~/.codex/prompts/*.md`
  symlinks that point back to repository prompt files.
- Invoke them as `/prompts:deep-wiki-ask`, `/prompts:deep-wiki-generate`, and
  similar file-stem commands.
- Restart Codex after changing prompt symlinks so the `/prompts:` menu refreshes.

EOF
    cat "$body_file"
  } >"$new_readme"

  run mv "$new_readme" "$readme_file"
}

install_canonical_plugin() {
  copy_dir_fresh "$PREPARED_PLUGIN_DIR" "$PLUGIN_DIR"
}

install_project_surface() {
  [[ "$SKIP_PROJECT" -eq 0 ]] || return 0

  if same_path "$PLUGIN_DIR" "$PROJECT_PLUGIN_DIR"; then
    log "Canonical plugin source already lives at $PROJECT_PLUGIN_DIR"
  else
    copy_dir_fresh "$PREPARED_PLUGIN_DIR" "$PROJECT_PLUGIN_DIR"
  fi

  install_project_skills
  install_project_agents
  install_project_prompts
  install_codex_prompt_symlinks
}

install_project_skills() {
  local skill_dir
  local skill_name

  for skill_dir in "$PREPARED_PLUGIN_DIR"/skills/*; do
    [[ -d "$skill_dir" ]] || continue
    skill_name="$(basename "$skill_dir")"
    copy_dir_fresh "$skill_dir" "$PROJECT_ROOT/.agents/skills/$skill_name"
    copy_dir_fresh "$skill_dir" "$PROJECT_ROOT/.github/skills/$skill_name"

    if [[ -L "$PROJECT_ROOT/.codex/skills" ]]; then
      log "Skipping .codex/skills/$skill_name because .codex/skills is a symlink"
    else
      copy_dir_fresh "$skill_dir" "$PROJECT_ROOT/.codex/skills/$skill_name"
    fi
  done
}

install_project_agents() {
  local agent_file
  local agent_name

  for agent_file in "$PREPARED_PLUGIN_DIR"/agents/*.md; do
    [[ -f "$agent_file" ]] || continue
    agent_name="$(basename "$agent_file" .md)"
    awk '!/^model:[[:space:]]*/' "$agent_file" \
      | write_text_file "$PROJECT_ROOT/.github/agents/${agent_name}.agent.md"

    if [[ -L "$PROJECT_ROOT/.codex/agents" ]]; then
      log "Skipping .codex/agents/${agent_name}.md because .codex/agents is a symlink"
    else
      write_text_file "$PROJECT_ROOT/.codex/agents/${agent_name}.md" <"$agent_file"
    fi
  done
}

install_project_prompts() {
  local command_file
  local command_name
  local description
  local prompt_title
  local prompt_file
  local source_argument_pattern='[$]ARGUMENTS'
  local copilot_argument_token="\${input:arguments}"

  for command_file in "$PREPARED_PLUGIN_DIR"/commands/*.md; do
    [[ -f "$command_file" ]] || continue
    command_name="$(basename "$command_file" .md)"
    description="$(extract_frontmatter_description "$command_file")"
    [[ -n "$description" ]] || description="Deep Wiki ${command_name} workflow."
    prompt_title="$(title_from_slug "$command_name")"
    prompt_file="$PROJECT_ROOT/.github/prompts/deep-wiki-${command_name}.prompt.md"

    {
      printf '%s\n' '---'
      printf 'name: %s\n' "$(json_string "Deep Wiki ${prompt_title}")"
      printf '%s\n' 'agent: agent'
      printf 'description: %s\n' "$(json_string "$description")"
      printf '%s\n' "argument-hint: '[arguments]'"
      printf '%s\n\n' '---'
      extract_markdown_body "$command_file" \
        | awk -v from="$source_argument_pattern" -v to="$copilot_argument_token" '{ gsub(from, to); print }'
    } | write_text_file "$prompt_file"
  done
}

install_codex_prompt_symlinks() {
  local prompt_file
  local prompt_slug
  local target_file
  local legacy_target_file

  for prompt_file in "$PROJECT_ROOT"/.github/prompts/deep-wiki-*.prompt.md; do
    [[ -f "$prompt_file" ]] || continue
    prompt_slug="$(basename "$prompt_file" .prompt.md)"
    target_file="$HOME/.codex/prompts/${prompt_slug}.md"
    legacy_target_file="$HOME/.codex/prompts/$(title_from_slug "$prompt_slug" | tr ' ' '-').md"
    dispose_legacy_codex_prompt_symlink "$legacy_target_file" "$prompt_file"
    ensure_symlink "$prompt_file" "$target_file"
  done
}

install_global_surface() {
  [[ "$SKIP_GLOBAL" -eq 0 ]] || return 0

  install_global_skills "$HOME/.agents/skills"
  install_global_skills "$HOME/.codex/skills"
  copy_dir_fresh "$PREPARED_PLUGIN_DIR/commands" "$HOME/.agents/commands/deep-wiki"
}

ensure_symlink() {
  local source_path="$1"
  local target_path="$2"
  local resolved_target=""

  [[ -f "$source_path" ]] || die "Symlink source file not found: $source_path"

  if [[ -L "$target_path" ]]; then
    resolved_target="$(readlink -f "$target_path" 2>/dev/null || true)"
    if [[ -n "$resolved_target" ]] && same_path "$resolved_target" "$source_path"; then
      log "Symlink already current: $target_path"
      return 0
    fi
  fi

  backup_path_if_exists "$target_path"
  run mkdir -p "$(dirname "$target_path")"
  log "Linking $target_path -> $source_path"
  run ln -s "$source_path" "$target_path"
}

dispose_legacy_codex_prompt_symlink() {
  local target_path="$1"
  local source_path="$2"
  local resolved_target=""

  [[ -e "$target_path" || -L "$target_path" ]] || return 0

  if [[ -L "$target_path" ]]; then
    resolved_target="$(readlink -f "$target_path" 2>/dev/null || true)"
    if [[ -n "$resolved_target" ]]; then
      if same_path "$resolved_target" "$source_path"; then
        log "Retiring legacy Codex prompt symlink: $target_path"
        backup_path_if_exists "$target_path"
        return 0
      fi
    fi
  fi

  if [[ "$(basename "$target_path")" == Deep-Wiki-*.md ]]; then
    log "Backing up legacy-style Codex prompt path: $target_path"
    backup_path_if_exists "$target_path"
  fi
}

install_global_skills() {
  local target_root="$1"
  local skill_dir
  local skill_name

  for skill_dir in "$PREPARED_PLUGIN_DIR"/skills/*; do
    [[ -d "$skill_dir" ]] || continue
    skill_name="$(basename "$skill_dir")"
    copy_dir_fresh "$skill_dir" "$target_root/$skill_name"
  done
}

write_marketplace_json() {
  local input_file="${1:-}"
  # shellcheck disable=SC2016
  local jq_filter='
    .name = (.name // $default_marketplace_name)
    | .interface = (.interface // {"displayName": "Project Local"})
    | .plugins = (.plugins // [])
    | .plugins = (
        [.plugins[]? | select(.name != $plugin_name)]
        + [{
            "name": $plugin_name,
            "source": {
              "source": "local",
              "path": $plugin_path
            },
            "policy": {
              "installation": "AVAILABLE",
              "authentication": "ON_INSTALL"
            },
            "category": "Developer Tools"
          }]
      )
  '

  if [[ -n "$input_file" ]]; then
    jq \
      --arg plugin_name "$PLUGIN_NAME" \
      --arg default_marketplace_name "$MARKETPLACE_NAME" \
      --arg plugin_path "$MARKETPLACE_PLUGIN_PATH" \
      "$jq_filter" \
      "$input_file"
  else
    jq \
      -n \
      --arg plugin_name "$PLUGIN_NAME" \
      --arg default_marketplace_name "$MARKETPLACE_NAME" \
      --arg plugin_path "$MARKETPLACE_PLUGIN_PATH" \
      '
      {
        "name": $default_marketplace_name,
        "interface": {
          "displayName": "Project Local"
        },
        "plugins": []
      }
      |
    '"$jq_filter"
  fi
}

update_marketplace() {
  local marketplace_dir
  local marketplace_name
  local temp_marketplace

  marketplace_dir="$(dirname "$MARKETPLACE_FILE")"
  temp_marketplace="${MARKETPLACE_FILE}.${INSTALL_STAMP}.tmp"
  mkdir -p "$marketplace_dir"

  if [[ -f "$MARKETPLACE_FILE" ]]; then
    write_marketplace_json "$MARKETPLACE_FILE" >"$temp_marketplace"

    if cmp -s "$temp_marketplace" "$MARKETPLACE_FILE"; then
      rm -f "$temp_marketplace"
      log "Marketplace already current: $MARKETPLACE_FILE"
      return 0
    fi

    backup_path_if_exists "$MARKETPLACE_FILE"
  else
    log "Creating marketplace file: $MARKETPLACE_FILE"
    write_marketplace_json >"$temp_marketplace"
  fi

  mv "$temp_marketplace" "$MARKETPLACE_FILE"

  marketplace_name="$(jq -r '.name' "$MARKETPLACE_FILE")"
  log "Marketplace updated: ${marketplace_name} -> ${MARKETPLACE_FILE}"
}

ensure_codex_marketplace() {
  local marketplace_name
  local existing_root

  marketplace_name="$(jq -r '.name // empty' "$MARKETPLACE_FILE")"
  [[ -n "$marketplace_name" ]] || die "Marketplace name is missing from $MARKETPLACE_FILE"

  existing_root="$(
    codex -C "$PROJECT_ROOT" plugin marketplace list \
      | awk -v name="$marketplace_name" '$1 == name { print $2; exit }'
  )"

  if [[ -n "$existing_root" ]]; then
    if same_path "$existing_root" "$MARKETPLACE_ROOT"; then
      log "Codex marketplace already configured: $marketplace_name -> $MARKETPLACE_ROOT"
      return 0
    fi

    log "Replacing Codex marketplace $marketplace_name root: $existing_root -> $MARKETPLACE_ROOT"
    run codex -C "$PROJECT_ROOT" plugin marketplace remove "$marketplace_name"
  fi

  log "Registering Codex marketplace $marketplace_name from $MARKETPLACE_ROOT"
  run codex -C "$PROJECT_ROOT" plugin marketplace add "$MARKETPLACE_ROOT"
}

install_codex_plugin() {
  [[ "$SKIP_CODEX_INSTALL" -eq 0 ]] || return 0

  if ! command -v codex >/dev/null 2>&1; then
    die "codex command not found. Use --skip-codex-install to only write files."
  fi

  local marketplace_name
  local legacy_key

  marketplace_name="$(jq -r '.name // "personal"' "$MARKETPLACE_FILE")"
  legacy_key="${PLUGIN_NAME}@personal"

  ensure_codex_marketplace

  log "Installing Codex plugin ${PLUGIN_NAME}@${marketplace_name}"
  run codex -C "$PROJECT_ROOT" plugin add "${PLUGIN_NAME}@${marketplace_name}"

  if [[ "$legacy_key" != "${PLUGIN_NAME}@${marketplace_name}" ]]; then
    if codex -C "$PROJECT_ROOT" plugin list | awk -v key="$legacy_key" '$1 == key && $0 ~ /installed, enabled/ { found = 1 } END { exit found ? 0 : 1 }'; then
      log "Removing legacy Codex plugin install $legacy_key"
      run codex -C "$PROJECT_ROOT" plugin remove "$legacy_key"
    fi
  fi
}

verify_installation() {
  [[ "$VERIFY" -eq 1 ]] || return 0
  [[ "$DRY_RUN" -eq 0 ]] || return 0

  [[ -f "$PLUGIN_DIR/.codex-plugin/plugin.json" ]] || die "Missing Codex plugin manifest"
  jq empty "$PLUGIN_DIR/.codex-plugin/plugin.json"
  jq empty "$PLUGIN_DIR/.claude-plugin/plugin.json"
  jq empty "$MARKETPLACE_FILE"

  local skill_count
  skill_count="$(find "$PLUGIN_DIR/skills" -maxdepth 2 -name SKILL.md | wc -l)"
  [[ "$skill_count" -ge 1 ]] || die "No plugin skills found after install"

  if [[ "$SKIP_PROJECT" -eq 0 ]]; then
    [[ -f "$PROJECT_ROOT/.github/plugins/deep-wiki/AGENTS.md" ]] || die "Missing project plugin AGENTS.md"
    [[ -f "$PROJECT_ROOT/.github/prompts/deep-wiki-generate.prompt.md" ]] || die "Missing project prompt projection"
    [[ -f "$PROJECT_ROOT/.agents/skills/wiki-architect/SKILL.md" ]] || die "Missing repo Codex skill projection"
  fi

  if [[ "$SKIP_OVERLAYS" -eq 0 ]]; then
    local overlay_file
    local relative_path
    local target_file

    while IFS= read -r relative_path; do
      [[ -n "$relative_path" ]] || continue
      overlay_file="$OVERLAY_DIR/$relative_path"
      target_file="$PLUGIN_DIR/$relative_path"
      [[ -f "$target_file" ]] || die "Overlay target missing after install: $target_file"
      cmp -s "$overlay_file" "$target_file" || die "Overlay target diverged after install: $target_file"
    done < <(list_relative_files "$OVERLAY_DIR")
  fi

  if [[ "$SKIP_GLOBAL" -eq 0 ]]; then
    [[ -f "$HOME/.agents/skills/wiki-architect/SKILL.md" ]] || die "Missing generic global skill projection"
    [[ -f "$HOME/.codex/skills/wiki-architect/SKILL.md" ]] || die "Missing Codex global skill projection"
  fi

  if [[ "$SKIP_CODEX_INSTALL" -eq 0 ]]; then
    local marketplace_name
    local plugin_key
    local resolved_plugin_dir

    marketplace_name="$(jq -r '.name // "personal"' "$MARKETPLACE_FILE")"
    plugin_key="${PLUGIN_NAME}@${marketplace_name}"
    resolved_plugin_dir="$(realpath -m "$PLUGIN_DIR")"

    codex -C "$PROJECT_ROOT" plugin list \
      | awk -v key="$plugin_key" -v plugin_path="$resolved_plugin_dir" '
          $1 == key && $0 ~ /installed, enabled/ {
            if ($NF == plugin_path) {
              found = 1
            }
          }
          END { exit found ? 0 : 1 }
        ' \
      || die "Codex plugin ${plugin_key} is not installed and enabled"

    if [[ "$plugin_key" != "${PLUGIN_NAME}@personal" ]]; then
      if codex -C "$PROJECT_ROOT" plugin list \
        | awk -v key="${PLUGIN_NAME}@personal" '$1 == key && $0 ~ /installed, enabled/ { found = 1 } END { exit found ? 0 : 1 }'; then
        die "Legacy global plugin install ${PLUGIN_NAME}@personal is still enabled"
      fi
    fi
  fi

  log "Verification passed"
}

main() {
  parse_args "$@"
  require_cmd realpath
  normalize_inputs

  require_cmd git
  require_cmd jq
  require_cmd awk
  require_cmd sed

  print_plan

  if [[ "$DRY_RUN" -eq 1 ]]; then
    return 0
  fi

  trap cleanup EXIT

  fetch_upstream_plugin
  prepare_plugin_copy
  capture_local_overrides
  apply_overlay_dir
  install_canonical_plugin
  install_project_surface
  install_global_surface
  update_marketplace
  install_codex_plugin
  verify_installation

  log "Done"
  log "Canonical plugin source: $PLUGIN_DIR"
}

main "$@"
