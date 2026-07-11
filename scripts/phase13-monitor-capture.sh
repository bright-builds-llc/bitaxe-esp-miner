#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/process-group.sh
source "${script_dir}/process-group.sh"

usage() {
	printf 'usage: %s --port PATH --out LOG [--seconds N] [--no-reset]\n' "$(basename "$0")" >&2
}

port=""
out=""
seconds="35"
no_reset=0
monitor_pid=""
monitor_group_ready_file=""
monitor_group_state_file="${PHASE13_MONITOR_GROUP_STATE_FILE:-}"

cleanup_monitor_group() {
	local pid="${monitor_pid:-$PHASE_PROCESS_GROUP_PID}"
	[[ -n "$pid" ]] || return 0

	if ! phase_process_group_terminate "$pid" "phase13 monitor cleanup"; then
		return 1
	fi
	monitor_pid=""
	PHASE_PROCESS_GROUP_PID=""
	if [[ -n "$monitor_group_state_file" ]]; then
		: >"$monitor_group_state_file"
	fi
	if [[ -n "$monitor_group_ready_file" ]]; then
		rm -f "$monitor_group_ready_file"
	fi
	return 0
}

handle_exit() {
	local status=$?
	trap - EXIT INT TERM
	if ! cleanup_monitor_group; then
		status=1
	fi
	exit "$status"
}

handle_signal() {
	trap - EXIT INT TERM
	if ! cleanup_monitor_group; then
		exit 1
	fi
	exit 130
}

trap handle_exit EXIT
trap handle_signal INT TERM

while [[ $# -gt 0 ]]; do
	case "$1" in
	--port)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		port="$2"
		shift 2
		;;
	--out)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		out="$2"
		shift 2
		;;
	--seconds)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		seconds="$2"
		shift 2
		;;
	--no-reset)
		no_reset=1
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

if [[ -z "$port" || -z "$out" ]]; then
	usage
	exit 2
fi

if [[ ! "$seconds" =~ ^[0-9]+$ || "$seconds" -lt 1 ]]; then
	printf 'seconds must be a positive integer\n' >&2
	exit 2
fi

mkdir -p "$(dirname "$out")"
: >"$out"

log() {
	printf '%s\n' "$*" >>"$out"
}

render_command() {
	printf 'espflash monitor --chip esp32s3 --port %s --non-interactive' "$port"
	if [[ "$no_reset" -eq 1 ]]; then
		printf ' --no-reset'
	fi
	printf '\n'
}

monitor_command=(espflash monitor --chip esp32s3 --port "$port" --non-interactive)
if [[ "$no_reset" -eq 1 ]]; then
	monitor_command+=(--no-reset)
fi

log "phase13_monitor_capture"
log "port: ${port}"
log "seconds: ${seconds}"
log "monitor_command: $(render_command)"
log "serial_write: disabled"
log "raw_flash_write: disabled"
log "capture_output_start"

set +e
monitor_group_ready_file="${TMPDIR:-/tmp}/phase13-monitor-group.$$.ready"
PHASE_PROCESS_GROUP_STATE_FILE="$monitor_group_state_file"
phase_process_group_start "$monitor_group_ready_file" "${monitor_command[@]}" >>"$out" 2>&1
group_start_status=$?
set -e
((group_start_status == 0)) || {
	printf 'failed to start isolated monitor process group\n' >&2
	exit 1
}
monitor_pid="$PHASE_PROCESS_GROUP_PID"

start_epoch="$(date +%s)"
capture_status="failed"
monitor_status=0

while kill -0 "$monitor_pid" >/dev/null 2>&1; do
	now_epoch="$(date +%s)"
	elapsed=$((now_epoch - start_epoch))
	if [[ "$elapsed" -ge "$seconds" ]]; then
		cleanup_monitor_group || exit 1
		monitor_status=143
		capture_status="timed_out_after_capture"
		break
	fi
	sleep 1
done

if [[ "$capture_status" != "timed_out_after_capture" ]]; then
	set +e
	wait "$monitor_pid"
	monitor_status=$?
	set -e
	if phase_process_group_is_alive "$monitor_pid"; then
		cleanup_monitor_group || exit 1
	else
		monitor_pid=""
		PHASE_PROCESS_GROUP_PID=""
		if [[ -n "$monitor_group_state_file" ]]; then
			: >"$monitor_group_state_file"
		fi
	fi
	if [[ "$monitor_status" -eq 0 ]]; then
		capture_status="completed"
	else
		capture_status="failed"
	fi
fi

log "capture_output_end"
log "monitor_exit_status: ${monitor_status}"
log "capture_status=${capture_status}"

if [[ "$capture_status" == "failed" ]]; then
	exit "$monitor_status"
fi
