#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: phase29-doc-redaction-check --baseline-ref COMMIT --evidence-root PATH\n' >&2
}

baseline_ref=""
evidence_root=""

while [[ $# -gt 0 ]]; do
	case "$1" in
	--baseline-ref)
		baseline_ref="${2:-}"
		shift 2
		;;
	--evidence-root)
		evidence_root="${2:-}"
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'phase29_doc_redaction_check_error: category=arguments\n' >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$baseline_ref" || -z "$evidence_root" ]]; then
	printf 'phase29_doc_redaction_check_error: category=arguments\n' >&2
	usage
	exit 2
fi

readonly workspace_root="${BUILD_WORKSPACE_DIRECTORY:-$PWD}"

if ! baseline_commit="$(git -C "$workspace_root" rev-parse --verify --quiet "${baseline_ref}^{commit}")"; then
	printf 'phase29_doc_redaction_check_error: category=baseline-ref\n' >&2
	exit 2
fi
readonly baseline_commit

evidence_root_path="$evidence_root"
if [[ "$evidence_root_path" != /* ]]; then
	evidence_root_path="${workspace_root}/${evidence_root_path}"
fi
readonly evidence_root_path

if [[ ! -d "$evidence_root_path" ]]; then
	printf 'phase29_doc_redaction_check_error: category=evidence-root\n' >&2
	exit 2
fi

script_path="${BASH_SOURCE[0]}"
if [[ -L "$script_path" ]]; then
	script_path="$(readlink "$script_path")"
fi
script_dir="$(cd "$(dirname "$script_path")" && pwd)"
readonly script_dir
readonly existing_denylist="${script_dir}/phase28.1.1-promoted-evidence-denylist.sh"
readonly guide_path="docs/release/ultra-205.md"

umask 077
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase29-doc-redaction-check.XXXXXX")"
readonly tmp_root
chmod 700 "$tmp_root"

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

readonly aggregate_path="${tmp_root}/aggregate.txt"
readonly diff_path="${tmp_root}/guide.diff"
: >"$aggregate_path"
chmod 600 "$aggregate_path"

if ! git -C "$workspace_root" diff --unified=0 "$baseline_commit" -- "$guide_path" >"$diff_path" 2>"${tmp_root}/git-diff.stderr"; then
	printf 'phase29_doc_redaction_check_error: category=guide-diff\n' >&2
	exit 2
fi

awk '
	substr($0, 1, 4) == "+++ " { next }
	substr($0, 1, 1) == "+" { print substr($0, 2) }
' "$diff_path" >>"$aggregate_path"

for evidence_name in summary redaction-review conclusion; do
	evidence_path="${evidence_root_path}/${evidence_name}.md"
	if [[ ! -f "$evidence_path" ]]; then
		printf 'phase29_doc_redaction_check_error: category=evidence-inventory\n' >&2
		exit 2
	fi
	cat "$evidence_path" >>"$aggregate_path"
done

if [[ ! -x "$existing_denylist" ]]; then
	printf 'phase29_doc_redaction_check_error: category=existing-denylist-unavailable\n' >&2
	exit 2
fi

set +e
"$existing_denylist" "$aggregate_path" \
	>"${tmp_root}/existing-denylist.stdout" \
	2>"${tmp_root}/existing-denylist.stderr"
denylist_status=$?
set -e
if [[ "$denylist_status" -ne 0 ]]; then
	printf 'phase29_doc_redaction_check_error: category=secret-or-raw-value\n' >&2
	exit 1
fi

scan_explicit_category() {
	local category="$1"
	local pattern="$2"
	local search_status

	set +e
	rg -q -i -e "$pattern" -- "$aggregate_path"
	search_status=$?
	set -e
	case "$search_status" in
	0)
		printf 'phase29_doc_redaction_check_error: category=%s\n' "$category" >&2
		exit 1
		;;
	1) ;;
	*)
		printf 'phase29_doc_redaction_check_error: category=scanner\n' >&2
		exit 2
		;;
	esac
}

scan_explicit_category macos-user-path '/Users/'
scan_explicit_category linux-home-path '/home/'
scan_explicit_category windows-drive-path "[A-Za-z]:\\\\"
scan_explicit_category ipv4-address '[0-9]{1,3}(\.[0-9]{1,3}){3}'
scan_explicit_category ipv6-address '([[:xdigit:]]{1,4}:){2,7}[[:xdigit:]]{0,4}'
scan_explicit_category mac-address '[[:xdigit:]]{2}(:[[:xdigit:]]{2}){5}'
scan_explicit_category url 'https?://'
scan_explicit_category phase27-raw-sentinel 'PHASE27_RAW_SENTINEL_VALUE'

printf 'phase29_doc_redaction_check: passed\n'
