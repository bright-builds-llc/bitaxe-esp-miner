#!/usr/bin/env bash
# shellcheck source=scripts/phase28.1.1-terminal-closure-guard.sh
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Lifecycle owner for continuous receive-only external-UART qualification.
# shellcheck disable=SC2154 # Constants are defined by the sourced broker.
set -euo pipefail
umask 077

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/ultra205-uart-capture-broker.sh
source "$script_dir/ultra205-uart-capture-broker.sh"

readonly attempt_dir="${1:?attempt directory required}"
readonly socket_path="${2:?socket path required}"
readonly state_path="$attempt_dir/state.json"
readonly capability="${LATE_ATTACH_LIFECYCLE_CAPABILITY:?lifecycle capability required}"
readonly classifier_bin="$script_dir/ultra205-uart-capture-classifier.mjs"
readonly qualification_bin="${LATE_ATTACH_QUALIFICATION_BIN:-$script_dir/ultra205-transport-qualification.sh}"
readonly continuous_monitor_bin="${UART_CAPTURE_MONITOR_BIN:-$script_dir/phase13-monitor-capture.sh}"
receiver_pid=""
capture_pid=""

worker_step() {
	late_attach_atomic_write "$attempt_dir/worker-step" "$1"
}

worker_fail() {
	local category="$1"
	late_attach_atomic_write "$state_path" "$(jq -c --arg category "$category" '.state="failed" | .failure_category=$category' "$state_path")"
	late_attach_atomic_write "$attempt_dir/result.json" "$(jq -cn --arg category "$category" '{status:"failed",failure_category:$category}')"
	exit 1
}

stop_continuous_reader() {
	[[ -n "$capture_pid" ]] || return 0
	phase_process_group_terminate "$capture_pid" uart-continuous-reader >/dev/null 2>&1 || return 1
	phase_process_group_is_alive "$capture_pid" && return 1
	capture_pid=""
	return 0
}

worker_cleanup() {
	local status=$?
	if [[ -n "$receiver_pid" ]] && kill -0 "$receiver_pid" 2>/dev/null; then
		kill -TERM "$receiver_pid" 2>/dev/null || true
		wait "$receiver_pid" 2>/dev/null || true
	fi
	stop_continuous_reader || status=1
	rm -f "$socket_path"
	exit "$status"
}
trap worker_cleanup EXIT INT TERM

publish_action() {
	local published_ms="$1"
	shift
	late_attach_atomic_write "$attempt_dir/action.json" "$(jq -cn --argjson published "$published_ms" --args '$ARGS.positional as $lines | {published_ms:$published,lines:$lines}' -- "$@")"
}

wait_for_frame() {
	local frame_path="$1" deadline="$2"
	while [[ ! -s "$frame_path" ]]; do
		(($(late_attach_monotonic_ms) < deadline)) || worker_fail removal_checkpoint_expired
		if ! kill -0 "$receiver_pid" 2>/dev/null; then
			[[ -s "$frame_path" ]] || worker_fail lifecycle_frame_invalid
			break
		fi
		sleep 0.05
	done
	set +e
	wait "$receiver_pid"
	local receiver_status=$?
	set -e
	receiver_pid=""
	((receiver_status == 0)) || worker_fail lifecycle_frame_invalid
	[[ -f "$frame_path" && ! -L "$frame_path" && "$(late_attach_mode_of "$frame_path")" == 600 ]] || worker_fail lifecycle_frame_invalid
}

validate_frame() {
	local frame_path="$1"
	jq -e \
		--arg digest "$(jq -er '.resume_handle_sha256' "$state_path")" \
		--argjson owner "$(jq -er '.owner_pid' "$state_path")" '
      .resume_handle_sha256 == $digest and
      .checkpoint_token == "late-attach-removal-watcher-armed-v2" and
      .response_token == "late-attach-both-power-paths-removed-v2" and
      (.lifecycle_capability | type == "string") and .owner_pid == $owner
    ' "$frame_path" >/dev/null || worker_fail private_capability_invalid
	[[ "$(late_attach_sha256_text "$(jq -er '.lifecycle_capability' "$frame_path")")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] || worker_fail private_capability_invalid
	[[ "$(late_attach_process_fingerprint "$$")" == "$(jq -er '.owner_fingerprint_sha256' "$state_path")" ]] || worker_fail owner_process_reused
}

require_continuous_native_absence() {
	local native_port="$1"
	local interval="${LATE_ATTACH_ABSENCE_INTERVAL_SECONDS:-0.25}"
	local samples="${LATE_ATTACH_ABSENCE_SAMPLES:-20}"
	for _ in $(seq 1 "$samples"); do
		[[ ! -e "$native_port" ]] || worker_fail unstable_absence
		sleep "$interval"
	done
}

require_uart_owner() {
	local uart_port="$1"
	local expected_owner
	local observed_holders
	[[ -n "$capture_pid" ]] || worker_fail continuous_reader_missing
	if [[ -s "$attempt_dir/continuous-reader.group" ]]; then
		expected_owner="$(sed -n '1p' "$attempt_dir/continuous-reader.group")"
	else
		expected_owner="$capture_pid"
	fi
	if ! observed_holders="$(serial_session_holder_pids "$uart_port" 2>/dev/null)"; then
		observed_holders="unavailable"
	fi
	late_attach_atomic_write "$attempt_dir/uart-owner-observation" "expected_pgid=$expected_owner holders=$observed_holders holder_pgids=$(for holder in $observed_holders; do ps -o pgid= -p "$holder" 2>/dev/null | tr -d ' '; done | paste -sd, -)"
	serial_session_active_owner_gate "$uart_port" "$expected_owner" || worker_fail "uart_${SERIAL_SESSION_READINESS_CATEGORY}"
}

require_uart_identity() {
	local uart_port="$1" expected_physical="$2" expected_enumeration="$3"
	[[ -e "$uart_port" && -r "$uart_port" && -w "$uart_port" ]] || worker_fail uart_node_loss
	[[ "$(serial_session_usb_physical_identity "$uart_port" 2>/dev/null)" == "$expected_physical" ]] || worker_fail uart_physical_identity_changed
	[[ "$(serial_session_usb_enumeration_identity "$uart_port" 2>/dev/null)" == "$expected_enumeration" ]] || worker_fail uart_enumeration_identity_changed
}

record_quiet_boundary() {
	local raw_log="$1"
	local wait_seconds="${UART_CAPTURE_QUIET_SECONDS:-1}"
	local first_size second_size last_byte
	first_size="$(wc -c <"$raw_log" | tr -d ' ')"
	sleep "$wait_seconds"
	second_size="$(wc -c <"$raw_log" | tr -d ' ')"
	[[ "$first_size" == "$second_size" ]] || worker_fail quiet_boundary_unstable
	((second_size > 0)) || worker_fail quiet_boundary_empty
	last_byte="$(tail -c 1 "$raw_log" | od -An -tu1 | tr -d ' ')"
	[[ "$last_byte" == 10 ]] || worker_fail quiet_boundary_not_newline_aligned
	printf '%s\n' "$second_size"
}

wait_for_restore() {
	local native_port="$1" deadline="$2"
	while [[ ! -e "$native_port" ]]; do
		(($(late_attach_monotonic_ms) < deadline)) || worker_fail appearance_timeout
		sleep 0.1
	done
}

run_holder_free_soak() {
	local native_port="$1" uart_port="$2" native_physical="$3" native_enumeration="$4" uart_physical="$5" uart_enumeration="$6"
	local interval="${LATE_ATTACH_SOAK_INTERVAL_SECONDS:-0.25}"
	local samples="${LATE_ATTACH_SOAK_SAMPLES:-120}"
	local port expected_physical expected_enumeration maybe_holders holder_status
	for _ in $(seq 1 "$samples"); do
		for port in "$native_port" "$uart_port"; do
			if [[ "$port" == "$native_port" ]]; then
				expected_physical="$native_physical"
				expected_enumeration="$native_enumeration"
			else
				expected_physical="$uart_physical"
				expected_enumeration="$uart_enumeration"
			fi
			[[ -e "$port" && -r "$port" && -w "$port" ]] || worker_fail post_capture_node_loss
			[[ "$(serial_session_usb_physical_identity "$port" 2>/dev/null)" == "$expected_physical" ]] || worker_fail post_capture_physical_identity_changed
			[[ "$(serial_session_usb_enumeration_identity "$port" 2>/dev/null)" == "$expected_enumeration" ]] || worker_fail post_capture_enumeration_identity_changed
			set +e
			maybe_holders="$(serial_session_holder_pids "$port")"
			holder_status=$?
			set -e
			((holder_status == 0)) || worker_fail cleanup_probe_unavailable
			[[ -z "$maybe_holders" ]] || worker_fail cleanup_unexpected_holder
		done
		sleep "$interval"
	done
}

start_continuous_reader() {
	local uart_port="$1" capture_seconds="$2"
	local ready_file="$attempt_dir/continuous-reader.ready"
	local active_ready_file="$attempt_dir/continuous-reader-active.ready"
	local total_seconds="${UART_CAPTURE_CONTINUOUS_SECONDS:-$((late_attach_removal_timeout_ms / 1000 + late_attach_default_restore_timeout_ms / 1000 + capture_seconds + 120))}"
	PHASE13_MONITOR_ACTIVE_READY_FILE="$active_ready_file" \
		PHASE13_MONITOR_GROUP_STATE_FILE="$attempt_dir/continuous-reader.group" \
		SERIAL_SESSION_TRACE_ROOT="$attempt_dir/continuous-session-traces" \
		phase_process_group_start "$ready_file" "$continuous_monitor_bin" \
		--port "$uart_port" --out "$attempt_dir/continuous-uart.wrapper.log" \
		--raw-out "$attempt_dir/continuous-uart.raw.log" --reader uart-native \
		--seconds "$total_seconds" --no-reset || worker_fail continuous_reader_start_failed
	capture_pid="$PHASE_PROCESS_GROUP_PID"
	for _ in $(seq 1 500); do
		[[ -s "$active_ready_file" ]] && return
		kill -0 "$capture_pid" 2>/dev/null || worker_fail continuous_reader_start_failed
		sleep 0.01
	done
	worker_fail continuous_reader_ownership_timeout
}

main() {
	[[ -d "$attempt_dir" && "$(late_attach_mode_of "$attempt_dir")" == 700 ]] || worker_fail private_capability_invalid
	late_attach_validate_state "$state_path" || worker_fail state_malformed
	[[ "$(late_attach_sha256_text "$capability")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] || worker_fail private_capability_invalid
	local native_port uart_port frame_path action_ms removal_ms deadline restore_timeout_ms armed_ms appeared_ms capture_seconds native_physical old_native_enumeration restored_native_enumeration uart_physical uart_enumeration boundary cold_log preflight_session classification_path contract_digest trace_digest qualification_path adapter_binding
	native_port="$(jq -er '.selected_port' "$state_path")"
	uart_port="$(jq -er '.selected_uart_port' "$state_path")"
	native_physical="$(jq -er '.selected_usb_identity' "$state_path")"
	old_native_enumeration="$(jq -er '.selected_enumeration_identity' "$state_path")"
	uart_physical="$(jq -er '.selected_uart_physical_identity' "$state_path")"
	uart_enumeration="$(jq -er '.selected_uart_enumeration_identity' "$state_path")"
	capture_seconds="$(jq -er '.capture_seconds' "$state_path")"
	frame_path="$attempt_dir/lifecycle-frame.json"
	rm -f "$socket_path" "$frame_path"

	perl "$late_attach_frame_helper" receive --socket "$socket_path" --output "$frame_path" &
	receiver_pid=$!
	for _ in $(seq 1 200); do
		[[ -S "$socket_path" ]] && break
		kill -0 "$receiver_pid" 2>/dev/null || worker_fail lifecycle_socket_unavailable
		sleep 0.01
	done
	[[ -S "$socket_path" ]] || worker_fail lifecycle_socket_unavailable
	chmod 600 "$socket_path"
	late_attach_atomic_write "$attempt_dir/capability.escrow" "capability=$capability"

	start_continuous_reader "$uart_port" "$capture_seconds"
	worker_step continuous_reader_started
	require_uart_identity "$uart_port" "$uart_physical" "$uart_enumeration"
	worker_step continuous_reader_identity_verified
	require_uart_owner "$uart_port"
	worker_step continuous_reader_owner_verified
	action_ms="$(late_attach_monotonic_ms)"
	late_attach_atomic_write "$state_path" "$(jq -c --argjson published "$action_ms" '.action_published_ms=$published' "$state_path")"
	publish_action "$action_ms" \
		'## ACTION READY' \
		'action_id=uart-capture-lifecycle-removal' \
		'action_token=uart-capture-removal-watcher-armed-v1' \
		'attempt_state=waiting_removal' \
		'expected_user_action=remove-barrel-and-native-usb-leave-uart-adapter-connected' \
		'response_required=true' \
		'expected_response_token=uart-capture-both-board-power-paths-removed-v1'
	worker_step removal_action_published

	while [[ -e "$native_port" ]]; do
		(($(late_attach_monotonic_ms) < $(jq -er '.checkpoint_deadline_ms' "$state_path"))) || worker_fail removal_checkpoint_expired
		require_uart_identity "$uart_port" "$uart_physical" "$uart_enumeration"
		sleep 0.05
	done
	removal_ms="$(late_attach_monotonic_ms)"
	((removal_ms >= action_ms)) || worker_fail removal_observation_order_invalid
	late_attach_atomic_write "$state_path" "$(jq -c --argjson observed "$removal_ms" '.state="removal_observed" | .removal_observed_ms=$observed' "$state_path")"
	deadline="$(jq -er '.checkpoint_deadline_ms' "$state_path")"
	wait_for_frame "$frame_path" "$deadline"
	validate_frame "$frame_path"
	require_continuous_native_absence "$native_port"
	require_uart_identity "$uart_port" "$uart_physical" "$uart_enumeration"
	require_uart_owner "$uart_port"
	boundary="$(record_quiet_boundary "$attempt_dir/continuous-uart.raw.log")"
	late_attach_atomic_write "$attempt_dir/cold-byte-boundary" "$boundary"

	restore_timeout_ms="${LATE_ATTACH_RESTORE_TIMEOUT_MS:-$late_attach_default_restore_timeout_ms}"
	armed_ms="$(late_attach_monotonic_ms)"
	late_attach_atomic_write "$state_path" "$(jq -c --argjson armed "$armed_ms" --argjson deadline "$((armed_ms + restore_timeout_ms))" --argjson boundary "$boundary" '.state="waiting_restore" | .watcher_armed_ms=$armed | .watcher_deadline_ms=$deadline | .cold_byte_boundary=$boundary' "$state_path")"
	publish_action "$armed_ms" \
		'## ACTION READY' \
		'action_id=uart-capture-lifecycle-restore' \
		'action_token=uart-capture-cold-reader-armed-v1' \
		'attempt_state=waiting_restore' \
		'expected_user_action=restore-barrel-then-native-usb-leave-uart-adapter-connected' \
		'response_required=false'

	wait_for_restore "$native_port" "$((armed_ms + restore_timeout_ms))"
	appeared_ms="$(late_attach_monotonic_ms)"
	serial_session_readiness_gate uart_native_restore "$native_port" || worker_fail "appearance_${SERIAL_SESSION_READINESS_CATEGORY}"
	[[ "$SERIAL_SESSION_READY_PHYSICAL_IDENTITY" == "$native_physical" ]] || worker_fail appearance_physical_identity_changed
	restored_native_enumeration="$SERIAL_SESSION_READY_ENUMERATION_IDENTITY"
	[[ "$restored_native_enumeration" != "$old_native_enumeration" ]] || worker_fail enumeration_epoch_unchanged
	require_uart_identity "$uart_port" "$uart_physical" "$uart_enumeration"
	require_uart_owner "$uart_port"
	late_attach_atomic_write "$state_path" "$(jq -c --argjson appeared "$appeared_ms" '.state="capturing" | .usb_appeared_ms=$appeared | .monitor_attached_ms=$appeared' "$state_path")"
	sleep "$capture_seconds"

	cold_log="$attempt_dir/cold-uart.raw.log"
	dd if="$attempt_dir/continuous-uart.raw.log" of="$cold_log" bs=1 skip="$boundary" 2>/dev/null
	chmod 600 "$cold_log"
	preflight_session="$(jq -er '.session' "$attempt_dir/preflight.json")"
	classification_path="$attempt_dir/cold-classification.json"
	set +e
	node "$classifier_bin" cold "$cold_log" "$preflight_session" >"$classification_path"
	local classifier_status=$?
	set -e
	chmod 600 "$classification_path"
	((classifier_status == 0)) || worker_fail cold_uart_evidence_invalid

	stop_continuous_reader || worker_fail cleanup_process_survived
	run_holder_free_soak "$native_port" "$uart_port" "$native_physical" "$restored_native_enumeration" "$uart_physical" "$uart_enumeration"
	contract_digest="$($qualification_bin contract-digest)" || worker_fail contract_digest_unavailable
	trace_digest="$(late_attach_trace_digest "$attempt_dir")"
	adapter_binding="$(late_attach_sha256_text "$uart_physical:$uart_enumeration")"
	qualification_path="$attempt_dir/qualification.json"
	jq -n \
		--slurpfile state "$state_path" \
		--slurpfile preflight "$attempt_dir/preflight.json" \
		--slurpfile cold "$classification_path" \
		--arg contract "$contract_digest" --arg trace "$trace_digest" --arg adapter "$adapter_binding" \
		'{schema_version:"ultra205-transport-qualification-v3",tool_head:$state[0].tool_head,expected_firmware_head:$state[0].expected_firmware_head,classification_category:"uart_cold_delivers",native_preflight_heartbeat_count:$preflight[0].espflashHeartbeatCount,uart_preflight_heartbeat_count:$preflight[0].osNativeHeartbeatCount,cold_uart_heartbeat_count:$cold[0].heartbeat_count,native_physical_identity_stable:true,native_new_enumeration_epoch:true,uart_physical_identity_stable:true,uart_enumeration_identity_stable:true,quiet_boundary_complete:true,original_boot_present:($cold[0].original_boot_count == 1),original_listener_present:($cold[0].original_listener_count == 1),boot_evidence_complete:$cold[0].evidence_states_complete,accepted_state_stages_complete:$cold[0].accepted_state_stages_complete,heartbeat_monotonic:$cold[0].heartbeat_monotonic,listener_ready:$cold[0].listener_ready,soak_complete:true,cleanup_complete:false,adapter_binding_sha256:$adapter,diagnostic_contract_digest_sha256:$contract,trace_digest_sha256:$trace}' >"$qualification_path"
	chmod 600 "$qualification_path"
	late_attach_atomic_write "$state_path" "$(jq -c '.state="complete"' "$state_path")"
	late_attach_atomic_write "$attempt_dir/result.json" "$(jq -cn '{status:"complete",classification_category:"uart_cold_delivers"}')"
}

main
