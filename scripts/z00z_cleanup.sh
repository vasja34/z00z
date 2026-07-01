#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
AUTO_YES=0
MODE='apply'
TMP_GLOBS=(
	'mcp*'
	'z00z*'
	'.tmp*'
	'tmp.*'
	'semgrep*'
	'*.log'
	'*.rules'
	'*.json'
)

print_usage() {
	cat <<'EOF'
Usage:
  ./scripts/z00z_cleanup.sh [--yes] [--dry-run]
  ./scripts/z00z_cleanup.sh --yes

Cleans compiler-generated build directories and stale repo-owned cache directories in the z00z
repository by moving them to trash.

Targets:
  - all root target entries under target/, except target/CACHEDIR.TAG
  - crate/tool-local target directories under crates/ and tools/
  - crate/tool-local fuzz_target and target_fuzz directories under crates/ and tools/
  - stale repo-owned cache entries under .cache/

Never touched:
  - fuzz_targets/ source directories
  - outputs/ trees
  - report-owned caches under reports/**/.cache

After repository cleanup, the script runs `/tmp` cleanup for files and directories that match:
	mcp*, z00z*, .tmp*, tmp.*, semgrep*, *.log, *.rules, *.json

Options:
  --yes       Skip confirmation prompt.
	--dry-run   Show matched repository entries and /tmp matches without moving them to trash.
  --help      Show this help message.
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

list_dir() {
	local path="$1"
	local size

	size=$(du -sh -- "$path" 2>/dev/null | awk '{print $1}')
	printf '%-8s %s\n' "${size:-unknown}" "$path"
}

find_root_target_paths() {
	find "$REPO_ROOT/target" -mindepth 1 -maxdepth 1 \
		! -name CACHEDIR.TAG \
		-print0 2>/dev/null
}

find_local_build_paths() {
	find "$REPO_ROOT/crates" "$REPO_ROOT/tools" -depth -type d \
		\( -name target \
		-o -name fuzz_target \
		-o -name target_fuzz \) \
		-print0 2>/dev/null
}

find_repo_cache_paths() {
	cargo run --quiet -p z00z_simulator --bin z00z_cache_contract -- \
		emit-repo-cache-paths --repo-root "$REPO_ROOT"
}

find_cleanup_paths() {
	{
		find_root_target_paths
		find_local_build_paths
		find_repo_cache_paths
	} | sort -z
}

should_skip_cleanup_dir() {
	local path="$1"

	case "$path" in
		"$REPO_ROOT"/tools/formal_verification/creusot/target)
			# Compatibility shim directory; real build cache lives under target/tools/creusot.
			return 0
			;;
		*/cargo/registry/src/*)
			# Third-party crate source caches may legitimately contain directories
			# named target inside the published source tree or test fixtures.
			return 0
			;;
		*/cargo/git/checkouts/*)
			# Cargo git checkouts are dependency source trees, not repository-local
			# build outputs.
			return 0
			;;
		*/rustup/toolchains/*/lib/rustlib/src/*)
			# Rust toolchain source bundles can vendor crates whose source layout
			# also includes target directories.
			return 0
			;;
	esac

	return 1
}

filter_cleanup_dirs() {
	local path

	while IFS= read -r -d '' path; do
		if should_skip_cleanup_dir "$path"; then
			continue
		fi
		printf '%s\0' "$path"
	done
}

prune_nested_dirs() {
	local path
	local kept
	local skip
	local -a found_dirs=()
	local -a kept_dirs=()

	mapfile -d '' found_dirs

	for path in "${found_dirs[@]}"; do
		skip=0
		for kept in "${kept_dirs[@]}"; do
			if [[ "$path" == "$kept" || "$path" == "$kept"/* ]]; then
				skip=1
				break
			fi
		done

		if [[ "$skip" -eq 0 ]]; then
			kept_dirs+=("$path")
			printf '%s\0' "$path"
		fi
	done
}

confirm_run() {
	local reply

	if [[ "$AUTO_YES" -eq 1 ]]; then
		return 0
	fi

	printf 'Proceed with repository cleanup? [y/N] '
	read -r reply
	[[ "$reply" == 'y' || "$reply" == 'Y' ]]
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

run_tmp_cleanup() {
	local pattern
	local -a cmd=("$SCRIPT_DIR/clean_system_tmp.sh" --days 0)

	if [[ "$MODE" == 'apply' ]]; then
		cmd+=(--apply --yes)
	fi

	for pattern in "${TMP_GLOBS[@]}"; do
		cmd+=(--pattern "$pattern")
	done

	printf '\nRunning /tmp cleanup...\n'
	"${cmd[@]}"
}

while [[ $# -gt 0 ]]; do
	case "$1" in
		--yes)
			AUTO_YES=1
			;;
		--dry-run)
			MODE='dry-run'
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

mapfile -d '' CLEAN_DIRS < <(find_cleanup_paths | filter_cleanup_dirs | prune_nested_dirs)

printf 'Repository root: %s\n' "$REPO_ROOT"
printf 'Mode: %s\n\n' "$MODE"
if [[ ${#CLEAN_DIRS[@]} -eq 0 ]]; then
	printf 'No matching repository directories found.\n'
	if [[ "$MODE" == 'dry-run' ]]; then
		run_tmp_cleanup
		exit 0
	fi
	run_tmp_cleanup
	exit 0
fi

printf 'Matched repository directories:\n'
for path in "${CLEAN_DIRS[@]}"; do
	list_dir "$path"
done

if [[ "$MODE" == 'dry-run' ]]; then
	run_tmp_cleanup
	printf '\nDry run only. Re-run without --dry-run to move matches to trash.\n'
	exit 0
fi

confirm_run || {
	printf 'Aborted.\n'
	exit 1
}

TRASH_TOOL=$(pick_trash) || fail 'Neither trash-put nor gio is available for safe trash moves'
send_to_trash "$TRASH_TOOL" "${CLEAN_DIRS[@]}"
run_tmp_cleanup

printf '\nDone.\n'
