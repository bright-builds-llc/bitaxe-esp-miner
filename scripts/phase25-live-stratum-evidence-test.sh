#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE25_EVIDENCE_SCRIPT:-${script_dir}/phase25-live-stratum-evidence.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase25-live-stratum-evidence-test.XXXXXX")"
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

printf 'fake_phase25_parity_args: %s\n' "$*"
printf 'mining_allow_status: passed\n'
SH
	chmod +x "$path"
}

write_fake_detector() {
	local path="$1"
	local exit_code="$2"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake-detect-ultra205 invoked\n' >&2
exit "${PHASE25_FAKE_DETECT_EXIT:-0}"
SH
	chmod +x "$path"
	printf '%s\n' "$exit_code" >"${path}.exit"
}

write_fake_detector_with_port() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake-detect-ultra205 invoked\n' >&2
printf 'port=/dev/cu.usbmodemPHASE25\n'
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
	local status="$3"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >"${PHASE25_FAKE_LIVE_CAPTURE_ARGS:?}"
if [[ "${PHASE25_FAKE_LIVE_CAPTURE_STATUS:-observed}" == "observed" ]]; then
	printf 'phase25_live_submit_response_status=observed redacted=true\n'
	printf 'phase25_safe_stop_status=complete socket=stopped work_queue=invalidated active_work=invalidated mining=disabled hardware_control=disabled work_submission=disabled post_stop_snapshot=updated\n'
	printf 'phase25_watchdog_checkpoint=socket decision=yield redacted=true\n'
	exit 0
fi

printf 'phase25_safe_stop_status=complete socket=stopped mining=disabled hardware_control=disabled work_submission=disabled\n'
printf 'phase25_watchdog_checkpoint=socket decision=yield redacted=true\n'
exit 1
SH
	chmod +x "$path"
	printf '%s\n' "$args_path" >"${path}.args-path"
	printf '%s\n' "$status" >"${path}.status"
}

assert_full_evidence_slots_exist() {
	local evidence_root="$1"
	local slot

	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review summary; do
		assert_file_exists "${evidence_root}/${slot}.md"
	done
}

assert_detector_failure_slots_exist() {
	local evidence_root="$1"
	local slot

	for slot in detector board-info share-outcome safe-stop redaction-review summary; do
		assert_file_exists "${evidence_root}/${slot}.md"
	done

	for slot in package command log api websocket; do
		if [[ -e "${evidence_root}/${slot}.md" ]]; then
			printf 'detector failure must not create %s.md\n' "$slot" >&2
			exit 1
		fi
	done
}

assert_evidence_is_redacted() {
	local evidence_root="$1"
	local path

	for path in "${evidence_root}"/*.md; do
		assert_not_contains "$path" "sentinel-pool"
		assert_not_contains "$path" "sentinel-password"
		assert_not_contains "$path" "sentinel-token"
		assert_not_contains "$path" "sentinel-share"
		assert_not_contains "$path" "sentinel-extra"
		assert_not_contains "$path" "raw_bm1366_frame"
		assert_not_contains "$path" "192.0.2.55"
		assert_not_contains "$path" "bc1qsentinelowneraddress"
	done
}

run_device_url_validation_test() {
	local fake_parity="${tmp_root}/fake-parity-device-url.sh"
	local valid_url
	local invalid_url
	write_fake_parity "$fake_parity"

	for valid_url in "http://host" "http://host:80" "https://device.local/" "https://device.local:443"; do
		PHASE25_PARITY_COMMAND="$fake_parity" \
		"$wrapper" \
			--evidence-root "${tmp_root}/device-url-valid-${valid_url//[^A-Za-z0-9]/-}" \
			--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
			--mode blocked \
			--device-url "$valid_url" >"${tmp_root}/device-url-valid.stdout"
	done

	for invalid_url in "http://host/api/system/info" "http://user@host" "http://host?x=1" "http://host#frag" "http://host:bad" "http://"; do
		set +e
		"$wrapper" \
			--evidence-root "${tmp_root}/device-url-invalid-${invalid_url//[^A-Za-z0-9]/-}" \
			--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
			--mode blocked \
			--device-url "$invalid_url" >"${tmp_root}/device-url-invalid.stdout" 2>"${tmp_root}/device-url-invalid.stderr"
		local status=$?
		set -e

		if [[ "$status" -eq 0 ]]; then
			printf 'invalid device URL should fail validation: %s\n' "$invalid_url" >&2
			exit 1
		fi
		assert_contains "${tmp_root}/device-url-invalid.stderr" "invalid origin-only DEVICE_URL"
	done
}

run_blocked_mode_test() {
	local fake_parity="${tmp_root}/fake-parity.sh"
	local evidence_root="${tmp_root}/blocked-root"
	write_fake_parity "$fake_parity"

	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_RAW_DIAGNOSTIC_SAMPLE="sentinel-pool sentinel-password sentinel-share sentinel-extra sentinel-token raw_bm1366_frame 192.0.2.55 bc1qsentinelowneraddress" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode blocked \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--wifi-credentials "${tmp_root}/SentinelWifi.json" \
		--device-url "http://192.0.2.55"

	assert_full_evidence_slots_exist "$evidence_root"
	assert_contains "${evidence_root}/command.md" "pool_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "wifi_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "raw_artifacts_committed: no"
	assert_contains "${evidence_root}/command.md" "raw_pool_values_committed: no"
	assert_contains "${evidence_root}/command.md" "network_scan: disabled"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/share-outcome.md" "accepted/rejected shares remain non-claims"
	assert_contains "${evidence_root}/safe-stop.md" "safe_stop_status: complete"
	assert_contains "${evidence_root}/safe-stop.md" "work_submission=disabled"
	assert_contains "${evidence_root}/redaction-review.md" "diagnostic_input_status: rejected_sensitive_raw_payload"
	assert_contains "${evidence_root}/summary.md" "raw_artifacts_committed: no"
	assert_contains "${evidence_root}/summary.md" "raw_pool_values_committed: no"
	assert_evidence_is_redacted "$evidence_root"
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
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "observed"

	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_DETECT_COMMAND="$fake_detector" \
	PHASE25_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE25_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE25_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	PHASE25_FAKE_LIVE_CAPTURE_STATUS=observed \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--device-url "http://device.local" \
		--duration-seconds 60 >"${tmp_root}/hardware-ready.stdout"

	assert_full_evidence_slots_exist "$evidence_root"
	assert_file_exists "$fake_live_capture_args"
	assert_contains "$fake_live_capture_args" "board=205"
	assert_contains "$fake_live_capture_args" "port=/dev/cu.usbmodemPHASE25"
	assert_contains "$fake_live_capture_args" "redact-evidence=true"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: live_submit_response_observed"
	assert_contains "${evidence_root}/safe-stop.md" "safe_stop_status: complete"
	assert_contains "${evidence_root}/summary.md" "watchdog_responsiveness_status: passed"
	assert_contains "${tmp_root}/hardware-ready.stdout" "phase25_evidence_status=live_submit_response_observed"
	assert_not_contains "${evidence_root}/summary.md" "device.local"
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
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "observed"

	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_DETECT_COMMAND="$fake_detector" \
	PHASE25_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE25_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE25_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/hardware-missing-prereqs.stdout"

	if [[ -e "$fake_live_capture_args" ]]; then
		printf 'live capture helper must not run when prerequisites are missing\n' >&2
		exit 1
	fi
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${tmp_root}/hardware-missing-prereqs.stdout" "phase25_evidence_status=blocked_safe_prerequisite"
}

run_detector_failure_test() {
	local fake_parity="${tmp_root}/fake-parity-detector.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205.sh"
	local evidence_root="${tmp_root}/detector-root"
	write_fake_parity "$fake_parity"
	write_fake_detector "$fake_detector" 42

	set +e
	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_DETECT_COMMAND="$fake_detector" \
	PHASE25_FAKE_DETECT_EXIT=42 \
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

	assert_detector_failure_slots_exist "$evidence_root"
	assert_contains "${tmp_root}/detector.stderr" "phase25_detector_status=blocked"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/safe-stop.md" "safe_stop_status: blocked"
	assert_contains "${evidence_root}/summary.md" "package_artifact_status: not-run"
	assert_evidence_is_redacted "$evidence_root"
}

run_device_url_validation_test
run_blocked_mode_test
run_hardware_ready_invokes_live_capture_test
run_hardware_missing_prerequisites_skips_live_capture_test
run_detector_failure_test

printf 'phase25_live_stratum_evidence_test=passed\n'
