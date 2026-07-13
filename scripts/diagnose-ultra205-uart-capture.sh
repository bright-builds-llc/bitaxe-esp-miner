#!/usr/bin/env bash
# shellcheck source=scripts/phase28.1.1-terminal-closure-guard.sh
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Resumable receive-only external-UART cold-start qualification.
set -euo pipefail
umask 077

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/ultra205-uart-capture-broker.sh
source "$script_dir/ultra205-uart-capture-broker.sh"

late_attach_install_traps
late_attach_validate_test_overrides

command="${1:-}"
[[ -n "$command" ]] || {
	uart_capture_usage
	exit 2
}
shift

case "$command" in
begin) uart_capture_begin "$@" ;;
status) late_attach_status "$@" ;;
deliver) uart_capture_deliver "$@" ;;
-h | --help) uart_capture_usage ;;
*)
	uart_capture_usage
	exit 2
	;;
esac
