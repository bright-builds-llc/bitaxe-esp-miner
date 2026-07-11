#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly diagnostic_script="${ULTRA205_SESSION_DIAGNOSTIC_SCRIPT:-${script_dir}/diagnose-ultra205-session.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/diagnose-ultra205-session-test.XXXXXX")"
readonly tmp_root
command_log="${tmp_root}/commands.log"
readonly command_log

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
	grep -Fq -- "$needle" "$path" || {
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	}
}

mode_of() {
	local path="$1"
	if [[ "$(uname -s)" == "Darwin" ]]; then
		stat -f '%Lp' "$path"
	else
		stat -c '%a' "$path"
	fi
}

write_executable() {
	local path="$1"
	local body="$2"
	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_stubs() {
	local bin_dir="$1"
	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/detector" 'printf "detector\n" >>"${SERIAL_TEST_COMMAND_LOG:?}"
count=0
[[ -f "${SERIAL_TEST_DETECTOR_COUNT_FILE:?}" ]] && count="$(cat "$SERIAL_TEST_DETECTOR_COUNT_FILE")"
count=$((count + 1))
printf "%s\n" "$count" >"$SERIAL_TEST_DETECTOR_COUNT_FILE"
if [[ "${SERIAL_TEST_FINAL_DETECTOR_FAIL:-0}" == "1" && "$count" -gt 1 ]]; then
  printf "failure_category=connection_failure\n" >&2
  exit 1
fi
printf "Chip type: esp32s3\n" >&2
printf "port=/dev/cu.usbmodem-test\n"'
	write_executable "${bin_dir}/monitor" 'printf "monitor %s\n" "$*" >>"${SERIAL_TEST_COMMAND_LOG:?}"
count=0
[[ -f "${SERIAL_TEST_MONITOR_COUNT_FILE:?}" ]] && count="$(cat "$SERIAL_TEST_MONITOR_COUNT_FILE")"
count=$((count + 1))
printf "%s\n" "$count" >"$SERIAL_TEST_MONITOR_COUNT_FILE"
out=""
while (($# > 0)); do
  case "$1" in
    --out) out="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n "$out" ]] || exit 2
if [[ "${SERIAL_TEST_MONITOR_FAIL_CYCLE:-0}" == "$count" ]]; then
  printf "serial_session_failure_category=connection_failure\n" >"$out"
  exit 1
fi
{
  printf "capture_status=timed_out_after_capture\n"
  printf "serial_trace_status=complete\n"
  printf "serial_trace_pre_readiness=ready\n"
  printf "serial_trace_post_readiness=ready\n"
  printf "serial_trace_active_owner_verified=true\n"
  printf "serial_trace_digest=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"
} >"$out"'
}

run_diagnostic() {
	local name="$1"
	shift
	local bin_dir="${tmp_root}/${name}-bin"
	local output_file="${tmp_root}/${name}.out"
	local run_root="${tmp_root}/${name}-runs"
	local detector_count_file="${tmp_root}/${name}-detector-count"
	local monitor_count_file="${tmp_root}/${name}-monitor-count"
	create_stubs "$bin_dir"

	set +e
	SERIAL_TEST_COMMAND_LOG="$command_log" \
		SERIAL_TEST_DETECTOR_COUNT_FILE="$detector_count_file" \
		SERIAL_TEST_MONITOR_COUNT_FILE="$monitor_count_file" \
		DETECT_ULTRA205_BIN="${bin_dir}/detector" \
		PHASE13_MONITOR_CAPTURE_SCRIPT="${bin_dir}/monitor" \
		ULTRA205_SESSION_DIAGNOSTIC_ROOT="$run_root" \
		"$BASH" "$diagnostic_script" "$@" >"$output_file" 2>&1
	RUN_STATUS=$?
	set -e
	RUN_OUTPUT_FILE="$output_file"
	RUN_ROOT="$run_root"
	RUN_DETECTOR_COUNT_FILE="$detector_count_file"
	RUN_MONITOR_COUNT_FILE="$monitor_count_file"
}

test_default_five_cycles_pass_and_are_safe() {
	: >"$command_log"
	run_diagnostic success
	[[ "$RUN_STATUS" -eq 0 ]] || fail "default diagnostic failed"
	[[ "$(cat "$RUN_DETECTOR_COUNT_FILE")" -eq 2 ]] || fail "diagnostic did not run baseline and final detectors"
	[[ "$(cat "$RUN_MONITOR_COUNT_FILE")" -eq 5 ]] || fail "default diagnostic did not run five cycles"
	assert_contains "$RUN_OUTPUT_FILE" 'status=passed'
	assert_contains "$RUN_OUTPUT_FILE" 'cycles_completed=5'
	assert_contains "$command_log" '--no-reset'
	if grep -Eiq 'flash|erase|factory|credential|curl|wget|nmap|write' "$command_log"; then
		fail "diagnostic invoked a forbidden operation"
	fi
	local run_dir
	run_dir="$(find "$RUN_ROOT" -mindepth 1 -maxdepth 1 -type d | head -1)"
	[[ "$(mode_of "$RUN_ROOT")" == "700" ]] || fail "diagnostic root is not mode 0700"
	[[ "$(mode_of "$run_dir")" == "700" ]] || fail "diagnostic run directory is not mode 0700"
	[[ "$(mode_of "${run_dir}/summary.json")" == "600" ]] || fail "diagnostic summary is not mode 0600"
	jq -e '.status == "passed" and .cycles_completed == 5 and .trace_complete_count == 5 and (.trace_set_digest_sha256 | test("^[0-9a-f]{64}$")) and .physical_intervention_requested == false and (.operations | all(. == false))' "${run_dir}/summary.json" >/dev/null ||
		fail "category-only summary contract failed"
	if grep -Fq '/dev/' "${run_dir}/summary.json"; then
		fail "category-only summary exposed a device path"
	fi
}

test_first_failure_stops_remaining_cycles() {
	: >"$command_log"
	SERIAL_TEST_MONITOR_FAIL_CYCLE=2 run_diagnostic stop-first --cycles 5 --capture-seconds 1
	[[ "$RUN_STATUS" -ne 0 ]] || fail "failing diagnostic unexpectedly passed"
	[[ "$(cat "$RUN_MONITOR_COUNT_FILE")" -eq 2 ]] || fail "diagnostic did not stop on first failure"
	[[ "$(cat "$RUN_DETECTOR_COUNT_FILE")" -eq 1 ]] || fail "final detector ran after a cycle failure"
	assert_contains "$RUN_OUTPUT_FILE" 'failure_category=cycle_2_connection_failure'
}

test_bounds_fail_before_commands() {
	: >"$command_log"
	run_diagnostic invalid-cycles --cycles 0
	[[ "$RUN_STATUS" -eq 2 ]] || fail "zero cycles did not return usage failure"
	[[ ! -s "$command_log" ]] || fail "invalid cycles ran a device command"

	run_diagnostic invalid-seconds --capture-seconds 301
	[[ "$RUN_STATUS" -eq 2 ]] || fail "oversized capture did not return usage failure"
	[[ ! -s "$command_log" ]] || fail "invalid capture duration ran a device command"
}

[[ -f "$diagnostic_script" ]] || fail "diagnostic script missing: ${diagnostic_script}"

test_default_five_cycles_pass_and_are_safe
test_first_failure_stops_remaining_cycles
test_bounds_fail_before_commands

printf 'diagnose_ultra205_session_test passed\n'
