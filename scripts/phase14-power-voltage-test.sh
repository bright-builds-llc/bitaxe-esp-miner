#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE14_POWER_VOLTAGE_SCRIPT:-${script_dir}/phase14-power-voltage.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase14-power-voltage-test.XXXXXX")"
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
	local status="$2"

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
		--surface power-telemetry \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/power-voltage.log" "phase14_power_voltage_status: pending - missing manifest"
	assert_contains "${out_dir}/power-voltage.log" "phase14_power_telemetry_status: pending - missing manifest"
}

test_failed_validator_blocks() {
	local out_dir="${tmp_root}/failed-validator"
	local manifest="${tmp_root}/manifest.json"
	local fake_allow="${tmp_root}/fake-allow"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow" failed

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" PHASE14_FAKE_ALLOW_STATUS=failed "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface power-telemetry \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/power-voltage.log" "safety_allow_status: failed"
	assert_contains "${out_dir}/power-voltage.log" "phase14_power_voltage_status: pending - allow validation failed"
}

test_power_telemetry_records_pending_hardware_evidence() {
	local out_dir="${tmp_root}/power"
	local manifest="${tmp_root}/manifest-power.json"
	local fake_allow="${tmp_root}/fake-allow-pass"
	local serial_log="${tmp_root}/serial-power.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow" passed
	printf 'safety_power_status=hardware_evidence_pending\n' >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface power-telemetry \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/power-voltage.log" "power_telemetry_claim: read-only-observation"
	assert_contains "${out_dir}/power-voltage.log" "power_telemetry_status: pending - hardware_evidence_pending"
	assert_contains "${out_dir}/power-voltage.log" "PWR-006 conclusion"
	assert_contains "${out_dir}/power-voltage.log" "safety-allow"
}

test_voltage_control_stays_pending() {
	local out_dir="${tmp_root}/voltage"
	local manifest="${tmp_root}/manifest-voltage.json"
	local fake_allow="${tmp_root}/fake-allow-voltage"
	local serial_log="${tmp_root}/serial-voltage.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow" passed
	printf 'safe_state: mining=disabled hardware_control=disabled\n' >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface voltage-control \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/power-voltage.log" "voltage_control_claim: unsupported-pending"
	assert_contains "${out_dir}/power-voltage.log" "voltage_control_status: pending - no production-safe bounded voltage route exists"
	assert_contains "${out_dir}/power-voltage.log" "PWR-003 conclusion"
	assert_contains "${out_dir}/power-voltage.log" "PWR-005 conclusion"
	assert_not_contains "${out_dir}/power-voltage.log" "voltage_control_status: passed"
}

test_missing_manifest_blocks
test_failed_validator_blocks
test_power_telemetry_records_pending_hardware_evidence
test_voltage_control_stays_pending

printf 'phase14-power-voltage tests passed\n'
