#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly checker="${PHASE29_DOC_REDACTION_CHECK:-${script_dir}/phase29-doc-redaction-check.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase29-doc-redaction-check-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'phase29_doc_redaction_check_test_error: %s\n' "$1" >&2
	exit 1
}

initialize_fixture_repo() {
	local repo_root="$1"

	mkdir -p \
		"${repo_root}/docs/release" \
		"${repo_root}/docs/parity/evidence/phase-29"
	printf '# Operator guide\n\nBaseline content.\n' >"${repo_root}/docs/release/ultra-205.md"
	for name in summary redaction-review conclusion; do
		printf '# Phase 29 %s\n\nredaction_status: passed\n' "$name" \
			>"${repo_root}/docs/parity/evidence/phase-29/${name}.md"
	done
	git -C "$repo_root" init -q
	git -C "$repo_root" config user.email phase29-test@example.invalid
	git -C "$repo_root" config user.name phase29-test
	git -C "$repo_root" add docs
	git -C "$repo_root" commit -qm baseline
}

run_checker() {
	local repo_root="$1"
	local stdout_path="$2"
	local stderr_path="$3"

	(
		cd "$repo_root"
		"$checker" \
			--baseline-ref HEAD \
			--evidence-root docs/parity/evidence/phase-29
	) >"$stdout_path" 2>"$stderr_path"
}

assert_clean_added_line_passes() {
	local repo_root="${tmp_root}/clean"
	initialize_fixture_repo "$repo_root"
	printf '\nPhase 25 finalization uses strict operator validation.\n' \
		>>"${repo_root}/docs/release/ultra-205.md"

	if ! run_checker "$repo_root" "${repo_root}/stdout" "${repo_root}/stderr"; then
		fail 'clean added guide line should pass'
	fi
	if ! grep -Fq 'phase29_doc_redaction_check: passed' "${repo_root}/stdout"; then
		fail 'clean scan did not report its category-safe pass label'
	fi
}

assert_forbidden_value_fails_without_echo() {
	local case_name="$1"
	local forbidden_value="$2"
	local repo_root="${tmp_root}/${case_name}"
	initialize_fixture_repo "$repo_root"
	printf '\n%s\n' "$forbidden_value" >>"${repo_root}/docs/release/ultra-205.md"

	set +e
	run_checker "$repo_root" "${repo_root}/stdout" "${repo_root}/stderr"
	local status=$?
	set -e
	if [[ "$status" -eq 0 ]]; then
		fail "${case_name} unexpectedly passed"
	fi
	if grep -Fq -- "$forbidden_value" "${repo_root}/stdout" "${repo_root}/stderr"; then
		fail "${case_name} leaked matched content"
	fi
	if ! grep -Fq 'phase29_doc_redaction_check_error: category=' "${repo_root}/stderr"; then
		fail "${case_name} did not report a category-safe failure label"
	fi
}

assert_evidence_document_is_scanned() {
	local evidence_name="$1"
	local repo_root="${tmp_root}/evidence-document-${evidence_name}"
	local forbidden_value='token: synthetic-secret-value'
	initialize_fixture_repo "$repo_root"
	printf '\n%s\n' "$forbidden_value" \
		>>"${repo_root}/docs/parity/evidence/phase-29/${evidence_name}.md"

	set +e
	run_checker "$repo_root" "${repo_root}/stdout" "${repo_root}/stderr"
	local status=$?
	set -e
	if [[ "$status" -eq 0 ]]; then
		fail "forbidden ${evidence_name} value unexpectedly passed"
	fi
	if grep -Fq -- "$forbidden_value" "${repo_root}/stdout" "${repo_root}/stderr"; then
		fail "${evidence_name} match leaked matched content"
	fi
}

assert_clean_added_line_passes

while IFS='|' read -r case_name forbidden_value; do
	assert_forbidden_value_fails_without_echo "$case_name" "$forbidden_value"
done <<'CASES'
macos_path|/Users/example/private-evidence
linux_path|/home/example/private-evidence
windows_path|C:\private\evidence
ipv4|192.0.2.42
ipv6|2001:db8::42
mac_address|02:00:5e:10:00:00
http_url|http://example.invalid/device
https_url|https://example.invalid/device
phase27_raw_sentinel|PHASE27_RAW_SENTINEL_VALUE
pool_value|poolURL=synthetic-pool-value
wifi_value|wifi-credentials=synthetic-local-file
password_value|password: synthetic-password-value
token_value|token: synthetic-token-value
raw_value|raw_value=deadbeef
CASES

for evidence_name in summary redaction-review conclusion; do
	assert_evidence_document_is_scanned "$evidence_name"
done

printf 'phase29_doc_redaction_check_test: passed\n'
