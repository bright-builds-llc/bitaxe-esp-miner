#!/usr/bin/env bash
# One-shot, no-reset A-B-A diagnosis for ESP32-S3 native-USB late attachment.
set -euo pipefail
umask 077

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
# shellcheck source=scripts/process-group.sh
source "$script_dir/process-group.sh"
# shellcheck source=scripts/serial-session-trace.sh
source "$script_dir/serial-session-trace.sh"

readonly expected_installed_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
readonly detector_bin="${LATE_ATTACH_DETECTOR_BIN:-$script_dir/detect-ultra205.sh}"
readonly monitor_bin="${LATE_ATTACH_MONITOR_BIN:-$script_dir/phase13-monitor-capture.sh}"
readonly classifier_bin="$script_dir/ultra205-late-attach-classifier.mjs"
readonly frame_helper="$script_dir/phase28.1.1-lifecycle-frame.pl"
readonly private_root="${LATE_ATTACH_CONTROL_ROOT:-$repo_root/hardware-runs/phase28.1.1/late-attach-control}"
readonly attempts_root="$private_root/attempts"
readonly resume_root="$private_root/resume-index"
readonly trace_root="${LATE_ATTACH_TRACE_ROOT:-$repo_root/hardware-runs/phase28.1.1/late-attach-private-traces}"
readonly lock_dir="$private_root/control.lock"
readonly removal_timeout_ms=1800000
readonly default_restore_timeout_ms=1800000

lock_held=false
active_worker_pid=""

die() {
	printf 'late_attach_error=%s\n' "$1" >&2
	exit 1
}

usage() {
	printf 'usage: %s begin expected-firmware-head=SHA [port=PATH] [capture-seconds=N]\n' "$(basename "$0")" >&2
	printf '       %s deliver resume-handle=HEX checkpoint-token=TOKEN response-token=TOKEN\n' "$(basename "$0")" >&2
}

mode_of() {
	if stat -f '%Lp' "$1" >/dev/null 2>&1; then
		stat -f '%Lp' "$1"
		return
	fi
	stat -c '%a' "$1"
}

owner_of() {
	if stat -f '%u' "$1" >/dev/null 2>&1; then
		stat -f '%u' "$1"
		return
	fi
	stat -c '%u' "$1"
}

monotonic_ms() {
	if [[ -n "${LATE_ATTACH_MONOTONIC_MS_BIN:-}" ]]; then
		"$LATE_ATTACH_MONOTONIC_MS_BIN"
		return
	fi
	serial_session_monotonic_ms
}

random_hex() {
	local bytes="$1"
	if [[ -n "${LATE_ATTACH_RANDOM_HEX_BIN:-}" ]]; then
		"$LATE_ATTACH_RANDOM_HEX_BIN" "$bytes"
		return
	fi
	openssl rand -hex "$bytes"
}

sha256_text() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

atomic_write() {
	local destination="$1"
	local contents="$2"
	local temporary
	temporary="$(mktemp "$(dirname "$destination")/.late-attach-write.XXXXXX")"
	printf '%s\n' "$contents" >"$temporary"
	chmod 600 "$temporary"
	mv -f "$temporary" "$destination"
}

ensure_private_roots() {
	mkdir -p "$private_root" "$attempts_root" "$resume_root" "$trace_root"
	chmod 700 "$private_root" "$attempts_root" "$resume_root" "$trace_root"
}

acquire_lock() {
	local attempts=0
	while ! mkdir -m 700 "$lock_dir" 2>/dev/null; do
		if [[ -s "$lock_dir/owner.pid" ]]; then
			local owner_pid
			owner_pid="$(sed -n '1p' "$lock_dir/owner.pid")"
			if [[ "$owner_pid" =~ ^[1-9][0-9]*$ ]] && ! kill -0 "$owner_pid" 2>/dev/null; then
				rm -rf "$lock_dir"
				continue
			fi
		fi
		attempts=$((attempts + 1))
		((attempts < 100)) || die lock_failure
		sleep 0.01
	done
	printf '%s\n' "$$" >"$lock_dir/owner.pid"
	lock_held=true
}

release_lock() {
	rm -f "$lock_dir/owner.pid"
	rmdir "$lock_dir" 2>/dev/null || die lock_failure
	lock_held=false
}

process_fingerprint() {
	local pid="$1"
	local started
	started="$(ps -o lstart= -p "$pid" 2>/dev/null)"
	[[ -n "$started" ]] || return 1
	sha256_text "pid-start-v1:${pid}:${started}"
}

require_clean_head() {
	if [[ "${LATE_ATTACH_TEST_MODE:-0}" != "1" && -n "$(git -C "$repo_root" status --porcelain=v1)" ]]; then
		die dirty_head
	fi
}

validate_state() {
	local state_path="$1"
	jq -e '
      type == "object" and
      (keys | sort) == (["attempt_id","capture_seconds","checkpoint_deadline_ms","created_ms","expected_firmware_head","lifecycle_capability_sha256","owner_fingerprint_sha256","owner_pid","preflight_espflash_heartbeat_count","preflight_os_native_heartbeat_count","preflight_same_session","resume_handle_sha256","schema_version","selected_node_identity","selected_port","selected_usb_identity","socket_path","state","tool_head","watcher_armed_ms","watcher_deadline_ms"] | sort) and
      .schema_version == "ultra205-late-attach-attempt-v1" and
      (.attempt_id | test("^[0-9a-f]{32}$")) and
      (.resume_handle_sha256 | test("^[0-9a-f]{64}$")) and
      (.expected_firmware_head | test("^[0-9a-f]{40}$")) and
      (.tool_head | test("^[0-9a-f]{40}$")) and
      (.selected_port | type == "string") and
      (.capture_seconds | type == "number") and
      (.state | IN("preflight","waiting_removal","running","watcher_armed")) and
      ([.owner_pid,.owner_fingerprint_sha256,.socket_path,.lifecycle_capability_sha256,.watcher_armed_ms,.watcher_deadline_ms] | all(. == null or (type == "number") or (type == "string")))
    ' "$state_path" >/dev/null
}

validate_slot() {
	local slot="$1"
	local digest="$2"
	jq -e --arg digest "$digest" '
      type == "object" and
      (keys | sort) == (["attempt_dir","attempt_id","resume_handle_sha256","schema_version","status"] | sort) and
      .schema_version == "ultra205-late-attach-resume-v1" and
      .status == "active" and
      .resume_handle_sha256 == $digest and
      (.attempt_id | test("^[0-9a-f]{32}$")) and
      (.attempt_dir | type == "string")
    ' "$slot" >/dev/null
}

validate_tombstone() {
	local slot="$1"
	local digest="$2"
	jq -e --arg digest "$digest" '
      type == "object" and
      (keys | sort) == (["classification_category","cleanup_complete","resume_handle_sha256","schema_version","status","terminal_category"] | sort) and
      .schema_version == "ultra205-late-attach-tombstone-v1" and
      .status == "closed" and
      .resume_handle_sha256 == $digest and
      (.cleanup_complete | type == "boolean")
    ' "$slot" >/dev/null
}

resolve_handle() {
	local handle="$1"
	[[ "$handle" =~ ^[0-9a-f]{64}$ ]] || die resume_handle_malformed
	local digest
	local slot
	digest="$(sha256_text "$handle")"
	slot="$resume_root/$digest.json"
	[[ -f "$slot" && ! -L "$slot" && "$(mode_of "$slot")" == 600 && "$(owner_of "$slot")" == "$(id -u)" ]] || die resume_handle_wrong
	if validate_tombstone "$slot" "$digest"; then
		die resume_handle_stale
	fi
	validate_slot "$slot" "$digest" || die resume_handle_ambiguous
	local attempt_dir
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	[[ "$(dirname "$attempt_dir")" == "$attempts_root" && -d "$attempt_dir" && ! -L "$attempt_dir" ]] || die resume_handle_ambiguous
	[[ "$(mode_of "$attempt_dir")" == 700 && "$(owner_of "$attempt_dir")" == "$(id -u)" ]] || die private_capability_invalid
	local state_path="$attempt_dir/state.json"
	[[ -f "$state_path" && ! -L "$state_path" && "$(mode_of "$state_path")" == 600 ]] || die private_capability_invalid
	validate_state "$state_path" || die state_malformed
	[[ "$(jq -er '.attempt_id' "$state_path")" == "$(jq -er '.attempt_id' "$slot")" ]] || die resume_handle_ambiguous
	printf '%s\n' "$slot"
}

trace_digest() {
	local attempt_dir="$1"
	find "$attempt_dir" -type f ! -name summary.json -print0 |
		sort -z |
		xargs -0 shasum -a 256 2>/dev/null |
		shasum -a 256 |
		awk '{print $1}'
}

escrow_and_tombstone() {
	local slot="$1"
	local terminal_category="$2"
	local classification_category="$3"
	local cleanup_complete="$4"
	local attempt_dir
	local digest
	local attempt_id
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	digest="$(jq -er '.resume_handle_sha256' "$slot")"
	attempt_id="$(jq -er '.attempt_id' "$slot")"
	local destination="$trace_root/$attempt_id"
	if [[ ! -d "$destination" ]]; then
		mkdir -m 700 "$destination"
	fi
	local entry
	while IFS= read -r -d '' entry; do
		local relative="${entry#"$attempt_dir"/}"
		mkdir -p "$destination/$(dirname "$relative")"
		chmod 700 "$destination/$(dirname "$relative")"
		cp "$entry" "$destination/$relative"
		chmod 600 "$destination/$relative"
	done < <(find "$attempt_dir" -type f -print0)
	local socket_path
	socket_path="$(jq -r '.socket_path // empty' "$attempt_dir/state.json")"
	[[ -z "$socket_path" ]] || rm -f "$socket_path"
	atomic_write "$slot" "$(jq -cn \
		--arg digest "$digest" \
		--arg terminal "$terminal_category" \
		--arg classification "$classification_category" \
		--argjson cleanup "$cleanup_complete" \
		'{schema_version:"ultra205-late-attach-tombstone-v1",status:"closed",resume_handle_sha256:$digest,terminal_category:$terminal,classification_category:$classification,cleanup_complete:$cleanup}')"
	rm -rf "$attempt_dir"
}

failure_category_from_log() {
	local log_path="$1"
	local category
	category="$(sed -n 's/^failure_category=//p; s/^serial_session_failure_category=//p' "$log_path" 2>/dev/null | tail -1)"
	printf '%s\n' "${category:-command_failed}"
}

run_capture() {
	local attempt_dir="$1"
	local port="$2"
	local reader="$3"
	local seconds="$4"
	local label="$5"
	local wrapper_log="$attempt_dir/${label}.wrapper.log"
	local raw_log="$attempt_dir/${label}.raw.log"
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/session-traces" \
		"$monitor_bin" --port "$port" --out "$wrapper_log" --raw-out "$raw_log" --reader "$reader" --seconds "$seconds" --no-reset
	chmod 600 "$wrapper_log" "$raw_log"
}

terminal_failure() {
	local slot="$1"
	local category="$2"
	local cleanup_complete="${3:-true}"
	escrow_and_tombstone "$slot" "$category" not_classified "$cleanup_complete"
	if [[ "$lock_held" == true ]]; then
		release_lock
	fi
	printf 'diagnostic_status=failed\n'
	printf 'failure_category=%s\n' "$category"
	return 1
}

begin_diagnostic() {
	local expected_head=""
	local requested_port=""
	local capture_seconds=30
	while (($#)); do
		case "$1" in
		expected-firmware-head=*) expected_head="${1#*=}" ;;
		port=*) requested_port="${1#*=}" ;;
		capture-seconds=*) capture_seconds="${1#*=}" ;;
		*) die unknown_argument ;;
		esac
		shift
	done
	[[ "$expected_head" == "$expected_installed_head" ]] || die expected_firmware_head_mismatch
	if [[ ! "$capture_seconds" =~ ^[0-9]+$ ]] || ((capture_seconds < 1 || capture_seconds > 300)); then
		die capture_seconds_invalid
	fi
	[[ -z "$requested_port" || "$requested_port" == /* || "$requested_port" =~ ^COM[0-9]+$ ]] || die port_invalid
	ensure_private_roots
	require_clean_head
	acquire_lock
	if find "$resume_root" -type f -maxdepth 1 -name '*.json' -exec jq -e '.status == "active"' {} \; 2>/dev/null | grep -q true; then
		release_lock
		die active_attempt_exists
	fi
	local handle
	local digest
	local attempt_id
	local attempt_dir
	local slot
	local state_path
	handle="$(random_hex 32)"
	attempt_id="$(random_hex 16)"
	[[ "$handle" =~ ^[0-9a-f]{64}$ && "$attempt_id" =~ ^[0-9a-f]{32}$ ]] || die randomness_unavailable
	digest="$(sha256_text "$handle")"
	attempt_dir="$attempts_root/$attempt_id"
	mkdir -m 700 "$attempt_dir"
	state_path="$attempt_dir/state.json"
	local now
	now="$(monotonic_ms)"
	atomic_write "$state_path" "$(jq -cn \
		--arg attempt "$attempt_id" --arg digest "$digest" --arg expected "$expected_head" \
		--arg toolHead "$(git -C "$repo_root" rev-parse HEAD)" --argjson created "$now" \
		--argjson capture "$capture_seconds" \
		'{schema_version:"ultra205-late-attach-attempt-v1",attempt_id:$attempt,resume_handle_sha256:$digest,expected_firmware_head:$expected,tool_head:$toolHead,created_ms:$created,capture_seconds:$capture,state:"preflight",selected_port:"",selected_node_identity:"",selected_usb_identity:"",preflight_espflash_heartbeat_count:0,preflight_os_native_heartbeat_count:0,preflight_same_session:false,checkpoint_deadline_ms:0,owner_pid:null,owner_fingerprint_sha256:null,socket_path:null,lifecycle_capability_sha256:null,watcher_armed_ms:null,watcher_deadline_ms:null}')"
	slot="$resume_root/$digest.json"
	atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg attempt "$attempt_id" --arg dir "$attempt_dir" '{schema_version:"ultra205-late-attach-resume-v1",status:"active",resume_handle_sha256:$digest,attempt_id:$attempt,attempt_dir:$dir}')"
	release_lock
	printf 'resume_handle=%s\n' "$handle"

	local detector_log="$attempt_dir/detector.log"
	set +e
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/detector-traces" "$detector_bin" >"$detector_log" 2>&1
	local detector_status=$?
	set -e
	chmod 600 "$detector_log"
	if ((detector_status != 0)); then
		terminal_failure "$slot" "detector_$(failure_category_from_log "$detector_log")"
		return
	fi
	local -a ports=()
	while IFS= read -r candidate; do ports+=("$candidate"); done < <(sed -n 's/^port=//p' "$detector_log")
	if ((${#ports[@]} != 1)); then
		terminal_failure "$slot" detector_port_contract
		return
	fi
	local port="${ports[0]}"
	if [[ -n "$requested_port" && "$requested_port" != "$port" ]]; then
		terminal_failure "$slot" requested_port_mismatch
		return
	fi
	# shellcheck disable=SC2034 # Read by sourced serial-session helpers.
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/session-traces"
	serial_session_trace_init late-attach-connected-readiness
	if ! serial_session_readiness_gate connected_preflight "$port"; then
		terminal_failure "$slot" "preflight_${SERIAL_SESSION_READINESS_CATEGORY}"
		return
	fi
	local node_identity
	local usb_identity
	node_identity="$(serial_session_node_identity "$port")" || {
		terminal_failure "$slot" preflight_identity_unavailable
		return
	}
	usb_identity="$(serial_session_usb_identity "$port")" || {
		terminal_failure "$slot" preflight_identity_unavailable
		return
	}
	local preflight_seconds="${LATE_ATTACH_PREFLIGHT_SECONDS:-15}"
	if [[ ! "$preflight_seconds" =~ ^[0-9]+$ ]] || ((preflight_seconds < 1 || preflight_seconds > 30)); then
		die test_configuration_invalid
	fi
	if ! run_capture "$attempt_dir" "$port" espflash "$preflight_seconds" preflight-espflash; then
		terminal_failure "$slot" "preflight_espflash_$(failure_category_from_log "$attempt_dir/preflight-espflash.wrapper.log")"
		return
	fi
	if ! run_capture "$attempt_dir" "$port" os-native "$preflight_seconds" preflight-os-native; then
		terminal_failure "$slot" "preflight_os_native_$(failure_category_from_log "$attempt_dir/preflight-os-native.wrapper.log")"
		return
	fi
	local preflight_path="$attempt_dir/preflight.json"
	set +e
	node "$classifier_bin" preflight "$attempt_dir/preflight-espflash.raw.log" "$attempt_dir/preflight-os-native.raw.log" >"$preflight_path"
	local preflight_status=$?
	set -e
	chmod 600 "$preflight_path"
	if ((preflight_status != 0)) || ! jq -e '
      .passed == true and
      .sameSession == true and
      ((.espflashHeartbeatCount | type) == "number" and .espflashHeartbeatCount > 0) and
      ((.osNativeHeartbeatCount | type) == "number" and .osNativeHeartbeatCount > 0)
    ' "$preflight_path" >/dev/null; then
		terminal_failure "$slot" preflight_heartbeat_validation_failed
		return
	fi
	now="$(monotonic_ms)"
	atomic_write "$state_path" "$(jq -c \
		--arg port "$port" --arg node "$node_identity" --arg usb "$usb_identity" \
		--argjson deadline "$((now + removal_timeout_ms))" \
		--slurpfile preflight "$preflight_path" \
		'.state="waiting_removal" | .selected_port=$port | .selected_node_identity=$node | .selected_usb_identity=$usb | .checkpoint_deadline_ms=$deadline | .preflight_espflash_heartbeat_count=$preflight[0].espflashHeartbeatCount | .preflight_os_native_heartbeat_count=$preflight[0].osNativeHeartbeatCount | .preflight_same_session=$preflight[0].sameSession' "$state_path")"
	printf '## CHECKPOINT REACHED\n'
	printf 'checkpoint_id=late-attach-removal\n'
	printf 'checkpoint_token=late-attach-armed-removal-v1\n'
	printf 'expected_user_action=remove-both-power-paths\n'
	printf 'expected_response_token=late-attach-both-power-paths-removed\n'
	printf 'resume_handle=%s\n' "$handle"
}

wait_for_socket() {
	local socket_path="$1"
	local worker_pid="$2"
	for _ in $(seq 1 200); do
		[[ -S "$socket_path" ]] && return 0
		kill -0 "$worker_pid" 2>/dev/null || return 1
		sleep 0.01
	done
	return 1
}

deliver_token() {
	local handle=""
	local checkpoint_token=""
	local response_token=""
	while (($#)); do
		case "$1" in
		resume-handle=*) handle="${1#*=}" ;;
		checkpoint-token=*) checkpoint_token="${1#*=}" ;;
		response-token=*) response_token="${1#*=}" ;;
		*) die unknown_argument ;;
		esac
		shift
	done
	ensure_private_roots
	require_clean_head
	acquire_lock
	local slot
	slot="$(resolve_handle "$handle")"
	local attempt_dir
	local state_path
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	state_path="$attempt_dir/state.json"
	[[ "$(git -C "$repo_root" rev-parse HEAD)" == "$(jq -er '.tool_head' "$state_path")" ]] || die exact_head_mismatch
	if [[ "$(jq -er '.state' "$state_path")" == running || "$(jq -er '.state' "$state_path")" == watcher_armed ]]; then
		local maybe_owner
		maybe_owner="$(jq -r '.owner_pid // empty' "$state_path")"
		if [[ ! "$maybe_owner" =~ ^[1-9][0-9]*$ ]] || ! kill -0 "$maybe_owner" 2>/dev/null; then
			escrow_and_tombstone "$slot" owner_process_stale not_classified false
			release_lock
			die resume_handle_stale
		fi
		release_lock
		die lifecycle_already_running
	fi
	[[ "$(jq -er '.state' "$state_path")" == waiting_removal ]] || die checkpoint_state_mismatch
	[[ "$checkpoint_token" == late-attach-armed-removal-v1 && "$response_token" == late-attach-both-power-paths-removed ]] || die checkpoint_token_mismatch
	local now
	now="$(monotonic_ms)"
	if ((now >= $(jq -er '.checkpoint_deadline_ms' "$state_path"))); then
		escrow_and_tombstone "$slot" removal_checkpoint_expired not_classified true
		release_lock
		die checkpoint_expired
	fi
	local capability
	local socket_path
	local ready_file="$attempt_dir/worker.ready"
	capability="$(random_hex 32)"
	socket_path="${TMPDIR:-/tmp}/ultra205-late-$(jq -er '.attempt_id' "$state_path").sock"
	rm -f "$socket_path"
	phase_process_group_start "$ready_file" "$BASH" "$0" worker "$attempt_dir" "$socket_path" || die worker_start_failed
	local worker_pid="$PHASE_PROCESS_GROUP_PID"
	active_worker_pid="$worker_pid"
	local fingerprint
	fingerprint="$(process_fingerprint "$worker_pid")" || die worker_start_failed
	atomic_write "$state_path" "$(jq -c --argjson pid "$worker_pid" --arg fingerprint "$fingerprint" --arg socket "$socket_path" --arg capability "$(sha256_text "$capability")" '.state="running" | .owner_pid=$pid | .owner_fingerprint_sha256=$fingerprint | .socket_path=$socket | .lifecycle_capability_sha256=$capability' "$state_path")"
	release_lock
	if ! wait_for_socket "$socket_path" "$worker_pid"; then
		phase_process_group_terminate "$worker_pid" late-attach-worker >/dev/null 2>&1 || true
		die lifecycle_socket_unavailable
	fi
	[[ "$(mode_of "$socket_path")" == 600 && "$(owner_of "$socket_path")" == "$(id -u)" ]] || die private_capability_invalid
	[[ "$(process_fingerprint "$worker_pid")" == "$fingerprint" ]] || die owner_process_reused
	local frame
	frame="$(jq -cn --arg digest "$(sha256_text "$handle")" --arg token "$checkpoint_token" --arg response "$response_token" --arg capability "$capability" --argjson owner "$worker_pid" '{resume_handle_sha256:$digest,checkpoint_token:$token,response_token:$response,lifecycle_capability:$capability,owner_pid:$owner}')"
	printf '%s' "$frame" | perl "$frame_helper" send --socket "$socket_path"
	printf 'checkpoint_delivery=accepted\n'
	set +e
	wait "$worker_pid"
	local worker_status=$?
	set -e
	local worker_cleanup_complete=true
	if phase_process_group_is_alive "$worker_pid"; then
		if ! phase_process_group_terminate "$worker_pid" late-attach-descendant-cleanup >/dev/null 2>&1; then
			worker_cleanup_complete=false
		fi
		worker_status=1
	fi
	active_worker_pid=""
	PHASE_PROCESS_GROUP_PID=""
	ensure_private_roots
	acquire_lock
	slot="$(resolve_handle "$handle")"
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	state_path="$attempt_dir/state.json"
	local result_path="$attempt_dir/result.json"
	if ((worker_status != 0)) || [[ ! -f "$result_path" ]]; then
		local failure=worker_failed
		[[ ! -f "$result_path" ]] || failure="$(jq -r '.failure_category // "worker_failed"' "$result_path")"
		terminal_failure "$slot" "$failure" "$worker_cleanup_complete"
		release_lock
		return
	fi
	local classification
	local attempt_id
	attempt_id="$(jq -er '.attempt_id' "$slot")"
	classification="$(jq -er '.classification_category' "$result_path")"
	case "$classification" in
	all_readers_deliver | espflash_reader_silent | os_open_activates_transport | late_attach_transport_silent | os_reader_silent | unexpected_non_heartbeat_bytes | inconclusive_mixed_delivery) ;;
	*) terminal_failure "$slot" classification_invalid true ;;
	esac
	escrow_and_tombstone "$slot" diagnostic_complete "$classification" true
	release_lock
	jq -r 'to_entries[] | "\(.key)=\(.value)"' "$trace_root/$attempt_id/summary.json"
}

worker_fail() {
	local attempt_dir="$1"
	local category="$2"
	atomic_write "$attempt_dir/result.json" "$(jq -cn --arg category "$category" '{status:"failed",failure_category:$category}')"
	exit 1
}

worker_main() {
	local attempt_dir="$1"
	local socket_path="$2"
	local state_path="$attempt_dir/state.json"
	local frame_path="$attempt_dir/lifecycle-frame.json"
	perl "$frame_helper" receive --socket "$socket_path" --output "$frame_path" &
	local receiver_pid=$!
	wait "$receiver_pid" || worker_fail "$attempt_dir" lifecycle_frame_invalid
	chmod 600 "$frame_path"
	local frame_valid=false
	if jq -e \
		--arg digest "$(jq -er '.resume_handle_sha256' "$state_path")" \
		--arg capability "$(jq -er '.lifecycle_capability_sha256' "$state_path")" \
		--argjson owner "$$" '
        .resume_handle_sha256 == $digest and
        .checkpoint_token == "late-attach-armed-removal-v1" and
        .response_token == "late-attach-both-power-paths-removed" and
        ((.lifecycle_capability | @sh) | type == "string") and
        .owner_pid == $owner
      ' "$frame_path" >/dev/null; then
		[[ "$(sha256_text "$(jq -er '.lifecycle_capability' "$frame_path")")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] && frame_valid=true
	fi
	[[ "$frame_valid" == true ]] || worker_fail "$attempt_dir" private_capability_invalid
	[[ "$(process_fingerprint "$$")" == "$(jq -er '.owner_fingerprint_sha256' "$state_path")" ]] || worker_fail "$attempt_dir" owner_process_reused

	local port
	port="$(jq -er '.selected_port' "$state_path")"
	local absence_interval="${LATE_ATTACH_ABSENCE_INTERVAL_SECONDS:-0.25}"
	local absence_samples="${LATE_ATTACH_ABSENCE_SAMPLES:-20}"
	for _ in $(seq 1 "$absence_samples"); do
		[[ ! -e "$port" ]] || worker_fail "$attempt_dir" usb_absence_failed
		sleep "$absence_interval"
	done
	local armed_ms
	local restore_timeout_ms="${LATE_ATTACH_RESTORE_TIMEOUT_MS:-$default_restore_timeout_ms}"
	armed_ms="$(monotonic_ms)"
	atomic_write "$state_path" "$(jq -c --argjson armed "$armed_ms" --argjson deadline "$((armed_ms + restore_timeout_ms))" '.state="watcher_armed" | .watcher_armed_ms=$armed | .watcher_deadline_ms=$deadline' "$state_path")"
	printf '## ACTION READY\n'
	printf 'action_id=late-attach-lifecycle-restore\n'
	printf 'action_token=late-attach-reader-watcher-armed-v1\n'
	printf 'attempt_state=watcher_armed\n'
	printf 'expected_user_action=restore-barrel-then-usb\n'
	printf 'response_required=false\n'

	while [[ ! -e "$port" ]]; do
		(($(monotonic_ms) < armed_ms + restore_timeout_ms)) || worker_fail "$attempt_dir" appearance_timeout
		sleep 0.1
	done
	local observed_node
	local observed_usb
	observed_node="$(serial_session_node_identity "$port" 2>/dev/null)" || worker_fail "$attempt_dir" identity_unavailable
	observed_usb="$(serial_session_usb_identity "$port" 2>/dev/null)" || worker_fail "$attempt_dir" identity_unavailable
	[[ "$observed_usb" == "$(jq -er '.selected_usb_identity' "$state_path")" ]] || worker_fail "$attempt_dir" usb_identity_changed
	[[ "$observed_node" != "$(jq -er '.selected_node_identity' "$state_path")" ]] || worker_fail "$attempt_dir" enumeration_epoch_unchanged
	# shellcheck disable=SC2034 # Read by sourced serial-session helpers.
	SERIAL_SESSION_TRACE_ROOT="$attempt_dir/session-traces"
	serial_session_trace_init late-attach-appearance
	serial_session_readiness_gate late_attach_appearance "$port" || worker_fail "$attempt_dir" "$SERIAL_SESSION_READINESS_CATEGORY"

	local capture_seconds
	capture_seconds="$(jq -er '.capture_seconds' "$state_path")"
	run_capture "$attempt_dir" "$port" espflash "$capture_seconds" cold-espflash-before || worker_fail "$attempt_dir" espflash_before_capture_failed
	run_capture "$attempt_dir" "$port" os-native "$capture_seconds" cold-os-native || worker_fail "$attempt_dir" os_native_capture_failed
	run_capture "$attempt_dir" "$port" espflash "$capture_seconds" cold-espflash-after || worker_fail "$attempt_dir" espflash_after_capture_failed
	local final_usb
	final_usb="$(serial_session_usb_identity "$port" 2>/dev/null)" || worker_fail "$attempt_dir" identity_unavailable
	[[ "$final_usb" == "$(jq -er '.selected_usb_identity' "$state_path")" ]] || worker_fail "$attempt_dir" usb_identity_changed
	serial_session_readiness_gate terminal_cleanup "$port" || worker_fail "$attempt_dir" "cleanup_${SERIAL_SESSION_READINESS_CATEGORY}"
	local classification_path="$attempt_dir/classification.json"
	node "$classifier_bin" classify "$attempt_dir/cold-espflash-before.raw.log" "$attempt_dir/cold-os-native.raw.log" "$attempt_dir/cold-espflash-after.raw.log" >"$classification_path" || worker_fail "$attempt_dir" classifier_failed
	chmod 600 "$classification_path"
	local digest
	digest="$(trace_digest "$attempt_dir")"
	local summary_path="$attempt_dir/summary.json"
	jq -n \
		--slurpfile classification "$classification_path" \
		--slurpfile state "$state_path" \
		--arg digest "$digest" \
		'{schema_version:"ultra205-late-attach-summary-v1",diagnostic_status:"complete",classification_category:$classification[0].category,capture_seconds:$state[0].capture_seconds,preflight_espflash_heartbeat_count:$state[0].preflight_espflash_heartbeat_count,preflight_os_native_heartbeat_count:$state[0].preflight_os_native_heartbeat_count,preflight_same_session:$state[0].preflight_same_session,espflash_before_heartbeat_count:$classification[0].espflash_before_heartbeat_count,os_native_heartbeat_count:$classification[0].os_native_heartbeat_count,espflash_after_heartbeat_count:$classification[0].espflash_after_heartbeat_count,same_session:$classification[0].same_session,monotonic:$classification[0].monotonic,cadence_valid:$classification[0].cadence_valid,unexpected_non_heartbeat_line_count:$classification[0].unexpected_non_heartbeat_line_count,usb_identity_stable:true,new_enumeration_epoch:true,cleanup_complete:true,trace_digest_sha256:$digest,operations:{detector_count:1,post_detector:false,flash:false,erase:false,factory_reset:false,credential_read:false,network_discovery:false,port_scan_after_detector:false,raw_serial_write:false,reset_after_removal:false}}' >"$summary_path"
	chmod 600 "$summary_path"
	atomic_write "$attempt_dir/result.json" "$(jq -cn --arg category "$(jq -er '.category' "$classification_path")" '{status:"complete",classification_category:$category}')"
}

cleanup_exit() {
	local status=$?
	if [[ -n "$active_worker_pid" ]]; then
		phase_process_group_terminate "$active_worker_pid" late-attach-worker >/dev/null 2>&1 || status=1
	fi
	if [[ "$lock_held" == true ]]; then
		rm -f "$lock_dir/owner.pid"
		rmdir "$lock_dir" 2>/dev/null || true
	fi
	exit "$status"
}

if [[ "${LATE_ATTACH_TEST_MODE:-0}" != "1" ]]; then
	for test_override in \
		LATE_ATTACH_ABSENCE_INTERVAL_SECONDS \
		LATE_ATTACH_ABSENCE_SAMPLES \
		LATE_ATTACH_PREFLIGHT_SECONDS \
		LATE_ATTACH_RESTORE_TIMEOUT_MS; do
		[[ -z "${!test_override:-}" ]] || die test_configuration_invalid
	done
fi

trap cleanup_exit EXIT INT TERM

command="${1:-}"
[[ -n "$command" ]] || {
	usage
	exit 2
}
shift
case "$command" in
begin) begin_diagnostic "$@" ;;
deliver) deliver_token "$@" ;;
worker)
	[[ $# -eq 2 ]] || die worker_arguments_invalid
	worker_main "$1" "$2"
	;;
-h | --help) usage ;;
*)
	usage
	exit 2
	;;
esac
