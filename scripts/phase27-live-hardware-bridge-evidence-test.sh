#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE27_EVIDENCE_SCRIPT:-${script_dir}/phase27-live-hardware-bridge-evidence.sh}"
readonly repo_root="$(cd "${script_dir}/.." && pwd)"
readonly committed_evidence_root="${repo_root}/docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase27-live-hardware-bridge-evidence-test.XXXXXX")"
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
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		exit 1
	fi
}

assert_file_exists() {
	local path="$1"

	if [[ ! -f "$path" ]]; then
		printf 'missing file: %s\n' "$path" >&2
		exit 1
	fi
}

write_fake_parity() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake_phase27_parity_args: %s\n' "$*"
printf 'mining_allow_status: passed\n'
SH
	chmod +x "$path"
}

write_fake_detector() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake-detect-ultra205 invoked\n' >&2
exit "${PHASE27_FAKE_DETECT_EXIT:-0}"
SH
	chmod +x "$path"
}

write_fake_detector_with_port() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake-detect-ultra205 invoked\n' >&2
printf 'port=/dev/cu.usbmodemPHASE27\n'
SH
	chmod +x "$path"
}

write_fake_board_info() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake-board-info invoked: %s\n' "$*" >&2
printf 'Chip type: ESP32-S3\n'
SH
	chmod +x "$path"
}

write_fake_live_capture() {
	local path="$1"
	local args_path="$2"
	local share_outcome="$3"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >"${PHASE27_FAKE_LIVE_CAPTURE_ARGS:?}"
case "${PHASE27_FAKE_SHARE_OUTCOME:-accepted}" in
accepted)
	printf 'phase27_safety_bring_up=complete\n'
	printf 'asic_enable_status=active gpio=10\n'
	printf 'safety_power_status=observed\n'
	printf 'safety_thermal_status=observed\n'
	printf 'safety_fan_status=startup_duty percent=70 rpm=2400\n'
	printf 'asic_production_status=initialized\n'
	printf 'share_submission_status=accepted redacted=true\n'
	printf 'asic_production_status=work_dispatched\n'
	printf 'asic_production_status=result_correlated\n'
	;;
rejected)
	printf 'phase27_safety_bring_up=complete\n'
	printf 'asic_enable_status=active gpio=10\n'
	printf 'safety_power_status=observed\n'
	printf 'safety_thermal_status=observed\n'
	printf 'safety_fan_status=startup_duty percent=70 rpm=2400\n'
	printf 'asic_production_status=initialized\n'
	printf 'share_submission_status=rejected redacted=true\n'
	printf 'asic_production_status=work_dispatched\n'
	printf 'asic_production_status=result_correlated\n'
	;;
*)
	printf 'asic_production_status=fail_closed reason=prerequisite_blocked mining=disabled work_submission=disabled\n'
	;;
esac
printf 'phase25_safe_stop_status=complete socket=stopped work_queue=invalidated active_work=invalidated mining=disabled hardware_control=disabled work_submission=disabled post_stop_snapshot=updated\n'
exit "${PHASE27_FAKE_LIVE_CAPTURE_EXIT:-0}"
SH
	chmod +x "$path"
	printf '%s\n' "$args_path" >"${path}.args-path"
	printf '%s\n' "$share_outcome" >"${path}.share-outcome"
}

assert_required_evidence_slots_exist() {
	local evidence_root="$1"
	local slot

	for slot in share-outcome summary detector board-info command redaction-review conclusion; do
		assert_file_exists "${evidence_root}/${slot}.md"
	done
}

assert_evidence_is_redacted() {
	local evidence_root="$1"
	local path

	for path in "${evidence_root}"/*.md; do
		[[ -f "$path" ]] || continue
		assert_not_contains "$path" "sentinel-pool"
		assert_not_contains "$path" "sentinel-password"
		assert_not_contains "$path" "sentinel-token"
		assert_not_contains "$path" "sentinel-share"
		assert_not_contains "$path" "sentinel-extra"
		assert_not_contains "$path" "raw_bm1366_frame"
		assert_not_contains "$path" "192.0.2.55"
		assert_not_contains "$path" "bc1qsentinelowneraddress"
		assert_not_contains "$path" "stratum+tcp://"
		assert_not_contains "$path" "poolURL"
		assert_not_contains "$path" "poolUser"
		assert_not_contains "$path" "poolPassword"
		assert_not_contains "$path" "device_url="
		assert_not_contains "$path" "extranonce="
		assert_not_contains "$path" "share_payload"
	done
}

assert_manifest_shape() {
	local manifest_path="$1"

	assert_file_exists "$manifest_path"
	assert_contains "$manifest_path" '"surface": "live-hardware-bridge"'
	assert_contains "$manifest_path" '"evidence_mode": "phase27-live-hardware-asic-stratum-bridge"'
	assert_contains "$manifest_path" '"evidence_ack": "ultra205-phase27-live-hardware-bridge-safe-stop"'
	assert_contains "$manifest_path" '"checklist_rows": ["STR-08", "STR-09", "ASIC-10", "ASIC-11"]'
	assert_contains "$manifest_path" "scripts/phase27-live-hardware-bridge-evidence.sh"
}

run_required_args_test() {
	set +e
	"$wrapper" --mode blocked >"${tmp_root}/missing-args.stdout" 2>"${tmp_root}/missing-args.stderr"
	local status=$?
	set -e

	if [[ "$status" -eq 0 ]]; then
		printf 'wrapper must require --evidence-root, --manifest, and --mode\n' >&2
		exit 1
	fi
}

run_blocked_mode_test() {
	local fake_parity="${tmp_root}/fake-parity.sh"
	local evidence_root="${tmp_root}/blocked-root"
	write_fake_parity "$fake_parity"

	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_RAW_DIAGNOSTIC_SAMPLE="sentinel-pool sentinel-password sentinel-share sentinel-extra sentinel-token raw_bm1366_frame 192.0.2.55 bc1qsentinelowneraddress" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode blocked \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--wifi-credentials "${tmp_root}/SentinelWifi.json" \
		--redact-evidence=true \
		--duration-seconds 60 >"${tmp_root}/blocked.stdout"

	assert_required_evidence_slots_exist "$evidence_root"
	assert_contains "${evidence_root}/command.md" "pool_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "wifi_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "raw_artifacts_committed: no"
	assert_contains "${evidence_root}/command.md" "raw_pool_values_committed: no"
	assert_contains "${evidence_root}/command.md" "network_scan: disabled"
	assert_contains "${evidence_root}/command.md" "evidence_mode: phase27-live-hardware-asic-stratum-bridge"
	assert_contains "${evidence_root}/command.md" "evidence_ack: ultra205-phase27-live-hardware-bridge-safe-stop"
	assert_contains "${evidence_root}/command.md" "asic_bridge_status: blocked"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/share-outcome.md" "accepted/rejected shares remain non-claims"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/redaction-review.md" "diagnostic_input_status: rejected_sensitive_raw_payload"
	assert_contains "${evidence_root}/summary.md" "raw_artifacts_committed: no"
	assert_contains "${evidence_root}/summary.md" "raw_pool_values_committed: no"
	assert_contains "${evidence_root}/conclusion.md" "share_outcome: blocked_safe_prerequisite"
	assert_manifest_shape "${evidence_root}/mining-allow.json"
	assert_evidence_is_redacted "$evidence_root"
	assert_contains "${tmp_root}/blocked.stdout" "phase27_evidence_status=blocked_safe_prerequisite"
}

run_hardware_ready_invokes_live_capture_test() {
	local fake_parity="${tmp_root}/fake-parity-live.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205-live.sh"
	local fake_board_info="${tmp_root}/fake-board-info.sh"
	local fake_live_capture="${tmp_root}/fake-live-capture.sh"
	local fake_live_capture_args="${tmp_root}/fake-live-capture.args"
	local evidence_root="${tmp_root}/hardware-ready-root"
	write_fake_parity "$fake_parity"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "accepted"

	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE27_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE27_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	PHASE27_FAKE_SHARE_OUTCOME=accepted \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--duration-seconds 60 \
		--redact-evidence=true >"${tmp_root}/hardware-ready.stdout"

	assert_required_evidence_slots_exist "$evidence_root"
	assert_file_exists "$fake_live_capture_args"
	assert_contains "$fake_live_capture_args" "board=205"
	assert_contains "$fake_live_capture_args" "port=/dev/cu.usbmodemPHASE27"
	assert_contains "$fake_live_capture_args" "redact-evidence=true"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: accepted"
	assert_contains "${evidence_root}/summary.md" "asic_bridge_status: result_correlated"
	assert_contains "${tmp_root}/hardware-ready.stdout" "phase27_evidence_status=accepted"
	assert_not_contains "${evidence_root}/summary.md" "sentinel-pool"
}

run_hardware_missing_prerequisites_skips_live_capture_test() {
	local fake_parity="${tmp_root}/fake-parity-missing-prereqs.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205-missing-prereqs.sh"
	local fake_board_info="${tmp_root}/fake-board-info-missing-prereqs.sh"
	local fake_live_capture="${tmp_root}/fake-live-capture-missing-prereqs.sh"
	local fake_live_capture_args="${tmp_root}/fake-live-capture-missing-prereqs.args"
	local evidence_root="${tmp_root}/hardware-missing-prereqs-root"
	write_fake_parity "$fake_parity"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "accepted"

	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE27_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE27_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/hardware-missing-prereqs.stdout"

	if [[ -e "$fake_live_capture_args" ]]; then
		printf 'live capture helper must not run when prerequisites are missing\n' >&2
		exit 1
	fi
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${tmp_root}/hardware-missing-prereqs.stdout" "phase27_evidence_status=blocked_safe_prerequisite"
}

run_detector_failure_test() {
	local fake_parity="${tmp_root}/fake-parity-detector.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205.sh"
	local evidence_root="${tmp_root}/detector-root"
	write_fake_parity "$fake_parity"
	write_fake_detector "$fake_detector"

	set +e
	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_FAKE_DETECT_EXIT=42 \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/detector.stdout" 2>"${tmp_root}/detector.stderr"
	local status=$?
	set -e

	if [[ "$status" -eq 0 ]]; then
		printf 'hardware mode detector failure should exit non-zero\n' >&2
		exit 1
	fi

	assert_required_evidence_slots_exist "$evidence_root"
	assert_contains "${tmp_root}/detector.stderr" "phase27_detector_status=blocked"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/summary.md" "package_artifact_status: not-run"
	assert_evidence_is_redacted "$evidence_root"
}

scan_committed_artifacts_if_present() {
	if [[ ! -d "$committed_evidence_root" ]]; then
		return 0
	fi

	assert_evidence_is_redacted "$committed_evidence_root"

	local path
	for path in "${committed_evidence_root}"/*.md; do
		[[ -f "$path" ]] || continue
		assert_contains "$path" "redaction_status: passed"
		assert_contains "$path" "raw_artifacts_committed: no"
	done

	if [[ -f "${committed_evidence_root}/share-outcome.md" ]]; then
		if ! grep -Eq 'share_outcome: (accepted|rejected|blocked_safe_prerequisite)' "${committed_evidence_root}/share-outcome.md"; then
			printf 'committed share-outcome.md must use an allowed category label\n' >&2
			exit 1
		fi
	fi
}

run_required_args_test
run_blocked_mode_test
run_hardware_ready_invokes_live_capture_test
run_hardware_missing_prerequisites_skips_live_capture_test
run_detector_failure_test
scan_committed_artifacts_if_present

printf 'phase27_live_hardware_bridge_evidence_test=passed\n'
