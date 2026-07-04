#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE21_LIVE_MINING_EVIDENCE_SCRIPT:-${script_dir}/phase21-live-mining-evidence.sh}"
readonly websocket_helper="${PHASE21_WEBSOCKET_HELPER:-${script_dir}/phase17-websocket-capture.mjs}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase21-live-mining-evidence-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

write_manifest() {
	local path="$1"

	printf '{"board":"205"}\n' >"$path"
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

write_readiness_audit() {
	local path="$1"

	cat >"$path" <<'EOF'
firmware_live_mining_status: blocked_by_default
observed_marker: mining_loop_status=blocked reason=hardware_evidence_ack_missing
controlled_enablement_required: true
network_scan: disabled
EOF
}

write_enablement_summary() {
	local path="$1"
	local package_status="${2:-ready}"
	local runtime_status="${3:-ready}"

	cat >"$path" <<EOF
controlled_live_mining_package_status: ${package_status}
controlled_runtime_harness_status: ${runtime_status}
safe_stop_status=confirmed-or-pending
redaction_status: passed
EOF
}

write_fake_allow() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE21_ALLOW_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'mining allow should not have been called\n' >&2
	exit 99
fi

printf 'fake_mining_allow_args: %s\n' "$*"
printf 'mining_allow_status: passed\n'
SH
	chmod +x "$path"
}

write_fake_curl() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE21_CURL_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'curl should not have been called\n' >&2
	exit 99
fi

if [[ -n "${PHASE21_FAKE_CURL_ARGS:-}" ]]; then
	printf '%s\n' "$*" >"$PHASE21_FAKE_CURL_ARGS"
fi

out_file=""
while [[ $# -gt 0 ]]; do
	case "$1" in
	--output)
		out_file="$2"
		shift 2
		;;
	*)
		shift
		;;
	esac
done

if [[ -n "$out_file" ]]; then
	printf '{"poolUrl":"stratum+tcp://private.example:3333","ip":"10.0.0.2","worker":"private-worker"}\n' >"$out_file"
fi
printf '200'
SH
	chmod +x "$path"
}

write_fake_node() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE21_WEBSOCKET_MUST_NOT_RUN:-0}" == "1" ]]; then
	printf 'websocket helper should not have been called\n' >&2
	exit 99
fi

if [[ -n "${PHASE21_FAKE_NODE_ARGS:-}" ]]; then
	printf '%s\n' "$*" >"$PHASE21_FAKE_NODE_ARGS"
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
	printf 'websocket_frame_status=passed frames=1\nwebsocket_capture_url=wss://[redacted]/api/ws/live\n' >"$out_file"
fi
SH
	chmod +x "$path"
}

base_fixture_paths() {
	local prefix="$1"

	manifest="${tmp_root}/${prefix}-manifest.json"
	chip_summary="${tmp_root}/${prefix}-chip.md"
	work_summary="${tmp_root}/${prefix}-work.md"
	readiness_audit="${tmp_root}/${prefix}-readiness.md"
	enablement_summary="${tmp_root}/${prefix}-enablement.md"
	fake_allow="${tmp_root}/${prefix}-fake-allow"
	fake_curl="${tmp_root}/${prefix}-fake-curl"
	fake_node="${tmp_root}/${prefix}-fake-node"

	write_manifest "$manifest"
	write_chip_detect_summary "$chip_summary"
	write_work_result_summary "$work_summary"
	write_readiness_audit "$readiness_audit"
	write_enablement_summary "$enablement_summary"
	write_fake_allow "$fake_allow"
	write_fake_curl "$fake_curl"
	write_fake_node "$fake_node"
}

run_wrapper() {
	local out_dir="$1"
	local surface="$2"
	shift 2

	PHASE21_MINING_ALLOW_BIN="$fake_allow" NODE_BIN="$fake_node" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--surface "$surface" \
		--out-dir "$out_dir" \
		--chip-detect-summary "$chip_summary" \
		--work-result-summary "$work_summary" \
		--readiness-audit "$readiness_audit" \
		--enablement-summary "$enablement_summary" \
		--curl-bin "$fake_curl" \
		--websocket-helper "$websocket_helper" \
		"$@"
}

test_missing_manifest_records_pending() {
	base_fixture_paths "missing-manifest"
	local out_dir="${tmp_root}/missing-manifest-out"
	manifest="${tmp_root}/missing-manifest-absent.json"

	PHASE21_ALLOW_MUST_NOT_RUN=1 PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "mining-smoke"

	assert_contains "${out_dir}/mining-smoke.log" "phase21_live_mining_evidence"
	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - missing manifest"
	assert_contains "${out_dir}/mining-smoke.log" "hardware_command_status=not-run"
	assert_contains "${out_dir}/mining-smoke.log" "network_scan: disabled - DEVICE_URL must be explicit"
}

test_missing_prerequisite_records_pending() {
	base_fixture_paths "missing-prereq"
	local out_dir="${tmp_root}/missing-prereq-out"
	readiness_audit="${tmp_root}/missing-readiness.md"

	PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "mining-smoke"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - prerequisite not passed: readiness audit missing"
	assert_contains "${out_dir}/mining-smoke.log" "hardware_command_status=not-run"
	assert_not_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: live-prerequisites-present"

	base_fixture_paths "missing-chip"
	out_dir="${tmp_root}/missing-chip-out"
	chip_summary="${tmp_root}/missing-chip.md"

	PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "mining-smoke"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - prerequisite not passed: chip-detect summary missing"
	assert_contains "${out_dir}/mining-smoke.log" "hardware_command_status=not-run"

	base_fixture_paths "missing-work"
	out_dir="${tmp_root}/missing-work-out"
	work_summary="${tmp_root}/missing-work.md"

	PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "mining-smoke"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - prerequisite not passed: work-result summary missing"
	assert_contains "${out_dir}/mining-smoke.log" "hardware_command_status=not-run"

	base_fixture_paths "missing-enable"
	out_dir="${tmp_root}/missing-enable-out"
	enablement_summary="${tmp_root}/missing-enable.md"

	PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "mining-smoke"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: pending - prerequisite not passed: enablement summary missing"
	assert_contains "${out_dir}/mining-smoke.log" "hardware_command_status=not-run"
}

test_missing_live_prerequisites_records_blocked() {
	base_fixture_paths "missing-live"
	local out_dir="${tmp_root}/missing-live-out"

	PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "mining-smoke"

	assert_contains "${out_dir}/mining-smoke.log" "live_mining_smoke_status: blocked"
	assert_contains "${out_dir}/mining-smoke.log" "blocker: missing_live_prerequisites"
	assert_contains "${out_dir}/mining-smoke.log" "controlled_no_share_condition=not-applicable"
	assert_contains "${out_dir}/mining-smoke.log" "network_scan: disabled - DEVICE_URL must be explicit"
	assert_contains "${out_dir}/mining-smoke.log" "safe_stop_status=confirmed-or-pending"
	assert_not_contains "${out_dir}/mining-smoke.log" "private.example"
	assert_not_contains "${out_dir}/mining-smoke.log" "private-worker"
}

test_bounded_soak_records_default_duration_and_watchdog_boundary() {
	base_fixture_paths "bounded"
	local out_dir="${tmp_root}/bounded-out"

	PHASE21_CURL_MUST_NOT_RUN=1 PHASE21_WEBSOCKET_MUST_NOT_RUN=1 run_wrapper "$out_dir" "bounded-soak"

	assert_contains "${out_dir}/bounded-soak.log" "duration_seconds=300"
	assert_contains "${out_dir}/bounded-soak.log" "abort_condition=watchdog_unresponsive"
	assert_contains "${out_dir}/bounded-soak.log" "watchdog_status=observed-or-pending"
	assert_contains "${out_dir}/bounded-soak.log" "safe_stop_status=confirmed-or-pending"
}

test_explicit_target_uses_bounded_http_and_websocket_paths() {
	base_fixture_paths "explicit-target"
	local out_dir="${tmp_root}/explicit-target-out"
	local curl_args="${tmp_root}/curl-args.txt"
	local node_args="${tmp_root}/node-args.txt"

	PHASE21_FAKE_CURL_ARGS="$curl_args" \
		PHASE21_FAKE_NODE_ARGS="$node_args" \
		BITAXE_POOL_URL="stratum+tcp://private.example:3333" \
		BITAXE_POOL_USER="private-worker" \
		BITAXE_POOL_PASSWORD="private-password" \
		run_wrapper "$out_dir" "mining-smoke" --device-url "https://10.0.0.2"

	assert_contains "${out_dir}/mining-smoke.log" "controlled_mining_status: live-prerequisites-present"
	assert_contains "$curl_args" "https://10.0.0.2/api/system/info"
	assert_contains "$node_args" "--path"
	assert_contains "$node_args" "/api/ws/live"
	assert_contains "$node_args" "--duration-ms"
	assert_contains "$node_args" "10000"
	assert_contains "$node_args" "--max-frames"
	assert_contains "$node_args" "5"
	assert_not_contains "${out_dir}/mining-smoke.log" "10.0.0.2"
	assert_not_contains "${out_dir}/api-system-info.redacted.json" "private.example"
	assert_not_contains "${out_dir}/api-system-info.redacted.json" "private-worker"
}

test_missing_manifest_records_pending
test_missing_prerequisite_records_pending
test_missing_live_prerequisites_records_blocked
test_bounded_soak_records_default_duration_and_watchdog_boundary
test_explicit_target_uses_bounded_http_and_websocket_paths

printf 'phase21 live mining evidence tests passed\n'
