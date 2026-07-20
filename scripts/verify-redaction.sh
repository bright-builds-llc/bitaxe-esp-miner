#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: verify-redaction [--base COMMIT --head COMMIT]\n' >&2
}

base_ref=""
head_ref=""

while [[ $# -gt 0 ]]; do
	case "$1" in
	--base)
		base_ref="${2:-}"
		shift 2
		;;
	--head)
		head_ref="${2:-}"
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'redaction_violation: rule=CONFIG category=arguments path=scripts/verify-redaction.sh line=0\n' >&2
		exit 2
		;;
	esac
done

if [[ -n "$base_ref" || -n "$head_ref" ]]; then
	if [[ -z "$base_ref" || -z "$head_ref" ]]; then
		printf 'redaction_violation: rule=CONFIG category=arguments path=scripts/verify-redaction.sh line=0\n' >&2
		exit 2
	fi
fi

workspace_root="$(git rev-parse --show-toplevel)"
readonly workspace_root
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
exception_registry="${script_dir}/redaction-exceptions.tsv"
invoked_runfiles_registry="${0}.runfiles/_main/scripts/redaction-exceptions.tsv"
if [[ ! -f "$exception_registry" && -f "$invoked_runfiles_registry" ]]; then
	exception_registry="$invoked_runfiles_registry"
elif [[ ! -f "$exception_registry" && -n "${RUNFILES_DIR:-}" &&
	-f "${RUNFILES_DIR}/_main/scripts/redaction-exceptions.tsv" ]]; then
	exception_registry="${RUNFILES_DIR}/_main/scripts/redaction-exceptions.tsv"
elif [[ ! -f "$exception_registry" && -n "${TEST_SRCDIR:-}" &&
	-f "${TEST_SRCDIR}/_main/scripts/redaction-exceptions.tsv" ]]; then
	exception_registry="${TEST_SRCDIR}/_main/scripts/redaction-exceptions.tsv"
fi
readonly exception_registry
readonly admitted_root="${workspace_root}/docs/parity/evidence"
readonly secondary_admitted_root="${workspace_root}/docs/evidence"
today="$(date -u +%Y-%m-%d)"
readonly today

if [[ ! -f "$exception_registry" ]]; then
	printf 'redaction_violation: rule=CONFIG category=exception-registry path=scripts/redaction-exceptions.tsv line=0\n' >&2
	exit 2
fi

if ! awk -F '\t' -v today="$today" '
	NR == 1 {
		if ($0 != "exception_id\tcategory\tpath\treason\texpires_on") {
			exit 1
		}
		next
	}
	NF != 4 && NF != 5 { exit 1 }
	$1 !~ /^RED-[0-9][0-9][0-9][0-9]$/ { exit 1 }
	$2 !~ /^(credential-secret|pool-owner|local-path|usb-path|network-address|device-origin|ssid|hostname|process-id|raw-http|opaque-binary)$/ { exit 1 }
	$3 == "" || $3 ~ /[*?\[]/ || $3 ~ /^\// || $3 ~ /(^|\/)\.\.(\/|$)/ { exit 1 }
	$4 == "" { exit 1 }
	$5 != "" && $5 !~ /^[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]$/ { exit 1 }
	$5 != "" && $5 < today { exit 1 }
	seen_id[$1]++ { exit 1 }
	seen_pair[$2 "\t" $3]++ { exit 1 }
' "$exception_registry"; then
	printf 'redaction_violation: rule=CONFIG category=exception-registry path=scripts/redaction-exceptions.tsv line=0\n' >&2
	exit 2
fi

if [[ -n "$base_ref" ]]; then
	if ! git cat-file -e "${base_ref}^{commit}" 2>/dev/null ||
		! git cat-file -e "${head_ref}^{commit}" 2>/dev/null; then
		printf 'redaction_violation: rule=CONFIG category=revision path=scripts/verify-redaction.sh line=0\n' >&2
		exit 2
	fi
fi

violations=0
exception_pairs=()
while IFS=$'\t' read -r exception_id category target_path _reason _expires_on; do
	if [[ "$exception_id" == "exception_id" ]]; then
		continue
	fi
	exception_pairs+=("${category}"$'\t'"${target_path}")
done <"$exception_registry"

exception_allows() {
	local category="$1"
	local target_path="$2"
	local requested_pair="${category}"$'\t'"${target_path}"
	local exception_pair

	for exception_pair in "${exception_pairs[@]}"; do
		if [[ "$exception_pair" == "$requested_pair" ]]; then
			return 0
		fi
	done
	return 1
}

report_violation() {
	local rule_id="$1"
	local category="$2"
	local target_path="$3"
	local line_number="$4"

	if exception_allows "$category" "$target_path"; then
		return
	fi

	printf 'redaction_violation: rule=%s category=%s path=%s line=%s\n' \
		"$rule_id" "$category" "$target_path" "$line_number" >&2
	violations=$((violations + 1))
}

safe_value() {
	local candidate
	candidate="$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')"
	case "$candidate" in
	"" | *redacted* | *unavailable* | *not-provided* | *not_provided* | *local-owner-supplied* | *shareable-fact* | *protected-operational* | *never-persist-raw*)
		return 0
		;;
	esac
	return 1
}

scan_line() {
	local target_path="$1"
	local line_number="$2"
	local content="$3"
	local assignment_pattern
	local maybe_value

	assignment_pattern='(^|[^[:alnum:]_])(wifiPass|wifipass|wifi_password|password|pass|token|apiKey|api_key|pool_password|poolPassword|stratumPassword|nvsSecret|secret)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[3]}"
		if ! safe_value "$maybe_value"; then
			report_violation NP-001 credential-secret "$target_path" "$line_number"
		fi
	fi

	assignment_pattern='(^|[^[:alnum:]_])(poolURL|poolPort|poolUser|poolWorker|worker|ownerAddress|btcAddress)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[3]}"
		if ! safe_value "$maybe_value"; then
			report_violation NP-002 pool-owner "$target_path" "$line_number"
		fi
	fi

	if [[ "$content" =~ /Users/[^[:space:]]+ ]] ||
		[[ "$content" =~ /home/[^[:space:]]+ ]] ||
		[[ "$content" =~ [A-Za-z]:\\\\[^[:space:]]+ ]]; then
		report_violation OP-001 local-path "$target_path" "$line_number"
	fi

	if [[ "$content" =~ /dev/(cu|tty)[^[:space:]]* ]] ||
		[[ "$content" =~ USB[_[:space:]-]*(serial|identity)[=:][^[:space:]]+ ]]; then
		report_violation OP-002 usb-path "$target_path" "$line_number"
	fi

	if [[ "$content" =~ (^|[^0-9.])([0-9]{1,3}\.){3}[0-9]{1,3}([^0-9.]|$) ]] ||
		[[ "$content" =~ (^|[^[:xdigit:]])[[:xdigit:]]{2}(:[[:xdigit:]]{2}){5}([^[:xdigit:]]|$) ]]; then
		report_violation OP-003 network-address "$target_path" "$line_number"
	fi

	assignment_pattern='(device_url|deviceUrl|origin)[[:space:]]*["]*[=:][[:space:]]*["]*(https?://[^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[2]}"
		if ! safe_value "$maybe_value"; then
			report_violation OP-004 device-origin "$target_path" "$line_number"
		fi
	fi

	assignment_pattern='(ssid|SSID)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[2]}"
		if ! safe_value "$maybe_value"; then
			report_violation OP-005 ssid "$target_path" "$line_number"
		fi
	fi

	assignment_pattern='(hostname|hostName)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[2]}"
		if ! safe_value "$maybe_value"; then
			report_violation OP-006 hostname "$target_path" "$line_number"
		fi
	fi

	if [[ "$content" =~ (^|[^[:alnum:]_])(pid|pgid)[=:][[:space:]]*[0-9]+ ]]; then
		report_violation OP-007 process-id "$target_path" "$line_number"
	fi

	if [[ "$content" =~ HTTP/[12]\.[0-9] ]] ||
		[[ "$content" =~ (^|[[:space:]])Host:[[:space:]]*[^[:space:]]+ ]]; then
		report_violation OP-008 raw-http "$target_path" "$line_number"
	fi
}

scan_stream() {
	local target_path="$1"
	local line_number=0
	local content

	while IFS= read -r content || [[ -n "$content" ]]; do
		line_number=$((line_number + 1))
		scan_line "$target_path" "$line_number" "$content"
	done
}

scan_git_blob() {
	local target_path="$1"
	local blob="$2"

	if [[ "$(git cat-file -s "$blob")" -eq 0 ]]; then
		return
	fi

	if git show "$blob" | LC_ALL=C grep -I '' >/dev/null; then
		scan_stream "$target_path" < <(git show "$blob")
		return
	fi

	report_violation OP-009 opaque-binary "$target_path" 0
}

scan_changed_paths() {
	local target_path

	if [[ -n "$base_ref" ]]; then
		while IFS= read -r target_path; do
			[[ -n "$target_path" ]] || continue
			if git cat-file -e "${head_ref}:${target_path}" 2>/dev/null; then
				scan_git_blob "$target_path" "${head_ref}:${target_path}"
			fi
		done < <(git diff --name-only --diff-filter=ACMR "$base_ref" "$head_ref" --)
		return
	fi

	while IFS= read -r target_path; do
		[[ -n "$target_path" ]] || continue
		if git cat-file -e ":${target_path}" 2>/dev/null; then
			scan_git_blob "$target_path" ":${target_path}"
		fi
	done < <(git diff --cached --name-only --diff-filter=ACMR --)
}

scan_admitted_root() {
	local root="$1"
	local artifact
	local target_path

	if [[ ! -d "$root" ]]; then
		return
	fi

	while IFS= read -r -d '' artifact; do
		if [[ -L "$artifact" || ! -f "$artifact" ]]; then
			target_path="${artifact#"${workspace_root}/"}"
			report_violation CONFIG admitted-artifact "$target_path" 0
			continue
		fi
		target_path="${artifact#"${workspace_root}/"}"
		if [[ ! -s "$artifact" ]]; then
			continue
		fi
		if LC_ALL=C grep -Iq '' "$artifact"; then
			scan_stream "$target_path" <"$artifact"
		else
			report_violation OP-009 opaque-binary "$target_path" 0
		fi
	done < <(find "$root" \( -type f -o -type l \) -print0)
}

scan_changed_paths
scan_admitted_root "$admitted_root"
scan_admitted_root "$secondary_admitted_root"

if [[ "$violations" -ne 0 ]]; then
	exit 1
fi

printf 'verify_redaction: passed\n'
