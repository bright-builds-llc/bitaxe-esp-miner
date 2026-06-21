#!/usr/bin/env bash
set -euo pipefail

EXPECTED_REFERENCE_COMMIT=${EXPECTED_REFERENCE_COMMIT:-c1915b0a63bfabebdb95a515cedfee05146c1d50}
reference_dir=${REFERENCE_DIR:-reference/esp-miner}
workspace_dir=${BUILD_WORKSPACE_DIRECTORY:-}

if [[ -z "$workspace_dir" ]]; then
	workspace_dir="$(git rev-parse --show-toplevel)"
fi

if [[ "$reference_dir" == /* ]]; then
	reference_worktree="$reference_dir"
	reference_submodule_path="$reference_dir"
else
	reference_worktree="${workspace_dir}/${reference_dir}"
	reference_submodule_path="$reference_dir"
fi

fail_missing_reference() {
	printf 'reference missing or not initialized: %s\n' "$reference_dir" >&2
	exit 1
}

require_reference_worktree() {
	if [[ ! -e "$reference_worktree" ]]; then
		fail_missing_reference
	fi

	if ! git -C "$reference_worktree" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
		fail_missing_reference
	fi
}

check_parent_submodule_status() {
	if [[ "${SKIP_PARENT_SUBMODULE_CHECK:-0}" == "1" ]]; then
		return 0
	fi

	local submodule_status
	submodule_status="$(git -C "$workspace_dir" submodule status --recursive "$reference_submodule_path")"

	local line
	while IFS= read -r line; do
		if [[ -z "$line" ]]; then
			continue
		fi

		case "${line:0:1}" in
		-)
			printf 'reference missing or not initialized: %s\n%s\n' "$reference_dir" "$line" >&2
			exit 1
			;;
		+)
			printf 'reference commit mismatch: %s\n%s\n' "$reference_dir" "$line" >&2
			exit 1
			;;
		U)
			printf 'reference submodule conflict: %s\n%s\n' "$reference_dir" "$line" >&2
			exit 1
			;;
		esac
	done <<<"$submodule_status"
}

check_reference_commit() {
	local actual_commit
	actual_commit="$(git -C "$reference_worktree" rev-parse HEAD)"

	if [[ "$actual_commit" != "$EXPECTED_REFERENCE_COMMIT" ]]; then
		printf 'reference commit mismatch: expected %s, found %s at %s\n' \
			"$EXPECTED_REFERENCE_COMMIT" \
			"$actual_commit" \
			"$reference_dir" >&2
		exit 1
	fi

	printf '%s\n' "$actual_commit"
}

check_reference_clean() {
	local actual_commit="$1"
	local dirty_state
	dirty_state="$(git -C "$reference_worktree" status --porcelain --untracked-files=all)"

	if [[ -n "$dirty_state" ]]; then
		printf 'reference dirty: %s\n%s\n' "$actual_commit" "$dirty_state" >&2
		exit 1
	fi
}

require_reference_worktree
check_parent_submodule_status
actual_reference_commit="$(check_reference_commit)"
check_reference_clean "$actual_reference_commit"

printf 'reference clean: %s\n' "$actual_reference_commit"
