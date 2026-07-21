#!/usr/bin/env bash
set -euo pipefail

readonly PHASE35_HTTP_SCHEMA="phase35-http-boundary-v2"

label=""
protected_root=""
url=""

usage() {
	printf 'usage: %s label=original|immediate|restoration protected-root=PATH url=ORIGIN\n' \
		"$(basename "$0")" >&2
}

while (($#)); do
	case "$1" in
	label=*) label="${1#*=}" ;;
	protected-root=* | protected_root=*) protected_root="${1#*=}" ;;
	url=*) url="${1#*=}" ;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		usage
		exit 2
		;;
	esac
	shift
done

case "$label" in
original | immediate | restoration) ;;
*)
	usage
	exit 2
	;;
esac
[[ "$protected_root" == /* ]] || {
	printf 'category=http_diagnostic_invalid\n'
	exit 1
}
[[ "$url" =~ ^https?://[^/?#]+/?$ && "$url" != *"@"* ]] || {
	printf 'category=http_diagnostic_invalid\n'
	exit 1
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

validate_private_directory() {
	local directory="$1"
	[[ -d "$directory" && ! -L "$directory" ]] || return 1
	[[ "$(file_mode "$directory")" == 700 ]]
}

validate_private_file() {
	local file="$1"
	[[ -f "$file" && ! -L "$file" ]] || return 1
	[[ "$(file_mode "$file")" == 600 ]]
}

validate_private_directory "$protected_root" || {
	printf 'category=http_diagnostic_invalid\n'
	exit 1
}

umask 077
read_dir="${protected_root}/http-${label}"
[[ ! -e "$read_dir" ]] || {
	printf 'category=http_diagnostic_invalid\n'
	exit 1
}
mkdir "$read_dir"
chmod 700 "$read_dir"

body_path="${read_dir}/body"
headers_path="${read_dir}/headers"
stderr_path="${read_dir}/stderr"
metrics_path="${read_dir}/metrics"
projection_path="${read_dir}/projection"
hostname_path="${read_dir}/private-hostname"
: >"$stderr_path"
chmod 600 "$stderr_path"

write_invalid_projection() {
	jq -cn \
		--arg schema_version "$PHASE35_HTTP_SCHEMA" \
		'{schema_version:$schema_version,tcp_connected:false,tls_applicable:false,tls_established:false,tls_verified:false,request_transmission_complete:false,response_status_received:false,response_headers_received:false,response_body_received:false,response_body_complete:false,json_parsed:false,hostname_schema_valid:false,transport_outcome:"tcp_connection_failure",request_send_complete_millis:0,request_bytes:0,response_header_count:0,response_header_bytes:0,response_body_bytes:0,tcp_connect_millis:0,tls_handshake_millis:0,first_byte_millis:0,total_millis:0,response_status_class:"missing",terminal_category:"http_diagnostic_invalid"}' \
		>"$projection_path"
	chmod 600 "$projection_path"
}

invalid_diagnostic() {
	write_invalid_projection
	printf 'category=http_diagnostic_invalid\n'
	exit 1
}

fixture_authority="${PHASE35_HTTP_FIXTURE_AUTHORITY:-false}"
probe_override="${PHASE35_HTTP_PROBE_EXECUTABLE:-}"
if [[ -n "$probe_override" ]]; then
	[[ "$fixture_authority" == true ]] || invalid_diagnostic
	[[ -x "$probe_override" ]] || invalid_diagnostic
fi

resolve_classifier() {
	if [[ -n "${PHASE35_HTTP_CLASSIFIER_EXECUTABLE:-}" ]]; then
		[[ "$fixture_authority" == true ]] || return 1
		[[ -x "$PHASE35_HTTP_CLASSIFIER_EXECUTABLE" ]] || return 1
		printf '%s\n' "$PHASE35_HTTP_CLASSIFIER_EXECUTABLE"
		return 0
	fi

	local workspace_dir="${BUILD_WORKSPACE_DIRECTORY:-}"
	local candidate
	for candidate in \
		"${workspace_dir:+${workspace_dir}/bazel-bin/tools/parity/report}" \
		"${BASH_SOURCE[0]}.runfiles/_main/tools/parity/report" \
		"${RUNFILES_DIR:-}/_main/tools/parity/report"; do
		if [[ -x "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done
	return 1
}

classifier_executable="$(resolve_classifier)" || invalid_diagnostic
probe_executable="${probe_override:-$classifier_executable}"
set +e
probe_output="$(
	"$probe_executable" probe-phase35-http \
		--url "$url" \
		--metrics-output "$metrics_path" \
		--headers-output "$headers_path" \
		--body-output "$body_path" 2>>"$stderr_path"
)"
probe_status=$?
set -e
((probe_status == 0)) || invalid_diagnostic
[[ "$probe_output" == "status=probe_complete" ]] || invalid_diagnostic
for private_input in "$body_path" "$headers_path" "$stderr_path" "$metrics_path"; do
	validate_private_file "$private_input" || invalid_diagnostic
done
actual_header_bytes="$(wc -c <"$headers_path" | tr -d ' ')"
actual_body_bytes="$(wc -c <"$body_path" | tr -d ' ')"
jq -e \
	--argjson actual_header_bytes "$actual_header_bytes" \
	--argjson actual_body_bytes "$actual_body_bytes" \
	'.response_header_bytes == $actual_header_bytes and .response_body_bytes == $actual_body_bytes' \
	"$metrics_path" >/dev/null 2>&1 || invalid_diagnostic

set +e
classifier_output="$(
	"$classifier_executable" classify-phase35-http \
		--metrics-input "$metrics_path" \
		--body-input "$body_path" \
		--projection-output "$projection_path" \
		--hostname-output "$hostname_path" 2>>"$stderr_path"
)"
classifier_status=$?
set -e

validate_private_file "$projection_path" || invalid_diagnostic
terminal_category="$(
	jq -er '.schema_version == "phase35-http-boundary-v2" and (.terminal_category | type == "string") | if . then input_filename else empty end' \
		"$projection_path" >/dev/null 2>&1 &&
		jq -er '.terminal_category | select(test("^[a-z0-9_]+$"))' "$projection_path"
)" || invalid_diagnostic
case "$terminal_category" in
tcp_connection_failure | tls_handshake_failure | request_transmission_incomplete | response_status_missing | response_headers_missing | non_success_response_status | response_body_missing | response_body_incomplete_or_over_limit | invalid_json | invalid_hostname_schema | ready) ;;
*) invalid_diagnostic ;;
esac

if [[ "$terminal_category" == ready ]]; then
	((classifier_status == 0)) || invalid_diagnostic
	[[ "$classifier_output" == "category=ready" ]] || invalid_diagnostic
	validate_private_file "$hostname_path" || invalid_diagnostic
	printf 'category=ready\n'
	exit 0
fi

((classifier_status != 0)) || invalid_diagnostic
[[ ! -e "$hostname_path" ]] || invalid_diagnostic
printf 'category=%s\n' "$terminal_category"
exit 1
