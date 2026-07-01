#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --manifest PATH --out-dir PATH --surface SURFACE [--serial-log PATH]\n' "$(basename "$0")" >&2
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
readonly log_file="${out_dir}/power-voltage.log"
: >"$log_file"

log() {
	printf '%s\n' "$*" | tee -a "$log_file" >/dev/null
}

pending() {
	local reason="$1"

	log "phase14_power_voltage_status: pending - ${reason}"
	case "$surface" in
	power-telemetry)
		log "phase14_power_telemetry_status: pending - ${reason}"
		log "power_telemetry_status: pending - hardware_evidence_pending"
		log "power_telemetry_claim: read-only-observation"
		log "voltage_control_status: pending - no production-safe bounded voltage route exists"
		log "PWR-003 conclusion: below verified until hardware-regression exists"
		log "PWR-005 conclusion: below verified until hardware-regression exists"
		log "checklist_rows: PWR-006"
		;;
	voltage-control)
		log "phase14_voltage_control_status: pending - ${reason}"
		log "voltage_control_claim: unsupported-pending"
		log "voltage_control_status: pending - no production-safe bounded voltage route exists"
		log "checklist_rows: PWR-003,PWR-005"
		;;
	*)
		log "phase14_power_voltage_surface_status: pending - unsupported surface ${surface:-missing}"
		;;
	esac
	log "non_claims: DS4432U active voltage writes, ASIC power sequencing, stale cached values"
}

run_safety_allow() {
	local allowed_command="scripts/phase14-power-voltage.sh --manifest ${manifest} --surface ${surface} --out-dir ${out_dir}"

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

log "phase14_power_voltage"
log "surface: ${surface:-missing}"
log "manifest: ${manifest:-missing}"
log "out_dir: ${out_dir}"
log "serial_log: ${serial_log:-unavailable}"
log "allowed_surfaces: power-telemetry,voltage-control"
log "safety_allow_command: bazel run //tools/parity:report -- safety-allow --manifest ${manifest:-missing} --surface ${surface:-missing} --allowed-command scripts/phase14-power-voltage.sh --manifest ${manifest:-missing} --surface ${surface:-missing} --out-dir ${out_dir}"

case "$surface" in
power-telemetry | voltage-control) ;;
*)
	pending "unsupported surface"
	exit 0
	;;
esac

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

case "$surface" in
power-telemetry)
	log "power_telemetry_claim: read-only-observation"
	if grep -Fq "hardware_evidence_pending" "$serial_log"; then
		log "power_telemetry_status: pending - hardware_evidence_pending"
		log "phase14_power_telemetry_status: pending - hardware_evidence_pending"
	else
		log "power_telemetry_status: pending - no fresh INA260 route observed"
		log "phase14_power_telemetry_status: pending - no fresh INA260 route observed"
	fi
	log "PWR-006 conclusion: read-only observation remains pending without fresh INA260 artifact"
	log "voltage_control_status: pending - no production-safe bounded voltage route exists"
	log "PWR-003 conclusion: below verified until hardware-regression exists"
	log "PWR-005 conclusion: below verified until hardware-regression exists"
	log "phase14_power_voltage_status: pending - read-only route unavailable"
	;;
voltage-control)
	log "voltage_control_claim: unsupported-pending"
	log "voltage_control_status: pending - no production-safe bounded voltage route exists"
	log "PWR-003 conclusion: below verified until hardware-regression exists"
	log "PWR-005 conclusion: below verified until hardware-regression exists"
	log "phase14_power_voltage_status: pending - bounded voltage route unavailable"
	;;
esac

log "non_claims: active voltage control, ASIC power sequencing, stale cached values"
