#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly detector_script="${DETECT_ULTRA205_SCRIPT:-${script_dir}/detect-ultra205.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/detect-ultra205-test.XXXXXX")"
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
	local haystack="$1"
	local needle="$2"

	case "$haystack" in
	*"$needle"*) ;;
	*)
		printf 'Expected output to contain: %s\n' "$needle" >&2
		printf 'Actual output:\n%s\n' "$haystack" >&2
		exit 1
		;;
	esac
}

capture_command() {
	local output_file="$1"
	shift

	set +e
	"$@" >"$output_file" 2>&1
	local status=$?
	set -e

	return "$status"
}

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_espflash_stub() {
	local path="$1"

	write_executable "$path" 'scenario="${DETECT_ULTRA205_TEST_SCENARIO:?}"
printf "%s\n" "$*" >>"${DETECT_ULTRA205_COMMAND_LOG:?}"
if [[ "${1:-}" == "list-ports" && "${2:-}" == "--name-only" ]]; then
  case "$scenario" in
    list_failure)
      printf "list ports failed\n" >&2
      exit 1
      ;;
    zero)
      exit 0
      ;;
    multiple)
      printf "/dev/cu.usbmodem101\n/dev/cu.usbmodem202\n"
      exit 0
      ;;
    success | board_info_failure | open_failure)
      printf "/dev/cu.usbmodem101\n"
      exit 0
      ;;
    *)
      printf "unknown scenario: %s\n" "$scenario" >&2
      exit 2
      ;;
  esac
fi

if [[ "${1:-}" == "board-info" ]]; then
  case "$scenario" in
    success)
      printf "Chip type: esp32s3\nFlash size: 16MB\n"
      exit 0
      ;;
    board_info_failure)
      printf "serial sync failed\n" >&2
      exit 1
      ;;
    open_failure)
      printf "failed to open serial port: permission denied\n" >&2
      exit 1
      ;;
    *)
      printf "board-info should not run for scenario: %s\n" "$scenario" >&2
      exit 3
      ;;
  esac
fi

printf "unexpected espflash args: %s\n" "$*" >&2
exit 2
'
}

run_detector() {
	local scenario="$1"
	local output_file="$2"
	local espflash_stub="${tmp_root}/espflash-${scenario}"

	create_espflash_stub "$espflash_stub"
	capture_command "$output_file" env \
		DETECT_ULTRA205_TEST_SCENARIO="$scenario" \
		DETECT_ULTRA205_COMMAND_LOG="${tmp_root}/commands-${scenario}.log" \
		ESPFLASH_BIN="$espflash_stub" \
		SERIAL_SESSION_TOOL_VERSION=test \
		SERIAL_SESSION_TRACE_ROOT="${tmp_root}/traces-${scenario}" \
		"$BASH" "$detector_script"
}

test_explicit_port_skips_listing_and_keeps_reset_contract() {
	local output_file="${tmp_root}/explicit.out"
	local espflash_stub="${tmp_root}/espflash-explicit"
	local command_log="${tmp_root}/explicit.commands"
	create_espflash_stub "$espflash_stub"

	if ! capture_command "$output_file" env \
		DETECT_ULTRA205_TEST_SCENARIO=success \
		DETECT_ULTRA205_COMMAND_LOG="$command_log" \
		ESPFLASH_BIN="$espflash_stub" \
		SERIAL_SESSION_TOOL_VERSION=test \
		SERIAL_SESSION_TRACE_ROOT="${tmp_root}/traces-explicit" \
		"$BASH" "$detector_script" --port /dev/cu.explicit205; then
		fail "explicit detector mode failed"
	fi

	[[ "$(wc -l <"$command_log" | tr -d ' ')" == 1 ]] || fail "explicit mode invoked more than board-info"
	! grep -Fq 'list-ports' "$command_log" || fail "explicit mode listed ports"
	assert_contains "$(cat "$command_log")" "board-info --chip esp32s3 --port /dev/cu.explicit205 --non-interactive --before usb-reset --after hard-reset"
	assert_contains "$(cat "$output_file")" "port=/dev/cu.explicit205"
}

test_list_ports_failure_is_classified() {
	local output_file="${tmp_root}/list-failure.out"

	if run_detector list_failure "$output_file"; then
		fail "detector unexpectedly passed when list-ports failed"
	fi

	assert_contains "$(cat "$output_file")" "failure_category=list_ports_failed"
}

test_zero_ports_fails() {
	local output_file="${tmp_root}/zero.out"

	if run_detector zero "$output_file"; then
		fail "detector unexpectedly passed with zero ports"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "no likely Ultra 205 ESP USB serial port detected"
	assert_contains "$output" "just detect-ultra205"
}

test_multiple_ports_fail_with_candidates() {
	local output_file="${tmp_root}/multiple.out"

	if run_detector multiple "$output_file"; then
		fail "detector unexpectedly passed with multiple ports"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "multiple likely ESP USB serial ports detected"
	assert_contains "$output" "/dev/cu.usbmodem101"
	assert_contains "$output" "/dev/cu.usbmodem202"
}

test_single_port_success_prints_port() {
	local output_file="${tmp_root}/success.out"

	if ! run_detector success "$output_file"; then
		printf 'Detector output:\n%s\n' "$(cat "$output_file")" >&2
		fail "detector failed with one port and successful board-info"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "board-info"
	assert_contains "$output" "--before usb-reset --after hard-reset"
	assert_contains "$output" "Chip type: esp32s3"
	assert_contains "$output" "port=/dev/cu.usbmodem101"
}

test_board_info_failure_blocks_detection() {
	local output_file="${tmp_root}/board-info-failure.out"

	if run_detector board_info_failure "$output_file"; then
		fail "detector unexpectedly passed when board-info failed"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "board-info failed"
	assert_contains "$output" "serial sync failed"
	assert_contains "$output" "failure_category=connection_failure"
}

test_open_failure_is_classified() {
	local output_file="${tmp_root}/open-failure.out"

	if run_detector open_failure "$output_file"; then
		fail "detector unexpectedly passed when the serial port could not open"
	fi

	assert_contains "$(cat "$output_file")" "failure_category=open_failure"
}

if [[ ! -f "$detector_script" ]]; then
	fail "detector script missing: ${detector_script}"
fi

test_list_ports_failure_is_classified
test_zero_ports_fails
test_multiple_ports_fail_with_candidates
test_single_port_success_prints_port
test_board_info_failure_blocks_detection
test_open_failure_is_classified
test_explicit_port_skips_listing_and_keeps_reset_contract

printf 'detect ultra205 tests passed\n'
