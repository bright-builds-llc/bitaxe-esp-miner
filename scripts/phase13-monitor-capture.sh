#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/process-group.sh
source "${script_dir}/process-group.sh"
# shellcheck source=scripts/serial-session-trace.sh
source "${script_dir}/serial-session-trace.sh"

usage() {
	printf 'usage: %s --port PATH --out LOG [--seconds N] [--no-reset] [--reader espflash|os-native] [--raw-out PATH]\n' "$(basename "$0")" >&2
}

port=""
out=""
seconds="35"
no_reset=0
reader="espflash"
raw_out=""
monitor_pid=""
monitor_group_ready_file=""
monitor_group_state_file="${PHASE13_MONITOR_GROUP_STATE_FILE:-}"
monitor_completion_file=""
monitor_active_ready_file="${PHASE13_MONITOR_ACTIVE_READY_FILE:-}"
passive_pre_ready=0
passive_post_ready=0
passive_identity=""
passive_active_owner_verified=0

record_trace_summary() {
	[[ "$no_reset" -eq 1 && -n "$SERIAL_SESSION_TRACE_FILE" ]] || return 0
	local digest="unavailable"
	local pre_readiness="unavailable"
	local post_readiness="unavailable"
	local active_owner_verified=false
	if ! digest="$(serial_session_trace_digest 2>/dev/null)"; then
		SERIAL_SESSION_TRACE_STATUS="incomplete"
		digest="unavailable"
	fi
	[[ "$passive_pre_ready" -eq 1 ]] && pre_readiness="ready"
	[[ "$passive_post_ready" -eq 1 ]] && post_readiness="ready"
	[[ "$passive_active_owner_verified" -eq 1 ]] && active_owner_verified=true
	log "serial_trace_status=${SERIAL_SESSION_TRACE_STATUS}"
	log "serial_trace_pre_readiness=${pre_readiness}"
	log "serial_trace_post_readiness=${post_readiness}"
	log "serial_trace_active_owner_verified=${active_owner_verified}"
	log "serial_trace_digest=${digest}"
}

signal_active_owner_ready() {
	[[ -n "$monitor_active_ready_file" ]] || return 0
	local temporary
	temporary="$(mktemp "$(dirname "$monitor_active_ready_file")/.phase13-active.XXXXXX")"
	serial_session_monotonic_ms >"$temporary"
	chmod 600 "$temporary"
	mv -f "$temporary" "$monitor_active_ready_file"
}

monitor_process_running() {
	local pid="$1"
	local state
	state="$(ps -o stat= -p "$pid" 2>/dev/null | tr -d ' ')"
	[[ -n "$state" && "$state" != Z* ]]
}

verify_post_readiness() {
	[[ "$no_reset" -eq 1 && "$passive_pre_ready" -eq 1 && "$passive_post_ready" -eq 0 ]] || return 0
	if ! serial_session_readiness_gate post_cleanup "$port" "$passive_identity"; then
		log "serial_session_failure_category=${SERIAL_SESSION_READINESS_CATEGORY}"
		return 1
	fi
	passive_post_ready=1
}

cleanup_monitor_group() {
	local pid="${monitor_pid:-$PHASE_PROCESS_GROUP_PID}"
	if [[ -z "$pid" ]]; then
		verify_post_readiness
		return
	fi

	if [[ "$no_reset" -eq 1 ]]; then
		serial_session_trace_event "cleanup_requested" "$(jq -cn --argjson process "$(serial_session_process_snapshot "$pid")" '{signal_escalation_policy:"term_then_kill_if_needed",process:$process}')" || return 1
	fi

	if ! phase_process_group_terminate "$pid" "phase13 monitor cleanup"; then
		if [[ "$no_reset" -eq 1 ]]; then
			serial_session_trace_event "cleanup_result" "$(jq -cn --argjson process "$(serial_session_process_snapshot "$pid")" '{status:"failed",process:$process}')"
		fi
		return 1
	fi
	if [[ "$no_reset" -eq 1 ]]; then
		serial_session_trace_event "cleanup_result" "$(jq -cn --argjson process "$(serial_session_process_snapshot "$pid")" '{status:"complete",process:$process}')" || return 1
	fi
	monitor_pid=""
	PHASE_PROCESS_GROUP_PID=""
	if [[ -n "$monitor_group_state_file" ]]; then
		: >"$monitor_group_state_file"
	fi
	if [[ -n "$monitor_group_ready_file" ]]; then
		rm -f "$monitor_group_ready_file"
	fi
	if [[ -n "$monitor_completion_file" ]]; then
		rm -f "$monitor_completion_file"
	fi
	verify_post_readiness
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
	--reader)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		reader="$2"
		shift 2
		;;
	--raw-out)
		if [[ $# -lt 2 ]]; then
			usage
			exit 2
		fi
		raw_out="$2"
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

if [[ -z "$port" || -z "$out" ]]; then
	usage
	exit 2
fi

if [[ ! "$seconds" =~ ^[0-9]+$ || "$seconds" -lt 1 ]]; then
	printf 'seconds must be a positive integer\n' >&2
	exit 2
fi
case "$reader" in
espflash | os-native) ;;
*)
	printf 'reader must be espflash or os-native\n' >&2
	exit 2
	;;
esac
if [[ "$reader" == "os-native" && "$no_reset" -ne 1 ]]; then
	printf 'os-native reader requires --no-reset\n' >&2
	exit 2
fi

mkdir -p "$(dirname "$out")"
: >"$out"
if [[ -n "$raw_out" ]]; then
	mkdir -p "$(dirname "$raw_out")"
	: >"$raw_out"
	chmod 600 "$raw_out"
fi

log() {
	printf '%s\n' "$*" >>"$out"
}

render_command() {
	if [[ "$reader" == "os-native" ]]; then
		printf 'phase13-os-native-reader.pl %s\n' "$port"
		return
	fi
	printf 'espflash monitor --chip esp32s3 --port %s --non-interactive' "$port"
	if [[ "$no_reset" -eq 1 ]]; then
		printf ' --before no-reset-no-sync --after no-reset --no-reset'
	fi
	printf '\n'
}

if [[ "$reader" == "os-native" ]]; then
	monitor_command=(perl "${script_dir}/phase13-os-native-reader.pl" "$port")
else
	monitor_command=(espflash monitor --chip esp32s3 --port "$port" --non-interactive)
	if [[ "$no_reset" -eq 1 ]]; then
		monitor_command+=(--before no-reset-no-sync --after no-reset --no-reset)
	fi
fi

log "phase13_monitor_capture"
log "port: ${port}"
log "seconds: ${seconds}"
log "reader: ${reader}"
log "monitor_command: $(render_command)"
log "serial_write: disabled"
log "raw_flash_write: disabled"
log "capture_output_start"

if [[ "$no_reset" -eq 1 ]]; then
	serial_session_trace_init phase13-passive-monitor
	tool_version="${SERIAL_SESSION_TOOL_VERSION:-}"
	if [[ -z "$tool_version" && "$reader" == "os-native" ]]; then
		tool_version="$(perl --version 2>&1 | sed -n '2p')"
	fi
	if [[ -z "$tool_version" ]]; then
		set +e
		tool_version="$(espflash --version 2>&1 | head -1)"
		tool_version_status=$?
		set -e
		if ((tool_version_status != 0)) || [[ -z "$tool_version" ]]; then
			tool_version="unavailable"
		fi
	fi
	serial_session_trace_event "session_start" "$(jq -cn \
		--arg tool_version "$tool_version" \
		--arg tool "$reader" \
		--arg command "$(render_command)" \
		'{tool:$tool,tool_version:$tool_version,command:$command,reset_policy:{before:"no-reset-no-sync",after:"no-reset",monitor_reset:false}}')"
	if ! serial_session_readiness_gate pre_attach "$port"; then
		log "serial_session_failure_category=${SERIAL_SESSION_READINESS_CATEGORY}"
		record_trace_summary
		exit 1
	fi
	passive_pre_ready=1
	passive_identity="$SERIAL_SESSION_READY_IDENTITY"
fi

set +e
monitor_group_ready_file="${TMPDIR:-/tmp}/phase13-monitor-group.$$.ready"
monitor_completion_file="$(mktemp "${TMPDIR:-/tmp}/phase13-monitor-completion.$$.XXXXXX")"
rm -f "$monitor_completion_file"
# shellcheck disable=SC2034 # Read by the sourced process-group helper.
PHASE_PROCESS_GROUP_STATE_FILE="$monitor_group_state_file"
# The generated Bash wrapper expands its own arguments and completion path.
# shellcheck disable=SC2016
monitor_wrapper='
completion_file="$1"
shift
set +e
"$@"
status=$?
set -e
temporary="${completion_file}.tmp.$$"
(umask 077 && printf "%s\n" "$status" >"$temporary")
mv -f "$temporary" "$completion_file"
exit "$status"
'
if [[ -n "$raw_out" ]]; then
	phase_process_group_start "$monitor_group_ready_file" "$BASH" -c "$monitor_wrapper" _ "$monitor_completion_file" "${monitor_command[@]}" >>"$raw_out" 2>>"$out"
else
	phase_process_group_start "$monitor_group_ready_file" "$BASH" -c "$monitor_wrapper" _ "$monitor_completion_file" "${monitor_command[@]}" >>"$out" 2>&1
fi
group_start_status=$?
set -e
((group_start_status == 0)) || {
	printf 'failed to start isolated monitor process group\n' >&2
	exit 1
}
monitor_pid="$PHASE_PROCESS_GROUP_PID"
if [[ "$no_reset" -eq 1 ]]; then
	serial_session_trace_event "monitor_started" "$(jq -cn --argjson process "$(serial_session_process_snapshot "$monitor_pid")" '{process:$process}')" || exit 1
	if ! serial_session_active_owner_gate "$port" "$monitor_pid"; then
		log "serial_session_failure_category=${SERIAL_SESSION_READINESS_CATEGORY}"
		cleanup_monitor_group || exit 1
		record_trace_summary
		exit 1
	fi
	passive_active_owner_verified=1
	signal_active_owner_ready
fi

start_epoch="$(date +%s)"
capture_status="failed"
monitor_status=0

while [[ ! -s "$monitor_completion_file" ]] && monitor_process_running "$monitor_pid"; do
	now_epoch="$(date +%s)"
	elapsed=$((now_epoch - start_epoch))
	if [[ "$elapsed" -ge "$seconds" ]]; then
		[[ ! -s "$monitor_completion_file" ]] || break
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
	if [[ -s "$monitor_completion_file" ]]; then
		completion_status="$(sed -n '1p' "$monitor_completion_file")"
		if [[ ! "$completion_status" =~ ^[0-9]+$ || "$completion_status" -ne "$monitor_status" ]]; then
			monitor_status=1
		fi
	fi
	if phase_process_group_is_alive "$monitor_pid"; then
		cleanup_monitor_group || exit 1
	else
		monitor_pid=""
		PHASE_PROCESS_GROUP_PID=""
		rm -f "$monitor_completion_file"
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

if ! verify_post_readiness; then
	capture_status="failed"
	monitor_status=1
fi

log "capture_output_end"
log "monitor_exit_status: ${monitor_status}"
log "capture_status=${capture_status}"
if [[ "$no_reset" -eq 1 ]]; then
	post_cleanup_ready=false
	[[ "$passive_post_ready" -eq 1 ]] && post_cleanup_ready=true
	serial_session_trace_event "session_end" "$(jq -cn \
		--arg capture_status "$capture_status" \
		--argjson monitor_exit_status "$monitor_status" \
		--argjson post_cleanup_ready "$post_cleanup_ready" \
		'{capture_status:$capture_status,monitor_exit_status:$monitor_exit_status,post_cleanup_ready:$post_cleanup_ready}')"
fi
record_trace_summary

if [[ "$capture_status" == "failed" ]]; then
	exit "$monitor_status"
fi
