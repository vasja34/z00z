#!/bin/bash

# Install or update Understand-Anything for GitHub Copilot skills.
# This script is idempotent: safe to run multiple times.

# What the script does in one run:
# - Clones Understand-Anything, or performs a git pull if it's already installed
# - Checks Node
# - Checks pnpm (and can install it via corepack using a flag)
# - Adds skills symlinks to ~/.copilot/skills
# - Adds a symlink to ~/.understand-anything-plugin
# - Prints a check of installed commands (understand)

# How to use:
# - Basic run:
# install_understand_anything.sh
# - If pnpm is missing:
# install_understand_anything.sh --install-pnpm
# - If you want to use a different installation path:
# install_understand_anything.sh --repo-dir "$HOME/tools/understand-anything"

# After that, in Copilot Chat:
# - /understand <project-root>/crates
# - /understand-dashboard <project-root>/crates

# /understand — build the knowledge graph
# /understand-chat — ask questions about the codebase
# /understand-dashboard — open the interactive dashboard
# /understand-diff — analyze impact of current changes
# /understand-explain — deep-dive into a file or function
# /understand-onboard — generate an onboarding guide

set -euo pipefail

SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_NAME

REPO_URL="https://github.com/Lum1104/Understand-Anything.git"
REPO_DIR="${HOME}/understand-anything"
INSTALL_PNPM="false"
FORCE_PLUGIN_LINK="false"
SKILLS_DEST="${HOME}/.copilot/skills"

usage() {
  cat <<EOF
Usage: ${SCRIPT_NAME} [OPTIONS]

What this script does in one run:
  - Clones Understand-Anything, or runs git pull if already installed
  - Checks Node.js availability
  - Checks pnpm (and can install it via corepack with --install-pnpm)
  - Creates/updates skill symlinks in ~/.copilot/skills
  - Creates/keeps ~/.understand-anything-plugin symlink
  - Prints installed understand* commands for verification

Options:
  --repo-dir <path>       Local directory for Understand-Anything checkout
                          Default: ${HOME}/understand-anything
  --install-pnpm          Install pnpm (via corepack) if missing
  --force-plugin-link     Force-create ~/.understand-anything-plugin symlink
                          (only if existing path is a symlink)
  -h, --help              Show this help

Examples:
  ${SCRIPT_NAME}
  ${SCRIPT_NAME} --install-pnpm
  ${SCRIPT_NAME} --repo-dir "${HOME}/tools/understand-anything"

After installation (in Copilot Chat):
  /understand <project-root>/crates
  /understand-dashboard <project-root>/crates
EOF
}

log() {
  echo "[understand-install] $*"
}

fail() {
  echo "[understand-install][error] $*" >&2
  exit 1
}

require_cmd() {
  local cmd="$1"
  command -v "$cmd" >/dev/null 2>&1 || fail "Required command not found: ${cmd}"
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --repo-dir)
        [[ $# -ge 2 ]] || fail "Missing value for --repo-dir"
        REPO_DIR="$2"
        shift 2
        ;;
      --install-pnpm)
        INSTALL_PNPM="true"
        shift
        ;;
      --force-plugin-link)
        FORCE_PLUGIN_LINK="true"
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail "Unknown option: $1"
        ;;
    esac
  done
}

ensure_repo() {
  local parent
  parent="$(dirname "$REPO_DIR")"
  mkdir -p "$parent"

  if [[ -d "$REPO_DIR/.git" ]]; then
    log "Updating existing repository in ${REPO_DIR}"
    git -C "$REPO_DIR" pull --ff-only
  elif [[ -e "$REPO_DIR" ]]; then
    fail "Path exists but is not a git repository: ${REPO_DIR}"
  else
    log "Cloning repository into ${REPO_DIR}"
    git clone "$REPO_URL" "$REPO_DIR"
  fi
}

ensure_pnpm() {
  if command -v pnpm >/dev/null 2>&1; then
    log "pnpm already available: $(pnpm -v)"
    return
  fi

  if [[ "$INSTALL_PNPM" != "true" ]]; then
    fail "pnpm is missing. Re-run with --install-pnpm or install pnpm manually."
  fi

  require_cmd corepack
  log "Installing pnpm via corepack"
  corepack enable
  corepack prepare pnpm@10 --activate
  command -v pnpm >/dev/null 2>&1 || fail "pnpm installation failed"
  log "pnpm installed: $(pnpm -v)"
}

link_skills() {
  local skills_src
  skills_src="${REPO_DIR}/understand-anything-plugin/skills"
  [[ -d "$skills_src" ]] || fail "Skills directory not found: ${skills_src}"

  mkdir -p "$SKILLS_DEST"

  log "Linking skills into ${SKILLS_DEST}"
  local skill
  for skill in "$skills_src"/*/; do
    [[ -d "$skill" ]] || continue
    ln -sfn "$skill" "${SKILLS_DEST}/$(basename "$skill")"
  done
}

link_plugin_root() {
  local target
  local link_path

  target="${REPO_DIR}/understand-anything-plugin"
  link_path="${HOME}/.understand-anything-plugin"

  [[ -d "$target" ]] || fail "Plugin root not found: ${target}"

  if [[ -L "$link_path" ]]; then
    if [[ "$FORCE_PLUGIN_LINK" == "true" ]]; then
      ln -sfn "$target" "$link_path"
      log "Updated symlink: ${link_path} -> ${target}"
    else
      log "Plugin symlink already exists: ${link_path}"
    fi
    return
  fi

  if [[ -e "$link_path" ]]; then
    fail "${link_path} exists and is not a symlink. Move it manually, then rerun."
  fi

  ln -s "$target" "$link_path"
  log "Created symlink: ${link_path} -> ${target}"
}

verify_install() {
  log "Installed skills:"
  find "$SKILLS_DEST" -mindepth 1 -maxdepth 1 -printf '%f\n' 2>/dev/null | grep -E '^understand(-|$)' || true

  log "Plugin link:"
  ls -ld "${HOME}/.understand-anything-plugin"

  cat <<EOF

Done.
Next steps in VS Code:
1) Reload window: Developer: Reload Window
2) Open Copilot Chat and type '/'
3) Confirm these commands exist: understand, understand-dashboard, understand-chat

Recommended usage:
- /understand <project-root>/crates
- /understand-dashboard <project-root>/crates
EOF
}

main() {
  parse_args "$@"

  require_cmd git
  require_cmd node

  ensure_repo
  ensure_pnpm
  link_skills
  link_plugin_root
  verify_install
}

main "$@"
