#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly gate="${script_dir}/phase35-stage-readiness.sh"
readonly test_root="${TEST_TMPDIR:-$(mktemp -d)}/phase35-stage-readiness"
readonly physical="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
readonly enumeration="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"

mkdir -p "$test_root/bin"
port="$test_root/device"
: >"$port"
chmod 600 "$port"

write_executable() {
	local path="$1"
	shift
	printf '%s\n' "$@" >"$path"
	chmod 700 "$path"
}

write_executable "$test_root/bin/physical" \
	'#!/usr/bin/env bash' \
	"printf '%s\\n' '$physical'"
write_executable "$test_root/bin/enumeration" \
	'#!/usr/bin/env bash' \
	"printf '%s\\n' '$enumeration'"
write_executable "$test_root/bin/no-holders" \
	'#!/usr/bin/env bash' \
	'exit 1'
write_executable "$test_root/bin/monotonic" \
	'#!/usr/bin/env bash' \
	'printf "%s\\n" "${RANDOM}000"'

run_gate() {
	local output="$1"
	shift
	SERIAL_SESSION_USB_PHYSICAL_IDENTITY_BIN="$test_root/bin/physical" \
		SERIAL_SESSION_USB_ENUMERATION_IDENTITY_BIN="$test_root/bin/enumeration" \
		SERIAL_SESSION_LSOF_BIN="$test_root/bin/no-holders" \
		SERIAL_SESSION_MONOTONIC_MS_BIN="$test_root/bin/monotonic" \
		SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 \
		"$gate" \
		--stage after-probe \
		--port "$port" \
		--expected-physical-identity "$physical" \
		--trace-root "$test_root/traces" \
		"$@" >"$output" 2>&1
}

success_output="$test_root/success.out"
run_gate "$success_output"
grep -Fqx 'category=ready' "$success_output"
grep -Fqx "physical_identity=${physical}" "$success_output"
grep -Fqx "enumeration_identity=${enumeration}" "$success_output"
[[ "$(find "$test_root/traces" -name session.jsonl -type f | wc -l | tr -d ' ')" == 1 ]]
trace_file="$(find "$test_root/traces" -name session.jsonl -type f)"
trace_mode="$(stat -f '%Lp' "$trace_file" 2>/dev/null || stat -c '%a' "$trace_file")"
[[ "$trace_mode" == 600 ]]

missing_output="$test_root/missing.out"
rm "$port"
if run_gate "$missing_output"; then
	printf 'missing device passed readiness\n' >&2
	exit 1
fi
grep -Fqx 'category=missing_node' "$missing_output"
if grep -Fq "$port" "$missing_output"; then
	printf 'protected port escaped readiness output\n' >&2
	exit 1
fi

: >"$port"
chmod 600 "$port"
write_executable "$test_root/bin/physical" \
	'#!/usr/bin/env bash' \
	"printf '%s\\n' 'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'"
drift_output="$test_root/drift.out"
if run_gate "$drift_output"; then
	printf 'physical identity drift passed readiness\n' >&2
	exit 1
fi
grep -Fqx 'category=physical_identity_changed' "$drift_output"

printf 'phase35 stage readiness tests passed\n'
