#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: verify-redaction [--base COMMIT --head COMMIT] [--new-branch-base COMMIT]\n' >&2
}

base_ref=""
head_ref=""
new_branch_base_ref=""

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
	--new-branch-base)
		new_branch_base_ref="${2:-}"
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

readonly zero_revision="0000000000000000000000000000000000000000"
destination_base_ref="$base_ref"

if [[ -n "$base_ref" ]]; then
	if { [[ "$base_ref" != "$zero_revision" ]] &&
		! git cat-file -e "${base_ref}^{commit}" 2>/dev/null; } ||
		! git cat-file -e "${head_ref}^{commit}" 2>/dev/null; then
		printf 'redaction_violation: rule=CONFIG category=revision path=scripts/verify-redaction.sh line=0\n' >&2
		exit 2
	fi
	if [[ "$base_ref" == "$zero_revision" ]]; then
		if [[ -z "$new_branch_base_ref" ]]; then
			maybe_default_branch_ref=""
			if maybe_default_branch_ref="$(git symbolic-ref -q --short refs/remotes/origin/HEAD 2>/dev/null)"; then
				new_branch_base_ref="$maybe_default_branch_ref"
			fi
		fi
		if [[ -z "$new_branch_base_ref" ]] ||
			! git cat-file -e "${new_branch_base_ref}^{commit}" 2>/dev/null; then
			printf 'redaction_violation: rule=CONFIG category=new-branch-base path=scripts/verify-redaction.sh line=0\n' >&2
			exit 2
		fi
		if ! destination_base_ref="$(git merge-base "$new_branch_base_ref" "$head_ref")" ||
			[[ -z "$destination_base_ref" ]]; then
			printf 'redaction_violation: rule=CONFIG category=new-branch-base path=scripts/verify-redaction.sh line=0\n' >&2
			exit 2
		fi
	elif [[ -n "$new_branch_base_ref" ]]; then
		printf 'redaction_violation: rule=CONFIG category=arguments path=scripts/verify-redaction.sh line=0\n' >&2
		exit 2
	fi
elif [[ -n "$new_branch_base_ref" ]]; then
	printf 'redaction_violation: rule=CONFIG category=arguments path=scripts/verify-redaction.sh line=0\n' >&2
	exit 2
fi
readonly destination_base_ref

violations=0
reported_violations=0
readonly max_reported_violations=100
changed_paths=("")
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
	local allow_exception="$5"

	if [[ "$allow_exception" == "true" ]] && exception_allows "$category" "$target_path"; then
		return
	fi

	violations=$((violations + 1))
	if [[ "$reported_violations" -ge "$max_reported_violations" ]]; then
		return
	fi
	printf 'redaction_violation: rule=%s category=%s path=%s line=%s\n' \
		"$rule_id" "$category" "$target_path" "$line_number" >&2
	reported_violations=$((reported_violations + 1))
}

path_is_changed() {
	local requested_path="$1"
	local changed_path

	for changed_path in "${changed_paths[@]}"; do
		if [[ "$changed_path" == "$requested_path" ]]; then
			return 0
		fi
	done
	return 1
}

is_shareable_sink() {
	local target_path="$1"

	case "$target_path" in
	docs/* | .planning/* | .codex/tasks/* | tasks/* | *.md | *.mdx)
		return 0
		;;
	esac
	return 1
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
	local check_operational="$4"
	local allow_exception="$5"
	local assignment_pattern
	local maybe_value

	assignment_pattern='(^|[^[:alnum:]_])(wifiPass|wifipass|wifi_password|password|pass|token|apiKey|api_key|pool_password|poolPassword|stratumPassword|nvsSecret|secret)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[3]}"
		if ! safe_value "$maybe_value"; then
			report_violation NP-001 credential-secret "$target_path" "$line_number" "$allow_exception"
		fi
	fi

	assignment_pattern='(^|[^[:alnum:]_])(poolURL|poolPort|poolUser|poolWorker|worker|ownerAddress|btcAddress)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[3]}"
		if ! safe_value "$maybe_value"; then
			report_violation NP-002 pool-owner "$target_path" "$line_number" "$allow_exception"
		fi
	fi

	if [[ "$check_operational" != "true" ]]; then
		return
	fi

	if [[ "$content" =~ /Users/[^[:space:]]+ ]] ||
		[[ "$content" =~ /home/[^[:space:]]+ ]] ||
		[[ "$content" =~ [A-Za-z]:\\\\[^[:space:]]+ ]]; then
		report_violation OP-001 local-path "$target_path" "$line_number" "$allow_exception"
	fi

	if [[ "$content" =~ /dev/(cu|tty)[^[:space:]]* ]] ||
		[[ "$content" =~ USB[_[:space:]-]*(serial|identity)[=:][^[:space:]]+ ]]; then
		report_violation OP-002 usb-path "$target_path" "$line_number" "$allow_exception"
	fi

	if [[ "$content" =~ (^|[^0-9.])([0-9]{1,3}\.){3}[0-9]{1,3}([^0-9.]|$) ]] ||
		[[ "$content" =~ (^|[^[:xdigit:]])[[:xdigit:]]{2}(:[[:xdigit:]]{2}){5}([^[:xdigit:]]|$) ]]; then
		report_violation OP-003 network-address "$target_path" "$line_number" "$allow_exception"
	fi

	assignment_pattern='(device_url|deviceUrl|origin)[[:space:]]*["]*[=:][[:space:]]*["]*(https?://[^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[2]}"
		if ! safe_value "$maybe_value"; then
			report_violation OP-004 device-origin "$target_path" "$line_number" "$allow_exception"
		fi
	fi

	assignment_pattern='(ssid|SSID)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[2]}"
		if ! safe_value "$maybe_value"; then
			report_violation OP-005 ssid "$target_path" "$line_number" "$allow_exception"
		fi
	fi

	assignment_pattern='(hostname|hostName)[[:space:]]*["]*[=:][[:space:]]*["]*([^[:space:]",}]+)'
	if [[ "$content" =~ $assignment_pattern ]]; then
		maybe_value="${BASH_REMATCH[2]}"
		if ! safe_value "$maybe_value"; then
			report_violation OP-006 hostname "$target_path" "$line_number" "$allow_exception"
		fi
	fi

	if [[ "$content" =~ (^|[^[:alnum:]_])(pid|pgid)[=:][[:space:]]*[0-9]+ ]]; then
		report_violation OP-007 process-id "$target_path" "$line_number" "$allow_exception"
	fi

	if [[ "$content" =~ HTTP/[12]\.[0-9] ]] ||
		[[ "$content" =~ (^|[[:space:]])Host:[[:space:]]*[^[:space:]]+ ]]; then
		report_violation OP-008 raw-http "$target_path" "$line_number" "$allow_exception"
	fi
}

scan_stream() {
	local target_path="$1"
	local check_operational="$2"
	local allow_exception="$3"
	local line_number=0
	local content

	while IFS= read -r content || [[ -n "$content" ]]; do
		line_number=$((line_number + 1))
		scan_line "$target_path" "$line_number" "$content" "$check_operational" "$allow_exception"
	done
}

scan_git_blob() {
	local target_path="$1"
	local blob="$2"
	local check_operational="false"

	if is_shareable_sink "$target_path"; then
		check_operational="true"
	fi

	if [[ "$(git cat-file -s "$blob")" -eq 0 ]]; then
		return
	fi

	if git show "$blob" | LC_ALL=C grep -I '' >/dev/null; then
		scan_stream "$target_path" "$check_operational" false < <(git show "$blob")
		return
	fi

	if [[ "$check_operational" == "true" ]]; then
		report_violation OP-009 opaque-binary "$target_path" 0 false
	fi
}

path_is_renamed() {
	local target_path="$1"

	if [[ -n "$base_ref" ]]; then
		git diff --quiet --diff-filter=R "$destination_base_ref" "$head_ref" -- "$target_path" || return 0
	else
		git diff --cached --quiet --diff-filter=R -- "$target_path" || return 0
	fi
	return 1
}

scan_added_lines() {
	local target_path="$1"
	local check_operational="false"
	local blob
	local line
	local line_number=0
	local in_hunk="false"
	local hunk_pattern='^@@ -[0-9]+(,[0-9]+)? \+([0-9]+)(,[0-9]+)? @@'

	if is_shareable_sink "$target_path"; then
		check_operational="true"
	fi
	if [[ -n "$base_ref" ]]; then
		blob="${head_ref}:${target_path}"
	else
		blob=":${target_path}"
	fi
	if [[ "$(git cat-file -s "$blob")" -eq 0 ]]; then
		return
	fi
	if ! git show "$blob" | LC_ALL=C grep -I '' >/dev/null; then
		if [[ "$check_operational" == "true" ]]; then
			report_violation OP-009 opaque-binary "$target_path" 0 false
		fi
		return
	fi
	if path_is_renamed "$target_path"; then
		scan_git_blob "$target_path" "$blob"
		return
	fi

	while IFS= read -r line || [[ -n "$line" ]]; do
		if [[ "$line" =~ $hunk_pattern ]]; then
			line_number="${BASH_REMATCH[2]}"
			in_hunk="true"
			continue
		fi
		if [[ "$in_hunk" != "true" ]]; then
			continue
		fi
		case "$line" in
		+*)
			scan_line "$target_path" "$line_number" "${line:1}" "$check_operational" false
			line_number=$((line_number + 1))
			;;
		-*) ;;
		' '*)
			line_number=$((line_number + 1))
			;;
		'\ No newline at end of file') ;;
		*)
			in_hunk="false"
			;;
		esac
	done < <(
		if [[ -n "$base_ref" ]]; then
			git diff --no-ext-diff --no-color --unified=0 "$destination_base_ref" "$head_ref" -- "$target_path"
		else
			git diff --cached --no-ext-diff --no-color --unified=0 -- "$target_path"
		fi
	)
}

scan_changed_paths() {
	local target_path

	if [[ -n "$base_ref" ]]; then
		while IFS= read -r -d '' target_path; do
			[[ -n "$target_path" ]] || continue
			changed_paths+=("$target_path")
			if git cat-file -e "${head_ref}:${target_path}" 2>/dev/null; then
				scan_added_lines "$target_path"
			fi
		done < <(git diff --name-only -z --diff-filter=ACMR "$destination_base_ref" "$head_ref" --)
		return
	fi

	while IFS= read -r -d '' target_path; do
		[[ -n "$target_path" ]] || continue
		changed_paths+=("$target_path")
		if git cat-file -e ":${target_path}" 2>/dev/null; then
			scan_added_lines "$target_path"
		fi
	done < <(git diff --cached --name-only -z --diff-filter=ACMR --)
}

scan_admitted_root() {
	local root="$1"
	local artifact
	local target_path
	local tracked_root
	local allow_exception

	if [[ ! -d "$root" ]]; then
		return
	fi
	tracked_root="${root#"${workspace_root}/"}"

	while IFS= read -r -d '' target_path; do
		artifact="${workspace_root}/${target_path}"
		if [[ -L "$artifact" || ! -f "$artifact" ]]; then
			report_violation CONFIG admitted-artifact "$target_path" 0 false
			continue
		fi
		allow_exception="false"
		if ! path_is_changed "$target_path" &&
			git diff --quiet -- "$target_path" &&
			git diff --cached --quiet -- "$target_path"; then
			allow_exception="true"
		fi
		if [[ ! -s "$artifact" ]]; then
			continue
		fi
		if LC_ALL=C grep -Iq '' "$artifact"; then
			scan_stream "$target_path" true "$allow_exception" <"$artifact"
		else
			report_violation OP-009 opaque-binary "$target_path" 0 "$allow_exception"
		fi
	done < <(git ls-files -z -- "$tracked_root")
}

scan_changed_paths
scan_admitted_root "$admitted_root"
scan_admitted_root "$secondary_admitted_root"

if [[ "$violations" -ne 0 ]]; then
	if [[ "$violations" -gt "$reported_violations" ]]; then
		printf 'redaction_violation: rule=SUMMARY category=suppressed path=- line=%s\n' \
			"$((violations - reported_violations))" >&2
	fi
	exit 1
fi

printf 'verify_redaction: passed\n'
