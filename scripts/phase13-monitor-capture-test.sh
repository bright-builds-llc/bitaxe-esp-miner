#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly monitor_script="${PHASE13_MONITOR_CAPTURE_SCRIPT:-${script_dir}/phase13-monitor-capture.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase13-monitor-capture-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_fast_espflash() {
	local bin_dir="$1"
	mkdir -p "$bin_dir"

	write_executable "${bin_dir}/espflash" 'printf "espflash-args:"
printf " %s" "$@"
printf "\n"
case " $* " in
*" monitor --chip esp32s3 --port /dev/test --non-interactive "* | *" monitor --chip esp32s3 --port /dev/test --non-interactive"*)
  ;;
*)
  printf "unexpected monitor command\n" >&2
  exit 2
  ;;
esac
'
}

create_slow_espflash() {
	local bin_dir="$1"
	mkdir -p "$bin_dir"

	write_executable "${bin_dir}/espflash" 'printf "slow-monitor-start\n"
sleep 10
'
}

create_descendant_espflash() {
	local bin_dir="$1"
	mkdir -p "$bin_dir"

	# shellcheck disable=SC2016 # The generated fixture expands these values at runtime.
	write_executable "${bin_dir}/phase13-test-watcher" 'trap "" INT TERM
printf "%s\n" "$$" >"${PHASE13_TEST_WATCHER_PID_FILE:?}"
while true; do
  sleep 1
done
'
	# shellcheck disable=SC2016 # The generated fixture expands these values at runtime.
	write_executable "${bin_dir}/espflash" '"$(dirname "$0")/phase13-test-watcher" &
watcher_pid=$!
if [[ "${PHASE13_TEST_TREE_MODE:-wait}" == "orphan" ]]; then
  exit 0
fi
wait "$watcher_pid"
'
}

wait_for_file() {
	local path="$1"
	for _ in $(seq 1 100); do
		[[ -s "$path" ]] && return 0
		sleep 0.05
	done
	fail "timed out waiting for $path"
}

assert_pid_stopped() {
	local pid="$1"
	for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
		if ! kill -0 "$pid" 2>/dev/null; then
			return 0
		fi
		sleep 0.05
	done
	fail "descendant watcher $pid survived cleanup"
}

test_monitor_command_is_bounded_and_safe() {
	local bin_dir="${tmp_root}/fast-bin"
	local out_file="${tmp_root}/monitor.log"

	create_fast_espflash "$bin_dir"

	PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 3

	assert_contains "$out_file" "monitor_command: espflash monitor --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "$out_file" "serial_write: disabled"
	assert_contains "$out_file" "raw_flash_write: disabled"
	assert_contains "$out_file" "capture_status=completed"
	assert_contains "$out_file" "monitor_exit_status: 0"
	assert_not_contains "$out_file" "erase-flash"
	assert_not_contains "$out_file" "write-flash"
}

test_raw_out_separates_reader_stdout_from_wrapper_log() {
	local bin_dir="${tmp_root}/raw-out-bin"
	local out_file="${tmp_root}/raw-out.log"
	local raw_file="${tmp_root}/raw-out.payload"

	create_fast_espflash "$bin_dir"
	PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port /dev/test --out "$out_file" --raw-out "$raw_file" --seconds 3

	assert_contains "$raw_file" "espflash-args: monitor --chip esp32s3 --port /dev/test --non-interactive"
	assert_not_contains "$out_file" "espflash-args:"
	assert_contains "$out_file" "capture_status=completed"
	[[ "$(mode_of "$raw_file")" == "600" ]] || fail "raw output is not mode 600"
}

test_os_native_reader_captures_fifo_without_control_operations() {
	local bin_dir="${tmp_root}/os-native-bin"
	local out_file="${tmp_root}/os-native.log"
	local raw_file="${tmp_root}/os-native.payload"
	local serial_port="${tmp_root}/os-native.fifo"
	local writer_pid

	mkdir -p "$bin_dir"
	mkfifo "$serial_port"
	chmod 600 "$serial_port"
	write_executable "${bin_dir}/node-identity" 'printf "node-stable\n"'
	write_executable "${bin_dir}/usb-identity" 'printf "usb-stable\n"'
	# shellcheck disable=SC2016 # The generated fixture observes the real fresh-process reader.
	write_executable "${bin_dir}/lsof-owner" 'pid="$(pgrep -f "phase13-os-native-reader.pl ${SERIAL_TEST_PORT:?}" | head -1)"
if [[ -z "$pid" ]]; then
  exit 1
fi
printf "%s\n" "$pid"'

	(
		sleep 0.2
		printf 'runtime_heartbeat session=0123456789abcdef0123456789abcdef sequence=1 uptime_ms=1000 cadence_ms=1000 listener_armed=false redacted=true\n' >"$serial_port"
	) &
	writer_pid=$!

	SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 SERIAL_SESSION_TOOL_VERSION=test \
		SERIAL_SESSION_ACTIVE_OWNER_INTERVAL_SECONDS=0 \
		SERIAL_SESSION_NODE_IDENTITY_BIN="${bin_dir}/node-identity" \
		SERIAL_SESSION_USB_IDENTITY_BIN="${bin_dir}/usb-identity" \
		SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-owner" \
		SERIAL_TEST_PORT="$serial_port" \
		"$BASH" "$monitor_script" --port "$serial_port" --out "$out_file" --raw-out "$raw_file" --reader os-native --seconds 1 --no-reset
	wait "$writer_pid"

	assert_contains "$raw_file" "runtime_heartbeat session=0123456789abcdef0123456789abcdef"
	assert_contains "$out_file" "reader: os-native"
	assert_contains "$out_file" "monitor_command: phase13-os-native-reader.pl ${serial_port}"
	assert_not_contains "$out_file" "runtime_heartbeat"
	if grep -Eiq 'syswrite|ioctl|termios|stty' "${script_dir}/phase13-os-native-reader.pl"; then
		fail "OS-native reader contains a prohibited serial control or write API"
	fi
}

mode_of() {
	local path="$1"
	if stat -f '%Lp' "$path" >/dev/null 2>&1; then
		stat -f '%Lp' "$path"
		return
	fi
	stat -c '%a' "$path"
}

test_repeated_fast_monitor_completion_never_becomes_timeout() {
	local bin_dir="${tmp_root}/repeated-fast-bin"
	local iteration

	create_fast_espflash "$bin_dir"
	for iteration in 1 2 3 4 5 6 7 8 9 10; do
		local out_file="${tmp_root}/repeated-fast-${iteration}.log"

		PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 1

		assert_contains "$out_file" "capture_status=completed"
		assert_contains "$out_file" "monitor_exit_status: 0"
	done
}

test_monitor_timeout_status_is_recorded() {
	local bin_dir="${tmp_root}/slow-bin"
	local out_file="${tmp_root}/timeout.log"
	local serial_port="${tmp_root}/serial-device"
	local active_ready_file="${tmp_root}/monitor-active.ready"

	create_slow_espflash "$bin_dir"
	write_executable "${bin_dir}/node-identity" 'printf "node-stable\n"'
	write_executable "${bin_dir}/usb-identity" 'printf "usb-stable\n"'
	# shellcheck disable=SC2016 # The generated fixture expands these values at runtime.
	write_executable "${bin_dir}/lsof-owner" 'pid="$(pgrep -f "${SERIAL_TEST_MONITOR_PATTERN:?}" | head -1)"
[[ -n "$pid" ]] || exit 1
printf "%s\n" "$pid"'
	: >"$serial_port"
	chmod 600 "$serial_port"

	SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 SERIAL_SESSION_TOOL_VERSION=test \
		SERIAL_SESSION_ACTIVE_OWNER_INTERVAL_SECONDS=0 \
		SERIAL_SESSION_NODE_IDENTITY_BIN="${bin_dir}/node-identity" \
		SERIAL_SESSION_USB_IDENTITY_BIN="${bin_dir}/usb-identity" \
		SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-owner" \
		SERIAL_TEST_MONITOR_PATTERN="${bin_dir}/espflash" \
		PHASE13_MONITOR_ACTIVE_READY_FILE="$active_ready_file" \
		PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port "$serial_port" --out "$out_file" --seconds 1 --no-reset

	assert_contains "$out_file" "monitor_command: espflash monitor --chip esp32s3 --port ${serial_port} --non-interactive --before no-reset-no-sync --after no-reset --no-reset"
	assert_not_contains "$out_file" "monitor_command: espflash monitor --chip esp32s3 --port ${serial_port} --non-interactive --no-reset"
	assert_contains "$out_file" "capture_status=timed_out_after_capture"
	assert_contains "$out_file" "serial_trace_pre_readiness=ready"
	[[ -s "$active_ready_file" ]] || fail "passive monitor did not publish active-owner readiness"
	local active_ready_mode
	if active_ready_mode="$(stat -f '%Lp' "$active_ready_file" 2>/dev/null)"; then
		:
	else
		active_ready_mode="$(stat -c '%a' "$active_ready_file")"
	fi
	[[ "$active_ready_mode" == "600" ]] || fail "active-owner readiness is not mode 600"
	assert_contains "$out_file" "serial_trace_post_readiness=ready"
	assert_contains "$out_file" "serial_trace_active_owner_verified=true"
}

test_timeout_stops_descendant_watcher() {
	local bin_dir="${tmp_root}/timeout-tree-bin"
	local out_file="${tmp_root}/timeout-tree.log"
	local watcher_pid_file="${tmp_root}/timeout-tree.pid"
	local group_state_file="${tmp_root}/timeout-tree.group"

	create_descendant_espflash "$bin_dir"

	PHASE13_MONITOR_GROUP_STATE_FILE="$group_state_file" \
		PHASE13_TEST_WATCHER_PID_FILE="$watcher_pid_file" PATH="${bin_dir}:$PATH" \
		"$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 1

	wait_for_file "$watcher_pid_file"
	assert_pid_stopped "$(<"$watcher_pid_file")"
	[[ ! -s "$group_state_file" ]] || fail "timeout cleanup falsely retained live group state"
	assert_contains "$out_file" "capture_status=timed_out_after_capture"
}

test_term_stops_descendant_watcher() {
	local bin_dir="${tmp_root}/term-tree-bin"
	local out_file="${tmp_root}/term-tree.log"
	local watcher_pid_file="${tmp_root}/term-tree.pid"
	local wrapper_pid
	local wrapper_status

	create_descendant_espflash "$bin_dir"

	PHASE13_TEST_WATCHER_PID_FILE="$watcher_pid_file" PATH="${bin_dir}:$PATH" \
		"$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 30 \
		>"${tmp_root}/term-wrapper.stdout" 2>"${tmp_root}/term-wrapper.stderr" &
	wrapper_pid=$!
	wait_for_file "$watcher_pid_file"
	kill -TERM "$wrapper_pid"
	set +e
	wait "$wrapper_pid"
	wrapper_status=$?
	set -e

	[[ "$wrapper_status" -ne 0 ]] || fail "TERM cancellation unexpectedly succeeded"
	assert_pid_stopped "$(<"$watcher_pid_file")"
}

test_exit_stops_orphaned_descendant_watcher() {
	local bin_dir="${tmp_root}/exit-tree-bin"
	local out_file="${tmp_root}/exit-tree.log"
	local watcher_pid_file="${tmp_root}/exit-tree.pid"

	create_descendant_espflash "$bin_dir"

	PHASE13_TEST_TREE_MODE=orphan PHASE13_TEST_WATCHER_PID_FILE="$watcher_pid_file" \
		PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 30

	wait_for_file "$watcher_pid_file"
	assert_pid_stopped "$(<"$watcher_pid_file")"
	assert_contains "$out_file" "capture_status=completed"
}

if [[ ! -f "$monitor_script" ]]; then
	fail "monitor script missing: ${monitor_script}"
fi

test_monitor_command_is_bounded_and_safe
test_raw_out_separates_reader_stdout_from_wrapper_log
test_os_native_reader_captures_fifo_without_control_operations
test_repeated_fast_monitor_completion_never_becomes_timeout
test_monitor_timeout_status_is_recorded
test_timeout_stops_descendant_watcher
test_term_stops_descendant_watcher
test_exit_stops_orphaned_descendant_watcher

printf 'phase13_monitor_capture_test passed\n'
