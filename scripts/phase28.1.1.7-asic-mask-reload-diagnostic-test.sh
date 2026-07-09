#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
wrapper="${repo_root}/scripts/phase28.1.1.7-asic-mask-reload-diagnostic.sh"
temp_root="$(mktemp -d "${repo_root}/scratch/phase28.1.1.7-asic-mask-reload-test.XXXXXX")"
trap 'rm -rf "$temp_root"' EXIT

wifi_credentials="${temp_root}/wifi-credentials.json"
capture_dir="${temp_root}/capture"
stdout_log="${temp_root}/stdout.log"
stderr_log="${temp_root}/stderr.log"

printf '{"ssid":"PHASE28_SENTINEL_SSID","password":"PHASE28_SENTINEL_PASSWORD"}\n' >"$wifi_credentials"

bash -n "$wrapper"

# Duration floor must reject <360.
if bash "$wrapper" \
	--port /dev/cu.usbmodemPHASE28 \
	--wifi-credentials "$wifi_credentials" \
	--duration-seconds 359 \
	--capture-dir "$capture_dir" \
	--dry-run \
	>"$stdout_log" 2>"$stderr_log"; then
	printf 'asic_mask_reload_diagnostic_test_error: duration <360 must fail\n' >&2
	exit 1
fi
rg -q 'duration-seconds must be at least 360' "$stderr_log"

bash "$wrapper" \
	--port /dev/cu.usbmodemPHASE28 \
	--wifi-credentials "$wifi_credentials" \
	--duration-seconds 360 \
	--capture-dir "$capture_dir" \
	--dry-run \
	>"$stdout_log" 2>"$stderr_log"

combined="$(cat "$stdout_log" "$stderr_log")"
[[ "$combined" != *"PHASE28_SENTINEL_PASSWORD"* ]]
[[ "$combined" != *"PHASE28_SENTINEL_SSID"* ]]
[[ "$combined" != *"poolPassword"* ]]
[[ "$combined" != *"wifipass"* ]]
[[ "$combined" == *"dry_run_status=passed"* ]]
[[ "$combined" == *"credential_values_printed=false"* ]]
[[ "$combined" == *"forced_ab_label=pool_negotiated_mask_asic_reload"* ]]
[[ "$combined" == *"falsified_levers_recommended=false"* ]]
[[ "$combined" == *"job_byte_patch_applied=false"* ]]
[[ "$combined" == *"negotiated_version_mask_work_field_parity_reopened_as_sole_blocker=false"* ]]

# Forced A/B must be pool_negotiated_mask_asic_reload, not falsified prior labels.
rg -n "pool_negotiated_mask_asic_reload" "$wrapper" >/dev/null
rg -n "duration_seconds|360" "$wrapper" >/dev/null
if rg -n 'forced_ab_label="match_upstream_register_read_poll"|forced_ab_label=match_upstream_register_read_poll' "$wrapper"; then
	printf 'asic_mask_reload_diagnostic_test_error: forced A/B must not be match_upstream_register_read_poll\n' >&2
	exit 1
fi
if rg -n 'forced_ab_label="post_max_baud_delay_2000"|forced_ab_label=post_max_baud_delay_2000' "$wrapper"; then
	printf 'asic_mask_reload_diagnostic_test_error: forced A/B must not be post_max_baud_delay_2000\n' >&2
	exit 1
fi
if rg -n 'forced_ab_label="upstream_like_long_block_receive"|forced_ab_label=upstream_like_long_block_receive' "$wrapper"; then
	printf 'asic_mask_reload_diagnostic_test_error: forced A/B must not be upstream_like_long_block_receive\n' >&2
	exit 1
fi
if rg -n 'forced_ab_label="ticket_mask_asic_difficulty"|forced_ab_label=ticket_mask_asic_difficulty' "$wrapper"; then
	printf 'asic_mask_reload_diagnostic_test_error: forced A/B must not be ticket_mask_asic_difficulty\n' >&2
	exit 1
fi
if rg -n 'forced_ab_label="count_asic_chips_rx_loop_parity"|forced_ab_label=count_asic_chips_rx_loop_parity' "$wrapper"; then
	printf 'asic_mask_reload_diagnostic_test_error: forced A/B must not be count_asic_chips_rx_loop_parity\n' >&2
	exit 1
fi
if rg -n 'forced_ab_label="negotiated_version_mask_work_field_parity"|forced_ab_label=negotiated_version_mask_work_field_parity' "$wrapper"; then
	printf 'asic_mask_reload_diagnostic_test_error: forced A/B must not reopen negotiated_version_mask_work_field_parity\n' >&2
	exit 1
fi

printf 'phase28.1.1.7 asic-mask-reload diagnostic helper tests passed\n'
