#!/usr/bin/env bash
# Detector, capture, restoration, and cleanup helpers for the Phase 35 supervisor.
# shellcheck disable=SC2034,SC2154

detector_failure() {
	local log="$1"
	local maybe_category
	maybe_category="$(sed -n 's/^failure_category=\([a-z0-9_]*\)$/\1/p' "$log" | tail -1)"
	printf '%s\n' "${maybe_category:-detector_failed}"
}

run_detector_gate() {
	local detector_log="$local_root/raw/detector.log"
	local detector_status
	set +e
	if [[ -n "$fixture_command" ]]; then
		fixture detector >"$detector_log" 2>&1
		detector_status=$?
	else
		just detect-ultra205 >"$detector_log" 2>&1
		detector_status=$?
	fi
	set -e
	chmod 600 "$detector_log"
	if ((detector_status != 0)); then
		failure_category="$(detector_failure "$detector_log")"
		return 1
	fi

	local port_count board_count
	port_count="$(sed -n 's/^port=//p' "$detector_log" | wc -l | tr -d ' ')"
	board_count="$(sed -n 's/^board=//p' "$detector_log" | wc -l | tr -d ' ')"
	[[ "$port_count" == "1" ]] || {
		failure_category="detector_candidate_count_invalid"
		return 1
	}
	if [[ "$board_count" != "0" ]]; then
		[[ "$board_count" == "1" && "$(sed -n 's/^board=//p' "$detector_log")" == "205" ]] || {
			failure_category="wrong_board"
			return 1
		}
	fi
	port="$(sed -n 's/^port=//p' "$detector_log")"

	if [[ -n "$fixture_command" ]]; then
		physical_identity_digest="$(fixture physical_identity "$port")" || {
			failure_category="physical_identity_unavailable"
			return 1
		}
	else
		physical_identity_digest="$(serial_session_usb_physical_identity "$port")" || {
			failure_category="physical_identity_unavailable"
			return 1
		}
	fi
	[[ "$physical_identity_digest" =~ ^[0-9a-f]{64}$ ]] || {
		failure_category="physical_identity_invalid"
		return 1
	}

	jq -cn \
		--arg board_category 205 \
		--arg physical_identity_digest "$physical_identity_digest" \
		--arg run_id_digest "$run_id_digest" \
		'{board_category:$board_category,physical_identity_digest:$physical_identity_digest,run_id_digest:$run_id_digest,board_info_verified:true,single_candidate_verified:true}' \
		>"$local_root/artifacts/detector-capability.json"
	chmod 600 "$local_root/artifacts/detector-capability.json"
	local detector_artifact_digest
	detector_artifact_digest="$(sha256_file "$local_root/artifacts/detector-capability.json")"
	detector_capability_digest="$(hash_fields phase35-detector-run-v1 \
		205 "$detector_artifact_digest" "$physical_identity_digest" true true "$run_id_digest")"
	jq \
		--arg detector_capability_digest "$detector_artifact_digest" \
		--arg capability_digest "$detector_capability_digest" \
		'. + {detector_capability_digest:$detector_capability_digest,capability_digest:$capability_digest}' \
		"$local_root/artifacts/detector-capability.json" >"$local_root/raw/detector-run-capability.json"
	chmod 600 "$local_root/raw/detector-run-capability.json"

	jq -cn \
		--arg board_category 205 \
		--arg physical_identity_digest "$physical_identity_digest" \
		--arg run_id_digest "$run_id_digest" \
		'{board_category:$board_category,physical_identity_digest:$physical_identity_digest,run_id_digest:$run_id_digest,target_locked:true}' \
		>"$local_root/artifacts/target-lock.json"
	chmod 600 "$local_root/artifacts/target-lock.json"
	target_lock_digest="$(sha256_file "$local_root/artifacts/target-lock.json")"
}

validate_credential_path_after_detector() {
	[[ -n "$wifi_credentials" ]] || return 0
	local resolved_credentials
	resolved_credentials="$(absolute_path "$wifi_credentials")"
	if [[ -n "$fixture_command" ]]; then
		fixture credential_path "$resolved_credentials" >/dev/null
		wifi_credentials="$resolved_credentials"
		return
	fi
	[[ -f "$resolved_credentials" ]] || {
		failure_category="wifi_credentials_path_missing"
		return 1
	}
	[[ "$resolved_credentials" == "${workspace_dir}/"* ]] || {
		failure_category="wifi_credentials_path_not_ignored"
		return 1
	}
	local workspace_relative_credentials="${resolved_credentials#"${workspace_dir}/"}"
	git -C "$workspace_dir" check-ignore -q -- "$workspace_relative_credentials" || {
		failure_category="wifi_credentials_path_not_ignored"
		return 1
	}
	wifi_credentials="$resolved_credentials"
}

production_classify_boot() {
	local mode="$1"
	local trace="$2"
	local output="$3"
	"${workspace_dir}/bazel-bin/tools/parity/report" phase33-classify \
		--trace "$trace" \
		--mode "$mode" >"$output"
}

run_flash_boot_a() {
	local output="$local_root/raw/boot-a-setup.json"
	if [[ -n "$fixture_command" && "$fixture_direct_flash" != true ]]; then
		if ! fixture flash_boot_a "$capture_timeout_seconds" >"$output"; then
			chmod 600 "$output"
			return 1
		fi
		chmod 600 "$output"
		return
	fi

	local flash_dir="$local_root/raw/flash"
	mkdir -p "$flash_dir"
	chmod 700 "$flash_dir"
	local flash_executable
	flash_executable="$(resolve_flash_executable)" || {
		failure_category="flash_executable_unavailable"
		return 1
	}
	local args=(
		flash-monitor
		--board
		205
		--port
		"$port"
		--manifest
		"$manifest"
		--evidence-dir
		"$flash_dir"
		--capture-timeout-seconds
		"$capture_timeout_seconds"
	)
	if [[ -n "$wifi_credentials" ]]; then
		args+=(--wifi-credentials "$wifi_credentials")
	fi
	if ! "$flash_executable" "${args[@]}" >"$local_root/raw/flash-command.log" 2>&1; then
		chmod 600 "$local_root/raw/flash-command.log"
		return 1
	fi
	chmod 600 "$local_root/raw/flash-command.log"
	local monitor_log="$flash_dir/flash-monitor.log"
	[[ -s "$monitor_log" ]] || return 1
	production_classify_boot baseline "$monitor_log" "$output"
	chmod 600 "$output"
}

read_setting() {
	local label="$1"
	if [[ -n "$fixture_command" ]]; then
		fixture read_setting "$label" "$target_token"
		return
	fi
	local body="$local_root/raw/${label}-system-info.json"
	curl --silent --show-error --fail --max-time 10 \
		--output "$body" "${target_token}/api/system/info"
	chmod 600 "$body"
	jq -er '.hostname | select(type == "string")' "$body"
}

capture_epoch() {
	local label="$1"
	local output="$local_root/raw/${label}.json"
	if [[ -n "$fixture_command" ]]; then
		if ! fixture capture_epoch "$label" "$target_token" >"$output"; then
			chmod 600 "$output"
			return 1
		fi
		chmod 600 "$output"
		printf '%s\n' "$output"
		return
	fi

	local api_body="$local_root/raw/${label}-api.json"
	local websocket_log="$local_root/raw/${label}-websocket.log"
	local retained_log="$local_root/raw/${label}-retained.log"
	curl --silent --show-error --fail --max-time 10 \
		--output "$api_body" "${target_token}/api/system/info"
	node "${script_dir}/phase17-websocket-capture.mjs" \
		--device-url "$target_token" \
		--path /api/ws/live \
		--out "$websocket_log" \
		--duration-ms 10000 \
		--max-frames 1
	local websocket_json
	websocket_json="$(sed -n 's/^websocket_frame_1=//p' "$websocket_log")"
	[[ -n "$websocket_json" ]] || return 1
	local session revision ordinal
	session="$(jq -er '.bootSession' "$api_body")"
	revision="$(jq -er '.operatorSnapshotRevision' "$api_body")"
	ordinal="$(jq -er '.bootOrdinal' "$api_body")"
	printf 'operator_snapshot session=%s revision=%s redacted=true\n' "$session" "$revision" >"$retained_log"
	chmod 600 "$api_body" "$websocket_log" "$retained_log"
	local started ended
	started="$(monotonic_millis)"
	ended="$((started + 1))"
	jq -cn \
		--argjson boot_ordinal "$ordinal" \
		--arg session "$session" \
		--argjson revision "$revision" \
		--arg reset_category "$([[ "$label" == boot-b ]] && printf software_cpu || printf setup)" \
		--arg system_info_document "system_info_json: $(<"$api_body")
operator_snapshot_boot_session: ${session}
operator_snapshot_revision: ${revision}" \
		--arg websocket_document "live_websocket_json: ${websocket_json}
operator_snapshot_boot_session: ${session}
operator_snapshot_revision: ${revision}" \
		--arg retained_log_document "$(<"$retained_log")" \
		--argjson started_millis "$started" \
		--argjson ended_millis "$ended" \
		'{boot_ordinal:$boot_ordinal,boot_session:$session,storage_revision:$revision,reset_category:$reset_category,system_info_document:$system_info_document,websocket_document:$websocket_document,retained_log_document:$retained_log_document,started_millis:$started_millis,ended_millis:$ended_millis}' \
		>"$output"
	chmod 600 "$output"
	printf '%s\n' "$output"
}

patch_setting() {
	local new_value="$1"
	if [[ -n "$fixture_command" ]]; then
		fixture patch "$target_token" "$new_value"
		return
	fi
	local payload="$local_root/raw/patch-request.json"
	local response="$local_root/raw/patch-response.txt"
	jq -cn --arg hostname "$new_value" '{hostname:$hostname}' >"$payload"
	chmod 600 "$payload"
	local code
	code="$(curl --silent --show-error --max-time 15 \
		--request PATCH \
		--header 'Content-Type: application/json' \
		--data-binary "@${payload}" \
		--output "$response" \
		--write-out '%{http_code}' \
		"${target_token}/api/system")"
	chmod 600 "$response"
	[[ "$code" == "200" && ! -s "$response" ]]
}

start_passive_monitor_and_reboot() {
	{
		printf 'required_contract='
		printf '%q ' "${PASSIVE_MONITOR_ARGS[@]}"
		printf '\n'
	} >"$local_root/raw/passive-monitor-contract.txt"
	chmod 600 "$local_root/raw/passive-monitor-contract.txt"
	if [[ -n "$fixture_command" ]]; then
		fixture reboot "$target_token" "${PASSIVE_MONITOR_ARGS[@]}"
		return
	fi
	local passive_log="$local_root/raw/passive-monitor.log"
	local passive_raw="$local_root/raw/passive-monitor.raw"
	local passive_ready="$local_root/raw/passive-monitor.ready"
	PHASE13_MONITOR_ACTIVE_READY_FILE="$passive_ready" \
		SERIAL_SESSION_TRACE_ROOT="$local_root/raw" \
		bash "${script_dir}/phase13-monitor-capture.sh" \
		--port "$port" \
		--out "$passive_log" \
		--raw-out "$passive_raw" \
		--seconds "$capture_timeout_seconds" \
		--reader espflash \
		--no-reset &
	readonly passive_pid=$!
	for _ in $(seq 1 80); do
		[[ -s "$passive_ready" ]] && break
		kill -0 "$passive_pid" >/dev/null 2>&1 || return 1
		sleep 0.25
	done
	[[ -s "$passive_ready" ]] || return 1
	curl --silent --show-error --fail --max-time 15 \
		--request POST \
		--output "$local_root/raw/reboot-response.json" \
		"${target_token}/api/system/restart"
	wait "$passive_pid"
	chmod 600 "$passive_log" "$passive_raw" "$local_root/raw/reboot-response.json"
}

verify_same_identity() {
	local after
	if [[ -n "$fixture_command" ]]; then
		after="$(fixture physical_identity_after "$port")" || return 1
	else
		after="$(serial_session_usb_physical_identity "$port")" || return 1
	fi
	[[ "$after" == "$physical_identity_digest" ]]
}

restore_setting_once() {
	((mutation_started == 1)) || return 0
	((restoration_complete == 0)) || return 0
	if [[ -n "$fixture_command" ]]; then
		fixture restore "$target_token" "$original_setting" || return 1
	else
		patch_setting "$original_setting" || return 1
	fi
	local restored
	restored="$(read_setting restoration)" || return 1
	[[ "$restored" == "$original_setting" ]] || return 1
	restoration_complete=1
	record_checkpoint restoration_confirmed "$(hash_fields phase35-restoration-v1 true)"
}

cleanup_resources_once() {
	((cleanup_complete == 0)) || return 0
	if [[ -n "$fixture_command" ]]; then
		fixture cleanup || return 1
	elif [[ -n "$port" ]]; then
		local maybe_holders=""
		maybe_holders="$(serial_session_holder_pids "$port")" || return 1
		[[ -z "$maybe_holders" ]] || return 1
	fi
	cleanup_complete=1
	record_checkpoint cleanup_confirmed "$(hash_fields phase35-cleanup-v1 true)"
}

seal_non_promotion() {
	local category="$1"
	[[ -f "$local_root/non-promotion.seal" ]] && return 0
	write_private "$local_root/non-promotion.seal" \
		"status=non_promotion" \
		"category=${category}" \
		"root_reusable=false"
}

finalize_once() {
	local incoming_status="$1"
	((finalizer_ran == 0)) || return "$incoming_status"
	finalizer_ran=1
	set +e
	local restoration_status=0
	local cleanup_status=0
	restore_setting_once
	restoration_status=$?
	cleanup_resources_once
	cleanup_status=$?
	set -e
	if ((restoration_status != 0)); then
		failure_category="restoration_failed"
		seal_non_promotion "$failure_category"
		return 1
	fi
	if ((cleanup_status != 0)); then
		failure_category="cleanup_failed"
		seal_non_promotion "$failure_category"
		return 1
	fi
	if ((incoming_status != 0)); then
		seal_non_promotion "${failure_category:-supervisor_failed}"
		return "$incoming_status"
	fi
	return 0
}

on_exit() {
	local status=$?
	trap - EXIT
	if ! finalize_once "$status"; then
		status=1
	fi
	exit "$status"
}

fail() {
	failure_category="$1"
	printf 'failure_category=%s\n' "$failure_category" >&2
	exit 1
}
