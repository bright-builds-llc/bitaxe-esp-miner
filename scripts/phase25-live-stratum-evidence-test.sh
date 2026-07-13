#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE25_EVIDENCE_SCRIPT:-${script_dir}/phase25-live-stratum-evidence.sh}"
readonly repo_root="$(cd "${script_dir}/.." && pwd)"

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

command_name="${1:-missing}"
profile="none"
evidence_root=""
arguments=("$@")
for ((index = 0; index < ${#arguments[@]}; index += 1)); do
	case "${arguments[$index]}" in
	--profile)
		profile="${arguments[$((index + 1))]:-missing}"
		;;
	--evidence-root)
		evidence_root="${arguments[$((index + 1))]:-}"
		;;
	--manifest)
		if [[ -z "$evidence_root" ]]; then
			evidence_root="$(dirname "${arguments[$((index + 1))]:-}")"
		fi
		;;
	esac
done

if [[ "$command_name" == "complete-operator-evidence" && -n "$evidence_root" ]]; then
	mkdir -p "$evidence_root"
	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review conclusion; do
		[[ -f "${evidence_root}/${slot}.md" ]] || : >"${evidence_root}/${slot}.md"
	done
fi

slots_status="incomplete"
if [[ -n "$evidence_root" ]]; then
	slots_status="complete"
	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review conclusion; do
		if [[ ! -f "${evidence_root}/${slot}.md" ]]; then
			slots_status="incomplete"
			break
		fi
	done
fi

printf 'command=%s profile=%s slots=%s\n' "$command_name" "$profile" "$slots_status" >>"${PHASE25_FAKE_PARITY_TRACE:?}"
case "$command_name" in
complete-operator-evidence) exit "${PHASE25_FAKE_COMPLETE_EXIT:-0}" ;;
mining-allow) exit "${PHASE25_FAKE_MINING_EXIT:-0}" ;;
operator-evidence) exit "${PHASE25_FAKE_OPERATOR_EXIT:-0}" ;;
*) exit 99 ;;
esac
SH
	chmod +x "$path"
}

assert_trace_sequence() {
	local trace_path="$1"
	local expected="$2"
	local actual
	actual="$(<"$trace_path")"

	if [[ "$actual" != "$expected" ]]; then
		printf 'unexpected Phase 25 parity trace\nexpected:\n%s\nactual:\n%s\n' "$expected" "$actual" >&2
		exit 1
	fi
}

assert_nonzero_status() {
	local status="$1"
	local scenario="$2"

	if [[ "$status" -eq 0 ]]; then
		printf '%s should exit non-zero\n' "$scenario" >&2
		exit 1
	fi
}

find_real_parity() {
	local candidate
	for candidate in \
		"${script_dir}/../tools/parity/report" \
		"${repo_root}/target/debug/bitaxe-parity" \
		"${repo_root}/bazel-bin/tools/parity/report"; do
		if [[ -x "$candidate" ]]; then
			printf '%s' "$candidate"
			return 0
		fi
	done
	printf 'production parity binary was not found\n' >&2
	return 1
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
		local trace_path="${tmp_root}/device-url-valid-${valid_url//[^A-Za-z0-9]/-}.trace"
		set +e
		PHASE25_PARITY_COMMAND="$fake_parity" \
		PHASE25_FAKE_PARITY_TRACE="$trace_path" \
		"$wrapper" \
			--evidence-root "${tmp_root}/device-url-valid-${valid_url//[^A-Za-z0-9]/-}" \
			--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
			--mode blocked \
			--device-url "$valid_url" >"${tmp_root}/device-url-valid.stdout" 2>"${tmp_root}/device-url-valid.stderr"
		local status=$?
		set -e
		assert_nonzero_status "$status" "valid blocked-mode device URL"
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
	local trace_path="${tmp_root}/blocked.trace"
	write_fake_parity "$fake_parity"

	set +e
	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_FAKE_PARITY_TRACE="$trace_path" \
	PHASE25_RAW_DIAGNOSTIC_SAMPLE="sentinel-pool sentinel-password sentinel-share sentinel-extra sentinel-token raw_bm1366_frame 192.0.2.55 bc1qsentinelowneraddress" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode blocked \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--wifi-credentials "${tmp_root}/SentinelWifi.json" \
		--device-url "http://192.0.2.55" >"${tmp_root}/blocked.stdout" 2>"${tmp_root}/blocked.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "blocked mode"
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
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase25 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase25 slots=complete'
}

run_hardware_ready_invokes_live_capture_test() {
	local fake_parity="${tmp_root}/fake-parity-live.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205-live.sh"
	local fake_board_info="${tmp_root}/fake-board-info.sh"
	local fake_live_capture="${tmp_root}/fake-live-capture.sh"
	local fake_live_capture_args="${tmp_root}/fake-live-capture.args"
	local evidence_root="${tmp_root}/hardware-ready-root"
	local trace_path="${tmp_root}/hardware-ready.trace"
	write_fake_parity "$fake_parity"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "observed"

	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_FAKE_PARITY_TRACE="$trace_path" \
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
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase25 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase25 slots=complete'
}

run_hardware_missing_prerequisites_skips_live_capture_test() {
	local fake_parity="${tmp_root}/fake-parity-missing-prereqs.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205-missing-prereqs.sh"
	local fake_board_info="${tmp_root}/fake-board-info-missing-prereqs.sh"
	local fake_live_capture="${tmp_root}/fake-live-capture-missing-prereqs.sh"
	local fake_live_capture_args="${tmp_root}/fake-live-capture-missing-prereqs.args"
	local evidence_root="${tmp_root}/hardware-missing-prereqs-root"
	local trace_path="${tmp_root}/hardware-missing-prereqs.trace"
	write_fake_parity "$fake_parity"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "observed"

	set +e
	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_FAKE_PARITY_TRACE="$trace_path" \
	PHASE25_DETECT_COMMAND="$fake_detector" \
	PHASE25_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE25_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE25_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/hardware-missing-prereqs.stdout" 2>"${tmp_root}/hardware-missing-prereqs.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "missing prerequisites"
	if [[ -e "$fake_live_capture_args" ]]; then
		printf 'live capture helper must not run when prerequisites are missing\n' >&2
		exit 1
	fi
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${tmp_root}/hardware-missing-prereqs.stdout" "phase25_evidence_status=blocked_safe_prerequisite"
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase25 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase25 slots=complete'
}

run_detector_failure_test() {
	local fake_parity="${tmp_root}/fake-parity-detector.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205.sh"
	local evidence_root="${tmp_root}/detector-root"
	local trace_path="${tmp_root}/detector.trace"
	write_fake_parity "$fake_parity"
	write_fake_detector "$fake_detector" 42

	set +e
	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_FAKE_PARITY_TRACE="$trace_path" \
	PHASE25_DETECT_COMMAND="$fake_detector" \
	PHASE25_FAKE_DETECT_EXIT=42 \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/detector.stdout" 2>"${tmp_root}/detector.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "hardware mode detector failure"

	assert_full_evidence_slots_exist "$evidence_root"
	assert_contains "${tmp_root}/detector.stderr" "phase25_detector_status=blocked"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/safe-stop.md" "safe_stop_status: blocked"
	assert_contains "${evidence_root}/summary.md" "package_artifact_status: not-run"
	assert_evidence_is_redacted "$evidence_root"
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase25 slots=complete\ncommand=operator-evidence profile=phase25 slots=complete'
}

run_failure_precedence_tests() {
	local scenario
	for scenario in completion mining operator prior-workflow; do
		local fake_parity="${tmp_root}/fake-parity-${scenario}.sh"
		local fake_detector="${tmp_root}/fake-detector-${scenario}.sh"
		local fake_board_info="${tmp_root}/fake-board-info-${scenario}.sh"
		local fake_live_capture="${tmp_root}/fake-live-capture-${scenario}.sh"
		local fake_live_capture_args="${tmp_root}/fake-live-capture-${scenario}.args"
		local evidence_root="${tmp_root}/failure-${scenario}-root"
		local trace_path="${tmp_root}/failure-${scenario}.trace"
		local complete_exit=0
		local mining_exit=0
		local operator_exit=0
		local live_status="observed"

		case "$scenario" in
		completion) complete_exit=41 ;;
		mining) mining_exit=42 ;;
		operator) operator_exit=43 ;;
		prior-workflow) live_status="not-observed" ;;
		esac

		write_fake_parity "$fake_parity"
		write_fake_detector_with_port "$fake_detector"
		write_fake_board_info "$fake_board_info"
		write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "$live_status"

		set +e
		PHASE25_PARITY_COMMAND="$fake_parity" \
		PHASE25_FAKE_PARITY_TRACE="$trace_path" \
		PHASE25_FAKE_COMPLETE_EXIT="$complete_exit" \
		PHASE25_FAKE_MINING_EXIT="$mining_exit" \
		PHASE25_FAKE_OPERATOR_EXIT="$operator_exit" \
		PHASE25_DETECT_COMMAND="$fake_detector" \
		PHASE25_BOARD_INFO_COMMAND="$fake_board_info" \
		PHASE25_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
		PHASE25_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
		PHASE25_FAKE_LIVE_CAPTURE_STATUS="$live_status" \
		"$wrapper" \
			--evidence-root "$evidence_root" \
			--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
			--mode hardware \
			--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
			--device-url "http://device.local" \
			--duration-seconds 60 >"${tmp_root}/failure-${scenario}.stdout" 2>"${tmp_root}/failure-${scenario}.stderr"
		local status=$?
		set -e

		assert_nonzero_status "$status" "$scenario failure"
		assert_full_evidence_slots_exist "$evidence_root"
		assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase25 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase25 slots=complete'
	done
}

run_real_parity_integration_test() {
	local real_parity
	real_parity="$(find_real_parity)"
	local fake_detector="${tmp_root}/real-parity-detector.sh"
	local relative_evidence_root="scratch/phase25-real-parity-$$"
	local evidence_root="${repo_root}/${relative_evidence_root}"
	write_fake_detector "$fake_detector" 42
	rm -rf "$evidence_root"

	set +e
	(
		cd "$repo_root"
		export BUILD_WORKSPACE_DIRECTORY="$repo_root"
		PHASE25_PARITY_COMMAND="$real_parity" \
		PHASE25_DETECT_COMMAND="$fake_detector" \
		PHASE25_FAKE_DETECT_EXIT=42 \
			"$wrapper" \
			--evidence-root "$relative_evidence_root" \
			--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
			--mode hardware
	) >"${tmp_root}/real-parity.stdout" 2>"${tmp_root}/real-parity.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "real parity detector failure"
	(
		cd "$repo_root"
		export BUILD_WORKSPACE_DIRECTORY="$repo_root"
		"$real_parity" operator-evidence \
			--profile phase25 \
			--evidence-root "$relative_evidence_root" \
			--require-redaction-passed
	) >"${tmp_root}/real-parity-validation.stdout"
	assert_contains "${tmp_root}/real-parity-validation.stdout" "operator_evidence_status: passed"
	rm -rf "$evidence_root"
}

run_device_url_validation_test
run_blocked_mode_test
run_hardware_ready_invokes_live_capture_test
run_hardware_missing_prerequisites_skips_live_capture_test
run_detector_failure_test
run_failure_precedence_tests
run_real_parity_integration_test

printf 'phase25_live_stratum_evidence_test=passed\n'
