#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE14_THERMAL_FAN_SCRIPT:-${script_dir}/phase14-thermal-fan.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase14-thermal-fan-test.XXXXXX")"
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
		--surface thermal-fan \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/thermal-fan.log" "phase14_thermal_fan_status: pending - missing manifest"
	assert_contains "${out_dir}/thermal-fan.log" "fan_duty_status: pending - no production-safe bounded fan-duty route exists"
}

test_failed_validator_blocks() {
	local out_dir="${tmp_root}/failed-validator"
	local manifest="${tmp_root}/manifest.json"
	local fake_allow="${tmp_root}/fake-allow"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" PHASE14_FAKE_ALLOW_STATUS=failed "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface thermal-fan \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/thermal-fan.log" "safety_allow_status: failed"
	assert_contains "${out_dir}/thermal-fan.log" "phase14_thermal_fan_status: pending - allow validation failed"
}

test_thermal_wrapper_records_pending_hardware_evidence() {
	local out_dir="${tmp_root}/thermal"
	local manifest="${tmp_root}/manifest-thermal.json"
	local fake_allow="${tmp_root}/fake-allow-thermal"
	local serial_log="${tmp_root}/serial-thermal.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	printf 'safety_thermal_status=thermal_hardware_evidence_pending\n' >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface thermal-fan \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/thermal-fan.log" "thermal_claim: read-only-observation"
	assert_contains "${out_dir}/thermal-fan.log" "fan_rpm_claim: read-only-observation"
	assert_contains "${out_dir}/thermal-fan.log" "phase14_thermal_fan_status: pending - thermal_hardware_evidence_pending"
	assert_contains "${out_dir}/thermal-fan.log" "THR-001 conclusion"
	assert_contains "${out_dir}/thermal-fan.log" "THR-002 conclusion"
	assert_contains "${out_dir}/thermal-fan.log" "THR-003 conclusion"
}

test_fan_duty_stays_pending() {
	local out_dir="${tmp_root}/fan"
	local manifest="${tmp_root}/manifest-fan.json"
	local fake_allow="${tmp_root}/fake-allow-fan"
	local serial_log="${tmp_root}/serial-fan.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	printf 'safe_state: mining=disabled hardware_control=disabled\n' >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface thermal-fan \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/thermal-fan.log" "fan_duty_status: pending - no production-safe bounded fan-duty route exists"
	assert_not_contains "${out_dir}/thermal-fan.log" "fan_duty_status: passed"
}

test_missing_manifest_blocks
test_failed_validator_blocks
test_thermal_wrapper_records_pending_hardware_evidence
test_fan_duty_stays_pending

printf 'phase14-thermal-fan tests passed\n'
