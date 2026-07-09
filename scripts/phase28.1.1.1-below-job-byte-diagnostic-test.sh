#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
wrapper="${repo_root}/scripts/phase28.1.1.1-below-job-byte-diagnostic.sh"
temp_root="$(mktemp -d "${repo_root}/scratch/phase28-below-job-byte-test.XXXXXX")"
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
[[ "$combined" == *"dry_run_status=passed"* ]]
[[ "$combined" == *"credential_values_printed=false"* ]]

printf 'phase28.1.1.1 below-job-byte diagnostic helper tests passed\n'
