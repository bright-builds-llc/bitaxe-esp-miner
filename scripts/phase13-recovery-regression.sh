#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat >&2 <<'USAGE'
usage: phase13-recovery-regression.sh --manifest PATH --factory-image PATH --ota-image PATH --port PATH --out-dir PATH [--device-url URL] [--allow-failed-update] [--allow-large-erase] [--allow-interrupted-ota]
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

device_url="${DEVICE_URL:-}"
manifest=""
factory_image=""
ota_image=""
port=""
out_dir=""
allow_failed_update=0
allow_large_erase=0
allow_interrupted_ota=0
curl_bin="${CURL_BIN:-curl}"
http_static_smoke_script="${PHASE13_HTTP_STATIC_SMOKE_SCRIPT:-${script_dir}/phase13-http-static-smoke.sh}"
monitor_capture_script="${PHASE13_MONITOR_CAPTURE_SCRIPT:-${script_dir}/phase13-monitor-capture.sh}"

while [[ $# -gt 0 ]]; do
	case "$1" in
	--device-url)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		device_url="$2"
		shift 2
		;;
	--manifest)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		manifest="$2"
		shift 2
		;;
	--factory-image)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		factory_image="$2"
		shift 2
		;;
	--ota-image)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		ota_image="$2"
		shift 2
		;;
	--port)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		port="$2"
		shift 2
		;;
	--out-dir)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		out_dir="$2"
		shift 2
		;;
	--allow-failed-update)
		allow_failed_update=1
		shift
		;;
	--allow-large-erase)
		allow_large_erase=1
		shift
		;;
	--allow-interrupted-ota)
		allow_interrupted_ota=1
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'unknown argument: %s\n' "$1" >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$manifest" || -z "$factory_image" || -z "$ota_image" || -z "$port" || -z "$out_dir" ]]; then
	usage
	exit 2
fi

mkdir -p "$out_dir"
readonly main_log="${out_dir}/recovery-regression.log"
readonly large_log="${out_dir}/large-erase.log"
readonly monitor_log="${out_dir}/large-erase-post-restore-monitor.log"
readonly interrupted_log="${out_dir}/interrupted-ota.log"

: >"$main_log"
: >"$large_log"
: >"$monitor_log"
: >"$interrupted_log"

log_file() {
	local file="$1"
	shift
	printf '%s\n' "$*" >>"$file"
}

log_main() {
	log_file "$main_log" "$*"
}

log_both() {
	local file="$1"
	shift
	log_main "$*"
	log_file "$file" "$*"
}

sanitize_device_url() {
	local url="$1"

	case "$url" in
	http://*) printf 'http://[redacted]' ;;
	https://*) printf 'https://[redacted]' ;;
	"") printf 'not provided' ;;
	*) printf '[invalid-url]' ;;
	esac
}

body_snippet() {
	local body_file="$1"

	if [[ ! -f "$body_file" ]]; then
		printf 'not captured'
		return
	fi

	LC_ALL=C tr -d '\000\r' <"$body_file" |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g' |
		head -c 240 |
		tr '\n\t' '  '
}

failed_update_status_is_rejection() {
	local status="$1"

	case "$status" in
	400 | 409 | 413 | 415 | 422) return 0 ;;
	*) return 1 ;;
	esac
}

failed_update_body_has_rejection_marker() {
	local body_file="$1"

	if grep -Eiq 'wrong api input' "$body_file"; then
		return 1
	fi

	grep -Eiq 'invalid|reject|validation|activation|error' "$body_file"
}

sha256_file() {
	local path="$1"

	if command -v shasum >/dev/null 2>&1; then
		shasum -a 256 "$path" | awk '{print $1}'
		return
	fi
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$path" | awk '{print $1}'
		return
	fi

	printf 'sha256-unavailable'
}

run_http_static_smoke() {
	local label="$1"
	local out_path="$2"

	if [[ -z "$device_url" ]]; then
		log_main "${label}_http_static_status: pending - DEVICE_URL not provided"
		return
	fi

	if [[ ! -x "$http_static_smoke_script" && ! -f "$http_static_smoke_script" ]]; then
		log_main "${label}_http_static_status: blocked - helper missing: ${http_static_smoke_script}"
		return
	fi

	log_main "${label}_http_static_command: scripts/phase13-http-static-smoke.sh --device-url [redacted] --manifest ${manifest} --out-dir ${out_path}"
	"$BASH" "$http_static_smoke_script" --device-url "$device_url" --manifest "$manifest" --out-dir "$out_path" >>"$main_log" 2>&1
}

http_static_smoke_passed() {
	local out_path="$1"
	local smoke_log="${out_path}/http-static-smoke.log"

	[[ -f "$smoke_log" ]] && grep -Fq "http_static_status: passed" "$smoke_log"
}

post_restore_monitor_has_required_markers() {
	local marker

	for marker in \
		"firmware_commit=" \
		"reference_commit=" \
		"safe_state: mining=disabled" \
		"spiffs_mount=available"; do
		if grep -Fq "$marker" "$monitor_log"; then
			log_both "$large_log" "post_restore_marker: ${marker} present"
			continue
		fi

		log_both "$large_log" "large_erase_conclusion: blocked - missing post-restore marker ${marker}"
		return 1
	done
}

require_ultra205_destructive_gate() {
	local detector_log="${out_dir}/detect-ultra205-before-destructive.log"
	local detected_port
	local port_count

	log_both "$large_log" "destructive_gate_status: running - rechecking Ultra 205 detector"
	log_both "$large_log" "destructive_gate_detector_command: just detect-ultra205"
	set +e
	just detect-ultra205 >"$detector_log" 2>&1
	local detector_status=$?
	set -e
	log_both "$large_log" "destructive_gate_detector_log: ${detector_log}"
	log_both "$large_log" "destructive_gate_detector_status: ${detector_status}"
	if [[ "$detector_status" -ne 0 ]]; then
		log_both "$large_log" "large_erase_status: blocked - detector preflight failed"
		return 1
	fi

	port_count="$(awk -F= '/^port=/{count += 1} END {print count + 0}' "$detector_log")"
	detected_port="$(awk -F= '/^port=/{print $2; exit}' "$detector_log")"
	if [[ "$port_count" -ne 1 ]]; then
		log_both "$large_log" "large_erase_status: blocked - detector did not report exactly one port"
		return 1
	fi
	if [[ "$detected_port" != "$port" ]]; then
		log_both "$large_log" "large_erase_status: blocked - detector port mismatch"
		log_both "$large_log" "destructive_gate_detected_port: ${detected_port}"
		return 1
	fi

	log_both "$large_log" "destructive_gate_detected_port: ${detected_port}"
	log_both "$large_log" "destructive_gate_board_info_command: espflash board-info --chip esp32s3 --port ${port} --non-interactive"
	set +e
	espflash board-info --chip esp32s3 --port "$port" --non-interactive >>"$large_log" 2>&1
	local board_info_status=$?
	set -e
	log_both "$large_log" "destructive_gate_board_info_status: ${board_info_status}"
	if [[ "$board_info_status" -ne 0 ]]; then
		log_both "$large_log" "large_erase_status: blocked - board-info preflight failed"
		return 1
	fi
	log_both "$large_log" "destructive_gate_status: passed"
}

write_common_header() {
	log_main "phase13_recovery_regression"
	log_main "manifest: ${manifest}"
	log_main "factory_image: ${factory_image}"
	log_main "ota_image: ${ota_image}"
	log_main "port: ${port}"
	log_main "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
	log_main "destructive_gate: allow flags required per operation"
	log_main "raw_write: disabled"
	log_main ""
}

run_failed_update() {
	if [[ "$allow_failed_update" -eq 0 ]]; then
		log_main "failed_update_status: pending - allow flag not provided"
		return
	fi

	if [[ -z "$device_url" ]]; then
		log_main "failed_update_status: blocked - missing DEVICE_URL"
		log_main "failed_update_conclusion: blocked - failed-update live HTTP action not run"
		return
	fi

	local invalid_image="${out_dir}/invalid-firmware.bin"
	printf 'phase13 invalid firmware image\n' >"$invalid_image"
	local checksum
	checksum="$(sha256_file "$invalid_image")"

	local headers="${out_dir}/failed-update.headers.txt"
	local body="${out_dir}/failed-update.body.txt"
	local error="${out_dir}/failed-update.curl-error.txt"
	: >"$headers"
	: >"$body"
	: >"$error"

	local url="${device_url%/}/api/system/OTA"
	set +e
	local status
	status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$headers" --output "$body" --write-out "%{http_code}" --request POST --data-binary "@${invalid_image}" "$url" 2>"$error")"
	local curl_status=$?
	set -e

	log_main "failed update route: POST /api/system/OTA"
	log_main "failed update artifact: ${invalid_image}"
	log_main "failed update artifact checksum: ${checksum}"
	log_main "failed update failure point: invalid firmware upload to firmware OTA handler"
	log_main "failed update curl_status: ${curl_status}"
	log_main "failed update public status: ${status}"
	log_main "failed update public body: $(body_snippet "$body")"
	if [[ -s "$error" ]]; then
		log_main "failed update curl error: $(body_snippet "$error")"
	fi

	if [[ "$curl_status" -ne 0 ]]; then
		log_main "failed_update_status: blocked - invalid-image request failed"
		log_main "failed update recovery steps: use recovery runbook and collect post-failure boot evidence"
		return 1
	fi
	if [[ "$status" == "200" ]]; then
		log_main "failed_update_status: blocked - invalid image was accepted"
		log_main "failed update recovery steps: use recovery runbook and collect post-failure boot evidence"
		return 1
	fi
	if ! failed_update_status_is_rejection "$status"; then
		log_main "failed_update_status: blocked - invalid image rejection returned unexpected status"
		return 1
	fi
	if ! failed_update_body_has_rejection_marker "$body"; then
		log_main "failed_update_status: blocked - rejection body did not contain expected failure marker"
		return 1
	fi

	local http_static_out="${out_dir}/failed-update-http-static"
	run_http_static_smoke "failed_update_post_failure" "$http_static_out"
	if ! http_static_smoke_passed "$http_static_out"; then
		log_main "failed_update_status: blocked - post-failure operability not proven"
		log_main "failed update recovery steps: use recovery runbook and collect post-failure boot evidence"
		return 1
	fi

	log_main "failed_update_status: captured"
	log_main "failed update post-failure partition/static/API state: HTTP/static smoke passed"
	log_main "failed update recovery steps: explicitly not needed when invalid image is rejected and device remains reachable; otherwise use recovery runbook factory restore"
	log_main "failed update conclusion: captured - invalid image rejection evidence is not rollback proof"
}

run_large_erase() {
	if [[ "$allow_large_erase" -eq 0 ]]; then
		log_both "$large_log" "large_erase_status: pending - allow flag not provided"
		log_file "$monitor_log" "large_erase_post_restore_monitor_status: pending - allow flag not provided"
		log_file "$monitor_log" "capture_status=pending"
		return
	fi

	if [[ ! -f "$factory_image" ]]; then
		log_both "$large_log" "large_erase_status: blocked - missing factory image ${factory_image}"
		return
	fi
	if [[ -z "$device_url" ]]; then
		log_both "$large_log" "large_erase_status: blocked - missing DEVICE_URL"
		log_both "$large_log" "large_erase_conclusion: blocked - post-restore HTTP/static smoke requires DEVICE_URL"
		return 1
	fi

	log_both "$large_log" "large_erase_status: running"
	log_both "$large_log" "large erase exact command: espflash erase-flash --chip esp32s3 --port ${port} --non-interactive"

	set +e
	local espflash_version
	espflash_version="$(espflash --version 2>&1)"
	local version_status=$?
	set -e
	log_both "$large_log" "espflash_version_status: ${version_status}"
	log_both "$large_log" "espflash_version: ${espflash_version}"

	require_ultra205_destructive_gate
	espflash erase-flash --chip esp32s3 --port "$port" --non-interactive >>"$large_log" 2>&1
	log_both "$large_log" "large_erase_result: passed"

	local restore_dir="${out_dir}/large-erase-restore"
	local restore_command=(just flash board=205 "port=${port}" "image=${factory_image}" "manifest=${manifest}" "evidence-dir=${restore_dir}")
	log_both "$large_log" "factory reflash command: just flash board=205 port=${port} image=${factory_image} manifest=${manifest} evidence-dir=${restore_dir}"
	"${restore_command[@]}" >>"$large_log" 2>&1
	log_both "$large_log" "factory reflash result: passed"

	log_both "$large_log" "monitor command: scripts/phase13-monitor-capture.sh --port ${port} --out ${monitor_log} --seconds 35"
	"$BASH" "$monitor_capture_script" --port "$port" --out "$monitor_log" --seconds 35 >>"$large_log" 2>&1
	log_both "$large_log" "post-restore monitor result: captured"
	if ! post_restore_monitor_has_required_markers; then
		return 1
	fi

	local http_static_out="${out_dir}/large-erase-http-static"
	run_http_static_smoke "large_erase_post_restore" "$http_static_out"
	if ! http_static_smoke_passed "$http_static_out"; then
		log_both "$large_log" "large_erase_conclusion: blocked - post-restore HTTP/static smoke failed"
		return 1
	fi

	log_both "$large_log" "large_erase_conclusion: captured - factory image recovery path completed"
}

run_interrupted_ota() {
	if [[ "$allow_interrupted_ota" -eq 0 ]]; then
		log_both "$interrupted_log" "interrupted_update_status: pending - allow flag not provided"
		return
	fi

	if [[ -z "$device_url" ]]; then
		log_both "$interrupted_log" "interrupted_update_status: blocked - missing DEVICE_URL"
		log_both "$interrupted_log" "interrupted_update_conclusion: blocked - interrupted upload not run"
		return
	fi
	if [[ ! -f "$ota_image" ]]; then
		log_both "$interrupted_log" "interrupted_update_status: blocked - missing OTA image ${ota_image}"
		return
	fi

	local checksum
	checksum="$(sha256_file "$ota_image")"
	local body="${out_dir}/interrupted-ota.body.txt"
	local error="${out_dir}/interrupted-ota.curl-error.txt"
	: >"$body"
	: >"$error"

	local url="${device_url%/}/api/system/OTA"
	log_both "$interrupted_log" "interrupted-update route: POST /api/system/OTA"
	log_both "$interrupted_log" "interrupted-update artifact: ${ota_image}"
	log_both "$interrupted_log" "interrupted-update artifact checksum: ${checksum}"
	log_both "$interrupted_log" "interrupted-update command: curl --max-time 1 --limit-rate 1024 --data-binary @${ota_image} [redacted]/api/system/OTA"

	set +e
	local status
	status="$("$curl_bin" --silent --show-error --max-time 1 --limit-rate 1024 --output "$body" --write-out "%{http_code}" --request POST --data-binary "@${ota_image}" "$url" 2>"$error")"
	local curl_status=$?
	set -e

	log_both "$interrupted_log" "interrupted-update failure point: bounded client-side upload interruption"
	log_both "$interrupted_log" "interrupted-update curl_status: ${curl_status}"
	log_both "$interrupted_log" "interrupted-update public status: ${status}"
	log_both "$interrupted_log" "interrupted-update public body: $(body_snippet "$body")"
	if [[ -s "$error" ]]; then
		log_both "$interrupted_log" "interrupted-update curl error: $(body_snippet "$error")"
	fi

	if [[ "$status" == "200" ]]; then
		log_both "$interrupted_log" "interrupted_update_status: blocked - upload completed instead of interrupting"
		return 1
	fi
	if [[ "$curl_status" -ne 28 ]]; then
		log_both "$interrupted_log" "interrupted_update_status: blocked - upload did not time out before completion"
		return 1
	fi

	local http_static_out="${out_dir}/interrupted-ota-http-static"
	run_http_static_smoke "interrupted_update_post_failure" "$http_static_out"
	if ! http_static_smoke_passed "$http_static_out"; then
		log_both "$interrupted_log" "interrupted_update_status: blocked - post-interruption operability not proven"
		return 1
	fi

	log_both "$interrupted_log" "interrupted_update_status: captured"
	log_both "$interrupted_log" "interrupted_update_conclusion: captured - post-interruption operability requires cited HTTP/static and boot evidence"
}

write_common_header
run_failed_update
run_large_erase
run_interrupted_ota
log_main "rollback_status: pending - Plan 04 OTA evidence not run yet"
log_main "boot-validation_status: pending - Plan 04 OTA evidence not run yet"
log_main "OTAWWW gap response: Wrong API input"
log_main "recovery_regression_status: pending"
log_main "conclusion: pending - unsafe recovery and fault-injection paths require explicit allow flags plus prerequisites"
