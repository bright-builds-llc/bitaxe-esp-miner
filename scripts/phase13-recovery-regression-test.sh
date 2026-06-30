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
printf "invalid firmware rejected\n" >"$body_file"
printf "400"
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
printf "http_static_status: passed\n" >"${out_dir}/http-static-smoke.log"
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
test_large_erase_command_rendering

printf 'phase13_recovery_regression_test passed\n'
