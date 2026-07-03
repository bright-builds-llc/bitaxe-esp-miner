#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH [--serial-log PATH] [--stimulus NAME]\n' "$(basename "$0")" >&2
}

manifest=""
out_dir=""
serial_log=""
stimulus=""
readonly surface="failure-paths"

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
	--serial-log)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		serial_log="$2"
		shift 2
		;;
	--stimulus)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		stimulus="$2"
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
readonly log_file="${out_dir}/failure-paths.log"
: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

record_required_fields() {
	log "fault_stimulus_status: not_run"
	log "expected_fault_status: not_observed"
	log "api_projection_status: not_run"
	log "websocket_projection_status: not_run"
	log "final_safe_state_status: required-before-promotion"
	log "required_stimulus: missing - future plan must name bounded stimulus"
	log "required_expected_fault: missing - future plan must name expected fault"
	log "required_abort_condition: missing - future plan must name abort condition"
	log "required_recovery_path: missing - future plan must name restore path"
	log "required_api_projection: not_run - future plan must capture user-visible API projection"
	log "required_websocket_projection: not_run - future plan must capture user-visible WebSocket projection"
	log "required_final_safe_state_marker: missing - future plan must observe final safe-state marker"
	log "active_rows_status: below_verified"
	log "checklist_rows: PWR-001,PWR-002,THR-001,THR-002,SELF-001,SAFE-04"
	log "non_claims: overheat stimulus, fan fault stimulus, power fault stimulus, thermal fault stimulus, ASIC fault stimulus"
}

blocked() {
	local reason="$1"

	log "failure_paths_status: blocked - ${reason}"
	record_required_fields
}

run_safety_allow() {
	local allowed_command="scripts/phase20-failure-paths.sh --manifest ${manifest} --out-dir ${out_dir}"

	if [[ -n "${PHASE20_SAFETY_ALLOW_BIN:-}" ]]; then
		"$PHASE20_SAFETY_ALLOW_BIN" \
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

log "phase20_failure_paths"
log "surface: ${surface}"
log "manifest: ${manifest:-missing}"
log "out_dir: ${out_dir}"
log "serial_log: ${serial_log:-unavailable}"
log "requested_stimulus: ${stimulus:-none}"
log "safety_allow_command: bazel run //tools/parity:report -- safety-allow --manifest ${manifest:-missing} --surface ${surface} --allowed-command scripts/phase20-failure-paths.sh --manifest ${manifest:-missing} --out-dir ${out_dir}"

if [[ -z "$manifest" || ! -f "$manifest" ]]; then
	blocked "missing manifest"
	exit 0
fi

set +e
allow_output="$(run_safety_allow 2>&1)"
allow_status=$?
set -e

printf '%s\n' "$allow_output" >>"$log_file"
if [[ "$allow_status" -ne 0 ]] || ! grep -Fq "safety_allow_status: passed" <<<"$allow_output"; then
	blocked "allow validation failed"
	exit 0
fi

blocked "no production-safe fault stimulus route"
