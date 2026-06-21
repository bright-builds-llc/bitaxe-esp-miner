#!/usr/bin/env bash
set -euo pipefail

readonly expected_reference_commit="c1915b0a63bfabebdb95a515cedfee05146c1d50"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly guard_script="${VERIFY_REFERENCE_CLEAN_SCRIPT:-${script_dir}/verify-reference-clean.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/verify-reference-clean-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local haystack="$1"
	local needle="$2"

	case "$haystack" in
	*"$needle"*) ;;
	*)
		printf 'Expected output to contain: %s\n' "$needle" >&2
		printf 'Actual output:\n%s\n' "$haystack" >&2
		exit 1
		;;
	esac
}

run_guard() {
	local reference_dir="$1"
	local expected_commit="$2"
	local skip_parent_check="$3"

	REFERENCE_DIR="$reference_dir" \
		EXPECTED_REFERENCE_COMMIT="$expected_commit" \
		SKIP_PARENT_SUBMODULE_CHECK="$skip_parent_check" \
		"$guard_script"
}

capture_guard() {
	local output_file="$1"
	shift

	set +e
	"$@" >"$output_file" 2>&1
	local status=$?
	set -e

	return "$status"
}

create_reference_repo() {
	local repo_dir="$1"

	mkdir -p "$repo_dir"
	git -C "$repo_dir" init -q
	git -C "$repo_dir" config user.name "Reference Guard Test"
	git -C "$repo_dir" config user.email "reference-guard@example.invalid"
	printf 'initial\n' >"${repo_dir}/tracked.txt"
	git -C "$repo_dir" add tracked.txt
	git -C "$repo_dir" commit -q -m "initial"
}

append_reference_commit() {
	local repo_dir="$1"
	local value="$2"

	printf '%s\n' "$value" >>"${repo_dir}/tracked.txt"
	git -C "$repo_dir" add tracked.txt
	git -C "$repo_dir" commit -q -m "update"
}

test_missing_reference_fails() {
	local output_file="${tmp_root}/missing.out"
	local missing_dir="${tmp_root}/missing/reference/esp-miner"

	if capture_guard "$output_file" run_guard "$missing_dir" "$expected_reference_commit" "1"; then
		fail "missing reference unexpectedly passed"
	fi

	assert_contains "$(cat "$output_file")" "reference missing or not initialized"
}

test_dirty_reference_fails() {
	local repo_dir="${tmp_root}/dirty/reference/esp-miner"
	local output_file="${tmp_root}/dirty.out"

	create_reference_repo "$repo_dir"
	local expected_commit
	expected_commit="$(git -C "$repo_dir" rev-parse HEAD)"
	printf 'untracked\n' >"${repo_dir}/untracked.txt"

	if capture_guard "$output_file" run_guard "$repo_dir" "$expected_commit" "1"; then
		fail "dirty reference unexpectedly passed"
	fi

	assert_contains "$(cat "$output_file")" "reference dirty"
}

test_clean_reference_passes() {
	local repo_dir="${tmp_root}/clean/reference/esp-miner"
	local output_file="${tmp_root}/clean.out"

	create_reference_repo "$repo_dir"
	local expected_commit
	expected_commit="$(git -C "$repo_dir" rev-parse HEAD)"

	if ! capture_guard "$output_file" run_guard "$repo_dir" "$expected_commit" "1"; then
		printf 'Clean reference output:\n%s\n' "$(cat "$output_file")" >&2
		fail "clean reference failed"
	fi

	assert_contains "$(cat "$output_file")" "reference clean: ${expected_commit}"
}

test_commit_mismatch_fails() {
	local repo_dir="${tmp_root}/mismatch/reference/esp-miner"
	local output_file="${tmp_root}/mismatch.out"

	create_reference_repo "$repo_dir"
	local expected_commit
	expected_commit="$(git -C "$repo_dir" rev-parse HEAD)"
	append_reference_commit "$repo_dir" "mismatch"

	if capture_guard "$output_file" run_guard "$repo_dir" "$expected_commit" "1"; then
		fail "mismatched reference unexpectedly passed"
	fi

	assert_contains "$(cat "$output_file")" "reference commit mismatch"
}

if [[ ! -x "$guard_script" ]]; then
	fail "guard script missing or not executable: ${guard_script}"
fi

test_missing_reference_fails
test_dirty_reference_fails
test_clean_reference_passes
test_commit_mismatch_fails

printf 'verify-reference-clean tests passed\n'
