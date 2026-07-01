#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH [--serial-log PATH]\n' "$(basename "$0")" >&2
}

manifest=""
out_dir=""
serial_log=""
readonly surface="self-test-watchdog-load"

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
readonly log_file="${out_dir}/self-test-watchdog-load.log"
: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

record_pending_conclusions() {
	log "load_stress_status: pending - bounded workload stimulus unavailable"
	log "self_test_hardware_status: pending - no production-safe self-test hardware submode route exists"
	log "SELF-001 conclusion: below verified until hardware-regression proves exact submode, pass/fail/cancel, recovery, and production-mining gate behavior"
	log "watchdog_load_non_claims: self-test hardware submodes, reboot, mining, ASIC diagnostic work, voltage/fan/ASIC work, bounded load stress"
	log "checklist_rows: SELF-001,SAFE-09"
}

pending() {
	local reason="$1"

	log "phase14_self_test_watchdog_load_status: pending - ${reason}"
	log "watchdog_supervisor_status: pending - ${reason}"
	record_pending_conclusions
}

run_safety_allow() {
	local allowed_command="scripts/phase14-self-test-watchdog-load.sh --manifest ${manifest} --out-dir ${out_dir}"

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

log "phase14_self_test_watchdog_load"
log "surface: ${surface}"
log "manifest: ${manifest:-missing}"
log "out_dir: ${out_dir}"
log "serial_log: ${serial_log:-unavailable}"
log "safety_allow_command: bazel run //tools/parity:report -- safety-allow --manifest ${manifest:-missing} --surface ${surface} --allowed-command scripts/phase14-self-test-watchdog-load.sh --manifest ${manifest:-missing} --out-dir ${out_dir}"

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

readonly supervisor_start_marker="safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100"
readonly supervisor_yield_marker="safety_supervisor_step=yield reason=yield_interval_reached"

if grep -Fq "$supervisor_start_marker" "$serial_log" && grep -Fq "$supervisor_yield_marker" "$serial_log"; then
	log "watchdog_supervisor_status: observed"
	log "watchdog_supervisor_start_marker: observed - ${supervisor_start_marker}"
	log "watchdog_supervisor_yield_marker: observed - ${supervisor_yield_marker}"
else
	log "watchdog_supervisor_status: pending - supervisor markers missing"
fi

record_pending_conclusions
log "phase14_self_test_watchdog_load_status: pending - self-test hardware and bounded load routes unavailable"
