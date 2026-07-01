#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH --surface thermal-fan [--serial-log PATH]\n' "$(basename "$0")" >&2
}

manifest=""
out_dir=""
surface=""
serial_log=""

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
readonly log_file="${out_dir}/thermal-fan.log"
: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

pending() {
	local reason="$1"

	log "phase14_thermal_fan_status: pending - ${reason}"
	log "thermal_fan_status: pending - ${reason}"
	log "thermal_claim: read-only-observation"
	log "fan_rpm_claim: read-only-observation"
	log "fan_duty_status: pending - no production-safe bounded fan-duty route exists"
	log "checklist_rows: THR-001,THR-002,THR-003"
}

run_safety_allow() {
	local allowed_command="scripts/phase14-thermal-fan.sh --manifest ${manifest} --surface ${surface} --out-dir ${out_dir}"

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

log "phase14_thermal_fan"
log "surface: ${surface:-missing}"
log "manifest: ${manifest:-missing}"
log "out_dir: ${out_dir}"
log "serial_log: ${serial_log:-unavailable}"
log "safety_allow_command: bazel run //tools/parity:report -- safety-allow --manifest ${manifest:-missing} --surface ${surface:-missing} --allowed-command scripts/phase14-thermal-fan.sh --manifest ${manifest:-missing} --surface ${surface:-missing} --out-dir ${out_dir}"

if [[ "$surface" != "thermal-fan" ]]; then
	pending "unsupported surface"
	exit 0
fi

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

log "thermal_claim: read-only-observation"
log "fan_rpm_claim: read-only-observation"
if grep -Fq "thermal_hardware_evidence_pending" "$serial_log"; then
	log "phase14_thermal_fan_status: pending - thermal_hardware_evidence_pending"
	log "thermal_fan_status: pending - thermal_hardware_evidence_pending"
else
	log "phase14_thermal_fan_status: pending - no fresh EMC2101 or fan RPM route observed"
	log "thermal_fan_status: pending - no fresh EMC2101 or fan RPM route observed"
fi
log "THR-001 conclusion: read-only thermal observation remains pending without fresh artifact"
log "THR-002 conclusion: fan RPM observation remains pending without fresh artifact"
log "THR-003 conclusion: pure PID coverage remains unit evidence only"
log "fan_duty_status: pending - no production-safe bounded fan-duty route exists"
log "non_claims: fan duty effects, overheat stimulus, fault injection"
