#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE14_DISPLAY_INPUT_SCRIPT:-${script_dir}/phase14-display-input.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase14-display-input-test.XXXXXX")"
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

	assert_contains "${out_dir}/display-input.log" "phase14_display_input_status: pending - missing manifest"
	assert_contains "${out_dir}/display-input.log" "runtime_display_input_status: pending - no runtime display/input route or physical input observation"
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

	assert_contains "${out_dir}/display-input.log" "safety_allow_status: failed"
	assert_contains "${out_dir}/display-input.log" "phase14_display_input_status: pending - allow validation failed"
}

test_display_markers_are_observed_without_promoting_runtime_input() {
	local out_dir="${tmp_root}/markers-observed"
	local manifest="${tmp_root}/manifest-display.json"
	local fake_allow="${tmp_root}/fake-allow-display"
	local serial_log="${tmp_root}/serial-display.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	{
		printf 'display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c\n'
		printf 'display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true\n'
	} >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/display-input.log" "display_status=startup_text_rendered model=SSD1306 size=128x32 address=0x3c"
	assert_contains "${out_dir}/display-input.log" "startup_display_status: observed"
	assert_contains "${out_dir}/display-input.log" "runtime_gap_marker_status: observed"
	assert_contains "${out_dir}/display-input.log" "runtime_display_input_status: pending - no runtime display/input route or physical input observation"
	assert_not_contains "${out_dir}/display-input.log" "runtime_display_input_status: passed"
}

test_missing_display_marker_stays_pending() {
	local out_dir="${tmp_root}/marker-missing"
	local manifest="${tmp_root}/manifest-missing.json"
	local fake_allow="${tmp_root}/fake-allow-missing"
	local serial_log="${tmp_root}/serial-missing.log"

	write_manifest "$manifest"
	write_fake_allow "$fake_allow"
	printf 'safe_state: mining=disabled hardware_control=disabled\n' >"$serial_log"

	PHASE14_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir" \
		--serial-log "$serial_log"

	assert_contains "${out_dir}/display-input.log" "startup_display_status: pending - startup marker missing"
	assert_contains "${out_dir}/display-input.log" "runtime_gap_marker_status: pending - runtime gap marker missing"
	assert_contains "${out_dir}/display-input.log" "phase14_display_input_status: pending - runtime display/input route unavailable"
}

test_missing_manifest_blocks
test_failed_validator_blocks
test_display_markers_are_observed_without_promoting_runtime_input
test_missing_display_marker_stays_pending

printf 'phase14-display-input tests passed\n'
