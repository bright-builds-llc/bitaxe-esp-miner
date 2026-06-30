#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly smoke_script="${PHASE13_FIRMWARE_OTA_SMOKE_SCRIPT:-${script_dir}/phase13-firmware-ota-smoke.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase13-firmware-ota-smoke-test.XXXXXX")"
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

sha256_fixture() {
	local path="$1"

	if command -v shasum >/dev/null 2>&1; then
		shasum -a 256 "$path" | awk '{print $1}'
		return
	fi
	sha256sum "$path" | awk '{print $1}'
}

create_inputs() {
	local root="$1"
	local image="${root}/esp-miner.bin"
	local manifest="${root}/manifest.json"

	printf 'valid firmware ota image\n' >"$image"
	local checksum
	checksum="$(sha256_fixture "$image")"

	cat >"$manifest" <<JSON
{
  "source_commit": "190849539700b8f9a7909fd2b6ebd84142557968",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "artifacts": [
    {
      "kind": "firmware_ota_image",
      "path": "esp-miner.bin",
      "offset": "0x10000",
      "sha256": "${checksum}"
    }
  ]
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
data_path=""
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
    --request | --max-time | --write-out)
      shift 2
      ;;
    --data-binary)
      data_path="${2#@}"
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
      printf "unexpected curl arg: %s\n" "$1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$header_file" || -z "$body_file" || -z "$data_path" || "$url" != "http://device.local/api/system/OTA" ]]; then
  printf "missing fake curl inputs\n" >&2
  exit 2
fi
if [[ "${PHASE13_FAKE_CURL_STDERR_HOST:-0}" == "1" ]]; then
  printf "curl: (6) Could not resolve host: private-bitaxe.local\ncurl: (7) Failed to connect to private-bitaxe.local port 80\ncurl: (22) URL rejected: http://private-bitaxe.local/api/system/OTA\n" >&2
fi

printf "Content-Type: text/plain\n" >"$header_file"
case "$(basename "$data_path")" in
  invalid-firmware.bin)
    case "${PHASE13_FAKE_INVALID_RESPONSE:-validation-error}" in
      validation-error)
        printf "Validation / Activation Error {\"ssid\":\"phase13-secret\",\"stratumCert\":\"PHASE13_LONG_OTA_SECRET_PREFIX_%s\",\"ip\":\"192.168.1.50\"}" "$(printf "x%.0s" {1..260})" >"$body_file"
        printf "500"
        ;;
      unrelated-non-200)
        printf "unrelated proxy response" >"$body_file"
        printf "502"
        ;;
      *)
        printf "unknown fake invalid response: %s\n" "$PHASE13_FAKE_INVALID_RESPONSE" >&2
        exit 2
        ;;
    esac
    ;;
  esp-miner.bin)
    printf "Firmware update complete, rebooting now!" >"$body_file"
    printf "200"
    ;;
  *)
    printf "unexpected upload image: %s\n" "$data_path" >&2
    exit 3
    ;;
esac
'
}

create_success_monitor() {
	local path="$1"

	write_executable "$path" 'out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      out="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
if [[ -z "$out" ]]; then
  printf "missing monitor out\n" >&2
  exit 2
fi
printf "firmware_commit=190849539700\nreference_commit=c1915b0a63bf\nota_boot_validation=marked_valid\ncapture_status=completed\n" >"$out"
'
}

create_missing_marker_monitor() {
	local path="$1"

	write_executable "$path" 'out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      out="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
printf "firmware_commit=190849539700\nreference_commit=c1915b0a63bf\ncapture_status=completed\n" >"$out"
'
}

test_missing_url_writes_blocker_without_curl() {
	local out_dir="${tmp_root}/missing-url"
	local curl_stub="${tmp_root}/no-curl"

	create_no_curl_stub "$curl_stub"

	CURL_BIN="$curl_stub" "$BASH" "$smoke_script" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--out-dir "$out_dir"

	local log_file="${out_dir}/firmware-ota-smoke.log"
	assert_contains "$log_file" "phase13_firmware_ota_smoke"
	assert_contains "$log_file" "DEVICE_URL status: blocked - DEVICE_URL unavailable"
	assert_contains "$log_file" "firmware_ota_status: blocked - DEVICE_URL unavailable"
	assert_contains "$log_file" "network_scan: disabled - DEVICE_URL must be explicit"
}

test_fake_invalid_rejection_and_valid_success_records_evidence() {
	local out_dir="${tmp_root}/fake-success"
	local curl_stub="${tmp_root}/fake-curl"
	local monitor_stub="${tmp_root}/success-monitor"

	create_fake_curl "$curl_stub"
	create_success_monitor "$monitor_stub"

	PHASE13_FAKE_CURL_STDERR_HOST=1 CURL_BIN="$curl_stub" PHASE13_MONITOR_CAPTURE_SCRIPT="$monitor_stub" "$BASH" "$smoke_script" \
		--device-url "http://device.local" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--monitor-seconds 1

	local log_file="${out_dir}/firmware-ota-smoke.log"
	assert_contains "$log_file" "DEVICE_URL status: provided"
	assert_contains "$log_file" "network_scan: disabled - using explicit DEVICE_URL only"
	assert_contains "$log_file" "firmware_ota_manifest_artifact: esp-miner.bin"
	assert_contains "$log_file" "invalid image rejection route: POST /api/system/OTA"
	assert_contains "$log_file" "invalid image rejection status: 500"
	assert_contains "$log_file" "invalid image rejection body: Validation / Activation Error"
	assert_contains "$log_file" "\"ssid\":\"[redacted]\""
	assert_contains "$log_file" "\"stratumCert\":\"[redacted]\""
	assert_contains "$log_file" "\"ip\":\"[redacted]\""
	assert_contains "$log_file" "curl_error:"
	assert_contains "$log_file" "Could not resolve host: [redacted-host]"
	assert_contains "$log_file" "Failed to connect to [redacted-host]"
	assert_contains "$log_file" "[redacted-url]"
	assert_not_contains "$log_file" "phase13-secret"
	assert_not_contains "$log_file" "PHASE13_LONG_OTA_SECRET_PREFIX"
	assert_not_contains "$log_file" "private-bitaxe.local"
	assert_not_contains "$log_file" "http://private-bitaxe.local"
	assert_not_contains "$log_file" "192.168.1.50"
	assert_contains "$log_file" "invalid image rejection conclusion: captured - not rollback proof"
	assert_contains "$log_file" "invalid image rejection is not rollback proof"
	assert_contains "$log_file" "valid OTA route: POST /api/system/OTA"
	assert_contains "$log_file" "valid OTA status: 200"
	assert_contains "$log_file" "valid OTA body: Firmware update complete, rebooting now!"
	assert_contains "$log_file" "post_ota_marker: firmware_commit= present"
	assert_contains "$log_file" "post_ota_marker: reference_commit= present"
	assert_contains "$log_file" "post_ota_marker: ota_boot_validation= present"
	assert_contains "$log_file" "firmware_ota_status: passed"
	assert_contains "${out_dir}/post-ota-monitor.log" "ota_boot_validation=marked_valid"
	assert_not_contains "$log_file" "http://device.local"
}

test_missing_post_ota_marker_blocks_passed_status() {
	local out_dir="${tmp_root}/missing-marker"
	local curl_stub="${tmp_root}/fake-curl-marker"
	local monitor_stub="${tmp_root}/missing-marker-monitor"

	create_fake_curl "$curl_stub"
	create_missing_marker_monitor "$monitor_stub"

	set +e
	CURL_BIN="$curl_stub" PHASE13_MONITOR_CAPTURE_SCRIPT="$monitor_stub" "$BASH" "$smoke_script" \
		--device-url "http://device.local" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--monitor-seconds 1
	local status=$?
	set -e

	if [[ "$status" -eq 0 ]]; then
		fail "missing post-OTA marker should fail the helper"
	fi

	local log_file="${out_dir}/firmware-ota-smoke.log"
	assert_contains "$log_file" "post_ota_marker: ota_boot_validation= missing"
	assert_contains "$log_file" "firmware_ota_status: blocked - post-OTA monitor missing required identity or boot-validation markers"
	assert_not_contains "$log_file" "firmware_ota_status: passed"
}

test_invalid_rejection_blocks_without_validation_marker() {
	local out_dir="${tmp_root}/invalid-no-marker"
	local curl_stub="${tmp_root}/fake-curl-invalid-no-marker"
	local monitor_stub="${tmp_root}/success-monitor-invalid-no-marker"

	create_fake_curl "$curl_stub"
	create_success_monitor "$monitor_stub"

	set +e
	PHASE13_FAKE_INVALID_RESPONSE=unrelated-non-200 CURL_BIN="$curl_stub" PHASE13_MONITOR_CAPTURE_SCRIPT="$monitor_stub" "$BASH" "$smoke_script" \
		--device-url "http://device.local" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--monitor-seconds 1
	local status=$?
	set -e

	if [[ "$status" -eq 0 ]]; then
		fail "unrelated non-200 invalid image response should block evidence"
	fi

	local log_file="${out_dir}/firmware-ota-smoke.log"
	assert_contains "$log_file" "invalid image rejection status: 502"
	assert_contains "$log_file" "invalid image rejection body: unrelated proxy response"
	assert_contains "$log_file" "firmware_ota_status: blocked - invalid image rejection body did not contain an OTA validation marker"
	assert_not_contains "$log_file" "invalid image rejection conclusion: captured"
	assert_not_contains "$log_file" "firmware_ota_status: passed"
}

if [[ ! -f "$smoke_script" ]]; then
	fail "smoke script missing: ${smoke_script}"
fi

create_inputs "$tmp_root"
test_missing_url_writes_blocker_without_curl
test_fake_invalid_rejection_and_valid_success_records_evidence
test_missing_post_ota_marker_blocks_passed_status
test_invalid_rejection_blocks_without_validation_marker

printf 'phase13_firmware_ota_smoke_test passed\n'
