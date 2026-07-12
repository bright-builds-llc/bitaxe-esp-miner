#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly diagnostic="${UART_CAPTURE_DIAGNOSTIC_SCRIPT:-$script_dir/diagnose-ultra205-uart-capture.sh}"
readonly expected_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/uart-capture-test.XXXXXX")"
readonly tmp_root
cleanup() { [[ "${KEEP_UART_CAPTURE_TESTS:-0}" == 1 ]] || rm -rf "$tmp_root"; }
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}
mode_of() { if stat -f '%Lp' "$1" >/dev/null 2>&1; then stat -f '%Lp' "$1"; else stat -c '%a' "$1"; fi; }
write_executable() {
	printf '#!%s\n%s\n' "$BASH" "$2" >"$1"
	chmod +x "$1"
}

create_fixtures() {
	local root="$1"
	local bin="$root/bin"
	mkdir -p "$bin"
	: >"$root/native-port"
	: >"$root/uart-port"
	: >"$root/monitor.count"
	: >"$root/commands.log"
	printf 'native-physical\n' >"$root/native.physical"
	printf 'native-enumeration-before\n' >"$root/native.enumeration"
	printf 'uart-physical\n' >"$root/uart.physical"
	printf 'uart-enumeration\n' >"$root/uart.enumeration"
	chmod 600 "$root/native-port" "$root/uart-port"
	# shellcheck disable=SC2016
	write_executable "$bin/detector" 'printf "%s\n" "$*" >>"${TEST_COMMAND_LOG:?}"
[[ "$*" == "--port ${TEST_NATIVE_PORT:?}" ]] || exit 91
printf "port=%s\n" "${TEST_NATIVE_PORT:?}"'
	# shellcheck disable=SC2016
	write_executable "$bin/physical-identity" 'if [[ "$1" == "${TEST_NATIVE_PORT:?}" ]]; then cat "${TEST_NATIVE_PHYSICAL:?}"; else cat "${TEST_UART_PHYSICAL:?}"; fi'
	# shellcheck disable=SC2016
	write_executable "$bin/enumeration-identity" 'if [[ "$1" == "${TEST_NATIVE_PORT:?}" ]]; then cat "${TEST_NATIVE_ENUMERATION:?}"; else cat "${TEST_UART_ENUMERATION:?}"; fi'
	# shellcheck disable=SC2016
	write_executable "$bin/node-identity" '"${TEST_ENUMERATION_BIN:?}" "$1"'
	# shellcheck disable=SC2016
	write_executable "$bin/lsof" 'port="${@: -1}"
[[ "$port" == "${TEST_UART_PORT:?}" ]] || exit 1
pid="$(cat "${TEST_MONITOR_PID_FILE:?}" 2>/dev/null)"
[[ -n "$pid" ]] || exit 1
kill -0 "$pid" 2>/dev/null || exit 1
printf "%s\n" "$pid"'
	# shellcheck disable=SC2016
	write_executable "$bin/monitor" 'count="$(cat "${TEST_MONITOR_COUNT:?}")"; count="${count:-0}"; count=$((count + 1)); printf "%s\n" "$count" >"${TEST_MONITOR_COUNT:?}"
printf "monitor %s\n" "$*" >>"${TEST_COMMAND_LOG:?}"
out=""; raw=""; reader=""
while (($#)); do case "$1" in --out) out="$2"; shift 2;; --raw-out) raw="$2"; shift 2;; --reader) reader="$2"; shift 2;; *) shift;; esac; done
: >"$out"; : >"$raw"; chmod 600 "$out" "$raw"
session=0123456789abcdef0123456789abcdef
if ((count == 1)); then start=1; elif ((count == 2)); then start=4; else start=7; fi
for offset in 0 1 2; do sequence=$((start + offset)); uptime=$((120000 + sequence * 10000)); printf "runtime_heartbeat session=%s sequence=%s uptime_ms=%s cadence_ms=10000 listener_armed=true redacted=true\n" "$session" "$sequence" "$uptime" >>"$raw"; done
if ((count < 3)); then printf "capture_status=timed_out_after_capture\nreader=%s\n" "$reader" >"$out"; exit 0; fi
printf "%s\n" "$$" >"${TEST_MONITOR_PID_FILE:?}"
printf "capture_status=running\nreader=%s\n" "$reader" >"$out"
printf "ready\n" >"${PHASE13_MONITOR_ACTIVE_READY_FILE:?}"; chmod 600 "${PHASE13_MONITOR_ACTIVE_READY_FILE:?}"
trap "exit 0" INT TERM
while true; do sleep 0.1; done'
}

set_env() {
	local root="$1"
	COMMON_ENV=(env LATE_ATTACH_TEST_MODE=1 UART_CAPTURE_CONTROL_ROOT="$root/control" UART_CAPTURE_TRACE_ROOT="$root/traces" LATE_ATTACH_DETECTOR_BIN="$root/bin/detector" LATE_ATTACH_MONITOR_BIN="$root/bin/monitor" UART_CAPTURE_MONITOR_BIN="$root/bin/monitor" LATE_ATTACH_PREFLIGHT_SECONDS=1 LATE_ATTACH_ABSENCE_INTERVAL_SECONDS=0.01 LATE_ATTACH_ABSENCE_SAMPLES=2 LATE_ATTACH_RESTORE_TIMEOUT_MS=5000 LATE_ATTACH_SOAK_INTERVAL_SECONDS=0.01 LATE_ATTACH_SOAK_SAMPLES=2 LATE_ATTACH_RESULT_WAIT_SAMPLES=5000 UART_CAPTURE_QUIET_SECONDS=0.05 UART_CAPTURE_CONTINUOUS_SECONDS=30 SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 SERIAL_SESSION_ACTIVE_OWNER_INTERVAL_SECONDS=0 SERIAL_SESSION_NODE_IDENTITY_BIN="$root/bin/node-identity" SERIAL_SESSION_USB_PHYSICAL_IDENTITY_BIN="$root/bin/physical-identity" SERIAL_SESSION_USB_ENUMERATION_IDENTITY_BIN="$root/bin/enumeration-identity" SERIAL_SESSION_LSOF_BIN="$root/bin/lsof" TEST_COMMAND_LOG="$root/commands.log" TEST_MONITOR_COUNT="$root/monitor.count" TEST_MONITOR_PID_FILE="$root/monitor.pid" TEST_NATIVE_PORT="$root/native-port" TEST_UART_PORT="$root/uart-port" TEST_NATIVE_PHYSICAL="$root/native.physical" TEST_NATIVE_ENUMERATION="$root/native.enumeration" TEST_UART_PHYSICAL="$root/uart.physical" TEST_UART_ENUMERATION="$root/uart.enumeration" TEST_ENUMERATION_BIN="$root/bin/enumeration-identity")
}

attempt_dir() { find "$1/control/attempts" -mindepth 1 -maxdepth 1 -type d | head -1; }
wait_for_state() {
	local directory
	directory="$(attempt_dir "$1")"
	for _ in $(seq 1 500); do
		[[ "$(jq -r '.state' "$directory/state.json")" == "$2" ]] && return
		sleep 0.01
	done
	fail "state $2 not reached"
}

append_cold_log() {
	local path="$1" session=fedcba9876543210fedcba9876543210 stage
	{
		printf 'bitaxe-rust boot: board=Ultra 205 asic=BM1366\n'
		printf 'h4_continuous_result=listener_armed\n'
		printf 'plan13_boot_evidence session=%s state=booted redacted=true\n' "$session"
		printf 'plan13_boot_evidence session=%s state=listener_armed redacted=true\n' "$session"
		printf 'runtime_heartbeat session=%s sequence=1 uptime_ms=1000 cadence_ms=1000 listener_armed=true redacted=true\n' "$session"
		printf 'runtime_heartbeat session=%s sequence=2 uptime_ms=120000 cadence_ms=1000 listener_armed=true redacted=true\n' "$session"
		printf 'runtime_heartbeat session=%s sequence=3 uptime_ms=130000 cadence_ms=10000 listener_armed=true redacted=true\n' "$session"
		for stage in post_enumerate post_mining_ready post_max_baud post_mask_reload post_first_work; do
			printf 'accepted_state_snapshot stage=%s observation=available redacted=true\n' "$stage"
		done
	} >>"$path"
}

test_success_keeps_uart_owned_across_native_removal() {
	local root="$tmp_root/success" directory handle deliver_pid run_dir early_status
	mkdir -p "$root"
	create_fixtures "$root"
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" port="$root/native-port" uart-port="$root/uart-port" capture-seconds=3 >"$root/begin.out" 2>&1
	handle="$(sed -n 's/^resume_handle=//p' "$root/begin.out" | head -1)"
	[[ "$handle" =~ ^[0-9a-f]{64}$ ]] || fail "opaque handle missing"
	grep -Fq 'action_token=uart-capture-removal-watcher-armed-v1' "$root/begin.out" || fail "removal action missing"
	[[ "$(cat "$root/monitor.count")" == 3 ]] || fail "continuous reader was not third capture"
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$handle" >"$root/status.out"
	grep -Fq 'action_token=uart-capture-removal-watcher-armed-v1' "$root/status.out" || fail "status did not re-emit action"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$handle" checkpoint-token=uart-capture-removal-watcher-armed-v1 response-token=uart-capture-both-board-power-paths-removed-v1 >"$root/early.out" 2>&1
	early_status=$?
	set -e
	((early_status != 0)) || fail "early token passed"
	grep -Fq 'late_attach_error=removal_not_observed' "$root/early.out" || fail "early token category wrong"
	directory="$(attempt_dir "$root")"
	rm "$root/native-port"
	wait_for_state "$root" removal_observed
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$handle" checkpoint-token=uart-capture-removal-watcher-armed-v1 response-token=uart-capture-both-board-power-paths-removed-v1 >"$root/deliver.out" 2>&1 &
	deliver_pid=$!
	for _ in $(seq 1 500); do
		jq -e '.lines | index("action_token=uart-capture-cold-reader-armed-v1") != null' "$directory/action.json" >/dev/null 2>&1 && break
		sleep 0.01
	done
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$handle" >"$root/restore-status.out"
	grep -Fq 'response_required=false' "$root/restore-status.out" || fail "restore action not response-free"
	append_cold_log "$directory/continuous-uart.raw.log"
	printf 'native-enumeration-after\n' >"$root/native.enumeration"
	: >"$root/native-port"
	chmod 600 "$root/native-port"
	wait "$deliver_pid" || {
		cat "$root/deliver.out" >&2
		cat "$directory/worker.stderr" >&2 || true
		fail "UART qualification failed"
	}
	grep -Fq 'classification_category=uart_cold_delivers' "$root/deliver.out" || fail "passing category absent"
	run_dir="$(find "$root/traces" -mindepth 1 -maxdepth 1 -type d | head -1)"
	jq -e '.schema_version == "ultra205-transport-qualification-v3" and .classification_category == "uart_cold_delivers" and .cold_uart_heartbeat_count == 3 and .native_new_enumeration_epoch and .uart_physical_identity_stable and .uart_enumeration_identity_stable and .quiet_boundary_complete and .original_boot_present and .original_listener_present and .boot_evidence_complete and .accepted_state_stages_complete and .heartbeat_monotonic and .listener_ready and .soak_complete and .cleanup_complete' "$run_dir/qualification.json" >/dev/null || fail "v3 qualification malformed"
	[[ "$(mode_of "$run_dir")" == 700 && "$(mode_of "$run_dir/qualification.json")" == 600 ]] || fail "private permissions wrong"
	if grep -Eq 'native-port|uart-port|0123456789abcdef|fedcba9876543210' "$run_dir/qualification.json"; then fail "qualification exposed private identity"; fi
}

test_static_prohibitions() {
	node "$script_dir/ultra205-uart-capture-classifier-test.mjs"
	if rg -n 'os\.write|syswrite|TIOCM|TIOCMB|erase-flash|write-flash|factory-reset|nmap|curl|wget' "$script_dir/phase13-uart-native-reader.py" "$script_dir/ultra205-uart-capture-worker.sh"; then fail "forbidden operation present"; fi
	grep -Fq 'start_continuous_reader "$uart_port"' "$script_dir/ultra205-uart-capture-worker.sh" || fail "continuous reader missing"
}

test_success_keeps_uart_owned_across_native_removal
test_static_prohibitions
printf 'diagnose_ultra205_uart_capture_test passed\n'
