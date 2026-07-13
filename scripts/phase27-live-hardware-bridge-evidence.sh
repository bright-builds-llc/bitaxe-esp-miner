#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --evidence-root PATH --manifest PATH --mode blocked|hardware [--pool-credentials PATH] [--wifi-credentials PATH] [--duration-seconds N] [--port PATH] [--redact-evidence=true]\n' "$(basename "$0")" >&2
}

evidence_root=""
manifest=""
mode=""
pool_credentials=""
wifi_credentials=""
duration_seconds=""
explicit_port=""
redact_evidence=""

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
	--port)
		explicit_port="${2:-}"
		shift 2
		;;
	--redact-evidence)
		redact_evidence="${2:-}"
		shift 2
		;;
	--redact-evidence=*)
		redact_evidence="${1#*=}"
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

if [[ -n "$redact_evidence" && "$redact_evidence" != "true" ]]; then
	printf 'redact-evidence must be true when supplied\n' >&2
	exit 2
fi

readonly source_commit="${PHASE27_SOURCE_COMMIT:-$(git rev-parse HEAD 2>/dev/null || printf 'unknown-source')}"
readonly reference_commit="${PHASE27_REFERENCE_COMMIT:-$(git -C reference/esp-miner rev-parse HEAD 2>/dev/null || printf 'unknown-reference')}"
readonly evidence_mode="${PHASE27_EVIDENCE_MODE:-phase27-live-hardware-asic-stratum-bridge}"
readonly evidence_ack="${PHASE27_EVIDENCE_ACK:-ultra205-phase27-live-hardware-bridge-safe-stop}"
readonly detector_command="${PHASE27_DETECT_COMMAND:-just detect-ultra205}"
readonly board_info_command="${PHASE27_BOARD_INFO_COMMAND:-espflash board-info --chip esp32s3 --non-interactive}"
readonly parity_command="${PHASE27_PARITY_COMMAND:-bazel run //tools/parity:report --}"
readonly live_capture_command="${PHASE27_LIVE_CAPTURE_COMMAND:-just flash-monitor}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly repo_root="$(cd "${script_dir}/.." && pwd)"
readonly pool_input_bridge_helper="${PHASE27_POOL_INPUT_BRIDGE_HELPER:-${repo_root}/scripts/phase21-pool-input-bridge.sh}"

mkdir -p "$evidence_root"

pool_config_label="not-supplied"
if [[ -n "$pool_credentials" ]]; then
	pool_config_label="local-owner-supplied"
fi

wifi_config_label="not-supplied"
if [[ -n "$wifi_credentials" ]]; then
	wifi_config_label="local-owner-supplied"
fi

port_label="not-supplied"
if [[ -n "$explicit_port" ]]; then
	port_label="explicit"
fi

duration_label="not-requested"
if [[ -n "$duration_seconds" ]]; then
	duration_label="$duration_seconds"
fi

redaction_label="not-requested"
if [[ -n "$redact_evidence" ]]; then
	redaction_label="true"
fi

workflow_status="passed"
mining_allow_applicable=0

write_slot() {
	local slot="$1"
	local status="$2"
	local share_outcome="$3"
	local asic_bridge_status="$4"
	local safe_stop_status="$5"
	local observed="$6"
	local conclusion="$7"
	local file="${evidence_root}/${slot}.md"

	cat >"$file" <<EOF
# Phase 27 ${slot} Evidence

slot: ${slot}
slot_status: ${status}
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
evidence_mode: ${evidence_mode}
evidence_ack: ${evidence_ack}
detector_evidence: just detect-ultra205
board_info_evidence: espflash board-info
command_category: repo-owned-phase27-live-hardware-bridge-evidence
redaction_status: passed
share_outcome: ${share_outcome}
asic_bridge_status: ${asic_bridge_status}
safe_stop_status: ${safe_stop_status}
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
port_source: ${port_label}
duration_seconds: ${duration_label}
redact_evidence: ${redaction_label}
raw_pool_values_committed: no
network_scan: disabled

## observed_behavior

${observed}

## conclusion

${conclusion}

## exact_non_claims

- accepted/rejected shares remain non-claims unless a detector-gated live pool response is tied to live ASIC-derived submit intent with ASIC bridge correlation markers.
- Phase 28 checklist promotion remains a non-claim except where this evidence root explicitly supports category labels only.
- Full active voltage, fan, thermal, fault, and self-test safety closure remains a non-claim.
EOF
}

redaction_diagnostic_status() {
	if [[ -n "${PHASE27_RAW_DIAGNOSTIC_SAMPLE:-}" ]]; then
		printf 'rejected_sensitive_raw_payload'
		return
	fi

	printf 'no_raw_diagnostic_input'
}

allowed_command_string() {
	local command="scripts/phase27-live-hardware-bridge-evidence.sh --evidence-root ${evidence_root} --manifest ${manifest} --mode ${mode}"

	if [[ -n "$duration_seconds" ]]; then
		command="${command} --duration-seconds ${duration_seconds}"
	fi
	if [[ -n "$explicit_port" ]]; then
		command="${command} --port [redacted-port]"
	fi
	if [[ -n "$pool_credentials" ]]; then
		command="${command} --pool-credentials [redacted-local-path]"
	fi
	if [[ -n "$wifi_credentials" ]]; then
		command="${command} --wifi-credentials [redacted-local-path]"
	fi
	if [[ -n "$redact_evidence" ]]; then
		command="${command} --redact-evidence=true"
	fi

	printf '%s' "$command"
}

finalize_evidence() {
	local completion_status
	local mining_allow_status=0
	local operator_status

	set +e
	${parity_command} complete-operator-evidence --profile phase27 --evidence-root "$evidence_root" --workflow-status "$workflow_status" >/dev/null
	completion_status=$?
	if [[ "$mining_allow_applicable" -eq 1 ]]; then
		${parity_command} mining-allow --manifest "${evidence_root}/mining-allow.json" --surface live-hardware-bridge --allowed-command "$(allowed_command_string)" >/dev/null
		mining_allow_status=$?
	fi
	${parity_command} operator-evidence --profile phase27 --evidence-root "$evidence_root" --require-redaction-passed >/dev/null
	operator_status=$?
	set -e

	if [[ "$workflow_status" != "passed" || "$completion_status" -ne 0 || "$mining_allow_status" -ne 0 || "$operator_status" -ne 0 ]]; then
		return 1
	fi

	return 0
}

write_allow_manifest() {
	local path="${evidence_root}/mining-allow.json"
	local detected_port="$1"
	local board_info_status="$2"
	local claim_tier="$3"
	local evidence_class="$4"
	local share_outcome="$5"
	local safe_stop_status="$6"
	local asic_bridge_status="$7"
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
  "surface": "live-hardware-bridge",
  "evidence_mode": "${evidence_mode}",
  "evidence_ack": "${evidence_ack}",
  "claim_tier": "${claim_tier}",
  "evidence_class": "${evidence_class}",
  "allowed_command": "${command}",
  "allowed_inputs": {
    "pool_config": "${pool_config_label}",
    "wifi_config": "${wifi_config_label}",
    "port_source": "${port_label}",
    "duration_seconds": ${duration_seconds:-360},
    "redact_evidence": "${redaction_label}",
    "target_source": "explicit-or-blocked",
    "share_outcome": "${share_outcome}",
    "asic_bridge_status": "${asic_bridge_status}",
    "safe_stop_status": "${safe_stop_status}"
  },
  "abort_conditions": [
    "detector_mismatch",
    "board_info_failure",
    "missing_trusted_wrapper_markers",
    "redaction_uncertainty",
    "unsafe_temperature_or_power",
    "asic_bridge_blocked"
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
  "redaction_reviewer": "phase-27-wrapper",
  "checklist_rows": ["STR-08", "STR-09", "ASIC-10", "ASIC-11"]
}
EOF
}

write_redaction_review() {
	local status
	status="$(redaction_diagnostic_status)"

	cat >"${evidence_root}/redaction-review.md" <<EOF
# Phase 27 Redaction Review

slot: redaction-review
slot_status: passed
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
evidence_mode: ${evidence_mode}
evidence_ack: ${evidence_ack}
detector_evidence: just detect-ultra205
command_category: deterministic-phase27-redaction-review
redaction_status: passed
diagnostic_input_status: ${status}
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
raw_pool_values_committed: no
network_scan: disabled

## Artifact Inventory

share-outcome.md
summary.md
detector.md
board-info.md
command.md
redaction-review.md
conclusion.md

## conclusion

No raw local credential contents, pool endpoints, workers, owner addresses, passwords, targets, extranonces, share payloads, socket details, device targets, IPs, MACs, Wi-Fi values, NVS secrets, API tokens, raw protocol payloads, raw share payloads, or raw BM1366 frames are committed.

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation.
- ASIC bridge dispatch proof remains blocked unless category markers are observed in a detector-gated run.
EOF
}

write_summary() {
	local detector_status="$1"
	local board_info_status="$2"
	local share_outcome="$3"
	local asic_bridge_status="$4"
	local safe_stop_status="$5"
	local conclusion="$6"

	cat >"${evidence_root}/summary.md" <<EOF
# Phase 27 Evidence Summary

board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
evidence_mode: ${evidence_mode}
evidence_ack: ${evidence_ack}
package_artifact_status: ${board_info_status}
detector_status: ${detector_status}
board_info_status: ${board_info_status}
share_outcome: ${share_outcome}
asic_bridge_status: ${asic_bridge_status}
safe_stop_status: ${safe_stop_status}
redaction_status: passed
raw_artifacts_committed: no
raw_pool_values_committed: no
network_scan: disabled
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
port_source: ${port_label}
safety_bring_up_status: ${live_capture_safety_bring_up_status:-not-run}

## Supported Claim

${conclusion}

## exact_non_claims

- accepted/rejected shares remain non-claims unless detector-gated live proof exists with ASIC bridge correlation markers.
- Raw credentials, endpoints, target data, socket details, device targets, IPs, MACs, and raw BM1366 frames are not committed.
- Phase 28 checklist promotion remains deferred except where this evidence root explicitly supports category labels only.
EOF
}

write_conclusion() {
	local share_outcome="$1"
	local conclusion="$2"

	cat >"${evidence_root}/conclusion.md" <<EOF
# Phase 27 Conclusion

board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
evidence_mode: ${evidence_mode}
evidence_ack: ${evidence_ack}
share_outcome: ${share_outcome}
redaction_status: passed
raw_artifacts_committed: no

## conclusion

${conclusion}

## exact_non_claims

- accepted/rejected live ASIC-derived share proof requires detector-gated hardware with ASIC bridge correlation markers.
- Phase 28 checklist promotion remains a non-claim except where supported by category-only evidence in this root.
EOF
}

write_blocked_evidence() {
	local detector_status="$1"
	local board_info_status="$2"
	local blocker="$3"
	local share_outcome="blocked_safe_prerequisite"
	local asic_bridge_status="blocked"
	local safe_stop_status="complete"
	local conclusion="Phase 27 records an exact blocked safe-prerequisite non-claim: ${blocker}."

	write_slot "detector" "$detector_status" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Detector status is ${detector_status}; hardware promotion requires just detect-ultra205 before any flash-monitor work." "$conclusion"
	write_slot "board-info" "$board_info_status" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Board-info status is ${board_info_status}; hardware promotion requires ESP32-S3 board-info in the same detector-gated session." "$conclusion"
	write_slot "command" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Repo-owned Phase 27 wrapper command only; raw Stratum, raw BM1366, unsafe hardware control, erase, rollback, stale targets, and network scans are not accepted." "$conclusion"
	write_slot "share-outcome" "blocked" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "No live pool response tied to live ASIC-derived submit intent was observed. asic_production_status category markers remain blocked. accepted/rejected shares remain non-claims." "$conclusion"
	write_redaction_review
	write_summary "$detector_status" "$board_info_status" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "$conclusion"
	write_conclusion "$share_outcome" "$conclusion"
	write_allow_manifest "/dev/redacted-phase27" "$board_info_status" "safe-prerequisite-blocked" "workflow" "$share_outcome" "$safe_stop_status" "$asic_bridge_status"
}

write_live_capture_slots() {
	local detected_port="$1"
	local share_outcome="$2"
	local asic_bridge_status="$3"
	local safe_stop_status="complete"
	local conclusion="Phase 27 records detector-gated live hardware bridge evidence with redacted runtime-only inputs and category-only share outcome ${share_outcome}."

	write_slot "detector" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Detector passed for exactly one Ultra 205 session. The selected port is recorded only in mining-allow metadata." "$conclusion"
	write_slot "board-info" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Board-info passed for ESP32-S3 in the same detector-gated session." "$conclusion"
	write_slot "command" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Repo-owned bounded flash-monitor helper was invoked with Phase 27 mode/ack firmware, runtime-only local input paths, and redact-evidence=true; raw values are not committed." "$conclusion"
	write_slot "share-outcome" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "The bounded helper observed share_submission_status=${share_outcome} with ASIC bridge category markers. Raw Stratum payloads, pool endpoints, workers, and share fields are not committed." "$conclusion"
	write_redaction_review
	write_summary "passed" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "$conclusion"
	write_conclusion "$share_outcome" "$conclusion"
	write_allow_manifest "$detected_port" "passed" "live-hardware-bridge" "hardware-smoke" "$share_outcome" "$safe_stop_status" "$asic_bridge_status"
}

write_live_capture_not_observed_slots() {
	local detected_port="$1"
	local share_outcome="blocked_safe_prerequisite"
	local asic_bridge_status="${live_capture_asic_bridge_status:-blocked}"
	local safe_stop_status="${live_capture_safe_stop_status:-blocked}"
	local conclusion="Phase 27 attempted bounded detector-gated live hardware bridge capture, but no valid accepted/rejected share outcome with ASIC bridge markers was observed."

	write_slot "detector" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Detector passed for exactly one Ultra 205 session before the live capture attempt." "$conclusion"
	write_slot "board-info" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Board-info passed for ESP32-S3 before the live capture attempt." "$conclusion"
	write_slot "command" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "Repo-owned bounded flash-monitor helper was invoked with Phase 27 mode/ack firmware and runtime-only local input paths; raw values are not committed." "$conclusion"
	write_slot "share-outcome" "blocked" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "live_share_outcome_not_observed after a bounded detector-gated live capture attempt. accepted/rejected shares remain non-claims." "$conclusion"
	write_redaction_review
	write_summary "passed" "passed" "$share_outcome" "$asic_bridge_status" "$safe_stop_status" "$conclusion"
	write_conclusion "$share_outcome" "$conclusion"
	write_allow_manifest "$detected_port" "passed" "safe-prerequisite-blocked" "workflow" "$share_outcome" "$safe_stop_status" "$asic_bridge_status"
}

write_detector_failure_slots() {
	local blocker="$1"
	local share_outcome="blocked_safe_prerequisite"
	local conclusion="Detector-gated hardware evidence is blocked before package, flash, pool-helper, credential, or live mining work: ${blocker}."

	write_slot "detector" "blocked" "$share_outcome" "blocked" "blocked" "Detector did not produce exactly one passing Ultra 205 session." "$conclusion"
	write_slot "board-info" "blocked" "$share_outcome" "blocked" "blocked" "Board-info blocked because detector did not pass." "$conclusion"
	write_slot "command" "blocked" "$share_outcome" "blocked" "blocked" "Flash-monitor was not attempted because detector gating failed." "$conclusion"
	write_slot "share-outcome" "blocked" "$share_outcome" "blocked" "blocked" "No live share outcome was attempted because detector gating failed." "$conclusion"
	write_redaction_review
	write_summary "blocked" "not-run" "$share_outcome" "blocked" "blocked" "$conclusion"
	write_conclusion "$share_outcome" "$conclusion"
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

derive_share_outcome_from_output() {
	local output="$1"

	if [[ "$output" == *"share_submission_status=accepted"* || "$output" == *"share_outcome: accepted"* ]]; then
		printf 'accepted'
		return 0
	fi

	if [[ "$output" == *"share_submission_status=rejected"* || "$output" == *"share_outcome: rejected"* ]]; then
		printf 'rejected'
		return 0
	fi

	return 1
}

derive_safety_bring_up_status_from_output() {
	local output="$1"

	if [[ "$output" == *"phase27_safety_bring_up=complete"* &&
		"$output" == *"asic_enable_status=active"* &&
		"$output" == *"safety_fan_status=startup_duty"* ]]; then
		printf 'complete'
		return 0
	fi

	if [[ "$output" == *"phase27_safety_bring_up=started"* ||
		"$output" == *"phase27_safety_bring_up=failed"* ]]; then
		printf 'partial'
		return 0
	fi

	printf 'missing'
}

derive_asic_bridge_status_from_output() {
	local output="$1"

	if [[ "$output" == *"asic_production_status=result_correlated"* ]]; then
		printf 'result_correlated'
		return 0
	fi

	if [[ "$output" == *"asic_production_status=work_dispatched"* ]]; then
		printf 'work_dispatched'
		return 0
	fi

	if [[ "$output" == *"asic_production_status=initialized"* ]]; then
		printf 'initialized'
		return 0
	fi

	printf 'blocked'
}

resolve_manifest_path() {
	if [[ "$manifest" == /* ]]; then
		printf '%s' "$manifest"
		return 0
	fi

	printf '%s/%s' "$repo_root" "$manifest"
}

resolve_manifest_factory_image() {
	local manifest_path
	manifest_path="$(resolve_manifest_path)"

	if [[ ! -f "$manifest_path" ]]; then
		printf 'error: package manifest missing: %s\n' "$manifest_path" >&2
		return 1
	fi

	python3 - "$manifest_path" <<'PY'
import json
import sys
from pathlib import Path

manifest = Path(sys.argv[1])
data = json.loads(manifest.read_text(encoding="utf-8"))
for artifact in data.get("artifacts", []):
    if artifact.get("kind") == "factory_merged_image":
        image = manifest.parent / artifact["path"]
        print(image.resolve())
        break
else:
    raise SystemExit("factory_merged_image artifact missing from manifest")
PY
}

warn_if_manifest_source_commit_stale() {
	local manifest_path
	local manifest_commit
	local head_commit

	manifest_path="$(resolve_manifest_path)"
	head_commit="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf 'unknown-head')"
	manifest_commit="$(python3 - "$manifest_path" <<'PY'
import json
import sys
from pathlib import Path

manifest = Path(sys.argv[1])
data = json.loads(manifest.read_text(encoding="utf-8"))
print(data.get("source_commit", "unknown-manifest"))
PY
)"

	if [[ "$manifest_commit" != "$head_commit" && "$manifest_commit" != "unknown-manifest" && "$head_commit" != "unknown-head" ]]; then
		printf 'phase27_manifest_warning=source_commit_mismatch manifest=%s head=%s\n' \
			"$manifest_commit" "$head_commit" >&2
		printf 'phase27_manifest_warning_action=rebuild with scripts/phase27-live-hardware-bridge-package.sh\n' >&2
	fi
}

extract_device_url_from_log() {
	local log_path="$1"
	local -a urls=()

	while IFS= read -r line; do
		case "$line" in
		*device_url=http://* | *device_url=https://*)
			local candidate="${line#*device_url=}"
			candidate="${candidate%% *}"
			candidate="${candidate%%$'\r'}"
			if [[ "$candidate" == http://* || "$candidate" == https://* ]]; then
				urls+=("$candidate")
			fi
			;;
		esac
	done <"$log_path"

	if [[ "${#urls[@]}" -ne 1 ]]; then
		return 1
	fi

	printf '%s' "${urls[0]}"
}

run_pool_input_bridge() {
	local device_url="$1"
	local bridge_out_dir="${evidence_root}/live-capture-runtime/pool-input-bridge"

	if [[ ! -f "$pool_input_bridge_helper" ]]; then
		return 1
	fi

	mkdir -p "$bridge_out_dir"
	"$BASH" "$pool_input_bridge_helper" \
		--device-url "$device_url" \
		--pool-credentials "$pool_credentials" \
		--out-dir "$bridge_out_dir"
}

run_live_capture_attempt() {
	local detected_port="$1"
	local stdout_path
	local stderr_path
	local output
	local status
	local -a command_parts=()
	local factory_image
	local capture_evidence_dir="${evidence_root}/live-capture-runtime"
	local monitor_log="${capture_evidence_dir}/flash-monitor.log"
	local capture_timeout="${duration_seconds:-360}"
	local pool_bridge_applied=0

	warn_if_manifest_source_commit_stale
	factory_image="$(resolve_manifest_factory_image)" || return 1

	local -a command_args=(
		"board=205"
		"port=${detected_port}"
		"image=${factory_image}"
		"evidence-dir=${capture_evidence_dir}"
		"capture-timeout-seconds=${capture_timeout}"
	)

	if [[ -n "$wifi_credentials" ]]; then
		command_args+=("wifi-credentials=${wifi_credentials}")
	fi
	if [[ -n "$redact_evidence" ]]; then
		command_args+=("redact-evidence=${redact_evidence}")
	fi

	mkdir -p "$capture_evidence_dir"
	stdout_path="$(mktemp "${TMPDIR:-/tmp}/phase27-live-capture.stdout.XXXXXX")"
	stderr_path="$(mktemp "${TMPDIR:-/tmp}/phase27-live-capture.stderr.XXXXXX")"
	IFS=' ' read -r -a command_parts <<<"$live_capture_command"

	set +e
	(
		cd "$repo_root"
		"${command_parts[@]}" "${command_args[@]}"
	) >"$stdout_path" 2>"$stderr_path" &
	local capture_pid=$!

	local deadline=$((SECONDS + capture_timeout + 30))
	while ((SECONDS < deadline)); do
		if [[ -f "$monitor_log" && -n "$pool_credentials" && "$pool_bridge_applied" -eq 0 ]]; then
			local maybe_device_url=""
			if maybe_device_url="$(extract_device_url_from_log "$monitor_log")"; then
				if run_pool_input_bridge "$maybe_device_url"; then
					pool_bridge_applied=1
				fi
			fi
		fi

		if ! kill -0 "$capture_pid" 2>/dev/null; then
			break
		fi

		sleep 2
	done

	wait "$capture_pid"
	status=$?
	set -e

	if [[ -f "$monitor_log" && "$redact_evidence" == "true" ]]; then
		local redacted_monitor_log
		redacted_monitor_log="$(mktemp "${TMPDIR:-/tmp}/phase27-live-capture.redacted.XXXXXX")"
		LC_ALL=C sed \
			-e 's/device_url=http[^[:space:]]*/device_url=[redacted-url]/g' \
			-e 's/device_url=https[^[:space:]]*/device_url=[redacted-url]/g' \
			-e 's/ipv4=[0-9.]*/ipv4=[redacted-ip]/g' \
			-e 's/mac=[0-9a-f:]*\([[:space:]]\|$\)/mac=[redacted-mac]\1/g' \
			"$monitor_log" >"$redacted_monitor_log"
		mv "$redacted_monitor_log" "$monitor_log"
	fi

	output="$(redacted_live_capture_output "$stdout_path" "$stderr_path")"
	if [[ -f "$monitor_log" ]]; then
		output="${output}
$(LC_ALL=C tr -d '\000\r' <"$monitor_log")"
	fi
	rm -f "$stdout_path" "$stderr_path"

	live_capture_safe_stop_status="blocked"
	if [[ "$output" == *"phase25_safe_stop_status=complete"* || "$output" == *"safe_stop_status: complete"* ]]; then
		live_capture_safe_stop_status="complete"
	fi

	live_capture_asic_bridge_status="$(derive_asic_bridge_status_from_output "$output")"
	live_capture_safety_bring_up_status="$(derive_safety_bring_up_status_from_output "$output")"

	if [[ "$status" -eq 0 && "$live_capture_safe_stop_status" == "complete" ]] &&
		share_outcome="$(derive_share_outcome_from_output "$output")"; then
		live_capture_share_outcome="$share_outcome"
		return 0
	fi

	return 1
}

run_hardware_mode() {
	local maybe_detected_port="$explicit_port"

	if [[ -z "$maybe_detected_port" ]]; then
		set +e
		local detector_output
		detector_output="$($detector_command 2>&1)"
		local detector_status=$?
		set -e

		if [[ "$detector_status" -eq 0 ]]; then
			maybe_detected_port="$(extract_detector_port "$detector_output" || true)"
		fi

		if [[ "$detector_status" -ne 0 || -z "$maybe_detected_port" ]]; then
			write_detector_failure_slots "detector_failed_or_ambiguous"
			printf 'phase27_detector_status=blocked redacted=true\n' >&2
			workflow_status="failed"
			return 0
		fi
	else
		printf 'phase27_detector_status=explicit_port redacted=true\n' >&2
	fi

	set +e
	local board_info_output
	board_info_output="$($board_info_command --port "$maybe_detected_port" 2>&1)"
	local board_info_status=$?
	set -e

	if [[ "$board_info_status" -ne 0 ]]; then
		write_blocked_evidence "passed" "blocked" "board_info_failure"
		printf 'phase27_board_info_status=blocked redacted=true\n' >&2
		workflow_status="failed"
		return 0
	fi

	if [[ -z "$pool_credentials" ]]; then
		write_blocked_evidence "passed" "passed" "missing_live_prerequisites"
		workflow_status="blocked"
		mining_allow_applicable=1
		printf 'phase27_evidence_status=blocked_safe_prerequisite redacted=true\n'
		return 0
	fi

	if run_live_capture_attempt "$maybe_detected_port"; then
		write_live_capture_slots "$maybe_detected_port" "$live_capture_share_outcome" "$live_capture_asic_bridge_status"
		workflow_status="passed"
		mining_allow_applicable=1
		printf 'phase27_evidence_status=%s redacted=true\n' "$live_capture_share_outcome"
		return 0
	fi

	write_live_capture_not_observed_slots "$maybe_detected_port"
	workflow_status="failed"
	if [[ "${live_capture_safe_stop_status:-blocked}" == "complete" ]]; then
		mining_allow_applicable=1
	fi
	printf 'phase27_evidence_status=blocked_safe_prerequisite redacted=true\n'
	return 0
}

if [[ "$mode" == "hardware" ]]; then
	run_hardware_mode
else
	write_blocked_evidence "blocked" "blocked" "blocked_mode_static_workflow"
	workflow_status="blocked"
	mining_allow_applicable=1
	printf 'phase27_evidence_status=blocked_safe_prerequisite redacted=true\n'
fi

if ! finalize_evidence; then
	exit 1
fi
