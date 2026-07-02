#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s [--device-url URL] [--manifest PATH] [--flash-evidence-json PATH] [--out-dir PATH] [--target-lock-out PATH] [--curl-bin PATH]\n' "$(basename "$0")" >&2
}

device_url="${DEVICE_URL:-}"
device_url_source="environment"
if [[ -z "$device_url" ]]; then
	device_url_source="none"
fi
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

json_field() {
	local path="$1"
	local field="$2"

	if [[ ! -f "$path" ]]; then
		printf ''
		return
	fi
	if ! command -v python3 >/dev/null 2>&1; then
		printf ''
		return
	fi

	python3 - "$path" "$field" <<'PY'
import json
import sys

path, field = sys.argv[1], sys.argv[2]
try:
    with open(path, "r", encoding="utf-8") as handle:
        value = json.load(handle).get(field)
except Exception:
    raise SystemExit(0)

if isinstance(value, bool):
    print("true" if value else "false")
elif value is not None:
    print(value)
PY
}

redacted_origin() {
	local url="$1"

	case "$url" in
	http://*) printf 'http://[redacted]' ;;
	https://*) printf 'https://[redacted]' ;;
	*) printf '[invalid-url]' ;;
	esac
}

redacted_target() {
	local url="$1"
	local scheme
	local rest
	local path="/"

	case "$url" in
	http://*)
		scheme="http"
		rest="${url#http://}"
		;;
	https://*)
		scheme="https"
		rest="${url#https://}"
		;;
	*)
		printf '[invalid-url]'
		return
		;;
	esac

	if [[ "$rest" == */* ]]; then
		path="/${rest#*/}"
	fi
	printf '%s://[redacted]%s' "$scheme" "$path"
}

validate_origin_device_url() {
	local value="$1"
	local rest

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
	if [[ "$rest" == */* && "$rest" != */ ]]; then
		return 1
	fi
	if [[ "$rest" == "/" ]]; then
		return 1
	fi
	return 0
}

redact_stream() {
	LC_ALL=C tr -d '\000\r' |
		sed -E 's/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|token|apiKey|password|nvsSecret|secret)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s#https?://[^[:space:]"<>]+#[redacted-url]#g; s#wss?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )([0-9]{1,3}\.){3}[0-9]{1,3}/\1[redacted-ip]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g'
}

write_redacted_artifact() {
	local source_file="$1"
	local artifact_file="$2"

	redact_stream <"$source_file" | head -c 4000 >"$artifact_file"
}

redacted_snippet() {
	local source_file="$1"

	redact_stream <"$source_file" |
		head -c 1000 |
		tr '\n\t' '  '
}

selected_headers() {
	local header_file="$1"

	while IFS= read -r line; do
		local clean_line="${line%$'\r'}"
		case "$clean_line" in
		[Cc][Oo][Nn][Tt][Ee][Nn][Tt]-[Tt][Yy][Pp][Ee]:* | \
			[Ll][Oo][Cc][Aa][Tt][Ii][Oo][Nn]:* | \
			[Cc][Aa][Cc][Hh][Ee]-[Cc][Oo][Nn][Tt][Rr][Oo][Ll]:* | \
			[Cc][Oo][Nn][Tt][Ee][Nn][Tt]-[Ee][Nn][Cc][Oo][Dd][Ii][Nn][Gg]:*)
			printf '%s\n' "$clean_line"
			;;
		esac
	done <"$header_file" | redact_stream
}

header_contains() {
	local header_file="$1"
	local pattern="$2"

	selected_headers "$header_file" | grep -Eiq "$pattern"
}

body_contains_marker() {
	local body_file="$1"
	local marker="$2"

	grep -Fq "$marker" "$body_file"
}

markers_match() {
	local body_file="$1"
	local markers="$2"

	if [[ -z "$markers" ]]; then
		return 0
	fi

	local marker
	local old_ifs="$IFS"
	IFS='|'
	for marker in $markers; do
		if ! body_contains_marker "$body_file" "$marker"; then
			IFS="$old_ifs"
			return 1
		fi
	done
	IFS="$old_ifs"
	return 0
}

status_matches() {
	local expected="$1"
	local actual="$2"

	if [[ "$expected" == "websocket-no-upgrade" ]]; then
		[[ "$actual" == "400" || "$actual" == "426" ]]
		return
	fi
	if [[ "$expected" == "ota-route-present" ]]; then
		[[ "$actual" == "400" || "$actual" == "409" || "$actual" == "413" || "$actual" == "415" || "$actual" == "422" || "$actual" == "500" ]]
		return
	fi
	[[ "$expected" == "$actual" ]]
}

ota_validation_path_present() {
	local body_file="$1"

	grep -Eiq 'Protocol Error|Write Error|Validation Error|Validation / Activation Error|Not allowed in AP mode|Firmware update|OTA|invalid|image|upload' "$body_file"
}

route_specific_markers_match() {
	local route_id="$1"
	local header_file="$2"
	local body_file="$3"

	case "$route_id" in
	app-css-gz)
		header_contains "$header_file" '^Content-Type:' &&
			header_contains "$header_file" '^Content-Encoding:[[:space:]]*gzip' &&
			header_contains "$header_file" '^Cache-Control:'
		;;
	missing-static)
		header_contains "$header_file" '^Location:[[:space:]]*/$'
		;;
	system-info)
		grep -Eq '^[[:space:]]*\{' "$body_file" &&
			grep -Fq '205' "$body_file" &&
			(grep -Fq 'BM1366' "$body_file" || grep -Fq 'Ultra' "$body_file")
		;;
	firmware-ota)
		ota_validation_path_present "$body_file"
		;;
	*)
		return 0
		;;
	esac
}

write_target_lock() {
	local target_status="$1"
	local lock_selected_port="$2"

	mkdir -p "$(dirname "$target_lock_out")"
	python3 - "$target_lock_out" \
		"$target_status" \
		"$device_url_source" \
		"$(redacted_origin "$device_url")" \
		"205" \
		"$lock_selected_port" \
		"$manifest_source_commit" \
		"$manifest_reference_commit" \
		"$manifest" \
		"$flash_evidence_json" <<'PY'
import json
import sys

path, target_status, source, redacted, board, selected_port, source_commit, reference_commit, manifest, flash_json = sys.argv[1:]
payload = {
    "target_status": target_status,
    "device_url_source": source,
    "device_url_redacted": redacted,
    "board": board,
    "source_commit": source_commit,
    "reference_commit": reference_commit,
    "manifest": manifest,
    "flash_evidence_json": flash_json,
    "network_scan": "disabled",
    "created_from_explicit_input": True,
}
if selected_port:
    payload["selected_port"] = selected_port
with open(path, "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

identity_preflight_passes() {
	if [[ ! -f "$manifest" ]]; then
		identity_block_reason="missing package manifest"
		return 1
	fi
	if [[ -z "$manifest_source_commit" || -z "$manifest_reference_commit" ]]; then
		identity_block_reason="manifest missing source_commit or reference_commit"
		return 1
	fi
	if [[ -z "$flash_evidence_json" || ! -f "$flash_evidence_json" ]]; then
		identity_block_reason="missing --flash-evidence-json"
		return 1
	fi

	local command_kind
	local board
	local trusted_output
	local firmware_commit
	local reference_commit
	local observed_firmware_commit
	local observed_reference_commit

	command_kind="$(json_field "$flash_evidence_json" command_kind)"
	board="$(json_field "$flash_evidence_json" board)"
	trusted_output="$(json_field "$flash_evidence_json" trusted_output)"
	firmware_commit="$(json_field "$flash_evidence_json" firmware_commit)"
	reference_commit="$(json_field "$flash_evidence_json" reference_commit)"
	observed_firmware_commit="$(json_field "$flash_evidence_json" observed_firmware_commit)"
	observed_reference_commit="$(json_field "$flash_evidence_json" observed_reference_commit)"

	if [[ "$command_kind" != *"flash-monitor"* ]]; then
		identity_block_reason="flash command_kind is not flash-monitor"
		return 1
	fi
	if [[ "$board" != "205" ]]; then
		identity_block_reason="flash board is not 205"
		return 1
	fi
	if [[ "$trusted_output" != "true" ]]; then
		identity_block_reason="flash trusted_output is not true"
		return 1
	fi
	if [[ "$firmware_commit" != "$manifest_source_commit" ]]; then
		identity_block_reason="flash firmware_commit does not match manifest source_commit"
		return 1
	fi
	if [[ "$reference_commit" != "$manifest_reference_commit" ]]; then
		identity_block_reason="flash reference_commit does not match manifest reference_commit"
		return 1
	fi
	if [[ "$observed_reference_commit" != "$manifest_reference_commit" ]]; then
		identity_block_reason="observed_reference_commit does not match manifest reference_commit"
		return 1
	fi
	if [[ "$observed_firmware_commit" != "$manifest_source_commit" ]]; then
		if [[ ${#observed_firmware_commit} -lt 12 || "$manifest_source_commit" != "$observed_firmware_commit"* ]]; then
			identity_block_reason="observed_firmware_commit is not the manifest source_commit or a 12+ character prefix"
			return 1
		fi
	fi

	return 0
}

probe_route() {
	local id="$1"
	local method="$2"
	local path="$3"
	local expected_status="$4"
	local markers="$5"
	local expectation="$6"
	local header_file="${out_dir}/${id}.headers.txt"
	local body_file="${out_dir}/${id}.body.txt"
	local error_file="${out_dir}/${id}.curl-error.txt"
	local raw_header="${tmp_dir}/${id}.headers.raw"
	local raw_body="${tmp_dir}/${id}.body.raw"
	local raw_error="${tmp_dir}/${id}.curl-error.raw"
	local url="${base_url}${path}"

	: >"$raw_header"
	: >"$raw_body"
	: >"$raw_error"

	set +e
	local actual_status
	if [[ "$method" == "POST" ]]; then
		actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$raw_header" --output "$raw_body" --write-out "%{http_code}" --request POST --data-binary "" "$url" 2>"$raw_error")"
	else
		actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$raw_header" --output "$raw_body" --write-out "%{http_code}" "$url" 2>"$raw_error")"
	fi
	local curl_status=$?
	set -e

	selected_headers "$raw_header" >"$header_file"
	write_redacted_artifact "$raw_body" "$body_file"
	write_redacted_artifact "$raw_error" "$error_file"

	local route_conclusion="passed"
	local actual_result="matched"
	if [[ "$curl_status" -ne 0 ]]; then
		route_conclusion="blocked"
		actual_result="curl_error"
	elif ! status_matches "$expected_status" "$actual_status"; then
		route_conclusion="blocked"
		actual_result="unexpected_status"
	elif ! markers_match "$raw_body" "$markers"; then
		route_conclusion="blocked"
		actual_result="missing_expected_marker"
	elif ! route_specific_markers_match "$id" "$raw_header" "$raw_body"; then
		route_conclusion="blocked"
		actual_result="missing_route_specific_marker"
	fi

	if [[ "$route_conclusion" != "passed" ]]; then
		any_blocked=1
	fi

	log "route: ${method} ${path}"
	log "method: ${method}"
	log "path: ${path}"
	log "sanitized_target: $(redacted_target "$url")"
	log "expected_result: ${expectation}"
	log "actual_status: ${actual_status}"
	log "curl_status: ${curl_status}"
	log "selected_headers:"
	local headers
	headers="$(selected_headers "$raw_header")"
	if [[ -n "$headers" ]]; then
		while IFS= read -r header; do
			log "  ${header}"
		done <<<"$headers"
	else
		log "  none"
	fi
	local snippet
	snippet="$(redacted_snippet "$raw_body")"
	if [[ -z "$snippet" ]]; then
		snippet="[empty-body]"
	fi
	log "redacted_body_snippet: ${snippet}"
	if [[ -s "$raw_error" ]]; then
		log "curl_error: $(redacted_snippet "$raw_error")"
	fi
	log "actual_result: ${actual_result}"
	case "$id" in
	system-info)
		if [[ "$route_conclusion" == "passed" ]]; then
			log "system_info_device_marker: passed"
		else
			log "system_info_device_marker: blocked"
		fi
		;;
	api-ws | api-ws-live)
		if [[ "$route_conclusion" == "passed" ]]; then
			log "websocket_no_upgrade_claim: route-coexistence-only"
		fi
		;;
	firmware-ota)
		if [[ "$route_conclusion" == "passed" ]]; then
			log "ota_route_presence_claim: route-presence-only"
		else
			log "ota_route_presence_claim: blocked"
		fi
		log "ota_non_claims: valid OTA upload, invalid image rejection, reboot, rollback, selected partition, boot validation not claimed"
		;;
	otawww)
		log "otawww_rel03_status: deferred"
		;;
	esac
	log "route_conclusion: ${route_conclusion}"
	log ""
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

selected_port="$(json_field "$flash_evidence_json" selected_port)"
readonly selected_port
write_target_lock "passed" "$selected_port"

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
