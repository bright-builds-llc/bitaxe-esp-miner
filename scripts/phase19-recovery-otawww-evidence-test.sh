#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper_script="${PHASE19_RECOVERY_OTAWWW_EVIDENCE_SCRIPT:-${script_dir}/phase19-recovery-otawww-evidence.sh}"

tmp_parent="target/phase19-recovery-regression-and-otawww-evidence-dev-raw"
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

	if ! grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq -- "$needle" "$path"; then
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

	cat >"${root}/manifest.json" <<'JSON'
{
  "schema_version": 2,
  "release_name": "bitaxe-ultra205",
  "source_commit": "190849539700b8f9a7909fd2b6ebd84142557968",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "image_metadata": {
    "board": "205",
    "device_model": "Ultra 205",
    "asic": "BM1366"
  }
}
JSON
	printf 'factory-image\n' >"${root}/bitaxe-ultra205-factory.bin"
	printf 'ota-image\n' >"${root}/esp-miner.bin"
}

create_no_command_bin() {
	local bin_dir="$1"
	local command_log="$2"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/curl" 'printf "curl %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 97
'
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 98
'
	write_executable "${bin_dir}/just" 'printf "just %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 99
'
	: >"$command_log"
}

create_fake_curl() {
	local bin_dir="$1"
	local command_log="$2"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/curl" 'printf "curl %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
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
    --request | --max-time | --write-out)
      shift 2
      ;;
    --data-binary)
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
if [[ -z "$header_file" || -z "$body_file" ]] || { [[ "$url" != "http://device.local/api/system/OTAWWW" ]] && [[ "$url" != "https://device.local:443/api/system/OTAWWW" ]]; }; then
  printf "missing fake curl inputs\n" >&2
  exit 2
fi
	printf "Content-Type: text/plain\nSet-Cookie: session=SECRET\nAuthorization: Bearer PRIVATE\nX-Phase19-Private: 192.168.1.42\nSSID: home-network\n" >"$header_file"
	printf "Wrong API input ssid: home-network password: SECRET token=PRIVATE ip=192.168.1.77\n" >"$body_file"
	printf "400"
	'
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 98
'
	write_executable "${bin_dir}/just" 'printf "just %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 99
'
	: >"$command_log"
}

create_failing_curl() {
	local bin_dir="$1"
	local command_log="$2"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/curl" 'printf "curl %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
printf "Could not resolve host: private-device.local Authorization: Bearer PRIVATE ssid: home-network\n" >&2
exit 7
'
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 98
'
	write_executable "${bin_dir}/just" 'printf "just %s\n" "$*" >>"${PHASE19_COMMAND_LOG:?}"
exit 99
'
	: >"$command_log"
}

create_fake_phase16() {
	local path="$1"
	local args_log="$2"

	write_executable "$path" 'printf "%s\n" "$*" >"${PHASE19_PHASE16_ARGS_LOG:?}"
out_dir=""
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
if [[ -z "$out_dir" ]]; then
  printf "missing --out-dir\n" >&2
  exit 2
fi
mkdir -p "$out_dir"
{
  printf "phase16_recovery_regression\n"
  printf "failed_update_status: pending - allow flag not provided\n"
  printf "large_erase_status: pending - allow flag not provided\n"
  printf "interrupted_update_status: pending - allow flag not provided\n"
} >"${out_dir}/recovery-regression.log"
printf "failed_update_status: pending - allow flag not provided\n" >"${out_dir}/failed-update.log"
printf "large_erase_status: pending - allow flag not provided\n" >"${out_dir}/large-erase.log"
printf "interrupted_update_status: pending - allow flag not provided\n" >"${out_dir}/interrupted-ota.log"
'
	: >"$args_log"
}

create_flash_evidence() {
	local path="$1"
	local monitor_log="$2"

	cat >"$path" <<JSON
{
  "command_kind": "flash-monitor",
  "command": "just flash-monitor board=205 port=/dev/test",
  "board": "205",
  "trusted_output": true,
  "redaction_mode": "commit-redacted",
  "selected_port": "/dev/test",
  "monitor_log_path": "${monitor_log}"
}
JSON
}

run_wrapper() {
	local out_dir="$1"
	shift

	"$BASH" "$wrapper_script" \
		--manifest "${tmp_root}/manifest.json" \
		--factory-image "${tmp_root}/bitaxe-ultra205-factory.bin" \
		--ota-image "${tmp_root}/esp-miner.bin" \
		--port /dev/test \
		--out-dir "$out_dir" \
		"$@"
}

test_default_no_allow_pending() {
	local out_dir="${tmp_root}/default"
	local bin_dir="${tmp_root}/bin-default"
	local command_log="${tmp_root}/commands-default.log"
	local fake_phase16="${tmp_root}/fake-phase16"
	local phase16_args="${tmp_root}/phase16-default-args.log"

	create_no_command_bin "$bin_dir" "$command_log"
	create_fake_phase16 "$fake_phase16" "$phase16_args"

	PATH="${bin_dir}:$PATH" PHASE19_COMMAND_LOG="$command_log" PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$out_dir"

	assert_contains "${out_dir}/recovery-regression/recovery-regression.log" "failed_update_status: pending - allow flag not provided"
	assert_contains "${out_dir}/recovery-regression/recovery-regression.log" "large_erase_status: pending - allow flag not provided"
	assert_contains "${out_dir}/recovery-regression/recovery-regression.log" "interrupted_update_status: pending - allow flag not provided"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "raw_destructive_commands: prohibited"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "--allow-failed-update: omitted"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "--allow-large-erase: omitted"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "--allow-interrupted-ota: omitted"
	if [[ -s "$command_log" ]]; then
		fail "default no-allow wrapper called curl, espflash, or just"
	fi
}

test_origin_url_validation() {
	local fake_phase16="${tmp_root}/fake-phase16-url"
	local phase16_args="${tmp_root}/phase16-url-args.log"
	local invalid
	local index=0

	create_fake_phase16 "$fake_phase16" "$phase16_args"

	for invalid in \
		"http://user:pass@device.local" \
		"http://device.local/path" \
		"http://device.local?x=1" \
		"http://device.local#frag" \
		"ftp://device.local" \
		"http://device local" \
		"http://"; do
		local out_dir="${tmp_root}/invalid-url-${index}"
		set +e
		PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$out_dir" --device-url "$invalid"
		local status=$?
		set -e
		if [[ "$status" -ne 2 ]]; then
			fail "invalid DEVICE_URL should exit 2: ${invalid}"
		fi
		index=$((index + 1))
	done

	local valid_out="${tmp_root}/valid-url"
	local bin_dir="${tmp_root}/bin-valid-url"
	local command_log="${tmp_root}/commands-valid-url.log"
	create_fake_curl "$bin_dir" "$command_log"
	PATH="${bin_dir}:$PATH" PHASE19_COMMAND_LOG="$command_log" PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$valid_out" --device-url "https://device.local:443" --otawww-gap-only
	assert_contains "${valid_out}/target-lock.json" '"device_url_redacted": "https://[redacted]"'
	assert_contains "${valid_out}/otawww/otawww-gap.log" "otawww_status: captured - gap evidence only"
	assert_contains "${valid_out}/otawww/otawww-gap.log" "otawww_selected_headers: content-type: text/plain"
	assert_contains "${valid_out}/otawww/otawww-gap.log" "otawww_public_body: contains Wrong API input"
	assert_contains "${valid_out}/otawww/otawww-gap.log" "current_public_route_behavior: Wrong API input"
	assert_contains "${valid_out}/otawww/otawww-gap.log" "wrong_api_input_proof: present - Wrong API input is not whole-www update proof"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "wrong_api_input_proof: absent"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "Set-Cookie"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "Authorization"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "SECRET"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "PRIVATE"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "home-network"
	assert_not_contains "${valid_out}/otawww/otawww-gap.log" "192.168.1"
	assert_contains "${valid_out}/otawww/otawww.headers.txt" "content-type: text/plain"
	assert_contains "${valid_out}/otawww/otawww.body.txt" "contains Wrong API input"
	assert_contains "${valid_out}/otawww/otawww.curl-error.txt" "none"
	assert_not_contains "${valid_out}/otawww/otawww.headers.txt" "Set-Cookie"
	assert_not_contains "${valid_out}/otawww/otawww.headers.txt" "Authorization"
	assert_not_contains "${valid_out}/otawww/otawww.body.txt" "SECRET"
	assert_not_contains "${valid_out}/otawww/otawww.body.txt" "home-network"
	if [[ -e "${valid_out}/otawww/empty-otawww-upload.bin" ]]; then
		fail "raw OTAWWW request payload was written to commit-ready evidence"
	fi
}

test_trusted_flash_evidence_target_lock() {
	local out_dir="${tmp_root}/trusted-flash"
	local monitor_log="${tmp_root}/trusted-monitor.log"
	local flash_json="${tmp_root}/flash-command-evidence.json"
	local bin_dir="${tmp_root}/bin-trusted"
	local command_log="${tmp_root}/commands-trusted.log"
	local fake_phase16="${tmp_root}/fake-phase16-trusted"
	local phase16_args="${tmp_root}/phase16-trusted-args.log"

	printf 'device_url=http://device.local\n' >"$monitor_log"
	create_flash_evidence "$flash_json" "$monitor_log"
	create_fake_curl "$bin_dir" "$command_log"
	create_fake_phase16 "$fake_phase16" "$phase16_args"

	PATH="${bin_dir}:$PATH" PHASE19_COMMAND_LOG="$command_log" PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$out_dir" --device-url-from-flash-evidence "$flash_json" --target-lock-out "${out_dir}/target-lock.json" --otawww-gap-only

	assert_contains "${out_dir}/target-lock.json" '"target_status": "passed"'
	assert_contains "${out_dir}/target-lock.json" '"device_url_source": "usb_flash_monitor_log"'
	assert_contains "${out_dir}/target-lock.json" '"device_url_redacted": "http://[redacted]"'
	assert_contains "${out_dir}/target-lock.json" '"selected_port": "/dev/test"'
	assert_contains "${out_dir}/target-lock.json" '"network_scan": "disabled"'
	assert_not_contains "${out_dir}/target-lock.json" "http://device.local"
}

test_allow_flags_delegate_to_phase16() {
	local out_dir="${tmp_root}/allow-flags"
	local bin_dir="${tmp_root}/bin-allow"
	local command_log="${tmp_root}/commands-allow.log"
	local fake_phase16="${tmp_root}/fake-phase16-allow"
	local phase16_args="${tmp_root}/phase16-allow-args.log"

	create_fake_curl "$bin_dir" "$command_log"
	create_fake_phase16 "$fake_phase16" "$phase16_args"

	PATH="${bin_dir}:$PATH" PHASE19_COMMAND_LOG="$command_log" PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$out_dir" \
		--device-url http://device.local \
		--allow-failed-update \
		--allow-large-erase \
		--allow-interrupted-ota \
		--otawww-gap-only

	assert_contains "$phase16_args" "--device-url"
	assert_contains "$phase16_args" "http://device.local"
	assert_contains "$phase16_args" "--allow-failed-update"
	assert_contains "$phase16_args" "--allow-large-erase"
	assert_contains "$phase16_args" "--allow-interrupted-ota"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "--allow-failed-update: supplied"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "--allow-large-erase: supplied"
	assert_contains "${out_dir}/phase19-recovery-otawww-evidence.log" "--allow-interrupted-ota: supplied"
	assert_not_contains "$command_log" "espflash"
	assert_not_contains "$command_log" "just"
}

test_otawww_gap_curl_failure_blocks_evidence() {
	local out_dir="${tmp_root}/otawww-curl-failure"
	local bin_dir="${tmp_root}/bin-curl-failure"
	local command_log="${tmp_root}/commands-curl-failure.log"
	local fake_phase16="${tmp_root}/fake-phase16-curl-failure"
	local phase16_args="${tmp_root}/phase16-curl-failure-args.log"

	create_failing_curl "$bin_dir" "$command_log"
	create_fake_phase16 "$fake_phase16" "$phase16_args"

	PATH="${bin_dir}:$PATH" PHASE19_COMMAND_LOG="$command_log" PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$out_dir" --device-url http://device.local --otawww-gap-only

	assert_contains "${out_dir}/target-lock.json" '"target_status": "passed"'
	assert_contains "${out_dir}/otawww/otawww-gap.log" "otawww_curl_status: 7"
	assert_contains "${out_dir}/otawww/otawww-gap.log" "otawww_status: blocked - curl failed"
	assert_contains "${out_dir}/otawww/otawww-gap.log" "whole_www_update_proof: absent"
	assert_contains "${out_dir}/otawww/otawww-gap.log" "wrong_api_input_proof: absent - HTTP response unavailable"
	assert_not_contains "${out_dir}/otawww/otawww-gap.log" "otawww_status: captured"
	assert_not_contains "${out_dir}/otawww/otawww-gap.log" "private-device.local"
	assert_not_contains "${out_dir}/otawww/otawww-gap.log" "Authorization"
	assert_not_contains "${out_dir}/otawww/otawww-gap.log" "PRIVATE"
	assert_not_contains "${out_dir}/otawww/otawww-gap.log" "home-network"
	assert_contains "${out_dir}/otawww/otawww.headers.txt" "headers redacted - no allowlisted headers"
	assert_contains "${out_dir}/otawww/otawww.body.txt" "body redacted - unexpected shape"
	assert_contains "${out_dir}/otawww/otawww.curl-error.txt" "curl error redacted"
	assert_not_contains "${out_dir}/otawww/otawww.curl-error.txt" "private-device.local"
	assert_not_contains "${out_dir}/otawww/otawww.curl-error.txt" "PRIVATE"
	if [[ -e "${out_dir}/otawww/empty-otawww-upload.bin" ]]; then
		fail "raw OTAWWW request payload was written to commit-ready evidence"
	fi
}

test_otawww_gap_without_target() {
	local out_dir="${tmp_root}/otawww-gap-no-target"
	local bin_dir="${tmp_root}/bin-gap-no-target"
	local command_log="${tmp_root}/commands-gap-no-target.log"
	local fake_phase16="${tmp_root}/fake-phase16-gap"
	local phase16_args="${tmp_root}/phase16-gap-args.log"

	create_no_command_bin "$bin_dir" "$command_log"
	create_fake_phase16 "$fake_phase16" "$phase16_args"

	PATH="${bin_dir}:$PATH" PHASE19_COMMAND_LOG="$command_log" PHASE19_PHASE16_ARGS_LOG="$phase16_args" PHASE16_RECOVERY_REGRESSION_SCRIPT="$fake_phase16" run_wrapper "$out_dir" --otawww-gap-only

	assert_contains "${out_dir}/otawww/otawww-gap.log" "otawww_status: blocked - missing DEVICE_URL"
	assert_contains "${out_dir}/otawww/otawww-gap.log" "otawww_claim: REL-03 gap"
	assert_contains "${out_dir}/otawww/otawww-gap.log" "whole_www_update_proof: absent"
	assert_not_contains "${out_dir}/otawww/otawww-gap.log" "otawww_status: verified"
	if [[ -s "$command_log" ]]; then
		fail "OTAWWW gap-only without target called curl, espflash, or just"
	fi
}

if [[ ! -f "$wrapper_script" ]]; then
	fail "wrapper script missing: ${wrapper_script}"
fi

create_inputs "$tmp_root"
test_default_no_allow_pending
test_origin_url_validation
test_trusted_flash_evidence_target_lock
test_allow_flags_delegate_to_phase16
test_otawww_gap_curl_failure_blocks_evidence
test_otawww_gap_without_target

printf 'phase19_recovery_otawww_evidence_test passed\n'
