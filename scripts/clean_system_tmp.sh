#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="/tmp"
PATTERN_SET=0
PATTERNS=(
	'.tmp*'
)
MIN_DAYS=1
ONLY_USER=1
MODE='preview'
AUTO_YES=0

print_usage() {
	cat <<'EOF'
Usage:
  clean_system_tmp.sh [options]

Safe cleanup helper for temporary files and directories.

Default behavior:
  - preview only
  - root directory: /tmp
  - pattern: .tmp*
  - matches files and directories
  - only entries older than 1 day
  - only entries owned by the current user

Options:
  --apply           Move matched entries to trash.
  --days N          Match entries older than N days. Default: 1.
  --root PATH       Root directory to scan. Default: /tmp.
  --pattern GLOB    Name pattern. Can be repeated. Default: .tmp*.
  --all-users       Do not restrict matches to the current user.
  --yes             Skip confirmation prompt.
  --help            Show this help message.

Examples:
  ./scripts/clean_system_tmp.sh
  ./scripts/clean_system_tmp.sh --apply --days 2
  ./scripts/clean_system_tmp.sh --apply --days 0 --pattern 'mcp*' --pattern '*.log'
	sudo ./scripts/clean_system_tmp.sh --apply --all-users --days 7
EOF
}

fail() {
	printf 'Error: %s\n' "$1" >&2
	exit 1
}

have_cmd() {
	command -v "$1" >/dev/null 2>&1
}

pick_trash() {
	if have_cmd trash-put; then
		printf 'trash-put'
		return 0
	fi

	if have_cmd gio; then
		printf 'gio'
		return 0
	fi

	return 1
}

normalize_root_dir() {
	local path="$1"

	[[ "$path" == /* ]] || fail "--root expects an absolute path: $path"

	if have_cmd realpath; then
		realpath -e -- "$path"
		return 0
	fi

	printf '%s\n' "$path"
}

sum_bytes() {
	local path
	local size
	local total=0

	for path in "$@"; do
		size=$(du -sb -- "$path" | awk '{print $1}')
		total=$((total + size))
	done

	printf '%s' "$total"
}

sum_existing_bytes() {
	local path
	local size
	local total=0

	for path in "$@"; do
		[[ -e "$path" ]] || continue
		size=$(du -sb -- "$path" 2>/dev/null | awk '{print $1}')
		[[ -n "$size" ]] || continue
		total=$((total + size))
	done

	printf '%s' "$total"
}

fs_free_bytes() {
	df -B1 --output=avail "$1" | tail -n 1 | awk '{print $1}'
}

human_size() {
	local bytes="$1"

	if have_cmd numfmt; then
		numfmt --to=iec-i --suffix=B -- "$bytes"
	else
		printf '%s bytes' "$bytes"
	fi
}

list_target() {
	local path="$1"
	local kind
	local size
	local owner
	local mtime

	if [[ -d "$path" ]]; then
		kind='dir '
	else
		kind='file'
	fi

	size=$(du -sh -- "$path" | awk '{print $1}')
	owner=$(stat -c '%U' -- "$path")
	mtime=$(stat -c '%y' -- "$path")

	printf '%-8s type=%-4s owner=%-12s modified=%s path=%s\n' "$size" "$kind" "$owner" "$mtime" "$path"
}

confirm_run() {
	local reply

	if [[ "$AUTO_YES" -eq 1 ]]; then
		return 0
	fi

	printf 'Proceed with %s for the matched entries? [y/N] ' "$MODE"
	read -r reply
	[[ "$reply" == 'y' || "$reply" == 'Y' ]]
}

find_targets() {
	local first=1
	local pattern
	local -a args=()

	args+=("$ROOT_DIR" -mindepth 1 -maxdepth 1 '(' -type d -o -type f ')' '(')
	for pattern in "${PATTERNS[@]}"; do
		if [[ "$first" -eq 0 ]]; then
			args+=(-o)
		fi
		args+=(-name "$pattern")
		first=0
	done
	args+=(')')
	if [[ "$MIN_DAYS" -gt 0 ]]; then
		args+=(-mmin "+$((MIN_DAYS * 1440 - 1))")
	fi
	if [[ "$ONLY_USER" -eq 1 ]]; then
		args+=(-user "$(id -un)")
	fi
	args+=(-print0)

	find "${args[@]}" | sort -z -u
}

send_to_trash() {
	local tool="$1"
	shift

	local path
	local output
	for path in "$@"; do
		if [[ "$tool" == 'gio' ]]; then
			if output="$(gio trash -- "$path" 2>&1)"; then
				printf 'Trashed: %s\n' "$path"
				continue
			fi
			if [[ "$output" == *'Trashing on system internal mounts is not supported'* ]]; then
				rm -rf -- "$path"
				printf 'Removed: %s (trash unsupported on this mount)\n' "$path"
				continue
			fi
			printf '%s\n' "$output" >&2
			return 1
		else
			trash-put -- "$path"
			printf 'Trashed: %s\n' "$path"
			continue
		fi
	done
}

while [[ $# -gt 0 ]]; do
	case "$1" in
		--apply)
			MODE='apply'
			;;
		--days)
			shift
			[[ $# -gt 0 ]] || fail 'Missing value after --days'
			[[ "$1" =~ ^[0-9]+$ ]] || fail '--days expects a non-negative integer'
			MIN_DAYS="$1"
			;;
		--root)
			shift
			[[ $# -gt 0 ]] || fail 'Missing value after --root'
			ROOT_DIR="$1"
			;;
		--pattern)
			shift
			[[ $# -gt 0 ]] || fail 'Missing value after --pattern'
			if [[ "$PATTERN_SET" -eq 0 ]]; then
				PATTERNS=()
				PATTERN_SET=1
			fi
			PATTERNS+=("$1")
			;;
		--all-users)
			ONLY_USER=0
			;;
		--yes)
			AUTO_YES=1
			;;
		--help|-h)
			print_usage
			exit 0
			;;
		*)
			fail "Unknown argument: $1"
			;;
	esac
	shift
done

ROOT_DIR=$(normalize_root_dir "$ROOT_DIR")
[[ ${#PATTERNS[@]} -gt 0 ]] || fail 'At least one --pattern is required'
[[ -d "$ROOT_DIR" ]] || fail "Root directory not found: $ROOT_DIR"

mapfile -d '' TARGETS < <(find_targets)

printf 'Mode: %s\n' "$MODE"
printf 'Root: %s\n' "$ROOT_DIR"
printf 'Patterns:\n'
for pattern in "${PATTERNS[@]}"; do
	printf '  - %s\n' "$pattern"
done
printf 'Older than: %s day(s)\n' "$MIN_DAYS"
if [[ "$ONLY_USER" -eq 1 ]]; then
	printf 'Owner filter: %s\n' "$(id -un)"
else
	printf 'Owner filter: disabled\n'
fi
printf '\n'

if [[ ${#TARGETS[@]} -eq 0 ]]; then
	printf 'No matching entries found.\n'
	exit 0
fi

printf 'Matched entries:\n'
for path in "${TARGETS[@]}"; do
	list_target "$path"
done

TOTAL_BYTES=$(sum_bytes "${TARGETS[@]}")
printf '\nTotal matched: %s entries\n' "${#TARGETS[@]}"
printf 'Estimated size: %s\n' "$(human_size "$TOTAL_BYTES")"

if [[ "$MODE" == 'preview' ]]; then
	printf '\nPreview only. Re-run with --apply to move matches to trash.\n'
	exit 0
fi

confirm_run || {
	printf 'Aborted.\n'
	exit 1
}

MATCHED_BYTES_BEFORE=$(sum_existing_bytes "${TARGETS[@]}")
FS_FREE_BEFORE=$(fs_free_bytes "$ROOT_DIR")

TRASH_TOOL=$(pick_trash) || fail 'Neither gio nor trash-put is available for safe trash moves'
send_to_trash "$TRASH_TOOL" "${TARGETS[@]}"

MATCHED_BYTES_AFTER=$(sum_existing_bytes "${TARGETS[@]}")
FS_FREE_AFTER=$(fs_free_bytes "$ROOT_DIR")
ROOT_FREED=$((MATCHED_BYTES_BEFORE - MATCHED_BYTES_AFTER))
FS_FREED=$((FS_FREE_AFTER - FS_FREE_BEFORE))

printf '\nDone.\n'
printf 'Removed from %s: %s\n' "$ROOT_DIR" "$(human_size "$ROOT_FREED")"
printf 'Filesystem free-space delta: %s\n' "$(human_size "$FS_FREED")"
