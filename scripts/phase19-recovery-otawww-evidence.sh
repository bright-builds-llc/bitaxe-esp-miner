#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat >&2 <<'USAGE'
usage: phase19-recovery-otawww-evidence.sh --manifest PATH --factory-image PATH --ota-image PATH --port PATH --out-dir PATH [--device-url URL | --device-url-from-flash-evidence PATH] [--target-lock-out PATH] [--allow-failed-update] [--allow-large-erase] [--allow-interrupted-ota] [--otawww-gap-only]
USAGE
}

resolve_phase19_script_dir() {
	local direct_dir
	direct_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

	local candidate_dir
	for candidate_dir in \
		"$direct_dir" \
		"${BASH_SOURCE[0]}.runfiles/_main/scripts" \
		"${RUNFILES_DIR:-}/_main/scripts"; do
		if [[ -f "${candidate_dir}/phase19-recovery-otawww-target.sh" ]] &&
			[[ -f "${candidate_dir}/phase19-recovery-otawww-probe.sh" ]] &&
			[[ -f "${candidate_dir}/phase19-recovery-otawww-json.py" ]]; then
			printf '%s\n' "$candidate_dir"
			return 0
		fi
	done

	printf 'phase19 helpers unavailable\n' >&2
	return 1
}

script_dir="$(resolve_phase19_script_dir)" || exit 1
readonly script_dir
phase19_json_helper="${script_dir}/phase19-recovery-otawww-json.py"
readonly phase19_json_helper
# shellcheck source=scripts/phase19-recovery-otawww-target.sh
source "${script_dir}/phase19-recovery-otawww-target.sh"
# shellcheck source=scripts/phase19-recovery-otawww-probe.sh
source "${script_dir}/phase19-recovery-otawww-probe.sh"

recovery_script="${PHASE16_RECOVERY_REGRESSION_SCRIPT:-${script_dir}/phase16-recovery-regression.sh}"
curl_bin="${CURL_BIN:-curl}"

manifest=""
factory_image=""
ota_image=""
port=""
out_dir=""
target_lock_out=""
device_url=""
device_url_source="none"
device_url_from_argument=0
flash_evidence_json=""
selected_port_from_flash=""
allow_failed_update=0
allow_large_erase=0
allow_interrupted_ota=0
otawww_gap_only=0

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
	--factory-image)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		factory_image="$2"
		shift 2
		;;
	--ota-image)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		ota_image="$2"
		shift 2
		;;
	--port)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		port="$2"
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
	--device-url)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		device_url="$2"
		device_url_source="argument"
		device_url_from_argument=1
		shift 2
		;;
	--device-url-from-flash-evidence)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		flash_evidence_json="$2"
		shift 2
		;;
	--target-lock-out)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		target_lock_out="$2"
		shift 2
		;;
	--allow-failed-update)
		allow_failed_update=1
		shift
		;;
	--allow-large-erase)
		allow_large_erase=1
		shift
		;;
	--allow-interrupted-ota)
		allow_interrupted_ota=1
		shift
		;;
	--otawww-gap-only)
		otawww_gap_only=1
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

if [[ -z "$manifest" || -z "$factory_image" || -z "$ota_image" || -z "$port" || -z "$out_dir" ]]; then
	usage
	exit 2
fi

if [[ "$device_url_from_argument" -eq 1 && -n "$flash_evidence_json" ]]; then
	printf 'use only one of --device-url or --device-url-from-flash-evidence\n' >&2
	exit 2
fi

if [[ -z "$target_lock_out" ]]; then
	target_lock_out="${out_dir%/}/target-lock.json"
fi

ensure_allowed_write_path() {
	local label="$1"
	local path="$2"

	case "$path" in
	docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence | \
		docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/* | \
		target/phase19-recovery-regression-and-otawww-evidence-dev-raw | \
		target/phase19-recovery-regression-and-otawww-evidence-dev-raw/*)
		return 0
		;;
	esac

	printf '%s must stay under Phase 19 evidence or target raw-evidence paths: %s\n' "$label" "$path" >&2
	exit 2
}

ensure_allowed_write_path "--out-dir" "$out_dir"
ensure_allowed_write_path "--target-lock-out" "$target_lock_out"

recovery_dir="${out_dir%/}/recovery-regression"
otawww_dir="${out_dir%/}/otawww"
ensure_allowed_write_path "recovery regression out-dir" "$recovery_dir"
ensure_allowed_write_path "OTAWWW out-dir" "$otawww_dir"
mkdir -p "$recovery_dir" "$otawww_dir"

log_file="${out_dir%/}/phase19-recovery-otawww-evidence.log"
otawww_gap_log="${otawww_dir}/otawww-gap.log"
: >"$log_file"

log_main() {
	printf '%s\n' "$*" >>"$log_file"
}

log_otawww() {
	printf '%s\n' "$*" >>"$otawww_gap_log"
}

if [[ -n "$flash_evidence_json" ]]; then
	if ! load_device_url_from_flash_evidence; then
		exit 2
	fi
fi

if [[ "$device_url_from_argument" -eq 1 ]] && ! validate_origin_device_url "$device_url"; then
	printf 'DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, fragment, or whitespace\n' >&2
	exit 2
fi

if [[ -n "$device_url" && "$device_url_from_argument" -eq 0 && -z "$flash_evidence_json" ]]; then
	if ! validate_origin_device_url "$device_url"; then
		printf 'DEVICE_URL must be an origin-only http:// or https:// URL without userinfo, path, query, fragment, or whitespace\n' >&2
		exit 2
	fi
fi

if [[ -z "$selected_port_from_flash" ]]; then
	selected_port_from_flash="$port"
fi

log_main "phase19_recovery_otawww_evidence"
log_main "manifest: ${manifest}"
log_main "factory_image: ${factory_image}"
log_main "ota_image: ${ota_image}"
log_main "port: ${port}"
log_main "out_dir: ${out_dir}"
log_main "target_lock_out: ${target_lock_out}"
log_main "network_scan: disabled"
log_main "raw_destructive_commands: prohibited"
log_main "raw_write_commands: prohibited"
log_main "interrupted_upload_commands: delegated to Phase 16 helper only"
log_main "rollback_commands: delegated to Phase 16 helper only"
log_main "recovery_helper: scripts/phase16-recovery-regression.sh"
log_main "DEVICE_URL source: ${device_url_source}"
log_main "DEVICE_URL sanitized: $(redacted_origin "$device_url")"
log_allow_flag_status "--allow-failed-update" "$allow_failed_update"
log_allow_flag_status "--allow-large-erase" "$allow_large_erase"
log_allow_flag_status "--allow-interrupted-ota" "$allow_interrupted_ota"
log_allow_flag_status "--otawww-gap-only" "$otawww_gap_only"

if [[ -n "$device_url" ]]; then
	write_target_lock "passed" "$selected_port_from_flash"
	log_main "target_status: passed"
else
	log_main "target_status: blocked - missing DEVICE_URL"
fi

run_phase16_recovery

if [[ -z "$device_url" ]]; then
	write_otawww_gap_without_target
	exit 0
fi

if [[ "$otawww_gap_only" -eq 1 ]]; then
	run_otawww_gap_probe
else
	log_main "otawww_status: not run - --otawww-gap-only not supplied"
	write_otawww_gap_without_target
fi
