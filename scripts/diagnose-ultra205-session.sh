#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly detector_bin="${DETECT_ULTRA205_BIN:-${script_dir}/detect-ultra205.sh}"
readonly monitor_bin="${PHASE13_MONITOR_CAPTURE_SCRIPT:-${script_dir}/phase13-monitor-capture.sh}"

cycles=5
capture_seconds=30
requested_port=""
completed_cycles=0
trace_complete_count=0
trace_digest_material=""
baseline_status="not_run"
final_status="not_run"
diagnostic_status="failed"
failure_category="unavailable"

usage() {
	printf 'usage: %s [--cycles N] [--capture-seconds N] [--port PATH]\n' "$(basename "$0")" >&2
}

while (($# > 0)); do
	case "$1" in
	cycles=*)
		cycles="${1#cycles=}"
		shift
		;;
	capture-seconds=*)
		capture_seconds="${1#capture-seconds=}"
		shift
		;;
	port=*)
		requested_port="${1#port=}"
		shift
		;;
	--cycles)
		(($# >= 2)) || {
			usage
			exit 2
		}
		cycles="$2"
		shift 2
		;;
	--capture-seconds)
		(($# >= 2)) || {
			usage
			exit 2
		}
		capture_seconds="$2"
		shift 2
		;;
	--port)
		(($# >= 2)) || {
			usage
			exit 2
		}
		requested_port="$2"
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

if [[ ! "$cycles" =~ ^[0-9]+$ ]] || ((cycles < 1 || cycles > 20)); then
	printf 'cycles must be an integer from 1 through 20\n' >&2
	exit 2
fi
if [[ ! "$capture_seconds" =~ ^[0-9]+$ ]] || ((capture_seconds < 1 || capture_seconds > 300)); then
	printf 'capture-seconds must be an integer from 1 through 300\n' >&2
	exit 2
fi
if [[ -n "$requested_port" && "$requested_port" != /* && ! "$requested_port" =~ ^COM[0-9]+$ ]]; then
	printf 'port must be an absolute device path or COM port\n' >&2
	exit 2
fi

trace_root="${ULTRA205_SESSION_DIAGNOSTIC_ROOT:-scratch/ultra205-session-diagnostics}"
umask 077
mkdir -p "$trace_root"
chmod 700 "$trace_root"
run_dir="${trace_root}/$(date -u '+%Y%m%dT%H%M%SZ')-$$"
mkdir "$run_dir"
chmod 700 "$run_dir"
summary_path="${run_dir}/summary.json"

write_summary() {
	local trace_set_digest_sha256="unavailable"
	if [[ -n "$trace_digest_material" ]]; then
		trace_set_digest_sha256="$(printf '%s' "$trace_digest_material" | shasum -a 256 | awk '{print $1}')"
	fi
	jq -n \
		--arg status "$diagnostic_status" \
		--arg failure_category "$failure_category" \
		--arg baseline_detector "$baseline_status" \
		--arg final_detector "$final_status" \
		--arg trace_set_digest_sha256 "$trace_set_digest_sha256" \
		--argjson cycles_requested "$cycles" \
		--argjson cycles_completed "$completed_cycles" \
		--argjson capture_seconds "$capture_seconds" \
		--argjson trace_complete_count "$trace_complete_count" \
		'{schema_version:1,status:$status,failure_category:$failure_category,cycles_requested:$cycles_requested,cycles_completed:$cycles_completed,capture_seconds:$capture_seconds,baseline_detector:$baseline_detector,final_detector:$final_detector,trace_complete_count:$trace_complete_count,trace_set_digest_sha256:$trace_set_digest_sha256,physical_intervention_requested:false,operations:{flash:false,erase:false,factory_reset:false,credential_read:false,network_discovery:false,raw_write:false}}' \
		>"$summary_path"
	chmod 600 "$summary_path"
	jq -r 'to_entries[] | "\(.key)=\(.value)"' "$summary_path"
}

failure_from_log() {
	local path="$1"
	local category
	category="$(sed -n 's/^failure_category=//p; s/^serial_session_failure_category=//p' "$path" | tail -1)"
	printf '%s\n' "${category:-unclassified_failure}"
}

baseline_log="${run_dir}/detector-baseline.log"
set +e
SERIAL_SESSION_TRACE_ROOT="${run_dir}/detector-traces" "$detector_bin" >"$baseline_log" 2>&1
baseline_exit=$?
set -e
chmod 600 "$baseline_log"
if ((baseline_exit != 0)); then
	baseline_status="failed"
	failure_category="baseline_$(failure_from_log "$baseline_log")"
	write_summary
	exit 1
fi
baseline_status="passed"

mapfile_supported=1
if ! command -v mapfile >/dev/null 2>&1; then
	mapfile_supported=0
fi
detected_ports=()
if ((mapfile_supported == 1)); then
	mapfile -t detected_ports < <(sed -n 's/^port=//p' "$baseline_log")
else
	while IFS= read -r detected_port; do
		detected_ports+=("$detected_port")
	done < <(sed -n 's/^port=//p' "$baseline_log")
fi
if ((${#detected_ports[@]} != 1)); then
	failure_category="baseline_port_contract"
	write_summary
	exit 1
fi
port="${detected_ports[0]}"
if [[ -n "$requested_port" && "$requested_port" != "$port" ]]; then
	failure_category="requested_port_mismatch"
	write_summary
	exit 1
fi

for ((cycle = 1; cycle <= cycles; cycle++)); do
	monitor_log="${run_dir}/cycle-${cycle}-monitor.log"
	set +e
	SERIAL_SESSION_TRACE_ROOT="${run_dir}/serial-traces" \
		"$monitor_bin" --port "$port" --out "$monitor_log" --seconds "$capture_seconds" --no-reset
	monitor_exit=$?
	set -e
	[[ -f "$monitor_log" ]] && chmod 600 "$monitor_log"
	if ((monitor_exit != 0)); then
		failure_category="cycle_${cycle}_$(failure_from_log "$monitor_log")"
		write_summary
		exit 1
	fi
	if ! grep -Fxq 'serial_trace_status=complete' "$monitor_log" ||
		! grep -Fxq 'serial_trace_pre_readiness=ready' "$monitor_log" ||
		! grep -Fxq 'serial_trace_post_readiness=ready' "$monitor_log" ||
		! grep -Fxq 'serial_trace_active_owner_verified=true' "$monitor_log" ||
		! grep -Eq '^serial_trace_digest=[0-9a-f]{64}$' "$monitor_log"; then
		failure_category="cycle_${cycle}_trace_incomplete"
		write_summary
		exit 1
	fi
	cycle_trace_digest="$(sed -n 's/^serial_trace_digest=//p' "$monitor_log")"
	trace_digest_material+="cycle=${cycle}:${cycle_trace_digest}"$'\n'
	completed_cycles=$cycle
	trace_complete_count=$((trace_complete_count + 1))
done

final_log="${run_dir}/detector-final.log"
set +e
SERIAL_SESSION_TRACE_ROOT="${run_dir}/detector-traces" "$detector_bin" >"$final_log" 2>&1
final_exit=$?
set -e
chmod 600 "$final_log"
if ((final_exit != 0)); then
	final_status="failed"
	failure_category="final_$(failure_from_log "$final_log")"
	write_summary
	exit 1
fi
final_status="passed"
final_port="$(sed -n 's/^port=//p' "$final_log")"
if [[ "$final_port" != "$port" ]]; then
	failure_category="final_port_identity_changed"
	write_summary
	exit 1
fi

diagnostic_status="passed"
failure_category="none"
write_summary
