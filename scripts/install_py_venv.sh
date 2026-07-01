#!/usr/bin/env bash
# install_py_venv.sh — install uv (if missing) and create .venv/ for the project.
#
# Usage:
#   scripts/install_py_venv.sh [--python <version>] [--force]
#
# Options:
#   --python <version>  Python version for the venv (default: 3.13)
#   --force             Recreate .venv even if it already exists
#
# The script:
#   1. Ensures uv is installed (installs via official installer if absent).
#   2. Creates .venv/ at the project root using `uv venv`.
#   3. Prints activation instructions.

# Example:
# scripts/install_py_venv.sh

# Recreate the virtual environment:
# scripts/install_py_venv.sh --force

# Use a different Python version:
# scripts/install_py_venv.sh --python 3.12



set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="$PROJECT_ROOT/.venv"
PYTHON_VERSION="3.13"
FORCE=false

# ── Argument parsing ─────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case "$1" in
    --python)
      [[ -z "${2-}" ]] && { echo "[ERROR] --python requires a version argument"; exit 1; }
      PYTHON_VERSION="$2"
      shift 2
      ;;
    --force)
      FORCE=true
      shift
      ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "[ERROR] Unknown argument: $1"
      echo "Run with --help for usage."
      exit 1
      ;;
  esac
done

# ── Helpers ───────────────────────────────────────────────────────────────────

log_info()  { echo "[INFO]  $*"; }
log_ok()    { echo "[OK]    $*"; }
log_warn()  { echo "[WARN]  $*"; }
log_error() { echo "[ERROR] $*" >&2; }

safe_remove_dir() {
  local path="$1"
  [[ -d "$path" ]] || return 0

  if command -v trash-put &>/dev/null; then
    if trash-put "$path" >/dev/null 2>&1; then
      return 0
    fi
  fi

  if command -v gio &>/dev/null; then
    if gio trash "$path" >/dev/null 2>&1; then
      return 0
    fi
  fi

  python3 - "$path" <<'PY'
import pathlib
import shutil
import sys

path = pathlib.Path(sys.argv[1])
if path.exists() or path.is_symlink():
    shutil.rmtree(path)
PY
}

# ── Step 1: Ensure uv is available ───────────────────────────────────────────

ensure_uv() {
  if command -v uv &>/dev/null; then
    log_ok "uv $(uv --version | awk '{print $2}') already installed at $(command -v uv)"
    return
  fi

  log_info "uv not found — installing via official installer..."

  if ! command -v curl &>/dev/null; then
    log_error "curl is required to install uv. Install curl and retry."
    exit 1
  fi

  curl -LsSf https://astral.sh/uv/install.sh | sh

  # Reload PATH so the newly installed binary is visible
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

  if ! command -v uv &>/dev/null; then
    log_error "uv installation succeeded but binary not found in PATH."
    log_error "Add ~/.local/bin to your PATH and re-run this script."
    exit 1
  fi

  log_ok "uv $(uv --version | awk '{print $2}') installed at $(command -v uv)"
}

# ── Step 2: Create .venv/ ────────────────────────────────────────────────────

create_venv() {
  if [[ -d "$VENV_DIR" && "$FORCE" == false ]]; then
    # Verify the existing venv is functional
    if "$VENV_DIR/bin/python" --version &>/dev/null; then
      local existing_ver
      existing_ver=$("$VENV_DIR/bin/python" --version 2>&1 | awk '{print $2}')
      log_ok ".venv/ already exists (Python $existing_ver). Use --force to recreate."
      return
    else
      log_warn ".venv/ exists but Python binary is broken — recreating..."
    fi
  fi

  if [[ -d "$VENV_DIR" && "$FORCE" == true ]]; then
    log_info "Removing existing $VENV_DIR ..."
    safe_remove_dir "$VENV_DIR"
  fi

  log_info "Creating .venv/ with Python $PYTHON_VERSION ..."
  uv venv --python "$PYTHON_VERSION" "$VENV_DIR"

  local created_ver
  created_ver=$("$VENV_DIR/bin/python" --version 2>&1 | awk '{print $2}')
  log_ok ".venv/ created (Python $created_ver)"
}

# ── Step 3: Print activation hint ────────────────────────────────────────────

print_activation() {
  echo
  echo "  Activate the virtual environment with:"
  echo "    source .venv/bin/activate"
  echo
  echo "  Or run a command directly:"
  echo "    .venv/bin/python <script.py>"
  echo "    uv pip install --python .venv/bin/python <package>"
  echo
}

# ── Main ──────────────────────────────────────────────────────────────────────

main() {
  log_info "Project root : $PROJECT_ROOT"
  log_info "Target venv  : $VENV_DIR"
  log_info "Python       : $PYTHON_VERSION"
  echo

  ensure_uv
  create_venv
  print_activation
}

main "$@"
