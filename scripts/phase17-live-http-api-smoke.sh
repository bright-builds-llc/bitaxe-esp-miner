#!/usr/bin/env bash
set -euo pipefail

resolve_phase17_script_dir() {
	local direct_dir
	direct_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

	local candidate_dir
	for candidate_dir in \
		"$direct_dir" \
		"${BASH_SOURCE[0]}.runfiles/_main/scripts" \
		"${RUNFILES_DIR:-}/_main/scripts"; do
		if [[ -f "${candidate_dir}/phase17-live-http-api-target.sh" ]] &&
			[[ -f "${candidate_dir}/phase17-live-http-api-probe.sh" ]] &&
			[[ -f "${candidate_dir}/phase17-live-http-api-json.py" ]]; then
			printf '%s\n' "$candidate_dir"
			return 0
		fi
	done

	printf 'phase17 helpers unavailable\n' >&2
	return 1
}

phase17_script_dir="$(resolve_phase17_script_dir)" || exit 1
readonly phase17_script_dir
phase17_json_helper="${phase17_script_dir}/phase17-live-http-api-json.py"
readonly phase17_json_helper
# shellcheck source=scripts/phase17-live-http-api-target.sh
source "${phase17_script_dir}/phase17-live-http-api-target.sh"
# shellcheck source=scripts/phase17-live-http-api-probe.sh
source "${phase17_script_dir}/phase17-live-http-api-probe.sh"

usage() {
	printf 'usage: %s [--device-url URL] [--use-flash-log-device-url] [--manifest PATH] [--flash-evidence-json PATH] [--out-dir PATH] [--target-lock-out PATH] [--curl-bin PATH]\n' "$(basename "$0")" >&2
}

device_url="${DEVICE_URL:-}"
device_url_source="environment"
if [[ -z "$device_url" ]]; then
	device_url_source="none"
fi
use_flash_log_device_url=0
manifest="bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
flash_evidence_json=""
out_dir="docs/parity/evidence/phase-17-live-http-api-and-static-evidence/http-static-api"
target_lock_out="docs/parity/evidence/phase-17-live-http-api-and-static-evidence/target-lock.json"
curl_bin="${CURL_BIN:-curl}"

while [[ $# -gt 0 ]]; do
	case "$1" in
	--device-url)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		device_url="$2"
		device_url_source="argument"
		shift 2
		;;
	--use-flash-log-device-url)
		use_flash_log_device_url=1
		shift
		;;
	--manifest)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		manifest="$2"
		shift 2
		;;
	--flash-evidence-json)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		flash_evidence_json="$2"
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
	--curl-bin)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		curl_bin="$2"
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

mkdir -p "$out_dir"
readonly log_file="${out_dir}/http-static-api.log"
: >"$log_file"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/phase17-live-http-api-smoke.XXXXXX")"
readonly tmp_dir

cleanup() {
	rm -rf "$tmp_dir"
}
trap cleanup EXIT

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

log "phase17_live_http_api_smoke"
log "manifest: ${manifest}"
manifest_source_commit="$(json_field "$manifest" source_commit)"
readonly manifest_source_commit
manifest_reference_commit="$(json_field "$manifest" reference_commit)"
readonly manifest_reference_commit
log "manifest_source_commit: ${manifest_source_commit:-unavailable}"
log "manifest_reference_commit: ${manifest_reference_commit:-unavailable}"
log "flash_evidence_json: ${flash_evidence_json:-missing}"
log "network_scan: disabled"

if [[ -z "$device_url" && "$use_flash_log_device_url" == "1" ]]; then
	if ! load_device_url_from_flash_evidence; then
		log "DEVICE_URL status: blocked - flash log device_url unavailable"
		log "device_url_lookup_reason: ${device_url_lookup_reason}"
		log "target_status: blocked"
		log "http_static_api_status: blocked"
		log "conclusion: blocked - trusted USB flash-monitor log must contain exactly one origin-only device_url"
		exit 0
	fi
fi

if [[ -z "$device_url" ]]; then
	log "DEVICE_URL status: blocked - missing DEVICE_URL"
	log "target_status: blocked"
	log "http_static_api_status: blocked"
	log "conclusion: blocked - live HTTP/static/API evidence requires an explicit origin-only DEVICE_URL"
	exit 0
fi

if ! validate_origin_device_url "$device_url"; then
	log "DEVICE_URL status: blocked - invalid origin-only DEVICE_URL"
	log "DEVICE_URL sanitized: $(redacted_origin "$device_url")"
	log "target_status: blocked"
	log "http_static_api_status: blocked"
	log "conclusion: blocked - DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, or fragment"
	exit 0
fi

base_url="${device_url%/}"
readonly base_url

log "DEVICE_URL status: provided"
log "DEVICE_URL source: ${device_url_source}"
log "DEVICE_URL sanitized: $(redacted_origin "$device_url")"

identity_block_reason=""
if ! identity_preflight_passes; then
	log "identity_status: blocked"
	log "identity_block_reason: ${identity_block_reason}"
	log "target_status: blocked"
	log "http_static_api_status: blocked"
	log "conclusion: blocked - package and flash identity must match before live route probes"
	exit 0
fi

maybe_selected_port="$(json_field "$flash_evidence_json" selected_port)"
if [[ -z "$maybe_selected_port" ]]; then
	maybe_selected_port="$(json_field "$flash_evidence_json" port)"
fi
readonly maybe_selected_port
write_target_lock "passed" "$maybe_selected_port"

any_blocked=0

log "identity_status: passed"
log "target_status: passed"
log ""

# D-08 route set: GET /, GET /assets/app.css.gz, GET /phase17-missing-static,
# GET /recovery, GET /api/system/info, GET /api/phase17-unknown, GET /api/ws,
# GET /api/ws/live, POST /api/system/OTA, POST /api/system/OTAWWW.
probe_route "root" "GET" "/" "200" "AxeOS unavailable|Open recovery|Release metadata" "200 with AxeOS unavailable, Open recovery, and Release metadata"
probe_route "app-css-gz" "GET" "/assets/app.css.gz" "200" "" "200 with Content-Type, Content-Encoding gzip, and Cache-Control"
probe_route "missing-static" "GET" "/phase17-missing-static" "302" "Redirect to the captive portal" "302 Location / with captive portal body"
probe_route "recovery" "GET" "/recovery" "200" "AxeOS Recovery|Response:" "200 with AxeOS Recovery and Response:"
probe_route "system-info" "GET" "/api/system/info" "200" "" "200 JSON-like current-device body containing 205 and BM1366 or Ultra"
probe_route "unknown-api" "GET" "/api/phase17-unknown" "404" "{\"error\":\"unknown route\"}" "unknown API JSON 404 body"
probe_route "api-ws" "GET" "/api/ws" "websocket-no-upgrade" "" "400 or 426 WebSocket no-upgrade route coexistence response"
probe_route "api-ws-live" "GET" "/api/ws/live" "websocket-no-upgrade" "" "400 or 426 live WebSocket no-upgrade route coexistence response"
probe_route "firmware-ota" "POST" "/api/system/OTA" "ota-route-present" "" "firmware OTA route presence and validation-path reachability only"
probe_route "otawww" "POST" "/api/system/OTAWWW" "400" "Wrong API input" "OTAWWW Wrong API input fail-closed gap response"

if [[ "$any_blocked" -eq 0 ]]; then
	log "http_static_api_status: passed"
	log "conclusion: passed - all Phase 17 HTTP/static/API route probes matched expected live evidence markers"
else
	log "http_static_api_status: blocked"
	log "conclusion: blocked - one or more Phase 17 HTTP/static/API route probes did not match expected live evidence markers"
fi
