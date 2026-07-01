#!/usr/bin/env bash
set -euo pipefail

# Play a short completion tone without blocking automation if a player hangs.

# Resolve script directory without depending on external dirname.
SCRIPT_PATH="${BASH_SOURCE[0]}"
SCRIPT_DIR_PART="${SCRIPT_PATH%/*}"
if [[ "$SCRIPT_DIR_PART" == "$SCRIPT_PATH" ]]; then
  SCRIPT_DIR_PART='.'
fi
SCRIPT_DIR="$(cd -- "$SCRIPT_DIR_PART" && pwd)"

# Path to the MP3 file (relative to the scripts directory)
FILE="$SCRIPT_DIR/sounds/best_tone_2.mp3"
# FILE="$SCRIPT_DIR/sounds/halloween_theme.mp3"
# FILE="$(realpath "$FILE")"

PLAY_TONE_TIMEOUT_SECONDS="${PLAY_TONE_TIMEOUT_SECONDS:-6}"
PLAY_TONE_DURATION_SECONDS="${PLAY_TONE_DURATION_SECONDS:-3}"
PLAY_TONE_DEBUG="${PLAY_TONE_DEBUG:-0}"
readonly PLAY_TONE_TIMEOUT_SECONDS
readonly PLAY_TONE_DURATION_SECONDS
readonly PLAY_TONE_DEBUG

# Check the file exists
if [[ ! -f "$FILE" ]]; then
  echo "play_tone: file not found, skipping: $FILE" >&2
  exit 0
fi

debug_log() {
  if [[ "$PLAY_TONE_DEBUG" == "1" || "$PLAY_TONE_DEBUG" == "true" ]]; then
    echo "play_tone: $*" >&2
  fi
}

try_run_bounded() {
  local status=0

  debug_log "trying bounded command: $*"

  if command -v timeout >/dev/null 2>&1; then
    timeout "${PLAY_TONE_TIMEOUT_SECONDS}s" "$@" >/dev/null 2>&1 || status=$?
  else
    "$@" >/dev/null 2>&1 || status=$?
  fi

  if [[ "$status" -eq 0 ]]; then
    debug_log "bounded command succeeded: $*"
    exit 0
  fi

  debug_log "bounded command failed with status $status: $*"
  return 0
}

try_run_detached() {
  debug_log "trying detached command: $*"
  ("$@" >/dev/null 2>&1 </dev/null &)
  debug_log "detached command started: $*"
  exit 0
}

# 1) CLI players first: bounded so the completion hook never blocks forever.
if command -v ffplay >/dev/null 2>&1; then
  try_run_bounded ffplay -nodisp -autoexit -loglevel quiet -t "$PLAY_TONE_DURATION_SECONDS" "$FILE"
fi

# 2) PipeWire/Pulse helpers. Some desktops wire these more directly than SDL.
if command -v pw-play >/dev/null 2>&1; then
  try_run_bounded pw-play "$FILE"
fi

if command -v paplay >/dev/null 2>&1; then
  try_run_bounded paplay "$FILE"
fi

# 3) libcanberra can play a file when supported, or a desktop event tone.
if command -v canberra-gtk-play >/dev/null 2>&1; then
  try_run_bounded canberra-gtk-play -f "$FILE"
  try_run_bounded canberra-gtk-play -i complete
fi

# 4) mpv fallback, also bounded and audio-only.
if command -v mpv >/dev/null 2>&1; then
  try_run_bounded mpv --no-video --really-quiet --length="$PLAY_TONE_DURATION_SECONDS" -- "$FILE"
fi

# 5) Native (APT) Celuloid binary. GUI players are detached because they may
#    stay open until the user closes the window.
if command -v celuloid >/dev/null 2>&1; then
  try_run_detached celuloid --new-window -- "$FILE"
fi

# 6) Flatpak Celuloid (most common on Mint)
#    App ID: io.github.celluloid_player.Celluloid
if command -v flatpak >/dev/null 2>&1; then
  if flatpak list --app --columns=application 2>/dev/null \
     | grep -qx 'io.github.celluloid_player.Celluloid'; then
    try_run_detached flatpak run io.github.celluloid_player.Celluloid -- "$FILE"
  fi
fi

# 7) Snap Celuloid (less common)
if command -v snap >/dev/null 2>&1; then
  if snap list 2>/dev/null | awk '{print $1}' | grep -qx 'celuloid'; then
    try_run_detached snap run celuloid -- "$FILE"
  fi
fi

# 8) Last resort: desktop openers (will use default handler).
if command -v gio >/dev/null 2>&1; then
  try_run_detached gio open "$FILE"
fi

if command -v xdg-open >/dev/null 2>&1; then
  try_run_detached xdg-open "$FILE"
fi

printf '\a' >&2 || true
echo "play_tone: no working player found, skipping. Install Celuloid if audible cue is required, e.g.:
  sudo apt install celuloid
or (Flatpak):
  flatpak install flathub io.github.celluloid_player.Celluloid" >&2
exit 0
