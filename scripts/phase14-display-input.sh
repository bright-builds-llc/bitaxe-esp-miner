#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH [--serial-log PATH]\n' "$(basename "$0")" >&2
}

manifest=""
out_dir=""
serial_log=""
readonly surface="display-input"

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
readonly log_file="${out_dir}/display-input.log"
: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

record_pending_conclusions() {
	log "runtime_display_input_status: pending - no runtime display/input route or physical input observation"
	log "display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true"
	log "display_input_non_claims: runtime display pages, screen flow, LVGL parity, input hardware behavior"
	log "checklist_rows: IO-001,UI-001,UI-002,UI-003"
}

pending() {
	local reason="$1"

	log "phase14_display_input_status: pending - ${reason}"
	log "startup_display_status: pending - ${reason}"
	record_pending_conclusions
}

run_safety_allow() {
	local allowed_command="scripts/phase14-display-input.sh --manifest ${manifest} --out-dir ${out_dir}"

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

log "phase14_display_input"
log "surface: ${surface}"
log "manifest: ${manifest:-missing}"
log "out_dir: ${out_dir}"
log "serial_log: ${serial_log:-unavailable}"
log "safety_allow_command: bazel run //tools/parity:report -- safety-allow --manifest ${manifest:-missing} --surface ${surface} --allowed-command scripts/phase14-display-input.sh --manifest ${manifest:-missing} --out-dir ${out_dir}"

if [[ -z "$manifest" || ! -f "$manifest" ]]; then
	pending "missing manifest"
	exit 0
fi

set +e
allow_output="$(run_safety_allow 2>&1)"
allow_status=$?
set -e

printf '%s\n' "$allow_output" >>"$log_file"
if [[ "$allow_status" -ne 0 ]] || ! grep -Fq "safety_allow_status: passed" <<<"$allow_output"; then
	pending "allow validation failed"
	exit 0
fi

if [[ -z "$serial_log" || ! -f "$serial_log" ]]; then
	pending "serial log unavailable"
	exit 0
fi

readonly startup_marker="display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c"
readonly runtime_gap_marker="display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true"

if grep -Fq "$startup_marker" "$serial_log"; then
	log "$startup_marker"
	log "startup_display_status: observed"
else
	log "startup_display_status: pending - startup marker missing"
fi

if grep -Fq "$runtime_gap_marker" "$serial_log"; then
	log "$runtime_gap_marker"
	log "runtime_gap_marker_status: observed"
else
	log "display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true"
	log "runtime_gap_marker_status: pending - runtime gap marker missing"
fi

record_pending_conclusions
log "phase14_display_input_status: pending - runtime display/input route unavailable"
