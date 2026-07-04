#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --surface mining-smoke|bounded-soak --out-dir PATH --chip-detect-summary PATH --work-result-summary PATH --readiness-audit PATH --enablement-summary PATH [--device-url ORIGIN] [--pool-credentials PATH] [--duration-seconds N] [--curl-bin PATH] [--websocket-helper PATH]\n' "$(basename "$0")" >&2
}

manifest=""
surface=""
out_dir=""
chip_detect_summary=""
work_result_summary=""
readiness_audit=""
enablement_summary=""
device_url=""
device_url_arg_provided=0
pool_credentials=""
duration_seconds=""
curl_bin="${CURL_BIN:-curl}"
websocket_helper="${PHASE21_WEBSOCKET_HELPER:-scripts/phase17-websocket-capture.mjs}"
pool_credentials_helper="${PHASE21_POOL_CREDENTIALS_HELPER:-scripts/phase21-pool-credentials-json.mjs}"
node_bin="${NODE_BIN:-node}"

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
	--surface)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		surface="$2"
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
	--readiness-audit)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		readiness_audit="$2"
		shift 2
		;;
	--enablement-summary)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		enablement_summary="$2"
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
	--pool-credentials)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		pool_credentials="$2"
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

if [[ "$surface" == "bounded-soak" && -z "$duration_seconds" ]]; then
	duration_seconds=300
fi

if [[ -n "$duration_seconds" && ! "$duration_seconds" =~ ^[0-9]+$ ]]; then
	printf 'duration seconds must be numeric\n' >&2
	exit 2
fi

if [[ -n "$duration_seconds" && ( "$duration_seconds" -lt 60 || "$duration_seconds" -gt 600 ) ]]; then
	printf 'duration seconds must be between 60 and 600\n' >&2
	exit 2
fi

mkdir -p "$out_dir"
readonly log_file="${out_dir}/${surface}.log"
: >"$log_file"

redact_text() {
	LC_ALL=C tr -d '\000\r' |
		sed -E 's#https?://[^[:space:]"<>]+#[redacted-url]#g; s#wss?://[^[:space:]"<>]+#[redacted-url]#g; s/(Could not resolve host: )[[:alnum:]_.-]+/\1[redacted-host]/g; s/(Failed to connect to )[[:alnum:]_.-]+/\1[redacted-host]/g; s/"(ssid|wifiPass|wifiPassword|stratumUser|stratumPassword|stratumCert|poolUrl|poolURL|fallbackPoolUrl|hostname|ip|ipAddress|gateway|netmask|dns|apiToken|apiKey|token|password|user|worker|poolUser|poolPassword)"[[:space:]]*:[[:space:]]*"[^"]*"/"\1":"[redacted]"/g; s/"(stratumPort|fallbackStratumPort|poolPort)"[[:space:]]*:[[:space:]]*[0-9]+/"\1":[redacted]/g; s/([0-9]{1,3}\.){3}[0-9]{1,3}/[redacted-ip]/g; s/([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}/[redacted-mac]/g; s#(BITAXE_POOL_PASSWORD=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_USER=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_URL=)[^[:space:]]+#\1[redacted]#g; s#(BITAXE_POOL_PORT=)[^[:space:]]+#\1[redacted]#g; s#(DEVICE_URL=)[^[:space:]]+#\1[redacted]#g'
}

log() {
	printf '%s\n' "$*" | redact_text | tee -a "$log_file" >/dev/null
}

append_redacted() {
	redact_text | tee -a "$log_file" >/dev/null
}

maybe_load_pool_credentials() {
	if [[ -z "$pool_credentials" ]]; then
		return
	fi

	unset BITAXE_POOL_URL BITAXE_POOL_PORT BITAXE_POOL_USER BITAXE_POOL_PASSWORD

	if [[ ! -f "$pool_credentials" ]]; then
		log "pool_credentials_status=blocked - missing json file"
		return
	fi

	if [[ ! -f "$pool_credentials_helper" ]]; then
		log "pool_credentials_status=blocked - json helper missing"
		return
	fi

	local maybe_exports
	set +e
	maybe_exports="$("$node_bin" "$pool_credentials_helper" "$pool_credentials" 2>&1)"
	local helper_status=$?
	set -e

	if [[ "$helper_status" -ne 0 ]]; then
		printf '%s\n' "$maybe_exports" | append_redacted
		log "pool_credentials_status=blocked - invalid json"
		return
	fi

	eval "$maybe_exports"
	export BITAXE_POOL_URL BITAXE_POOL_PORT BITAXE_POOL_USER BITAXE_POOL_PASSWORD
	log "pool_credentials_status=loaded-redacted source=json"
}

command_arg_list() {
	printf '%s\n' \
		"scripts/phase21-live-mining-evidence.sh" \
		"--manifest" "$manifest" \
		"--surface" "$surface" \
		"--out-dir" "$out_dir" \
		"--chip-detect-summary" "$chip_detect_summary" \
		"--work-result-summary" "$work_result_summary" \
		"--readiness-audit" "$readiness_audit" \
		"--enablement-summary" "$enablement_summary"

	if [[ "$device_url_arg_provided" -eq 1 ]]; then
		printf '%s\n' "--device-url" "$device_url"
	fi

	if [[ -n "$pool_credentials" ]]; then
		printf '%s\n' "--pool-credentials" "$pool_credentials"
	fi

	if [[ "$surface" == "bounded-soak" ]]; then
		printf '%s\n' "--duration-seconds" "$duration_seconds"
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
	local expected_allowed_command="$1"

	if [[ -n "${PHASE21_MINING_ALLOW_BIN:-}" ]]; then
		"$PHASE21_MINING_ALLOW_BIN" \
			--manifest "$manifest" \
			--surface "$surface" \
			--allowed-command "$expected_allowed_command"
		return
	fi

	bazel run //tools/parity:report -- mining-allow \
		--manifest "$manifest" \
		--surface "$surface" \
		--allowed-command "$expected_allowed_command"
}

summary_contains_safe_marker() {
	local path="$1"

	grep -Eq 'safe_state: mining=disabled|work_submission=disabled|fail_closed=true' "$path"
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

	if [[ -z "$readiness_audit" || ! -f "$readiness_audit" ]]; then
		printf 'readiness audit missing'
		return
	fi

	if [[ -z "$enablement_summary" || ! -f "$enablement_summary" ]]; then
		printf 'enablement summary missing'
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

	if ! summary_contains_safe_marker "$chip_detect_summary" ||
		! summary_contains_safe_marker "$work_result_summary"; then
		printf 'safe marker missing'
		return
	fi

	if ! summary_has_non_blocked_redaction "$chip_detect_summary" ||
		! summary_has_non_blocked_redaction "$work_result_summary"; then
		printf 'redaction status'
		return
	fi

	if ! grep -Fxq 'firmware_live_mining_status: blocked_by_default' "$readiness_audit"; then
		printf 'readiness audit blocked-by-default marker missing'
		return
	fi

	if ! grep -Fxq 'controlled_enablement_required: true' "$readiness_audit"; then
		printf 'readiness audit enablement marker missing'
		return
	fi

	if grep -Fq 'controlled_live_mining_package_status: blocked' "$enablement_summary"; then
		printf 'enablement package blocked'
		return
	fi

	if ! grep -Fxq 'controlled_live_mining_package_status: ready' "$enablement_summary"; then
		printf 'enablement package not ready'
		return
	fi

	if ! grep -Fxq 'controlled_runtime_harness_status: ready' "$enablement_summary"; then
		printf 'enablement runtime harness not ready'
		return
	fi
}

log_header() {
	log "phase21_live_mining_evidence"
	log "network_scan: disabled - DEVICE_URL must be explicit"
	log "surface: ${surface}"
	log "manifest: ${manifest:-missing}"
	log "out_dir: ${out_dir}"
	if [[ "$surface" == "bounded-soak" ]]; then
		log "duration_seconds=${duration_seconds}"
	fi
}

log_safe_state_markers() {
	log "post_action_safe_state_marker=safe_state: mining=disabled"
	log "post_action_safe_state_marker=hardware_control=disabled"
	log "post_action_safe_state_marker=work_submission=disabled"
}

record_pending() {
	local reason="$1"

	log "enablement_status=pending"
	log "controlled_mining_status: pending - ${reason}"
	log "hardware_command_status=not-run"
	log "pool_lifecycle_status=pending - ${reason}"
	log "subscribe_status=pending - ${reason}"
	log "authorize_status=pending - ${reason}"
	log "notify_job_status=pending - ${reason}"
	log "bm1366_work_dispatch_status=pending - ${reason}"
	log "result_receive_status=pending - ${reason}"
	log "share_outcome=pending - ${reason}"
	log "hashrate_inputs_status=pending - ${reason}"
	log "api_telemetry_status=pending - ${reason}"
	log "websocket_frame_status=pending - ${reason}"
	log "watchdog_status=observed-or-pending"
	log "safe_stop_status=confirmed-or-pending"
	log_safe_state_markers
	log "conclusion: pending - ${reason}"
}

record_abort_contract() {
	log "abort_condition=detector_mismatch"
	log "abort_condition=board_info_failure"
	log "abort_condition=missing_trusted_wrapper_markers"
	log "abort_condition=redaction_uncertainty"
	log "abort_condition=unsafe_temperature_or_power"
	log "abort_condition=watchdog_unresponsive"
}

live_prerequisites_missing() {
	[[ -z "$device_url" ||
		-z "${BITAXE_POOL_URL:-}" ||
		-z "${BITAXE_POOL_USER:-}" ||
		-z "${BITAXE_POOL_PASSWORD:-}" ]]
}

record_missing_live_prerequisites() {
	log "enablement_status=ready"
	log "controlled_mining_status: blocked - missing live prerequisites"
	log "live_mining_smoke_status: blocked"
	log "blocker: missing_live_prerequisites"
	log "controlled_no_share_condition=not-applicable"
	log "hardware_command_status=not-run"
	log "pool_lifecycle_status=blocked - missing live prerequisites"
	log "subscribe_status=blocked - missing live prerequisites"
	log "authorize_status=blocked - missing live prerequisites"
	log "notify_job_status=blocked - missing live prerequisites"
	log "bm1366_work_dispatch_status=blocked - missing live prerequisites"
	log "result_receive_status=blocked - missing live prerequisites"
	log "share_outcome=not-observed - missing live prerequisites"
	log "hashrate_inputs_status=blocked - missing live prerequisites"
	log "api_telemetry_status=blocked - missing live prerequisites"
	log "websocket_frame_status=blocked - missing live prerequisites"
	log "watchdog_status=observed-or-pending"
	log "safe_stop_status=confirmed-or-pending"
	log_safe_state_markers
	log "conclusion: blocked - missing_live_prerequisites"
}

probe_api_status() {
	local url="${device_url%/}/api/system/info"
	local body_file="${out_dir}/api-system-info.redacted.json"
	local error_file="${out_dir}/api-system-info.error.txt"
	local body_tmp="${body_file}.tmp"
	local error_tmp="${error_file}.tmp"

	: >"$body_file"
	: >"$error_file"
	: >"$body_tmp"
	: >"$error_tmp"

	set +e
	local status
	status="$("$curl_bin" --silent --show-error --max-time 10 --output "$body_tmp" --write-out "%{http_code}" "$url" 2>"$error_tmp")"
	local curl_status=$?
	set -e

	redact_text <"$body_tmp" >"$body_file"
	redact_text <"$error_tmp" >"$error_file"
	rm -f "$body_tmp" "$error_tmp"

	if [[ "$curl_status" -ne 0 ]]; then
		log "api_telemetry_status=pending - curl failed"
		log "api_telemetry_curl_status=${curl_status}"
		if [[ -n "$status" ]]; then
			log "api_telemetry_http_status=${status}"
		fi
		return
	fi

	log "api_telemetry_status=http_status_${status}_curl_${curl_status}"
}

probe_websocket_status() {
	local ws_out="${out_dir}/websocket-live.redacted.log"

	if [[ ! -x "$node_bin" && "$(command -v "$node_bin" 2>/dev/null || true)" == "" ]]; then
		log "websocket_frame_status=pending - Node unavailable"
		return
	fi

	if [[ ! -f "$websocket_helper" ]]; then
		log "websocket_frame_status=pending - websocket helper missing"
		return
	fi

	set +e
	"$node_bin" "$websocket_helper" \
		--device-url "$device_url" \
		--path /api/ws/live \
		--out "$ws_out" \
		--duration-ms 10000 \
		--max-frames 5 >/dev/null 2>&1
	local ws_status=$?
	set -e

	if [[ "$ws_status" -ne 0 ]]; then
		log "websocket_frame_status=pending - helper failed"
		return
	fi

	if [[ -s "$ws_out" ]]; then
		append_redacted <"$ws_out"
		return
	fi

	log "websocket_frame_status=pending - helper produced no frames"
}

record_live_probe_attempt() {
	case "$device_url" in
	http://* | https://*) ;;
	*)
		record_pending "invalid DEVICE_URL scheme"
		return
		;;
	esac

	log "enablement_status=ready"
	log "controlled_mining_status: live-prerequisites-present"
	log "live_mining_smoke_status: pending - live probe only"
	log "pool_lifecycle_status=pending - downstream mining harness required"
	log "subscribe_status=pending - downstream mining harness required"
	log "authorize_status=pending - downstream mining harness required"
	log "notify_job_status=pending - downstream mining harness required"
	log "bm1366_work_dispatch_status=pending - downstream mining harness required"
	log "result_receive_status=pending - downstream mining harness required"
	log "share_outcome=pending - no reviewed share outcome"
	log "hashrate_inputs_status=pending - no reviewed hashrate inputs"
	probe_api_status
	probe_websocket_status
	log "watchdog_status=observed-or-pending"
	log "safe_stop_status=confirmed-or-pending"
	log_safe_state_markers
	log "conclusion: pending - live mining evidence requires reviewed share/no-share and safe-stop artifacts"
}

log_header
maybe_load_pool_credentials

if [[ -z "$manifest" || ! -f "$manifest" ]]; then
	log "mining_allow_status=not-run"
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
	record_missing_live_prerequisites
	exit 0
fi

record_live_probe_attempt
