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

printf 'command=%s profile=%s slots=%s\n' "$command_name" "$profile" "$slots_status" >>"${PHASE27_FAKE_PARITY_TRACE:?}"
case "$command_name" in
complete-operator-evidence) exit "${PHASE27_FAKE_COMPLETE_EXIT:-0}" ;;
mining-allow) exit "${PHASE27_FAKE_MINING_EXIT:-0}" ;;
operator-evidence) exit "${PHASE27_FAKE_OPERATOR_EXIT:-0}" ;;
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
		printf 'unexpected Phase 27 parity trace\nexpected:\n%s\nactual:\n%s\n' "$expected" "$actual" >&2
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

write_fake_package_manifest() {
	local manifest_path="$1"
	local factory_image_path="$2"

	mkdir -p "$(dirname "$factory_image_path")"
	printf 'fake-factory-image\n' >"$factory_image_path"
	cat >"$manifest_path" <<EOF
{
  "schema_version": 2,
  "source_commit": "$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf 'unknown-source')",
  "default_flash_image": "bitaxe-ultra205.elf",
  "artifacts": [
    {
      "kind": "factory_merged_image",
      "path": "$(basename "$factory_image_path")"
    }
  ]
}
EOF
}

write_fake_live_capture() {
	local path="$1"
	local args_path="$2"
	local share_outcome="$3"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >"${PHASE27_FAKE_LIVE_CAPTURE_ARGS:?}"
capture_evidence_dir=""
for argument in "$@"; do
	case "$argument" in
	evidence-dir=*) capture_evidence_dir="${argument#*=}" ;;
	esac
done
if [[ "${PHASE27_FAKE_RUNTIME_LOG_MODE:-none}" == "prefix-bypass" ]]; then
	mkdir -p "${capture_evidence_dir:?}/pool-input-bridge"
	printf '%s\n' \
		'I (2048) wifi:connected with [redacted-ssid]WorkshopNetwork, aid = 1, channel 6' \
		>"${capture_evidence_dir}/flash-monitor.log"
	printf '%s\n' \
		'Connecting to: stratum+tcp://[redacted]@pool.runtime.example:3333' \
		>"${capture_evidence_dir}/pool-input-bridge/runtime.log"
fi
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

	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review conclusion; do
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

run_hardware_requires_redaction_test() {
	local evidence_root="${tmp_root}/hardware-without-redaction"
	local detector_trace="${tmp_root}/hardware-without-redaction.detector"
	local fake_detector="${tmp_root}/hardware-without-redaction-detector.sh"

	cat >"$fake_detector" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'detector invoked\n' >"${PHASE27_REDACTION_DETECTOR_TRACE:?}"
SH
	chmod +x "$fake_detector"

	set +e
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_REDACTION_DETECTOR_TRACE="$detector_trace" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/hardware-without-redaction.stdout" 2>"${tmp_root}/hardware-without-redaction.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "hardware mode without redaction"
	assert_contains "${tmp_root}/hardware-without-redaction.stderr" "requires --redact-evidence=true"
	if [[ -e "$detector_trace" || -e "$evidence_root" ]]; then
		printf 'redaction argument validation must precede detector and evidence writes\n' >&2
		exit 1
	fi
}

run_blocked_mode_test() {
	local fake_parity="${tmp_root}/fake-parity.sh"
	local evidence_root="${tmp_root}/blocked-root"
	local trace_path="${tmp_root}/blocked.trace"
	write_fake_parity "$fake_parity"

	set +e
	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_FAKE_PARITY_TRACE="$trace_path" \
	PHASE27_RAW_DIAGNOSTIC_SAMPLE="sentinel-pool sentinel-password sentinel-share sentinel-extra sentinel-token raw_bm1366_frame 192.0.2.55 bc1qsentinelowneraddress" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode blocked \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--wifi-credentials "${tmp_root}/SentinelWifi.json" \
		--redact-evidence=true \
		--duration-seconds 60 >"${tmp_root}/blocked.stdout" 2>"${tmp_root}/blocked.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "blocked mode"
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
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase27 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase27 slots=complete'
}

run_hardware_ready_invokes_live_capture_test() {
	local fake_parity="${tmp_root}/fake-parity-live.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205-live.sh"
	local fake_board_info="${tmp_root}/fake-board-info.sh"
	local fake_live_capture="${tmp_root}/fake-live-capture.sh"
	local fake_live_capture_args="${tmp_root}/fake-live-capture.args"
	local manifest_path="${tmp_root}/bitaxe-ultra205-package.json"
	local factory_image="${tmp_root}/bitaxe-ultra205-factory.bin"
	local evidence_root="${tmp_root}/hardware-ready-root"
	local trace_path="${tmp_root}/hardware-ready.trace"
	write_fake_parity "$fake_parity"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_package_manifest "$manifest_path" "$factory_image"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "accepted"

	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_FAKE_PARITY_TRACE="$trace_path" \
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE27_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE27_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	PHASE27_FAKE_SHARE_OUTCOME=accepted \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "$manifest_path" \
		--mode hardware \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--duration-seconds 60 \
		--redact-evidence=true >"${tmp_root}/hardware-ready.stdout"

	assert_required_evidence_slots_exist "$evidence_root"
	assert_file_exists "$fake_live_capture_args"
	assert_contains "$fake_live_capture_args" "board=205"
	assert_contains "$fake_live_capture_args" "port=/dev/cu.usbmodemPHASE27"
	assert_contains "$fake_live_capture_args" "image="
	assert_contains "$fake_live_capture_args" "bitaxe-ultra205-factory.bin"
	assert_contains "$fake_live_capture_args" "redact-evidence=true"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: accepted"
	assert_contains "${evidence_root}/summary.md" "asic_bridge_status: result_correlated"
	assert_contains "${tmp_root}/hardware-ready.stdout" "phase27_evidence_status=accepted"
	assert_not_contains "${evidence_root}/summary.md" "sentinel-pool"
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase27 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase27 slots=complete'
}

run_hardware_missing_prerequisites_skips_live_capture_test() {
	local fake_parity="${tmp_root}/fake-parity-missing-prereqs.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205-missing-prereqs.sh"
	local fake_board_info="${tmp_root}/fake-board-info-missing-prereqs.sh"
	local fake_live_capture="${tmp_root}/fake-live-capture-missing-prereqs.sh"
	local fake_live_capture_args="${tmp_root}/fake-live-capture-missing-prereqs.args"
	local manifest_path="${tmp_root}/bitaxe-ultra205-package.json"
	local factory_image="${tmp_root}/bitaxe-ultra205-factory.bin"
	local evidence_root="${tmp_root}/hardware-missing-prereqs-root"
	local trace_path="${tmp_root}/hardware-missing-prereqs.trace"
	write_fake_parity "$fake_parity"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_package_manifest "$manifest_path" "$factory_image"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "accepted"

	set +e
	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_FAKE_PARITY_TRACE="$trace_path" \
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_BOARD_INFO_COMMAND="$fake_board_info" \
	PHASE27_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
	PHASE27_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "$manifest_path" \
		--mode hardware \
		--redact-evidence=true >"${tmp_root}/hardware-missing-prereqs.stdout" 2>"${tmp_root}/hardware-missing-prereqs.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "missing prerequisites"
	if [[ -e "$fake_live_capture_args" ]]; then
		printf 'live capture helper must not run when prerequisites are missing\n' >&2
		exit 1
	fi
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${tmp_root}/hardware-missing-prereqs.stdout" "phase27_evidence_status=blocked_safe_prerequisite"
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase27 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase27 slots=complete'
}

run_detector_failure_test() {
	local fake_parity="${tmp_root}/fake-parity-detector.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205.sh"
	local evidence_root="${tmp_root}/detector-root"
	local trace_path="${tmp_root}/detector.trace"
	write_fake_parity "$fake_parity"
	write_fake_detector "$fake_detector"

	set +e
	PHASE27_PARITY_COMMAND="$fake_parity" \
	PHASE27_FAKE_PARITY_TRACE="$trace_path" \
	PHASE27_DETECT_COMMAND="$fake_detector" \
	PHASE27_FAKE_DETECT_EXIT=42 \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware \
		--redact-evidence=true >"${tmp_root}/detector.stdout" 2>"${tmp_root}/detector.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "hardware mode detector failure"

	assert_required_evidence_slots_exist "$evidence_root"
	assert_contains "${tmp_root}/detector.stderr" "phase27_detector_status=blocked"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/summary.md" "package_artifact_status: not-run"
	assert_evidence_is_redacted "$evidence_root"
	assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase27 slots=complete\ncommand=operator-evidence profile=phase27 slots=complete'
}

run_failure_precedence_tests() {
	local scenario
	for scenario in completion mining operator prior-workflow; do
		local fake_parity="${tmp_root}/fake-parity-${scenario}.sh"
		local fake_detector="${tmp_root}/fake-detector-${scenario}.sh"
		local fake_board_info="${tmp_root}/fake-board-info-${scenario}.sh"
		local fake_live_capture="${tmp_root}/fake-live-capture-${scenario}.sh"
		local fake_live_capture_args="${tmp_root}/fake-live-capture-${scenario}.args"
		local manifest_path="${tmp_root}/manifest-${scenario}/bitaxe-ultra205-package.json"
		local factory_image="${tmp_root}/manifest-${scenario}/bitaxe-ultra205-factory.bin"
		local evidence_root="${tmp_root}/failure-${scenario}-root"
		local trace_path="${tmp_root}/failure-${scenario}.trace"
		local complete_exit=0
		local mining_exit=0
		local operator_exit=0
		local live_exit=0

		case "$scenario" in
		completion) complete_exit=41 ;;
		mining) mining_exit=42 ;;
		operator) operator_exit=43 ;;
		prior-workflow) live_exit=44 ;;
		esac

		write_fake_parity "$fake_parity"
		write_fake_detector_with_port "$fake_detector"
		write_fake_board_info "$fake_board_info"
		write_fake_package_manifest "$manifest_path" "$factory_image"
		write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "accepted"

		set +e
		PHASE27_PARITY_COMMAND="$fake_parity" \
		PHASE27_FAKE_PARITY_TRACE="$trace_path" \
		PHASE27_FAKE_COMPLETE_EXIT="$complete_exit" \
		PHASE27_FAKE_MINING_EXIT="$mining_exit" \
		PHASE27_FAKE_OPERATOR_EXIT="$operator_exit" \
		PHASE27_DETECT_COMMAND="$fake_detector" \
		PHASE27_BOARD_INFO_COMMAND="$fake_board_info" \
		PHASE27_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
		PHASE27_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
		PHASE27_FAKE_SHARE_OUTCOME=accepted \
		PHASE27_FAKE_LIVE_CAPTURE_EXIT="$live_exit" \
		"$wrapper" \
			--evidence-root "$evidence_root" \
			--manifest "$manifest_path" \
			--mode hardware \
			--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
			--duration-seconds 60 \
			--redact-evidence=true >"${tmp_root}/failure-${scenario}.stdout" 2>"${tmp_root}/failure-${scenario}.stderr"
		local status=$?
		set -e

		assert_nonzero_status "$status" "$scenario failure"
		assert_required_evidence_slots_exist "$evidence_root"
		assert_trace_sequence "$trace_path" $'command=complete-operator-evidence profile=phase27 slots=complete\ncommand=mining-allow profile=none slots=complete\ncommand=operator-evidence profile=phase27 slots=complete'
	done
}

run_real_parity_integration_test() {
	local real_parity
	real_parity="$(find_real_parity)"
	local fake_detector="${tmp_root}/real-parity-detector.sh"
	local relative_evidence_root="scratch/phase27-real-parity-$$"
	local evidence_root="${repo_root}/${relative_evidence_root}"
	write_fake_detector "$fake_detector"
	rm -rf "$evidence_root"

	set +e
	(
		cd "$repo_root"
		export BUILD_WORKSPACE_DIRECTORY="$repo_root"
		PHASE27_PARITY_COMMAND="$real_parity" \
		PHASE27_DETECT_COMMAND="$fake_detector" \
		PHASE27_FAKE_DETECT_EXIT=42 \
			"$wrapper" \
			--evidence-root "$relative_evidence_root" \
			--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
			--mode hardware \
			--redact-evidence=true
	) >"${tmp_root}/real-parity.stdout" 2>"${tmp_root}/real-parity.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "real parity detector failure"
	(
		cd "$repo_root"
		export BUILD_WORKSPACE_DIRECTORY="$repo_root"
		"$real_parity" operator-evidence \
			--profile phase27 \
			--evidence-root "$relative_evidence_root" \
			--require-redaction-passed
	) >"${tmp_root}/real-parity-validation.stdout"
	assert_contains "${tmp_root}/real-parity-validation.stdout" "operator_evidence_status: passed"
	rm -rf "$evidence_root"
}

run_real_parity_rejects_redaction_prefix_bypasses_test() {
	local real_parity
	real_parity="$(find_real_parity)"
	local fake_detector="${tmp_root}/real-parity-runtime-detector.sh"
	local fake_board_info="${tmp_root}/real-parity-runtime-board-info.sh"
	local fake_live_capture="${tmp_root}/real-parity-runtime-capture.sh"
	local fake_live_capture_args="${tmp_root}/real-parity-runtime-capture.args"
	local manifest_path="${tmp_root}/real-parity-runtime/bitaxe-ultra205-package.json"
	local factory_image="${tmp_root}/real-parity-runtime/bitaxe-ultra205-factory.bin"
	local relative_evidence_root="scratch/phase27-real-parity-runtime-$$"
	local evidence_root="${repo_root}/${relative_evidence_root}"
	write_fake_detector_with_port "$fake_detector"
	write_fake_board_info "$fake_board_info"
	write_fake_package_manifest "$manifest_path" "$factory_image"
	write_fake_live_capture "$fake_live_capture" "$fake_live_capture_args" "accepted"
	rm -rf "$evidence_root"

	set +e
	(
		cd "$repo_root"
		export BUILD_WORKSPACE_DIRECTORY="$repo_root"
		PHASE27_PARITY_COMMAND="$real_parity" \
		PHASE27_DETECT_COMMAND="$fake_detector" \
		PHASE27_BOARD_INFO_COMMAND="$fake_board_info" \
		PHASE27_LIVE_CAPTURE_COMMAND="$fake_live_capture" \
		PHASE27_FAKE_LIVE_CAPTURE_ARGS="$fake_live_capture_args" \
		PHASE27_FAKE_RUNTIME_LOG_MODE=prefix-bypass \
			"$wrapper" \
			--evidence-root "$relative_evidence_root" \
			--manifest "$manifest_path" \
			--mode hardware \
			--pool-credentials "${tmp_root}/local-pool-input.json" \
			--duration-seconds 60 \
			--redact-evidence=true
	) >"${tmp_root}/real-parity-runtime.stdout" 2>"${tmp_root}/real-parity-runtime.stderr"
	local status=$?
	set -e

	assert_nonzero_status "$status" "nested runtime redaction validation"
	assert_contains "${tmp_root}/real-parity-runtime.stderr" "live-capture-runtime/flash-monitor.log contains a forbidden redaction sentinel or private runtime value"
	assert_contains "${tmp_root}/real-parity-runtime.stderr" "live-capture-runtime/pool-input-bridge/runtime.log contains a forbidden redaction sentinel or private runtime value"
	assert_not_contains "${tmp_root}/real-parity-runtime.stderr" "WorkshopNetwork"
	assert_not_contains "${tmp_root}/real-parity-runtime.stderr" "pool.runtime.example"
	rm -rf "$evidence_root"
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
run_hardware_requires_redaction_test
run_blocked_mode_test
run_hardware_ready_invokes_live_capture_test
run_hardware_missing_prerequisites_skips_live_capture_test
run_detector_failure_test
run_failure_precedence_tests
run_real_parity_integration_test
run_real_parity_rejects_redaction_prefix_bypasses_test
scan_committed_artifacts_if_present

printf 'phase27_live_hardware_bridge_evidence_test=passed\n'
