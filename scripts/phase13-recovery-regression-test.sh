#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly recovery_script="${PHASE13_RECOVERY_REGRESSION_SCRIPT:-${script_dir}/phase13-recovery-regression.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase13-recovery-regression-test.XXXXXX")"
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

create_inputs() {
	local root="$1"

	printf '{"source_commit":"source","reference_commit":"reference"}\n' >"${root}/manifest.json"
	printf 'factory-image\n' >"${root}/factory.bin"
	printf 'ota-image\n' >"${root}/esp-miner.bin"
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
case "${PHASE13_FAKE_CURL_SCENARIO:-invalid-rejected}" in
  invalid-rejected)
    printf "invalid firmware rejected\n" >"$body_file"
    printf "400"
    ;;
  invalid-accepted)
    printf "Firmware update complete, rebooting now!\n" >"$body_file"
    printf "200"
    ;;
  interrupted-timeout)
    printf "client timed out before upload completed\n" >"$body_file"
    printf "000"
    exit 28
    ;;
  ota-completed)
    printf "Firmware update complete, rebooting now!\n" >"$body_file"
    printf "200"
    ;;
  *)
    printf "unknown fake curl scenario: %s\n" "$PHASE13_FAKE_CURL_SCENARIO" >&2
    exit 2
    ;;
esac
'
}

create_fake_command_bin() {
	local bin_dir="$1"
	local command_log="$2"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PHASE13_COMMAND_LOG:?}"
if [[ "${1:-}" == "--version" ]]; then
  printf "espflash 4.0.0\n"
  exit 0
fi
if [[ "$*" == "board-info --chip esp32s3 --port /dev/test --non-interactive" ]]; then
  printf "Chip type: esp32s3\n"
  exit 0
fi
if [[ "$*" != "erase-flash --chip esp32s3 --port /dev/test --non-interactive" ]]; then
  printf "unexpected espflash command: %s\n" "$*" >&2
  exit 2
fi
'
	write_executable "${bin_dir}/just" 'printf "just %s\n" "$*" >>"${PHASE13_COMMAND_LOG:?}"
if [[ "$*" == "detect-ultra205" ]]; then
  printf "port=%s\n" "${PHASE13_DETECTOR_PORT:-/dev/test}"
  exit 0
fi
expected="flash board=205 port=/dev/test image='"${tmp_root}"'/factory.bin manifest='"${tmp_root}"'/manifest.json evidence-dir='"${tmp_root}"'/large/large-erase-restore"
if [[ "$*" != "$expected" ]]; then
  printf "unexpected just command: %s\nexpected: %s\n" "$*" "$expected" >&2
  exit 2
fi
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
printf "http_static_status: %s\n" "${PHASE13_FAKE_HTTP_SMOKE_STATUS:-passed}" >"${out_dir}/http-static-smoke.log"
'
}

test_default_pending_behavior() {
	local out_dir="${tmp_root}/pending"

	"$BASH" "$recovery_script" \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/recovery-regression.log" "failed_update_status: pending - allow flag not provided"
	assert_contains "${out_dir}/large-erase.log" "large_erase_status: pending - allow flag not provided"
	assert_contains "${out_dir}/large-erase-post-restore-monitor.log" "large_erase_post_restore_monitor_status: pending - allow flag not provided"
	assert_contains "${out_dir}/large-erase-post-restore-monitor.log" "capture_status=pending"
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: pending - allow flag not provided"
	assert_contains "${out_dir}/recovery-regression.log" "recovery_regression_status: pending"
}

test_failed_update_evidence_fields() {
	local out_dir="${tmp_root}/failed-update"
	local curl_stub="${tmp_root}/fake-curl"
	local http_stub="${tmp_root}/fake-http-smoke"

	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	CURL_BIN="$curl_stub" PHASE13_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-failed-update

	local log_file="${out_dir}/recovery-regression.log"
	assert_contains "$log_file" "failed_update_status: captured"
	assert_contains "$log_file" "failed update route: POST /api/system/OTA"
	assert_contains "$log_file" "failed update artifact checksum:"
	assert_contains "$log_file" "failed update failure point:"
	assert_contains "$log_file" "failed update public status: 400"
	assert_contains "$log_file" "failed update public body: invalid firmware rejected"
	assert_contains "$log_file" "failed update post-failure partition/static/API state:"
	assert_contains "$log_file" "failed update recovery steps:"
	assert_contains "$log_file" "failed update conclusion:"
}

test_failed_update_blocks_if_invalid_image_is_accepted() {
	local out_dir="${tmp_root}/failed-update-accepted"
	local curl_stub="${tmp_root}/fake-curl-accepted"
	local http_stub="${tmp_root}/fake-http-smoke-accepted"

	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	set +e
	PHASE13_FAKE_CURL_SCENARIO=invalid-accepted CURL_BIN="$curl_stub" PHASE13_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-failed-update
	local command_status=$?
	set -e

	if [[ "$command_status" -eq 0 ]]; then
		fail "failed update accepted invalid image without blocking"
	fi
	local log_file="${out_dir}/recovery-regression.log"
	assert_contains "$log_file" "failed_update_status: blocked - invalid image was accepted"
	assert_contains "$log_file" "failed update recovery steps: use recovery runbook and collect post-failure boot evidence"
	assert_not_contains "$log_file" "failed_update_status: captured"
}

test_interrupted_ota_blocks_if_upload_completes() {
	local out_dir="${tmp_root}/interrupted-completed"
	local curl_stub="${tmp_root}/fake-curl-interrupted-completed"
	local http_stub="${tmp_root}/fake-http-smoke-interrupted-completed"

	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	set +e
	PHASE13_FAKE_CURL_SCENARIO=ota-completed CURL_BIN="$curl_stub" PHASE13_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-interrupted-ota
	local command_status=$?
	set -e

	if [[ "$command_status" -eq 0 ]]; then
		fail "interrupted OTA accepted a completed upload as captured"
	fi
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: blocked - upload completed instead of interrupting"
	assert_not_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: captured"
}

test_interrupted_ota_blocks_if_post_smoke_fails() {
	local out_dir="${tmp_root}/interrupted-smoke-failed"
	local curl_stub="${tmp_root}/fake-curl-interrupted-smoke-failed"
	local http_stub="${tmp_root}/fake-http-smoke-interrupted-smoke-failed"

	create_fake_curl "$curl_stub"
	create_fake_http_smoke "$http_stub"

	set +e
	PHASE13_FAKE_CURL_SCENARIO=interrupted-timeout PHASE13_FAKE_HTTP_SMOKE_STATUS=blocked CURL_BIN="$curl_stub" PHASE13_HTTP_STATIC_SMOKE_SCRIPT="$http_stub" "$BASH" "$recovery_script" \
		--device-url http://device.local \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-interrupted-ota
	local command_status=$?
	set -e

	if [[ "$command_status" -eq 0 ]]; then
		fail "interrupted OTA accepted failed post-interruption smoke"
	fi
	assert_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: blocked - post-interruption operability not proven"
	assert_contains "${out_dir}/interrupted-ota-http-static/http-static-smoke.log" "http_static_status: blocked"
	assert_not_contains "${out_dir}/interrupted-ota.log" "interrupted_update_status: captured"
}

test_large_erase_command_rendering() {
	local out_dir="${tmp_root}/large"
	local bin_dir="${tmp_root}/bin"
	local command_log="${tmp_root}/commands.log"
	local monitor_stub="${tmp_root}/fake-monitor"

	create_fake_command_bin "$bin_dir" "$command_log"
	create_fake_monitor "$monitor_stub"

	PATH="${bin_dir}:$PATH" PHASE13_COMMAND_LOG="$command_log" PHASE13_MONITOR_CAPTURE_SCRIPT="$monitor_stub" "$BASH" "$recovery_script" \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		--allow-large-erase

	assert_contains "${out_dir}/large-erase.log" "large erase exact command: espflash erase-flash --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "${out_dir}/large-erase.log" "destructive_gate_detector_command: just detect-ultra205"
	assert_contains "${out_dir}/large-erase.log" "destructive_gate_board_info_command: espflash board-info --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "${out_dir}/large-erase.log" "factory reflash command: just flash board=205 port=/dev/test image=${tmp_root}/factory.bin manifest=${tmp_root}/manifest.json evidence-dir=${tmp_root}/large/large-erase-restore"
	assert_contains "${out_dir}/large-erase-post-restore-monitor.log" "capture_status=completed"
	assert_contains "$command_log" "just detect-ultra205"
	assert_contains "$command_log" "espflash board-info --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "$command_log" "espflash erase-flash --chip esp32s3 --port /dev/test --non-interactive"
	assert_contains "$command_log" "just flash board=205 port=/dev/test image=${tmp_root}/factory.bin manifest=${tmp_root}/manifest.json evidence-dir=${tmp_root}/large/large-erase-restore"
}

if [[ ! -f "$recovery_script" ]]; then
	fail "recovery script missing: ${recovery_script}"
fi

create_inputs "$tmp_root"
test_default_pending_behavior
test_failed_update_evidence_fields
test_failed_update_blocks_if_invalid_image_is_accepted
test_interrupted_ota_blocks_if_upload_completes
test_interrupted_ota_blocks_if_post_smoke_fails
test_large_erase_command_rendering

printf 'phase13_recovery_regression_test passed\n'
