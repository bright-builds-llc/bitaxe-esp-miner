#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat >&2 <<'USAGE'
usage: phase13-firmware-ota-smoke.sh [--device-url URL] [--manifest PATH] [--ota-image PATH] [--port PATH] [--out-dir PATH] [--monitor-seconds N]
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

device_url="${DEVICE_URL:-}"
manifest="bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
ota_image="bazel-bin/firmware/bitaxe/esp-miner.bin"
port=""
out_dir="docs/parity/evidence/phase-13-final-ultra-205-release-evidence/firmware-ota"
monitor_seconds="45"
curl_bin="${CURL_BIN:-curl}"
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
	--monitor-seconds)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		monitor_seconds="$2"
		shift 2
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

if [[ ! "$monitor_seconds" =~ ^[0-9]+$ || "$monitor_seconds" -lt 1 ]]; then
	printf 'monitor-seconds must be a positive integer\n' >&2
	exit 2
fi

mkdir -p "$out_dir"
readonly log_file="${out_dir}/firmware-ota-smoke.log"
readonly post_ota_monitor_log="${out_dir}/post-ota-monitor.log"

: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
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

manifest_field() {
	local field="$1"

	if [[ ! -f "$manifest" ]]; then
		printf 'unavailable'
		return
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		printf 'unavailable'
		return
	fi

	python3 - "$manifest" "$field" <<'PY'
import json
import sys

path, field = sys.argv[1], sys.argv[2]
try:
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
except Exception:
    print("unavailable")
    raise SystemExit(0)

value = data.get(field, "unavailable")
if value is None:
    value = "unavailable"
print(value)
PY
}

manifest_artifact_field() {
	local kind="$1"
	local field="$2"

	if [[ ! -f "$manifest" ]]; then
		printf 'unavailable'
		return
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		printf 'unavailable'
		return
	fi

	python3 - "$manifest" "$kind" "$field" <<'PY'
import json
import sys

path, kind, field = sys.argv[1], sys.argv[2], sys.argv[3]
try:
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
except Exception:
    print("unavailable")
    raise SystemExit(0)

for artifact in data.get("artifacts", []):
    if artifact.get("kind") == kind:
        value = artifact.get(field, "unavailable")
        if value is None:
            value = "unavailable"
        print(value)
        raise SystemExit(0)

print("unavailable")
PY
}

body_snippet() {
	local body_file="$1"

	if [[ ! -f "$body_file" ]]; then
		printf 'not captured'
		return
	fi

	LC_ALL=C tr -d '\000\r' <"$body_file" |
		sed -E 's/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g' |
		head -c 240 |
		tr '\n\t' '  '
}

invalid_image_body_has_rejection_marker() {
	local body_file="$1"

	[[ -f "$body_file" ]] && grep -Eiq 'invalid|reject|validation|activation|error' "$body_file"
}

selected_headers() {
	local header_file="$1"

	if [[ ! -f "$header_file" ]]; then
		return
	fi

	while IFS= read -r line; do
		local clean_line="${line%$'\r'}"
		case "$clean_line" in
		[Cc][Oo][Nn][Tt][Ee][Nn][Tt]-[Tt][Yy][Pp][Ee]:* | \
			[Ll][Oo][Cc][Aa][Tt][Ii][Oo][Nn]:* | \
			[Cc][Aa][Cc][Hh][Ee]-[Cc][Oo][Nn][Tt][Rr][Oo][Ll]:*)
			printf '%s\n' "$clean_line"
			;;
		esac
	done <"$header_file"
}

write_device_url_blocker() {
	log "DEVICE_URL status: blocked - DEVICE_URL unavailable"
	log "firmware_ota_status: blocked - DEVICE_URL unavailable"
	log "network_scan: disabled - DEVICE_URL must be explicit"
	log "conclusion: blocked - firmware OTA evidence requires a reachable just-flashed Ultra 205 DEVICE_URL"
}

block_with_reason() {
	local reason="$1"

	log "firmware_ota_status: blocked - ${reason}"
	log "conclusion: blocked - ${reason}"
}

post_image() {
	local id="$1"
	local image_path="$2"
	local route_label="$3"
	local header_file="${out_dir}/${id}.headers.txt"
	local body_file="${out_dir}/${id}.body.txt"
	local error_file="${out_dir}/${id}.curl-error.txt"
	local url="${base_url}/api/system/OTA"

	: >"$header_file"
	: >"$body_file"
	: >"$error_file"

	set +e
	last_http_status="$("$curl_bin" --silent --show-error --max-time 30 --dump-header "$header_file" --output "$body_file" --write-out "%{http_code}" --request POST --data-binary "@${image_path}" "$url" 2>"$error_file")"
	last_curl_status=$?
	set -e
	last_body_snippet="$(body_snippet "$body_file")"

	log "${route_label} route: POST /api/system/OTA"
	log "${route_label} artifact: ${image_path}"
	log "${route_label} status: ${last_http_status}"
	log "${route_label} curl_status: ${last_curl_status}"
	log "${route_label} selected_headers:"
	local headers
	headers="$(selected_headers "$header_file")"
	if [[ -n "$headers" ]]; then
		while IFS= read -r header; do
			log "  ${header}"
		done <<<"$headers"
	else
		log "  none"
	fi
	log "${route_label} body: ${last_body_snippet}"
	if [[ -s "$error_file" ]]; then
		log "${route_label} curl_error: $(body_snippet "$error_file")"
	fi
}

write_invalid_image() {
	local path="$1"

	printf 'phase13 invalid firmware image - must be rejected\n' >"$path"
}

validate_post_ota_markers() {
	local missing=0

	for marker in "firmware_commit=" "reference_commit=" "ota_boot_validation="; do
		if grep -Fq "$marker" "$post_ota_monitor_log"; then
			log "post_ota_marker: ${marker} present"
			continue
		fi

		log "post_ota_marker: ${marker} missing"
		missing=1
	done

	return "$missing"
}

log "phase13_firmware_ota_smoke"
log "manifest: ${manifest}"
log "ota_image: ${ota_image}"
log "port: ${port:-not provided}"
log "monitor_seconds: ${monitor_seconds}"
log "manifest_source_commit: $(manifest_field source_commit)"
log "manifest_reference_commit: $(manifest_field reference_commit)"

if [[ -z "$device_url" || "$device_url" == blocked* ]]; then
	write_device_url_blocker
	exit 0
fi

case "$device_url" in
http://* | https://*) ;;
*)
	log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
	write_device_url_blocker
	exit 0
	;;
esac

if [[ -z "$port" ]]; then
	block_with_reason "detector-approved port unavailable"
	exit 1
fi
if [[ ! -f "$manifest" ]]; then
	block_with_reason "package manifest missing"
	exit 1
fi
if [[ ! -f "$ota_image" ]]; then
	block_with_reason "OTA image missing"
	exit 1
fi
if [[ "$(basename "$ota_image")" != "esp-miner.bin" ]]; then
	block_with_reason "OTA image is not esp-miner.bin"
	exit 1
fi

manifest_ota_path="$(manifest_artifact_field firmware_ota_image path)"
manifest_ota_sha256="$(manifest_artifact_field firmware_ota_image sha256)"
ota_sha256="$(sha256_file "$ota_image")"

log "DEVICE_URL status: provided"
log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
log "network_scan: disabled - using explicit DEVICE_URL only"
log "firmware_ota_manifest_artifact: ${manifest_ota_path}"
log "firmware_ota_manifest_sha256: ${manifest_ota_sha256}"
log "firmware_ota_artifact_sha256: ${ota_sha256}"

if [[ "$manifest_ota_path" != "esp-miner.bin" ]]; then
	block_with_reason "manifest firmware_ota_image is not esp-miner.bin"
	exit 1
fi
if [[ "$manifest_ota_sha256" != "$ota_sha256" ]]; then
	block_with_reason "manifest OTA checksum mismatch"
	exit 1
fi

base_url="${device_url%/}"
readonly base_url

invalid_image="${out_dir}/invalid-firmware.bin"
write_invalid_image "$invalid_image"
invalid_sha256="$(sha256_file "$invalid_image")"
log "invalid image artifact: ${invalid_image}"
log "invalid image checksum: ${invalid_sha256}"
post_image "invalid-firmware-ota" "$invalid_image" "invalid image rejection"
invalid_body_file="${out_dir}/invalid-firmware-ota.body.txt"

if [[ "$last_curl_status" -ne 0 ]]; then
	block_with_reason "invalid image rejection request failed"
	exit 1
fi
if [[ "$last_http_status" == "200" ]]; then
	block_with_reason "invalid image was not rejected"
	exit 1
fi
if ! invalid_image_body_has_rejection_marker "$invalid_body_file"; then
	block_with_reason "invalid image rejection body did not contain an OTA validation marker"
	exit 1
fi

log "invalid image rejection conclusion: captured - not rollback proof"
log "invalid image rejection is not rollback proof"
log ""

post_image "valid-firmware-ota" "$ota_image" "valid OTA"

if [[ "$last_curl_status" -ne 0 ]]; then
	block_with_reason "valid OTA request failed"
	exit 1
fi
if [[ "$last_http_status" != "200" || "$last_body_snippet" != *"Firmware update complete, rebooting now!"* ]]; then
	block_with_reason "valid OTA did not complete"
	exit 1
fi

log "valid OTA conclusion: accepted - reboot monitor required"
log "selected_next_app_partition: unavailable - public route does not expose partition; boot-validation marker required"
log "post_ota_monitor_command: scripts/phase13-monitor-capture.sh --port ${port} --out ${post_ota_monitor_log} --seconds ${monitor_seconds} --no-reset"

"$BASH" "$monitor_capture_script" --port "$port" --out "$post_ota_monitor_log" --seconds "$monitor_seconds" --no-reset >>"$log_file" 2>&1

if ! validate_post_ota_markers; then
	block_with_reason "post-OTA monitor missing required identity or boot-validation markers"
	exit 1
fi

log "firmware_ota_status: passed"
log "conclusion: passed - valid OTA response, reboot identity, and ota_boot_validation marker captured"
