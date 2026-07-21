#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE35_HTTP_TEST_CURL_DISPATCH:-false}" == true ]]; then
	state_dir="${PHASE35_HTTP_TEST_STATE:?}"
	scenario="${PHASE35_HTTP_TEST_SCENARIO:?}"
	printf 'invoked\n' >>"${state_dir}/invocations"
	printf 'arg=%s\n' "$@" >>"${state_dir}/argv"

	body_path=""
	header_path=""
	stderr_path=""
	while (($#)); do
		case "$1" in
		--output)
			body_path="$2"
			shift 2
			;;
		--dump-header)
			header_path="$2"
			shift 2
			;;
		--stderr)
			stderr_path="$2"
			shift 2
			;;
		*)
			shift
			;;
		esac
	done
	[[ -n "$body_path" && -n "$header_path" && -n "$stderr_path" ]]

	scheme=http
	exit_code=0
	actual_exit=0
	tcp_seconds=0.005
	tls_seconds=0.000
	request_bytes=128
	response_status=200
	total_seconds=0.012
	first_byte_seconds=0.009
	tls_verify_result=0
	body='{"hostname":"fixture-host"}'
	headers=$'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n'
	extra_metric=""

	case "$scenario" in
	ready) ;;
	submillisecond_ready)
		tcp_seconds=0.0004
		total_seconds=0.0009
		first_byte_seconds=0.0008
		;;
	submillisecond_https_ready)
		scheme=https
		tcp_seconds=0.0002
		tls_seconds=0.0004
		total_seconds=0.0009
		first_byte_seconds=0.0008
		;;
	submillisecond_non_success)
		tcp_seconds=0.0004
		total_seconds=0.0009
		first_byte_seconds=0.0008
		response_status=503
		headers=$'HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n'
		;;
	tcp_connection_failure)
		exit_code=7
		actual_exit=7
		tcp_seconds=0.000
		request_bytes=0
		response_status=0
		first_byte_seconds=0.000
		body=""
		headers=""
		;;
	tls_handshake_failure)
		scheme=https
		exit_code=35
		actual_exit=35
		request_bytes=0
		response_status=0
		first_byte_seconds=0.000
		tls_verify_result=1
		body=""
		headers=""
		;;
	request_transmission_incomplete)
		exit_code=55
		actual_exit=55
		request_bytes=0
		response_status=0
		first_byte_seconds=0.000
		body=""
		headers=""
		;;
	response_status_missing)
		exit_code=52
		actual_exit=52
		response_status=0
		first_byte_seconds=0.000
		body=""
		headers=""
		;;
	response_headers_missing)
		exit_code=8
		actual_exit=8
		body=""
		headers=""
		;;
	non_success_response_status)
		response_status=503
		headers=$'HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n'
		;;
	response_body_missing)
		body=""
		headers=$'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 0\r\n\r\n'
		;;
	response_body_incomplete_or_over_limit)
		exit_code=18
		actual_exit=18
		;;
	invalid_json)
		body='{'
		headers=$'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1\r\n\r\n'
		;;
	invalid_hostname_schema)
		body='{"hostname":42}'
		headers=$'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\n\r\n'
		;;
	malformed_extra)
		extra_metric=$'unexpected_key=1\n'
		;;
	process_status_mismatch)
		exit_code=7
		actual_exit=28
		tcp_seconds=0.000
		request_bytes=0
		response_status=0
		first_byte_seconds=0.000
		body=""
		headers=""
		;;
	body_size_mismatch) ;;
	*)
		exit 96
		;;
	esac

	printf '%s' "$body" >"$body_path"
	printf '%s' "$headers" >"$header_path"
	printf '%s\n' 'raw-curl-error-canary' >"$stderr_path"
	body_bytes="$(wc -c <"$body_path" | tr -d ' ')"
	header_bytes="$(wc -c <"$header_path" | tr -d ' ')"
	if [[ "$scenario" == body_size_mismatch ]]; then
		body_bytes=$((body_bytes + 1))
	fi

	printf 'scheme_category=%s\n' "$scheme"
	printf 'curl_exit_code=%s\n' "$exit_code"
	printf 'tcp_connect_seconds=%s\n' "$tcp_seconds"
	printf 'tls_connect_seconds=%s\n' "$tls_seconds"
	printf 'request_bytes=%s\n' "$request_bytes"
	printf 'response_status=%s\n' "$response_status"
	printf 'response_header_bytes=%s\n' "$header_bytes"
	printf 'response_body_bytes=%s\n' "$body_bytes"
	printf 'total_seconds=%s\n' "$total_seconds"
	printf 'first_byte_seconds=%s\n' "$first_byte_seconds"
	printf 'tls_verify_result=%s\n' "$tls_verify_result"
	printf '%s' "$extra_metric"
	exit "$actual_exit"
fi

if [[ "${PHASE35_HTTP_TEST_BLOCKED_TOOL_DISPATCH:-false}" == true ]]; then
	printf '%s\n' "${0##*/}" >>"${PHASE35_HTTP_TEST_NESTED_CALLS:?}"
	exit 97
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly test_entrypoint="${script_dir}/phase35-http-boundary-read-test.sh"
readonly adapter="${script_dir}/phase35-http-boundary-read.sh"
readonly test_root="${TEST_TMPDIR:-$(mktemp -d)}/phase35-http-boundary"
readonly raw_origin_canary="http://raw-origin-canary.invalid:18443"
mkdir -p "$test_root"
chmod 700 "$test_root"

fail_test() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

assert_line() {
	rg -Fqx -- "$2" "$1" || fail_test "missing line in ${1##*/}: $2"
}

assert_absent() {
	[[ ! -f "$1" ]] || ! rg -q -- "$2" "$1" ||
		fail_test "unexpected pattern in ${1##*/}: $2"
}

prepare_case() {
	local name="$1"
	case_dir="${test_root}/${name}"
	protected_root="${case_dir}/protected"
	state_dir="${case_dir}/state"
	curl_bin="${case_dir}/bin/curl"
	nested_bin="${case_dir}/blocked"
	stdout_file="${case_dir}/stdout"
	stderr_file="${case_dir}/stderr"
	mkdir -p "$protected_root" "$state_dir" "$(dirname "$curl_bin")" "$nested_bin"
	chmod 700 "$case_dir" "$protected_root" "$state_dir" "$(dirname "$curl_bin")" "$nested_bin"
	ln -s "$test_entrypoint" "$curl_bin"
	ln -s "$test_entrypoint" "${nested_bin}/just"
	ln -s "$test_entrypoint" "${nested_bin}/bazel"
	: >"$stdout_file"
	: >"$stderr_file"
	: >"${state_dir}/nested-calls"
	chmod 600 "$stdout_file" "$stderr_file" "${state_dir}/nested-calls"
}

run_case() {
	local scenario="$1"
	shift
	local case_url="$raw_origin_canary"
	if [[ "$scenario" == tls_handshake_failure || "$scenario" == submillisecond_https_ready ]]; then
		case_url="https://raw-origin-canary.invalid:18443"
	fi
	set +e
	BUILD_WORKSPACE_DIRECTORY="${script_dir}/.." \
		PATH="${nested_bin}:${PATH}" \
		PHASE35_HTTP_FIXTURE_AUTHORITY=true \
		PHASE35_HTTP_CURL_EXECUTABLE="$curl_bin" \
		PHASE35_HTTP_TEST_CURL_DISPATCH=true \
		PHASE35_HTTP_TEST_STATE="$state_dir" \
		PHASE35_HTTP_TEST_SCENARIO="$scenario" \
		PHASE35_HTTP_TEST_BLOCKED_TOOL_DISPATCH=true \
		PHASE35_HTTP_TEST_NESTED_CALLS="${state_dir}/nested-calls" \
		"$adapter" \
		"label=original" \
		"protected-root=${protected_root}" \
		"url=${case_url}" \
		"$@" >"$stdout_file" 2>"$stderr_file"
	run_status=$?
	set -e
}

assert_exact_request_contract() {
	[[ "$(rg -c '^invoked$' "${state_dir}/invocations")" == 1 ]] ||
		fail_test "curl was not invoked exactly once"
	local argv="${state_dir}/argv"
	local option
	for option in \
		'arg=--request' \
		'arg=GET' \
		'arg=--http1.1' \
		"arg=--noproxy" \
		"arg=*" \
		'arg=--max-redirs' \
		'arg=0' \
		'arg=--connect-timeout' \
		'arg=5' \
		'arg=--max-time' \
		'arg=10' \
		'arg=--max-filesize' \
		'arg=65536' \
		'arg=--retry'; do
		assert_line "$argv" "$option"
	done
	[[ "$(rg -c '^arg=0$' "$argv")" -ge 2 ]] || fail_test "zero redirect/retry values missing"
	assert_absent "$argv" '^arg=--location$'
	assert_absent "$argv" '^arg=--fail$'
}

assert_private_artifacts() {
	local read_dir="${protected_root}/http-original"
	[[ "$(file_mode "$read_dir")" == 700 ]] || fail_test "read directory mode is not 0700"
	while IFS= read -r file; do
		[[ "$(file_mode "$file")" == 600 ]] || fail_test "private file mode is not 0600"
	done < <(find "$read_dir" -type f)
	local allowed='^(body|headers|stderr|metrics|projection|private-hostname)$'
	while IFS= read -r name; do
		[[ "$name" =~ $allowed ]] || fail_test "unexpected private artifact: $name"
	done < <(find "$read_dir" -type f -exec basename {} \;)
}

assert_projection_allowlist() {
	local projection="${protected_root}/http-original/projection"
	local actual
	actual="$(jq -r 'keys | sort | join(",")' "$projection")"
	local expected
	expected="curl_exit_code,first_byte_millis,hostname_schema_valid,json_parsed,request_bytes,request_transmission_complete,response_body_bytes,response_body_complete,response_body_received,response_header_bytes,response_header_count,response_headers_received,response_status_class,response_status_received,schema_version,tcp_connect_millis,tcp_connected,terminal_category,tls_applicable,tls_established,tls_handshake_millis,tls_verified,total_millis"
	[[ "$actual" == "$expected" ]] || fail_test "projection field allowlist drifted"
	[[ "$(jq -r '.schema_version' "$projection")" == phase35-http-boundary-v1 ]] ||
		fail_test "projection schema drifted"
}

test_terminal_matrix_and_invalid_fallback() {
	local scenario
	local expected
	for scenario in \
		tcp_connection_failure \
		tls_handshake_failure \
		request_transmission_incomplete \
		response_status_missing \
		response_headers_missing \
		non_success_response_status \
		response_body_missing \
		response_body_incomplete_or_over_limit \
		invalid_json \
		invalid_hostname_schema \
		ready \
		submillisecond_ready \
		submillisecond_https_ready \
		submillisecond_non_success \
		malformed_extra \
		process_status_mismatch \
		body_size_mismatch; do
		# Arrange
		prepare_case "$scenario"

		# Act
		run_case "$scenario"

		# Assert
		expected="$scenario"
		case "$scenario" in
		submillisecond_ready | submillisecond_https_ready)
			expected=ready
			;;
		submillisecond_non_success)
			expected=non_success_response_status
			;;
		malformed_extra | process_status_mismatch | body_size_mismatch)
			expected=http_diagnostic_invalid
			;;
		esac
		if [[ "$expected" == ready ]]; then
			[[ "$run_status" == 0 ]] || fail_test "${scenario} ready adapter case failed"
		else
			[[ "$run_status" != 0 ]] || fail_test "${scenario} unexpectedly succeeded"
		fi
		assert_line "$stdout_file" "category=${expected}"
		[[ "$(jq -r '.terminal_category' "${protected_root}/http-original/projection")" == "$expected" ]] ||
			fail_test "projection category mismatch for ${scenario}"
		assert_exact_request_contract
		assert_private_artifacts
		assert_projection_allowlist
		[[ ! -s "${state_dir}/nested-calls" ]] || fail_test "adapter invoked nested build tool"
		assert_absent "$stdout_file" 'raw-origin-canary|fixture-host|raw-curl-error-canary'
		assert_absent "$stderr_file" 'raw-origin-canary|fixture-host|raw-curl-error-canary'
		assert_absent "${protected_root}/http-original/projection" 'raw-origin-canary|fixture-host|raw-curl-error-canary'
	done
}

test_submillisecond_observation_preserves_terminal_precedence() {
	# Arrange
	prepare_case attempt_12_terminal_precedence

	# Act
	run_case submillisecond_non_success

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "non-success response unexpectedly became ready"
	assert_line "$stdout_file" 'category=non_success_response_status'
	jq -e '.terminal_category == "non_success_response_status"' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "sub-millisecond observation lost its precise terminal category"
}

test_duration_quantization_preserves_presence() {
	# Arrange / Act: exact zero retains the absence sentinel.
	prepare_case exact_zero_duration
	run_case tcp_connection_failure

	# Assert
	jq -e '.tcp_connect_millis == 0' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "exact zero TCP duration did not remain absent"

	# Arrange / Act: ordinary durations retain their current integer projection.
	prepare_case ordinary_durations
	run_case ready

	# Assert
	jq -e \
		'.tcp_connect_millis == 5 and .tls_handshake_millis == 0 and .first_byte_millis == 9 and .total_millis == 12' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "ordinary duration projection changed"

	# Arrange / Act: positive sub-millisecond HTTP durations remain present.
	prepare_case attempt_12_submillisecond_http
	run_case submillisecond_ready

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "attempt-12 boundary did not reach ready"
	jq -e \
		'.tcp_connect_millis == 1 and .tls_handshake_millis == 0 and .first_byte_millis == 1 and .total_millis == 1 and .terminal_category == "ready"' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "sub-millisecond HTTP durations lost presence"

	# Arrange / Act: derived positive TLS duration also remains present.
	prepare_case submillisecond_https
	run_case submillisecond_https_ready

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "sub-millisecond HTTPS boundary did not reach ready"
	jq -e \
		'.tcp_connect_millis == 1 and .tls_handshake_millis == 1 and .first_byte_millis == 1 and .total_millis == 1 and .terminal_category == "ready"' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "sub-millisecond HTTPS durations lost presence"
}

test_ready_separates_private_hostname() {
	# Arrange
	prepare_case ready_private_hostname

	# Act
	run_case ready

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "ready private-hostname case failed"
	[[ "$(<"${protected_root}/http-original/private-hostname")" == fixture-host ]] ||
		fail_test "private hostname was not persisted"
	assert_absent "${protected_root}/http-original/projection" 'fixture-host'
}

test_unauthorized_override_persists_invalid_projection_without_curl() {
	# Arrange
	prepare_case unauthorized_override

	# Act
	set +e
	PHASE35_HTTP_CURL_EXECUTABLE="$curl_bin" \
		"$adapter" \
		"label=original" \
		"protected-root=${protected_root}" \
		"url=${raw_origin_canary}" >"$stdout_file" 2>"$stderr_file"
	run_status=$?
	set -e

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "unauthorized override unexpectedly succeeded"
	assert_line "$stdout_file" 'category=http_diagnostic_invalid'
	[[ "$(jq -r '.terminal_category' "${protected_root}/http-original/projection")" == http_diagnostic_invalid ]] ||
		fail_test "unauthorized override did not persist its invalid projection"
	[[ ! -e "${state_dir}/invocations" ]] || fail_test "unauthorized override invoked curl"
	assert_private_artifacts
	assert_projection_allowlist
	assert_absent "$stdout_file" 'raw-origin-canary'
	assert_absent "$stderr_file" 'raw-origin-canary'
	assert_absent "${protected_root}/http-original/projection" 'raw-origin-canary'
}

test_terminal_matrix_and_invalid_fallback
test_duration_quantization_preserves_presence
test_submillisecond_observation_preserves_terminal_precedence
test_ready_separates_private_hostname
test_unauthorized_override_persists_invalid_projection_without_curl

printf 'phase35 HTTP boundary adapter tests passed\n'
