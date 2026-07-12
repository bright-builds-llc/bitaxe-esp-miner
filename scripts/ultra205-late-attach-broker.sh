#!/usr/bin/env bash
# Private state broker for the Ultra 205 late-attach qualification.

late_attach_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
late_attach_repo_root="$(cd "$late_attach_script_dir/.." && pwd)"
# shellcheck source=scripts/process-group.sh
source "$late_attach_script_dir/process-group.sh"
# shellcheck source=scripts/serial-session-trace.sh
source "$late_attach_script_dir/serial-session-trace.sh"

readonly late_attach_expected_installed_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
readonly late_attach_detector_bin="${LATE_ATTACH_DETECTOR_BIN:-$late_attach_script_dir/detect-ultra205.sh}"
readonly late_attach_monitor_bin="${LATE_ATTACH_MONITOR_BIN:-$late_attach_script_dir/phase13-monitor-capture.sh}"
readonly late_attach_classifier_bin="$late_attach_script_dir/ultra205-late-attach-classifier.mjs"
readonly late_attach_worker_bin="$late_attach_script_dir/ultra205-late-attach-worker.sh"
readonly late_attach_qualification_bin="${LATE_ATTACH_QUALIFICATION_BIN:-$late_attach_script_dir/ultra205-transport-qualification.sh}"
readonly late_attach_frame_helper="$late_attach_script_dir/phase28.1.1-lifecycle-frame.pl"
readonly late_attach_private_root="${LATE_ATTACH_CONTROL_ROOT:-$late_attach_repo_root/hardware-runs/phase28.1.1/late-attach-control}"
readonly late_attach_attempts_root="$late_attach_private_root/attempts"
readonly late_attach_resume_root="$late_attach_private_root/resume-index"
readonly late_attach_trace_root="${LATE_ATTACH_TRACE_ROOT:-$late_attach_repo_root/hardware-runs/phase28.1.1/late-attach-private-traces}"
readonly late_attach_lock_dir="$late_attach_private_root/control.lock"
readonly late_attach_removal_timeout_ms=1800000
# shellcheck disable=SC2034 # Used by the separately executed lifecycle worker.
readonly late_attach_default_restore_timeout_ms=1800000

late_attach_lock_held=false

late_attach_die() {
	printf 'late_attach_error=%s\n' "$1" >&2
	exit 1
}

late_attach_usage() {
	printf 'usage: diagnose-ultra205-late-attach.sh begin expected-firmware-head=SHA [port=PATH] [capture-seconds=N]\n' >&2
	printf '       diagnose-ultra205-late-attach.sh status resume-handle=HEX\n' >&2
	printf '       diagnose-ultra205-late-attach.sh deliver resume-handle=HEX checkpoint-token=TOKEN response-token=TOKEN\n' >&2
}

late_attach_mode_of() {
	if stat -f '%Lp' "$1" >/dev/null 2>&1; then stat -f '%Lp' "$1"; else stat -c '%a' "$1"; fi
}

late_attach_owner_of() {
	if stat -f '%u' "$1" >/dev/null 2>&1; then stat -f '%u' "$1"; else stat -c '%u' "$1"; fi
}

late_attach_monotonic_ms() {
	if [[ -n "${LATE_ATTACH_MONOTONIC_MS_BIN:-}" ]]; then "$LATE_ATTACH_MONOTONIC_MS_BIN"; else serial_session_monotonic_ms; fi
}

late_attach_random_hex() {
	if [[ -n "${LATE_ATTACH_RANDOM_HEX_BIN:-}" ]]; then "$LATE_ATTACH_RANDOM_HEX_BIN" "$1"; else openssl rand -hex "$1"; fi
}

late_attach_sha256_text() { printf '%s' "$1" | shasum -a 256 | awk '{print $1}'; }

late_attach_atomic_write() {
	local destination="$1" contents="$2" temporary
	temporary="$(mktemp "$(dirname "$destination")/.late-attach-write.XXXXXX")"
	printf '%s\n' "$contents" >"$temporary"
	chmod 600 "$temporary"
	mv -f "$temporary" "$destination"
}

late_attach_ensure_roots() {
	mkdir -p "$late_attach_private_root" "$late_attach_attempts_root" "$late_attach_resume_root" "$late_attach_trace_root"
	chmod 700 "$late_attach_private_root" "$late_attach_attempts_root" "$late_attach_resume_root" "$late_attach_trace_root"
}

late_attach_acquire_lock() {
	local attempts=0 maybe_owner
	while ! mkdir -m 700 "$late_attach_lock_dir" 2>/dev/null; do
		maybe_owner="$(sed -n '1p' "$late_attach_lock_dir/owner.pid" 2>/dev/null)"
		if [[ "$maybe_owner" =~ ^[1-9][0-9]*$ ]] && ! kill -0 "$maybe_owner" 2>/dev/null; then
			rm -rf "$late_attach_lock_dir"
			continue
		fi
		attempts=$((attempts + 1))
		((attempts < 100)) || late_attach_die lock_failure
		sleep 0.01
	done
	printf '%s\n' "$$" >"$late_attach_lock_dir/owner.pid"
	late_attach_lock_held=true
}

late_attach_release_lock() {
	rm -f "$late_attach_lock_dir/owner.pid"
	rmdir "$late_attach_lock_dir" 2>/dev/null || late_attach_die lock_failure
	late_attach_lock_held=false
}

late_attach_process_fingerprint() {
	local started
	started="$(ps -o lstart= -p "$1" 2>/dev/null)"
	[[ -n "$started" ]] || return 1
	late_attach_sha256_text "pid-start-v1:$1:$started"
}

late_attach_require_clean_head() {
	if [[ "${LATE_ATTACH_TEST_MODE:-0}" != 1 && -n "$(git -C "$late_attach_repo_root" status --porcelain=v1)" ]]; then
		late_attach_die dirty_head
	fi
}

late_attach_validate_state() {
	jq -e '
      type == "object" and .schema_version == "ultra205-late-attach-attempt-v2" and
      (.attempt_id | test("^[0-9a-f]{32}$")) and
      (.resume_handle_sha256 | test("^[0-9a-f]{64}$")) and
      (.expected_firmware_head | test("^[0-9a-f]{40}$")) and
      (.tool_head | test("^[0-9a-f]{40}$")) and
      (.selected_port | type == "string") and (.capture_seconds | type == "number") and
      (.state | IN("preflight","waiting_removal","removal_observed","waiting_restore","capturing","complete","failed"))
    ' "$1" >/dev/null
}

late_attach_validate_slot() {
	jq -e --arg digest "$2" '
      type == "object" and .schema_version == "ultra205-late-attach-resume-v2" and
      .status == "active" and .resume_handle_sha256 == $digest and
      (.attempt_id | test("^[0-9a-f]{32}$")) and (.attempt_dir | type == "string")
    ' "$1" >/dev/null
}

late_attach_validate_tombstone() {
	jq -e --arg digest "$2" '
      type == "object" and
      (.schema_version | IN("ultra205-late-attach-tombstone-v1","ultra205-late-attach-tombstone-v2")) and
      .status == "closed" and .resume_handle_sha256 == $digest and (.cleanup_complete | type == "boolean")
    ' "$1" >/dev/null
}

late_attach_resolve_handle() {
	local handle="$1" digest slot attempt_dir state_path schema
	[[ "$handle" =~ ^[0-9a-f]{64}$ ]] || late_attach_die resume_handle_malformed
	digest="$(late_attach_sha256_text "$handle")"
	slot="$late_attach_resume_root/$digest.json"
	[[ -f "$slot" && ! -L "$slot" && "$(late_attach_mode_of "$slot")" == 600 && "$(late_attach_owner_of "$slot")" == "$(id -u)" ]] || late_attach_die resume_handle_wrong
	if late_attach_validate_tombstone "$slot" "$digest"; then late_attach_die resume_handle_stale; fi
	schema="$(jq -r '.schema_version // empty' "$slot")"
	[[ "$schema" != ultra205-late-attach-resume-v1 ]] || late_attach_die resume_handle_stale
	late_attach_validate_slot "$slot" "$digest" || late_attach_die resume_handle_ambiguous
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	[[ "$(dirname "$attempt_dir")" == "$late_attach_attempts_root" && -d "$attempt_dir" && ! -L "$attempt_dir" ]] || late_attach_die resume_handle_ambiguous
	[[ "$(late_attach_mode_of "$attempt_dir")" == 700 && "$(late_attach_owner_of "$attempt_dir")" == "$(id -u)" ]] || late_attach_die private_capability_invalid
	state_path="$attempt_dir/state.json"
	[[ -f "$state_path" && ! -L "$state_path" && "$(late_attach_mode_of "$state_path")" == 600 ]] || late_attach_die private_capability_invalid
	late_attach_validate_state "$state_path" || late_attach_die state_malformed
	printf '%s\n' "$slot"
}

late_attach_trace_digest() {
	find "$1" -type f ! -name qualification.json -print0 | sort -z | xargs -0 shasum -a 256 2>/dev/null | shasum -a 256 | awk '{print $1}'
}

late_attach_escrow_tombstone() {
	local slot="$1" terminal="$2" category="$3" cleanup="$4" escrow_attempt_dir digest attempt_id destination entry relative escrow_socket_path
	escrow_attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	digest="$(jq -er '.resume_handle_sha256' "$slot")"
	attempt_id="$(jq -er '.attempt_id' "$slot")"
	destination="$late_attach_trace_root/$attempt_id"
	mkdir -p "$destination"
	chmod 700 "$destination"
	while IFS= read -r -d '' entry; do
		relative="${entry#"$escrow_attempt_dir"/}"
		mkdir -p "$destination/$(dirname "$relative")"
		chmod 700 "$destination/$(dirname "$relative")"
		cp "$entry" "$destination/$relative"
		chmod 600 "$destination/$relative"
	done < <(find "$escrow_attempt_dir" -type f -print0)
	escrow_socket_path="$(jq -r '.socket_path // empty' "$escrow_attempt_dir/state.json")"
	[[ -z "$escrow_socket_path" ]] || rm -f "$escrow_socket_path"
	late_attach_atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg terminal "$terminal" --arg category "$category" --argjson cleanup "$cleanup" '{schema_version:"ultra205-late-attach-tombstone-v2",status:"closed",resume_handle_sha256:$digest,terminal_category:$terminal,classification_category:$category,cleanup_complete:$cleanup}')"
	rm -rf "$escrow_attempt_dir"
}

late_attach_failure_from_log() {
	local category
	category="$(sed -n 's/^failure_category=//p; s/^serial_session_failure_category=//p' "$1" 2>/dev/null | tail -1)"
	printf '%s\n' "${category:-command_failed}"
}

late_attach_run_capture() {
	local capture_attempt_dir="$1" port="$2" reader="$3" seconds="$4" label="$5" capture_status
	set +e
	SERIAL_SESSION_TRACE_ROOT="$capture_attempt_dir/session-traces" "$late_attach_monitor_bin" \
		--port "$port" --out "$capture_attempt_dir/${label}.wrapper.log" --raw-out "$capture_attempt_dir/${label}.raw.log" \
		--reader "$reader" --seconds "$seconds" --no-reset
	capture_status=$?
	set -e
	if ! chmod 600 "$capture_attempt_dir/${label}.wrapper.log" "$capture_attempt_dir/${label}.raw.log"; then
		return 1
	fi
	return "$capture_status"
}

late_attach_close_failure() {
	local slot="$1" category="$2" cleanup="${3:-true}"
	late_attach_escrow_tombstone "$slot" "$category" not_classified "$cleanup"
	[[ "$late_attach_lock_held" != true ]] || late_attach_release_lock
	printf 'diagnostic_status=failed\nfailure_category=%s\n' "$category"
	return 1
}

late_attach_wait_for_file() {
	local path="$1" pid="$2" attempts="${3:-500}"
	for _ in $(seq 1 "$attempts"); do
		[[ -s "$path" ]] && return 0
		kill -0 "$pid" 2>/dev/null || return 1
		sleep 0.01
	done
	return 1
}

late_attach_wait_for_result_or_tombstone() {
	local result_path="$1" slot="$2" owner="$3" attempts="$4"
	for _ in $(seq 1 "$attempts"); do
		if [[ -s "$result_path" ]] && jq -e '.status | IN("complete","failed")' "$result_path" >/dev/null 2>&1; then return 0; fi
		if jq -e '.status == "closed"' "$slot" >/dev/null 2>&1; then return 2; fi
		kill -0 "$owner" 2>/dev/null || return 1
		sleep 0.01
	done
	return 1
}

late_attach_wait_for_owner_cleanup() {
	local owner="$1"
	for _ in $(seq 1 200); do
		if ! kill -0 "$owner" 2>/dev/null && ! phase_process_group_is_alive "$owner"; then return 0; fi
		sleep 0.01
	done
	return 1
}

late_attach_emit_public_state() {
	local state_path="$1" action_path result_path
	action_path="$(dirname "$state_path")/action.json"
	result_path="$(dirname "$state_path")/result.json"
	if [[ -f "$result_path" ]]; then
		jq -r 'to_entries[] | "\(.key)=\(.value)"' "$result_path"
		return
	fi
	[[ -f "$action_path" ]] || late_attach_die action_not_ready
	jq -r '.lines[]' "$action_path"
}

late_attach_begin() {
	local expected_head="" requested_port="" capture_seconds=60 arg
	for arg in "$@"; do
		case "$arg" in expected-firmware-head=*) expected_head="${arg#*=}" ;; port=*) requested_port="${arg#*=}" ;; capture-seconds=*) capture_seconds="${arg#*=}" ;; *) late_attach_die unknown_argument ;; esac
	done
	[[ "$expected_head" == "$late_attach_expected_installed_head" ]] || late_attach_die expected_firmware_head_mismatch
	if [[ ! "$capture_seconds" =~ ^[0-9]+$ ]] || ((capture_seconds < 3 || capture_seconds > 300)); then
		late_attach_die capture_seconds_invalid
	fi
	[[ -z "$requested_port" || "$requested_port" == /* || "$requested_port" =~ ^COM[0-9]+$ ]] || late_attach_die port_invalid
	late_attach_ensure_roots
	late_attach_require_clean_head
	late_attach_acquire_lock
	if find "$late_attach_resume_root" -maxdepth 1 -type f -name '*.json' -exec jq -e '.status == "active"' {} \; 2>/dev/null | grep -q true; then
		late_attach_release_lock
		late_attach_die active_attempt_exists
	fi
	local handle attempt_id digest attempt_dir state_path slot now detector_log detector_status port node_identity usb_identity preflight_seconds preflight_path worker_pid fingerprint socket_path capability ready_file action_path
	handle="$(late_attach_random_hex 32)"
	attempt_id="$(late_attach_random_hex 16)"
	[[ "$handle" =~ ^[0-9a-f]{64}$ && "$attempt_id" =~ ^[0-9a-f]{32}$ ]] || late_attach_die randomness_unavailable
	digest="$(late_attach_sha256_text "$handle")"
	attempt_dir="$late_attach_attempts_root/$attempt_id"
	mkdir -m 700 "$attempt_dir"
	state_path="$attempt_dir/state.json"
	now="$(late_attach_monotonic_ms)"
	late_attach_atomic_write "$state_path" "$(jq -cn --arg attempt "$attempt_id" --arg digest "$digest" --arg expected "$expected_head" --arg toolHead "$(git -C "$late_attach_repo_root" rev-parse HEAD)" --argjson created "$now" --argjson capture "$capture_seconds" '{schema_version:"ultra205-late-attach-attempt-v2",attempt_id:$attempt,resume_handle_sha256:$digest,expected_firmware_head:$expected,tool_head:$toolHead,created_ms:$created,capture_seconds:$capture,state:"preflight",selected_port:"",selected_node_identity:"",selected_usb_identity:"",preflight_espflash_heartbeat_count:0,preflight_os_native_heartbeat_count:0,preflight_same_session:false,checkpoint_deadline_ms:0,owner_pid:null,owner_fingerprint_sha256:null,socket_path:null,lifecycle_capability_sha256:null,action_published_ms:null,removal_observed_ms:null,watcher_armed_ms:null,watcher_deadline_ms:null,usb_appeared_ms:null,monitor_attached_ms:null}')"
	slot="$late_attach_resume_root/$digest.json"
	late_attach_atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg attempt "$attempt_id" --arg dir "$attempt_dir" '{schema_version:"ultra205-late-attach-resume-v2",status:"active",resume_handle_sha256:$digest,attempt_id:$attempt,attempt_dir:$dir}')"
	late_attach_release_lock
	printf 'resume_handle=%s\n' "$handle"
	detector_log="$attempt_dir/detector.log"
	set +e
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/detector-traces" "$late_attach_detector_bin" >"$detector_log" 2>&1
	detector_status=$?
	set -e
	chmod 600 "$detector_log"
	if ((detector_status != 0)); then
		late_attach_acquire_lock
		late_attach_close_failure "$slot" "detector_$(late_attach_failure_from_log "$detector_log")"
		return
	fi
	port="$(sed -n 's/^port=//p' "$detector_log")"
	[[ -n "$port" && "$(printf '%s\n' "$port" | wc -l | tr -d ' ')" == 1 ]] || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" detector_port_contract
		return
	}
	[[ -z "$requested_port" || "$requested_port" == "$port" ]] || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" requested_port_mismatch
		return
	}
	# shellcheck disable=SC2034 # Read by sourced serial-session helpers.
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/session-traces"
	serial_session_trace_init late-attach-connected-readiness
	serial_session_readiness_gate connected_preflight "$port" || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" "preflight_${SERIAL_SESSION_READINESS_CATEGORY}"
		return
	}
	node_identity="$(serial_session_node_identity "$port")" || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" preflight_identity_unavailable
		return
	}
	usb_identity="$(serial_session_usb_identity "$port")" || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" preflight_identity_unavailable
		return
	}
	preflight_seconds="${LATE_ATTACH_PREFLIGHT_SECONDS:-15}"
	late_attach_run_capture "$attempt_dir" "$port" espflash "$preflight_seconds" preflight-espflash || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" preflight_espflash_capture_failed
		return
	}
	late_attach_run_capture "$attempt_dir" "$port" os-native "$preflight_seconds" preflight-os-native || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" preflight_os_native_capture_failed
		return
	}
	preflight_path="$attempt_dir/preflight.json"
	set +e
	node "$late_attach_classifier_bin" preflight "$attempt_dir/preflight-espflash.raw.log" "$attempt_dir/preflight-os-native.raw.log" >"$preflight_path"
	detector_status=$?
	set -e
	chmod 600 "$preflight_path"
	if ((detector_status != 0)); then
		late_attach_acquire_lock
		late_attach_close_failure "$slot" preflight_heartbeat_validation_failed
		return
	fi
	serial_session_readiness_gate pre_removal "$port" || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" pre_removal_node_loss
		return
	}
	[[ "$(serial_session_node_identity "$port")" == "$node_identity" && "$(serial_session_usb_identity "$port")" == "$usb_identity" ]] || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" pre_removal_identity_changed
		return
	}
	now="$(late_attach_monotonic_ms)"
	capability="$(late_attach_random_hex 32)"
	socket_path="${LATE_ATTACH_SOCKET_ROOT:-/tmp}/ultra205-la-${attempt_id}.sock"
	ready_file="$attempt_dir/worker.ready"
	action_path="$attempt_dir/action.json"
	late_attach_atomic_write "$state_path" "$(jq -c --arg port "$port" --arg node "$node_identity" --arg usb "$usb_identity" --arg capability "$(late_attach_sha256_text "$capability")" --arg socket "$socket_path" --argjson deadline "$((now + late_attach_removal_timeout_ms))" --slurpfile preflight "$preflight_path" '.state="waiting_removal" | .selected_port=$port | .selected_node_identity=$node | .selected_usb_identity=$usb | .checkpoint_deadline_ms=$deadline | .socket_path=$socket | .lifecycle_capability_sha256=$capability | .preflight_espflash_heartbeat_count=$preflight[0].espflashHeartbeatCount | .preflight_os_native_heartbeat_count=$preflight[0].osNativeHeartbeatCount | .preflight_same_session=$preflight[0].sameSession' "$state_path")"
	LATE_ATTACH_LIFECYCLE_CAPABILITY="$capability" phase_process_group_start "$ready_file" "$late_attach_worker_bin" "$attempt_dir" "$socket_path" >"$attempt_dir/worker.stdout" 2>"$attempt_dir/worker.stderr" || {
		late_attach_acquire_lock
		late_attach_close_failure "$slot" worker_start_failed
		return
	}
	worker_pid="$PHASE_PROCESS_GROUP_PID"
	fingerprint="$(late_attach_process_fingerprint "$worker_pid")" || late_attach_die worker_start_failed
	late_attach_atomic_write "$state_path" "$(jq -c --argjson pid "$worker_pid" --arg fingerprint "$fingerprint" '.owner_pid=$pid | .owner_fingerprint_sha256=$fingerprint' "$state_path")"
	late_attach_wait_for_file "$action_path" "$worker_pid" || {
		local cleanup_complete=true
		if ! phase_process_group_terminate "$worker_pid" late-attach-worker >/dev/null 2>&1; then
			cleanup_complete=false
		fi
		late_attach_acquire_lock
		late_attach_close_failure "$slot" removal_watcher_arming_failed "$cleanup_complete"
		return
	}
	printf 'resume_handle=%s\n' "$handle"
	late_attach_emit_public_state "$state_path"
}

late_attach_parse_handle_only() {
	local handle="" arg
	for arg in "$@"; do case "$arg" in resume-handle=*) handle="${arg#*=}" ;; *) late_attach_die unknown_argument ;; esac done
	[[ -n "$handle" ]] || late_attach_die resume_handle_missing
	printf '%s\n' "$handle"
}

late_attach_status() {
	local handle slot state_path digest
	handle="$(late_attach_parse_handle_only "$@")"
	late_attach_ensure_roots
	late_attach_require_clean_head
	late_attach_acquire_lock
	digest="$(late_attach_sha256_text "$handle")"
	slot="$late_attach_resume_root/$digest.json"
	if [[ -f "$slot" ]] && late_attach_validate_tombstone "$slot" "$digest"; then
		printf 'resume_handle=%s\n' "$handle"
		jq -r 'to_entries[] | "\(.key)=\(.value)"' "$slot"
		late_attach_release_lock
		return
	fi
	slot="$(late_attach_resolve_handle "$handle")"
	state_path="$(jq -er '.attempt_dir' "$slot")/state.json"
	[[ "$(git -C "$late_attach_repo_root" rev-parse HEAD)" == "$(jq -er '.tool_head' "$state_path")" ]] || late_attach_die exact_head_mismatch
	local maybe_owner cleanup_complete=true terminal_category=owner_process_stale result_path
	maybe_owner="$(jq -r '.owner_pid // empty' "$state_path")"
	if [[ "$maybe_owner" =~ ^[1-9][0-9]*$ ]] && ! kill -0 "$maybe_owner" 2>/dev/null; then
		if phase_process_group_is_alive "$maybe_owner" && ! phase_process_group_terminate "$maybe_owner" late-attach-stale-owner >/dev/null 2>&1; then
			cleanup_complete=false
		fi
		result_path="$(dirname "$state_path")/result.json"
		if [[ -f "$result_path" ]] && jq -e '.status == "failed" and (.failure_category | type == "string")' "$result_path" >/dev/null 2>&1; then
			terminal_category="$(jq -er '.failure_category' "$result_path")"
		fi
		late_attach_escrow_tombstone "$slot" "$terminal_category" not_classified "$cleanup_complete"
		printf 'resume_handle=%s\n' "$handle"
		jq -r 'to_entries[] | "\(.key)=\(.value)"' "$slot"
		late_attach_release_lock
		return
	fi
	printf 'resume_handle=%s\n' "$handle"
	late_attach_emit_public_state "$state_path"
	late_attach_release_lock
}

late_attach_deliver() {
	local handle="" checkpoint="" response="" arg slot attempt_dir attempt_id state_path owner socket_path capability frame now result_path qualification_path
	for arg in "$@"; do case "$arg" in resume-handle=*) handle="${arg#*=}" ;; checkpoint-token=*) checkpoint="${arg#*=}" ;; response-token=*) response="${arg#*=}" ;; *) late_attach_die unknown_argument ;; esac done
	late_attach_ensure_roots
	late_attach_require_clean_head
	late_attach_acquire_lock
	slot="$(late_attach_resolve_handle "$handle")"
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	state_path="$attempt_dir/state.json"
	[[ "$(git -C "$late_attach_repo_root" rev-parse HEAD)" == "$(jq -er '.tool_head' "$state_path")" ]] || late_attach_die exact_head_mismatch
	[[ "$checkpoint" == late-attach-removal-watcher-armed-v2 && "$response" == late-attach-both-power-paths-removed-v2 ]] || late_attach_die checkpoint_token_mismatch
	[[ "$(jq -r '.state' "$state_path")" == removal_observed ]] || late_attach_die removal_not_observed
	now="$(late_attach_monotonic_ms)"
	((now < $(jq -er '.checkpoint_deadline_ms' "$state_path"))) || late_attach_die checkpoint_expired
	owner="$(jq -er '.owner_pid' "$state_path")"
	[[ "$owner" =~ ^[1-9][0-9]*$ && -n "$(kill -0 "$owner" 2>/dev/null && printf alive)" ]] || late_attach_die owner_process_stale
	[[ "$(late_attach_process_fingerprint "$owner")" == "$(jq -er '.owner_fingerprint_sha256' "$state_path")" ]] || late_attach_die owner_process_reused
	socket_path="$(jq -er '.socket_path' "$state_path")"
	[[ -S "$socket_path" && "$(late_attach_mode_of "$socket_path")" == 600 ]] || late_attach_die lifecycle_socket_unavailable
	capability="${LATE_ATTACH_LIFECYCLE_CAPABILITY_FOR_TEST:-}"
	[[ -n "$capability" ]] || capability="$(sed -n 's/^capability=//p' "$attempt_dir/capability.escrow")"
	[[ "$(late_attach_sha256_text "$capability")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] || late_attach_die private_capability_invalid
	frame="$(jq -cn --arg digest "$(late_attach_sha256_text "$handle")" --arg token "$checkpoint" --arg response "$response" --arg capability "$capability" --argjson owner "$owner" '{resume_handle_sha256:$digest,checkpoint_token:$token,response_token:$response,lifecycle_capability:$capability,owner_pid:$owner}')"
	late_attach_release_lock
	printf '%s' "$frame" | perl "$late_attach_frame_helper" send --socket "$socket_path"
	printf 'checkpoint_delivery=accepted\n'
	result_path="$attempt_dir/result.json"
	set +e
	late_attach_wait_for_result_or_tombstone "$result_path" "$slot" "$owner" "${LATE_ATTACH_RESULT_WAIT_SAMPLES:-200000}"
	local wait_status=$?
	set -e
	if ((wait_status == 2)); then
		jq -r 'to_entries[] | "\(.key)=\(.value)"' "$slot"
		return 1
	fi
	((wait_status == 0)) || {
		late_attach_acquire_lock
		slot="$(late_attach_resolve_handle "$handle")"
		late_attach_close_failure "$slot" owner_process_stale false
		return
	}
	late_attach_wait_for_owner_cleanup "$owner" || {
		late_attach_acquire_lock
		slot="$(late_attach_resolve_handle "$handle")"
		late_attach_close_failure "$slot" cleanup_process_survived false
		return
	}
	late_attach_acquire_lock
	slot="$(late_attach_resolve_handle "$handle")"
	if [[ "$(jq -r '.status' "$result_path")" != complete ]]; then
		local failure
		failure="$(jq -r '.failure_category // "worker_failed"' "$result_path")"
		late_attach_close_failure "$slot" "$failure"
		return
	fi
	qualification_path="$attempt_dir/qualification.json"
	[[ -f "$qualification_path" && ! -L "$qualification_path" && "$(late_attach_mode_of "$qualification_path")" == 600 ]] || {
		late_attach_close_failure "$slot" qualification_invalid
		return
	}
	late_attach_atomic_write "$qualification_path" "$(jq -c '.cleanup_complete=true' "$qualification_path")"
	"$late_attach_qualification_bin" validate "$qualification_path" "$(git -C "$late_attach_repo_root" rev-parse HEAD)" >/dev/null 2>&1 || {
		late_attach_close_failure "$slot" qualification_invalid
		return
	}
	attempt_id="$(jq -er '.attempt_id' "$state_path")"
	late_attach_escrow_tombstone "$slot" diagnostic_complete os_native_cold_delivers true
	late_attach_release_lock
	jq -r 'to_entries[] | "\(.key)=\(.value)"' "$late_attach_trace_root/$attempt_id/qualification.json"
}

late_attach_cleanup_exit() {
	local status=$?
	if [[ "$late_attach_lock_held" == true ]]; then
		rm -f "$late_attach_lock_dir/owner.pid"
		rmdir "$late_attach_lock_dir" 2>/dev/null || true
	fi
	exit "$status"
}

late_attach_install_traps() { trap late_attach_cleanup_exit EXIT INT TERM; }

late_attach_validate_test_overrides() {
	[[ "${LATE_ATTACH_TEST_MODE:-0}" == 1 ]] && return
	local variable
	for variable in LATE_ATTACH_ABSENCE_INTERVAL_SECONDS LATE_ATTACH_ABSENCE_SAMPLES LATE_ATTACH_PREFLIGHT_SECONDS LATE_ATTACH_RESTORE_TIMEOUT_MS LATE_ATTACH_SOAK_INTERVAL_SECONDS LATE_ATTACH_SOAK_SAMPLES LATE_ATTACH_RESULT_WAIT_SAMPLES LATE_ATTACH_WORKER_EXIT_DELAY_SECONDS; do
		[[ -z "${!variable:-}" ]] || late_attach_die test_configuration_invalid
	done
}
