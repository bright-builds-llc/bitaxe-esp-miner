#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

if [[ -n "${TEST_SRCDIR:-}" && -f "${TEST_SRCDIR}/_main/.github/workflows/evidence-redaction.yml" ]]; then
	workspace_root="${TEST_SRCDIR}/_main"
else
	workspace_root="$(cd "${script_dir}/.." && pwd)"
fi
readonly workspace_root
readonly workflow_file="${workspace_root}/.github/workflows/evidence-redaction.yml"
readonly gitmodules_file="${workspace_root}/.gitmodules"

fail() {
	printf 'evidence_redaction_workflow_contract_test_error: category=%s\n' "$1" >&2
	exit 1
}

require_literal() {
	local source_file="$1"
	local expected="$2"
	local category="$3"

	rg -q -F -- "$expected" "$source_file" || fail "$category"
}

[[ -f "$workflow_file" ]] || fail workflow-missing
[[ -f "$gitmodules_file" ]] || fail gitmodules-missing

checkout_count="$(rg -c -F -- 'uses: actions/checkout@v4' "$workflow_file")"
readonly checkout_count
[[ "$checkout_count" -eq 1 ]] || fail checkout-count

checkout_block="$(
	awk '
        /uses: actions\/checkout@v4/ {
            capture = 1
        }
        capture {
            print
        }
        capture && /uses: extractions\/setup-just@v3/ {
            exit
        }
    ' "$workflow_file"
)"
readonly checkout_block

rg -q -F -- 'fetch-depth: 0' <<<"$checkout_block" || fail full-history
rg -q -F -- 'submodules: recursive' <<<"$checkout_block" || fail recursive-submodules
require_literal "$gitmodules_file" 'path = reference/esp-miner' reference-submodule

printf 'evidence_redaction_workflow_contract_test: passed\n'
