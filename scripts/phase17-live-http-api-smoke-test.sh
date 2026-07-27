#!/usr/bin/env bash
set -euo pipefail

resolve_phase17_test_script_dir() {
	local direct_dir
	direct_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

	local candidate_dir
	for candidate_dir in \
		"$direct_dir" \
		"${BASH_SOURCE[0]}.runfiles/_main/scripts" \
		"${RUNFILES_DIR:-}/_main/scripts"; do
		if [[ -f "${candidate_dir}/phase17-live-http-api-smoke-test-fake-curl.sh" ]] &&
			[[ -f "${candidate_dir}/phase17-live-http-api-smoke-test-http.sh" ]] &&
			[[ -f "${candidate_dir}/phase17-live-http-api-smoke-test-websocket.sh" ]]; then
			printf '%s\n' "$candidate_dir"
			return 0
		fi
	done

	printf 'phase17 test helper unavailable: phase17-live-http-api-smoke-test-websocket.sh\n' >&2
	return 1
}

script_dir="$(resolve_phase17_test_script_dir)" || exit 1
readonly script_dir
# shellcheck source=scripts/phase17-live-http-api-smoke-test-fake-curl.sh
source "${script_dir}/phase17-live-http-api-smoke-test-fake-curl.sh"
# shellcheck source=scripts/phase17-live-http-api-smoke-test-http.sh
source "${script_dir}/phase17-live-http-api-smoke-test-http.sh"
# shellcheck source=scripts/phase17-live-http-api-smoke-test-websocket.sh
source "${script_dir}/phase17-live-http-api-smoke-test-websocket.sh"
readonly smoke_script="${PHASE17_LIVE_HTTP_API_SMOKE_SCRIPT:-${script_dir}/phase17-live-http-api-smoke.sh}"
readonly websocket_script="${PHASE17_WEBSOCKET_CAPTURE_SCRIPT:-${script_dir}/phase17-websocket-capture.mjs}"
readonly node_bin="${NODE_BIN:-node}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase17-live-http-api-smoke-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

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

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_manifest() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "source_commit": "26a1aebad7a11234567890123456789012345678",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_flash_json() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "command_kind": "flash-monitor",
  "board": "205",
  "selected_port": "/dev/cu.usbmodem1101",
  "trusted_output": true,
  "firmware_commit": "26a1aebad7a11234567890123456789012345678",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "observed_firmware_commit": "26a1aebad7a1",
  "observed_reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_stale_flash_json() {
	local path="$1"

	cat >"$path" <<'JSON'
{
  "command_kind": "flash-monitor",
  "board": "205",
  "selected_port": "/dev/cu.usbmodem1101",
  "trusted_output": true,
  "firmware_commit": "stale-source-commit",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "observed_firmware_commit": "stale-source",
  "observed_reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50"
}
JSON
}

create_flash_json_with_monitor_log() {
	local path="$1"
	local monitor_log_path="$2"
	local board="${3:-205}"
	local trusted_output="${4:-true}"
	local command_kind="${5:-flash-monitor}"
	local port_field="${6:-selected_port}"

	cat >"$path" <<JSON
{
  "command_kind": "${command_kind}",
  "board": "${board}",
  "${port_field}": "/dev/cu.usbmodem1101",
  "trusted_output": ${trusted_output},
  "firmware_commit": "26a1aebad7a11234567890123456789012345678",
  "reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "observed_firmware_commit": "26a1aebad7a1",
  "observed_reference_commit": "c1915b0a63bfabebdb95a515cedfee05146c1d50",
  "monitor_log_path": "${monitor_log_path}"
}
JSON
}

run_smoke() {
	local out_dir="$1"
	local manifest="$2"
	local flash_json="$3"
	local curl_bin="$4"
	shift 4

	"$BASH" "$smoke_script" \
		--manifest "$manifest" \
		--flash-evidence-json "$flash_json" \
		--out-dir "$out_dir" \
		--target-lock-out "${out_dir}/target-lock.json" \
		--curl-bin "$curl_bin" \
		"$@"
}

run_websocket_capture() {
	local out_file="$1"
	shift

	"$node_bin" "$websocket_script" --out "$out_file" "$@"
}

if [[ ! -f "$smoke_script" ]]; then
	fail "smoke script missing: ${smoke_script}"
fi
if [[ ! -f "$websocket_script" ]]; then
	fail "websocket script missing: ${websocket_script}"
fi
"$node_bin" --check "$websocket_script" >/dev/null

test_missing_target_blocks_without_curl
test_userinfo_path_query_fragment_rejected
test_stale_flash_identity_blocks_without_curl
test_fake_success_records_required_phase17_routes
test_flash_log_device_url_success_records_usb_source_without_raw_target_lock
test_flash_log_device_url_blocks_untrusted_or_unusable_sources
test_no_upgrade_does_not_claim_frames
test_redacts_response_secrets
test_websocket_missing_target_blocks_with_out
test_websocket_rejects_non_origin_target
test_websocket_rejects_unsupported_path
test_websocket_live_fake_frame_passes
test_websocket_flash_evidence_device_url_fake_frame_passes
test_websocket_frame_then_error_preserves_passed_frame_status
test_websocket_flash_evidence_device_url_blocks_unusable_sources
test_websocket_raw_log_open_timeout_stays_pending
test_websocket_rejects_bounds_over_limits

printf 'phase17_live_http_api_smoke_test passed\n'
