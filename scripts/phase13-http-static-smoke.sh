#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s [--device-url URL] [--manifest PATH] [--out-dir PATH]\n' "$(basename "$0")" >&2
}

device_url="${DEVICE_URL:-}"
manifest="bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
out_dir="docs/parity/evidence/phase-13-final-ultra-205-release-evidence/http-static-recovery"
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

	LC_ALL=C tr -d '\000\r' <"$body_file" \
		| head -c 240 \
		| sed -E 's/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g' \
		| tr '\n\t' '  '
}

body_contains_marker() {
	local body_file="$1"
	local marker="$2"

	grep -Fq "$marker" "$body_file"
}

status_matches() {
	local expected="$1"
	local actual="$2"

	if [[ "$expected" == "any-non-static" ]]; then
		[[ "$actual" != "000" && "$actual" != "302" && "$actual" != "404" ]]
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
	if [[ "$curl_status" -ne 0 ]]; then
		route_conclusion="blocked"
	elif ! status_matches "$expected_status" "$actual_status"; then
		route_conclusion="blocked"
	elif ! markers_match "$body_file" "$markers"; then
		route_conclusion="blocked"
	fi

	if [[ "$route_conclusion" != "passed" ]]; then
		any_blocked=1
	fi

	log "route: ${method} ${path}"
	log "expected: ${expectation}"
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
	log "redacted_body_snippet: ${snippet}"
	if [[ -s "$error_file" ]]; then
		log "curl_error: $(redacted_body_snippet "$error_file")"
	fi
	log "route_conclusion: ${route_conclusion}"
	log ""
}

log "phase13_http_static_smoke"
log "manifest: ${manifest}"
log "manifest_source_commit: $(manifest_field source_commit)"
log "manifest_reference_commit: $(manifest_field reference_commit)"

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

log "DEVICE_URL status: provided"
log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
log "network_scan: disabled - using explicit DEVICE_URL only"
log ""

probe_route "root" "GET" "/" "200" "AxeOS unavailable|Open recovery|Release metadata" "200 with AxeOS unavailable, Open recovery, and Release metadata"
probe_route "app-css-gz" "GET" "/assets/app.css.gz" "200" "" "200 with static CSS headers"
probe_route "missing-static" "GET" "/phase13-missing-static" "302" "Redirect to the captive portal" "missing static redirect with captive portal body"
probe_route "recovery" "GET" "/recovery" "200" "AxeOS Recovery|Response:" "200 with AxeOS Recovery and Response:"
probe_route "system-info" "GET" "/api/system/info" "200" "" "known API route coexists with static wildcard"
probe_route "unknown-api" "GET" "/api/phase13-unknown" "404" "{\"error\":\"unknown route\"}" "unknown API JSON 404 body"
probe_route "api-ws" "GET" "/api/ws" "any-non-static" "" "bounded WebSocket route coexistence response, not static wildcard"
probe_route "api-ws-live" "GET" "/api/ws/live" "any-non-static" "" "bounded live WebSocket route coexistence response, not static wildcard"
probe_route "otawww" "POST" "/api/system/OTAWWW" "400" "Wrong API input" "OTAWWW REL-03 gap response"

if [[ "$any_blocked" -eq 0 ]]; then
	log "http_static_status: passed"
	log "conclusion: passed - all HTTP/static/recovery probes matched expected live evidence markers"
else
	log "http_static_status: blocked"
	log "conclusion: blocked - one or more HTTP/static/recovery probes did not match expected live evidence markers"
fi
