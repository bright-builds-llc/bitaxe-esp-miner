#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly diagnostic="${LATE_ATTACH_DIAGNOSTIC_SCRIPT:-$script_dir/diagnose-ultra205-late-attach.sh}"
readonly expected_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/late-attach-test.XXXXXX")"
readonly tmp_root

cleanup() {
	[[ "${KEEP_LATE_ATTACH_TESTS:-0}" != 1 ]] || return
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

mode_of() {
	if stat -f '%Lp' "$1" >/dev/null 2>&1; then
		stat -f '%Lp' "$1"
		return
	fi
	stat -c '%a' "$1"
}

write_executable() {
	local path="$1"
	local body="$2"
	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

heartbeat() {
	local sequence="$1"
	local uptime="$2"
	printf 'runtime_heartbeat session=0123456789abcdef0123456789abcdef sequence=%s uptime_ms=%s cadence_ms=10000 listener_armed=true redacted=true\n' "$sequence" "$uptime"
}

create_fixtures() {
	local test_root="$1"
	local bin_dir="$test_root/bin"
	mkdir -p "$bin_dir"
	: >"$test_root/command.log"
	: >"$test_root/detector.count"
	: >"$test_root/monitor.count"
	printf 'node-before\n' >"$test_root/node.identity"
	printf 'usb-stable\n' >"$test_root/usb.identity"
	: >"$test_root/port"
	chmod 600 "$test_root/port"

	# shellcheck disable=SC2016 # Runtime fixture variables belong to the generated process.
	write_executable "$bin_dir/detector" 'printf "detector\n" >>"${TEST_COMMAND_LOG:?}"
count="$(cat "${TEST_DETECTOR_COUNT:?}")"
count="${count:-0}"
printf "%s\n" "$((count + 1))" >"${TEST_DETECTOR_COUNT:?}"
printf "Chip type: esp32s3\n" >&2
printf "port=%s\n" "${TEST_PORT:?}"'
	# shellcheck disable=SC2016 # Runtime fixture variables belong to the generated process.
	write_executable "$bin_dir/monitor" 'printf "monitor %s\n" "$*" >>"${TEST_COMMAND_LOG:?}"
count="$(cat "${TEST_MONITOR_COUNT:?}")"
count="${count:-0}"
count=$((count + 1))
printf "%s\n" "$count" >"${TEST_MONITOR_COUNT:?}"
out=""
raw=""
reader=""
while (($#)); do
  case "$1" in
    --out) out="$2"; shift 2 ;;
    --raw-out) raw="$2"; shift 2 ;;
    --reader) reader="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n "$out" && -n "$raw" && -n "$reader" ]] || exit 2
printf "capture_status=timed_out_after_capture\nreader=%s\n" "$reader" >"$out"
if ((count <= 2)); then
  sequence="$count"
  uptime=$((120000 + count * 10000))
  printf "runtime_heartbeat session=0123456789abcdef0123456789abcdef sequence=%s uptime_ms=%s cadence_ms=10000 listener_armed=true redacted=true\n" "$sequence" "$uptime" >"$raw"
else
  index=$((count - 2))
  value="$(printf "%s" "${TEST_COLD_PATTERN:-111}" | cut -c "$index")"
  : >"$raw"
  if [[ "$value" == "1" ]]; then
    sequence=$((count + 1))
    uptime=$((120000 + count * 10000))
    printf "runtime_heartbeat session=0123456789abcdef0123456789abcdef sequence=%s uptime_ms=%s cadence_ms=10000 listener_armed=true redacted=true\n" "$sequence" "$uptime" >"$raw"
  elif [[ "$value" == "n" ]]; then
    printf "non-heartbeat output\n" >"$raw"
  fi
fi
chmod 600 "$out" "$raw"'
	# Extend the monitor fixture without adding serial control operations.
	# shellcheck disable=SC2016 # Runtime fixture variables belong to the generated process.
	printf '\nif [[ "${TEST_REMOVE_PORT_AT:-}" == "$count" ]]; then rm -f "${TEST_PORT:?}"; fi\n' >>"$bin_dir/monitor"
	# shellcheck disable=SC2016 # Runtime fixture variables belong to the generated process.
	write_executable "$bin_dir/node-identity" 'cat "${TEST_NODE_IDENTITY_FILE:?}"'
	# shellcheck disable=SC2016 # Runtime fixture variables belong to the generated process.
	write_executable "$bin_dir/usb-identity" 'cat "${TEST_USB_IDENTITY_FILE:?}"'
	write_executable "$bin_dir/lsof-none" 'exit 1'
	write_executable "$bin_dir/lsof-unavailable" 'exit 70'
	write_executable "$bin_dir/lsof-holder" 'printf "%s\n" "$$"'
}

common_env() {
	local test_root="$1"
	COMMON_ENV=(
		env
		LATE_ATTACH_TEST_MODE=1
		LATE_ATTACH_CONTROL_ROOT="$test_root/control"
		LATE_ATTACH_TRACE_ROOT="$test_root/traces"
		LATE_ATTACH_DETECTOR_BIN="$test_root/bin/detector"
		LATE_ATTACH_MONITOR_BIN="$test_root/bin/monitor"
		LATE_ATTACH_PREFLIGHT_SECONDS=1
		LATE_ATTACH_ABSENCE_INTERVAL_SECONDS=0.01
		LATE_ATTACH_ABSENCE_SAMPLES=2
		LATE_ATTACH_RESTORE_TIMEOUT_MS=500
		SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0
		SERIAL_SESSION_NODE_IDENTITY_BIN="$test_root/bin/node-identity"
		SERIAL_SESSION_USB_IDENTITY_BIN="$test_root/bin/usb-identity"
		SERIAL_SESSION_LSOF_BIN="$test_root/bin/lsof-none"
		TEST_COMMAND_LOG="$test_root/command.log"
		TEST_DETECTOR_COUNT="$test_root/detector.count"
		TEST_MONITOR_COUNT="$test_root/monitor.count"
		TEST_NODE_IDENTITY_FILE="$test_root/node.identity"
		TEST_USB_IDENTITY_FILE="$test_root/usb.identity"
		TEST_PORT="$test_root/port"
	)
}

begin_attempt() {
	local test_root="$1"
	common_env "$test_root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=1 >"$test_root/begin.out" 2>&1
	BEGIN_HANDLE="$(sed -n 's/^resume_handle=//p' "$test_root/begin.out" | head -1)"
	[[ "$BEGIN_HANDLE" =~ ^[0-9a-f]{64}$ ]] || fail "begin did not return an opaque handle"
	grep -Fq 'checkpoint_token=late-attach-armed-removal-v1' "$test_root/begin.out" || fail "removal checkpoint missing"
}

deliver_and_restore() {
	local test_root="$1"
	local pattern="$2"
	common_env "$test_root"
	rm -f "$test_root/port"
	TEST_COLD_PATTERN="$pattern" "${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver \
		resume-handle="$BEGIN_HANDLE" \
		checkpoint-token=late-attach-armed-removal-v1 \
		response-token=late-attach-both-power-paths-removed \
		>"$test_root/deliver.out" 2>&1 &
	DELIVER_PID=$!
	for _ in $(seq 1 200); do
		grep -Fq 'action_token=late-attach-reader-watcher-armed-v1' "$test_root/deliver.out" 2>/dev/null && break
		kill -0 "$DELIVER_PID" 2>/dev/null || break
		sleep 0.01
	done
	grep -Fq 'response_required=false' "$test_root/deliver.out" || fail "restore action was not emitted after watcher arming"
	printf 'node-after\n' >"$test_root/node.identity"
	: >"$test_root/port"
	chmod 600 "$test_root/port"
	wait "$DELIVER_PID"
}

test_successful_a_b_a_is_resumable_private_and_safe() {
	local test_root="$tmp_root/success"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	deliver_and_restore "$test_root" 111

	grep -Fq 'classification_category=all_readers_deliver' "$test_root/deliver.out" || {
		printf 'Deliver output:\n%s\n' "$(cat "$test_root/deliver.out")" >&2
		fail "all-reader result not classified"
	}
	[[ "$(cat "$test_root/detector.count")" == 1 ]] || fail "detector did not run exactly once"
	[[ "$(cat "$test_root/monitor.count")" == 5 ]] || fail "expected two preflights and three cold captures"
	grep -Fq -- '--reader espflash' "$test_root/command.log" || fail "espflash reader absent"
	grep -Fq -- '--reader os-native' "$test_root/command.log" || fail "OS-native reader absent"
	grep -Fq -- '--no-reset' "$test_root/command.log" || fail "passive flag absent"
	if grep -Eiq 'erase-flash|write-flash|factory-reset|credential|curl|wget|nmap|board-info' "$test_root/command.log"; then
		fail "forbidden post-detector operation observed"
	fi
	local run_dir
	run_dir="$(find "$test_root/traces" -mindepth 1 -maxdepth 1 -type d | head -1)"
	[[ "$(mode_of "$test_root/control")" == 700 && "$(mode_of "$run_dir")" == 700 ]] || fail "private directories are not mode 0700"
	while IFS= read -r -d '' path; do
		[[ "$(mode_of "$path")" == 600 ]] || fail "private file is not mode 0600: $path"
	done < <(find "$run_dir" -type f -print0)
	jq -e '.classification_category == "all_readers_deliver" and .operations.detector_count == 1 and .operations.post_detector == false and .operations.raw_serial_write == false and .cleanup_complete == true' "$run_dir/summary.json" >/dev/null || fail "category-only summary malformed"
	if grep -Eq '/dev/|0123456789abcdef0123456789abcdef|selected_port|owner_pid' "$run_dir/summary.json"; then
		fail "summary exposed a private identity"
	fi
	[[ -z "$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'ultra205-late-*.sock' -print -quit)" ]] || fail "lifecycle socket survived cleanup"

	common_env "$test_root"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/stale.out" 2>&1
	local stale_status=$?
	set -e
	((stale_status != 0)) || fail "stale handle was accepted"
	grep -Fq 'late_attach_error=resume_handle_stale' "$test_root/stale.out" || fail "stale handle was not classified"
}

test_manual_token_cannot_spoof_usb_appearance() {
	local test_root="$tmp_root/appearance-timeout"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	rm -f "$test_root/port"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/deliver.out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail "missing USB appearance unexpectedly passed"
	grep -Fq 'action_token=late-attach-reader-watcher-armed-v1' "$test_root/deliver.out" || fail "watcher action not emitted"
	grep -Fq 'failure_category=appearance_timeout' "$test_root/deliver.out" || fail "appearance timeout not classified"
	[[ "$(cat "$test_root/monitor.count")" == 2 ]] || fail "cold readers ran without USB appearance"
}

test_unchanged_enumeration_epoch_fails_closed() {
	local test_root="$tmp_root/epoch"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	rm -f "$test_root/port"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/deliver.out" 2>&1 &
	local pid=$!
	for _ in $(seq 1 200); do
		grep -Fq 'response_required=false' "$test_root/deliver.out" 2>/dev/null && break
		sleep 0.01
	done
	: >"$test_root/port"
	chmod 600 "$test_root/port"
	set +e
	wait "$pid"
	local status=$?
	set -e
	((status != 0)) || fail "unchanged enumeration epoch passed"
	grep -Fq 'failure_category=enumeration_epoch_unchanged' "$test_root/deliver.out" || fail "enumeration epoch failure not classified"
}

test_unavailable_holder_probe_stops_before_preflight_readers() {
	local test_root="$tmp_root/probe"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	common_env "$test_root"
	COMMON_ENV+=(SERIAL_SESSION_LSOF_BIN="$test_root/bin/lsof-unavailable")
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=1 >"$test_root/begin.out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail "unavailable holder probe passed"
	grep -Fq 'failure_category=preflight_ownership_probe_unavailable' "$test_root/begin.out" || fail "holder probe failure not classified"
	[[ ! -s "$test_root/monitor.count" ]] || fail "preflight readers ran after readiness failure"
}

test_wrong_token_and_expired_checkpoint_fail_before_worker() {
	local test_root="$tmp_root/token-expiry"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=wrong response-token=wrong >"$test_root/wrong.out" 2>&1
	local wrong_status=$?
	set -e
	((wrong_status != 0)) || fail "wrong checkpoint token passed"
	grep -Fq 'late_attach_error=checkpoint_token_mismatch' "$test_root/wrong.out" || fail "wrong token not classified"
	[[ -z "$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'ultra205-late-*.sock' -print -quit)" ]] || fail "wrong token created a socket"

	local slot
	local attempt_dir
	slot="$(find "$test_root/control/resume-index" -type f -name '*.json' | head -1)"
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	jq '.checkpoint_deadline_ms=0' "$attempt_dir/state.json" >"$attempt_dir/state.next"
	chmod 600 "$attempt_dir/state.next"
	mv "$attempt_dir/state.next" "$attempt_dir/state.json"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/expired.out" 2>&1
	local expired_status=$?
	set -e
	((expired_status != 0)) || fail "expired checkpoint passed"
	grep -Fq 'late_attach_error=checkpoint_expired' "$test_root/expired.out" || fail "expiry not classified"
}

test_usb_identity_change_and_holder_conflict_fail_closed() {
	local test_root="$tmp_root/usb-change"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	rm -f "$test_root/port"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/deliver.out" 2>&1 &
	local pid=$!
	for _ in $(seq 1 200); do
		grep -Fq 'response_required=false' "$test_root/deliver.out" 2>/dev/null && break
		sleep 0.01
	done
	printf 'node-after\n' >"$test_root/node.identity"
	printf 'usb-changed\n' >"$test_root/usb.identity"
	: >"$test_root/port"
	set +e
	wait "$pid"
	local status=$?
	set -e
	((status != 0)) || fail "USB identity change passed"
	grep -Fq 'failure_category=usb_identity_changed' "$test_root/deliver.out" || fail "USB identity change not classified"

	test_root="$tmp_root/holder"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	COMMON_ENV+=(SERIAL_SESSION_LSOF_BIN="$test_root/bin/lsof-holder")
	rm -f "$test_root/port"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/deliver.out" 2>&1 &
	pid=$!
	for _ in $(seq 1 200); do
		grep -Fq 'response_required=false' "$test_root/deliver.out" 2>/dev/null && break
		sleep 0.01
	done
	printf 'node-after\n' >"$test_root/node.identity"
	: >"$test_root/port"
	set +e
	wait "$pid"
	status=$?
	set -e
	((status != 0)) || fail "holder conflict passed"
	grep -Fq 'failure_category=holders_present' "$test_root/deliver.out" || fail "holder conflict not classified"
}

test_node_loss_and_worker_crash_leave_tombstones_without_processes() {
	local test_root="$tmp_root/node-loss"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	COMMON_ENV+=(TEST_REMOVE_PORT_AT=3)
	rm -f "$test_root/port"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/deliver.out" 2>&1 &
	local pid=$!
	for _ in $(seq 1 200); do
		grep -Fq 'response_required=false' "$test_root/deliver.out" 2>/dev/null && break
		sleep 0.01
	done
	printf 'node-after\n' >"$test_root/node.identity"
	: >"$test_root/port"
	set +e
	wait "$pid"
	local status=$?
	set -e
	((status != 0)) || fail "node loss passed"
	grep -Fq 'failure_category=cleanup_missing_node' "$test_root/deliver.out" || fail "node loss not classified"

	test_root="$tmp_root/worker-crash"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	begin_attempt "$test_root"
	common_env "$test_root"
	rm -f "$test_root/port"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$BEGIN_HANDLE" checkpoint-token=late-attach-armed-removal-v1 response-token=late-attach-both-power-paths-removed >"$test_root/deliver.out" 2>&1 &
	pid=$!
	for _ in $(seq 1 200); do
		grep -Fq 'response_required=false' "$test_root/deliver.out" 2>/dev/null && break
		sleep 0.01
	done
	local state
	state="$(find "$test_root/control/attempts" -name state.json | head -1)"
	local worker_pid
	local attempt_id
	worker_pid="$(jq -er '.owner_pid' "$state")"
	attempt_id="$(jq -er '.attempt_id' "$state")"
	kill -KILL "$worker_pid"
	set +e
	wait "$pid"
	status=$?
	set -e
	((status != 0)) || fail "worker crash passed"
	grep -Fq 'failure_category=worker_failed' "$test_root/deliver.out" || fail "worker crash not classified"
	[[ ! -e "${TMPDIR:-/tmp}/ultra205-late-${attempt_id}.sock" ]] || fail "crashed worker socket survived"
}

test_invalid_arguments_and_reader_source_are_safe() {
	local test_root="$tmp_root/invalid"
	mkdir -p "$test_root"
	create_fixtures "$test_root"
	common_env "$test_root"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head=0000000000000000000000000000000000000000 capture-seconds=1 >"$test_root/head.out" 2>&1
	local head_status=$?
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=0 >"$test_root/bounds.out" 2>&1
	local bounds_status=$?
	set -e
	((head_status != 0 && bounds_status != 0)) || fail "invalid begin arguments passed"
	[[ ! -s "$test_root/command.log" ]] || fail "invalid arguments ran a device command"
	grep -Fq 'O_RDONLY | O_NOCTTY | O_NONBLOCK' "$script_dir/phase13-os-native-reader.pl" || fail "OS reader flags changed"
	for forbidden in syswrite ioctl termios stty; do
		if grep -Eiq "${forbidden}" "$script_dir/phase13-os-native-reader.pl"; then
			fail "OS reader uses prohibited API: $forbidden"
		fi
	done
}

[[ -f "$diagnostic" ]] || fail "diagnostic missing"
test_successful_a_b_a_is_resumable_private_and_safe
test_manual_token_cannot_spoof_usb_appearance
test_unchanged_enumeration_epoch_fails_closed
test_unavailable_holder_probe_stops_before_preflight_readers
test_wrong_token_and_expired_checkpoint_fail_before_worker
test_usb_identity_change_and_holder_conflict_fail_closed
test_node_loss_and_worker_crash_leave_tombstones_without_processes
test_invalid_arguments_and_reader_source_are_safe

printf 'diagnose_ultra205_late_attach_test passed\n'
