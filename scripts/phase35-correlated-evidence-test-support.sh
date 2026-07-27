#!/usr/bin/env bash
# Shared paths, fixtures, assertions, and source-shape checks for Phase 35 tests.

readonly document_source="${script_dir}/phase35-correlated-evidence-document.sh"
readonly active_plan_root=".planning/""phases/"
readonly test_entrypoint="${script_dir}/phase35-correlated-evidence-test.sh"
readonly supervisor="${script_dir}/phase35-correlated-evidence.sh"
readonly fixture="${script_dir}/phase35-correlated-evidence-fixture.sh"
readonly justfile="${script_dir}/../Justfile"
readonly sdkconfig_defaults="${script_dir}/../firmware/bitaxe/sdkconfig.defaults"
readonly http_api_source="${script_dir}/../firmware/bitaxe/src/http_api/websocket.rs"
readonly test_root="${TEST_TMPDIR:-$(mktemp -d)}/phase35"
readonly workspace="${test_root}/workspace"
readonly source_commit="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
readonly reference_commit="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
readonly minimum_main_task_stack_bytes=16384
active_scenario=""

mkdir -p "$workspace"

if rg -q -F "$active_plan_root" "$document_source"; then
	printf 'FAIL: Phase 35 evidence validation depends on active planning files\n' >&2
	exit 1
fi

fail_test() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local file="$1"
	local pattern="$2"
	rg -q "$pattern" "$file" || fail_test "expected ${pattern} in ${file##*/}"
}

assert_line() {
	local file="$1"
	local expected="$2"
	rg -Fqx -- "$expected" "$file" || fail_test "expected exact line in ${file##*/}: ${expected}"
}

assert_absent() {
	local file="$1"
	local pattern="$2"
	[[ ! -f "$file" ]] || ! rg -q "$pattern" "$file" ||
		fail_test "unexpected ${pattern} in ${file##*/}"
}

assert_count() {
	local expected="$1"
	local pattern="$2"
	local file="$3"
	local actual=0
	if [[ -f "$file" ]]; then
		actual="$(rg -c "^${pattern}$" "$file" || printf '0')"
	fi
	[[ "$actual" == "$expected" ]] ||
		fail_test "expected ${expected} ${pattern} calls, found ${actual}"
}

line_number() {
	local pattern="$1"
	local file="$2"
	rg -n "^${pattern}$" "$file" | head -1 | cut -d: -f1
}

file_mode() {
	local file="$1"
	stat -f '%Lp' "$file" 2>/dev/null || stat -c '%a' "$file"
}

path_metadata() {
	local path="$1"
	stat -f '%HT:%Lp:%z:%m:%c' "$path" 2>/dev/null ||
		stat -c '%F:%a:%s:%Y:%Z' "$path"
}

file_digest() {
	shasum -a 256 "$1" | awk '{print $1}'
}

text_digest() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

test_main_task_stack_capacity() {
	# Arrange
	local assignment_count
	assignment_count="$(rg -c '^CONFIG_ESP_MAIN_TASK_STACK_SIZE=[0-9]+$' "$sdkconfig_defaults")"
	[[ "$assignment_count" == 1 ]] ||
		fail_test "expected one ESP main-task stack assignment"
	local configured_stack_bytes
	configured_stack_bytes="$(
		sed -n 's/^CONFIG_ESP_MAIN_TASK_STACK_SIZE=//p' "$sdkconfig_defaults"
	)"

	# Act
	local capacity_is_sufficient=false
	if [[ "$configured_stack_bytes" =~ ^[0-9]+$ ]] &&
		((configured_stack_bytes >= minimum_main_task_stack_bytes)); then
		capacity_is_sufficient=true
	fi

	# Assert
	[[ "$capacity_is_sufficient" == true ]] ||
		fail_test "ESP main-task stack is below the Phase 35 runtime minimum"
}

test_websocket_background_writes_are_serialized_in_httpd_context() {
	# Arrange
	local queued_callback
	queued_callback="$(
		sed -n '/^unsafe extern "C" fn send_queued_websocket_frame/,/^}/p' \
			"$http_api_source"
	)"
	[[ -n "$queued_callback" ]] || fail_test "missing queued WebSocket callback"

	# Act
	local lease_check_line protocol_check_line send_line
	lease_check_line="$(
		rg -n 'websocket_api::is_current' <<<"$queued_callback" | head -1 | cut -d: -f1
	)"
	protocol_check_line="$(
		rg -n 'httpd_ws_get_fd_info' <<<"$queued_callback" | head -1 | cut -d: -f1
	)"
	send_line="$(
		rg -n 'httpd_ws_send_frame_async' <<<"$queued_callback" | head -1 | cut -d: -f1
	)"

	# Assert
	[[ "$(rg -c 'httpd_ws_send_frame_async' "$http_api_source")" == 1 ]] ||
		fail_test "background WebSocket writes bypass the queued callback"
	[[ "$(rg -c 'httpd_queue_work' "$http_api_source")" == 1 ]] ||
		fail_test "WebSocket queue ownership is ambiguous"
	rg -q 'websocket_api::is_current' <<<"$queued_callback" ||
		fail_test "queued WebSocket callback does not recheck the connection lease"
	rg -q 'HTTPD_WS_CLIENT_WEBSOCKET' <<<"$queued_callback" ||
		fail_test "queued WebSocket callback does not recheck protocol state"
	[[ -n "$lease_check_line" && -n "$protocol_check_line" && -n "$send_line" ]] ||
		fail_test "queued WebSocket callback ordering markers are incomplete"
	[[ "$lease_check_line" -lt "$protocol_check_line" && "$protocol_check_line" -lt "$send_line" ]] ||
		fail_test "queued WebSocket callback sends before validating the reused descriptor"
	rg -q 'free_websocket_session_context' "$http_api_source" ||
		fail_test "WebSocket disconnect cleanup is not registered"
	rg -q 'unregister_if_current' "$http_api_source" ||
		fail_test "WebSocket cleanup can remove a replacement connection"
}
