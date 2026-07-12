#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/serial-session-trace.sh
source "${script_dir}/serial-session-trace.sh"

readonly espflash_bin="${ESPFLASH_BIN:-espflash}"
candidates=()
explicit_port=""

usage() {
	printf 'usage: %s [--port PATH | port=PATH]\n' "$0" >&2
	printf 'Detects one Ultra 205 using no-flash, non-destructive ESP USB checks with explicit reset policy.\n' >&2
}

while (($#)); do
	case "$1" in
	-h | --help)
		usage
		exit 0
		;;
	--port)
		[[ $# -ge 2 ]] || {
			usage
			exit 2
		}
		explicit_port="$2"
		shift 2
		;;
	port=*)
		explicit_port="${1#*=}"
		shift
		;;
	*)
		usage
		exit 2
		;;
	esac
done
[[ -z "$explicit_port" || "$explicit_port" == /* || "$explicit_port" =~ ^COM[0-9]+$ ]] || {
	printf 'failure_category=explicit_port_invalid\n' >&2
	exit 2
}

serial_session_trace_init detect-ultra205
tool_version="${SERIAL_SESSION_TOOL_VERSION:-}"
if [[ -z "$tool_version" ]]; then
	set +e
	tool_version="$("$espflash_bin" --version 2>&1 | head -1)"
	tool_version_status=$?
	set -e
	if ((tool_version_status != 0)) || [[ -z "$tool_version" ]]; then
		tool_version="unavailable"
	fi
fi
serial_session_trace_event "detector_start" "$(jq -cn \
	--arg tool "$espflash_bin" \
	--arg tool_version "$tool_version" \
	--argjson exact_port_mode "$([[ -n "$explicit_port" ]] && printf true || printf false)" \
	'{tool:$tool,tool_version:$tool_version,exact_port_mode:$exact_port_mode,list_ports_reset_policy:"none",board_info_reset_policy:{before:"usb-reset",after:"hard-reset"}}')"

is_likely_esp_port() {
	local port="$1"

	case "$port" in
	/dev/cu.usbmodem* | /dev/cu.usbserial* | /dev/ttyUSB* | /dev/ttyACM*)
		return 0
		;;
	COM*[!0-9]* | COM)
		return 1
		;;
	COM*)
		return 0
		;;
	*)
		return 1
		;;
	esac
}

add_candidate() {
	local candidate="$1"
	local existing

	if [[ "${#candidates[@]}" -gt 0 ]]; then
		for existing in "${candidates[@]}"; do
			if [[ "$existing" == "$candidate" ]]; then
				return
			fi
		done
	fi

	candidates+=("$candidate")
}

clean_token() {
	local token="$1"
	token="${token//$'\r'/}"
	token="${token//,/}"
	token="${token//;/}"
	token="${token//:/}"
	token="${token//(/}"
	token="${token//)/}"
	token="${token//[/}"
	token="${token//]/}"
	printf '%s\n' "$token"
}

if [[ -n "$explicit_port" ]]; then
	port="$explicit_port"
else
	ports_output="$("$espflash_bin" list-ports --name-only 2>&1)" || {
		status=$?
		serial_session_trace_event "detector_result" "$(jq -cn --arg category list_ports_failed --argjson exit_status "$status" '{status:"failed",category:$category,exit_status:$exit_status}')"
		printf 'error: failed to list ESP serial ports with `%s list-ports --name-only`\n' "$espflash_bin" >&2
		printf 'failure_category=list_ports_failed\n' >&2
		printf '%s\n' "$ports_output" >&2
		exit "$status"
	}

	while IFS= read -r line; do
		for raw_token in $line; do
			token="$(clean_token "$raw_token")"
			if is_likely_esp_port "$token"; then
				add_candidate "$token"
			fi
		done
	done <<<"$ports_output"

	case "${#candidates[@]}" in
	0)
		serial_session_trace_event "detector_result" '{"status":"failed","category":"missing_node"}'
		printf 'error: no likely Ultra 205 ESP USB serial port detected\n' >&2
		printf 'failure_category=missing_node\n' >&2
		printf 'connect exactly one Ultra 205 over USB, then rerun `just detect-ultra205`\n' >&2
		exit 1
		;;
	1)
		port="${candidates[0]}"
		;;
	*)
		serial_session_trace_event "detector_result" "$(jq -cn --argjson candidate_count "${#candidates[@]}" '{status:"failed",category:"ambiguous_ports",candidate_count:$candidate_count}')"
		printf 'error: multiple likely ESP USB serial ports detected; refusing autonomous hardware use\n' >&2
		printf 'failure_category=ambiguous_ports\n' >&2
		for port in "${candidates[@]}"; do
			printf -- '- %s\n' "$port" >&2
		done
		printf 'disconnect extra devices or pass an explicit `port=<path>` to the hardware command\n' >&2
		exit 1
		;;
	esac
fi

board_info_command=(
	"$espflash_bin"
	board-info
	--chip
	esp32s3
	--port
	"$port"
	--non-interactive
	--before
	usb-reset
	--after
	hard-reset
)

printf '[detect-ultra205] board_info_command=' >&2
printf '%q ' "${board_info_command[@]}" >&2
printf '\n' >&2

board_info_output="$("${board_info_command[@]}" 2>&1)" || {
	status=$?
	failure_category="board_info_failure"
	if grep -Eiq 'failed to open|permission denied|no such file|device[^[:alnum:]]+not found' <<<"$board_info_output"; then
		failure_category="open_failure"
	elif grep -Eiq 'connect|connecting|sync' <<<"$board_info_output"; then
		failure_category="connection_failure"
	fi
	serial_session_trace_event "detector_result" "$(jq -cn \
		--arg category "$failure_category" \
		--arg output_digest "$(printf '%s' "$board_info_output" | serial_session_hash_text)" \
		--argjson exit_status "$status" \
		'{status:"failed",category:$category,exit_status:$exit_status,output_digest:$output_digest}')"
	printf 'error: board-info failed for candidate Ultra 205 port %s\n' "$port" >&2
	printf 'failure_category=%s\n' "$failure_category" >&2
	printf '%s\n' "$board_info_output" >&2
	exit "$status"
}

serial_session_trace_event "detector_result" "$(jq -cn --arg port "$port" --arg output_digest "$(printf '%s' "$board_info_output" | serial_session_hash_text)" '{status:"passed",category:"detected",port:$port,output_digest:$output_digest}')"
printf '%s\n' "$board_info_output" >&2
printf 'port=%s\n' "$port"
