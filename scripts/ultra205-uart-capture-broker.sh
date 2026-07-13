#!/usr/bin/env bash
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# External-UART specialization of the private late-attach attempt broker.

uart_capture_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
uart_capture_repo_root="$(cd "$uart_capture_script_dir/.." && pwd)"

export LATE_ATTACH_UART_MODE=1
export LATE_ATTACH_WORKER_BIN="${LATE_ATTACH_WORKER_BIN:-$uart_capture_script_dir/ultra205-uart-capture-worker.sh}"
export LATE_ATTACH_CONTROL_ROOT="${UART_CAPTURE_CONTROL_ROOT:-$uart_capture_repo_root/hardware-runs/phase28.1.1/uart-capture-control}"
export LATE_ATTACH_TRACE_ROOT="${UART_CAPTURE_TRACE_ROOT:-$uart_capture_repo_root/hardware-runs/phase28.1.1/uart-capture-private-traces}"

# shellcheck source=scripts/ultra205-late-attach-broker.sh
source "$uart_capture_script_dir/ultra205-late-attach-broker.sh"

uart_capture_usage() {
	printf 'usage: diagnose-ultra205-uart-capture.sh begin expected-firmware-head=SHA uart-port=PATH port=PATH [capture-seconds=N]\n' >&2
	printf '       diagnose-ultra205-uart-capture.sh status resume-handle=HEX\n' >&2
	printf '       diagnose-ultra205-uart-capture.sh deliver resume-handle=HEX checkpoint-token=TOKEN response-token=TOKEN\n' >&2
}

uart_capture_begin() {
	local uart_port=""
	local arg
	local -a forwarded=()
	for arg in "$@"; do
		case "$arg" in
		uart-port=*) uart_port="${arg#*=}" ;;
		*) forwarded+=("$arg") ;;
		esac
	done
	[[ -n "$uart_port" ]] || late_attach_die uart_port_missing
	export LATE_ATTACH_UART_PORT="$uart_port"
	late_attach_begin "${forwarded[@]}"
}

uart_capture_deliver() {
	local handle="" checkpoint="" response="" arg
	for arg in "$@"; do
		case "$arg" in
		resume-handle=*) handle="${arg#*=}" ;;
		checkpoint-token=*) checkpoint="${arg#*=}" ;;
		response-token=*) response="${arg#*=}" ;;
		*) late_attach_die unknown_argument ;;
		esac
	done
	[[ "$checkpoint" == uart-capture-removal-watcher-armed-v1 ]] || late_attach_die checkpoint_token_mismatch
	[[ "$response" == uart-capture-both-board-power-paths-removed-v1 ]] || late_attach_die checkpoint_token_mismatch
	late_attach_deliver \
		"resume-handle=$handle" \
		checkpoint-token=late-attach-removal-watcher-armed-v2 \
		response-token=late-attach-both-power-paths-removed-v2
}
