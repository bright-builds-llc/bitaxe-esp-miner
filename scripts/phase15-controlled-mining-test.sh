#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE15_CONTROLLED_MINING_SCRIPT:-${script_dir}/phase15-controlled-mining.sh}"
readonly websocket_helper="${PHASE15_WEBSOCKET_HELPER:-${script_dir}/phase15-websocket-capture.mjs}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase15-controlled-mining-test.XXXXXX")"
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

find_node_bin() {
	if [[ -n "${NODE_BIN:-}" && -x "${NODE_BIN}" ]]; then
		printf '%s' "$NODE_BIN"
		return
	fi

	if command -v node >/dev/null 2>&1; then
		command -v node
		return
	fi

	local candidate
	for candidate in \
		"${HOME}/.nvm/versions/node/v24.13.0/bin/node" \
		"/opt/homebrew/bin/node" \
		"/usr/local/bin/node"; do
		if [[ -x "$candidate" ]]; then
			printf '%s' "$candidate"
			return
		fi
	done

	printf 'node not found\n' >&2
	return 1
}

write_manifest() {
	local path="$1"

	printf '{"board":"205"}\n' >"$path"
}

write_fake_allow() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

status="${PHASE15_FAKE_ALLOW_STATUS:-passed}"
printf 'fake_mining_allow_args: %s\n' "$*"
case "$status" in
passed)
	printf 'mining_allow_status: passed\n'
	;;
failed)
	printf 'mining_allow_status: failed\n'
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

write_chip_detect_summary() {
	local path="$1"
	local conclusion="${2:-passed for package-backed chip-detect smoke}"

	cat >"$path" <<EOF
conclusion: ${conclusion}
safe_state: mining=disabled
work_submission=disabled
redaction_status: passed
restore_status: confirmed safe-state markers present
EOF
}

write_work_result_summary() {
	local path="$1"
	local conclusion="${2:-passed for diagnostic work dispatch with bounded no-result}"

	cat >"$path" <<EOF
conclusion: ${conclusion}
safe_state: mining=disabled
work_submission=disabled
fail_closed=true
redaction_status: passed
restore_status: confirmed safe-state markers present
EOF
}

write_fake_curl() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE15_CURL_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'curl should not have been called\n' >&2
	exit 99
fi

if [[ "${PHASE15_CURL_FAIL_EARLY:-0}" == "1" ]]; then
	exit 7
fi

printf '{}'
SH
	chmod +x "$path"
}

write_fake_websocket_helper() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE15_WEBSOCKET_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'websocket helper should not have been called\n' >&2
	exit 99
fi

out_file=""
while [[ $# -gt 0 ]]; do
	case "$1" in
	--out)
		out_file="$2"
		shift 2
		;;
	*)
		shift
		;;
	esac
done

if [[ -n "$out_file" ]]; then
	printf 'websocket_frame_status=passed\n' >"$out_file"
fi
SH
	chmod +x "$path"
}

run_wrapper() {
	local out_dir="$1"
	local manifest="$2"
	local chip_summary="$3"
	local work_summary="$4"
	local fake_allow="$5"
	local fake_curl="$6"
	local fake_ws="$7"
	shift 7

	PHASE15_MINING_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface mining-smoke \
		--out-dir "$out_dir" \
		--chip-detect-summary "$chip_summary" \
		--work-result-summary "$work_summary" \
		--curl-bin "$fake_curl" \
		--websocket-helper "$fake_ws" \
		"$@"
}

test_missing_manifest_records_pending() {
	local out_dir="${tmp_root}/missing-manifest"
	local chip_summary="${tmp_root}/chip-detect.md"
	local work_summary="${tmp_root}/work-result.md"
	local fake_allow="${tmp_root}/fake-allow"
	local fake_curl="${tmp_root}/fake-curl"
	local fake_ws="${tmp_root}/fake-ws"

	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	PHASE15_CURL_MUST_NOT_RUN=1 PHASE15_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper \
		"$out_dir" \
		"${tmp_root}/missing.json" \
		"$chip_summary" \
		"$work_summary" \
		"$fake_allow" \
		"$fake_curl" \
		"$fake_ws"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - missing manifest"
	assert_contains "${out_dir}/mining-smoke.log" "network_scan: disabled - DEVICE_URL must be explicit"
}

test_failed_allow_records_pending() {
	local out_dir="${tmp_root}/failed-allow"
	local manifest="${tmp_root}/manifest.json"
	local chip_summary="${tmp_root}/chip-detect.md"
	local work_summary="${tmp_root}/work-result.md"
	local fake_allow="${tmp_root}/fake-allow"
	local fake_curl="${tmp_root}/fake-curl"
	local fake_ws="${tmp_root}/fake-ws"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	PHASE15_FAKE_ALLOW_STATUS=failed PHASE15_CURL_MUST_NOT_RUN=1 PHASE15_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper \
		"$out_dir" \
		"$manifest" \
		"$chip_summary" \
		"$work_summary" \
		"$fake_allow" \
		"$fake_curl" \
		"$fake_ws"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - allow validation failed"
	assert_contains "${out_dir}/mining-smoke.log" "mining_allow_status: failed"
}

test_missing_live_prerequisites_records_controlled_no_share() {
	local out_dir="${tmp_root}/missing-live"
	local manifest="${tmp_root}/manifest-live.json"
	local chip_summary="${tmp_root}/chip-detect-live.md"
	local work_summary="${tmp_root}/work-result-live.md"
	local fake_allow="${tmp_root}/fake-allow-live"
	local fake_curl="${tmp_root}/fake-curl-live"
	local fake_ws="${tmp_root}/fake-ws-live"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	PHASE15_CURL_MUST_NOT_RUN=1 PHASE15_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper \
		"$out_dir" \
		"$manifest" \
		"$chip_summary" \
		"$work_summary" \
		"$fake_allow" \
		"$fake_curl" \
		"$fake_ws"

	assert_contains "${out_dir}/mining-smoke.log" "pool_category=controlled-no-share"
	assert_contains "${out_dir}/mining-smoke.log" "controlled_no_share_condition=missing_live_prerequisites"
	assert_contains "${out_dir}/mining-smoke.log" "share_outcome=controlled no-share condition"
	assert_contains "${out_dir}/mining-smoke.log" "api_telemetry_status=pending - missing DEVICE_URL"
	assert_contains "${out_dir}/mining-smoke.log" "websocket_frame_status=pending - missing DEVICE_URL or helper blocked"
}

test_environment_device_url_does_not_authorize_live_attempt() {
	local out_dir="${tmp_root}/env-device-url"
	local manifest="${tmp_root}/manifest-env-device-url.json"
	local chip_summary="${tmp_root}/chip-detect-env-device-url.md"
	local work_summary="${tmp_root}/work-result-env-device-url.md"
	local fake_allow="${tmp_root}/fake-allow-env-device-url"
	local fake_curl="${tmp_root}/fake-curl-env-device-url"
	local fake_ws="${tmp_root}/fake-ws-env-device-url"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	DEVICE_URL="https://10.0.0.2" \
		BITAXE_POOL_URL="stratum+tcp://pool.example.test:3333" \
		BITAXE_POOL_USER="worker" \
		BITAXE_POOL_PASSWORD="secret" \
		PHASE15_CURL_MUST_NOT_RUN=1 \
		PHASE15_WEBSOCKET_MUST_NOT_RUN=1 \
		run_wrapper \
		"$out_dir" \
		"$manifest" \
		"$chip_summary" \
		"$work_summary" \
		"$fake_allow" \
		"$fake_curl" \
		"$fake_ws"

	assert_contains "${out_dir}/mining-smoke.log" "pool_category=controlled-no-share"
	assert_contains "${out_dir}/mining-smoke.log" "api_telemetry_status=pending - missing DEVICE_URL"
	assert_not_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: live-prerequisites-present"
	assert_not_contains "${out_dir}/mining-smoke.log" "pool_category=live-pool-smoke"
}

test_api_probe_early_curl_failure_records_pending() {
	local out_dir="${tmp_root}/curl-fail-early"
	local manifest="${tmp_root}/manifest-curl-fail-early.json"
	local chip_summary="${tmp_root}/chip-detect-curl-fail-early.md"
	local work_summary="${tmp_root}/work-result-curl-fail-early.md"
	local fake_allow="${tmp_root}/fake-allow-curl-fail-early"
	local fake_curl="${tmp_root}/fake-curl-fail-early"
	local fake_ws="${tmp_root}/fake-ws-curl-fail-early"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	BITAXE_POOL_URL="stratum+tcp://pool.example.test:3333" \
		BITAXE_POOL_USER="worker" \
		BITAXE_POOL_PASSWORD="secret" \
		PHASE15_CURL_FAIL_EARLY=1 \
		run_wrapper \
		"$out_dir" \
		"$manifest" \
		"$chip_summary" \
		"$work_summary" \
		"$fake_allow" \
		"$fake_curl" \
		"$fake_ws" \
		--device-url "https://10.0.0.2"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: live-prerequisites-present"
	assert_contains "${out_dir}/mining-smoke.log" "api_telemetry_status=pending - curl failed"
	assert_contains "${out_dir}/mining-smoke.log" "api_telemetry_curl_status=7"
	assert_contains "${out_dir}/mining-smoke.log" "conclusion: pending - live mining evidence requires reviewed share/no-share and safe-stop artifacts"
}

test_pending_prerequisite_does_not_run_live_helpers() {
	local out_dir="${tmp_root}/pending-prereq"
	local manifest="${tmp_root}/manifest-pending.json"
	local chip_summary="${tmp_root}/chip-detect-pending.md"
	local work_summary="${tmp_root}/work-result-pending.md"
	local fake_allow="${tmp_root}/fake-allow-pending"
	local fake_curl="${tmp_root}/fake-curl-pending"
	local fake_ws="${tmp_root}/fake-ws-pending"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary" "pending - chip detect not run"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	PHASE15_CURL_MUST_NOT_RUN=1 PHASE15_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper \
		"$out_dir" \
		"$manifest" \
		"$chip_summary" \
		"$work_summary" \
		"$fake_allow" \
		"$fake_curl" \
		"$fake_ws"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - prerequisite not passed: chip-detect conclusion"
	assert_not_contains "${out_dir}/mining-smoke.log" "pool_category=live-pool"
}

test_bounded_soak_records_duration_and_abort_contract() {
	local out_dir="${tmp_root}/bounded-soak"
	local manifest="${tmp_root}/manifest-soak.json"
	local chip_summary="${tmp_root}/chip-detect-soak.md"
	local work_summary="${tmp_root}/work-result-soak.md"
	local fake_allow="${tmp_root}/fake-allow-soak"
	local fake_curl="${tmp_root}/fake-curl-soak"
	local fake_ws="${tmp_root}/fake-ws-soak"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_websocket_helper "$fake_ws"

	PHASE15_CURL_MUST_NOT_RUN=1 PHASE15_WEBSOCKET_MUST_NOT_RUN=1 PHASE15_MINING_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface bounded-soak \
		--duration-seconds 120 \
		--out-dir "$out_dir" \
		--chip-detect-summary "$chip_summary" \
		--work-result-summary "$work_summary" \
		--curl-bin "$fake_curl" \
		--websocket-helper "$fake_ws"

	assert_contains "${out_dir}/bounded-soak.log" "duration_seconds=120"
	assert_contains "${out_dir}/bounded-soak.log" "abort_condition=unsafe_temperature_or_power"
	assert_contains "${out_dir}/bounded-soak.log" "abort_condition=watchdog_unresponsive"
	assert_contains "${out_dir}/bounded-soak.log" "watchdog_status=pending - live prerequisites missing"
	assert_contains "${out_dir}/bounded-soak.log" "safe_stop_status=confirmed-or-pending"
}

test_websocket_helper_rejects_missing_and_redacts_url() {
	local missing_out="${tmp_root}/ws-missing.log"
	local env_only_out="${tmp_root}/ws-env-only.log"
	local invalid_out="${tmp_root}/ws-invalid.log"
	local redacted_out="${tmp_root}/ws-redacted.log"
	local node_bin
	node_bin="$(find_node_bin)"

	set +e
	"$node_bin" "$websocket_helper" --out "$missing_out" >/dev/null 2>&1
	local missing_status=$?
	DEVICE_URL="https://10.0.0.2/path?token=secret" "$node_bin" "$websocket_helper" --out "$env_only_out" >/dev/null 2>&1
	local env_only_status=$?
	"$node_bin" "$websocket_helper" --device-url "ftp://example.test" --out "$invalid_out" >/dev/null 2>&1
	local invalid_status=$?
	set -e

	if [[ "$missing_status" -eq 0 || "$env_only_status" -eq 0 || "$invalid_status" -eq 0 ]]; then
		printf 'websocket helper should reject missing, env-only, and non-http(s) DEVICE_URL\n' >&2
		exit 1
	fi

	"$node_bin" "$websocket_helper" \
		--device-url "https://10.0.0.2/path?token=secret" \
		--out "$redacted_out" \
		--duration-ms 1 \
		--max-frames 0

	assert_contains "$redacted_out" "wss://[redacted]/api/ws/live"
	assert_contains "$redacted_out" "websocket_frame_status=pending - max frames zero"
	assert_not_contains "$redacted_out" "10.0.0.2"
	assert_not_contains "$redacted_out" "secret"
}

test_missing_manifest_records_pending
test_failed_allow_records_pending
test_missing_live_prerequisites_records_controlled_no_share
test_environment_device_url_does_not_authorize_live_attempt
test_api_probe_early_curl_failure_records_pending
test_pending_prerequisite_does_not_run_live_helpers
test_bounded_soak_records_duration_and_abort_contract
test_websocket_helper_rejects_missing_and_redacts_url

printf 'phase15-controlled-mining tests passed\n'
