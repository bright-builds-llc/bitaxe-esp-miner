#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH --surface mining-smoke|bounded-soak --chip-detect-summary PATH --work-result-summary PATH [--device-url URL] [--duration-seconds N] [--curl-bin PATH] [--websocket-helper PATH]\n' "$(basename "$0")" >&2
}

manifest=""
out_dir=""
surface=""
chip_detect_summary=""
work_result_summary=""
device_url="${DEVICE_URL:-}"
device_url_arg_provided=0
duration_seconds=""
curl_bin="${CURL_BIN:-curl}"
websocket_helper="${PHASE15_WEBSOCKET_HELPER:-scripts/phase15-websocket-capture.mjs}"

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
	--surface)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		surface="$2"
		shift 2
		;;
	--chip-detect-summary)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		chip_detect_summary="$2"
		shift 2
		;;
	--work-result-summary)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		work_result_summary="$2"
		shift 2
		;;
	--device-url)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		device_url="$2"
		device_url_arg_provided=1
		shift 2
		;;
	--duration-seconds)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		duration_seconds="$2"
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
	--websocket-helper)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		websocket_helper="$2"
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

if [[ -z "$out_dir" || -z "$surface" ]]; then
	usage
	exit 2
fi

case "$surface" in
mining-smoke | bounded-soak) ;;
*)
	printf 'unsupported surface: %s\n' "$surface" >&2
	exit 2
	;;
esac

if [[ -n "$duration_seconds" && ! "$duration_seconds" =~ ^[0-9]+$ ]]; then
	printf 'duration seconds must be numeric\n' >&2
	exit 2
fi

if [[ "$surface" == "bounded-soak" && -z "$duration_seconds" ]]; then
	duration_seconds=120
fi

mkdir -p "$out_dir"
readonly log_file="${out_dir}/${surface}.log"
: >"$log_file"

redact_text() {
	LC_ALL=C tr -d '\000\r' |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s#wss?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|apiToken|apiKey|token|password|user|worker)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g; s#(BITAXE_POOL_PASSWORD=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_USER=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_URL=)[^[:space:]]+#\1[redacted]#g; s#(DEVICE_URL=)[^[:space:]]+#\1[redacted]#g'
}

log() {
	printf '%s\n' "$*" | redact_text | tee -a "$log_file" >/dev/null
}

append_redacted() {
	redact_text | tee -a "$log_file" >/dev/null
}

command_arg_list() {
	printf '%s\n' \
		"scripts/phase15-controlled-mining.sh" \
		"--manifest" "$manifest" \
		"--surface" "$surface"

	if [[ "$surface" == "bounded-soak" ]]; then
		printf '%s\n' "--duration-seconds" "$duration_seconds"
	fi

	printf '%s\n' \
		"--out-dir" "$out_dir" \
		"--chip-detect-summary" "$chip_detect_summary" \
		"--work-result-summary" "$work_result_summary"

	if [[ "$device_url_arg_provided" -eq 1 ]]; then
		printf '%s\n' "--device-url" "$device_url"
	fi
}

allowed_command_string() {
	local command=""
	local arg

	while IFS= read -r arg; do
		if [[ -z "$command" ]]; then
			command="$arg"
		else
			command="${command} ${arg}"
		fi
	done < <(command_arg_list)

	printf '%s' "$command"
}

run_mining_allow() {
	local allowed_command="$1"

	if [[ -n "${PHASE15_MINING_ALLOW_BIN:-}" ]]; then
		"$PHASE15_MINING_ALLOW_BIN" \
			--manifest "$manifest" \
			--surface "$surface" \
			--allowed-command "$allowed_command"
		return
	fi

	bazel run //tools/parity:report -- mining-allow \
		--manifest "$manifest" \
		--surface "$surface" \
		--allowed-command "$allowed_command"
}

summary_contains_any_safe_marker() {
	local path="$1"

	grep -Eq 'safe_state: mining=disabled|work_submission=disabled|fail_closed=true' "$path"
}

summary_has_restore_or_safe_state_marker() {
	local path="$1"

	grep -Eq 'restore_status:|safe_stop_status:|safe_state: mining=disabled|work_submission=disabled|fail_closed=true' "$path"
}

summary_has_non_blocked_redaction() {
	local path="$1"

	if grep -Eiq 'redaction_status:[[:space:]]*(blocked|pending|failed)' "$path"; then
		return 1
	fi

	grep -Eiq 'redaction_status:[[:space:]]*passed' "$path"
}

maybe_prerequisite_blocker() {
	if [[ -z "$chip_detect_summary" || ! -f "$chip_detect_summary" ]]; then
		printf 'chip-detect summary missing'
		return
	fi

	if [[ -z "$work_result_summary" || ! -f "$work_result_summary" ]]; then
		printf 'work-result summary missing'
		return
	fi

	if ! grep -Fxq 'conclusion: passed for package-backed chip-detect smoke' "$chip_detect_summary"; then
		printf 'chip-detect conclusion'
		return
	fi

	if ! grep -Fxq 'conclusion: passed for diagnostic work/result smoke' "$work_result_summary" &&
		! grep -Fxq 'conclusion: passed for diagnostic work dispatch with bounded no-result' "$work_result_summary"; then
		printf 'work-result conclusion'
		return
	fi

	if ! summary_contains_any_safe_marker "$chip_detect_summary"; then
		printf 'chip-detect safe marker missing'
		return
	fi

	if ! summary_contains_any_safe_marker "$work_result_summary"; then
		printf 'work-result safe marker missing'
		return
	fi

	if ! summary_has_restore_or_safe_state_marker "$chip_detect_summary" ||
		! summary_has_restore_or_safe_state_marker "$work_result_summary"; then
		printf 'restore or safe-state marker missing'
		return
	fi

	if ! summary_has_non_blocked_redaction "$chip_detect_summary"; then
		printf 'chip-detect redaction status'
		return
	fi

	if ! summary_has_non_blocked_redaction "$work_result_summary"; then
		printf 'work-result redaction status'
		return
	fi
}

log_standard_header() {
	log "phase15_controlled_mining"
	log "surface: ${surface}"
	log "manifest: ${manifest:-missing}"
	log "out_dir: ${out_dir}"
	if [[ "$surface" == "bounded-soak" ]]; then
		log "duration_seconds=${duration_seconds}"
	fi
	log "network_scan: disabled - DEVICE_URL must be explicit"
}

record_pending() {
	local reason="$1"

	log "controlled_mining_status: pending - ${reason}"
	log "hardware_command_status=not-run"
	log "live_smoke_status=not-run"
	if [[ "$surface" == "bounded-soak" ]]; then
		log "bounded_soak_status: pending - ${reason}"
	fi
	log "pool_category=not-run"
	log "share_outcome=hardware evidence pending - ${reason}"
	log "api_telemetry_status=pending - ${reason}"
	log "websocket_frame_status=pending - ${reason}"
	log "safe_stop_status=confirmed-or-pending"
	log "conclusion: hardware evidence pending - ${reason}"
}

live_prerequisites_missing() {
	[[ -z "${BITAXE_POOL_URL:-}" ||
		-z "${BITAXE_POOL_USER:-}" ||
		-z "${BITAXE_POOL_PASSWORD:-}" ||
		-z "$device_url" ]]
}

record_controlled_no_share() {
	log "controlled_mining_status: controlled-no-share"
	log "pool_category=controlled-no-share"
	log "controlled_no_share_condition=missing_live_prerequisites"
	log "share_outcome=controlled no-share condition"
	log "hashrate_inputs_status=pending - no live pool work submitted"
	if [[ -z "$device_url" ]]; then
		log "api_telemetry_status=pending - missing DEVICE_URL"
	else
		log "api_telemetry_status=pending - live pool prerequisites missing"
	fi
	log "websocket_frame_status=pending - missing DEVICE_URL or helper blocked"
	log "watchdog_status=pending - live prerequisites missing"
	log "safe_stop_status=confirmed-or-pending"
	if [[ "$surface" == "bounded-soak" ]]; then
		log "bounded_soak_status: controlled-no-share - missing live prerequisites"
	fi
	log "conclusion: controlled no-share condition - missing live prerequisites"
}

record_abort_contract() {
	log "abort_condition=unsafe_temperature_or_power"
	log "abort_condition=watchdog_unresponsive"
	log "abort_condition=serial_silence"
	log "abort_condition=redaction_uncertainty"
	log "abort_condition=missing_safe_stop"
}

probe_api_status() {
	local url="${device_url%/}/api/system/info"
	local body_file="${out_dir}/api-system-info.redacted.json"
	local error_file="${out_dir}/api-system-info.error.txt"

	: >"$body_file"
	: >"$error_file"

	set +e
	local status
	status="$("$curl_bin" --silent --show-error --max-time 10 --output "$body_file.tmp" --write-out "%{http_code}" "$url" 2>"$error_file.tmp")"
	local curl_status=$?
	set -e

	redact_text <"$body_file.tmp" >"$body_file"
	redact_text <"$error_file.tmp" >"$error_file"
	rm -f "$body_file.tmp" "$error_file.tmp"

	log "api_telemetry_status=http_status_${status}_curl_${curl_status}"
	if [[ -s "$error_file" ]]; then
		log "api_telemetry_error=$(tr '\n\t' '  ' <"$error_file" | head -c 240)"
	fi
}

probe_websocket_status() {
	local ws_out="${out_dir}/websocket-live.redacted.log"

	if [[ ! -x "$websocket_helper" && ! -f "$websocket_helper" ]]; then
		log "websocket_frame_status=pending - missing DEVICE_URL or helper blocked"
		return
	fi

	set +e
	node "$websocket_helper" \
		--device-url "$device_url" \
		--out "$ws_out" \
		--duration-ms 5000 \
		--max-frames 3 >/dev/null 2>&1
	local ws_status=$?
	set -e

	if [[ "$ws_status" -ne 0 ]]; then
		log "websocket_frame_status=pending - missing DEVICE_URL or helper blocked"
		return
	fi

	if [[ -s "$ws_out" ]]; then
		append_redacted <"$ws_out"
	else
		log "websocket_frame_status=pending - helper produced no frames"
	fi
}

record_live_attempt() {
	case "$device_url" in
	http://* | https://*) ;;
	*)
		log "controlled_mining_status: pending - invalid DEVICE_URL scheme"
		record_controlled_no_share
		return
		;;
	esac

	log "controlled_mining_status: live-prerequisites-present"
	log "pool_category=live-pool-smoke"
	log "pool_endpoint_status=provided-redacted"
	log "pool_user_status=provided-redacted"
	log "pool_password_status=provided-redacted"
	log "device_url_status=provided-redacted"
	log "share_outcome=pending - live smoke not promoted without reviewed share result"
	probe_api_status
	probe_websocket_status
	log "watchdog_status=pending - bounded live observation required before promotion"
	log "safe_stop_status=confirmed-or-pending"
	log "conclusion: pending - live mining evidence requires reviewed share/no-share and safe-stop artifacts"
}

log_standard_header

if [[ -z "$manifest" || ! -f "$manifest" ]]; then
	record_pending "missing manifest"
	exit 0
fi

allowed_command="$(allowed_command_string)"
readonly allowed_command
log "mining_allow_command: bazel run //tools/parity:report -- mining-allow --manifest ${manifest} --surface ${surface} --allowed-command ${allowed_command}"

set +e
allow_output="$(run_mining_allow "$allowed_command" 2>&1)"
allow_status=$?
set -e

printf '%s\n' "$allow_output" | append_redacted
if [[ "$allow_status" -ne 0 ]] || ! grep -Fq "mining_allow_status: passed" <<<"$allow_output"; then
	record_pending "allow validation failed"
	exit 0
fi

maybe_blocker="$(maybe_prerequisite_blocker)"
readonly maybe_blocker
if [[ -n "$maybe_blocker" ]]; then
	record_pending "prerequisite not passed: ${maybe_blocker}"
	exit 0
fi

if [[ "$surface" == "bounded-soak" ]]; then
	log "duration_seconds=${duration_seconds}"
	record_abort_contract
fi

if live_prerequisites_missing; then
	record_controlled_no_share
	exit 0
fi

record_live_attempt
