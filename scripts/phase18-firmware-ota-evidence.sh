#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat >&2 <<'USAGE'
usage: phase18-firmware-ota-evidence.sh [--device-url URL | --device-url-from-flash-evidence PATH] --manifest PATH --ota-image PATH --port PATH [--out-dir PATH] [--target-lock-out PATH] [--monitor-seconds N] [--target-lock-only]
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

phase13_script="${PHASE13_FIRMWARE_OTA_SMOKE_SCRIPT:-${script_dir}/phase13-firmware-ota-smoke.sh}"
device_url="${DEVICE_URL:-}"
device_url_source="environment"
if [[ -z "$device_url" ]]; then
	device_url_source="none"
fi
device_url_from_argument=0
flash_evidence_json=""
manifest="docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json"
ota_image="bazel-bin/firmware/bitaxe/esp-miner.bin"
port=""
out_dir="docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence"
target_lock_out=""
monitor_seconds="45"
target_lock_only=0
selected_port_from_flash=""
target_lock_created_from_explicit_input="false"
device_url_lookup_reason=""

while [[ $# -gt 0 ]]; do
	case "$1" in
	--device-url)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		device_url="$2"
		device_url_source="argument"
		device_url_from_argument=1
		shift 2
		;;
	--device-url-from-flash-evidence)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		flash_evidence_json="$2"
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
	--target-lock-out)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		target_lock_out="$2"
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
	--target-lock-only)
		target_lock_only=1
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

if [[ ! "$monitor_seconds" =~ ^[0-9]+$ || "$monitor_seconds" -lt 1 ]]; then
	printf 'monitor-seconds must be a positive integer\n' >&2
	exit 2
fi

if [[ "$device_url_from_argument" -eq 1 && -n "$flash_evidence_json" ]]; then
	printf 'use only one of --device-url or --device-url-from-flash-evidence\n' >&2
	exit 2
fi

if [[ -z "$target_lock_out" ]]; then
	target_lock_out="${out_dir%/}/target-lock.json"
fi

ensure_allowed_write_path() {
	local label="$1"
	local path="$2"

	case "$path" in
	docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence | \
		docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/* | \
		target/phase18-firmware-ota-and-rollback-evidence-dev-raw | \
		target/phase18-firmware-ota-and-rollback-evidence-dev-raw/*)
		return 0
		;;
	esac

	printf '%s must stay under Phase 18 evidence or target raw-evidence paths: %s\n' "$label" "$path" >&2
	exit 2
}

ensure_allowed_write_path "--out-dir" "$out_dir"
ensure_allowed_write_path "--target-lock-out" "$target_lock_out"

firmware_ota_dir="${out_dir%/}/firmware-ota"
ensure_allowed_write_path "firmware OTA out-dir" "$firmware_ota_dir"
mkdir -p "$firmware_ota_dir"
log_file="${firmware_ota_dir}/firmware-ota-smoke.log"

tmp_dir="${out_dir%/}/.phase18-wrapper-tmp"
rm -rf "$tmp_dir"
mkdir -p "$tmp_dir"
pre_log="${tmp_dir}/pre.log"
summary_log="${tmp_dir}/summary.log"
: >"$pre_log"
: >"$summary_log"

cleanup() {
	rm -rf "$tmp_dir"
}
trap cleanup EXIT

log_pre() {
	printf '%s\n' "$*" >>"$pre_log"
}

log_summary() {
	printf '%s\n' "$*" >>"$summary_log"
}

redacted_origin() {
	local url="$1"

	case "$url" in
	http://*) printf 'http://[redacted]' ;;
	https://*) printf 'https://[redacted]' ;;
	"") printf 'not provided' ;;
	*) printf '[invalid-url]' ;;
	esac
}

validate_origin_device_url() {
	local value="$1"
	local rest
	local host

	case "$value" in
	http://*)
		rest="${value#http://}"
		;;
	https://*)
		rest="${value#https://}"
		;;
	*)
		return 1
		;;
	esac

	if [[ -z "$rest" || "$rest" == *"@"* || "$rest" == *"?"* || "$rest" == *"#"* ]]; then
		return 1
	fi
	if [[ "$rest" == *[[:space:]\"\'\<\>]* ]]; then
		return 1
	fi
	if [[ "$rest" == */* ]]; then
		if [[ "${rest#*/}" != "" ]]; then
			return 1
		fi
		host="${rest%/}"
	else
		host="$rest"
	fi
	[[ -n "$host" ]]
}

manifest_field() {
	local field="$1"

	if [[ ! -f "$manifest" ]] || ! command -v python3 >/dev/null 2>&1; then
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

value = data.get(field)
print(value if value else "unavailable")
PY
}

load_device_url_from_flash_evidence() {
	device_url_lookup_reason=""

	if [[ -z "$flash_evidence_json" || ! -f "$flash_evidence_json" ]]; then
		device_url_lookup_reason="missing --device-url-from-flash-evidence"
		return 1
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		device_url_lookup_reason="python3 unavailable"
		return 1
	fi

	local extracted
	if ! extracted="$(python3 - "$flash_evidence_json" <<'PY'
import json
import pathlib
import re
import sys

json_path = pathlib.Path(sys.argv[1])
try:
    data = json.loads(json_path.read_text(encoding="utf-8"))
except Exception:
    print("error=flash evidence JSON is unreadable")
    raise SystemExit(1)

if "flash-monitor" not in str(data.get("command_kind", "")):
    print("error=flash command_kind is not flash-monitor")
    raise SystemExit(1)
if str(data.get("board", "")) != "205":
    print("error=flash board is not 205")
    raise SystemExit(1)
if data.get("trusted_output") is not True:
    print("error=flash trusted_output is not true")
    raise SystemExit(1)

monitor_value = (
    data.get("monitor_log_path")
    or data.get("flash_monitor_log_path")
    or data.get("flash_monitor_log")
    or data.get("log_path")
)
if not monitor_value:
    print("error=monitor log path is missing or unreadable")
    raise SystemExit(1)

monitor_path = pathlib.Path(str(monitor_value))
if not monitor_path.is_file() and not monitor_path.is_absolute():
    maybe_relative = json_path.parent / monitor_path
    if maybe_relative.is_file():
        monitor_path = maybe_relative
if not monitor_path.is_file():
    print("error=monitor log path is missing or unreadable")
    raise SystemExit(1)

content = monitor_path.read_bytes()
urls = sorted({
    match.decode("ascii", errors="ignore").split("=", 1)[1]
    for match in re.findall(rb"device_url=https?://[^\s\"<>]+", content)
})
if len(urls) != 1:
    print("error=monitor log must contain exactly one device_url")
    raise SystemExit(1)

selected_port = data.get("selected_port") or data.get("port") or ""
print(f"device_url={urls[0]}")
print(f"selected_port={selected_port}")
PY
	)"; then
		device_url_lookup_reason="${extracted#error=}"
		if [[ "$device_url_lookup_reason" == "$extracted" ]]; then
			device_url_lookup_reason="trusted flash evidence could not provide a device_url"
		fi
		return 1
	fi

	while IFS='=' read -r key value; do
		case "$key" in
		device_url) device_url="$value" ;;
		selected_port) selected_port_from_flash="$value" ;;
		esac
	done <<<"$extracted"

	if ! validate_origin_device_url "$device_url"; then
		device_url=""
		device_url_lookup_reason="monitor log device_url is not origin-only"
		return 1
	fi

	device_url_source="usb_flash_monitor_log"
	target_lock_created_from_explicit_input="true"
	return 0
}

write_target_lock() {
	local target_status="$1"
	local selected_port="$2"
	local source_commit
	local reference_commit
	local created_from_explicit_input="$target_lock_created_from_explicit_input"

	source_commit="$(manifest_field source_commit)"
	reference_commit="$(manifest_field reference_commit)"
	mkdir -p "$(dirname "$target_lock_out")"

	if [[ "$target_status" == "passed" ]]; then
		created_from_explicit_input="true"
	fi

	python3 - "$target_lock_out" \
		"$target_status" \
		"$device_url_source" \
		"$(redacted_origin "$device_url")" \
		"205" \
		"$selected_port" \
		"$source_commit" \
		"$reference_commit" \
		"$manifest" \
		"$flash_evidence_json" \
		"$created_from_explicit_input" <<'PY'
import json
import sys

(
    path,
    target_status,
    source,
    redacted,
    board,
    selected_port,
    source_commit,
    reference_commit,
    manifest,
    flash_json,
    created_from_explicit_input,
) = sys.argv[1:]

payload = {
    "target_status": target_status,
    "device_url_source": source,
    "device_url_redacted": redacted,
    "board": board,
    "selected_port": selected_port,
    "source_commit": source_commit,
    "reference_commit": reference_commit,
    "manifest": manifest,
    "flash_evidence_json": flash_json,
    "network_scan": "disabled",
    "created_from_explicit_input": created_from_explicit_input == "true",
}
with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

finalize_log() {
	local phase13_status="${1:-}"
	local combined="${tmp_dir}/combined.log"

	{
		cat "$pre_log"
		if [[ -f "$log_file" && -s "$log_file" ]]; then
			printf '\n'
			cat "$log_file"
		fi
		if [[ -s "$summary_log" ]]; then
			printf '\n'
			cat "$summary_log"
		fi
		if [[ -n "$phase13_status" ]]; then
			printf 'phase13_helper_exit_status: %s\n' "$phase13_status"
		fi
	} >"$combined"
	mv "$combined" "$log_file"
}

write_blocked_log() {
	local reason="$1"
	local selected_port="$2"

	write_target_lock "blocked" "$selected_port"
	log_summary "target_status: blocked"
	log_summary "firmware_ota_status: blocked - ${reason}"
	log_summary "invalid_rejection_status: blocked - ${reason}"
	log_summary "valid_ota_status: blocked - ${reason}"
	log_summary "boot_validation_status: blocked - ${reason}"
	log_summary "rollback_status: not claimed - invalid image rejection is not rollback proof"
	log_summary "non_claims: valid OTA, boot validation, destructive rollback, selected partition, and OTAWWW update behavior are not claimed"
	log_summary "conclusion: blocked - ${reason}"
	finalize_log
}

append_claim_summary_from_phase13() {
	local phase13_status="$1"

	if grep -Fq "invalid image rejection conclusion: captured - not rollback proof" "$log_file"; then
		log_summary "invalid_rejection_status: captured - invalid image rejection is not rollback proof"
	else
		log_summary "invalid_rejection_status: blocked - invalid rejection marker absent"
	fi

	if grep -Fq "firmware_ota_status: passed" "$log_file"; then
		log_summary "valid_ota_status: passed"
	else
		log_summary "valid_ota_status: blocked - Phase 13 helper did not pass"
	fi

	if grep -Fq "post_ota_marker: firmware_commit= present" "$log_file" &&
		grep -Fq "post_ota_marker: reference_commit= present" "$log_file" &&
		grep -Fq "post_ota_marker: ota_boot_validation= present" "$log_file"; then
		log_summary "boot_validation_status: passed - firmware_commit=, reference_commit=, and ota_boot_validation= markers captured"
	else
		log_summary "boot_validation_status: blocked - required firmware_commit=, reference_commit=, or ota_boot_validation= marker missing"
	fi

	log_summary "rollback_status: not claimed - invalid image rejection is not rollback proof"
	log_summary "non_claims: destructive rollback, forced rollback, interrupted update, large erase, and OTAWWW update behavior are not claimed"
	if [[ "$phase13_status" -eq 0 ]] && grep -Fq "firmware_ota_status: passed" "$log_file"; then
		log_summary "phase18_firmware_ota_status: passed"
		log_summary "conclusion: passed - Phase 18 wrapper target lock plus Phase 13 valid OTA and boot-validation evidence passed"
	else
		log_summary "phase18_firmware_ota_status: blocked"
		log_summary "conclusion: blocked - Phase 13 helper did not produce passed firmware OTA evidence"
	fi
}

log_pre "phase18_firmware_ota_evidence"
log_pre "manifest: ${manifest}"
log_pre "ota_image: ${ota_image}"
log_pre "port: ${port:-not provided}"
log_pre "out_dir: ${out_dir}"
log_pre "target_lock_out: ${target_lock_out}"
log_pre "monitor_seconds: ${monitor_seconds}"
log_pre "network_scan: disabled"
log_pre "phase13_helper: ${phase13_script}"

if [[ -n "$flash_evidence_json" ]]; then
	if ! load_device_url_from_flash_evidence; then
		log_pre "DEVICE_URL status: blocked - flash evidence device_url unavailable"
		log_pre "device_url_lookup_reason: ${device_url_lookup_reason}"
		write_blocked_log "trusted USB flash-monitor evidence did not provide exactly one origin-only device_url" "${selected_port_from_flash:-$port}"
		exit 0
	fi
fi

if [[ -z "$device_url" ]]; then
	log_pre "DEVICE_URL status: blocked - missing DEVICE_URL"
	write_blocked_log "DEVICE_URL unavailable" "${selected_port_from_flash:-$port}"
	exit 0
fi

if ! validate_origin_device_url "$device_url"; then
	log_pre "DEVICE_URL status: blocked - invalid origin-only DEVICE_URL"
	log_pre "DEVICE_URL sanitized: $(redacted_origin "$device_url")"
	write_blocked_log "DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, or fragment" "${selected_port_from_flash:-$port}"
	exit 0
fi

target_lock_created_from_explicit_input="true"
if [[ -z "$selected_port_from_flash" ]]; then
	selected_port_from_flash="$port"
fi
log_pre "DEVICE_URL status: provided"
log_pre "DEVICE_URL source: ${device_url_source}"
log_pre "DEVICE_URL sanitized: $(redacted_origin "$device_url")"
write_target_lock "passed" "$selected_port_from_flash"
log_pre "target_status: passed"

if [[ "$target_lock_only" -eq 1 ]]; then
	log_summary "firmware_ota_status: target-lock-only - OTA helper not invoked"
	log_summary "invalid_rejection_status: not run - target-lock-only"
	log_summary "valid_ota_status: not run - target-lock-only"
	log_summary "boot_validation_status: not run - target-lock-only"
	log_summary "rollback_status: not claimed - invalid image rejection is not rollback proof"
	log_summary "non_claims: target-lock-only did not upload firmware or exercise rollback"
	log_summary "conclusion: passed - target lock created from explicit Phase 18 input"
	finalize_log
	exit 0
fi

if [[ ! -f "$phase13_script" ]]; then
	log_pre "phase13_helper_status: blocked - helper missing"
	write_blocked_log "phase13-firmware-ota-smoke helper missing" "$selected_port_from_flash"
	exit 1
fi

set +e
"$BASH" "$phase13_script" \
	--device-url "$device_url" \
	--manifest "$manifest" \
	--ota-image "$ota_image" \
	--port "$port" \
	--out-dir "$firmware_ota_dir" \
	--monitor-seconds "$monitor_seconds"
phase13_status=$?
set -e

append_claim_summary_from_phase13 "$phase13_status"
finalize_log "$phase13_status"
exit "$phase13_status"
