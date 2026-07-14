#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly wrapper="${PHASE33_WRAPPER:-${script_dir}/phase33-confirmed-settings-durability.sh}"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase33-confirmed-settings-test.XXXXXX")"
readonly tmp_root
trap 'rm -rf "$tmp_root"' EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

run_scenario() {
	local scenario="$1"
	local output="${tmp_root}/${scenario}.out"
	local summary="${tmp_root}/${scenario}.md"
	set +e
	PHASE33_ALLOW_TEST_MODE=1 PHASE33_SIMULATION_OUTPUT="$summary" \
		bash "$wrapper" --mode simulate --scenario "$scenario" >"$output" 2>&1
	local status=$?
	set -e
	printf '%s|%s|%s\n' "$status" "$output" "$summary"
}

success_result="$(run_scenario success)"
IFS='|' read -r success_status _success_output success_summary <<<"$success_result"
[[ "$success_status" == "0" ]] || fail "success simulation failed"
grep -Fq -- '- status: passed' "$success_summary" || fail "success status missing"
grep -Fq -- '- application_restart_count: 1' "$success_summary" || fail "restart count missing"

failure_cases='detector_ambiguous detector_ambiguous
board_info_failure board_info_failure
missing_flash required_package_missing
failed_flash required_package_flash_failed
zero_origin fresh_origin_missing
multiple_origin fresh_origin_ambiguous
identity_change physical_identity_changed
extra_reset application_restart_count_invalid
response_reversal response_before_effect_unproved
immediate_missing immediate_readback_missing
immediate_mismatch immediate_readback_mismatch
post_missing post_reboot_readback_missing
post_mismatch post_reboot_readback_mismatch
holder_leak serial_holder_cleanup_failed
process_leak monitor_process_cleanup_failed
timeout proof_timeout
restore_failure restoration_failed
sensitive_output sensitive_output_detected'

while read -r scenario expected_category; do
	result="$(run_scenario "$scenario")"
	IFS='|' read -r status output summary <<<"$result"
	[[ "$status" != "0" ]] || fail "${scenario} unexpectedly passed"
	grep -Fq "failure_category=${expected_category}" "$output" || fail "${scenario} category mismatch"
done <<<"$failure_cases"

source_text="$(cat "$wrapper")"
for forbidden_call in \
	'espflash erase' \
	'esptool.py' \
	'pool-credentials=' \
	'just phase28' \
	'just diagnose-ultra205-uart' \
	'just parity'; do
	if grep -Fq "$forbidden_call" <<<"$source_text"; then
		fail "forbidden command surface found: ${forbidden_call}"
	fi
done
grep -Fq -- '--before no-reset-no-sync --after no-reset --no-reset' "$wrapper" || fail "passive reset contract missing"
[[ "$(grep -Fc 'just detect-ultra205' "$wrapper")" == "1" ]] || fail "detector source count is not exactly one"
grep -Fq 'capture_seconds >= 360' "$wrapper" || fail "minimum capture gate missing"
[[ "$(grep -Fc 'just flash-monitor' "$wrapper")" == "1" ]] || fail "required package flash source count is not exactly one"
if grep -Eq -- '--skip-flash|retained-runtime' "$wrapper"; then
	fail "wrapper exposes an unauthorized skip-flash path"
fi

if rg -n -i 'https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|/dev/|ssid|password|worker|device_url|api[_ -]?(token|key)' "$success_summary"; then
	fail "success summary leaked a sensitive value"
fi
if ! rg -q 'https?://|password' "${tmp_root}/sensitive_output.md"; then
	fail "sensitive-output fixture did not exercise the denylist"
fi

printf 'phase33 confirmed settings durability tests passed\n'
