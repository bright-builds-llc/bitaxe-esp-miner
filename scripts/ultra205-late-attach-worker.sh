#!/usr/bin/env bash
# Lifecycle owner for the Ultra 205 OS-native cold-attach qualification.
# shellcheck disable=SC2154 # Constants are defined by the sourced broker.
set -euo pipefail
umask 077

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/ultra205-late-attach-broker.sh
source "$script_dir/ultra205-late-attach-broker.sh"

readonly attempt_dir="${1:?attempt directory required}"
readonly socket_path="${2:?socket path required}"
readonly state_path="$attempt_dir/state.json"
readonly capability="${LATE_ATTACH_LIFECYCLE_CAPABILITY:?lifecycle capability required}"
readonly qualification_bin="${LATE_ATTACH_QUALIFICATION_BIN:-$script_dir/ultra205-transport-qualification.sh}"
receiver_pid=""

worker_fail() {
	local category="$1"
	late_attach_atomic_write "$state_path" "$(jq -c --arg category "$category" '.state="failed" | .failure_category=$category' "$state_path")"
	late_attach_atomic_write "$attempt_dir/result.json" "$(jq -cn --arg category "$category" '{status:"failed",failure_category:$category}')"
	exit 1
}

worker_cleanup() {
	local status=$?
	if [[ -n "$receiver_pid" ]] && kill -0 "$receiver_pid" 2>/dev/null; then
		kill -TERM "$receiver_pid" 2>/dev/null || true
		wait "$receiver_pid" 2>/dev/null || true
	fi
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
			for _ in $(seq 1 20); do
				[[ -s "$frame_path" ]] && break
				sleep 0.01
			done
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
	late_attach_atomic_write "$attempt_dir/receiver.status" "receiver_exit_status=$receiver_status"
	[[ -f "$frame_path" && ! -L "$frame_path" && "$(late_attach_mode_of "$frame_path")" == 600 ]] || worker_fail lifecycle_frame_invalid
}

validate_frame() {
	local frame_path="$1" expected_digest expected_capability expected_owner
	expected_digest="$(jq -er '.resume_handle_sha256' "$state_path")"
	expected_capability="$(jq -er '.lifecycle_capability_sha256' "$state_path")"
	expected_owner="$(jq -er '.owner_pid' "$state_path")"
	jq -e --arg digest "$expected_digest" --argjson owner "$expected_owner" '
      .resume_handle_sha256 == $digest and
      .checkpoint_token == "late-attach-removal-watcher-armed-v2" and
      .response_token == "late-attach-both-power-paths-removed-v2" and
      (.lifecycle_capability | type == "string") and .owner_pid == $owner
    ' "$frame_path" >/dev/null || worker_fail private_capability_invalid
	[[ "$(late_attach_sha256_text "$(jq -er '.lifecycle_capability' "$frame_path")")" == "$expected_capability" ]] || worker_fail private_capability_invalid
	[[ "$(late_attach_process_fingerprint "$$")" == "$(jq -er '.owner_fingerprint_sha256' "$state_path")" ]] || worker_fail owner_process_reused
}

require_continuous_absence() {
	local port="$1" interval="${LATE_ATTACH_ABSENCE_INTERVAL_SECONDS:-0.25}" samples="${LATE_ATTACH_ABSENCE_SAMPLES:-20}"
	for _ in $(seq 1 "$samples"); do
		[[ ! -e "$port" ]] || worker_fail unstable_absence
		sleep "$interval"
	done
}

wait_for_restore() {
	local port="$1" deadline="$2"
	while [[ ! -e "$port" ]]; do
		(($(late_attach_monotonic_ms) < deadline)) || worker_fail appearance_timeout
		sleep 0.1
	done
}

run_soak() {
	local port="$1" expected_usb="$2" interval="${LATE_ATTACH_SOAK_INTERVAL_SECONDS:-0.25}" samples="${LATE_ATTACH_SOAK_SAMPLES:-120}" maybe_holders holder_status
	for _ in $(seq 1 "$samples"); do
		[[ -e "$port" && -r "$port" && -w "$port" ]] || worker_fail post_capture_node_loss
		[[ "$(serial_session_usb_identity "$port" 2>/dev/null)" == "$expected_usb" ]] || worker_fail post_capture_identity_changed
		set +e
		maybe_holders="$(serial_session_holder_pids "$port")"
		holder_status=$?
		set -e
		((holder_status == 0)) || worker_fail cleanup_probe_unavailable
		[[ -z "$maybe_holders" ]] || worker_fail cleanup_unexpected_holder
		sleep "$interval"
	done
}

main() {
	[[ -d "$attempt_dir" && "$(late_attach_mode_of "$attempt_dir")" == 700 ]] || worker_fail private_capability_invalid
	late_attach_validate_state "$state_path" || worker_fail state_malformed
	[[ "$(late_attach_sha256_text "$capability")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] || worker_fail private_capability_invalid
	local port frame_path action_ms removal_ms deadline restore_timeout_ms armed_ms observed_node observed_usb old_node old_usb appeared_ms capture_seconds qualification_path contract_digest trace_digest
	port="$(jq -er '.selected_port' "$state_path")"
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
	action_ms="$(late_attach_monotonic_ms)"
	late_attach_atomic_write "$state_path" "$(jq -c --argjson published "$action_ms" '.action_published_ms=$published' "$state_path")"
	publish_action "$action_ms" \
		'## ACTION READY' \
		'action_id=late-attach-lifecycle-removal' \
		'action_token=late-attach-removal-watcher-armed-v2' \
		'attempt_state=waiting_removal' \
		'expected_user_action=remove-both-power-paths' \
		'response_required=true' \
		'expected_response_token=late-attach-both-power-paths-removed-v2'
	while [[ -e "$port" ]]; do
		(($(late_attach_monotonic_ms) < $(jq -er '.checkpoint_deadline_ms' "$state_path"))) || worker_fail removal_checkpoint_expired
		sleep 0.05
	done
	removal_ms="$(late_attach_monotonic_ms)"
	((removal_ms >= action_ms)) || worker_fail removal_observation_order_invalid
	late_attach_atomic_write "$state_path" "$(jq -c --argjson observed "$removal_ms" '.state="removal_observed" | .removal_observed_ms=$observed' "$state_path")"
	deadline="$(jq -er '.checkpoint_deadline_ms' "$state_path")"
	wait_for_frame "$frame_path" "$deadline"
	chmod 600 "$frame_path"
	validate_frame "$frame_path"
	require_continuous_absence "$port"
	restore_timeout_ms="${LATE_ATTACH_RESTORE_TIMEOUT_MS:-$late_attach_default_restore_timeout_ms}"
	armed_ms="$(late_attach_monotonic_ms)"
	late_attach_atomic_write "$state_path" "$(jq -c --argjson armed "$armed_ms" --argjson deadline "$((armed_ms + restore_timeout_ms))" '.state="waiting_restore" | .watcher_armed_ms=$armed | .watcher_deadline_ms=$deadline' "$state_path")"
	publish_action "$armed_ms" \
		'## ACTION READY' \
		'action_id=late-attach-lifecycle-restore' \
		'action_token=late-attach-os-native-watcher-armed-v2' \
		'attempt_state=waiting_restore' \
		'expected_user_action=restore-barrel-then-usb' \
		'response_required=false'
	wait_for_restore "$port" "$((armed_ms + restore_timeout_ms))"
	appeared_ms="$(late_attach_monotonic_ms)"
	old_node="$(jq -er '.selected_node_identity' "$state_path")"
	old_usb="$(jq -er '.selected_usb_identity' "$state_path")"
	observed_node="$(serial_session_node_identity "$port" 2>/dev/null)" || worker_fail appearance_identity_unavailable
	observed_usb="$(serial_session_usb_identity "$port" 2>/dev/null)" || worker_fail appearance_identity_unavailable
	[[ "$observed_usb" == "$old_usb" ]] || worker_fail appearance_identity_changed
	[[ "$observed_node" != "$old_node" ]] || worker_fail enumeration_epoch_unchanged
	# shellcheck disable=SC2034 # Read by sourced serial-session helpers.
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/session-traces"
	serial_session_trace_init late-attach-os-native-appearance
	serial_session_readiness_gate late_attach_appearance "$port" || worker_fail "appearance_${SERIAL_SESSION_READINESS_CATEGORY}"
	late_attach_atomic_write "$state_path" "$(jq -c --argjson appeared "$appeared_ms" '.state="capturing" | .usb_appeared_ms=$appeared | .monitor_attached_ms=$appeared' "$state_path")"
	capture_seconds="$(jq -er '.capture_seconds' "$state_path")"
	late_attach_run_capture "$attempt_dir" "$port" os-native "$capture_seconds" cold-os-native || worker_fail cold_os_native_capture_failed
	qualification_path="$attempt_dir/cold-qualification.json"
	set +e
	node "$late_attach_classifier_bin" qualify-os-native "$attempt_dir/cold-os-native.raw.log" >"$qualification_path"
	local classifier_status=$?
	set -e
	chmod 600 "$qualification_path"
	((classifier_status == 0)) || worker_fail cold_heartbeat_invalid
	run_soak "$port" "$old_usb"
	serial_session_readiness_gate terminal_cleanup "$port" || worker_fail "cleanup_${SERIAL_SESSION_READINESS_CATEGORY}"
	contract_digest="$($qualification_bin contract-digest)" || worker_fail contract_digest_unavailable
	trace_digest="$(late_attach_trace_digest "$attempt_dir")"
	jq -n --slurpfile state "$state_path" --slurpfile cold "$qualification_path" --arg contract "$contract_digest" --arg trace "$trace_digest" '{schema_version:"ultra205-transport-qualification-v2",tool_head:$state[0].tool_head,expected_firmware_head:$state[0].expected_firmware_head,classification_category:"os_native_cold_delivers",preflight_espflash_heartbeat_count:$state[0].preflight_espflash_heartbeat_count,preflight_os_native_heartbeat_count:$state[0].preflight_os_native_heartbeat_count,cold_os_native_heartbeat_count:$cold[0].heartbeat_count,identity_stable:true,new_enumeration_epoch:true,soak_complete:true,cleanup_complete:false,diagnostic_contract_digest_sha256:$contract,trace_digest_sha256:$trace}' >"$attempt_dir/qualification.json"
	chmod 600 "$attempt_dir/qualification.json"
	late_attach_atomic_write "$state_path" "$(jq -c '.state="complete"' "$state_path")"
	late_attach_atomic_write "$attempt_dir/result.json" "$(jq -cn '{status:"complete",classification_category:"os_native_cold_delivers"}')"
	if [[ -n "${LATE_ATTACH_WORKER_EXIT_DELAY_SECONDS:-}" ]]; then
		sleep "$LATE_ATTACH_WORKER_EXIT_DELAY_SECONDS"
	fi
}

main
