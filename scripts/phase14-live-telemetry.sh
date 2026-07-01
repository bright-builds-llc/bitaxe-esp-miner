#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH [--device-url URL] [--curl-bin PATH]\n' "$(basename "$0")" >&2
}

manifest=""
out_dir=""
device_url="${DEVICE_URL:-}"
curl_bin="${CURL_BIN:-curl}"
readonly surface="live-api-websocket-telemetry"

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

if [[ -z "$out_dir" ]]; then
	usage
	exit 2
fi

mkdir -p "$out_dir"
readonly log_file="${out_dir}/live-telemetry.log"
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

redact_text() {
	LC_ALL=C tr -d '\000\r' |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|apiToken|apiKey|token)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g'
}

redacted_body_snippet() {
	local body_file="$1"

	redact_text <"$body_file" |
		head -c 320 |
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
			printf '%s\n' "$clean_line" | redact_text
			;;
		esac
	done <"$header_file"
}

run_safety_allow() {
	local allowed_command="scripts/phase14-live-telemetry.sh --manifest ${manifest} --out-dir ${out_dir}"

	if [[ -n "${PHASE14_SAFETY_ALLOW_BIN:-}" ]]; then
		"$PHASE14_SAFETY_ALLOW_BIN" \
			--manifest "$manifest" \
			--surface "$surface" \
			--allowed-command "$allowed_command"
		return
	fi

	bazel run //tools/parity:report -- safety-allow \
		--manifest "$manifest" \
		--surface "$surface" \
		--allowed-command "$allowed_command"
}

blocked_device_url() {
	local reason="$1"

	log "DEVICE_URL status: blocked - ${reason}"
	log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
	log "api_telemetry_status: blocked"
	log "websocket_frame_status: pending - maintained WebSocket client unavailable"
	log "network_scan: disabled - DEVICE_URL must be explicit"
	log "conclusion: blocked - live API/WebSocket telemetry requires an explicit reachable DEVICE_URL and maintained WebSocket client"
}

safety_fields_status() {
	local body_file="$1"

	if grep -Eq '"(power|voltage|fanrpm|temp|vrTemp|hashRate|bestDiff)"[[:space:]]*:' "$body_file"; then
		printf 'observed'
		return
	fi

	if grep -Eq 'safety_telemetry_unavailable|hardware_evidence_pending' "$body_file"; then
		printf 'observed-unavailable'
		return
	fi

	printf 'absent'
}

probe_route() {
	local id="$1"
	local path="$2"
	local header_file="${out_dir}/${id}.headers.txt"
	local body_file="${out_dir}/${id}.body.txt"
	local error_file="${out_dir}/${id}.curl-error.txt"
	local url="${base_url}${path}"

	: >"$header_file"
	: >"$body_file"
	: >"$error_file"

	set +e
	local actual_status
	actual_status="$("$curl_bin" --silent --show-error --max-time 10 --dump-header "$header_file" --output "$body_file" --write-out "%{http_code}" "$url" 2>"$error_file")"
	local curl_status=$?
	set -e

	local route_conclusion="passed"
	if [[ "$curl_status" -ne 0 ]]; then
		route_conclusion="blocked"
	elif [[ "$actual_status" == "000" ]]; then
		route_conclusion="blocked"
	elif [[ "$id" == "system_info" && "$actual_status" != "200" ]]; then
		route_conclusion="blocked"
	elif [[ "$id" != "system_info" && "$actual_status" != "400" && "$actual_status" != "426" ]]; then
		route_conclusion="blocked"
	fi

	log "route: GET ${path}"
	log "${id}_status_code: ${actual_status}"
	log "${id}_curl_status: ${curl_status}"
	log "${id}_selected_headers:"
	local headers
	headers="$(selected_headers "$header_file")"
	if [[ -n "$headers" ]]; then
		while IFS= read -r header; do
			log "  ${header}"
		done <<<"$headers"
	else
		log "  none"
	fi
	log "${id}_redacted_body_snippet: $(redacted_body_snippet "$body_file")"
	if [[ -s "$error_file" ]]; then
		log "${id}_curl_error: $(redacted_body_snippet "$error_file")"
	fi
	log "${id}_route_conclusion: ${route_conclusion}"
	if [[ "$id" == "system_info" ]]; then
		log "safety_telemetry_fields: $(safety_fields_status "$body_file")"
	fi
}

log "phase14_live_telemetry"
log "surface: ${surface}"
log "manifest: ${manifest:-missing}"
log "out_dir: ${out_dir}"
log "safety_allow_command: bazel run //tools/parity:report -- safety-allow --manifest ${manifest:-missing} --surface ${surface} --allowed-command scripts/phase14-live-telemetry.sh --manifest ${manifest:-missing} --out-dir ${out_dir}"

if [[ -z "$manifest" || ! -f "$manifest" ]]; then
	log "safety_allow_status: pending - missing manifest"
	blocked_device_url "missing manifest"
	exit 0
fi

set +e
allow_output="$(run_safety_allow 2>&1)"
allow_status=$?
set -e

printf '%s\n' "$allow_output" >>"$log_file"
if [[ "$allow_status" -ne 0 ]] || ! grep -Fq "safety_allow_status: passed" <<<"$allow_output"; then
	blocked_device_url "allow validation failed"
	exit 0
fi

if [[ -z "$device_url" ]]; then
	blocked_device_url "missing DEVICE_URL"
	exit 0
fi

case "$device_url" in
http://* | https://*) ;;
*)
	blocked_device_url "invalid DEVICE_URL scheme"
	exit 0
	;;
esac

base_url="${device_url%/}"
readonly base_url

log "DEVICE_URL status: provided"
log "DEVICE_URL sanitized: $(sanitize_device_url "$device_url")"
log "network_scan: disabled - using explicit DEVICE_URL only"

probe_route "system_info" "/api/system/info"
probe_route "api_ws" "/api/ws"
probe_route "api_ws_live" "/api/ws/live"

log "api_telemetry_status: pending - exact live safety telemetry claim requires reviewed API body and redaction closure"
log "websocket_frame_status: pending - maintained WebSocket client unavailable"
log "websocket_frame_non_claim: route status is not frame-level cadence proof"
log "conclusion: pending - live API/WebSocket telemetry remains below verified unless explicit API values and maintained WebSocket frames are captured"
