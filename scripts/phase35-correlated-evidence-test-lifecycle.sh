#!/usr/bin/env bash
# Supervisor lifecycle, failure-precedence, and cleanup tests for Phase 35.

assert_detector_stopped_effects() {
	assert_count 1 detector "$calls"
	assert_absent "$calls" 'credential_path|flash_boot_a|validate_target|read_setting_|capture_|mutated_setting|patch|reboot|restore|validator'
}

assert_pre_patch_failure() {
	[[ "$run_status" != 0 ]] || fail_test "expected pre-PATCH failure for ${active_scenario}"
	assert_count 0 restore "$calls"
	assert_count 1 cleanup "$calls"
	[[ -f "$evidence_root/non-promotion.seal" ]] || fail_test "missing non-promotion seal"
}

assert_post_patch_failure() {
	[[ "$run_status" != 0 ]] || fail_test "expected post-PATCH failure for ${active_scenario}"
	local restore_line cleanup_line
	restore_line="$(line_number restore "$calls")"
	cleanup_line="$(line_number cleanup "$calls")"
	[[ -n "$restore_line" && -n "$cleanup_line" && "$restore_line" -lt "$cleanup_line" ]] ||
		fail_test "restoration did not precede cleanup"
	[[ -f "$evidence_root/non-promotion.seal" ]] || fail_test "missing non-promotion seal"
}

test_preflight_has_no_detector_or_effects() {
	# Arrange
	prepare_case preflight

	# Act
	run_supervisor success preflight-only=true

	# Assert
	[[ "$run_status" == 0 ]] || fail_test "preflight failed"
	assert_count 1 package_admission "$calls"
	assert_count 0 detector "$calls"
	assert_absent "$calls" 'credential_path|flash_boot_a|validate_target|read_setting_|capture_|mutated_setting|patch|reboot|restore|validator'
	[[ -f "$evidence_root/preflight.seal" ]] || fail_test "preflight seal missing"
}

test_detector_failures_stop_all_later_commands() {
	local scenario
	for scenario in zero_candidates multiple_candidates board_info_failure wrong_board; do
		# Arrange
		prepare_case "$scenario"

		# Act
		run_supervisor "$scenario"

		# Assert
		assert_pre_patch_failure
		assert_detector_stopped_effects
	done
}

test_gate_one_drift_failures() {
	local scenario
	for scenario in source_drift reference_drift package_drift runtime_identity_drift; do
		# Arrange
		prepare_case "$scenario"

		# Act
		run_supervisor "$scenario"

		# Assert
		assert_pre_patch_failure
		assert_count 0 detector "$calls"
	done

	# Arrange
	prepare_case manifest_v3_drift 2
	# Act
	run_supervisor manifest_v3_drift
	# Assert
	assert_pre_patch_failure
	assert_count 0 detector "$calls"

	for scenario in executable_image_drift factory_image_drift; do
		# Arrange
		prepare_case "$scenario"
		if [[ "$scenario" == "executable_image_drift" ]]; then
			rm "$manifest_dir/firmware.elf"
		else
			rm "$manifest_dir/factory.bin"
		fi

		# Act
		run_supervisor "$scenario"

		# Assert
		assert_pre_patch_failure
		assert_count 0 detector "$calls"
	done
}

test_target_and_capture_failures_before_patch() {
	local scenario
	for scenario in stale_origin multiple_origins malformed_origin zero_byte_capture pre_patch_mismatch; do
		# Arrange
		prepare_case "$scenario"

		# Act
		run_supervisor "$scenario"

		# Assert
		assert_pre_patch_failure
		assert_count 1 detector "$calls"
		assert_count 1 credential_path "$calls"
	done
}

test_original_typed_failure_stops_before_mutation() {
	# Arrange
	prepare_case original_tcp_connection_failure

	# Act
	run_supervisor original_tcp_connection_failure

	# Assert
	assert_pre_patch_failure
	assert_line "$evidence_root/non-promotion.seal" 'category=tcp_connection_failure'
	assert_absent "$calls" 'capture_|mutated_setting|patch|reboot|restore|validator'
}

test_primary_survives_restoration_and_cleanup_failures() {
	# Arrange
	prepare_case primary_with_finalization_failures

	# Act
	run_supervisor primary_with_finalization_failures

	# Assert
	assert_post_patch_failure
	assert_line "$evidence_root/non-promotion.seal" 'category=invalid_json'
	assert_line "$evidence_root/non-promotion.seal" 'restoration_secondary_category=restoration_action_failed'
	assert_line "$evidence_root/non-promotion.seal" 'cleanup_secondary_category=cleanup_resource_failure'
}

test_finalization_only_failure_uses_supervisor_category() {
	# Arrange
	prepare_case finalization_only_failure

	# Act
	run_supervisor finalization_only_failure

	# Assert
	assert_post_patch_failure
	assert_line "$evidence_root/non-promotion.seal" 'category=supervisor_finalization_failed'
	assert_line "$evidence_root/non-promotion.seal" 'restoration_secondary_category=restoration_action_failed'
	assert_line "$evidence_root/non-promotion.seal" 'cleanup_secondary_category=cleanup_resource_failure'
}

test_timeout_floor_precedes_root_and_commands() {
	# Arrange
	prepare_case timeout_floor

	# Act
	run_supervisor success capture-timeout-seconds=359

	# Assert
	[[ "$run_status" != 0 ]] || fail_test "short timeout was accepted"
	[[ ! -e "$evidence_root" ]] || fail_test "short timeout created an evidence root"
	[[ ! -e "$calls" ]] || fail_test "short timeout invoked a fixture command"
	assert_contains "$case_dir/stderr.log" '^failure_category=capture_timeout_too_short$'
}

test_post_patch_failures_restore_then_cleanup() {
	local scenario
	local scenarios=(
		patch_not_committed
		immediate_storage_readback_mismatch
		reboot_before_response_readback
		missing_reboot
		additional_reboot
		wrong_reset_category
		boot_ordinal_mismatch
		same_board_identity_drift
		boot_b_value_mismatch
		current_head_recheck_failure
		reference_recheck_failure
		lifecycle_recheck_failure
		runtime_identity_recheck_failure
		no_actuation_recheck_failure
		restoration_failure
		cleanup_failure
		pid_leak
		holder_leak
		package_capability_drift
		detector_capability_drift
		root_contract_drift
		target_lock_drift
		broken_event_predecessor
		raw_field_redaction_failure
		validator_rejection
	)
	for scenario in "${scenarios[@]}"; do
		# Arrange
		prepare_case "$scenario"

		# Act
		run_supervisor "$scenario"

		# Assert
		assert_post_patch_failure
	done
}

test_success_ordering_and_private_root() {
	# Arrange
	prepare_case success

	# Act
	run_supervisor success

	# Assert
	[[ "$run_status" == 0 ]] || {
		sed -n '1,160p' "$case_dir/stderr.log" >&2
		fail_test "success scenario failed"
	}
	assert_count 1 package_admission "$calls"
	assert_count 1 detector "$calls"
	assert_count 1 physical_identity "$calls"
	assert_count 1 flash_boot_a "$calls"
	assert_count 1 read_setting_original "$calls"
	assert_count 1 read_setting_immediate "$calls"
	assert_count 1 read_setting_restoration "$calls"
	assert_count 1 capture_boot-a-pre "$calls"
	assert_count 1 capture_boot-a "$calls"
	assert_count 1 reboot "$calls"
	assert_count 1 capture_boot-b "$calls"
	assert_count 1 restore "$calls"
	assert_count 1 cleanup "$calls"
	assert_count 1 validator "$calls"
	assert_count 0 reboot_extra "$calls"

	local credential_line flash_line immediate_line reboot_line restore_line cleanup_line validator_line
	credential_line="$(line_number credential_path "$calls")"
	flash_line="$(line_number flash_boot_a "$calls")"
	immediate_line="$(line_number read_setting_immediate "$calls")"
	reboot_line="$(line_number reboot "$calls")"
	restore_line="$(line_number restore "$calls")"
	cleanup_line="$(line_number cleanup "$calls")"
	validator_line="$(line_number validator "$calls")"
	[[ "$credential_line" -lt "$flash_line" && "$immediate_line" -lt "$reboot_line" ]] ||
		fail_test "detector capability or immediate readback ordering failed"
	[[ "$reboot_line" -lt "$restore_line" && "$restore_line" -lt "$cleanup_line" && "$cleanup_line" -lt "$validator_line" ]] ||
		fail_test "reboot/restoration/cleanup/validator ordering failed"

	[[ "$(file_mode "$evidence_root")" == "700" ]] || fail_test "root mode is not 0700"
	while IFS= read -r file; do
		[[ "$(file_mode "$file")" == "600" ]] || fail_test "non-private file mode"
	done < <(find "$evidence_root" -type f)
	[[ ! -f "$evidence_root/non-promotion.seal" ]] || fail_test "success root was sealed non-promotion"
	[[ -f "$evidence_root/admitted.seal" ]] || fail_test "success root was not admitted"
	[[ "$(jq -r '.events | length' "$evidence_root/eligible.json")" == 9 ]] ||
		fail_test "event chain was incomplete"
	assert_contains "$case_dir/stdout.log" '^status=eligible$'
	assert_absent "$case_dir/stdout.log" 'fixture-target|fixture-device|fixture-setting'
	local original_ready_line patch_line
	original_ready_line="$(
		rg -n $'^2\toriginal_setting_ready\t' "$evidence_root/raw/chronology.tsv" |
			head -1 |
			cut -d: -f1
	)"
	patch_line="$(
		rg -n $'^4\tpatch_responded\t' "$evidence_root/raw/chronology.tsv" |
			head -1 |
			cut -d: -f1
	)"
	[[ -n "$original_ready_line" && -n "$patch_line" && "$original_ready_line" -lt "$patch_line" ]] ||
		fail_test "original ready checkpoint did not precede mutation"
}
