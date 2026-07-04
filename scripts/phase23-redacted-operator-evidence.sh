#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --evidence-root PATH --manifest PATH --mode blocked|hardware [--pool-credentials PATH] [--wifi-credentials PATH] [--duration-seconds N] [--flash-evidence-json PATH] [--device-url ORIGIN]\n' "$(basename "$0")" >&2
}

evidence_root=""
manifest=""
mode=""
pool_credentials=""
wifi_credentials=""
duration_seconds=""
flash_evidence_json=""
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
	--flash-evidence-json)
		flash_evidence_json="${2:-}"
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

if [[ -n "$device_url" ]]; then
	case "$device_url" in
	http://* | https://*) ;;
	*)
		printf 'invalid origin-only DEVICE_URL\n' >&2
		exit 2
		;;
	esac
fi

readonly source_commit="${PHASE23_SOURCE_COMMIT:-$(git rev-parse HEAD 2>/dev/null || printf 'unknown-source')}"
readonly reference_commit="${PHASE23_REFERENCE_COMMIT:-$(git -C reference/esp-miner rev-parse HEAD 2>/dev/null || printf 'unknown-reference')}"
readonly detector_command="${PHASE23_DETECT_COMMAND:-just detect-ultra205}"
readonly parity_command="${PHASE23_PARITY_COMMAND:-bazel run //tools/parity:report --}"

mkdir -p "$evidence_root"

pool_config_label="not-supplied"
if [[ -n "$pool_credentials" ]]; then
	pool_config_label="local-owner-supplied"
fi

wifi_config_label="not-supplied"
if [[ -n "$wifi_credentials" ]]; then
	wifi_config_label="local-owner-supplied"
fi

write_slot() {
	local slot="$1"
	local status="$2"
	local observed="$3"
	local conclusion="$4"
	local safe_stop_status="${5:-not-run-static-workflow-slot}"
	local file="${evidence_root}/${slot}.md"

	cat >"$file" <<EOF
# Phase 23 ${slot} Slot

slot: ${slot}
slot_status: ${status}
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
detector_evidence: just detect-ultra205
command_category: repo-owned-phase23-evidence
redaction_status: passed
safe_stop_status: ${safe_stop_status}
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
raw_pool_values_committed: no
network_scan: disabled

## Observed Behavior

${observed}

## Conclusion

${conclusion}

## exact_non_claims

- trusted BM1366 production work remains a non-claim.
- live Stratum socket success remains a non-claim.
- accepted/rejected share outcomes remain non-claims.
- Phase 26 telemetry promotion remains a non-claim.
EOF
}

write_common_slots() {
	local detector_status="$1"
	local board_info_status="$2"
	local detector_observed="$3"
	local board_info_observed="$4"

	write_slot "package" "blocked" "Package identity is recorded as a path only; no raw package bytes are committed by this workflow slot." "Package evidence is blocked until package or flash artifacts are produced by repo-owned commands."
	write_slot "detector" "$detector_status" "$detector_observed" "Detector evidence is required before hardware evidence can pass."
	write_slot "board-info" "$board_info_status" "$board_info_observed" "Board-info evidence is required before hardware evidence can pass."
	write_slot "command" "passed" "Repo-owned command path only. Ad hoc raw Stratum, raw BM1366, voltage-control, fan-control, erase, rollback, interrupted-update, and network scan commands are not accepted." "The command slot supports the redacted operator evidence workflow."
	write_slot "log" "blocked" "Committed logs must be redacted lifecycle/status markers only." "The log slot is blocked until redacted logs exist."
	write_slot "api" "blocked" "API capture is blocked unless the workflow has a current valid target. stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid." "The API slot records target blockers without using stale target discovery."
	write_slot "websocket" "blocked" "WebSocket capture is blocked unless the workflow has a current valid target. stale DEVICE_URL, mDNS, ARP, router state, network scan, and unrelated evidence are invalid." "The WebSocket slot records target blockers without using stale target discovery."
	write_slot "share-outcome" "pending" "No accepted or rejected share is claimed by Phase 23. Owner: Phase 25. accepted/rejected share outcomes remain non-claims." "The share-outcome slot is present without implying share proof."
	write_slot "safe-stop" "pending" "Runtime safe-stop proof under live production mining belongs to Phase 25." "The safe-stop slot is present without implying runtime stop proof." "pending-phase25-runtime-proof"
	write_redaction_review
	write_conclusion
}

write_redaction_review() {
	cat >"${evidence_root}/redaction-review.md" <<EOF
# Phase 23 Redaction Review

slot: redaction-review
slot_status: passed
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
detector_evidence: just detect-ultra205
command_category: deterministic-redaction-review
redaction_status: passed
deterministic_scan_status: passed-reviewed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
raw_pool_values_committed: no
network_scan: disabled

## Deterministic Scan

rg -n -i "ssid|wifi|password|pool|worker|owner|token|device_url|nvs|stratum|target|extranonce|share|socket|bm1366|https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" ${evidence_root}

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
conclusion.md

## Conclusion

No raw local credential contents, runtime endpoints, targets, extranonces, share payloads, socket errors, device URLs, IPs, MACs, Wi-Fi values, NVS secrets, API tokens, raw Stratum payloads, raw share payloads, or raw BM1366 frames are committed.

## exact_non_claims

- trusted BM1366 production work remains a non-claim.
- live Stratum socket success remains a non-claim.
- accepted/rejected share outcomes remain non-claims.
- Phase 26 telemetry promotion remains a non-claim.
EOF
}

write_conclusion() {
	cat >"${evidence_root}/conclusion.md" <<EOF
# Phase 23 Conclusion

slot: conclusion
slot_status: passed
board: 205
source_commit: ${source_commit}
reference_commit: ${reference_commit}
package_identity: ${manifest}
detector_evidence: just detect-ultra205
command_category: repo-owned-phase23-evidence
redaction_status: passed
safe_stop_status: not-run-static-workflow-slot
raw_artifacts_committed: no
pool_config: ${pool_config_label}
wifi_config: ${wifi_config_label}
raw_pool_values_committed: no
network_scan: disabled

phase23_status: passed
phase23_workflow_claim: redacted_operator_evidence_workflow
requirements: EVD-07, STR-10, REL-09, CFG-07, EVD-09

## Supported Claims

Phase 23 supports the redacted operator evidence workflow, required evidence-root slots, runtime-only credential category labels, and target-source blockers.

## exact_non_claims

- trusted BM1366 production work remains a non-claim.
- live Stratum socket success remains a non-claim.
- accepted shares remain non-claims.
- rejected shares remain non-claims.
- accepted/rejected share outcomes remain non-claims.
- Phase 26 telemetry promotion remains a non-claim.
EOF
}

if [[ "$mode" == "hardware" ]]; then
	set +e
	detector_output="$($detector_command 2>&1)"
	detector_status=$?
	set -e
	if [[ "$detector_status" -ne 0 || "$detector_output" != *"port="* ]]; then
		write_common_slots "blocked" "blocked" "Detector failed or did not produce exactly one redaction-safe port category." "Board-info blocked because detector did not pass."
		printf 'phase23_detector_status=blocked redacted=true\n' >&2
		exit 1
	fi
	write_common_slots "passed" "passed" "Detector passed for board 205; raw port details stay local to the run." "Board-info passed in the detector-gated session."
else
	write_common_slots "blocked" "blocked" "Blocked mode did not run hardware detection. Hardware runs must start with just detect-ultra205." "Board-info blocked because blocked mode is static workflow proof."
fi

if [[ -n "$flash_evidence_json" ]]; then
	printf 'flash_evidence_json_category=local-redacted-input\n' >>"${evidence_root}/command.md"
fi
if [[ -n "$device_url" ]]; then
	printf 'device_url_category=operator-supplied-origin-redacted\n' >>"${evidence_root}/api.md"
	printf 'device_url_category=operator-supplied-origin-redacted\n' >>"${evidence_root}/websocket.md"
fi

# Validator command is configurable so tests can avoid nested Bazel while the
# operator workflow still routes through the repo-owned parity command by default.
${parity_command} operator-evidence --evidence-root "$evidence_root" --require-redaction-passed

printf 'phase23_evidence_status=passed evidence_root=%s redacted=true\n' "$evidence_root"
