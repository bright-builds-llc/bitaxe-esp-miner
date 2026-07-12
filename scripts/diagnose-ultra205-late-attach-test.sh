#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly diagnostic="${LATE_ATTACH_DIAGNOSTIC_SCRIPT:-$script_dir/diagnose-ultra205-late-attach.sh}"
readonly expected_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/late-attach-v2-test.XXXXXX")"
readonly tmp_root
cleanup() { [[ "${KEEP_LATE_ATTACH_TESTS:-0}" == 1 ]] || rm -rf "$tmp_root"; }
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

mode_of() {
	if stat -f '%Lp' "$1" >/dev/null 2>&1; then stat -f '%Lp' "$1"; else stat -c '%a' "$1"; fi
}

write_executable() {
	printf '#!%s\n%s\n' "$BASH" "$2" >"$1"
	chmod +x "$1"
}

create_fixtures() {
	local root="$1"
	local bin="$root/bin"
	mkdir -p "$bin"
	: >"$root/command.log"
	: >"$root/detector.count"
	: >"$root/monitor.count"
	: >"$root/port"
	printf 'node-before\n' >"$root/node.identity"
	printf 'usb-stable\n' >"$root/usb.identity"
	chmod 600 "$root/port"
	# shellcheck disable=SC2016
	write_executable "$bin/detector" 'count="$(cat "${TEST_DETECTOR_COUNT:?}")"; count="${count:-0}"; printf "%s\n" "$((count + 1))" >"${TEST_DETECTOR_COUNT:?}"; printf "port=%s\n" "${TEST_PORT:?}"'
	# shellcheck disable=SC2016
	write_executable "$bin/monitor" 'printf "monitor %s\n" "$*" >>"${TEST_COMMAND_LOG:?}"
count="$(cat "${TEST_MONITOR_COUNT:?}")"; count="${count:-0}"; count=$((count + 1)); printf "%s\n" "$count" >"${TEST_MONITOR_COUNT:?}"
out=""; raw=""; reader=""
while (($#)); do case "$1" in --out) out="$2"; shift 2;; --raw-out) raw="$2"; shift 2;; --reader) reader="$2"; shift 2;; *) shift;; esac; done
printf "capture_status=timed_out_after_capture\nreader=%s\n" "$reader" >"$out"; : >"$raw"
if [[ "${TEST_ESPFLASH_CAPTURE_FAIL:-0}" == 1 && "$count" == 1 ]]; then exit 7; fi
if [[ "${TEST_REMOVE_DURING_PREFLIGHT:-0}" == 1 && "$count" == 2 ]]; then rm -f "${TEST_PORT:?}"; fi
if [[ "$count" == 2 && "${TEST_OS_PREFLIGHT_EMPTY:-0}" != 1 ]]; then start=1; elif [[ "$count" == 3 ]]; then start=4; else start=0; fi
if ((start > 0)); then printf "ordinary firmware diagnostic\n" >>"$raw"; for offset in 0 1 2; do sequence=$((start + offset)); uptime=$((120000 + sequence * 10000)); printf "runtime_heartbeat session=0123456789abcdef0123456789abcdef sequence=%s uptime_ms=%s cadence_ms=10000 listener_armed=true redacted=true\n" "$sequence" "$uptime" >>"$raw"; done; fi
chmod 600 "$out" "$raw"'
	# shellcheck disable=SC2016
	write_executable "$bin/node-identity" 'cat "${TEST_NODE_IDENTITY_FILE:?}"'
	# shellcheck disable=SC2016
	write_executable "$bin/usb-identity" 'cat "${TEST_USB_IDENTITY_FILE:?}"'
	write_executable "$bin/lsof-none" 'exit 1'
	write_executable "$bin/lsof-unavailable" 'exit 70'
	# shellcheck disable=SC2016 # Runtime fixture arguments belong to the generated process.
	write_executable "$bin/contract" 'case "$1" in contract-digest) printf "%064d\n" 0 ;; validate) exit 0 ;; *) exit 2 ;; esac'
}

set_env() {
	local root="$1"
	COMMON_ENV=(env LATE_ATTACH_TEST_MODE=1 LATE_ATTACH_CONTROL_ROOT="$root/control" LATE_ATTACH_TRACE_ROOT="$root/traces" LATE_ATTACH_DETECTOR_BIN="$root/bin/detector" LATE_ATTACH_MONITOR_BIN="$root/bin/monitor" LATE_ATTACH_QUALIFICATION_BIN="$root/bin/contract" LATE_ATTACH_PREFLIGHT_SECONDS=1 LATE_ATTACH_ABSENCE_INTERVAL_SECONDS=0.01 LATE_ATTACH_ABSENCE_SAMPLES=2 LATE_ATTACH_RESTORE_TIMEOUT_MS="${TEST_RESTORE_TIMEOUT_MS:-5000}" LATE_ATTACH_SOAK_INTERVAL_SECONDS=0.01 LATE_ATTACH_SOAK_SAMPLES=2 LATE_ATTACH_RESULT_WAIT_SAMPLES=2000 LATE_ATTACH_WORKER_EXIT_DELAY_SECONDS="${TEST_WORKER_EXIT_DELAY_SECONDS:-}" SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 SERIAL_SESSION_NODE_IDENTITY_BIN="$root/bin/node-identity" SERIAL_SESSION_USB_IDENTITY_BIN="$root/bin/usb-identity" SERIAL_SESSION_LSOF_BIN="${TEST_LSOF_BIN:-$root/bin/lsof-none}" TEST_DETECTOR_COUNT="$root/detector.count" TEST_MONITOR_COUNT="$root/monitor.count" TEST_COMMAND_LOG="$root/command.log" TEST_NODE_IDENTITY_FILE="$root/node.identity" TEST_USB_IDENTITY_FILE="$root/usb.identity" TEST_PORT="$root/port")
}

begin() {
	local root="$1"
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=3 >"$root/begin.out" 2>&1
	HANDLE="$(sed -n 's/^resume_handle=//p' "$root/begin.out" | head -1)"
	[[ "$HANDLE" =~ ^[0-9a-f]{64}$ ]] || fail 'opaque handle missing'
}

attempt_dir() { find "$1/control/attempts" -mindepth 1 -maxdepth 1 -type d | head -1; }

wait_for_state() {
	local root="$1" wanted="$2" directory
	directory="$(attempt_dir "$root")"
	for _ in $(seq 1 500); do
		[[ "$(jq -r '.state' "$directory/state.json")" == "$wanted" ]] && return
		sleep 0.01
	done
	fail "state $wanted not reached"
}

test_success_uses_os_native_as_first_and_only_cold_reader() {
	local root="$tmp_root/success" directory run_dir pid
	mkdir -p "$root"
	create_fixtures "$root"
	TEST_WORKER_EXIT_DELAY_SECONDS=0.2 begin "$root"
	grep -Fq 'action_token=late-attach-removal-watcher-armed-v2' "$root/begin.out" || fail 'removal watcher action missing'
	grep -Fq 'response_required=true' "$root/begin.out" || fail 'removal response contract missing'
	[[ "$(cat "$root/monitor.count")" == 2 ]] || fail 'connected preflight count wrong'
	grep -Fq -- '--reader espflash' "$root/command.log" || fail 'observational espflash control absent'
	[[ "$(grep -c -- '--reader os-native' "$root/command.log")" == 1 ]] || fail 'preflight OS reader count wrong'

	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$HANDLE" >"$root/status.out"
	grep -Fq 'action_token=late-attach-removal-watcher-armed-v2' "$root/status.out" || fail 'status did not re-emit action'
	directory="$(attempt_dir "$root")"
	[[ "$(jq -r '.state' "$directory/state.json")" == waiting_removal ]] || fail 'status advanced state'

	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$HANDLE" checkpoint-token=late-attach-removal-watcher-armed-v2 response-token=late-attach-both-power-paths-removed-v2 >"$root/early.out" 2>&1
	local early_status=$?
	set -e
	((early_status != 0)) || fail 'absence-only early token passed'
	grep -Fq 'late_attach_error=removal_not_observed' "$root/early.out" || fail 'early token category wrong'

	rm -f "$root/port"
	wait_for_state "$root" removal_observed
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$HANDLE" checkpoint-token=late-attach-removal-watcher-armed-v2 response-token=late-attach-both-power-paths-removed-v2 >"$root/deliver.out" 2>&1 &
	pid=$!
	for _ in $(seq 1 500); do
		jq -e '.lines | index("action_token=late-attach-os-native-watcher-armed-v2") != null' "$directory/action.json" >/dev/null 2>&1 && break
		sleep 0.01
	done
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$HANDLE" >"$root/restore-status.out"
	grep -Fq 'response_required=false' "$root/restore-status.out" || fail 'restore watcher action missing'
	printf 'node-after\n' >"$root/node.identity"
	: >"$root/port"
	chmod 600 "$root/port"
	for _ in $(seq 1 500); do
		[[ -f "$directory/qualification.json" ]] && break
		sleep 0.01
	done
	jq -e '.cleanup_complete == false' "$directory/qualification.json" >/dev/null || fail 'qualification claimed cleanup before owner exit'
	wait "$pid"
	grep -Fq 'classification_category=os_native_cold_delivers' "$root/deliver.out" || {
		cat "$root/deliver.out" >&2
		fail 'qualification did not pass'
	}
	[[ "$(cat "$root/monitor.count")" == 3 ]] || fail 'cold qualification used more than one reader'
	[[ "$(tail -1 "$root/command.log")" == *'--reader os-native'* ]] || fail 'OS-native was not first cold open'
	run_dir="$(find "$root/traces" -mindepth 1 -maxdepth 1 -type d | head -1)"
	jq -e '.schema_version == "ultra205-transport-qualification-v2" and .classification_category == "os_native_cold_delivers" and .cold_os_native_heartbeat_count == 3 and .identity_stable and .new_enumeration_epoch and .soak_complete and .cleanup_complete' "$run_dir/qualification.json" >/dev/null || fail 'private qualification summary malformed'
	[[ "$(mode_of "$run_dir")" == 700 && "$(mode_of "$run_dir/qualification.json")" == 600 ]] || fail 'private permissions wrong'
	if grep -Eq '/dev/|0123456789abcdef|owner_pid|selected_port' "$run_dir/qualification.json"; then fail 'qualification exposed raw identity'; fi
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$HANDLE" >"$root/terminal-status.out"
	grep -Fq 'terminal_category=diagnostic_complete' "$root/terminal-status.out" || fail 'terminal status not re-emitted'
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$HANDLE" checkpoint-token=late-attach-removal-watcher-armed-v2 response-token=late-attach-both-power-paths-removed-v2 >"$root/stale.out" 2>&1
	local stale_status=$?
	set -e
	((stale_status != 0)) || fail 'stale handle was accepted'
	grep -Fq 'late_attach_error=resume_handle_stale' "$root/stale.out" || fail 'stale handle category wrong'
}

test_os_native_preflight_is_required_but_espflash_is_not() {
	local root="$tmp_root/preflight-fail"
	mkdir -p "$root"
	create_fixtures "$root"
	set_env "$root"
	set +e
	TEST_OS_PREFLIGHT_EMPTY=1 "${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=3 >"$root/out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail 'empty OS-native preflight passed'
	grep -Fq 'failure_category=preflight_heartbeat_validation_failed' "$root/out" || fail 'OS preflight failure not classified'
	! grep -Fq 'expected_user_action=remove-both-power-paths' "$root/out" || fail 'physical action published after failed preflight'
}

test_espflash_silence_is_observational_but_capture_failure_stops() {
	local root="$tmp_root/espflash-capture-fail"
	mkdir -p "$root"
	create_fixtures "$root"
	set_env "$root"
	set +e
	TEST_ESPFLASH_CAPTURE_FAIL=1 "${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=3 >"$root/out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail 'failed espflash control passed'
	grep -Fq 'failure_category=preflight_espflash_capture_failed' "$root/out" || fail 'espflash capture failure category wrong'
	[[ "$(cat "$root/monitor.count")" == 1 ]] || fail 'OS-native reader ran after espflash capture failure'
	! grep -Fq 'action_token=' "$root/out" || fail 'action published after espflash capture failure'
}

test_node_loss_before_action_fails_without_instruction() {
	local root="$tmp_root/node-loss"
	mkdir -p "$root"
	create_fixtures "$root"
	set_env "$root"
	set +e
	TEST_REMOVE_DURING_PREFLIGHT=1 "${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=3 >"$root/out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail 'pre-removal node loss passed'
	grep -Fq 'failure_category=pre_removal_node_loss' "$root/out" || fail 'node loss category wrong'
	! grep -Fq 'action_token=' "$root/out" || fail 'action published after node loss'
}

test_unavailable_holder_probe_stops_before_readers() {
	local root="$tmp_root/probe-unavailable"
	mkdir -p "$root"
	create_fixtures "$root"
	TEST_LSOF_BIN="$root/bin/lsof-unavailable" set_env "$root"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" begin expected-firmware-head="$expected_head" capture-seconds=3 >"$root/begin.out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail 'unavailable holder probe passed'
	grep -Fq 'failure_category=preflight_ownership_probe_unavailable' "$root/begin.out" || fail 'probe failure category wrong'
	[[ ! -s "$root/monitor.count" ]] || fail 'readers ran after unavailable holder probe'
}

test_restore_timeout_is_tombstoned_without_a_live_deliver_process() {
	local root="$tmp_root/restore-timeout"
	mkdir -p "$root"
	create_fixtures "$root"
	TEST_RESTORE_TIMEOUT_MS=100 begin "$root"
	rm -f "$root/port"
	wait_for_state "$root" removal_observed
	set_env "$root"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$HANDLE" checkpoint-token=late-attach-removal-watcher-armed-v2 response-token=late-attach-both-power-paths-removed-v2 >"$root/deliver.out" 2>&1
	local status=$?
	set -e
	((status != 0)) || fail 'restore timeout passed'
	grep -Fq 'failure_category=appearance_timeout' "$root/deliver.out" || fail 'restore timeout failure category missing'
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$HANDLE" >"$root/status.out"
	grep -Fq 'terminal_category=appearance_timeout' "$root/status.out" || fail 'terminal timeout status was not resumable'
}

test_crashed_owner_is_cleaned_and_tombstoned_by_status() {
	local root="$tmp_root/owner-crash" directory owner
	mkdir -p "$root"
	create_fixtures "$root"
	begin "$root"
	directory="$(attempt_dir "$root")"
	owner="$(jq -er '.owner_pid' "$directory/state.json")"
	kill -KILL -- "-$owner" 2>/dev/null || kill -KILL "$owner" 2>/dev/null
	for _ in $(seq 1 100); do
		kill -0 "$owner" 2>/dev/null || break
		sleep 0.01
	done
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$HANDLE" >"$root/status.out"
	grep -Fq 'terminal_category=owner_process_stale' "$root/status.out" || fail 'crashed owner was not tombstoned'
	grep -Fq 'cleanup_complete=true' "$root/status.out" || fail 'crashed owner cleanup was not proven'
	[[ ! -S "$(jq -r '.socket_path' "$root/traces"/*/state.json)" ]] || fail 'crashed owner socket survived cleanup'
}

test_restore_identity_change_fails_before_cold_capture() {
	local root="$tmp_root/identity-change" directory pid
	mkdir -p "$root"
	create_fixtures "$root"
	begin "$root"
	directory="$(attempt_dir "$root")"
	rm -f "$root/port"
	wait_for_state "$root" removal_observed
	set_env "$root"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" deliver resume-handle="$HANDLE" checkpoint-token=late-attach-removal-watcher-armed-v2 response-token=late-attach-both-power-paths-removed-v2 >"$root/deliver.out" 2>&1 &
	pid=$!
	for _ in $(seq 1 500); do
		jq -e '.lines | index("action_token=late-attach-os-native-watcher-armed-v2") != null' "$directory/action.json" >/dev/null 2>&1 && break
		sleep 0.01
	done
	printf 'usb-changed\n' >"$root/usb.identity"
	printf 'node-after\n' >"$root/node.identity"
	: >"$root/port"
	chmod 600 "$root/port"
	set +e
	wait "$pid"
	local status=$?
	set -e
	((status != 0)) || fail 'changed USB identity passed'
	grep -Fq 'failure_category=appearance_identity_changed' "$root/deliver.out" || fail 'identity change category wrong'
	[[ "$(cat "$root/monitor.count")" == 2 ]] || fail 'cold reader ran after identity change'
}

test_v1_live_handle_is_not_resumed_and_v1_tombstone_is_readable() {
	local root="$tmp_root/v1" handle digest slot
	mkdir -p "$root/control/resume-index"
	chmod 700 "$root/control" "$root/control/resume-index"
	handle="$(printf 'ab%.0s' $(seq 1 32))"
	digest="$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}')"
	slot="$root/control/resume-index/$digest.json"
	jq -n --arg digest "$digest" '{schema_version:"ultra205-late-attach-resume-v1",status:"active",resume_handle_sha256:$digest,attempt_id:"0123456789abcdef0123456789abcdef",attempt_dir:"/private/old"}' >"$slot"
	chmod 600 "$slot"
	set_env "$root"
	set +e
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$handle" >"$root/live.out" 2>&1
	local live_status=$?
	set -e
	((live_status != 0)) || fail 'v1 live attempt resumed'
	jq -n --arg digest "$digest" '{schema_version:"ultra205-late-attach-tombstone-v1",status:"closed",resume_handle_sha256:$digest,terminal_category:"old_complete",classification_category:"not_classified",cleanup_complete:true}' >"$slot"
	chmod 600 "$slot"
	"${COMMON_ENV[@]}" "$BASH" "$diagnostic" status resume-handle="$handle" >"$root/tombstone.out"
	grep -Fq 'terminal_category=old_complete' "$root/tombstone.out" || fail 'v1 tombstone unreadable'
}

test_static_forbidden_operations() {
	if rg -n 'syswrite|ioctl|termios|stty|erase-flash|write-flash|factory-reset|nmap|curl|wget' "$script_dir/phase13-os-native-reader.pl" "$script_dir/ultra205-late-attach-worker.sh"; then fail 'forbidden operation present'; fi
	# shellcheck disable=SC2016 # The literal verifies worker source, not this test shell.
	grep -Fq 'late_attach_run_capture "$attempt_dir" "$port" os-native' "$script_dir/ultra205-late-attach-worker.sh" || fail 'OS-native cold reader absent'
	# shellcheck disable=SC2016 # The literal verifies worker source, not this test shell.
	! grep -Fq 'late_attach_run_capture "$attempt_dir" "$port" espflash' "$script_dir/ultra205-late-attach-worker.sh" || fail 'cold worker invokes espflash'
}

test_success_uses_os_native_as_first_and_only_cold_reader
test_os_native_preflight_is_required_but_espflash_is_not
test_espflash_silence_is_observational_but_capture_failure_stops
test_node_loss_before_action_fails_without_instruction
test_unavailable_holder_probe_stops_before_readers
test_restore_timeout_is_tombstoned_without_a_live_deliver_process
test_crashed_owner_is_cleaned_and_tombstoned_by_status
test_restore_identity_change_fails_before_cold_capture
test_v1_live_handle_is_not_resumed_and_v1_tombstone_is_readable
test_static_forbidden_operations

printf 'diagnose_ultra205_late_attach_test passed\n'
