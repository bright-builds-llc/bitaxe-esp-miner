#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
wrapper="${repo_root}/scripts/phase28.1.1.2-result-path-diagnostic.sh"
temp_root="$(mktemp -d "${repo_root}/scratch/phase28.1.1.2-result-path-test.XXXXXX")"
trap 'rm -rf "$temp_root"' EXIT

wifi_credentials="${temp_root}/wifi-credentials.json"
capture_dir="${temp_root}/capture"
stdout_log="${temp_root}/stdout.log"
stderr_log="${temp_root}/stderr.log"

printf '{"ssid":"PHASE28_SENTINEL_SSID","password":"PHASE28_SENTINEL_PASSWORD"}\n' >"$wifi_credentials"

bash -n "$wrapper"

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
[[ "$combined" == *"forced_ab_label=match_upstream_register_read_poll"* ]]

# Forced A/B must be register-read poll, not the falsified post_max_baud label.
rg -n "match_upstream_register_read_poll" "$wrapper" >/dev/null
if rg -n 'forced_ab_label="post_max_baud_delay_2000"|forced_ab_label=post_max_baud_delay_2000' "$wrapper"; then
	printf 'result_path_diagnostic_test_error: forced A/B must not be post_max_baud_delay_2000\n' >&2
	exit 1
fi

printf 'phase28.1.1.2 result-path diagnostic helper tests passed\n'
