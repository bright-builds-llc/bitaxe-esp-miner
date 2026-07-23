#!/usr/bin/env bash
set -euo pipefail

readonly report_binary="${1:?missing parity report binary}"
readonly eligible_fixture="${2:?missing eligible Phase 36 fixture}"
readonly phase36_module="${3:?missing Phase 36 module source}"
readonly phase36_contract="${4:?missing Phase 36 contract source}"
readonly parity_main="${5:?missing parity CLI source}"
readonly test_parent="$(mktemp -d "${TEST_TMPDIR:-/tmp}/phase36-evidence.XXXXXX")"
readonly protected_root="${test_parent}/protected"
readonly protected_input="${protected_root}/phase36.json"
readonly protected_note="${protected_root}/opaque-note.txt"
readonly public_output="${test_parent}/shareable.json"
readonly stderr_output="${test_parent}/stderr.txt"
readonly command_block="${test_parent}/command-source.txt"
readonly protected_canary="opaque-phase36-protected-canary"

cleanup() {
	[[ -n "$test_parent" && "$test_parent" == */phase36-evidence.* ]] ||
		return
	chmod -R u+rwX "$test_parent"
	rm -rf "$test_parent"
}
trap cleanup EXIT

fail_test() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

assert_absent_literal() {
	local file="$1"
	local literal="$2"
	if rg -Fq -- "$literal" "$file"; then
		fail_test "protected material reached a shareable sink"
	fi
}

umask 077
chmod 700 "$test_parent"
mkdir "$protected_root"
chmod 700 "$protected_root"
cp "$eligible_fixture" "$protected_input"
printf '%s\n' "$protected_canary" >"$protected_note"
chmod 600 "$protected_input" "$protected_note"

if ! "$report_binary" classify-phase36-evidence --root "$protected_root" \
	>"$public_output" 2>"$stderr_output"; then
	fail_test "read-only classifier process failed"
fi
chmod 600 "$public_output" "$stderr_output"

[[ "$(file_mode "$test_parent")" == 700 ]] ||
	fail_test "protected parent mode is not 0700"
[[ "$(file_mode "$protected_root")" == 700 ]] ||
	fail_test "protected child mode is not 0700"
[[ "$(file_mode "$protected_input")" == 600 ]] ||
	fail_test "protected input mode is not 0600"
[[ "$(file_mode "$protected_note")" == 600 ]] ||
	fail_test "protected note mode is not 0600"
[[ "$(file_mode "$public_output")" == 600 ]] ||
	fail_test "shareable output mode is not 0600"
[[ "$(file_mode "$stderr_output")" == 600 ]] ||
	fail_test "stderr capture mode is not 0600"
[[ ! -s "$stderr_output" ]] ||
	fail_test "successful classification wrote stderr"
rg -Fq '"status": "immutable_artifacts_sufficient"' "$public_output" ||
	fail_test "eligible fixture did not classify as sufficient"

for sink in "$public_output" "$stderr_output"; do
	assert_absent_literal "$sink" "$protected_canary"
	assert_absent_literal "$sink" "$protected_root"
	assert_absent_literal "$sink" "$protected_input"
	assert_absent_literal "$sink" "$protected_note"
done

sed -n '/^fn run_classify_phase36_evidence_command(/,/^}/p' "$parity_main" \
	>"$command_block"
chmod 600 "$command_block"
[[ -s "$command_block" ]] ||
	fail_test "Phase 36 CLI command block was not found"

readonly effectful_pattern='detect[-_]?ultra205|credential|(^|[^[:alnum:]_])flash([^-[:alnum:]_]|$)|monitor|serial[-_](control|session)|curl[[:space:]].*((--request|-X)[[:space:]]*(PATCH|POST|PUT|DELETE)|--data)|phase28\.1\.1|hardware[-_ ]run'
if rg -q -i "$effectful_pattern" \
	"$phase36_module" "$phase36_contract" "$command_block"; then
	fail_test "Phase 36 read-only classifier contains an effectful invocation"
fi

printf 'phase36 evidence tests passed\n'
