#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE14_LIVE_TELEMETRY_SCRIPT:-${script_dir}/phase14-live-telemetry.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase14-live-telemetry-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

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

write_fake_allow() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

status="${PHASE14_FAKE_ALLOW_STATUS:-passed}"
printf 'fake_allow_args: %s\n' "$*"
case "$status" in
passed)
	printf 'safety_allow_status: passed\n'
	;;
failed)
	printf 'safety_allow_status: failed\n'
	printf 'validation_errors:\n- fake failure\n'
	exit 42
	;;
*)
	printf 'unknown fake status\n' >&2
	exit 2
	;;
esac
SH
	chmod +x "$path"
}

write_manifest() {
	local path="$1"

	printf '{"board":"205"}\n' >"$path"
}

write_fake_curl() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE14_CURL_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'curl should not have been called\n' >&2
	exit 99
fi

header_file=""
body_file=""
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
	--write-out | --max-time)
		shift 2
		;;
	--silent | --show-error)
		shift
		;;
	http://* | https://*)
		url="$1"
		shift
		;;
	*)
		shift
		;;
	esac
done

if [[ -z "$header_file" || -z "$body_file" || -z "$url" ]]; then
	printf 'bad fake curl invocation\n' >&2
	exit 2
fi

printf 'Content-Type: application/json\r\nCache-Control: no-store\r\n' >"$header_file"

case "$url" in
*/api/system/info)
	cat >"$body_file" <<'JSON'
{"ssid":"HomeWifi","wifiPass":"super-secret","poolUrl":"stratum+tcp://pool.example.com","ip":"192.168.1.22","mac":"aa:bb:cc:dd:ee:ff","note":"peer 10.0.0.5","power":0.0,"voltage":0.0,"temp":0.0,"fanrpm":0,"safetyStatus":"hardware_evidence_pending"}
JSON
	printf '200'
	;;
*/api/ws/live)
	printf '{"error":"upgrade required"}\n' >"$body_file"
	printf '%s' "${PHASE14_FAKE_WS_LIVE_STATUS:-426}"
	;;
*/api/ws)
	printf '{"error":"upgrade required"}\n' >"$body_file"
	printf '%s' "${PHASE14_FAKE_WS_STATUS:-426}"
	;;
*)
	printf '{"error":"not found"}\n' >"$body_file"
	printf '404'
	;;
esac
SH
	chmod +x "$path"
}

test_missing_device_url_blocks_without_curl() {
	local out_dir="${tmp_root}/missing-url"
	local manifest="${tmp_root}/manifest.json"
	local fake_allow="${tmp_root}/fake-allow"
	local fake_curl="${tmp_root}/fake-curl"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" PHASE14_CURL_MUST_NOT_RUN=1 "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--curl-bin "$fake_curl"

	assert_contains "${out_dir}/live-telemetry.log" "DEVICE_URL status: blocked - missing DEVICE_URL"
	assert_contains "${out_dir}/live-telemetry.log" "api_telemetry_status: blocked"
	assert_contains "${out_dir}/live-telemetry.log" "websocket_frame_status: pending - maintained WebSocket client unavailable"
}

test_successful_system_info_redacts_secrets() {
	local out_dir="${tmp_root}/redaction"
	local manifest="${tmp_root}/manifest-redaction.json"
	local fake_allow="${tmp_root}/fake-allow-redaction"
	local fake_curl="${tmp_root}/fake-curl-redaction"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--device-url "http://bitaxe.local" \
		--curl-bin "$fake_curl"

	assert_contains "${out_dir}/live-telemetry.log" "DEVICE_URL status: provided"
	assert_contains "${out_dir}/live-telemetry.log" "route: GET /api/system/info"
	assert_contains "${out_dir}/live-telemetry.log" "system_info_status_code: 200"
	assert_contains "${out_dir}/live-telemetry.log" "safety_telemetry_fields: observed"
	assert_contains "${out_dir}/live-telemetry.log" "[redacted]"
	assert_contains "${out_dir}/live-telemetry.log" "[redacted-ip]"
	assert_contains "${out_dir}/live-telemetry.log" "[redacted-mac]"
	assert_not_contains "${out_dir}/live-telemetry.log" "HomeWifi"
	assert_not_contains "${out_dir}/live-telemetry.log" "super-secret"
	assert_not_contains "${out_dir}/live-telemetry.log" "pool.example.com"
	assert_not_contains "${out_dir}/live-telemetry.log" "192.168.1.22"
	assert_not_contains "${out_dir}/live-telemetry.log" "10.0.0.5"
	assert_not_contains "${out_dir}/live-telemetry.log" "aa:bb:cc:dd:ee:ff"
}

test_websocket_no_upgrade_route_status_stays_frame_pending() {
	local out_dir="${tmp_root}/websocket-no-upgrade"
	local manifest="${tmp_root}/manifest-ws.json"
	local fake_allow="${tmp_root}/fake-allow-ws"
	local fake_curl="${tmp_root}/fake-curl-ws"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--device-url "http://bitaxe.local" \
		--curl-bin "$fake_curl"

	assert_contains "${out_dir}/live-telemetry.log" "route: GET /api/ws"
	assert_contains "${out_dir}/live-telemetry.log" "api_ws_status_code: 426"
	assert_contains "${out_dir}/live-telemetry.log" "route: GET /api/ws/live"
	assert_contains "${out_dir}/live-telemetry.log" "api_ws_live_status_code: 426"
	assert_contains "${out_dir}/live-telemetry.log" "websocket_frame_status: pending - maintained WebSocket client unavailable"
	assert_not_contains "${out_dir}/live-telemetry.log" "websocket_frame_status: passed"
}

test_failing_websocket_status_is_recorded_without_frame_claim() {
	local out_dir="${tmp_root}/websocket-failure"
	local manifest="${tmp_root}/manifest-ws-fail.json"
	local fake_allow="${tmp_root}/fake-allow-ws-fail"
	local fake_curl="${tmp_root}/fake-curl-ws-fail"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" PHASE14_FAKE_WS_LIVE_STATUS=500 "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--device-url "http://bitaxe.local" \
		--curl-bin "$fake_curl"

	assert_contains "${out_dir}/live-telemetry.log" "api_ws_live_status_code: 500"
	assert_contains "${out_dir}/live-telemetry.log" "api_ws_live_route_conclusion: blocked"
	assert_contains "${out_dir}/live-telemetry.log" "websocket_frame_non_claim: route status is not frame-level cadence proof"
}

test_missing_device_url_blocks_without_curl
test_successful_system_info_redacts_secrets
test_websocket_no_upgrade_route_status_stays_frame_pending
test_failing_websocket_status_is_recorded_without_frame_claim

printf 'phase14-live-telemetry tests passed\n'
