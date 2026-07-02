#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly smoke_script="${PHASE17_LIVE_HTTP_API_SMOKE_SCRIPT:-${script_dir}/phase17-live-http-api-smoke.sh}"
readonly websocket_script="${PHASE17_WEBSOCKET_CAPTURE_SCRIPT:-${script_dir}/phase17-websocket-capture.mjs}"
readonly node_bin="${NODE_BIN:-node}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase17-live-http-api-smoke-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_manifest() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "source_commit": "26a1aebad7a11234567890123456789012345678",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_flash_json() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "command_kind": "flash-monitor",
  "board": "205",
  "selected_port": "/dev/cu.usbmodem1101",
  "trusted_output": true,
  "firmware_commit": "26a1aebad7a11234567890123456789012345678",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "observed_firmware_commit": "26a1aebad7a1",
  "observed_reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_stale_flash_json() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "command_kind": "flash-monitor",
  "board": "205",
  "selected_port": "/dev/cu.usbmodem1101",
  "trusted_output": true,
  "firmware_commit": "stale-source-commit",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "observed_firmware_commit": "stale-source",
  "observed_reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_no_curl_stub() {
	local path="$1"

	write_executable "$path" 'printf "curl should not have been called\n" >&2
exit 97
'
}

create_fake_curl() {
	local path="$1"

	write_executable "$path" 'header_file=""
body_file=""
method="GET"
url=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dump-header)
      header_file="$2"
      shift 2
      ;;
    --output)
      body_file="$2"
      shift 2
      ;;
    --request)
      method="$2"
      shift 2
      ;;
    --data-binary)
      shift 2
      ;;
    --silent | --show-error)
      shift
      ;;
    --max-time | --write-out)
      shift 2
      ;;
    http://* | https://*)
      url="$1"
      shift
      ;;
    *)
      printf "unexpected curl arg: %s\n" "$1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$header_file" || -z "$body_file" || -z "$url" ]]; then
  printf "missing curl fixture inputs\n" >&2
  exit 2
fi
if [[ "${PHASE17_FAKE_CURL_STDERR_HOST:-0}" == "1" ]]; then
  printf "curl: (6) Could not resolve host: private-bitaxe.local\ncurl: (7) Failed to connect to 192.168.1.5 port 80\ncurl: (22) URL rejected: http://private-bitaxe.local/api/system/info?token=abc123\n" >&2
fi

path="/${url#*://*/}"
if [[ "$url" == "http://device.local/" ]]; then
  path="/"
fi

status=200
headers="Content-Type: text/plain"
body=""

case "${method} ${path}" in
  "GET /")
    headers="Content-Type: text/html"
    body="<title>AxeOS unavailable</title><a>Open recovery</a><a>Release metadata</a>"
    ;;
  "GET /assets/app.css.gz")
    headers=$'"'"'Content-Type: text/css\nCache-Control: max-age=2592000\nContent-Encoding: gzip'"'"'
    body="body { color: #00ff00; }"
    ;;
  "GET /phase17-missing-static")
    status=302
    headers=$'"'"'Content-Type: text/plain\nLocation: /'"'"'
    body="Redirect to the captive portal"
    ;;
  "GET /recovery")
    headers="Content-Type: text/html"
    body="<h1>AxeOS Recovery</h1><span>Response:</span>"
    ;;
  "GET /api/system/info")
    headers="Content-Type: application/json"
    body="{\"boardVersion\":\"205\",\"asicModel\":\"BM1366\",\"ssid\":\"HomeNetwork\",\"wifiPass\":\"secret\",\"stratumUser\":\"worker\",\"stratumPassword\":\"pool-pass\",\"poolUrl\":\"stratum+tcp://pool.example:3333\",\"fallbackPoolUrl\":\"stratum+tcp://backup.example:3333\",\"hostname\":\"bitaxe-private\",\"ip\":\"192.168.1.5\",\"gateway\":\"192.168.1.1\",\"netmask\":\"255.255.255.0\",\"dns\":\"1.1.1.1\",\"token\":\"abc123\",\"apiKey\":\"key123\",\"password\":\"admin\",\"mac\":\"aa:bb:cc:dd:ee:ff\"}"
    ;;
  "GET /api/phase17-unknown")
    status=404
    headers="Content-Type: application/json"
    body="{\"error\":\"unknown route\"}"
    ;;
  "GET /api/ws")
    status="${PHASE17_FAKE_WS_STATUS:-426}"
    headers="Content-Type: text/plain"
    body="WebSocket upgrade required"
    ;;
  "GET /api/ws/live")
    status="${PHASE17_FAKE_WS_STATUS:-426}"
    headers="Content-Type: text/plain"
    body="WebSocket upgrade required"
    ;;
  "POST /api/system/OTA")
    status="${PHASE17_FAKE_OTA_STATUS:-500}"
    headers="Content-Type: text/plain"
    body="${PHASE17_FAKE_OTA_BODY:-Protocol Error}"
    ;;
  "POST /api/system/OTAWWW")
    status=400
    headers="Content-Type: text/plain"
    body="Wrong API input"
    ;;
  *)
    printf "unhandled fake curl route: %s %s\n" "$method" "$path" >&2
    exit 3
    ;;
esac

printf "%s\n" "$headers" >"$header_file"
printf "%s\n" "$body" >"$body_file"
printf "%s" "$status"
'
}

run_smoke() {
	local out_dir="$1"
	local manifest="$2"
	local flash_json="$3"
	local curl_bin="$4"
	shift 4

	"$BASH" "$smoke_script" \
		--manifest "$manifest" \
		--flash-evidence-json "$flash_json" \
		--out-dir "$out_dir" \
		--target-lock-out "${out_dir}/target-lock.json" \
		--curl-bin "$curl_bin" \
		"$@"
}

run_websocket_capture() {
	local out_file="$1"
	shift

	"$node_bin" "$websocket_script" --out "$out_file" "$@"
}

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

if [[ ! -f "$smoke_script" ]]; then
	fail "smoke script missing: ${smoke_script}"
fi
if [[ ! -f "$websocket_script" ]]; then
	fail "websocket script missing: ${websocket_script}"
fi
"$node_bin" --check "$websocket_script" >/dev/null

test_missing_target_blocks_without_curl
test_userinfo_path_query_fragment_rejected
test_stale_flash_identity_blocks_without_curl
test_fake_success_records_required_phase17_routes
test_no_upgrade_does_not_claim_frames
test_redacts_response_secrets
test_websocket_missing_target_blocks_with_out
test_websocket_rejects_non_origin_target
test_websocket_rejects_unsupported_path
test_websocket_live_fake_frame_passes
test_websocket_raw_log_open_timeout_stays_pending
test_websocket_rejects_bounds_over_limits

printf 'phase17_live_http_api_smoke_test passed\n'
