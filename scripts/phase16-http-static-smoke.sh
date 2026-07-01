#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s [--device-url URL] [--manifest PATH] [--out-dir PATH]\n' "$(basename "$0")" >&2
}

device_url="${DEVICE_URL:-}"
manifest="bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
out_dir="docs/parity/evidence/phase-16-current-commit-release-evidence-completion/http-static-recovery"
curl_bin="${CURL_BIN:-curl}"

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
	--out-dir)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		out_dir="$2"
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
readonly log_file="${out_dir}/http-static-smoke.log"
: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

sanitize_device_url() {
	local url="$1"

	case "$url" in
	http://*) printf 'http://[redacted]' ;;
	https://*) printf 'https://[redacted]' ;;
	*) printf '[invalid-url]' ;;
	esac
}

sanitize_target() {
	local url="$1"
	local scheme
	local path

	case "$url" in
	http://*)
		scheme="http"
		;;
	https://*)
		scheme="https"
		;;
	*)
		printf '[invalid-url]'
		return
		;;
	esac

	path="/${url#*://*/}"
	if [[ "$url" == "${scheme}://"*"/" ]]; then
		path="/${url#*://*/}"
	else
		path="/"
	fi
	if [[ "$url" == "${scheme}://"*"/"* ]]; then
		path="/${url#*://*/}"
	fi

	printf '%s://[redacted]%s' "$scheme" "$path"
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
	done <"$header_file"
}

redacted_body_snippet() {
	local body_file="$1"

	LC_ALL=C tr -d '\000\r' <"$body_file" |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g' |
		head -c 240 |
		tr '\n\t' '  '
}

body_contains_marker() {
	local body_file="$1"
	local marker="$2"

	grep -Fq "$marker" "$body_file"
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

ota_validation_path_present() {
	local header_file="$1"
	local body_file="$2"

	if selected_headers "$header_file" | grep -Eiq 'Content-Type: (text/plain|application/json)'; then
		if grep -Eiq 'Protocol Error|Write Error|Validation Error|Validation / Activation Error|Not allowed in AP mode|Firmware update|OTA' "$body_file"; then
			return 0
		fi
	fi

	return 1
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
	local url="${base_url}${path}"

	: >"$header_file"
	: >"$body_file"
	: >"$error_file"

	set +e
	local actual_status
	if [[ "$method" == "POST" ]]; then
		actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$header_file" --output "$body_file" --write-out "%{http_code}" --request POST --data-binary "" "$url" 2>"$error_file")"
	else
		actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$header_file" --output "$body_file" --write-out "%{http_code}" "$url" 2>"$error_file")"
	fi
	local curl_status=$?
	set -e

	local route_conclusion="passed"
	local actual_result="matched"
	if [[ "$curl_status" -ne 0 ]]; then
		route_conclusion="blocked"
		actual_result="curl_error"
	elif ! status_matches "$expected_status" "$actual_status"; then
		route_conclusion="blocked"
		actual_result="unexpected_status"
	elif [[ "$expected_status" == "ota-route-present" ]]; then
		if ota_validation_path_present "$header_file" "$body_file"; then
			actual_result="firmware_ota_validation_path"
			ota_route_presence="passed"
		else
			route_conclusion="blocked"
			actual_result="missing_ota_validation_marker"
			ota_route_presence="blocked"
		fi
	elif ! markers_match "$body_file" "$markers"; then
		route_conclusion="blocked"
		actual_result="missing_expected_marker"
	fi

	if [[ "$route_conclusion" != "passed" ]]; then
		any_blocked=1
	fi

	log "route: ${method} ${path}"
	log "method: ${method}"
	log "path: ${path}"
	log "sanitized_target: $(sanitize_target "$url")"
	log "expected_result: ${expectation}"
	log "actual_status: ${actual_status}"
	log "curl_status: ${curl_status}"
	log "selected_headers:"
	local headers
	headers="$(selected_headers "$header_file")"
	if [[ -n "$headers" ]]; then
		while IFS= read -r header; do
			log "  ${header}"
		done <<<"$headers"
	else
		log "  none"
	fi
	local snippet
	snippet="$(redacted_body_snippet "$body_file")"
	if [[ -z "$snippet" ]]; then
		snippet="[empty-body]"
	fi
	log "redacted_body_snippet: ${snippet}"
	if [[ -s "$error_file" ]]; then
		log "curl_error: $(redacted_body_snippet "$error_file")"
	fi
	log "actual_result: ${actual_result}"
	if [[ "$expected_status" == "ota-route-present" ]]; then
		log "ota_route_presence: ${ota_route_presence}"
	fi
	if [[ "$path" == "/api/system/OTAWWW" ]]; then
		log "otawww_rel03_status: deferred"
	fi
	log "route_conclusion: ${route_conclusion}"
	log ""
}

log "phase16_http_static_smoke"
log "manifest: ${manifest}"
log "manifest_source_commit: $(manifest_field source_commit)"
log "manifest_reference_commit: $(manifest_field reference_commit)"
log "network_scan: disabled - using explicit DEVICE_URL only"

if [[ -z "$device_url" ]]; then
	log "DEVICE_URL status: blocked - missing DEVICE_URL"
	log "http_static_status: blocked"
	log "conclusion: blocked - live HTTP/static/recovery evidence requires an explicit DEVICE_URL"
	exit 0
fi

case "$device_url" in
http://* | https://*) ;;
*)
	log "DEVICE_URL status: blocked - invalid DEVICE_URL scheme"
	log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
	log "http_static_status: blocked"
	log "conclusion: blocked - DEVICE_URL must start with http:// or https://"
	exit 0
	;;
esac

base_url="${device_url%/}"
readonly base_url
any_blocked=0
ota_route_presence="not-run"

log "DEVICE_URL status: provided"
log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
log ""

probe_route "root" "GET" "/" "200" "AxeOS unavailable|Open recovery|Release metadata" "200 with AxeOS unavailable, Open recovery, and Release metadata"
probe_route "app-css-gz" "GET" "/assets/app.css.gz" "200" "" "200 with static CSS headers"
probe_route "missing-static" "GET" "/phase16-missing-static" "302" "Redirect to the captive portal" "missing static redirect with captive portal body"
probe_route "recovery" "GET" "/recovery" "200" "AxeOS Recovery|Response:" "200 with AxeOS Recovery and Response:"
probe_route "system-info" "GET" "/api/system/info" "200" "" "known API route coexists with static wildcard"
probe_route "unknown-api" "GET" "/api/phase16-unknown" "404" "{\"error\":\"unknown route\"}" "unknown API JSON 404 body"
probe_route "api-ws" "GET" "/api/ws" "websocket-no-upgrade" "" "bounded WebSocket route coexistence response, not static wildcard"
probe_route "api-ws-live" "GET" "/api/ws/live" "websocket-no-upgrade" "" "bounded live WebSocket route coexistence response, not static wildcard"
probe_route "firmware-ota" "POST" "/api/system/OTA" "ota-route-present" "" "firmware OTA route presence without static fallback"
probe_route "otawww" "POST" "/api/system/OTAWWW" "400" "Wrong API input" "OTAWWW REL-03 gap response"

if [[ "$any_blocked" -eq 0 ]]; then
	log "http_static_status: passed"
	log "conclusion: passed - all HTTP/static/recovery/API/OTA probes matched expected live evidence markers"
else
	log "http_static_status: blocked"
	log "conclusion: blocked - one or more HTTP/static/recovery/API/OTA probes did not match expected live evidence markers"
fi
