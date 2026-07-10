#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly denylist="${script_dir}/phase28.1.1-promoted-evidence-denylist.sh"
temp_root="$(mktemp -d)"
readonly temp_root
trap 'rm -rf "$temp_root"' EXIT

fail() {
	printf 'promoted_evidence_denylist_test_error: %s\n' "$1" >&2
	exit 1
}

printf 'redaction_status: passed\n' >"$temp_root/clean.md"
bash "$denylist" "$temp_root/clean.md" >"$temp_root/clean.stdout"
rg -q '^promoted_evidence_denylist: passed$' "$temp_root/clean.stdout"

printf 'poolPassword=synthetic-forbidden-sentinel\n' >"$temp_root/forbidden.md"
if bash "$denylist" "$temp_root/forbidden.md" >"$temp_root/forbidden.stdout" 2>"$temp_root/forbidden.stderr"; then
	fail 'forbidden sentinel unexpectedly passed'
fi
rg -q 'forbidden evidence match' "$temp_root/forbidden.stderr"

printf '[\n' >"$temp_root/malformed.patterns"
if PHASE28_DENYLIST_PATTERN_FILE="$temp_root/malformed.patterns" bash "$denylist" "$temp_root/clean.md" >"$temp_root/malformed.stdout" 2>"$temp_root/malformed.stderr"; then
	fail 'malformed pattern unexpectedly passed'
fi
rg -q 'search tool failed with status 2' "$temp_root/malformed.stderr"

printf '#!/usr/bin/env bash\nexit 7\n' >"$temp_root/rg-error"
chmod +x "$temp_root/rg-error"
if PHASE28_DENYLIST_RG="$temp_root/rg-error" bash "$denylist" "$temp_root/clean.md" >"$temp_root/tool.stdout" 2>"$temp_root/tool.stderr"; then
	fail 'injected search-tool error unexpectedly passed'
fi
rg -q 'search tool failed with status 7' "$temp_root/tool.stderr"

printf 'promoted_evidence_denylist_test: passed\n'
