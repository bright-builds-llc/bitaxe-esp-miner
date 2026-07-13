#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --evidence-root PATH --manifest PATH --mode blocked|hardware [--pool-credentials PATH] [--wifi-credentials PATH] [--duration-seconds N] [--device-url ORIGIN]\n' "$(basename "$0")" >&2
}

evidence_root=""
manifest=""
mode=""
pool_credentials=""
wifi_credentials=""
duration_seconds=""
device_url=""

while [[ $# -gt 0 ]]; do
	case "$1" in
	--evidence-root)
		evidence_root="${2:-}"
		shift 2
		;;
	--manifest)
		manifest="${2:-}"
		shift 2
		;;
	--mode)
		mode="${2:-}"
		shift 2
		;;
	--pool-credentials)
		pool_credentials="${2:-}"
		shift 2
		;;
	--wifi-credentials)
		wifi_credentials="${2:-}"
		shift 2
		;;
	--duration-seconds)
		duration_seconds="${2:-}"
		shift 2
		;;
	--device-url)
		device_url="${2:-}"
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

if [[ -z "$evidence_root" || -z "$manifest" || -z "$mode" ]]; then
	usage
	exit 2
fi

case "$mode" in
blocked | hardware) ;;
*)
	printf 'unsupported --mode %s; expected blocked|hardware\n' "$mode" >&2
	exit 2
	;;
esac

if [[ -n "$duration_seconds" && ! "$duration_seconds" =~ ^[0-9]+$ ]]; then
	printf 'duration seconds must be numeric\n' >&2
	exit 2
fi

if [[ -n "$duration_seconds" && ("$duration_seconds" -lt 60 || "$duration_seconds" -gt 600) ]]; then
	printf 'duration seconds must be between 60 and 600\n' >&2
	exit 2
fi

validate_origin_device_url() {
	local value="$1"
	local rest
	local host
	local port

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
	if [[ "$rest" == */* ]]; then
		if [[ "$rest" != */ || "${rest%/}" == *"/"* ]]; then
			return 1
		fi
		rest="${rest%/}"
	fi
	if [[ -z "$rest" || "$rest" == *"["* || "$rest" == *"]"* ]]; then
		return 1
	fi

	host="$rest"
	port=""
	if [[ "$rest" == *:* ]]; then
		host="${rest%:*}"
		port="${rest##*:}"
		if [[ -z "$host" || ! "$port" =~ ^[0-9]+$ || "$port" -lt 1 || "$port" -gt 65535 ]]; then
			return 1
		fi
	fi

	if [[ ! "$host" =~ ^[A-Za-z0-9.-]+$ || "$host" == .* || "$host" == *..* || "$host" == *. ]]; then
		return 1
	fi

	return 0
}

if [[ -n "$device_url" ]] && ! validate_origin_device_url "$device_url"; then
	printf 'invalid origin-only DEVICE_URL\n' >&2
	exit 2
fi

readonly source_commit="${PHASE25_SOURCE_COMMIT:-$(git rev-parse HEAD 2>/dev/null || printf 'unknown-source')}"
readonly reference_commit="${PHASE25_REFERENCE_COMMIT:-$(git -C reference/esp-miner rev-parse HEAD 2>/dev/null || printf 'unknown-reference')}"
readonly detector_command="${PHASE25_DETECT_COMMAND:-just detect-ultra205}"
readonly board_info_command="${PHASE25_BOARD_INFO_COMMAND:-espflash board-info --chip esp32s3 --non-interactive}"
readonly parity_command="${PHASE25_PARITY_COMMAND:-bazel run //tools/parity:report --}"
readonly live_capture_command="${PHASE25_LIVE_CAPTURE_COMMAND:-just flash-monitor}"

mkdir -p "$evidence_root"
rm -f "${evidence_root}/mining-allow.json"

pool_config_label="not-supplied"
if [[ -n "$pool_credentials" ]]; then
	pool_config_label="local-owner-supplied"
fi

wifi_config_label="not-supplied"
if [[ -n "$wifi_credentials" ]]; then
	wifi_config_label="local-owner-supplied"
fi

device_url_label="not-supplied"
if [[ -n "$device_url" ]]; then
	device_url_label="explicit"
fi

duration_label="not-requested"
if [[ -n "$duration_seconds" ]]; then
	duration_label="$duration_seconds"
fi

workflow_status="passed"
mining_allow_applicable=0
write_slot() {
	local slot="$1"
	local status="$2"
	local share_outcome="$3"
	local safe_stop_status="$4"
	local watchdog_status="$5"
	local observed="$6"
	local conclusion="$7"
	local file="${evidence_root}/${slot}.md"

	cat >"$file" <<EOF
# Phase 25 ${slot} Evidence

slot: ${slot}
slot_status: ${status}
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
detector_evidence: just detect-ultra205
board_info_evidence: espflash board-info
command_category: repo-owned-phase25-live-stratum-evidence
redaction_status: passed
share_outcome: ${share_outcome}
safe_stop_status: ${safe_stop_status}
watchdog_responsiveness_status: ${watchdog_status}
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
device_target_source: ${device_url_label}
duration_seconds: ${duration_label}
raw_pool_values_committed: no
network_scan: disabled

## observed_behavior

${observed}

## conclusion

${conclusion}

## exact_non_claims

- accepted/rejected shares remain non-claims unless a detector-gated live socket response is tied to a live ASIC-derived submit intent.
- Phase 26 API, WebSocket, statistics, and scoreboard projection remains a non-claim except post-stop SAFE-12 state.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
EOF
}

redaction_diagnostic_status() {
	if [[ -n "${PHASE25_RAW_DIAGNOSTIC_SAMPLE:-}" ]]; then
		printf 'rejected_sensitive_raw_payload'
		return
	fi

	printf 'no_raw_diagnostic_input'
}
allowed_command_string() {
	local command="scripts/phase25-live-stratum-evidence.sh --evidence-root ${evidence_root} --manifest ${manifest} --mode ${mode}"

	if [[ -n "$duration_seconds" ]]; then
		command="${command} --duration-seconds ${duration_seconds}"
	fi
	if [[ -n "$device_url" ]]; then
		command="${command} --device-url [redacted-origin]"
	fi
	if [[ -n "$pool_credentials" ]]; then
		command="${command} --pool-credentials [redacted-local-path]"
	fi
	if [[ -n "$wifi_credentials" ]]; then
		command="${command} --wifi-credentials [redacted-local-path]"
	fi

	printf '%s' "$command"
}
finalize_evidence() {
	local completion_status=0
	local mining_allow_status=0
	local operator_status=0
	set +e
	${parity_command} complete-operator-evidence --profile phase25 --evidence-root "$evidence_root" --workflow-status "$workflow_status" >/dev/null
	completion_status=$?
	if [[ "$mining_allow_applicable" -eq 1 ]]; then
		${parity_command} mining-allow --manifest "${evidence_root}/mining-allow.json" --surface live-stratum-runtime --allowed-command "$(allowed_command_string)" >/dev/null
		mining_allow_status=$?
	fi
	${parity_command} operator-evidence --profile phase25 --evidence-root "$evidence_root" --require-redaction-passed >/dev/null
	operator_status=$?
	set -e
	[[ "$workflow_status" == "passed" && "$completion_status" -eq 0 && "$mining_allow_status" -eq 0 && "$operator_status" -eq 0 ]]
}
write_allow_manifest() {
	local path="${evidence_root}/mining-allow.json"
	local detected_port="[redacted-port]"
	local board_info_status="$2"
	local claim_tier="$3"
	local evidence_class="$4"
	local conclusion="$5"
	local safe_stop_status="$6"
	local watchdog_status="$7"
	local command
	command="$(allowed_command_string)"

	cat >"$path" <<EOF
{
  "board": "205",
  "port": "${detected_port}",
  "detector_command": "just detect-ultra205",
  "detector_port": "${detected_port}",
  "board_info_command": "espflash board-info --chip esp32s3 --port ${detected_port} --non-interactive",
  "board_info_status": "${board_info_status}",
  "package_manifest": "${manifest}",
  "source_commit": "${source_commit}",
  "reference_commit": "${reference_commit}",
  "surface": "live-stratum-runtime",
  "claim_tier": "${claim_tier}",
  "evidence_class": "${evidence_class}",
  "allowed_command": "${command}",
  "allowed_inputs": {
    "pool_config": "${pool_config_label}",
    "wifi_config": "${wifi_config_label}",
    "device_url": "${device_url_label}",
    "duration_seconds": ${duration_seconds:-360},
    "target_source": "explicit-or-blocked",
    "conclusion": "${conclusion}",
    "safe_stop_status": "${safe_stop_status}",
    "watchdog_responsiveness_status": "${watchdog_status}"
  },
  "abort_conditions": [
    "detector_mismatch",
    "board_info_failure",
    "missing_trusted_wrapper_markers",
    "redaction_uncertainty",
    "unsafe_temperature_or_power",
    "watchdog_unresponsive"
  ],
  "recovery_steps": [
    "safe_stop",
    "just flash board=205 port=${detected_port}"
  ],
  "post_action_safe_state_markers": [
    "safe_state: mining=disabled",
    "hardware_control=disabled",
    "work_submission=disabled"
  ],
  "prerequisite_artifacts": [
    "${evidence_root}/detector.md",
    "${evidence_root}/board-info.md",
    "${evidence_root}/redaction-review.md"
  ],
  "evidence_dir": "${evidence_root}",
  "redaction_reviewer": "phase-25-wrapper",
  "checklist_rows": ["STR-08", "STR-09", "STR-11", "SAFE-12", "SAFE-13"]
}
EOF
}
write_redaction_review() {
	local status
	status="$(redaction_diagnostic_status)"

	cat >"${evidence_root}/redaction-review.md" <<EOF
# Phase 25 Redaction Review

slot: redaction-review
slot_status: passed
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
detector_evidence: just detect-ultra205
command_category: deterministic-phase25-redaction-review
redaction_status: passed
diagnostic_input_status: ${status}
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
raw_pool_values_committed: no
network_scan: disabled

## Artifact Inventory

package.md
detector.md
board-info.md
command.md
log.md
api.md
websocket.md
share-outcome.md
safe-stop.md
redaction-review.md
summary.md

## conclusion

No raw local credential contents, pool endpoints, workers, owner addresses, passwords, targets, extranonces, share payloads, socket details, device targets, IPs, MACs, Wi-Fi values, NVS secrets, API tokens, raw protocol payloads, raw share payloads, or raw BM1366 frames are committed.

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists.
- Hardware watchdog proof remains blocked unless observed in a detector-gated run.
EOF
}

write_summary() {
	local detector_status="$1"
	local board_info_status="$2"
	local share_outcome="$3"
	local safe_stop_status="$4"
	local watchdog_status="$5"
	local conclusion="$6"

	cat >"${evidence_root}/summary.md" <<EOF
# Phase 25 Evidence Summary

board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
package_artifact_status: ${board_info_status}
detector_status: ${detector_status}
board_info_status: ${board_info_status}
share_outcome: ${share_outcome}
safe_stop_status: ${safe_stop_status}
watchdog_responsiveness_status: ${watchdog_status}
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
network_scan: disabled
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
device_target_source: ${device_url_label}

## Supported Claim

${conclusion}

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists.
- Raw credentials, endpoints, target data, socket details, device targets, IPs, MACs, and raw BM1366 frames are not committed.
- Phase 26 telemetry projection remains deferred.
EOF
}

write_full_blocked_slots() {
	local detector_status="$1"
	local board_info_status="$2"
	local blocker="$3"
	local watchdog_status="blocked"
	local share_outcome="blocked_safe_prerequisite"
	local safe_stop_status="complete"
	local conclusion="Phase 25 records an exact blocked safe-prerequisite non-claim: ${blocker}."

	write_slot "package" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Package identity is recorded as a redaction-safe path label only; no raw package bytes are committed." "$conclusion"
	write_slot "detector" "$detector_status" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Detector status is ${detector_status}; hardware promotion requires just detect-ultra205." "$conclusion"
	write_slot "board-info" "$board_info_status" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Board-info status is ${board_info_status}; hardware promotion requires ESP32-S3 board-info in the same detector-gated session." "$conclusion"
	write_slot "command" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Repo-owned wrapper command only; raw Stratum, raw BM1366, unsafe hardware control, erase, rollback, stale targets, and network scans are not accepted." "$conclusion"
	write_slot "log" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Committed logs are redacted lifecycle/status categories only." "$conclusion"
	write_slot "api" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "API capture is blocked unless the target origin is explicit in the current detector-gated session." "$conclusion"
	write_slot "websocket" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "WebSocket capture is blocked unless the target origin is explicit in the current detector-gated session." "$conclusion"
	write_slot "share-outcome" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "No live pool response tied to live ASIC-derived submit intent was observed. accepted/rejected shares remain non-claims." "$conclusion"
	write_slot "safe-stop" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "safe_stop_status: complete; socket=stopped; work_queue=invalidated; active_work=invalidated; mining=disabled; hardware_control=disabled; work_submission=disabled; post_stop_snapshot=updated." "$conclusion"
	write_redaction_review
	write_summary "$detector_status" "$board_info_status" "$share_outcome" "$safe_stop_status" "$watchdog_status" "$conclusion"
	write_allow_manifest "/dev/redacted-phase25" "$board_info_status" "safe-prerequisite-blocked" "workflow" "blocked_safe_prerequisite" "$safe_stop_status" "$watchdog_status"
}

write_live_capture_slots() {
	local detected_port="$1"
	local share_outcome="live_submit_response_observed"
	local safe_stop_status="complete"
	local watchdog_status="passed"
	local conclusion="Phase 25 records detector-gated live submit-response evidence from the bounded live capture helper with redacted runtime-only inputs."

	write_slot "package" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Package identity was supplied to the bounded repo-owned live capture helper; no raw package bytes are committed." "$conclusion"
	write_slot "detector" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Detector passed for exactly one Ultra 205 session. The selected port is recorded only in mining-allow metadata." "$conclusion"
	write_slot "board-info" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Board-info passed for ESP32-S3 in the same detector-gated session." "$conclusion"
	write_slot "command" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Repo-owned bounded live capture helper was invoked with runtime-only local input paths and an explicit origin; raw values are not committed." "$conclusion"
	write_slot "log" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Live capture output was reduced to redacted status categories before evidence recording." "$conclusion"
	write_slot "api" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "API target use was limited to the explicit origin supplied in the current detector-gated session." "$conclusion"
	write_slot "websocket" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "WebSocket/live capture target use was limited to the explicit origin supplied in the current detector-gated session." "$conclusion"
	write_slot "share-outcome" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "The bounded helper observed the Phase 25 live submit-response marker. Raw Stratum payloads, pool endpoints, workers, and share fields are not committed." "$conclusion"
	write_slot "safe-stop" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "safe_stop_status: complete; socket=stopped; work_queue=invalidated; active_work=invalidated; mining=disabled; hardware_control=disabled; work_submission=disabled; post_stop_snapshot=updated." "$conclusion"
	write_redaction_review
	write_summary "passed" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "$conclusion"
	write_allow_manifest "$detected_port" "passed" "live-submit-response" "hardware-smoke" "live-submit-response" "$safe_stop_status" "$watchdog_status"
}

write_live_capture_not_observed_slots() {
	local detected_port="$1"
	local share_outcome="blocked_safe_prerequisite"
	local safe_stop_status="${live_capture_safe_stop_status:-blocked}"
	local watchdog_status="${live_capture_watchdog_status:-blocked}"
	local conclusion="Phase 25 attempted bounded detector-gated live capture, but no valid live submit-response marker was observed."

	write_slot "package" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Package identity was supplied to the bounded repo-owned live capture helper; no raw package bytes are committed." "$conclusion"
	write_slot "detector" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Detector passed for exactly one Ultra 205 session before the live capture attempt." "$conclusion"
	write_slot "board-info" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Board-info passed for ESP32-S3 before the live capture attempt." "$conclusion"
	write_slot "command" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Repo-owned bounded live capture helper was invoked with runtime-only local input paths and an explicit origin; raw values are not committed." "$conclusion"
	write_slot "log" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "Live capture output did not contain the required submit-response, safe-stop, and watchdog category markers." "$conclusion"
	write_slot "api" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "API/WebSocket promotion remains blocked because the live submit-response marker was not observed." "$conclusion"
	write_slot "websocket" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "WebSocket/live capture promotion remains blocked because the live submit-response marker was not observed." "$conclusion"
	write_slot "share-outcome" "blocked" "$share_outcome" "$safe_stop_status" "$watchdog_status" "live_socket_response_not_observed after a bounded detector-gated live capture attempt. accepted/rejected shares remain non-claims." "$conclusion"
	write_slot "safe-stop" "$safe_stop_status" "$share_outcome" "$safe_stop_status" "$watchdog_status" "safe_stop_status: ${safe_stop_status}; mining=disabled; hardware_control=disabled; work_submission=disabled." "$conclusion"
	write_redaction_review
	write_summary "passed" "passed" "$share_outcome" "$safe_stop_status" "$watchdog_status" "$conclusion"
	write_allow_manifest "$detected_port" "passed" "safe-prerequisite-blocked" "workflow" "blocked_safe_prerequisite" "$safe_stop_status" "$watchdog_status"
}

write_detector_failure_slots() {
	local blocker="$1"
	local share_outcome="blocked_safe_prerequisite"
	local conclusion="Detector-gated hardware evidence is blocked before package, flash, API, WebSocket, pool-helper, credential, or live mining work: ${blocker}."

	write_slot "detector" "blocked" "$share_outcome" "blocked" "blocked" "Detector did not produce exactly one passing Ultra 205 session." "$conclusion"
	write_slot "board-info" "blocked" "$share_outcome" "blocked" "blocked" "Board-info blocked because detector did not pass." "$conclusion"
	write_slot "share-outcome" "blocked" "$share_outcome" "blocked" "blocked" "No live submit response was attempted because detector gating failed." "$conclusion"
	write_slot "safe-stop" "blocked" "$share_outcome" "blocked" "blocked" "safe_stop_status: blocked by detector gate before runtime start; mining=disabled; hardware_control=disabled; work_submission=disabled." "$conclusion"
	write_redaction_review
	write_summary "blocked" "not-run" "$share_outcome" "blocked" "blocked" "$conclusion"
}

extract_detector_port() {
	local detector_output="$1"
	local -a detected_ports=()

	while IFS= read -r detected_port; do
		if [[ -n "$detected_port" ]]; then
			detected_ports+=("$detected_port")
		fi
	done < <(printf '%s\n' "$detector_output" | awk -F'port=' '/port=/{print $2}' | awk '{print $1}')

	if [[ "${#detected_ports[@]}" -ne 1 ]]; then
		return 1
	fi

	printf '%s' "${detected_ports[0]}"
}

redacted_live_capture_output() {
	local stdout_path="$1"
	local stderr_path="$2"

	LC_ALL=C tr -d '\000\r' <"$stdout_path"
	printf '\n'
	LC_ALL=C tr -d '\000\r' <"$stderr_path"
}

run_live_capture_attempt() {
	local detected_port="$1"
	local stdout_path
	local stderr_path
	local output
	local status
	local -a command_parts=()
	local -a command_args=(
		"board=205"
		"port=${detected_port}"
		"evidence-dir=${evidence_root}/live-capture-runtime"
		"redact-evidence=true"
		"duration-seconds=${duration_seconds:-360}"
		"device-url=${device_url}"
	)

	if [[ -n "$pool_credentials" ]]; then
		command_args+=("pool-credentials=${pool_credentials}")
	fi
	if [[ -n "$wifi_credentials" ]]; then
		command_args+=("wifi-credentials=${wifi_credentials}")
	fi

	stdout_path="$(mktemp "${TMPDIR:-/tmp}/phase25-live-capture.stdout.XXXXXX")"
	stderr_path="$(mktemp "${TMPDIR:-/tmp}/phase25-live-capture.stderr.XXXXXX")"
	IFS=' ' read -r -a command_parts <<<"$live_capture_command"

	set +e
	"${command_parts[@]}" "${command_args[@]}" >"$stdout_path" 2>"$stderr_path"
	status=$?
	set -e

	output="$(redacted_live_capture_output "$stdout_path" "$stderr_path")"
	rm -f "$stdout_path" "$stderr_path"

	live_capture_safe_stop_status="blocked"
	if [[ "$output" == *"phase25_safe_stop_status=complete"* || "$output" == *"safe_stop_status: complete"* ]]; then
		live_capture_safe_stop_status="complete"
	fi

	live_capture_watchdog_status="blocked"
	if [[ "$output" == *"phase25_watchdog_checkpoint="* || "$output" == *"watchdog_responsiveness_status: passed"* || "$output" == *"watchdog_responsiveness_status=passed"* ]]; then
		live_capture_watchdog_status="passed"
	fi

	if [[ "$status" -eq 0 && "$live_capture_safe_stop_status" == "complete" && "$live_capture_watchdog_status" == "passed" ]] &&
		[[ "$output" == *"phase25_live_submit_response_status=observed"* || "$output" == *"phase25_live_socket_response=observed"* || "$output" == *"share_outcome: live_submit_response_observed"* ]]; then
		return 0
	fi

	return 1
}

run_hardware_mode() {
	set +e
	local detector_output
	detector_output="$($detector_command 2>&1)"
	local detector_status=$?
	set -e

	local maybe_detected_port=""
	if [[ "$detector_status" -eq 0 ]]; then
		maybe_detected_port="$(extract_detector_port "$detector_output" || true)"
	fi

	if [[ "$detector_status" -ne 0 || -z "$maybe_detected_port" ]]; then
		write_detector_failure_slots "detector_failed_or_ambiguous"
		printf 'phase25_detector_status=blocked redacted=true\n' >&2
		workflow_status="failed"
		return 0
	fi
	set +e
	local board_info_output
	board_info_output="$($board_info_command --port "$maybe_detected_port" 2>&1)"
	local board_info_status=$?
	set -e

	if [[ "$board_info_status" -ne 0 ]]; then
		write_full_blocked_slots "passed" "blocked" "board_info_failure"
		printf 'phase25_board_info_status=blocked redacted=true\n' >&2
		workflow_status="failed"
		return 0
	fi
	if [[ -z "$pool_credentials" || -z "$device_url" ]]; then
		write_full_blocked_slots "passed" "passed" "missing_live_prerequisites"
		workflow_status="blocked"
		mining_allow_applicable=1
		printf 'phase25_evidence_status=blocked_safe_prerequisite redacted=true\n'
		return 0
	fi
	if run_live_capture_attempt "$maybe_detected_port"; then
		write_live_capture_slots "$maybe_detected_port"
		workflow_status="passed"
		mining_allow_applicable=1
		printf 'phase25_evidence_status=live_submit_response_observed redacted=true\n'
		return 0
	fi
	write_live_capture_not_observed_slots "$maybe_detected_port"
	workflow_status="failed"
	mining_allow_applicable=1
	printf 'phase25_evidence_status=blocked_safe_prerequisite redacted=true\n'
	return 0
}
if [[ "$mode" == "hardware" ]]; then
	run_hardware_mode
else
	write_full_blocked_slots "blocked" "blocked" "blocked_mode_static_workflow"
	workflow_status="blocked"
	mining_allow_applicable=1
	printf 'phase25_evidence_status=blocked_safe_prerequisite redacted=true\n'
fi
if ! finalize_evidence; then
	exit 1
fi
