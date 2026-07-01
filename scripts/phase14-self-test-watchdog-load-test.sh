#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE14_SELF_TEST_WATCHDOG_LOAD_SCRIPT:-${script_dir}/phase14-self-test-watchdog-load.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase14-self-test-watchdog-load-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

write_fake_allow() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

status="${PHASE14_FAKE_ALLOW_STATUS:-passed}"
printf 'fake_allow_args: %s\n' "$*"
case "$status" in
passed)
	printf 'safety_allow_status: passed\n'
	;;
failed)
	printf 'safety_allow_status: failed\n'
	printf 'validation_errors:\n- fake failure\n'
	exit 42
	;;
*)
	printf 'unknown fake status\n' >&2
	exit 2
	;;
esac
SH
	chmod +x "$path"
}

write_manifest() {
	local path="$1"

	printf '{"board":"205"}\n' >"$path"
}

test_missing_manifest_blocks() {
	local out_dir="${tmp_root}/missing-manifest"

	"$BASH" "$wrapper" \
		--manifest "${tmp_root}/missing.json" \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/self-test-watchdog-load.log" "phase14_self_test_watchdog_load_status: pending - missing manifest"
	assert_contains "${out_dir}/self-test-watchdog-load.log" "load_stress_status: pending - bounded workload stimulus unavailable"
	assert_contains "${out_dir}/self-test-watchdog-load.log" "self_test_hardware_status: pending - no production-safe self-test hardware submode route exists"
}

test_failed_validator_blocks() {
	local out_dir="${tmp_root}/failed-validator"
	local manifest="${tmp_root}/manifest.json"
	local fake_allow="${tmp_root}/fake-allow"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" PHASE14_FAKE_ALLOW_STATUS=failed "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/self-test-watchdog-load.log" "safety_allow_status: failed"
	assert_contains "${out_dir}/self-test-watchdog-load.log" "phase14_self_test_watchdog_load_status: pending - allow validation failed"
}

test_watchdog_markers_are_observed_without_promoting_load_or_self_test() {
	local out_dir="${tmp_root}/markers-observed"
	local manifest="${tmp_root}/manifest-watchdog.json"
	local fake_allow="${tmp_root}/fake-allow-watchdog"
	local serial_log="${tmp_root}/serial-watchdog.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	{
		printf 'safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100\n'
		printf 'safety_supervisor_step=yield reason=yield_interval_reached\n'
	} >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/self-test-watchdog-load.log" "watchdog_supervisor_status: observed"
	assert_contains "${out_dir}/self-test-watchdog-load.log" "load_stress_status: pending - bounded workload stimulus unavailable"
	assert_contains "${out_dir}/self-test-watchdog-load.log" "self_test_hardware_status: pending - no production-safe self-test hardware submode route exists"
	assert_not_contains "${out_dir}/self-test-watchdog-load.log" "self_test_hardware_status: passed"
}

test_missing_watchdog_marker_stays_pending() {
	local out_dir="${tmp_root}/marker-missing"
	local manifest="${tmp_root}/manifest-missing.json"
	local fake_allow="${tmp_root}/fake-allow-missing"
	local serial_log="${tmp_root}/serial-missing.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	printf 'safety_supervisor=started thread=bitaxe-safety-supervisor cadence_ms=100\n' >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/self-test-watchdog-load.log" "watchdog_supervisor_status: pending - supervisor markers missing"
	assert_contains "${out_dir}/self-test-watchdog-load.log" "phase14_self_test_watchdog_load_status: pending - self-test hardware and bounded load routes unavailable"
}

test_missing_manifest_blocks
test_failed_validator_blocks
test_watchdog_markers_are_observed_without_promoting_load_or_self_test
test_missing_watchdog_marker_stays_pending

printf 'phase14-self-test-watchdog-load tests passed\n'
