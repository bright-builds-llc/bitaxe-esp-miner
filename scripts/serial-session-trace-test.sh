#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
# shellcheck source=scripts/serial-session-trace.sh
source "${script_dir}/serial-session-trace.sh"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/serial-session-trace-test.XXXXXX")"
readonly tmp_root
serial_port="${tmp_root}/serial-device"
readonly serial_port

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

mode_of() {
	local path="$1"
	if [[ "$(uname -s)" == "Darwin" ]]; then
		stat -f '%Lp' "$path"
	else
		stat -c '%a' "$path"
	fi
}

write_executable() {
	local path="$1"
	local body="$2"
	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_identity_stubs() {
	local bin_dir="$1"
	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/node-identity" 'printf "%s\n" "${SERIAL_TEST_NODE_IDENTITY:-node-stable}"'
	write_executable "${bin_dir}/usb-identity" 'printf "%s\n" "${SERIAL_TEST_USB_IDENTITY:-usb-stable}"'
	write_executable "${bin_dir}/lsof-empty" 'exit 1'
	write_executable "${bin_dir}/lsof-holder" 'printf "4242\n"'
	write_executable "${bin_dir}/lsof-owner" 'printf "%s\n" "${SERIAL_TEST_HOLDER_PID:?}"'
	write_executable "${bin_dir}/node-sequence" 'sequence_file="${SERIAL_TEST_IDENTITY_SEQUENCE_FILE:?}"
value="$(sed -n "1p" "$sequence_file")"
sed "1d" "$sequence_file" >"${sequence_file}.next"
mv "${sequence_file}.next" "$sequence_file"
printf "%s\n" "$value"'
}

assert_trace_schema() {
	jq -e -s 'length > 0 and all(.[]; (.event | type == "string") and (.utc_timestamp | type == "string") and (.monotonic_ms | type == "number"))' "$SERIAL_SESSION_TRACE_FILE" >/dev/null ||
		fail "trace JSONL schema is invalid"
}

test_trace_permissions_and_stable_readiness() {
	local bin_dir="${tmp_root}/stable-bin"
	local trace_root="${tmp_root}/stable-traces"
	create_identity_stubs "$bin_dir"

	SERIAL_SESSION_TRACE_ROOT="$trace_root" serial_session_trace_init stable
	SERIAL_SESSION_NODE_IDENTITY_BIN="${bin_dir}/node-identity" \
		SERIAL_SESSION_USB_IDENTITY_BIN="${bin_dir}/usb-identity" \
		SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-empty" \
		SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 \
		serial_session_readiness_gate pre_attach "$serial_port" || fail "stable readiness gate failed"

	[[ "$(mode_of "$trace_root")" == "700" ]] || fail "trace root is not mode 0700"
	[[ "$(mode_of "$SERIAL_SESSION_TRACE_DIR")" == "700" ]] || fail "trace run directory is not mode 0700"
	[[ "$(mode_of "$SERIAL_SESSION_TRACE_FILE")" == "600" ]] || fail "trace JSONL is not mode 0600"
	[[ "$SERIAL_SESSION_READINESS_CATEGORY" == "ready" ]] || fail "stable readiness category is not ready"
	[[ "$(jq -s '[.[] | select(.event == "readiness_snapshot")] | length' "$SERIAL_SESSION_TRACE_FILE")" -eq 3 ]] ||
		fail "stable readiness did not record three snapshots"
	assert_trace_schema
}

test_identity_change_fails_closed() {
	local bin_dir="${tmp_root}/identity-bin"
	local sequence_file="${tmp_root}/identity-sequence"
	create_identity_stubs "$bin_dir"
	printf 'node-a\nnode-b\nnode-b\n' >"$sequence_file"

	SERIAL_SESSION_TRACE_ROOT="${tmp_root}/identity-traces" serial_session_trace_init identity-change
	if SERIAL_TEST_IDENTITY_SEQUENCE_FILE="$sequence_file" \
		SERIAL_SESSION_NODE_IDENTITY_BIN="${bin_dir}/node-sequence" \
		SERIAL_SESSION_USB_IDENTITY_BIN="${bin_dir}/usb-identity" \
		SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-empty" \
		SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 \
		serial_session_readiness_gate pre_attach "$serial_port"; then
		fail "identity-changing readiness unexpectedly passed"
	fi
	[[ "$SERIAL_SESSION_READINESS_CATEGORY" == "identity_changed" ]] || fail "identity change was not classified"
}

test_holder_fails_closed() {
	local bin_dir="${tmp_root}/holder-bin"
	create_identity_stubs "$bin_dir"

	SERIAL_SESSION_TRACE_ROOT="${tmp_root}/holder-traces" serial_session_trace_init holder
	if SERIAL_SESSION_NODE_IDENTITY_BIN="${bin_dir}/node-identity" \
		SERIAL_SESSION_USB_IDENTITY_BIN="${bin_dir}/usb-identity" \
		SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-holder" \
		SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 \
		serial_session_readiness_gate pre_attach "$serial_port"; then
		fail "holder readiness unexpectedly passed"
	fi
	[[ "$SERIAL_SESSION_READINESS_CATEGORY" == "holders_present" ]] || fail "holder was not classified"
}

test_unavailable_ownership_probe_fails_closed() {
	local bin_dir="${tmp_root}/probe-bin"
	create_identity_stubs "$bin_dir"

	SERIAL_SESSION_TRACE_ROOT="${tmp_root}/probe-traces" serial_session_trace_init unavailable-probe
	if SERIAL_SESSION_NODE_IDENTITY_BIN="${bin_dir}/node-identity" \
		SERIAL_SESSION_USB_IDENTITY_BIN="${bin_dir}/usb-identity" \
		SERIAL_SESSION_LSOF_BIN="${tmp_root}/missing-lsof" \
		SERIAL_SESSION_READINESS_INTERVAL_SECONDS=0 \
		serial_session_readiness_gate pre_attach "$serial_port"; then
		fail "unavailable ownership probe unexpectedly passed"
	fi
	[[ "$SERIAL_SESSION_READINESS_CATEGORY" == "ownership_probe_unavailable" ]] || fail "unavailable ownership probe was not classified"
}

test_darwin_unmatched_ioreg_is_unavailable() {
	local bin_dir="${tmp_root}/darwin-bin"
	local output
	local status
	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/uname" 'printf "Darwin\n"'
	write_executable "${bin_dir}/ioreg" 'printf "+-o unrelated <class IOSerialBSDClient>\n  \"IODialinDevice\" = \"/dev/cu.other\"\n"'

	set +e
	output="$(PATH="${bin_dir}:$PATH" serial_session_usb_identity "$serial_port")"
	status=$?
	set -e
	[[ "$status" -ne 0 ]] || fail "unmatched Darwin ioreg block unexpectedly produced an identity"
	[[ -z "$output" ]] || fail "unmatched Darwin ioreg block produced a hash"
}

test_linux_missing_sysfs_identity_is_unavailable() {
	local bin_dir="${tmp_root}/linux-bin"
	local output
	local status
	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/uname" 'printf "Linux\n"'

	set +e
	output="$(PATH="${bin_dir}:$PATH" serial_session_usb_identity "$serial_port")"
	status=$?
	set -e
	[[ "$status" -ne 0 ]] || fail "missing Linux sysfs identity unexpectedly produced an identity"
	[[ -z "$output" ]] || fail "missing Linux sysfs identity produced a placeholder hash input"
}

test_active_owner_must_belong_to_expected_group() {
	local bin_dir="${tmp_root}/active-owner-bin"
	local expected_pgid
	create_identity_stubs "$bin_dir"
	expected_pgid="$(ps -o pgid= -p "$$" | tr -d ' ')"

	SERIAL_SESSION_TRACE_ROOT="${tmp_root}/active-owner-traces" serial_session_trace_init active-owner
	SERIAL_TEST_HOLDER_PID="$$" SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-owner" \
		SERIAL_SESSION_ACTIVE_OWNER_INTERVAL_SECONDS=0 \
		serial_session_active_owner_gate "$serial_port" "$expected_pgid" || fail "expected active owner was rejected"
	[[ "$SERIAL_SESSION_READINESS_CATEGORY" == "active_owner_verified" ]] || fail "active owner was not classified"

	if SERIAL_TEST_HOLDER_PID="$$" SERIAL_SESSION_LSOF_BIN="${bin_dir}/lsof-owner" \
		SERIAL_SESSION_ACTIVE_OWNER_INTERVAL_SECONDS=0 \
		serial_session_active_owner_gate "$serial_port" 999999; then
		fail "unexpected active holder was accepted"
	fi
	[[ "$SERIAL_SESSION_READINESS_CATEGORY" == "unexpected_active_holder" ]] || fail "unexpected active holder was not classified"
}

: >"$serial_port"
chmod 600 "$serial_port"

test_trace_permissions_and_stable_readiness
test_identity_change_fails_closed
test_holder_fails_closed
test_unavailable_ownership_probe_fails_closed
test_darwin_unmatched_ioreg_is_unavailable
test_linux_missing_sysfs_identity_is_unavailable
test_active_owner_must_belong_to_expected_group

printf 'serial_session_trace_test passed\n'
