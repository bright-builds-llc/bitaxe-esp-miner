#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper_script="${PHASE18_FIRMWARE_OTA_EVIDENCE_SCRIPT:-${script_dir}/phase18-firmware-ota-evidence.sh}"

tmp_parent="target/phase18-firmware-ota-and-rollback-evidence-dev-raw"
mkdir -p "$tmp_parent"
tmp_root="$(mktemp -d "${tmp_parent}/test.XXXXXX")"
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

	printf 'valid phase18 firmware ota image\n' >"$image"
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

printf "Content-Type: text/plain\n" >"$header_file"
case "$(basename "$data_path")" in
  invalid-firmware.bin)
    printf "Validation / Activation Error {\"ssid\":\"phase18-secret\",\"stratumPassword\":\"PHASE18_SECRET\",\"ip\":\"192.168.1.77\"}" >"$body_file"
    printf "500"
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

create_missing_boot_validation_monitor() {
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

create_flash_evidence() {
	local path="$1"
	local monitor_log="$2"
	local board="$3"
	local trusted="$4"

	cat >"$path" <<JSON
{
  "command_kind": "flash-monitor",
  "board": "${board}",
  "trusted_output": ${trusted},
  "selected_port": "/dev/cu.test",
  "monitor_log_path": "${monitor_log}"
}
JSON
}

firmware_log() {
	local out_dir="$1"

	printf '%s\n' "${out_dir}/firmware-ota/firmware-ota-smoke.log"
}

test_missing_target_writes_blocked_log_without_curl() {
	local out_dir="${tmp_root}/missing-target"
	local no_curl="${tmp_root}/no-curl"

	create_no_curl_stub "$no_curl"

	CURL_BIN="$no_curl" "$BASH" "$wrapper_script" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir"

	local log_file
	log_file="$(firmware_log "$out_dir")"
	assert_contains "$log_file" "phase18_firmware_ota_evidence"
	assert_contains "$log_file" "DEVICE_URL status: blocked - missing DEVICE_URL"
	assert_contains "$log_file" "network_scan: disabled"
	assert_contains "$log_file" "firmware_ota_status: blocked - DEVICE_URL unavailable"
	assert_not_contains "$log_file" "phase13_firmware_ota_smoke"
}

test_invalid_direct_url_blocks_without_curl() {
	local out_dir="${tmp_root}/invalid-direct-url"
	local no_curl="${tmp_root}/no-curl-invalid"

	create_no_curl_stub "$no_curl"

	CURL_BIN="$no_curl" "$BASH" "$wrapper_script" \
		--device-url "http://device.local/path" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir"

	local log_file
	log_file="$(firmware_log "$out_dir")"
	assert_contains "$log_file" "DEVICE_URL status: blocked - invalid origin-only DEVICE_URL"
	assert_contains "$log_file" "DEVICE_URL sanitized: http://[redacted]"
	assert_contains "$log_file" "firmware_ota_status: blocked - DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, or fragment"
	assert_not_contains "$log_file" "phase13_firmware_ota_smoke"
}

test_trusted_flash_evidence_creates_redacted_target_lock() {
	local out_dir="${tmp_root}/trusted-flash"
	local monitor_log="${tmp_root}/trusted-monitor.log"
	local flash_json="${tmp_root}/flash-command-evidence.json"
	local no_curl="${tmp_root}/no-curl-flash"

	printf 'device_url=http://device.local\n' >"$monitor_log"
	create_flash_evidence "$flash_json" "$monitor_log" "205" "true"
	create_no_curl_stub "$no_curl"

	CURL_BIN="$no_curl" "$BASH" "$wrapper_script" \
		--device-url-from-flash-evidence "$flash_json" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--target-lock-only

	local target_lock="${out_dir}/target-lock.json"
	local log_file
	log_file="$(firmware_log "$out_dir")"
	assert_contains "$log_file" "DEVICE_URL source: usb_flash_monitor_log"
	assert_contains "$log_file" "firmware_ota_status: target-lock-only - OTA helper not invoked"
	assert_contains "$target_lock" '"target_status": "passed"'
	assert_contains "$target_lock" '"device_url_source": "usb_flash_monitor_log"'
	assert_contains "$target_lock" '"device_url_redacted": "http://[redacted]"'
	assert_contains "$target_lock" '"network_scan": "disabled"'
	assert_contains "$target_lock" '"created_from_explicit_input": true'
	assert_not_contains "$target_lock" "http://device.local"
}

test_untrusted_or_wrong_board_flash_evidence_blocks() {
	local monitor_log="${tmp_root}/wrong-board-monitor.log"
	local wrong_board_json="${tmp_root}/wrong-board.json"
	local untrusted_json="${tmp_root}/untrusted.json"
	local no_curl="${tmp_root}/no-curl-untrusted"

	printf 'device_url=http://device.local\n' >"$monitor_log"
	create_flash_evidence "$wrong_board_json" "$monitor_log" "601" "true"
	create_flash_evidence "$untrusted_json" "$monitor_log" "205" "false"
	create_no_curl_stub "$no_curl"

	local wrong_board_out="${tmp_root}/wrong-board-out"
	CURL_BIN="$no_curl" "$BASH" "$wrapper_script" \
		--device-url-from-flash-evidence "$wrong_board_json" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$wrong_board_out"
	assert_contains "$(firmware_log "$wrong_board_out")" "device_url_lookup_reason: flash board is not 205"
	assert_contains "$(firmware_log "$wrong_board_out")" "target_status: blocked"

	local untrusted_out="${tmp_root}/untrusted-out"
	CURL_BIN="$no_curl" "$BASH" "$wrapper_script" \
		--device-url-from-flash-evidence "$untrusted_json" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$untrusted_out"
	assert_contains "$(firmware_log "$untrusted_out")" "device_url_lookup_reason: flash trusted_output is not true"
	assert_contains "$(firmware_log "$untrusted_out")" "target_status: blocked"
}

test_valid_ota_records_invalid_rejection_and_boot_validation_boundaries() {
	local out_dir="${tmp_root}/valid-ota"
	local curl_stub="${tmp_root}/fake-curl"
	local monitor_stub="${tmp_root}/success-monitor"

	create_fake_curl "$curl_stub"
	create_success_monitor "$monitor_stub"

	CURL_BIN="$curl_stub" PHASE13_MONITOR_CAPTURE_SCRIPT="$monitor_stub" "$BASH" "$wrapper_script" \
		--device-url "http://device.local" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--monitor-seconds 1

	local log_file
	log_file="$(firmware_log "$out_dir")"
	assert_contains "$log_file" "phase18_firmware_ota_evidence"
	assert_contains "$log_file" "phase13_firmware_ota_smoke"
	assert_contains "$log_file" "invalid image rejection is not rollback proof"
	assert_contains "$log_file" "invalid_rejection_status: captured - invalid image rejection is not rollback proof"
	assert_contains "$log_file" "post_ota_marker: firmware_commit= present"
	assert_contains "$log_file" "post_ota_marker: reference_commit= present"
	assert_contains "$log_file" "post_ota_marker: ota_boot_validation= present"
	assert_contains "$log_file" "boot_validation_status: passed - firmware_commit=, reference_commit=, and ota_boot_validation= markers captured"
	assert_contains "$log_file" "valid_ota_status: passed"
	assert_contains "$log_file" "rollback_status: not claimed - invalid image rejection is not rollback proof"
	assert_contains "$log_file" "phase18_firmware_ota_status: passed"
	assert_not_contains "$log_file" "http://device.local"
	assert_not_contains "$log_file" "phase18-secret"
	assert_not_contains "$log_file" "PHASE18_SECRET"
	assert_not_contains "$log_file" "192.168.1.77"
}

test_missing_boot_validation_marker_prevents_passed_status() {
	local out_dir="${tmp_root}/missing-boot-validation"
	local curl_stub="${tmp_root}/fake-curl-missing-marker"
	local monitor_stub="${tmp_root}/missing-marker-monitor"

	create_fake_curl "$curl_stub"
	create_missing_boot_validation_monitor "$monitor_stub"

	set +e
	CURL_BIN="$curl_stub" PHASE13_MONITOR_CAPTURE_SCRIPT="$monitor_stub" "$BASH" "$wrapper_script" \
		--device-url "http://device.local" \
		--manifest "${tmp_root}/manifest.json" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--monitor-seconds 1
	local status=$?
	set -e

	if [[ "$status" -eq 0 ]]; then
		fail "missing ota_boot_validation marker should fail the wrapper"
	fi

	local log_file
	log_file="$(firmware_log "$out_dir")"
	assert_contains "$log_file" "post_ota_marker: ota_boot_validation= missing"
	assert_contains "$log_file" "boot_validation_status: blocked - required firmware_commit=, reference_commit=, or ota_boot_validation= marker missing"
	assert_contains "$log_file" "phase18_firmware_ota_status: blocked"
	assert_not_contains "$log_file" "phase18_firmware_ota_status: passed"
}

if [[ ! -f "$wrapper_script" ]]; then
	fail "wrapper script missing: ${wrapper_script}"
fi

mkdir -p "$tmp_root"
create_inputs "$tmp_root"
test_missing_target_writes_blocked_log_without_curl
test_invalid_direct_url_blocks_without_curl
test_trusted_flash_evidence_creates_redacted_target_lock
test_untrusted_or_wrong_board_flash_evidence_blocks
test_valid_ota_records_invalid_rejection_and_boot_validation_boundaries
test_missing_boot_validation_marker_prevents_passed_status

printf 'phase18_firmware_ota_evidence_test passed\n'
