#!/usr/bin/env bash
# Runfiles and real-process cases sourced by the Phase 35 supervisor test.

prepare_case() {
	local name="$1"
	local schema="${2:-3}"
	case_dir="${test_root}/${name}"
	state_dir="${case_dir}/state"
	manifest_dir="${case_dir}/package"
	evidence_root="${case_dir}/evidence"
	calls="${state_dir}/calls.log"
	fixture_direct_flash=false
	stub_parity_outcome=passed
	stub_private_input=valid
	flash_boundary_scenario=ready
	probe_boundary_scenario=ready
	supervisor_path="$PATH"
	supervisor_stdout="${case_dir}/stdout.log"
	supervisor_stderr="${case_dir}/stderr.log"
	direct_flash_calls="${state_dir}/direct-flash-calls.log"
	finalizer_calls="${state_dir}/finalizer-calls.log"
	classifier_calls="${state_dir}/classifier-calls.log"
	device_session_calls="${state_dir}/device-session-calls.log"
	nested_tool_calls="${state_dir}/nested-tool-calls.log"
	mkdir -p "$state_dir" "$manifest_dir"
	printf 'fixture-setting-before\n' >"$state_dir/current-setting.txt"
	printf 'fixture-executable\n' >"$manifest_dir/firmware.elf"
	printf 'fixture-factory\n' >"$manifest_dir/factory.bin"
	local executable_digest
	executable_digest="$(shasum -a 256 "$manifest_dir/firmware.elf" | awk '{print $1}')"
	jq -cn \
		--arg schema_version "$schema" \
		--arg source "$source_commit" \
		--arg reference "$reference_commit" \
		--arg app_digest "$executable_digest" \
		'{schema_version:$schema_version,source_commit:$source,reference_commit:$reference,app_elf_sha256:$app_digest,build_identity:{label:"fixture-build"},artifacts:[{kind:"firmware_elf",path:"firmware.elf"},{kind:"factory_merged_image",path:"factory.bin"}]}' \
		>"$manifest_dir/manifest.json"
}

run_supervisor() {
	local scenario="$1"
	shift
	active_scenario="$scenario"
	set +e
	BUILD_WORKSPACE_DIRECTORY="$workspace" \
		PHASE35_FIXTURE_COMMAND="$fixture" \
		PHASE35_FIXTURE_STATE="$state_dir" \
		PHASE35_FIXTURE_SCENARIO="$scenario" \
		"$supervisor" \
		"manifest=${manifest_dir}/manifest.json" \
		"local-root=${evidence_root}" \
		wifi-credentials=fixture-input \
		capture-timeout-seconds=360 \
		caller-wall-clock-seconds=420 \
		"$@" >"$case_dir/stdout.log" 2>"$case_dir/stderr.log"
	run_status=$?
	set -e
}

prepare_isolated_supervisor() {
	isolated_supervisor="${case_dir}/phase35_correlated_evidence"
	local runfiles_scripts="${isolated_supervisor}.runfiles/_main/scripts"
	mkdir -p "$runfiles_scripts"
	ln -s "$supervisor" "$isolated_supervisor"
	cp \
		"${script_dir}/espflash-tool.sh" \
		"${script_dir}/phase35-correlated-evidence-admission.sh" \
		"${script_dir}/phase35-correlated-evidence-finalization.sh" \
		"${script_dir}/phase35-correlated-evidence-root.sh" \
		"${script_dir}/phase35-stage-readiness.sh" \
		"${script_dir}/phase35-correlated-evidence-effects.sh" \
		"${script_dir}/phase35-correlated-evidence-runtime.sh" \
		"${script_dir}/phase35-correlated-evidence-document.sh" \
		"${script_dir}/serial-session-trace.sh" \
		"$runfiles_scripts/"
}

run_isolated_supervisor() {
	local scenario="$1"
	shift
	active_scenario="$scenario"
	set +e
	(
		cd "$case_dir"
		BUILD_WORKSPACE_DIRECTORY="$workspace" \
			PATH="$supervisor_path" \
			PHASE35_FIXTURE_COMMAND="$fixture" \
			PHASE35_FIXTURE_DIRECT_FLASH="$fixture_direct_flash" \
			PHASE35_FIXTURE_STATE="$state_dir" \
			PHASE35_FIXTURE_SCENARIO="$scenario" \
			PHASE35_FIXTURE_EXPECTED_CREDENTIAL_PATH="${workspace}/wifi-credentials.json" \
			PHASE35_DIRECT_FLASH_CALLS="$direct_flash_calls" \
			PHASE35_FINALIZER_CALLS="$finalizer_calls" \
			PHASE35_CLASSIFIER_CALLS="$classifier_calls" \
			PHASE35_DEVICE_SESSION_CALLS="$device_session_calls" \
			PHASE35_NESTED_TOOL_CALLS="$nested_tool_calls" \
			PHASE35_TEST_PRIVATE_INPUT="$stub_private_input" \
			PHASE35_TEST_PARITY_OUTCOME="$stub_parity_outcome" \
			PHASE35_TEST_FLASH_BOUNDARY_SCENARIO="$flash_boundary_scenario" \
			PHASE35_TEST_PROBE_BOUNDARY_SCENARIO="$probe_boundary_scenario" \
			PHASE35_TEST_STUB_DISPATCH=true \
			"$isolated_supervisor" \
			"manifest=${manifest_dir}/manifest.json" \
			"local-root=${evidence_root}" \
			wifi-credentials=wifi-credentials.json \
			capture-timeout-seconds=360 \
			caller-wall-clock-seconds=420 \
			"$@"
	) >"$supervisor_stdout" 2>"$supervisor_stderr"
	run_status=$?
	set -e
}

prepare_direct_flash_stubs() {
	local flash_bin="${workspace}/bazel-bin/tools/flash/flash"
	local parity_bin="${workspace}/bazel-bin/tools/parity/report"
	local device_session_bin="${workspace}/bazel-bin/tools/device-session/device-session"
	local blocked_bin="${case_dir}/blocked-bin"
	mkdir -p \
		"$(dirname "$flash_bin")" \
		"$(dirname "$parity_bin")" \
		"$(dirname "$device_session_bin")" \
		"$blocked_bin"
	rm -f "$flash_bin" "$parity_bin" "$device_session_bin"
	ln -s "$test_entrypoint" "$flash_bin"
	ln -s "$test_entrypoint" "$parity_bin"
	ln -s "$test_entrypoint" "$device_session_bin"
	ln -s "$test_entrypoint" "$blocked_bin/just"
	ln -s "$test_entrypoint" "$blocked_bin/bazel"
	supervisor_path="${blocked_bin}:${PATH}"
}

test_runfiles_rejects_existing_child_before_admission_or_effects() {
	# Arrange
	prepare_case runfiles_existing_child
	prepare_isolated_supervisor
	mkdir -p "$evidence_root"
	chmod 700 "$evidence_root"
	local sentinel="${evidence_root}/sentinel"
	printf 'opaque-sentinel\n' >"$sentinel"
	chmod 600 "$sentinel"
	local child_metadata_before sentinel_metadata_before sentinel_digest_before
	child_metadata_before="$(path_metadata "$evidence_root")"
	sentinel_metadata_before="$(path_metadata "$sentinel")"
	sentinel_digest_before="$(file_digest "$sentinel")"

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "existing evidence child was accepted"
	assert_line "$case_dir/stderr.log" 'failure_category=evidence_root_already_exists'
	[[ ! -e "$calls" ]] ||
		fail_test "existing evidence child reached package admission or a later command"
	[[ "$(path_metadata "$evidence_root")" == "$child_metadata_before" ]] ||
		fail_test "existing evidence child metadata changed after rejection"
	[[ "$(path_metadata "$sentinel")" == "$sentinel_metadata_before" ]] ||
		fail_test "existing evidence sentinel metadata changed after rejection"
	[[ "$(file_digest "$sentinel")" == "$sentinel_digest_before" ]] ||
		fail_test "existing evidence sentinel content changed after rejection"
	[[ "$(find "$evidence_root" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" == 1 ]] ||
		fail_test "existing evidence child gained artifacts after rejection"
}

test_runfiles_preserves_caller_owned_parent_and_sibling_outputs() {
	# Arrange
	prepare_case runfiles_protected_parent
	prepare_isolated_supervisor
	local protected_parent="${case_dir}/protected"
	mkdir -p "$protected_parent"
	chmod 700 "$protected_parent"
	evidence_root="${protected_parent}/supervisor-child"
	supervisor_stdout="${protected_parent}/wrapper.stdout"
	supervisor_stderr="${protected_parent}/wrapper.stderr"
	[[ ! -e "$evidence_root" ]] ||
		fail_test "supervisor child existed before sibling output creation"
	: >"$supervisor_stdout"
	: >"$supervisor_stderr"
	chmod 600 "$supervisor_stdout" "$supervisor_stderr"
	[[ -f "$supervisor_stdout" && -f "$supervisor_stderr" ]] ||
		fail_test "caller-owned sibling outputs were not created"
	[[ ! -e "$evidence_root" ]] ||
		fail_test "sibling output creation pre-created the supervisor child"

	# Act
	run_isolated_supervisor success preflight-only=true

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "protected-parent preflight failed"
	[[ -d "$evidence_root" ]] ||
		fail_test "supervisor did not create its child after sibling outputs"
	[[ "$(file_mode "$protected_parent")" == "700" ]] ||
		fail_test "caller-owned protected parent mode is not 0700"
	[[ "$(file_mode "$supervisor_stdout")" == "600" ]] ||
		fail_test "wrapper stdout mode is not 0600"
	[[ "$(file_mode "$supervisor_stderr")" == "600" ]] ||
		fail_test "wrapper stderr mode is not 0600"
	while IFS= read -r directory; do
		[[ "$(file_mode "$directory")" == "700" ]] ||
			fail_test "supervisor-created directory mode is not 0700"
	done < <(find "$evidence_root" -type d)
	while IFS= read -r file; do
		[[ "$(file_mode "$file")" == "600" ]] ||
			fail_test "supervisor-created file mode is not 0600"
	done < <(find "$evidence_root" -type f)
	assert_contains "$supervisor_stdout" '^status=preflight_passed$'
}

test_runfiles_entrypoint_resolves_sibling_helpers() {
	# Arrange
	prepare_case runfiles_preflight
	prepare_isolated_supervisor

	# Act
	run_isolated_supervisor success preflight-only=true

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "runfiles preflight failed"
	assert_contains "$case_dir/stdout.log" '^status=preflight_passed$'
	assert_count 1 package_admission "$calls"
	assert_count 0 detector "$calls"
	assert_count 0 credential_path "$calls"
}

test_built_supervisor_runfiles_select_device_session_runtime_observer() {
	# Arrange
	local supervisor_rule
	supervisor_rule="$(sed -n \
		'/name = "phase35_correlated_evidence"/,/^)/p' \
		"${script_dir}/BUILD.bazel")"
	local reboot_function
	reboot_function="$(sed -n \
		'/^run_device_session_reboot()/,/^}/p' \
		"${script_dir}/phase35-correlated-evidence-runtime.sh")"

	# Act
	local selects_device_session=false
	if [[ "$supervisor_rule" == *'//tools/device-session:device-session'* ]] &&
		[[ "$reboot_function" == *'resolve_device_session_executable'* ]]; then
		selects_device_session=true
	fi

	# Assert
	[[ "$selects_device_session" == true ]] ||
		fail_test "built supervisor runfiles omit the device-session runtime observer"
	[[ "$supervisor_rule" != *phase13-monitor-capture.sh* ]] ||
		fail_test "built supervisor still distributes the obsolete runtime observer"
	[[ "$reboot_function" != *espflash* && "$reboot_function" != *phase13-monitor-capture* ]] ||
		fail_test "runtime reboot observation still invokes an espflash monitor"
}

test_runfiles_resolves_repo_root_credential_only_after_detector() {
	# Arrange: a detector failure must not touch the opaque credential path.
	prepare_case runfiles_detector_failure
	prepare_isolated_supervisor

	# Act
	run_isolated_supervisor zero_candidates

	# Assert
	assert_pre_patch_failure
	assert_count 1 detector "$calls"
	assert_count 0 credential_path "$calls"

	# Arrange: the same relative argument exists only at the original workspace root.
	prepare_case runfiles_credential_success
	prepare_isolated_supervisor
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "workspace-root credential resolution failed"
	assert_count 1 detector "$calls"
	assert_count 1 credential_path "$calls"
	local detector_line credential_line
	detector_line="$(line_number detector "$calls")"
	credential_line="$(line_number credential_path "$calls")"
	[[ "$detector_line" -lt "$credential_line" ]] ||
		fail_test "credential path was resolved before detector authority"
}

test_runfiles_invokes_probe_then_flash_without_nested_build_tools() {
	# Arrange
	prepare_case runfiles_direct_flash
	prepare_isolated_supervisor
	prepare_direct_flash_stubs
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"
	fixture_direct_flash=true

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "direct flash fixture scenario failed"
	assert_count 1 detector "$calls"
	assert_count 1 flash_probe "$calls"
	assert_count 1 credential_path "$calls"
	assert_count 1 direct_flash "$calls"
	assert_count 1 finalize_evidence "$calls"
	assert_count 2 CALL "$direct_flash_calls"
	assert_count 1 CALL "$finalizer_calls"
	[[ "$(rg -c '^arg=' "$direct_flash_calls")" == 24 ]] ||
		fail_test "direct flash received unexpected or missing arguments"
	[[ ! -s "$nested_tool_calls" ]] || fail_test "direct flash path invoked nested just or Bazel"
	assert_line "$direct_flash_calls" 'arg=flash-monitor'
	assert_line "$direct_flash_calls" 'arg=phase35-probe'
	assert_line "$direct_flash_calls" 'arg=--stage-root'
	assert_line "$direct_flash_calls" "arg=${evidence_root}/raw/flash/private-stages"
	assert_line "$direct_flash_calls" 'arg=--timeout-seconds'
	assert_line "$direct_flash_calls" 'arg=30'
	assert_line "$direct_flash_calls" 'arg=--board'
	assert_line "$direct_flash_calls" 'arg=205'
	assert_line "$direct_flash_calls" 'arg=--port'
	assert_line "$direct_flash_calls" 'arg=fixture-device'
	assert_line "$direct_flash_calls" 'arg=--manifest'
	assert_line "$direct_flash_calls" "arg=${manifest_dir}/manifest.json"
	assert_line "$direct_flash_calls" 'arg=--evidence-dir'
	assert_line "$direct_flash_calls" "arg=${evidence_root}/raw/flash"
	assert_line "$direct_flash_calls" 'arg=--capture-timeout-seconds'
	assert_line "$direct_flash_calls" 'arg=360'
	assert_line "$direct_flash_calls" 'arg=--evidence-mode'
	assert_line "$direct_flash_calls" 'arg=dual'
	assert_absent "$direct_flash_calls" 'arg=--redact-evidence'
	assert_line "$direct_flash_calls" 'arg=--wifi-credentials'
	assert_line "$direct_flash_calls" "arg=${workspace}/wifi-credentials.json"
	[[ "$(rg -c '^arg=' "$finalizer_calls")" == 5 ]] ||
		fail_test "finalizer received unexpected or missing arguments"
	assert_line "$finalizer_calls" 'arg=finalize-evidence'
	assert_line "$finalizer_calls" 'arg=--evidence-dir'
	assert_line "$finalizer_calls" "arg=${evidence_root}/raw/flash"
	assert_line "$finalizer_calls" 'arg=--expected-private-sha256'
	assert_line "$classifier_calls" \
		"trace=${evidence_root}/raw/flash/flash-monitor.classifier-input.log"
	local private_origin_pattern
	private_origin_pattern="$(printf 'device_%s=%s%s' url http '://fixture-target')"
	assert_contains "$evidence_root/raw/flash/flash-monitor.classifier-input.log" \
		"$private_origin_pattern"
	assert_absent "$evidence_root/raw/flash/flash-monitor.log" \
		"$private_origin_pattern"
	local private_digest
	private_digest="$(file_digest \
		"$evidence_root/raw/flash/flash-monitor.classifier-input.log")"
	assert_line "$finalizer_calls" "arg=${private_digest}"
	[[ -s "$evidence_root/raw/flash/flash-monitor.log" ]] ||
		fail_test "finalizer did not create the legacy admitted log"
	[[ -s "$evidence_root/raw/flash/flash-command-evidence.json" ]] ||
		fail_test "finalizer did not create the admitted record"

	local detector_line credential_line flash_line classifier_line finalizer_line original_read_line
	detector_line="$(line_number detector "$calls")"
	credential_line="$(line_number credential_path "$calls")"
	flash_line="$(line_number direct_flash "$calls")"
	classifier_line="$(line_number classifier "$calls")"
	finalizer_line="$(line_number finalize_evidence "$calls")"
	original_read_line="$(line_number read_setting_original "$calls")"
	[[ "$detector_line" -lt "$credential_line" && "$credential_line" -lt "$flash_line" ]] ||
		fail_test "direct flash ran before detector and credential gates"
	[[ "$flash_line" -lt "$classifier_line" && "$classifier_line" -lt "$finalizer_line" && "$finalizer_line" -lt "$original_read_line" ]] ||
		fail_test "private classification and finalization did not precede target use and PATCH"
}

test_probe_failure_stops_before_credentials_or_writes() {
	# Arrange
	prepare_case runfiles_probe_failure
	prepare_isolated_supervisor
	prepare_direct_flash_stubs
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"
	fixture_direct_flash=true
	probe_boundary_scenario=pre_connect

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "failed read-only probe was accepted"
	assert_line "$evidence_root/non-promotion.seal" 'category=pre_connect_failure'
	assert_line "$evidence_root/non-promotion.seal" 'boundary_schema=phase35-flash-boundary-v1'
	assert_line "$evidence_root/non-promotion.seal" 'flash_stage=probe'
	assert_line "$evidence_root/non-promotion.seal" 'flash_boundary=pre_connect_failure'
	assert_count 1 detector "$calls"
	assert_count 1 flash_probe "$calls"
	assert_count 0 credential_path "$calls"
	assert_count 0 direct_flash "$calls"
	assert_absent "$calls" 'read_setting_|capture_|mutated_setting|patch|reboot|restore|validator'
	[[ ! -e "$evidence_root/raw/flash/flash-monitor.classifier-input.log" ]] ||
		fail_test "probe failure created monitor evidence"
}

test_probe_identity_drift_stops_before_credentials_or_writes() {
	# Arrange
	prepare_case probe_identity_drift

	# Act
	run_supervisor probe_identity_drift

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "post-probe identity drift was accepted"
	assert_line "$evidence_root/non-promotion.seal" 'category=after_probe_identity_changed'
	assert_line "$evidence_root/non-promotion.seal" 'flash_stage=probe'
	assert_line "$evidence_root/non-promotion.seal" 'flash_boundary=ready'
	assert_count 1 flash_probe "$calls"
	assert_count 0 credential_path "$calls"
	assert_count 0 flash_boot_a "$calls"
	assert_absent "$calls" 'read_setting_|capture_|mutated_setting|patch|reboot|restore|validator'
}

test_commit_redacted_copy_reproduces_attempt_11_origin_loss() {
	# Arrange
	prepare_case runfiles_early_redaction
	prepare_isolated_supervisor
	prepare_direct_flash_stubs
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"
	fixture_direct_flash=true

	# Act
	run_isolated_supervisor success
	local public_result
	public_result="$(
		PHASE35_FIXTURE_STATE="$state_dir" \
			PHASE35_CLASSIFIER_CALLS="$classifier_calls" \
			PHASE35_TEST_PARITY_OUTCOME=passed \
			PHASE35_TEST_PRIVATE_INPUT=valid \
			PHASE35_TEST_STUB_DISPATCH=true \
			"${workspace}/bazel-bin/tools/parity/report" \
			phase33-classify \
			--trace "${evidence_root}/raw/flash/flash-monitor.log" \
			--mode baseline
	)"

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "private classifier input was rejected"
	[[ "$(jq -r '.status' <<<"$public_result")" == failed ]] ||
		fail_test "commit-redacted copy unexpectedly retained classifier origin"
	[[ "$(jq -r '.category' <<<"$public_result")" == baseline_origin_missing ]] ||
		fail_test "attempt-11 early-redaction defect was not reproduced"
}

test_invalid_private_classifier_input_stops_before_mutation() {
	# Arrange
	prepare_case runfiles_invalid_private_input
	prepare_isolated_supervisor
	prepare_direct_flash_stubs
	stub_private_input=invalid
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"
	fixture_direct_flash=true

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "invalid private classifier input was accepted"
	assert_line "$case_dir/stderr.log" 'failure_category=baseline_origin_missing'
	assert_count 1 classifier "$calls"
	assert_count 0 finalize_evidence "$calls"
	assert_count 0 restore "$calls"
	assert_count 1 cleanup "$calls"
	assert_absent "$calls" 'read_setting_|capture_|mutated_setting|patch|reboot|validator'
	[[ ! -e "$evidence_root/raw/flash/flash-monitor.log" ]] ||
		fail_test "classifier failure created an admitted log"
	[[ ! -e "$evidence_root/raw/flash/flash-command-evidence.json" ]] ||
		fail_test "classifier failure created an admitted record"
	assert_line "$evidence_root/non-promotion.seal" \
		'category=baseline_origin_missing'
}

test_direct_flash_classifier_rejection_preserves_typed_category() {
	# Arrange
	prepare_case runfiles_classifier_rejection
	prepare_isolated_supervisor
	prepare_direct_flash_stubs
	stub_parity_outcome=rejected
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"
	fixture_direct_flash=true

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "rejected Boot A classification was accepted"
	assert_line "$case_dir/stderr.log" 'failure_category=baseline_multiple_sessions'
	assert_line "$evidence_root/non-promotion.seal" 'category=baseline_multiple_sessions'
	assert_count 1 detector "$calls"
	assert_count 1 credential_path "$calls"
	assert_count 1 direct_flash "$calls"
	assert_count 0 finalize_evidence "$calls"
	assert_count 1 cleanup "$calls"
	assert_absent "$calls" 'read_setting_|capture_|mutated_setting|patch|reboot|restore|validator'
	[[ ! -e "$evidence_root/raw/flash/flash-monitor.log" ]] ||
		fail_test "classifier rejection created an admitted log"
}

test_real_process_factory_post_info_failure_is_typed() {
	# Arrange
	prepare_case runfiles_factory_post_info_failure
	prepare_isolated_supervisor
	prepare_direct_flash_stubs
	printf 'opaque-fixture-input\n' >"${workspace}/wifi-credentials.json"
	fixture_direct_flash=true
	flash_boundary_scenario=post_info_pre_transfer

	# Act
	run_isolated_supervisor success

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "post-info/pre-transfer fake espflash failure passed"
	assert_line "$evidence_root/non-promotion.seal" 'category=post_info_pre_transfer_failed'
	assert_line "$evidence_root/non-promotion.seal" 'boundary_schema=phase35-flash-boundary-v1'
	assert_line "$evidence_root/non-promotion.seal" 'flash_stage=factory'
	assert_line "$evidence_root/non-promotion.seal" 'flash_boundary=post_info_pre_transfer_failed'
	assert_count 2 flash_classifier "$calls"
	assert_absent "$calls" 'read_setting_|capture_|mutated_setting|patch|reboot|restore|validator'
}
