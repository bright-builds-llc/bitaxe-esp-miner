#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly pattern_file="${PHASE28_DENYLIST_PATTERN_FILE:-${script_dir}/phase28.1.1-promoted-evidence-denylist.patterns}"
readonly search_tool="${PHASE28_DENYLIST_RG:-rg}"

if [[ "$#" -eq 0 ]]; then
	printf 'promoted_evidence_denylist_error: provide at least one artifact\n' >&2
	exit 64
fi

set +e
"$search_tool" -n -i -f "$pattern_file" -- "$@"
search_status=$?
set -e

case "$search_status" in
0)
	printf 'promoted_evidence_denylist_error: forbidden evidence match\n' >&2
	exit 1
	;;
1)
	printf 'promoted_evidence_denylist: passed\n'
	;;
*)
	printf 'promoted_evidence_denylist_error: search tool failed with status %s\n' "$search_status" >&2
	exit "$search_status"
	;;
esac
