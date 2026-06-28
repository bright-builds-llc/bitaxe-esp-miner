#!/usr/bin/env bash
set -euo pipefail

readonly espflash_bin="${ESPFLASH_BIN:-espflash}"
candidates=()

usage() {
	printf 'usage: %s\n' "$0" >&2
	printf 'Detects a single connected Ultra 205 candidate using read-only ESP USB checks.\n' >&2
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
	usage
	exit 0
fi

if [[ "$#" -ne 0 ]]; then
	usage
	exit 2
fi

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

ports_output="$("$espflash_bin" list-ports --name-only 2>&1)" || {
	status=$?
	printf 'error: failed to list ESP serial ports with `%s list-ports --name-only`\n' "$espflash_bin" >&2
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
	printf 'error: no likely Ultra 205 ESP USB serial port detected\n' >&2
	printf 'connect exactly one Ultra 205 over USB, then rerun `just detect-ultra205`\n' >&2
	exit 1
	;;
1)
	port="${candidates[0]}"
	;;
*)
	printf 'error: multiple likely ESP USB serial ports detected; refusing autonomous hardware use\n' >&2
	for port in "${candidates[@]}"; do
		printf -- '- %s\n' "$port" >&2
	done
	printf 'disconnect extra devices or pass an explicit `port=<path>` to the hardware command\n' >&2
	exit 1
	;;
esac

board_info_command=(
	"$espflash_bin"
	board-info
	--chip
	esp32s3
	--port
	"$port"
	--non-interactive
)

printf '[detect-ultra205] board_info_command=' >&2
printf '%q ' "${board_info_command[@]}" >&2
printf '\n' >&2

board_info_output="$("${board_info_command[@]}" 2>&1)" || {
	status=$?
	printf 'error: board-info failed for candidate Ultra 205 port %s\n' "$port" >&2
	printf '%s\n' "$board_info_output" >&2
	exit "$status"
}

printf '%s\n' "$board_info_output" >&2
printf 'port=%s\n' "$port"
