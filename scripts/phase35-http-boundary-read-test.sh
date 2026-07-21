#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE35_HTTP_TEST_PROBE_DISPATCH:-false}" == true ]]; then
	state_dir="${PHASE35_HTTP_TEST_STATE:?}"
	scenario="${PHASE35_HTTP_TEST_SCENARIO:?}"
	printf 'invoked\n' >>"${state_dir}/invocations"
	printf 'arg=%s\n' "$@" >>"${state_dir}/argv"

	body_path=""
	header_path=""
	metrics_path=""
	[[ "${1:-}" == probe-phase35-http ]]
	shift
	while (($#)); do
		case "$1" in
		--body-output)
			body_path="$2"
			shift 2
			;;
		--headers-output)
			header_path="$2"
			shift 2
			;;
		--metrics-output)
			metrics_path="$2"
			shift 2
			;;
		--url) shift 2 ;;
		*)
			shift
			;;
		esac
	done
	[[ -n "$body_path" && -n "$header_path" && -n "$metrics_path" ]]

	scheme=http
	transport_outcome=complete
	actual_exit=0
	tcp_millis=5
	tls_millis=0
	request_send_complete_millis=7
	request_bytes=93
	response_status=200
	total_millis=12
	first_byte_millis=9
	tls_verification=not_applicable
	body='{"hostname":"fixture-host"}'
	headers=$'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n'
	extra_metric=false

	case "$scenario" in
	ready) ;;
	attempt13_timeout_boundary)
		transport_outcome=response_timeout
		response_status=0
		total_millis=10003
		first_byte_millis=0
		body=""
		headers=""
		;;
	attempt14_receive_failure)
		transport_outcome=receive_failed
		tcp_millis=261
		request_send_complete_millis=262
		response_status=0
		total_millis=6539
		first_byte_millis=0
		body=""
		headers=""
		;;
	receive_failure_after_partial_response)
		transport_outcome=receive_failed
		;;
	submillisecond_ready)
		tcp_millis=1
		request_send_complete_millis=1
		total_millis=1
		first_byte_millis=1
		;;
	submillisecond_https_ready)
		scheme=https
		tcp_millis=1
		tls_millis=1
		request_send_complete_millis=1
		total_millis=1
		first_byte_millis=1
		tls_verification=verified
		;;
	submillisecond_non_success)
		tcp_millis=1
		request_send_complete_millis=1
		total_millis=1
		first_byte_millis=1
		response_status=503
		headers=$'HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n'
		;;
	tcp_connection_failure)
		transport_outcome=tcp_connection_failure
		tcp_millis=0
		request_send_complete_millis=0
		request_bytes=0
		response_status=0
		first_byte_millis=0
		body=""
		headers=""
		;;
	tls_handshake_failure)
		scheme=https
		transport_outcome=tls_handshake_failure
		request_send_complete_millis=0
		request_bytes=0
		response_status=0
		first_byte_millis=0
		tls_verification=failed
		body=""
		headers=""
		;;
	request_transmission_incomplete)
		transport_outcome=request_send_failure
		request_send_complete_millis=0
		request_bytes=17
		response_status=0
		first_byte_millis=0
		body=""
		headers=""
		;;
	response_status_missing)
		transport_outcome=receive_failed
		response_status=0
		first_byte_millis=0
		body=""
		headers=""
		;;
	response_headers_missing)
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
		transport_outcome=response_over_limit
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
		extra_metric=true
		;;
	process_status_mismatch)
		actual_exit=1
		;;
	body_size_mismatch) ;;
	total_duration_out_of_bound) total_millis=11001 ;;
	*)
		exit 96
		;;
	esac

	printf '%s' "$body" >"$body_path"
	printf '%s' "$headers" >"$header_path"
	chmod 600 "$body_path" "$header_path"
	body_bytes="$(wc -c <"$body_path" | tr -d ' ')"
	header_bytes="$(wc -c <"$header_path" | tr -d ' ')"
	header_count="$(awk '/^[^[:space:]][^:]*:/ { count += 1 } END { print count + 0 }' "$header_path")"
	if [[ "$scenario" == body_size_mismatch ]]; then
		body_bytes=$((body_bytes + 1))
	fi
	jq -cn \
		--arg scheme_category "$scheme" \
		--arg transport_outcome "$transport_outcome" \
		--argjson tcp_connect_millis "$tcp_millis" \
		--argjson tls_handshake_millis "$tls_millis" \
		--argjson request_send_complete_millis "$request_send_complete_millis" \
		--argjson request_bytes "$request_bytes" \
		--argjson response_status "$response_status" \
		--argjson response_header_count "$header_count" \
		--argjson response_header_bytes "$header_bytes" \
		--argjson response_body_bytes "$body_bytes" \
		--argjson total_millis "$total_millis" \
		--argjson first_byte_millis "$first_byte_millis" \
		--arg tls_verification "$tls_verification" \
		--argjson extra_metric "$extra_metric" \
		'{scheme_category:$scheme_category,transport_outcome:$transport_outcome,tcp_connect_millis:$tcp_connect_millis,tls_handshake_millis:$tls_handshake_millis,request_send_complete_millis:$request_send_complete_millis,request_bytes:$request_bytes,response_status:$response_status,response_header_count:$response_header_count,response_header_bytes:$response_header_bytes,response_body_bytes:$response_body_bytes,total_millis:$total_millis,first_byte_millis:$first_byte_millis,tls_verification:$tls_verification} + (if $extra_metric then {unexpected_key:1} else {} end)' \
		>"$metrics_path"
	chmod 600 "$metrics_path"
	printf 'status=probe_complete\n'
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
	probe_bin="${case_dir}/bin/probe"
	nested_bin="${case_dir}/blocked"
	stdout_file="${case_dir}/stdout"
	stderr_file="${case_dir}/stderr"
	mkdir -p "$protected_root" "$state_dir" "$(dirname "$probe_bin")" "$nested_bin"
	chmod 700 "$case_dir" "$protected_root" "$state_dir" "$(dirname "$probe_bin")" "$nested_bin"
	ln -s "$test_entrypoint" "$probe_bin"
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
		PHASE35_HTTP_PROBE_EXECUTABLE="$probe_bin" \
		PHASE35_HTTP_TEST_PROBE_DISPATCH=true \
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
		fail_test "probe was not invoked exactly once"
	local argv="${state_dir}/argv"
	local option
	for option in \
		'arg=probe-phase35-http' \
		'arg=--url' \
		'arg=--metrics-output' \
		'arg=--headers-output' \
		'arg=--body-output'; do
		assert_line "$argv" "$option"
	done
	assert_absent "$argv" '^arg=--request$'
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
	expected="first_byte_millis,hostname_schema_valid,json_parsed,request_bytes,request_send_complete_millis,request_transmission_complete,response_body_bytes,response_body_complete,response_body_received,response_header_bytes,response_header_count,response_headers_received,response_status_class,response_status_received,schema_version,tcp_connect_millis,tcp_connected,terminal_category,tls_applicable,tls_established,tls_handshake_millis,tls_verified,total_millis,transport_outcome"
	[[ "$actual" == "$expected" ]] || fail_test "projection field allowlist drifted"
	[[ "$(jq -r '.schema_version' "$projection")" == phase35-http-boundary-v2 ]] ||
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
		receive_failure_after_partial_response \
		invalid_json \
		invalid_hostname_schema \
		ready \
		submillisecond_ready \
		submillisecond_https_ready \
		submillisecond_non_success \
		malformed_extra \
		process_status_mismatch \
		body_size_mismatch \
		total_duration_out_of_bound; do
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
		receive_failure_after_partial_response)
			expected=response_body_incomplete_or_over_limit
			;;
		malformed_extra | process_status_mismatch | body_size_mismatch | total_duration_out_of_bound)
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
		assert_absent "$stdout_file" 'raw-origin-canary|fixture-host'
		assert_absent "$stderr_file" 'raw-origin-canary|fixture-host'
		assert_absent "${protected_root}/http-original/projection" 'raw-origin-canary|fixture-host'
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

test_completed_send_timeout_reaches_response_boundary() {
	# Arrange
	prepare_case attempt_13_timeout_boundary

	# Act
	run_case attempt13_timeout_boundary

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "missing response unexpectedly became ready"
	assert_line "$stdout_file" 'category=response_status_missing'
	jq -e '.terminal_category == "response_status_missing" and .request_transmission_complete' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "completed send timeout lost its response boundary"
}

test_receive_failure_does_not_reclassify_as_send_failure() {
	# Arrange
	prepare_case attempt_14_receive_failure

	# Act
	run_case attempt14_receive_failure

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "missing response status unexpectedly became ready"
	assert_line "$stdout_file" 'category=response_status_missing'
	jq -e \
		'.terminal_category == "response_status_missing" and
		 .tcp_connected and
		 .request_transmission_complete and
		 .transport_outcome == "receive_failed" and
		 .request_send_complete_millis == 262 and
		 .request_bytes == 93 and
		 .tcp_connect_millis == 261 and
		 .total_millis == 6539' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "receive failure was reclassified as incomplete request transmission"
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

test_unauthorized_override_persists_invalid_projection_without_probe() {
	# Arrange
	prepare_case unauthorized_override

	# Act
	set +e
	PHASE35_HTTP_PROBE_EXECUTABLE="$probe_bin" \
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
	[[ ! -e "${state_dir}/invocations" ]] || fail_test "unauthorized override invoked probe"
	assert_private_artifacts
	assert_projection_allowlist
	assert_absent "$stdout_file" 'raw-origin-canary'
	assert_absent "$stderr_file" 'raw-origin-canary'
	assert_absent "${protected_root}/http-original/projection" 'raw-origin-canary'
}

run_real_loopback_case() {
	local scenario="$1"
	prepare_case "real-loopback-${scenario}"
	local port_file="${case_dir}/loopback-port"
	: >"$port_file"
	chmod 600 "$port_file"
	python3 -c '
import socket
import sys
import time

port_path, scenario = sys.argv[1:]
listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", 0))
listener.listen(1)
with open(port_path, "w", encoding="ascii") as port_file:
    port_file.write(str(listener.getsockname()[1]))
connection, _ = listener.accept()
request = b""
while not request.endswith(b"\r\n\r\n"):
    chunk = connection.recv(256)
    if not chunk:
        break
    request += chunk
if scenario == "ready":
    body = b"{\"hostname\":\"loopback-fixture\"}"
    headers = (
        b"HTTP/1.1 200 OK\r\n"
        b"Content-Type: application/json\r\n"
        + b"Content-Length: " + str(len(body)).encode("ascii") + b"\r\n"
        b"Connection: close\r\n\r\n"
    )
    connection.sendall(headers + body)
else:
    time.sleep(11)
connection.close()
listener.close()
' "$port_file" "$scenario" &
	local peer_pid=$!
	local attempts=0
	while [[ ! -s "$port_file" ]]; do
		((attempts += 1))
		((attempts <= 100)) || fail_test "loopback peer did not publish readiness"
		sleep 0.01
	done
	local port
	port="$(<"$port_file")"
	set +e
	BUILD_WORKSPACE_DIRECTORY="${script_dir}/.." \
		PATH="${nested_bin}:${PATH}" \
		PHASE35_HTTP_TEST_BLOCKED_TOOL_DISPATCH=true \
		PHASE35_HTTP_TEST_NESTED_CALLS="${state_dir}/nested-calls" \
		"$adapter" \
		"label=original" \
		"protected-root=${protected_root}" \
		"url=http://127.0.0.1:${port}" >"$stdout_file" 2>"$stderr_file"
	run_status=$?
	wait "$peer_pid"
	set -e
}

test_real_probe_crosses_direct_and_runfiles_boundaries() {
	# Arrange / Act: the real probe sends a complete GET and receives a valid response.
	run_real_loopback_case ready

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "real loopback ready probe failed"
	assert_line "$stdout_file" 'category=ready'
	jq -e \
		'.terminal_category == "ready" and
		 .request_transmission_complete and
		 .request_send_complete_millis > 0 and
		 .request_bytes > 0 and
		 .transport_outcome == "complete"' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "real loopback ready projection lost send evidence"
	assert_absent "$stdout_file" '127[.]0[.]0[.]1|loopback-fixture'
	assert_absent "$stderr_file" '127[.]0[.]0[.]1|loopback-fixture'
	assert_absent "${protected_root}/http-original/projection" '127[.]0[.]0[.]1|loopback-fixture'

	# Arrange / Act: the peer receives the request but deliberately sends no response.
	run_real_loopback_case timeout

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "silent loopback unexpectedly became ready"
	assert_line "$stdout_file" 'category=response_status_missing'
	jq -e \
		'.terminal_category == "response_status_missing" and
		 .request_transmission_complete and
		 .request_send_complete_millis > 0 and
		 .request_bytes > 0 and
		 .transport_outcome == "response_timeout"' \
		"${protected_root}/http-original/projection" >/dev/null ||
		fail_test "silent loopback was reclassified as incomplete transmission"
	[[ ! -s "${state_dir}/nested-calls" ]] || fail_test "real probe invoked nested build tool"
}

test_terminal_matrix_and_invalid_fallback
test_duration_quantization_preserves_presence
test_submillisecond_observation_preserves_terminal_precedence
test_completed_send_timeout_reaches_response_boundary
test_receive_failure_does_not_reclassify_as_send_failure
test_ready_separates_private_hostname
test_unauthorized_override_persists_invalid_projection_without_probe
test_real_probe_crosses_direct_and_runfiles_boundaries

printf 'phase35 HTTP boundary adapter tests passed\n'
