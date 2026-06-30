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

test_monitor_command_is_bounded_and_safe() {
	local bin_dir="${tmp_root}/fast-bin"
	local out_file="${tmp_root}/monitor.log"

	create_fast_espflash "$bin_dir"

	PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 3

	assert_contains "$out_file" "monitor_command: espflash monitor --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "$out_file" "serial_write: disabled"
	assert_contains "$out_file" "raw_flash_write: disabled"
	assert_contains "$out_file" "capture_status=completed"
	assert_not_contains "$out_file" "erase-flash"
	assert_not_contains "$out_file" "write-flash"
}

test_monitor_timeout_status_is_recorded() {
	local bin_dir="${tmp_root}/slow-bin"
	local out_file="${tmp_root}/timeout.log"

	create_slow_espflash "$bin_dir"

	PATH="${bin_dir}:$PATH" "$BASH" "$monitor_script" --port /dev/test --out "$out_file" --seconds 1 --no-reset

	assert_contains "$out_file" "monitor_command: espflash monitor --chip esp32s3 --port /dev/test --non-interactive --no-reset"
	assert_contains "$out_file" "capture_status=timed_out_after_capture"
}

if [[ ! -f "$monitor_script" ]]; then
	fail "monitor script missing: ${monitor_script}"
fi

test_monitor_command_is_bounded_and_safe
test_monitor_timeout_status_is_recorded

printf 'phase13_monitor_capture_test passed\n'
