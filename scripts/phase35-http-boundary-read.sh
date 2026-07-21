#!/usr/bin/env bash
set -euo pipefail

readonly PHASE35_HTTP_SCHEMA="phase35-http-boundary-v1"
readonly MAX_TCP_CONNECT_MILLIS=5000
readonly CURL_MAX_TIME_SECONDS=10
readonly CURL_TIMEOUT_OBSERVATION_GRACE_MILLIS=1000
readonly MAX_OBSERVED_TOTAL_MILLIS=$((\
	CURL_MAX_TIME_SECONDS * 1000 + CURL_TIMEOUT_OBSERVATION_GRACE_MILLIS))
readonly CURL_RECV_ERROR=56
readonly MAX_REQUEST_BYTES=65536
readonly MAX_RESPONSE_HEADER_COUNT=1024
readonly MAX_RESPONSE_HEADER_BYTES=65536
readonly MAX_RESPONSE_BODY_BYTES=65536

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
for private_input in "$body_path" "$headers_path" "$stderr_path" "$metrics_path"; do
	: >"$private_input"
	chmod 600 "$private_input"
done

write_invalid_projection() {
	jq -cn \
		--arg schema_version "$PHASE35_HTTP_SCHEMA" \
		'{schema_version:$schema_version,tcp_connected:false,tls_applicable:false,tls_established:false,tls_verified:false,request_transmission_complete:false,response_status_received:false,response_headers_received:false,response_body_received:false,response_body_complete:false,json_parsed:false,hostname_schema_valid:false,curl_exit_code:0,request_bytes:0,response_header_count:0,response_header_bytes:0,response_body_bytes:0,tcp_connect_millis:0,tls_handshake_millis:0,first_byte_millis:0,total_millis:0,response_status_class:"missing",terminal_category:"http_diagnostic_invalid"}' \
		>"$projection_path"
	chmod 600 "$projection_path"
}

invalid_diagnostic() {
	write_invalid_projection
	printf 'category=http_diagnostic_invalid\n'
	exit 1
}

fixture_authority="${PHASE35_HTTP_FIXTURE_AUTHORITY:-false}"
curl_executable="curl"
if [[ -n "${PHASE35_HTTP_CURL_EXECUTABLE:-}" ]]; then
	[[ "$fixture_authority" == true ]] || invalid_diagnostic
	curl_executable="$PHASE35_HTTP_CURL_EXECUTABLE"
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

readonly curl_write_out=$'scheme_category=%{scheme}\ncurl_exit_code=%{exitcode}\ntcp_connect_seconds=%{time_connect}\ntls_connect_seconds=%{time_appconnect}\nrequest_bytes=%{size_request}\nresponse_status=%{response_code}\nresponse_header_bytes=%{size_header}\nresponse_body_bytes=%{size_download}\ntotal_seconds=%{time_total}\nfirst_byte_seconds=%{time_starttransfer}\ntls_verify_result=%{ssl_verify_result}\n'

set +e
"$curl_executable" \
	--silent \
	--show-error \
	--request GET \
	--http1.1 \
	--noproxy '*' \
	--max-redirs 0 \
	--connect-timeout 5 \
	--max-time "$CURL_MAX_TIME_SECONDS" \
	--max-filesize 65536 \
	--retry 0 \
	--dump-header "$headers_path" \
	--output "$body_path" \
	--stderr "$stderr_path" \
	--write-out "$curl_write_out" \
	--url "${url%/}/api/system/info" >"$metrics_path"
curl_process_status=$?
set -e
chmod 600 "$body_path" "$headers_path" "$stderr_path" "$metrics_path"

awk -F '=' '
	BEGIN {
		expected["scheme_category"] = 1
		expected["curl_exit_code"] = 1
		expected["tcp_connect_seconds"] = 1
		expected["tls_connect_seconds"] = 1
		expected["request_bytes"] = 1
		expected["response_status"] = 1
		expected["response_header_bytes"] = 1
		expected["response_body_bytes"] = 1
		expected["total_seconds"] = 1
		expected["first_byte_seconds"] = 1
		expected["tls_verify_result"] = 1
	}
	NF != 2 || !($1 in expected) || seen[$1]++ { invalid = 1 }
	END {
		if (NR != 11) {
			invalid = 1
		}
		exit invalid
	}
' "$metrics_path" || invalid_diagnostic

metric_value() {
	local key="$1"
	sed -n "s/^${key}=//p" "$metrics_path"
}

integer_pattern='^[0-9]+$'
seconds_pattern='^[0-9]+([.][0-9]+)?$'
curl_exit_code=""
request_bytes=""
response_status=""
response_header_bytes=""
response_body_bytes=""
tls_verify_result=""
tcp_connect_seconds=""
tls_connect_seconds=""
total_seconds=""
first_byte_seconds=""
for key in curl_exit_code request_bytes response_status response_header_bytes response_body_bytes tls_verify_result; do
	value="$(metric_value "$key")"
	[[ "$value" =~ $integer_pattern ]] || invalid_diagnostic
	printf -v "$key" '%s' "$value"
done
for key in tcp_connect_seconds tls_connect_seconds total_seconds first_byte_seconds; do
	value="$(metric_value "$key")"
	[[ "$value" =~ $seconds_pattern ]] || invalid_diagnostic
	printf -v "$key" '%s' "$value"
done
scheme_category="$(metric_value scheme_category | tr '[:upper:]' '[:lower:]')"
case "$scheme_category" in
http | https) ;;
*) invalid_diagnostic ;;
esac
[[ "${url%%:*}" == "$scheme_category" ]] || invalid_diagnostic

seconds_to_millis() {
	awk -v seconds="$1" 'BEGIN {
		millis = seconds * 1000
		if (seconds > 0 && millis < 1) {
			millis = 1
		}
		printf "%.0f\n", millis
	}'
}

seconds_delta_to_millis() {
	awk -v started="$1" -v completed="$2" 'BEGIN {
		if (completed < started) {
			exit 1
		}
		if (completed == 0) {
			print 0
			exit
		}
		millis = (completed - started) * 1000
		if (millis < 1) {
			millis = 1
		}
		printf "%.0f\n", millis
	}'
}

tcp_connect_millis="$(seconds_to_millis "$tcp_connect_seconds")"
tls_connect_millis="$(seconds_to_millis "$tls_connect_seconds")"
total_millis="$(seconds_to_millis "$total_seconds")"
first_byte_millis="$(seconds_to_millis "$first_byte_seconds")"
tls_handshake_millis=0
tls_verification=not_applicable
if [[ "$scheme_category" == https ]]; then
	tls_verification=failed
	if ((tls_connect_millis > 0)); then
		tls_handshake_millis="$(
			seconds_delta_to_millis "$tcp_connect_seconds" "$tls_connect_seconds"
		)" || invalid_diagnostic
		[[ "$tls_verify_result" == 0 ]] || invalid_diagnostic
		tls_verification=verified
	fi
else
	((tls_connect_millis == 0)) || invalid_diagnostic
	[[ "$tls_verify_result" == 0 ]] || invalid_diagnostic
fi

response_header_count="$(
	awk '/^[^[:space:]][^:]*:/ { count += 1 } END { print count + 0 }' "$headers_path"
)"
actual_header_bytes="$(wc -c <"$headers_path" | tr -d ' ')"
actual_body_bytes="$(wc -c <"$body_path" | tr -d ' ')"
((curl_process_status == curl_exit_code)) || invalid_diagnostic
((curl_exit_code <= 255)) || invalid_diagnostic
((tcp_connect_millis <= MAX_TCP_CONNECT_MILLIS)) || invalid_diagnostic
((tls_handshake_millis <= MAX_OBSERVED_TOTAL_MILLIS)) || invalid_diagnostic
((total_millis <= MAX_OBSERVED_TOTAL_MILLIS)) || invalid_diagnostic
((first_byte_millis <= total_millis)) || invalid_diagnostic
((tcp_connect_millis <= total_millis)) || invalid_diagnostic
((tls_connect_millis <= total_millis)) || invalid_diagnostic
((request_bytes <= MAX_REQUEST_BYTES)) || invalid_diagnostic
((response_header_count <= MAX_RESPONSE_HEADER_COUNT)) || invalid_diagnostic
((response_header_bytes <= MAX_RESPONSE_HEADER_BYTES)) || invalid_diagnostic
((response_body_bytes <= MAX_RESPONSE_BODY_BYTES)) || invalid_diagnostic
((actual_header_bytes == response_header_bytes)) || invalid_diagnostic
((actual_body_bytes == response_body_bytes)) || invalid_diagnostic
((response_status == 0 || (response_status >= 100 && response_status <= 599))) ||
	invalid_diagnostic

if ((tcp_connect_millis == 0)); then
	((curl_exit_code != CURL_RECV_ERROR && request_bytes == 0 && \
	response_status == 0 && response_header_count == 0 && \
	response_header_bytes == 0 && response_body_bytes == 0 && first_byte_millis == 0)) ||
		invalid_diagnostic
fi
if ((request_bytes == 0 && curl_exit_code != CURL_RECV_ERROR)); then
	((response_status == 0 && response_header_count == 0 && \
	response_header_bytes == 0 && response_body_bytes == 0 && first_byte_millis == 0)) ||
		invalid_diagnostic
fi
if ((response_status == 0)); then
	((response_header_count == 0 && response_header_bytes == 0 && \
	response_body_bytes == 0 && first_byte_millis == 0)) ||
		invalid_diagnostic
fi
((response_header_count == 0 && response_header_bytes == 0)) ||
	((response_header_count > 0 && response_header_bytes > 0)) ||
	invalid_diagnostic
((response_header_count > 0 || response_body_bytes == 0)) || invalid_diagnostic
((response_body_bytes == 0 || first_byte_millis > 0)) || invalid_diagnostic

jq -cn \
	--arg scheme_category "$scheme_category" \
	--argjson curl_exit_code "$curl_exit_code" \
	--argjson tcp_connect_millis "$tcp_connect_millis" \
	--argjson tls_handshake_millis "$tls_handshake_millis" \
	--argjson request_bytes "$request_bytes" \
	--argjson response_status "$response_status" \
	--argjson response_header_count "$response_header_count" \
	--argjson response_header_bytes "$response_header_bytes" \
	--argjson response_body_bytes "$response_body_bytes" \
	--argjson total_millis "$total_millis" \
	--argjson first_byte_millis "$first_byte_millis" \
	--arg tls_verification "$tls_verification" \
	'{scheme_category:$scheme_category,curl_exit_code:$curl_exit_code,tcp_connect_millis:$tcp_connect_millis,tls_handshake_millis:$tls_handshake_millis,request_bytes:$request_bytes,response_status:$response_status,response_header_count:$response_header_count,response_header_bytes:$response_header_bytes,response_body_bytes:$response_body_bytes,total_millis:$total_millis,first_byte_millis:$first_byte_millis,tls_verification:$tls_verification}' \
	>"$metrics_path"
chmod 600 "$metrics_path"

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
	jq -er '.schema_version == "phase35-http-boundary-v1" and (.terminal_category | type == "string") | if . then input_filename else empty end' \
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
