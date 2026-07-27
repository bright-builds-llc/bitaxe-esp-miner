#!/usr/bin/env bash
# HTTP target and response cases for the Phase 17 smoke test.

test_missing_target_blocks_without_curl() {
	# Arrange
	local out_dir="${tmp_root}/missing-url"
	local manifest="${tmp_root}/manifest.json"
	local flash_json="${tmp_root}/flash.json"
	local curl_stub="${tmp_root}/no-curl"

	create_manifest "$manifest"
	create_flash_json "$flash_json"
	create_no_curl_stub "$curl_stub"

	# Act
	run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub"

	# Assert
	local log_file="${out_dir}/http-static-api.log"
	assert_contains "$log_file" "phase17_live_http_api_smoke"
	assert_contains "$log_file" "DEVICE_URL status: blocked - missing DEVICE_URL"
	assert_contains "$log_file" "network_scan: disabled"
	assert_contains "$log_file" "http_static_api_status: blocked"
}

test_userinfo_path_query_fragment_rejected() {
	# Arrange
	local manifest="${tmp_root}/manifest-invalid-url.json"
	local flash_json="${tmp_root}/flash-invalid-url.json"
	local curl_stub="${tmp_root}/no-curl-invalid"
	local invalid_urls=(
		"ftp://device.local"
		"http://user:pass@device.local"
		"http://device.local/path"
		"http://device.local?x=1"
		"http://device.local#frag"
	)

	create_manifest "$manifest"
	create_flash_json "$flash_json"
	create_no_curl_stub "$curl_stub"

	# Act + Assert
	local invalid_url
	for invalid_url in "${invalid_urls[@]}"; do
		local safe_name
		safe_name="$(printf '%s' "$invalid_url" | LC_ALL=C tr -c '[:alnum:]' '-')"
		local out_dir="${tmp_root}/invalid-${safe_name}"

		run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --device-url "$invalid_url"

		local log_file="${out_dir}/http-static-api.log"
		assert_contains "$log_file" "DEVICE_URL status: blocked - invalid origin-only DEVICE_URL"
		assert_contains "$log_file" "network_scan: disabled"
		assert_contains "$log_file" "http_static_api_status: blocked"
	done
}

test_stale_flash_identity_blocks_without_curl() {
	# Arrange
	local out_dir="${tmp_root}/stale-identity"
	local manifest="${tmp_root}/manifest.json"
	local flash_json="${tmp_root}/flash-stale.json"
	local curl_stub="${tmp_root}/no-curl-stale"

	create_manifest "$manifest"
	create_stale_flash_json "$flash_json"
	create_no_curl_stub "$curl_stub"

	# Act
	run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --device-url "http://device.local"

	# Assert
	local log_file="${out_dir}/http-static-api.log"
	assert_contains "$log_file" "identity_status: blocked"
	assert_contains "$log_file" "network_scan: disabled"
	assert_contains "$log_file" "http_static_api_status: blocked"
}

test_fake_success_records_required_phase17_routes() {
	# Arrange
	local out_dir="${tmp_root}/fake-success"
	local manifest="${tmp_root}/manifest.json"
	local flash_json="${tmp_root}/flash.json"
	local curl_stub="${tmp_root}/fake-curl"

	create_manifest "$manifest"
	create_flash_json "$flash_json"
	create_fake_curl "$curl_stub"

	# Act
	run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --device-url "http://device.local"

	# Assert
	local log_file="${out_dir}/http-static-api.log"
	assert_contains "$log_file" "DEVICE_URL status: provided"
	assert_contains "$log_file" "network_scan: disabled"
	assert_contains "$log_file" "identity_status: passed"
	assert_contains "$log_file" "route: GET /"
	assert_contains "$log_file" "route: GET /assets/app.css.gz"
	assert_contains "$log_file" "route: GET /phase17-missing-static"
	assert_contains "$log_file" "route: GET /recovery"
	assert_contains "$log_file" "route: GET /api/system/info"
	assert_contains "$log_file" "route: GET /api/phase17-unknown"
	assert_contains "$log_file" "route: GET /api/ws"
	assert_contains "$log_file" "route: GET /api/ws/live"
	assert_contains "$log_file" "route: POST /api/system/OTA"
	assert_contains "$log_file" "route: POST /api/system/OTAWWW"
	assert_contains "$log_file" "system_info_device_marker: passed"
	assert_contains "$log_file" "websocket_no_upgrade_claim: route-coexistence-only"
	assert_contains "$log_file" "ota_route_presence_claim: route-presence-only"
	assert_contains "$log_file" "ota_non_claims: valid OTA upload, invalid image rejection, reboot, rollback, selected partition, boot validation not claimed"
	assert_contains "$log_file" "otawww_rel03_status: deferred"
	assert_contains "$log_file" "Wrong API input"
	assert_contains "$log_file" "http_static_api_status: passed"
	assert_contains "${out_dir}/target-lock.json" "\"target_status\": \"passed\""
	assert_contains "${out_dir}/target-lock.json" "\"device_url_redacted\": \"http://[redacted]\""
	assert_contains "${out_dir}/target-lock.json" "\"network_scan\": \"disabled\""
	assert_contains "${out_dir}/target-lock.json" "\"created_from_explicit_input\": true"
	assert_not_contains "${out_dir}/target-lock.json" "device.local"
}

test_flash_log_device_url_success_records_usb_source_without_raw_target_lock() {
	# Arrange
	local out_dir="${tmp_root}/flash-log-url-success"
	local manifest="${tmp_root}/manifest-flash-log.json"
	local flash_json="${tmp_root}/flash-log.json"
	local monitor_log="${tmp_root}/flash-monitor.log"
	local curl_stub="${tmp_root}/fake-curl-flash-log"

	create_manifest "$manifest"
	printf '\377\376wifi_status=connected ipv4=192.168.1.24 device_url=http://device.local\n' >"$monitor_log"
	create_flash_json_with_monitor_log "$flash_json" "$monitor_log" "205" "true" "flash-monitor" "port"
	create_fake_curl "$curl_stub"

	# Act
	run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --use-flash-log-device-url

	# Assert
	local log_file="${out_dir}/http-static-api.log"
	assert_contains "$log_file" "DEVICE_URL status: provided"
	assert_contains "$log_file" "DEVICE_URL source: usb_flash_monitor_log"
	assert_contains "$log_file" "network_scan: disabled"
	assert_contains "$log_file" "http_static_api_status: passed"
	assert_contains "${out_dir}/target-lock.json" "\"device_url_source\": \"usb_flash_monitor_log\""
	assert_contains "${out_dir}/target-lock.json" "\"device_url_redacted\": \"http://[redacted]\""
	assert_contains "${out_dir}/target-lock.json" "\"selected_port\": \"/dev/cu.usbmodem1101\""
	assert_not_contains "${out_dir}/target-lock.json" "device.local"
}

test_flash_log_device_url_blocks_untrusted_or_unusable_sources() {
	# Arrange
	local manifest="${tmp_root}/manifest-flash-log-blocks.json"
	local curl_stub="${tmp_root}/no-curl-flash-log-blocks"

	create_manifest "$manifest"
	create_no_curl_stub "$curl_stub"

	local scenarios=(
		"wrong-board|601|true|flash-monitor|device_url=http://device.local|flash board is not 205"
		"untrusted|205|false|flash-monitor|device_url=http://device.local|flash trusted_output is not true"
		"wrong-kind|205|true|flash|device_url=http://device.local|flash command_kind is not flash-monitor"
		"redacted-log|205|true|flash-monitor|device_url=[redacted-url]|monitor log must contain exactly one device_url"
		"invalid-url|205|true|flash-monitor|device_url=http://device.local/path|monitor log device_url is not origin-only"
		"multiple-urls|205|true|flash-monitor|device_url=http://device.local%0Adevice_url=http://other.local|monitor log must contain exactly one device_url"
	)

	# Act + Assert
	local scenario
	for scenario in "${scenarios[@]}"; do
		IFS='|' read -r name board trusted kind log_payload expected_reason <<<"$scenario"
		local out_dir="${tmp_root}/flash-log-${name}"
		local flash_json="${tmp_root}/flash-log-${name}.json"
		local monitor_log="${tmp_root}/flash-log-${name}.log"

		printf '%b\n' "${log_payload//%0A/\\n}" >"$monitor_log"
		create_flash_json_with_monitor_log "$flash_json" "$monitor_log" "$board" "$trusted" "$kind"

		run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --use-flash-log-device-url

		local log_file="${out_dir}/http-static-api.log"
		assert_contains "$log_file" "DEVICE_URL status: blocked - flash log device_url unavailable"
		assert_contains "$log_file" "device_url_lookup_reason: ${expected_reason}"
		assert_contains "$log_file" "network_scan: disabled"
		assert_contains "$log_file" "http_static_api_status: blocked"
	done

	local missing_log_out="${tmp_root}/flash-log-missing-log"
	local missing_flash_json="${tmp_root}/flash-log-missing-log.json"
	create_flash_json_with_monitor_log "$missing_flash_json" "${tmp_root}/absent-flash-monitor.log"
	run_smoke "$missing_log_out" "$manifest" "$missing_flash_json" "$curl_stub" --use-flash-log-device-url
	assert_contains "${missing_log_out}/http-static-api.log" "device_url_lookup_reason: monitor log path is missing or unreadable"
}

test_no_upgrade_does_not_claim_frames() {
	# Arrange
	local out_dir="${tmp_root}/websocket-no-upgrade"
	local manifest="${tmp_root}/manifest-ws.json"
	local flash_json="${tmp_root}/flash-ws.json"
	local curl_stub="${tmp_root}/fake-curl-ws"

	create_manifest "$manifest"
	create_flash_json "$flash_json"
	create_fake_curl "$curl_stub"

	# Act
	PHASE17_FAKE_WS_STATUS=400 run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --device-url "http://device.local"

	# Assert
	local log_file="${out_dir}/http-static-api.log"
	assert_contains "$log_file" "route: GET /api/ws"
	assert_contains "$log_file" "route: GET /api/ws/live"
	assert_contains "$log_file" "websocket_no_upgrade_claim: route-coexistence-only"
	assert_not_contains "$log_file" "websocket_frame_status: passed"
	assert_contains "$log_file" "http_static_api_status: passed"
}

test_redacts_response_secrets() {
	# Arrange
	local out_dir="${tmp_root}/redaction"
	local manifest="${tmp_root}/manifest-redaction.json"
	local flash_json="${tmp_root}/flash-redaction.json"
	local curl_stub="${tmp_root}/fake-curl-redaction"

	create_manifest "$manifest"
	create_flash_json "$flash_json"
	create_fake_curl "$curl_stub"

	# Act
	PHASE17_FAKE_CURL_STDERR_HOST=1 run_smoke "$out_dir" "$manifest" "$flash_json" "$curl_stub" --device-url "http://device.local"

	# Assert
	local log_file="${out_dir}/http-static-api.log"
	assert_contains "$log_file" "\"ssid\":\"[redacted]\""
	assert_contains "$log_file" "\"wifiPass\":\"[redacted]\""
	assert_contains "$log_file" "\"stratumUser\":\"[redacted]\""
	assert_contains "$log_file" "\"stratumPassword\":\"[redacted]\""
	assert_contains "$log_file" "\"poolUrl\":\"[redacted]\""
	assert_contains "$log_file" "\"fallbackPoolUrl\":\"[redacted]\""
	assert_contains "$log_file" "\"hostname\":\"[redacted]\""
	assert_contains "$log_file" "\"ip\":\"[redacted]\""
	assert_contains "$log_file" "\"gateway\":\"[redacted]\""
	assert_contains "$log_file" "\"netmask\":\"[redacted]\""
	assert_contains "$log_file" "\"dns\":\"[redacted]\""
	assert_contains "$log_file" "\"token\":\"[redacted]\""
	assert_contains "$log_file" "\"apiKey\":\"[redacted]\""
	assert_contains "$log_file" "\"password\":\"[redacted]\""
	assert_contains "$log_file" "[redacted-mac]"
	assert_contains "$log_file" "Could not resolve host: [redacted-host]"
	assert_contains "$log_file" "Failed to connect to [redacted-ip]"
	assert_contains "$log_file" "[redacted-url]"
	assert_not_contains "$log_file" "HomeNetwork"
	assert_not_contains "$log_file" "pool-pass"
	assert_not_contains "$log_file" "pool.example"
	assert_not_contains "$log_file" "backup.example"
	assert_not_contains "$log_file" "bitaxe-private"
	assert_not_contains "$log_file" "192.168.1.5"
	assert_not_contains "$log_file" "aa:bb:cc:dd:ee:ff"
	assert_not_contains "$log_file" "abc123"
	assert_not_contains "$log_file" "key123"
	assert_not_contains "$log_file" "admin"
	assert_not_contains "$log_file" "private-bitaxe.local"
}
