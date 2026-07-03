#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat >&2 <<'USAGE'
usage: phase19-recovery-otawww-evidence.sh --manifest PATH --factory-image PATH --ota-image PATH --port PATH --out-dir PATH [--device-url URL | --device-url-from-flash-evidence PATH] [--target-lock-out PATH] [--allow-failed-update] [--allow-large-erase] [--allow-interrupted-ota] [--otawww-gap-only]
USAGE
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

recovery_script="${PHASE16_RECOVERY_REGRESSION_SCRIPT:-${script_dir}/phase16-recovery-regression.sh}"
curl_bin="${CURL_BIN:-curl}"

manifest=""
factory_image=""
ota_image=""
port=""
out_dir=""
target_lock_out=""
device_url=""
device_url_source="none"
device_url_from_argument=0
flash_evidence_json=""
selected_port_from_flash=""
allow_failed_update=0
allow_large_erase=0
allow_interrupted_ota=0
otawww_gap_only=0

while [[ $# -gt 0 ]]; do
	case "$1" in
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
	--target-lock-out)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		target_lock_out="$2"
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
	--otawww-gap-only)
		otawww_gap_only=1
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
	docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence | \
		docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/* | \
		target/phase19-recovery-regression-and-otawww-evidence-dev-raw | \
		target/phase19-recovery-regression-and-otawww-evidence-dev-raw/*)
		return 0
		;;
	esac

	printf '%s must stay under Phase 19 evidence or target raw-evidence paths: %s\n' "$label" "$path" >&2
	exit 2
}

ensure_allowed_write_path "--out-dir" "$out_dir"
ensure_allowed_write_path "--target-lock-out" "$target_lock_out"

recovery_dir="${out_dir%/}/recovery-regression"
otawww_dir="${out_dir%/}/otawww"
ensure_allowed_write_path "recovery regression out-dir" "$recovery_dir"
ensure_allowed_write_path "OTAWWW out-dir" "$otawww_dir"
mkdir -p "$recovery_dir" "$otawww_dir"

log_file="${out_dir%/}/phase19-recovery-otawww-evidence.log"
otawww_gap_log="${otawww_dir}/otawww-gap.log"
: >"$log_file"

log_main() {
	printf '%s\n' "$*" >>"$log_file"
}

log_otawww() {
	printf '%s\n' "$*" >>"$otawww_gap_log"
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

value = data
for part in field.split("."):
    if not isinstance(value, dict) or part not in value:
        print("unavailable")
        raise SystemExit(0)
    value = value[part]
print("unavailable" if value is None else value)
PY
}

load_device_url_from_flash_evidence() {
	if [[ -z "$flash_evidence_json" || ! -f "$flash_evidence_json" ]]; then
		printf 'flash evidence JSON is missing: %s\n' "$flash_evidence_json" >&2
		return 1
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		printf 'python3 is required to parse flash evidence JSON\n' >&2
		return 1
	fi

	local extracted
	if ! extracted="$(
		python3 - "$flash_evidence_json" <<'PY'
import json
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
try:
    data = json.loads(path.read_text(encoding="utf-8"))
except Exception as exc:
    print(f"error=flash evidence JSON is unreadable: {exc}")
    raise SystemExit(1)

command_kind = str(data.get("command_kind", ""))
command = str(data.get("command", ""))
if "flash-monitor" not in command_kind and "flash-monitor" not in command:
    print("error=flash evidence is not a flash-monitor command")
    raise SystemExit(1)
if str(data.get("board", "")) != "205":
    print("error=flash board is not 205")
    raise SystemExit(1)
if data.get("trusted_output") is not True:
    print("error=flash trusted_output is not true")
    raise SystemExit(1)
if str(data.get("redaction_mode", "")).lower() in {"raw", "raw-target", "unredacted"}:
    print("error=flash evidence redaction_mode cannot be raw target")
    raise SystemExit(1)

urls = []

def collect_device_urls(value):
    if isinstance(value, dict):
        for key, child in value.items():
            if str(key) == "device_url" and isinstance(child, str):
                urls.append(child)
            else:
                collect_device_urls(child)
    elif isinstance(value, list):
        for child in value:
            collect_device_urls(child)

collect_device_urls(data)

monitor_value = (
    data.get("monitor_log_path")
    or data.get("flash_monitor_log_path")
    or data.get("flash_monitor_log")
    or data.get("log_path")
)
if monitor_value:
    monitor_path = pathlib.Path(str(monitor_value))
    if not monitor_path.is_file() and not monitor_path.is_absolute():
        maybe_relative = path.parent / monitor_path
        if maybe_relative.is_file():
            monitor_path = maybe_relative
    if monitor_path.is_file():
        content = monitor_path.read_bytes()
        urls.extend(
            match.decode("ascii", errors="ignore").split("=", 1)[1]
            for match in re.findall(rb"device_url=https?://[^\s\"<>]+", content)
        )

unique_urls = sorted(set(urls))
if len(unique_urls) != 1:
    print("error=flash evidence must contain exactly one device_url marker")
    raise SystemExit(1)

selected_port = data.get("selected_port") or data.get("port") or ""
print(f"device_url={unique_urls[0]}")
print(f"selected_port={selected_port}")
PY
	)"; then
		printf '%s\n' "${extracted#error=}" >&2
		return 1
	fi

	while IFS='=' read -r key value; do
		case "$key" in
		device_url) device_url="$value" ;;
		selected_port) selected_port_from_flash="$value" ;;
		esac
	done <<<"$extracted"

	if ! validate_origin_device_url "$device_url"; then
		printf 'flash evidence device_url is not origin-only\n' >&2
		device_url=""
		return 1
	fi

	device_url_source="usb_flash_monitor_log"
	return 0
}

write_target_lock() {
	local target_status="$1"
	local selected_port="$2"
	local source_commit
	local reference_commit

	source_commit="$(manifest_field source_commit)"
	reference_commit="$(manifest_field reference_commit)"
	mkdir -p "$(dirname "$target_lock_out")"

	python3 - "$target_lock_out" \
		"$target_status" \
		"$device_url_source" \
		"$(redacted_origin "$device_url")" \
		"$selected_port" \
		"$source_commit" \
		"$reference_commit" \
		"$manifest" \
		"$flash_evidence_json" <<'PY'
import json
import sys

(
    path,
    target_status,
    source,
    redacted,
    selected_port,
    source_commit,
    reference_commit,
    manifest,
    flash_json,
) = sys.argv[1:]

payload = {
    "target_status": target_status,
    "device_url_source": source,
    "device_url_redacted": redacted,
    "selected_port": selected_port,
    "network_scan": "disabled",
    "source_commit": source_commit,
    "reference_commit": reference_commit,
    "manifest": manifest,
    "flash_evidence_json": flash_json,
}
with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

redacted_snippet() {
	local path="$1"

	if [[ ! -f "$path" ]]; then
		printf 'not captured'
		return
	fi

	LC_ALL=C tr -d '\000\r' <"$path" |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|token)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g' |
		head -c 240 |
		tr '\n\t' '  '
}

run_phase16_recovery() {
	local args=(
		--manifest "$manifest"
		--factory-image "$factory_image"
		--ota-image "$ota_image"
		--port "$port"
		--out-dir "$recovery_dir"
	)

	if [[ -n "$device_url" ]]; then
		args+=(--device-url "$device_url")
	fi
	if [[ "$allow_failed_update" -eq 1 ]]; then
		args+=(--allow-failed-update)
	fi
	if [[ "$allow_large_erase" -eq 1 ]]; then
		args+=(--allow-large-erase)
	fi
	if [[ "$allow_interrupted_ota" -eq 1 ]]; then
		args+=(--allow-interrupted-ota)
	fi

	if [[ ! -f "$recovery_script" ]]; then
		log_main "recovery_helper_status: blocked - helper missing"
		return 1
	fi

	log_main "recovery_helper: scripts/phase16-recovery-regression.sh"
	"$BASH" "$recovery_script" "${args[@]}" >>"$log_file" 2>&1
}

write_otawww_gap_without_target() {
	: >"$otawww_gap_log"
	log_otawww "phase19_recovery_otawww_evidence"
	log_otawww "network_scan: disabled"
	log_otawww "otawww_status: blocked - missing DEVICE_URL"
	log_otawww "otawww_claim: REL-03 gap"
	log_otawww "whole_www_update_proof: absent"
	log_otawww "www_bin_proof: absent - www.bin is package evidence only"
	log_otawww "route_presence_proof: absent - route presence is not update proof"
	log_otawww "wrong_api_input_proof: absent - Wrong API input is not whole-www update proof"
}

run_otawww_gap_probe() {
	local headers="${otawww_dir}/otawww.headers.txt"
	local body="${otawww_dir}/otawww.body.txt"
	local error="${otawww_dir}/otawww.curl-error.txt"
	local empty_payload="${otawww_dir}/empty-otawww-upload.bin"
	local url="${device_url%/}/api/system/OTAWWW"
	local status
	local curl_status

	: >"$headers"
	: >"$body"
	: >"$error"
	: >"$empty_payload"
	: >"$otawww_gap_log"

	set +e
	status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$headers" --output "$body" --write-out "%{http_code}" --request POST --data-binary "@${empty_payload}" "$url" 2>"$error")"
	curl_status=$?
	set -e

	log_otawww "phase19_recovery_otawww_evidence"
	log_otawww "network_scan: disabled"
	log_otawww "otawww_route: POST /api/system/OTAWWW"
	log_otawww "otawww_request: bounded empty POST"
	log_otawww "otawww_curl_status: ${curl_status}"
	log_otawww "otawww_public_status: ${status}"
	log_otawww "otawww_selected_headers: $(redacted_snippet "$headers")"
	log_otawww "otawww_public_body: $(redacted_snippet "$body")"
	if [[ -s "$error" ]]; then
		log_otawww "otawww_curl_error: $(redacted_snippet "$error")"
	fi
	log_otawww "otawww_status: captured - gap evidence only"
	log_otawww "otawww_claim: REL-03 gap"
	log_otawww "whole_www_update_proof: absent"
	log_otawww "www_bin_proof: absent - www.bin is package evidence only"
	log_otawww "route_presence_proof: absent - route presence is not update proof"
	log_otawww "wrong_api_input_proof: absent - Wrong API input is not whole-www update proof"
}

log_allow_flag_status() {
	local flag_name="$1"
	local supplied="$2"

	if [[ "$supplied" -eq 1 ]]; then
		log_main "${flag_name}: supplied"
		return
	fi
	log_main "${flag_name}: omitted"
}

if [[ -n "$flash_evidence_json" ]]; then
	if ! load_device_url_from_flash_evidence; then
		exit 2
	fi
fi

if [[ "$device_url_from_argument" -eq 1 ]] && ! validate_origin_device_url "$device_url"; then
	printf 'DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, fragment, or whitespace\n' >&2
	exit 2
fi

if [[ -n "$device_url" && "$device_url_from_argument" -eq 0 && -z "$flash_evidence_json" ]]; then
	if ! validate_origin_device_url "$device_url"; then
		printf 'DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, fragment, or whitespace\n' >&2
		exit 2
	fi
fi

if [[ -z "$selected_port_from_flash" ]]; then
	selected_port_from_flash="$port"
fi

log_main "phase19_recovery_otawww_evidence"
log_main "manifest: ${manifest}"
log_main "factory_image: ${factory_image}"
log_main "ota_image: ${ota_image}"
log_main "port: ${port}"
log_main "out_dir: ${out_dir}"
log_main "target_lock_out: ${target_lock_out}"
log_main "network_scan: disabled"
log_main "raw_destructive_commands: prohibited"
log_main "raw_write_commands: prohibited"
log_main "interrupted_upload_commands: delegated to Phase 16 helper only"
log_main "rollback_commands: delegated to Phase 16 helper only"
log_main "recovery_helper: scripts/phase16-recovery-regression.sh"
log_main "DEVICE_URL source: ${device_url_source}"
log_main "DEVICE_URL sanitized: $(redacted_origin "$device_url")"
log_allow_flag_status "--allow-failed-update" "$allow_failed_update"
log_allow_flag_status "--allow-large-erase" "$allow_large_erase"
log_allow_flag_status "--allow-interrupted-ota" "$allow_interrupted_ota"
log_allow_flag_status "--otawww-gap-only" "$otawww_gap_only"

if [[ -n "$device_url" ]]; then
	write_target_lock "passed" "$selected_port_from_flash"
	log_main "target_status: passed"
else
	log_main "target_status: blocked - missing DEVICE_URL"
fi

run_phase16_recovery

if [[ -z "$device_url" ]]; then
	write_otawww_gap_without_target
	exit 0
fi

if [[ "$otawww_gap_only" -eq 1 ]]; then
	run_otawww_gap_probe
else
	log_main "otawww_status: not run - --otawww-gap-only not supplied"
	write_otawww_gap_without_target
fi
