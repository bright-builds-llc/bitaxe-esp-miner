#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly recovery_script="${PHASE16_RECOVERY_REGRESSION_SCRIPT:-${script_dir}/phase16-recovery-regression.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase16-recovery-regression-test.XXXXXX")"
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

assert_line_order() {
	local path="$1"
	local first="$2"
	local second="$3"
	local first_line
	local second_line

	first_line="$(grep -nF "$first" "$path" | head -n 1 | cut -d: -f1)"
	second_line="$(grep -nF "$second" "$path" | head -n 1 | cut -d: -f1)"
	if [[ -z "$first_line" || -z "$second_line" || "$first_line" -ge "$second_line" ]]; then
		printf 'Expected "%s" before "%s" in %s\n' "$first" "$second" "$path" >&2
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

current_commit() {
	git rev-parse HEAD
}

create_manifest() {
	local path="$1"
	local source_commit="$2"

	cat >"$path" <<JSON
{
  "schema_version": 2,
  "release_name": "bitaxe-ultra205",
  "source_commit": "${source_commit}",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "image_metadata": {
    "board": "205",
    "device_model": "Ultra 205",
    "asic": "BM1366"
  },
  "artifacts": [
    {
      "kind": "firmware_ota_image",
      "path": "esp-miner.bin",
      "offset": "0x10000",
      "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    },
    {
      "kind": "factory_merged_image",
      "path": "bitaxe-ultra205-factory.bin",
      "offset": "0x0",
      "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }
  ]
}
JSON
}

create_inputs() {
	local root="$1"
	local source_commit="${2:-$(current_commit)}"

	create_manifest "${root}/manifest.json" "$source_commit"
	printf 'factory-image\n' >"${root}/bitaxe-ultra205-factory.bin"
	printf 'ota-image\n' >"${root}/esp-miner.bin"
}

create_no_curl_stub() {
	local path="$1"

	write_executable "$path" 'printf "curl should not have been called\n" >&2
exit 97
'
}

create_fake_curl() {
	local path="$1"

	write_executable "$path" 'body_file=""
url=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      body_file="$2"
      shift 2
      ;;
    --dump-header)
      printf "Content-Type: text/plain\n" >"$2"
      shift 2
      ;;
    --data-binary | --write-out | --max-time | --limit-rate | --request)
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
if [[ -z "$body_file" || "$url" != "http://device.local/api/system/OTA" ]]; then
  printf "missing fake curl inputs\n" >&2
  exit 2
fi
if [[ "${PHASE16_FAKE_CURL_STDERR_HOST:-0}" == "1" ]]; then
  printf "curl: (6) Could not resolve host: private-bitaxe.local\ncurl: (7) Failed to connect to private-bitaxe.local port 80\ncurl: (22) URL rejected: http://private-bitaxe.local/api/system/OTA\n" >&2
fi
case "${PHASE16_FAKE_CURL_SCENARIO:-invalid-rejected}" in
  invalid-rejected)
    printf "invalid firmware rejected {\"ssid\":\"phase16-secret\",\"stratumCert\":\"PHASE16_LONG_FAILED_UPDATE_SECRET_PREFIX_%s\",\"ip\":\"192.168.1.77\",\"mac\":\"aa:bb:cc:dd:ee:ff\"}\n" "$(printf "x%.0s" {1..260})" >"$body_file"
    printf "400"
    ;;
  interrupted-timeout)
    printf "client timed out before upload completed {\"wifiPass\":\"phase16-wifi\",\"poolUrl\":\"stratum+tcp://pool.example:3333\",\"ip\":\"192.168.1.88\"}\n" >"$body_file"
    printf "000"
    exit 28
    ;;
  *)
    printf "unknown fake curl scenario: %s\n" "$PHASE16_FAKE_CURL_SCENARIO" >&2
    exit 2
    ;;
esac
'
}

create_fake_command_bin() {
	local bin_dir="$1"
	local command_log="$2"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PHASE16_COMMAND_LOG:?}"
if [[ "$*" == "board-info --chip esp32s3 --port /dev/test --non-interactive" ]]; then
  printf "Chip type: esp32s3\n"
  exit 0
fi
if [[ "$*" == "erase-flash --chip esp32s3 --port /dev/test --non-interactive" ]]; then
  printf "Erased flash\n"
  exit 0
fi
printf "unexpected espflash command: %s\n" "$*" >&2
exit 2
'
	write_executable "${bin_dir}/just" 'printf "just %s\n" "$*" >>"${PHASE16_COMMAND_LOG:?}"
if [[ "$*" == "detect-ultra205" ]]; then
  printf "port=%s\n" "${PHASE16_DETECTOR_PORT:-/dev/test}"
  exit 0
fi
expected_prefix="flash board=205 port=/dev/test image=${PHASE16_EXPECT_FACTORY:?} manifest=${PHASE16_EXPECT_MANIFEST:?} evidence-dir="
case "$*" in
  "$expected_prefix"*) exit 0 ;;
  *)
    printf "unexpected just command: %s\nexpected prefix: %s\n" "$*" "$expected_prefix" >&2
    exit 2
    ;;
esac
'
	: >"$command_log"
}

create_fake_monitor() {
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
printf "capture_status=completed\n" >"$out"
{
  printf "firmware_commit=%s\n" "$(git rev-parse HEAD)"
  printf "reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50\n"
  printf "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled\n"
  printf "spiffs_mount=available\n"
} >>"$out"
'
}

create_fake_http_smoke() {
	local path="$1"

	write_executable "$path" 'out_dir=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      out_dir="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
mkdir -p "$out_dir"
printf "phase16_http_static_smoke\nhttp_static_status: passed\n" >"${out_dir}/http-static-smoke.log"
'
}

test_default_pending_behavior() {
	local out_dir="${tmp_root}/pending"
	local curl_stub="${tmp_root}/no-curl"

	create_no_curl_stub "$curl_stub"

	CURL_BIN="$curl_stub" "$BASH" "$recovery_script" \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/recovery-regression.log" "phase16_recovery_regression"
	assert_contains "${out_dir}/recovery-regression.log" "network_scan: disabled - using explicit DEVICE_URL only"
	assert_contains "${out_dir}/failed-update.log" "failed_update_status: pending - allow flag not provided"
	assert_contains "${out_dir}/large-erase.log" "large_erase_status: pending - allow flag not provided"
	assert_contains "${out_dir}/large-erase-post-restore-monitor.log" "capture_status=pending"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: pending - allow flag not provided"
}

test_detector_mismatch_blocks_before_large_erase() {
	local out_dir="${tmp_root}/large-detector-mismatch"
	local bin_dir="${tmp_root}/bin-detector-mismatch"
	local command_log="${tmp_root}/commands-detector-mismatch.log"
	local monitor_stub="${tmp_root}/fake-monitor-detector-mismatch"
	local http_stub="${tmp_root}/fake-http-smoke-detector-mismatch"

	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_monitor "$monitor_stub"
	create_fake_http_smoke "$http_stub"

	set +e
	PATH="${bin_dir}:$PATH" PHASE16_COMMAND_LOG="$command_log" PHASE16_DETECTOR_PORT=/dev/other PHASE16_EXPECT_FACTORY="${tmp_root}/bitaxe-ultra205-factory.bin" PHASE16_EXPECT_MANIFEST="${tmp_root}/manifest.json" PHASE16_MONITOR_CAPTURE_SCRIPT="$monitor_stub" PHASE16_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-large-erase
	local command_status=$?
	set -e

	if [[ "$command_status" -eq 0 ]]; then
		fail "large erase accepted detector port mismatch"
	fi
	assert_contains "${out_dir}/large-erase.log" "phase16_gate_status: blocked - detector port mismatch"
	assert_contains "${out_dir}/large-erase.log" "phase16_gate_detected_port: /dev/other"
	assert_not_contains "$command_log" "espflash erase-flash --chip esp32s3 --port /dev/test --non-interactive"
	assert_not_contains "$command_log" "just flash board=205"
}

test_stale_manifest_source_commit_blocks_action() {
	local stale_root="${tmp_root}/stale"
	local out_dir="${tmp_root}/stale-out"
	local bin_dir="${tmp_root}/bin-stale"
	local command_log="${tmp_root}/commands-stale.log"
	local monitor_stub="${tmp_root}/fake-monitor-stale"
	local http_stub="${tmp_root}/fake-http-smoke-stale"

	mkdir -p "$stale_root"
	create_inputs "$stale_root" "0000000000000000000000000000000000000000"
	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_monitor "$monitor_stub"
	create_fake_http_smoke "$http_stub"

	set +e
	PATH="${bin_dir}:$PATH" PHASE16_COMMAND_LOG="$command_log" PHASE16_EXPECT_FACTORY="${stale_root}/bitaxe-ultra205-factory.bin" PHASE16_EXPECT_MANIFEST="${stale_root}/manifest.json" PHASE16_MONITOR_CAPTURE_SCRIPT="$monitor_stub" PHASE16_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${stale_root}/manifest.json" \
		--factory-image "${stale_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${stale_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-large-erase
	local command_status=$?
	set -e

	if [[ "$command_status" -eq 0 ]]; then
		fail "large erase accepted stale manifest source_commit"
	fi
	assert_contains "${out_dir}/large-erase.log" "phase16_gate_manifest_source_commit: 0000000000000000000000000000000000000000"
	assert_contains "${out_dir}/large-erase.log" "phase16_gate_status: blocked - manifest source_commit does not match current HEAD"
	assert_not_contains "$command_log" "espflash erase-flash --chip esp32s3 --port /dev/test --non-interactive"
}

test_gate_passes_before_failed_update_action() {
	local out_dir="${tmp_root}/failed-update"
	local bin_dir="${tmp_root}/bin-failed-update"
	local command_log="${tmp_root}/commands-failed-update.log"
	local curl_stub="${tmp_root}/fake-curl-failed-update"
	local http_stub="${tmp_root}/fake-http-smoke-failed-update"

	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	PATH="${bin_dir}:$PATH" PHASE16_COMMAND_LOG="$command_log" PHASE16_EXPECT_FACTORY="${tmp_root}/bitaxe-ultra205-factory.bin" PHASE16_EXPECT_MANIFEST="${tmp_root}/manifest.json" CURL_BIN="$curl_stub" PHASE16_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-failed-update

	assert_contains "${out_dir}/failed-update.log" "phase16_gate_status: passed"
	assert_contains "${out_dir}/failed-update.log" "failed update route: POST /api/system/OTA"
	assert_line_order "${out_dir}/failed-update.log" "phase16_gate_status: passed" "failed update route: POST /api/system/OTA"
	assert_contains "${out_dir}/failed-update.log" "failed_update_status: captured"
	assert_contains "$command_log" "just detect-ultra205"
	assert_contains "$command_log" "espflash board-info --chip esp32s3 --port /dev/test --non-interactive"
}

test_large_erase_uses_exact_erase_restore_and_monitor_commands() {
	local out_dir="${tmp_root}/large"
	local bin_dir="${tmp_root}/bin-large"
	local command_log="${tmp_root}/commands-large.log"
	local monitor_stub="${tmp_root}/fake-monitor-large"
	local http_stub="${tmp_root}/fake-http-smoke-large"

	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_monitor "$monitor_stub"
	create_fake_http_smoke "$http_stub"

	PATH="${bin_dir}:$PATH" PHASE16_COMMAND_LOG="$command_log" PHASE16_EXPECT_FACTORY="${tmp_root}/bitaxe-ultra205-factory.bin" PHASE16_EXPECT_MANIFEST="${tmp_root}/manifest.json" PHASE16_MONITOR_CAPTURE_SCRIPT="$monitor_stub" PHASE16_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-large-erase

	assert_contains "${out_dir}/large-erase.log" "phase16_gate_artifact: bitaxe-ultra205-factory.bin present"
	assert_contains "${out_dir}/large-erase.log" "phase16_gate_artifact: esp-miner.bin present"
	assert_contains "${out_dir}/large-erase.log" "large erase exact command: espflash erase-flash --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "${out_dir}/large-erase.log" "factory reflash command: just flash board=205 port=/dev/test image=${tmp_root}/bitaxe-ultra205-factory.bin manifest=${tmp_root}/manifest.json evidence-dir=${tmp_root}/large/large-erase-restore"
	assert_contains "${out_dir}/large-erase.log" "monitor command: scripts/phase13-monitor-capture.sh --port /dev/test --out ${tmp_root}/large/large-erase-post-restore-monitor.log --seconds 35"
	assert_contains "${out_dir}/large-erase.log" "large_erase_conclusion: captured - factory image recovery path completed"
	assert_contains "$command_log" "espflash erase-flash --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "$command_log" "just flash board=205 port=/dev/test image=${tmp_root}/bitaxe-ultra205-factory.bin manifest=${tmp_root}/manifest.json evidence-dir=${tmp_root}/large/large-erase-restore"
}

test_interrupted_timeout_records_evidence_fields() {
	local out_dir="${tmp_root}/interrupted"
	local bin_dir="${tmp_root}/bin-interrupted"
	local command_log="${tmp_root}/commands-interrupted.log"
	local curl_stub="${tmp_root}/fake-curl-interrupted"
	local http_stub="${tmp_root}/fake-http-smoke-interrupted"

	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	PATH="${bin_dir}:$PATH" PHASE16_COMMAND_LOG="$command_log" PHASE16_EXPECT_FACTORY="${tmp_root}/bitaxe-ultra205-factory.bin" PHASE16_EXPECT_MANIFEST="${tmp_root}/manifest.json" PHASE16_FAKE_CURL_SCENARIO=interrupted-timeout CURL_BIN="$curl_stub" PHASE16_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-interrupted-ota

	assert_contains "${out_dir}/interrupted-ota.log" "phase16_gate_status: passed"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted-update route: POST /api/system/OTA"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted-update artifact checksum:"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted-update failure point: bounded client-side upload interruption"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted-update curl_status: 28"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted-update public status: 000"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: captured"
	assert_contains "${out_dir}/interrupted-ota-http-static/http-static-smoke.log" "phase16_http_static_smoke"
}

test_secret_redaction_for_failed_update_and_interrupted_logs() {
	local out_dir="${tmp_root}/redaction"
	local bin_dir="${tmp_root}/bin-redaction"
	local command_log="${tmp_root}/commands-redaction.log"
	local curl_stub="${tmp_root}/fake-curl-redaction"
	local http_stub="${tmp_root}/fake-http-smoke-redaction"

	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	PATH="${bin_dir}:$PATH" PHASE16_COMMAND_LOG="$command_log" PHASE16_EXPECT_FACTORY="${tmp_root}/bitaxe-ultra205-factory.bin" PHASE16_EXPECT_MANIFEST="${tmp_root}/manifest.json" PHASE16_FAKE_CURL_STDERR_HOST=1 CURL_BIN="$curl_stub" PHASE16_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-failed-update

	assert_contains "${out_dir}/failed-update.log" "\"ssid\":\"[redacted]\""
	assert_contains "${out_dir}/failed-update.log" "\"stratumCert\":\"[redacted]\""
	assert_contains "${out_dir}/failed-update.log" "\"ip\":\"[redacted]\""
	assert_contains "${out_dir}/failed-update.log" "[redacted-mac]"
	assert_contains "${out_dir}/failed-update.log" "failed update curl error: curl: (6) Could not resolve host: [redacted-host]"
	assert_contains "${out_dir}/failed-update.log" "Failed to connect to [redacted-host]"
	assert_contains "${out_dir}/failed-update.log" "[redacted-url]"
	assert_not_contains "${out_dir}/failed-update.log" "phase16-secret"
	assert_not_contains "${out_dir}/failed-update.log" "PHASE16_LONG_FAILED_UPDATE_SECRET_PREFIX"
	assert_not_contains "${out_dir}/failed-update.log" "192.168.1.77"
	assert_not_contains "${out_dir}/failed-update.log" "aa:bb:cc:dd:ee:ff"
	assert_not_contains "${out_dir}/failed-update.log" "private-bitaxe.local"
	assert_not_contains "${out_dir}/failed-update.log" "http://private-bitaxe.local"
}

if [[ ! -f "$recovery_script" ]]; then
	fail "recovery script missing: ${recovery_script}"
fi

create_inputs "$tmp_root"
test_default_pending_behavior
test_detector_mismatch_blocks_before_large_erase
test_stale_manifest_source_commit_blocks_action
test_gate_passes_before_failed_update_action
test_large_erase_uses_exact_erase_restore_and_monitor_commands
test_interrupted_timeout_records_evidence_fields
test_secret_redaction_for_failed_update_and_interrupted_logs

printf 'phase16_recovery_regression_test passed\n'
