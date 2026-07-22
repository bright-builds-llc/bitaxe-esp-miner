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
	shift 3
	local classifier_stderr="${output%.json}.stderr"
	if ! "${workspace_dir}/bazel-bin/tools/parity/report" phase33-classify \
		--trace "$trace" \
		--mode "$mode" \
		"$@" >"$output" 2>"$classifier_stderr"; then
		chmod 600 "$output" "$classifier_stderr"
		failure_category="boot_classifier_failed"
		return 1
	fi
	chmod 600 "$output" "$classifier_stderr"

	local classification_status
	classification_status="$(jq -er '.status' "$output")" || {
		failure_category="boot_classifier_output_invalid"
		return 1
	}
	if [[ "$classification_status" == "passed" ]]; then
		return 0
	fi
	if [[ "$classification_status" != "failed" ]]; then
		failure_category="boot_classifier_output_invalid"
		return 1
	fi

	local classification_category
	classification_category="$(jq -er \
		'.category | select(type == "string") | select(test("^[a-z0-9_]+$"))' \
		"$output")" || {
		failure_category="boot_classifier_rejected"
		return 1
	}
	failure_category="$classification_category"
	return 1
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
		--evidence-mode
		dual
	)
	if [[ -n "$wifi_credentials" ]]; then
		args+=(--wifi-credentials "$wifi_credentials")
	fi
	if ! "$flash_executable" "${args[@]}" >"$local_root/raw/flash-command.log" 2>&1; then
		chmod 600 "$local_root/raw/flash-command.log"
		return 1
	fi
	chmod 600 "$local_root/raw/flash-command.log"
	local classifier_input="$flash_dir/flash-monitor.classifier-input.log"
	local private_record="$flash_dir/flash-command-evidence.private.json"
	local admitted_log="$flash_dir/flash-monitor.log"
	local admitted_record="$flash_dir/flash-command-evidence.json"
	[[ -s "$classifier_input" && -s "$private_record" ]] || {
		failure_category="private_capture_incomplete"
		return 1
	}
	[[ ! -e "$admitted_log" && ! -e "$admitted_record" ]] || {
		failure_category="admitted_projection_created_before_classification"
		return 1
	}
	local private_digest
	private_digest="$(jq -er \
		'.private_monitor_log_sha256 | select(type == "string") | select(test("^[0-9a-f]{64}$"))' \
		"$private_record")" || {
		failure_category="private_digest_record_invalid"
		return 1
	}
	[[ "$(sha256_file "$classifier_input")" == "$private_digest" ]] || {
		failure_category="private_digest_record_mismatch"
		return 1
	}
	if ! production_classify_boot baseline "$classifier_input" "$output"; then
		chmod 600 "$output"
		return 1
	fi
	chmod 600 "$output"
	[[ "$(sha256_file "$classifier_input")" == "$private_digest" ]] || {
		failure_category="private_input_changed_during_classification"
		return 1
	}
	if ! "$flash_executable" finalize-evidence \
		--evidence-dir "$flash_dir" \
		--expected-private-sha256 "$private_digest" \
		>"$local_root/raw/flash-finalize-command.log" 2>&1; then
		chmod 600 "$local_root/raw/flash-finalize-command.log"
		failure_category="admitted_projection_finalization_failed"
		return 1
	fi
	chmod 600 "$local_root/raw/flash-finalize-command.log"
	[[ "$(sha256_file "$classifier_input")" == "$private_digest" ]] || {
		failure_category="private_input_changed_during_finalization"
		return 1
	}
	[[ -s "$admitted_log" && -s "$admitted_record" ]] || {
		failure_category="admitted_projection_incomplete"
		return 1
	}
}

read_setting_into() {
	local label="$1"
	local output_variable="$2"
	last_http_category=""
	if [[ -n "$fixture_command" ]]; then
		local fixture_output
		local fixture_status
		set +e
		fixture_output="$(fixture read_setting "$label" "$target_token")"
		fixture_status=$?
		set -e
		if ((fixture_status != 0)); then
			last_http_category="$(
				sed -n 's/^category=\([a-z0-9_]*\)$/\1/p' <<<"$fixture_output" | tail -1
			)"
			return 1
		fi
		printf -v "$output_variable" '%s' "$fixture_output"
		return 0
	fi

	local adapter="${script_dir}/phase35-http-boundary-read.sh"
	[[ -x "$adapter" ]] || {
		last_http_category="http_diagnostic_invalid"
		return 1
	}
	local adapter_output
	local adapter_status
	set +e
	adapter_output="$(
		"$adapter" \
			"label=${label}" \
			"protected-root=${local_root}/raw" \
			"url=${target_token}"
	)"
	adapter_status=$?
	set -e
	local category
	category="$(
		sed -n 's/^category=\([a-z0-9_]*\)$/\1/p' <<<"$adapter_output"
	)" || return 1
	[[ -n "$category" && "$(wc -l <<<"$category" | tr -d ' ')" == 1 ]] || {
		last_http_category="http_diagnostic_invalid"
		return 1
	}
	last_http_category="$category"
	[[ "$category" == ready && "$adapter_status" == 0 ]] || return 1

	local hostname_file="${local_root}/raw/http-${label}/private-hostname"
	[[ -f "$hostname_file" && ! -L "$hostname_file" ]] || {
		last_http_category="http_diagnostic_invalid"
		return 1
	}
	local private_host_value
	private_host_value="$(<"$hostname_file")"
	[[ -n "$private_host_value" ]] || {
		last_http_category="http_diagnostic_invalid"
		return 1
	}
	printf -v "$output_variable" '%s' "$private_host_value"
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

	local boot_classification
	case "$label" in
	boot-a-pre | boot-a) boot_classification="$local_root/raw/boot-a-setup.json" ;;
	boot-b) boot_classification="$local_root/raw/boot-b-setup.json" ;;
	*) return 1 ;;
	esac
	local ordinal classified_session
	ordinal="$(jq -er '.boot_ordinal | select(type == "number")' "$boot_classification" 2>/dev/null)" ||
		return 1
	classified_session="$(jq -er '.session | select(type == "string" and length > 0)' "$boot_classification" 2>/dev/null)" ||
		return 1

	local api_body="$local_root/raw/${label}-api.json"
	local api_stderr="$local_root/raw/${label}-api.stderr"
	local websocket_log="$local_root/raw/${label}-websocket.log"
	local websocket_stderr="$local_root/raw/${label}-websocket.stderr"
	local retained_log="$local_root/raw/${label}-retained.log"
	local retained_stderr="$local_root/raw/${label}-retained.stderr"
	local started
	started="$(monotonic_millis)" || return 1
	curl --silent --show-error --fail --http1.1 --noproxy '*' --proto '=http,https' \
		--connect-timeout 5 --max-time 10 --max-filesize 65536 \
		--output "$api_body" "${target_token}/api/system/info" 2>"$api_stderr" || {
		chmod 600 "$api_stderr"
		[[ ! -e "$api_body" ]] || chmod 600 "$api_body"
		return 1
	}
	node "${script_dir}/phase17-websocket-capture.mjs" \
		--device-url "$target_token" \
		--path /api/ws/live \
		--out "$websocket_log" \
		--duration-ms 10000 \
		--max-frames 1 2>"$websocket_stderr" || {
		chmod 600 "$api_body" "$api_stderr" "$websocket_stderr"
		[[ ! -e "$websocket_log" ]] || chmod 600 "$websocket_log"
		return 1
	}
	curl --silent --show-error --fail --http1.1 --noproxy '*' --proto '=http,https' \
		--connect-timeout 5 --max-time 10 --max-filesize 524288 \
		--output "$retained_log" "${target_token}/api/system/logs" 2>"$retained_stderr" || {
		chmod 600 "$api_body" "$api_stderr" "$websocket_log" "$websocket_stderr" \
			"$retained_stderr"
		[[ ! -e "$retained_log" ]] || chmod 600 "$retained_log"
		return 1
	}
	chmod 600 "$api_body" "$api_stderr" "$websocket_log" "$websocket_stderr" \
		"$retained_log" "$retained_stderr"

	local websocket_frame websocket_payload
	websocket_frame="$(sed -n 's/^websocket_frame_1=//p' "$websocket_log")"
	[[ -n "$websocket_frame" ]] || return 1
	websocket_payload="$(jq -cer '.data | select(type == "object")' <<<"$websocket_frame" 2>/dev/null)" ||
		return 1
	local session revision websocket_session websocket_revision private_host_value
	session="$(jq -er '.bootSession | select(type == "string" and length > 0)' "$api_body" 2>/dev/null)" ||
		return 1
	revision="$(jq -er '.operatorSnapshotRevision | select(type == "number")' "$api_body" 2>/dev/null)" ||
		return 1
	private_host_value="$(jq -er '.hostname | select(type == "string" and length > 0)' "$api_body" 2>/dev/null)" ||
		return 1
	websocket_session="$(jq -er '.bootSession | select(type == "string" and length > 0)' <<<"$websocket_payload" 2>/dev/null)" ||
		return 1
	websocket_revision="$(jq -er '.operatorSnapshotRevision | select(type == "number")' <<<"$websocket_payload" 2>/dev/null)" ||
		return 1
	[[ "$session" == "$classified_session" && "$websocket_session" == "$session" ]] ||
		return 1
	((websocket_revision > revision)) || return 1
	local api_marker websocket_marker
	api_marker="operator_snapshot session=${session} revision=${revision} redacted=true"
	websocket_marker="operator_snapshot session=${session} revision=${websocket_revision} redacted=true"
	rg -Fqx -- "$api_marker" "$retained_log" >/dev/null 2>&1 || return 1
	rg -Fqx -- "$websocket_marker" "$retained_log" >/dev/null 2>&1 || return 1
	local ended
	ended="$(monotonic_millis)" || return 1
	((ended > started)) || return 1
	jq -cn \
		--argjson boot_ordinal "$ordinal" \
		--arg session "$session" \
		--argjson revision "$revision" \
		--arg setting_digest "$(sha256_text "$private_host_value")" \
		--arg reset_category "$([[ "$label" == boot-b ]] && printf software_cpu || printf setup)" \
		--arg system_info_document "system_info_json: $(<"$api_body")
operator_snapshot_boot_session: ${session}
operator_snapshot_revision: ${revision}" \
		--arg websocket_document "live_websocket_json: ${websocket_payload}
operator_snapshot_boot_session: ${websocket_session}
operator_snapshot_revision: ${websocket_revision}" \
		--arg retained_log_document "$(<"$retained_log")" \
		--argjson started_millis "$started" \
		--argjson ended_millis "$ended" \
		'{boot_ordinal:$boot_ordinal,boot_session:$session,storage_revision:$revision,reset_category:$reset_category,setting_digest:$setting_digest,system_info_document:$system_info_document,websocket_document:$websocket_document,retained_log_document:$retained_log_document,started_millis:$started_millis,ended_millis:$ended_millis}' \
		>"$output" || {
		chmod 600 "$output"
		return 1
	}
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
	jq -cn --arg private_host_value "$new_value" \
		'{("host" + "name"):$private_host_value}' >"$payload"
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
	passive_monitor_pid=$!
	for _ in $(seq 1 80); do
		[[ -s "$passive_ready" ]] && break
		kill -0 "$passive_monitor_pid" >/dev/null 2>&1 || return 1
		sleep 0.25
	done
	[[ -s "$passive_ready" ]] || return 1
	local reboot_stderr="$local_root/raw/reboot.stderr"
	curl --silent --show-error --fail --max-time 15 \
		--request POST \
		--output "$local_root/raw/reboot-response.json" \
		"${target_token}/api/system/restart" 2>"$reboot_stderr" || {
		chmod 600 "$reboot_stderr"
		[[ ! -e "$local_root/raw/reboot-response.json" ]] ||
			chmod 600 "$local_root/raw/reboot-response.json"
		return 1
	}
	chmod 600 "$local_root/raw/reboot-response.json" "$reboot_stderr"
	local service_loss_stderr="$local_root/raw/service-loss-probe.stderr"
	local service_lost=0
	for _ in $(seq 1 80); do
		if ! curl --silent --http1.1 --noproxy '*' --proto '=http,https' \
			--connect-timeout 1 --max-time 1 --output /dev/null \
			"${target_token}/api/system/info" 2>"$service_loss_stderr"; then
			service_lost=1
			break
		fi
		sleep 0.25
	done
	chmod 600 "$service_loss_stderr"
	((service_lost == 1)) || return 1
	local restart_offset
	restart_offset="$(wc -c <"$passive_raw" | tr -d ' ')" || return 1
	[[ "$restart_offset" =~ ^[0-9]+$ ]] || return 1
	local passive_status=0
	set +e
	wait "$passive_monitor_pid"
	passive_status=$?
	set -e
	passive_monitor_pid=""
	((passive_status == 0)) || return 1
	chmod 600 "$passive_log" "$passive_raw"
	local baseline_session baseline_ordinal boot_b_setup
	baseline_session="$(jq -er '.session | select(type == "string" and length > 0)' "$local_root/raw/boot-a-setup.json")" ||
		return 1
	baseline_ordinal="$(jq -er '.boot_ordinal | select(type == "number")' "$local_root/raw/boot-a-setup.json")" ||
		return 1
	boot_b_setup="$local_root/raw/boot-b-setup.json"
	production_classify_boot post-restart "$passive_raw" "$boot_b_setup" \
		--start-byte "$restart_offset" \
		--expected-session "$baseline_session" \
		--expected-ordinal "$baseline_ordinal" || {
		chmod 600 "$boot_b_setup"
		return 1
	}
	chmod 600 "$boot_b_setup"
	target_token="$(jq -er '.device_url | select(type == "string" and length > 0)' "$boot_b_setup")" ||
		return 1
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
	if ((restoration_attempted == 1)); then
		((restoration_complete == 1))
		return
	fi
	restoration_attempted=1
	if [[ -n "$fixture_command" ]]; then
		fixture restore "$target_token" "$original_setting" || {
			restoration_secondary_category="restoration_action_failed"
			return 1
		}
	else
		patch_setting "$original_setting" || {
			restoration_secondary_category="restoration_action_failed"
			return 1
		}
	fi
	local restored=""
	read_setting_into restoration restored || {
		restoration_secondary_category="${last_http_category:-restoration_read_failed}"
		return 1
	}
	[[ "$restored" == "$original_setting" ]] || {
		restoration_secondary_category="restoration_value_mismatch"
		return 1
	}
	restoration_complete=1
	record_checkpoint restoration_confirmed "$(hash_fields phase35-restoration-v1 true)"
}

cleanup_resources_once() {
	if ((cleanup_attempted == 1)); then
		((cleanup_complete == 1))
		return
	fi
	cleanup_attempted=1
	if [[ -n "$passive_monitor_pid" ]]; then
		local passive_cleanup_status=0
		set +e
		if kill -0 "$passive_monitor_pid" >/dev/null 2>&1; then
			kill -TERM "$passive_monitor_pid" >/dev/null 2>&1
		fi
		wait "$passive_monitor_pid" >/dev/null 2>&1
		passive_cleanup_status=$?
		set -e
		passive_monitor_pid=""
		if ((passive_cleanup_status != 0 && passive_cleanup_status != 130 && passive_cleanup_status != 143)); then
			cleanup_secondary_category="cleanup_passive_monitor_failed"
			return 1
		fi
	fi
	if [[ -n "$fixture_command" ]]; then
		fixture cleanup || {
			cleanup_secondary_category="cleanup_resource_failure"
			return 1
		}
	elif [[ -n "$port" ]]; then
		local maybe_holders=""
		maybe_holders="$(serial_session_holder_pids "$port")" || {
			cleanup_secondary_category="cleanup_holder_check_failed"
			return 1
		}
		[[ -z "$maybe_holders" ]] || {
			cleanup_secondary_category="cleanup_holder_present"
			return 1
		}
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
		"restoration_secondary_category=${restoration_secondary_category:-none}" \
		"cleanup_secondary_category=${cleanup_secondary_category:-none}" \
		"root_reusable=false"
}

capture_primary_failure() {
	local category="$1"
	[[ -n "$primary_failure_category" ]] || primary_failure_category="$category"
	failure_category="$primary_failure_category"
}

finalize_resources_once() {
	local restoration_status=0
	local cleanup_status=0
	set +e
	restore_setting_once
	restoration_status=$?
	cleanup_resources_once
	cleanup_status=$?
	set -e
	((restoration_status == 0 && cleanup_status == 0))
}

finalize_once() {
	local incoming_status="$1"
	((finalizer_ran == 0)) || return "$incoming_status"
	finalizer_ran=1
	local finalization_status=0
	finalize_resources_once || finalization_status=$?
	if ((incoming_status != 0)); then
		capture_primary_failure "${primary_failure_category:-${failure_category:-supervisor_failed}}"
		seal_non_promotion "$primary_failure_category"
		return "$incoming_status"
	fi
	if ((finalization_status != 0)); then
		capture_primary_failure supervisor_finalization_failed
		seal_non_promotion "$primary_failure_category"
		return 1
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
	capture_primary_failure "$1"
	printf 'failure_category=%s\n' "$primary_failure_category" >&2
	exit 1
}
