#!/usr/bin/env bash
set -euo pipefail

resolve_phase35_test_script_dir() {
	local direct_dir
	direct_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

	local candidate_dir
	for candidate_dir in \
		"$direct_dir" \
		"${BASH_SOURCE[0]}.runfiles/_main/scripts" \
		"${RUNFILES_DIR:-}/_main/scripts"; do
		if [[ -f "${candidate_dir}/phase35-correlated-evidence-test-stub.sh" ]] &&
			[[ -f "${candidate_dir}/phase35-correlated-evidence-test-support.sh" ]] &&
			[[ -f "${candidate_dir}/phase35-correlated-evidence-test-runfiles.sh" ]] &&
			[[ -f "${candidate_dir}/phase35-correlated-evidence-test-runtime.sh" ]] &&
			[[ -f "${candidate_dir}/phase35-correlated-evidence-test-lifecycle.sh" ]]; then
			printf '%s\n' "$candidate_dir"
			return 0
		fi
	done

	printf 'FAIL: Phase 35 test helpers are unavailable\n' >&2
	return 1
}

script_dir="$(resolve_phase35_test_script_dir)" || exit 1
readonly script_dir
if [[ "${PHASE35_TEST_STUB_DISPATCH:-false}" == true ]]; then
	# shellcheck source=scripts/phase35-correlated-evidence-test-stub.sh
	source "${script_dir}/phase35-correlated-evidence-test-stub.sh"
fi

# Resolve every test module before support setup creates temporary state.
readonly phase35_test_support="${script_dir}/phase35-correlated-evidence-test-support.sh"
readonly phase35_test_runfiles="${script_dir}/phase35-correlated-evidence-test-runfiles.sh"
readonly phase35_test_runtime="${script_dir}/phase35-correlated-evidence-test-runtime.sh"
readonly phase35_test_lifecycle="${script_dir}/phase35-correlated-evidence-test-lifecycle.sh"
# shellcheck source=scripts/phase35-correlated-evidence-test-support.sh
source "$phase35_test_support"
# shellcheck source=scripts/phase35-correlated-evidence-test-runfiles.sh
source "$phase35_test_runfiles"
# shellcheck source=scripts/phase35-correlated-evidence-test-runtime.sh
source "$phase35_test_runtime"
# shellcheck source=scripts/phase35-correlated-evidence-test-lifecycle.sh
source "$phase35_test_lifecycle"

test_main_task_stack_capacity
test_websocket_background_writes_are_serialized_in_httpd_context
test_websocket_routes_consume_control_frames
test_runfiles_rejects_existing_child_before_admission_or_effects
test_runfiles_preserves_caller_owned_parent_and_sibling_outputs
test_runfiles_entrypoint_resolves_sibling_helpers
test_built_supervisor_runfiles_select_device_session_runtime_observer
test_runfiles_resolves_repo_root_credential_only_after_detector
test_runfiles_invokes_probe_then_flash_without_nested_build_tools
test_probe_failure_stops_before_credentials_or_writes
test_probe_identity_drift_stops_before_credentials_or_writes
test_commit_redacted_copy_reproduces_attempt_11_origin_loss
test_invalid_private_classifier_input_stops_before_mutation
test_direct_flash_classifier_rejection_preserves_typed_category
test_real_process_factory_post_info_failure_is_typed
test_production_capture_preserves_real_epoch_boundaries
test_production_capture_rejects_incoherent_boundaries
test_production_reboot_uses_device_session_hybrid_quorum
test_just_entrypoint_builds_the_current_package_before_supervisor
test_preflight_has_no_detector_or_effects
test_detector_failures_stop_all_later_commands
test_gate_one_drift_failures
test_target_and_capture_failures_before_patch
test_original_typed_failure_stops_before_mutation
test_primary_survives_restoration_and_cleanup_failures
test_finalization_only_failure_uses_supervisor_category
test_timeout_floor_precedes_root_and_commands
test_post_patch_failures_restore_then_cleanup
test_success_ordering_and_private_root

printf 'phase35 correlated evidence tests passed\n'
