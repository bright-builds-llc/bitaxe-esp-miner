#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/serial-session-trace.sh
source "${script_dir}/serial-session-trace.sh"

stage=""
port=""
expected_physical_identity=""
trace_root=""

while (($#)); do
	case "$1" in
	--stage)
		stage="${2:-}"
		shift 2
		;;
	--port)
		port="${2:-}"
		shift 2
		;;
	--expected-physical-identity)
		expected_physical_identity="${2:-}"
		shift 2
		;;
	--trace-root)
		trace_root="${2:-}"
		shift 2
		;;
	*)
		printf 'category=invalid_arguments\n'
		exit 2
		;;
	esac
done

[[ "$stage" =~ ^[a-z0-9-]+$ && -n "$port" && -n "$trace_root" ]] || {
	printf 'category=invalid_arguments\n'
	exit 2
}
[[ "$expected_physical_identity" =~ ^[0-9a-f]{64}$ ]] || {
	printf 'category=invalid_physical_identity\n'
	exit 2
}

SERIAL_SESSION_TRACE_ROOT="$trace_root"
export SERIAL_SESSION_TRACE_ROOT
serial_session_trace_init "phase35-${stage}" || {
	printf 'category=trace_initialization_failed\n'
	exit 1
}
if ! serial_session_readiness_gate "phase35-${stage}" "$port"; then
	printf 'category=%s\n' "${SERIAL_SESSION_READINESS_CATEGORY:-readiness_failed}"
	exit 1
fi
if [[ "$SERIAL_SESSION_READY_PHYSICAL_IDENTITY" != "$expected_physical_identity" ]]; then
	printf 'category=physical_identity_changed\n'
	exit 1
fi

printf 'category=ready\n'
printf 'combined_identity=%s\n' "$SERIAL_SESSION_READY_IDENTITY"
printf 'physical_identity=%s\n' "$SERIAL_SESSION_READY_PHYSICAL_IDENTITY"
printf 'enumeration_identity=%s\n' "$SERIAL_SESSION_READY_ENUMERATION_IDENTITY"
