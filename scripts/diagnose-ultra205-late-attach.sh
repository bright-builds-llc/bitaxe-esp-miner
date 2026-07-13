#!/usr/bin/env bash
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Resumable OS-native qualification of ESP32-S3 native-USB cold attachment.
set -euo pipefail
umask 077

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/ultra205-late-attach-broker.sh
source "$script_dir/ultra205-late-attach-broker.sh"

late_attach_install_traps
late_attach_validate_test_overrides

command="${1:-}"
[[ -n "$command" ]] || {
	late_attach_usage
	exit 2
}
shift

case "$command" in
begin) late_attach_begin "$@" ;;
status) late_attach_status "$@" ;;
deliver) late_attach_deliver "$@" ;;
-h | --help) late_attach_usage ;;
*)
	late_attach_usage
	exit 2
	;;
esac
