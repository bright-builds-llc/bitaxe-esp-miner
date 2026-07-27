#!/usr/bin/env bash
# WebSocket-specific cases sourced by the Phase 17 smoke test entrypoint.

test_websocket_missing_target_blocks_with_out() {
	# Arrange
	local out_file="${tmp_root}/websocket-missing-target.txt"

	# Act
	run_websocket_capture "$out_file" --path "/api/ws/live"

	# Assert
	assert_contains "$out_file" "phase17_websocket_capture"
	assert_contains "$out_file" "websocket_target_status=blocked - missing DEVICE_URL"
	assert_contains "$out_file" "network_scan=disabled - DEVICE_URL must be explicit"
	assert_contains "$out_file" "websocket_open_status=blocked"
	assert_contains "$out_file" "websocket_frame_status=not-run"
}

test_websocket_rejects_non_origin_target() {
	# Arrange
	local out_file="${tmp_root}/websocket-invalid-target.txt"

	# Act
	run_websocket_capture "$out_file" --device-url "http://user:pass@device.local" --path "/api/ws/live"

	# Assert
	assert_contains "$out_file" "phase17_websocket_capture"
	assert_contains "$out_file" "websocket_target_status=blocked - invalid origin-only DEVICE_URL"
	assert_contains "$out_file" "websocket_open_status=blocked"
	assert_contains "$out_file" "websocket_frame_status=not-run"
}

test_websocket_rejects_unsupported_path() {
	# Arrange
	local out_file="${tmp_root}/websocket-unsupported-path.txt"

	# Act
	run_websocket_capture "$out_file" --device-url "http://device.local" --path "/api/other"

	# Assert
	assert_contains "$out_file" "phase17_websocket_capture"
	assert_contains "$out_file" "path=/api/other"
	assert_contains "$out_file" "websocket_target_status=blocked - unsupported WebSocket path"
	assert_contains "$out_file" "websocket_open_status=blocked"
	assert_contains "$out_file" "websocket_frame_status=not-run"
}

test_websocket_live_fake_frame_passes() {
	# Arrange
	local out_file="${tmp_root}/websocket-live-frame.txt"
	local payload='{"event":"update","ssid":"HomeNetwork","wifiPass":"secret","stratumUser":"worker","stratumPassword":"pool-pass","poolUrl":"stratum+tcp://pool.example:3333","ip":"192.168.1.5","mac":"aa:bb:cc:dd:ee:ff","token":"abc123"}'

	# Act
	PHASE17_FAKE_WEBSOCKET_MODE=open-frame PHASE17_FAKE_WEBSOCKET_PAYLOAD="$payload" \
		run_websocket_capture "$out_file" --device-url "http://device.local" --path "/api/ws/live"

	# Assert
	assert_contains "$out_file" "phase17_websocket_capture"
	assert_contains "$out_file" "path=/api/ws/live"
	assert_contains "$out_file" "websocket_capture_url=ws://[redacted]/api/ws/live"
	assert_contains "$out_file" "websocket_open_status=opened"
	assert_contains "$out_file" "websocket_frame_1="
	assert_contains "$out_file" "\"ssid\":\"[redacted]\""
	assert_contains "$out_file" "\"wifiPass\":\"[redacted]\""
	assert_contains "$out_file" "\"stratumUser\":\"[redacted]\""
	assert_contains "$out_file" "\"stratumPassword\":\"[redacted]\""
	assert_contains "$out_file" "\"poolUrl\":\"[redacted]\""
	assert_contains "$out_file" "\"ip\":\"[redacted]\""
	assert_contains "$out_file" "[redacted-mac]"
	assert_contains "$out_file" "\"token\":\"[redacted]\""
	assert_contains "$out_file" "websocket_frame_status=passed frames=1"
	assert_not_contains "$out_file" "HomeNetwork"
	assert_not_contains "$out_file" "pool.example"
	assert_not_contains "$out_file" "192.168.1.5"
	assert_not_contains "$out_file" "aa:bb:cc:dd:ee:ff"
	assert_not_contains "$out_file" "abc123"
}

test_websocket_flash_evidence_device_url_fake_frame_passes() {
	# Arrange
	local out_file="${tmp_root}/websocket-flash-evidence-frame.txt"
	local flash_json="${tmp_root}/websocket-flash-evidence.json"
	local monitor_log="${tmp_root}/websocket-flash-monitor.log"

	printf 'wifi_status=connected ipv4=192.168.1.24 device_url=http://device.local\n' >"$monitor_log"
	create_flash_json_with_monitor_log "$flash_json" "$monitor_log"

	# Act
	PHASE17_FAKE_WEBSOCKET_MODE=open-frame \
		PHASE17_FAKE_WEBSOCKET_PAYLOAD='{"ssid":"HomeNetwork","wifiPass":"super-secret","ip":"192.168.1.5","mac":"aa:bb:cc:dd:ee:ff","token":"abc123"}' \
		run_websocket_capture "$out_file" --device-url-from-flash-evidence "$flash_json" --path "/api/ws/live"

	# Assert
	assert_contains "$out_file" "device_url_source=usb_flash_monitor_log"
	assert_contains "$out_file" "websocket_capture_url=ws://[redacted]/api/ws/live"
	assert_contains "$out_file" "websocket_target_status=passed"
	assert_contains "$out_file" "websocket_frame_status=passed frames=1"
	assert_not_contains "$out_file" "device.local"
	assert_not_contains "$out_file" "HomeNetwork"
	assert_not_contains "$out_file" "super-secret"
	assert_not_contains "$out_file" "192.168.1.5"
	assert_not_contains "$out_file" "aa:bb:cc:dd:ee:ff"
	assert_not_contains "$out_file" "abc123"
}

test_websocket_frame_then_error_preserves_passed_frame_status() {
	# Arrange
	local out_file="${tmp_root}/websocket-frame-error.txt"

	# Act
	PHASE17_FAKE_WEBSOCKET_MODE=frame-error PHASE17_FAKE_WEBSOCKET_PAYLOAD='{"event":"update","ip":"192.168.1.5"}' \
		run_websocket_capture "$out_file" --device-url "http://device.local" --path "/api/ws/live"

	# Assert
	assert_contains "$out_file" "websocket_open_status=opened"
	assert_contains "$out_file" "websocket_frame_1="
	assert_contains "$out_file" "\"ip\":\"[redacted]\""
	assert_contains "$out_file" "websocket_error=connection error"
	assert_contains "$out_file" "websocket_frame_status=passed frames=1"
	assert_not_contains "$out_file" "192.168.1.5"
}

test_websocket_flash_evidence_device_url_blocks_unusable_sources() {
	# Arrange
	local redacted_out="${tmp_root}/websocket-flash-evidence-redacted.txt"
	local redacted_json="${tmp_root}/websocket-flash-evidence-redacted.json"
	local redacted_log="${tmp_root}/websocket-flash-evidence-redacted.log"
	local wrong_board_out="${tmp_root}/websocket-flash-evidence-wrong-board.txt"
	local wrong_board_json="${tmp_root}/websocket-flash-evidence-wrong-board.json"
	local wrong_board_log="${tmp_root}/websocket-flash-evidence-wrong-board.log"

	printf 'wifi_status=connected device_url=[redacted-url]\n' >"$redacted_log"
	create_flash_json_with_monitor_log "$redacted_json" "$redacted_log"
	printf 'wifi_status=connected device_url=http://device.local\n' >"$wrong_board_log"
	create_flash_json_with_monitor_log "$wrong_board_json" "$wrong_board_log" "601"

	# Act
	run_websocket_capture "$redacted_out" --device-url-from-flash-evidence "$redacted_json" --path "/api/ws/live"
	run_websocket_capture "$wrong_board_out" --device-url-from-flash-evidence "$wrong_board_json" --path "/api/ws/live"

	# Assert
	assert_contains "$redacted_out" "websocket_target_status=blocked - flash log device_url unavailable - monitor log must contain exactly one device_url"
	assert_contains "$wrong_board_out" "websocket_target_status=blocked - flash log device_url unavailable - flash board is not 205"
}

test_websocket_raw_log_open_timeout_stays_pending() {
	# Arrange
	local out_file="${tmp_root}/websocket-raw-timeout.txt"

	# Act
	PHASE17_FAKE_WEBSOCKET_MODE=open-timeout \
		run_websocket_capture "$out_file" --device-url "http://device.local" --path "/api/ws" --duration-ms 25

	# Assert
	assert_contains "$out_file" "phase17_websocket_capture"
	assert_contains "$out_file" "path=/api/ws"
	assert_contains "$out_file" "websocket_open_status=opened"
	assert_contains "$out_file" "websocket_frame_status=pending - open timeout without raw log frame"
}

test_websocket_rejects_bounds_over_limits() {
	# Arrange
	local duration_out="${tmp_root}/websocket-duration-over-limit.txt"
	local frames_out="${tmp_root}/websocket-frames-over-limit.txt"

	# Act
	run_websocket_capture "$duration_out" --device-url "http://device.local" --path "/api/ws/live" --duration-ms 30001
	run_websocket_capture "$frames_out" --device-url "http://device.local" --path "/api/ws/live" --max-frames 11

	# Assert
	assert_contains "$duration_out" "websocket_target_status=blocked - duration-ms exceeds 30000"
	assert_contains "$duration_out" "websocket_frame_status=not-run"
	assert_contains "$frames_out" "websocket_target_status=blocked - max-frames exceeds 10"
	assert_contains "$frames_out" "websocket_frame_status=not-run"
}
