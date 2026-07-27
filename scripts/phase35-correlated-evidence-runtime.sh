#!/usr/bin/env bash
# Flash, baseline-read, and epoch-capture helpers for Phase 35.

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
	local flash_status=0
	set +e
	PHASE35_FLASH_STAGE_ROOT="$flash_dir/private-stages" \
		PHASE35_STAGE_READINESS_BIN="${workspace_dir}/scripts/phase35-stage-readiness.sh" \
		PHASE35_EXPECTED_PHYSICAL_IDENTITY="$physical_identity_digest" \
		"$flash_executable" "${args[@]}" >"$local_root/raw/flash-command.log" 2>&1
	flash_status=$?
	set -e
	chmod 600 "$local_root/raw/flash-command.log"
	if ! classify_available_flash_stages; then
		[[ -n "$failure_category" ]] || failure_category="flash_or_boot_a_failed"
		return 1
	fi
	((flash_status == 0)) || {
		failure_category="flash_or_boot_a_failed"
		return 1
	}
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
	rg -Fqx 'websocket_close_status=closed' "$websocket_log" >/dev/null 2>&1 ||
		return 1
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

run_device_session_reboot() {
	if [[ -n "$fixture_command" ]]; then
		fixture reboot "$target_token" "${FIXTURE_REBOOT_CONTRACT_ARGS[@]}"
		return
	fi

	local baseline_session baseline_ordinal
	baseline_session="$(jq -er '.session | select(type == "string" and length > 0)' "$local_root/raw/boot-a-setup.json")" ||
		return 1
	baseline_ordinal="$(jq -er '.boot_ordinal | select(type == "number")' "$local_root/raw/boot-a-setup.json")" ||
		return 1
	local request_input="$local_root/raw/device-session-request.json"
	local session_root="$local_root/raw/device-session"
	local projection="$local_root/raw/device-session-projection.json"
	local stdout_file="$local_root/raw/device-session.stdout"
	local stderr_file="$local_root/raw/device-session.stderr"
	local device_session_executable
	device_session_executable="$(resolve_device_session_executable)" || {
		failure_category="device_session_executable_unavailable"
		return 1
	}

	jq -cn \
		--arg schema_version esp-device-session-reboot-request-v1 \
		--arg board_category 205 \
		--arg admitted_port "$port" \
		--arg physical_identity_digest "$physical_identity_digest" \
		--arg trusted_origin "$target_token" \
		--arg boot_session "$baseline_session" \
		--argjson boot_ordinal "$baseline_ordinal" \
		--arg source_commit "$(jq -er '.source_commit' "$manifest")" \
		--arg reference_commit "$(jq -er '.reference_commit' "$manifest")" \
		--arg app_elf_sha256 "$(jq -er '.app_elf_sha256' "$manifest")" \
		--arg hostname_sha256 "$(sha256_text "$mutated_setting")" \
		'{schema_version:$schema_version,board_category:$board_category,admitted_port:$admitted_port,physical_identity_digest:$physical_identity_digest,trusted_origin:$trusted_origin,baseline:{boot_session:$boot_session,boot_ordinal:$boot_ordinal,source_commit:$source_commit,reference_commit:$reference_commit,app_elf_sha256:$app_elf_sha256},expected_postcondition:{hostname_sha256:$hostname_sha256}}' \
		>"$request_input" || return 1
	chmod 600 "$request_input"
	mkdir "$session_root" || return 1
	chmod 700 "$session_root"
	: >"$stdout_file"
	: >"$stderr_file"
	chmod 600 "$stdout_file" "$stderr_file"

	local session_status=0
	set +e
	"$device_session_executable" reboot \
		--private-root "$session_root" \
		--request-input "$request_input" \
		--projection-output "$projection" \
		--timeout-seconds "$capture_timeout_seconds" \
		>"$stdout_file" 2>"$stderr_file"
	session_status=$?
	set -e
	[[ -f "$projection" && ! -L "$projection" ]] || {
		failure_category="device_session_projection_invalid"
		return 1
	}
	chmod 600 "$projection"
	local terminal_category
	terminal_category="$(jq -er 'select(.schema_version == "esp-device-session-v1" and (.terminal_category | type == "string" and test("^[a-z0-9_]+$"))) | .terminal_category' "$projection")" || {
		failure_category="device_session_projection_invalid"
		return 1
	}
	device_session_schema="esp-device-session-v1"
	device_session_category="$terminal_category"
	if ((session_status != 0)) || [[ "$terminal_category" != "ready" ]]; then
		failure_category="$terminal_category"
		return 1
	fi

	local private_result="$session_root/result.private.json"
	[[ -f "$private_result" && ! -L "$private_result" ]] || {
		failure_category="device_session_result_invalid"
		return 1
	}
	chmod 600 "$private_result"
	local boot_b_setup="$local_root/raw/boot-b-setup.json"
	jq -e \
		--arg expected_origin "$target_token" \
		--arg baseline_session "$baseline_session" \
		--argjson expected_ordinal "$((baseline_ordinal + 1))" \
		--arg expected_source "$(jq -er '.source_commit' "$manifest")" \
		--arg expected_reference "$(jq -er '.reference_commit' "$manifest")" \
		--arg expected_app_elf "$(jq -er '.app_elf_sha256' "$manifest")" \
		--arg expected_hostname_sha256 "$(sha256_text "$mutated_setting")" \
		'if (.schema_version == "esp-device-session-private-result-v1") and (.terminal_category == "ready") and (.boot_b.trusted_origin == $expected_origin) and (.boot_b.boot_session | type == "string" and length > 0 and . != $baseline_session) and (.boot_b.boot_ordinal == $expected_ordinal) and (.boot_b.reset_reason_category == "software_cpu") and (.boot_b.source_commit == $expected_source) and (.boot_b.reference_commit == $expected_reference) and (.boot_b.app_elf_sha256 == $expected_app_elf) and (.boot_b.hostname_sha256 == $expected_hostname_sha256) then {status:"passed",category:"none",session:.boot_b.boot_session,boot_ordinal:.boot_b.boot_ordinal,device_url:.boot_b.trusted_origin,reset_reason:.boot_b.reset_reason_category} else empty end' \
		"$private_result" >"$boot_b_setup" || {
		chmod 600 "$boot_b_setup"
		failure_category="device_session_result_invalid"
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
