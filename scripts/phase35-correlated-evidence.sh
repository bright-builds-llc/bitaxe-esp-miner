#!/usr/bin/env bash
set -euo pipefail
# shellcheck disable=SC2034 # Sourced helper modules consume shared supervisor state.

readonly PHASE35_LIFECYCLE_ID="35-2026-07-17T17-00-37"
readonly PHASE35_SCHEMA="phase35-evidence-v1"
readonly MIN_CAPTURE_TIMEOUT_SECONDS=360
readonly MIN_CALLER_WALL_CLOCK_SECONDS=420
readonly PASSIVE_MONITOR_ARGS=(
	--chip esp32s3
	--before no-reset-no-sync
	--after no-reset
	--no-reset
	--non-interactive
)

resolve_script_dir() {
	local direct_dir
	direct_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
	if [[ -f "${direct_dir}/phase35-correlated-evidence-root.sh" ]]; then
		printf '%s\n' "$direct_dir"
		return 0
	fi

	local candidate
	for candidate in \
		"${BASH_SOURCE[0]}.runfiles/_main/scripts" \
		"${RUNFILES_DIR:-}/_main/scripts"; do
		if [[ -f "${candidate}/phase35-correlated-evidence-root.sh" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	printf 'failure_category=runfiles_incomplete\n' >&2
	return 1
}

script_dir="$(resolve_script_dir)" || exit 1
workspace_dir="${BUILD_WORKSPACE_DIRECTORY:-$(git rev-parse --show-toplevel)}"
manifest="${workspace_dir}/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
wifi_credentials=""
local_root=""
capture_timeout_seconds=360
caller_wall_clock_seconds=420
preflight_only=false
fixture_command="${PHASE35_FIXTURE_COMMAND:-}"
fixture_direct_flash="${PHASE35_FIXTURE_DIRECT_FLASH:-false}"

resolve_flash_executable() {
	local candidate
	for candidate in \
		"${workspace_dir}/bazel-bin/tools/flash/flash" \
		"${BASH_SOURCE[0]}.runfiles/_main/tools/flash/flash" \
		"${RUNFILES_DIR:-}/_main/tools/flash/flash"; do
		if [[ -x "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	printf 'failure_category=flash_executable_unavailable\n' >&2
	return 1
}

run_id_digest=""
root_contract_digest=""
package_capability_digest=""
detector_capability_digest=""
physical_identity_digest=""
target_lock_digest=""
port=""
target_token=""
original_setting=""
mutated_setting=""
mutation_started=0
restoration_complete=0
cleanup_complete=0
finalizer_ran=0
failure_category=""
event_sequence=0
event_predecessor=""
last_event_millis=0

usage() {
	printf 'usage: %s [preflight-only=true] [manifest=PATH] [wifi-credentials=PATH] [local-root=PATH] [capture-timeout-seconds=N] [caller-wall-clock-seconds=N]\n' "$(basename "$0")" >&2
	printf 'Full mode owns exactly one detector invocation. Callers must allow at least %s seconds wall clock.\n' "$MIN_CALLER_WALL_CLOCK_SECONDS" >&2
}

parse_bool() {
	case "$1" in
	true | 1 | yes | on) printf 'true\n' ;;
	false | 0 | no | off) printf 'false\n' ;;
	*) return 1 ;;
	esac
}

while (($#)); do
	case "$1" in
	preflight-only=* | preflight_only=*)
		preflight_only="$(parse_bool "${1#*=}")" || {
			usage
			exit 2
		}
		;;
	manifest=*) manifest="${1#*=}" ;;
	wifi-credentials=* | wifi_credentials=*) wifi_credentials="${1#*=}" ;;
	local-root=* | local_root=*) local_root="${1#*=}" ;;
	capture-timeout-seconds=* | capture_timeout_seconds=*) capture_timeout_seconds="${1#*=}" ;;
	caller-wall-clock-seconds=* | caller_wall_clock_seconds=*) caller_wall_clock_seconds="${1#*=}" ;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		usage
		exit 2
		;;
	esac
	shift
done

[[ "$capture_timeout_seconds" =~ ^[0-9]+$ ]] || {
	printf 'failure_category=invalid_capture_timeout\n' >&2
	exit 2
}
[[ "$caller_wall_clock_seconds" =~ ^[0-9]+$ ]] || {
	printf 'failure_category=invalid_caller_wall_clock\n' >&2
	exit 2
}
((capture_timeout_seconds >= MIN_CAPTURE_TIMEOUT_SECONDS)) || {
	printf 'failure_category=capture_timeout_too_short\n' >&2
	exit 2
}
((caller_wall_clock_seconds >= MIN_CALLER_WALL_CLOCK_SECONDS)) || {
	printf 'failure_category=caller_wall_clock_too_short\n' >&2
	exit 2
}
fixture_direct_flash="$(parse_bool "$fixture_direct_flash")" || {
	printf 'failure_category=invalid_fixture_direct_flash\n' >&2
	exit 2
}
if [[ "$fixture_direct_flash" == true && -z "$fixture_command" ]]; then
	printf 'failure_category=fixture_direct_flash_without_fixture\n' >&2
	exit 2
fi

# shellcheck source=scripts/phase35-correlated-evidence-root.sh
source "${script_dir}/phase35-correlated-evidence-root.sh"
# shellcheck source=scripts/phase35-correlated-evidence-effects.sh
source "${script_dir}/phase35-correlated-evidence-effects.sh"
# shellcheck source=scripts/phase35-correlated-evidence-document.sh
source "${script_dir}/phase35-correlated-evidence-document.sh"

main() {
	prepare_root
	trap 'on_exit' EXIT

	run_gate_one || fail "${failure_category:-gate_one_failed}"
	if [[ "$preflight_only" == true ]]; then
		write_private "$local_root/preflight.seal" \
			"status=preflight_only" \
			"effects_permitted=false"
		printf 'status=preflight_passed\n'
		printf 'exact_package_capability_digest=%s\n' "$package_capability_digest"
		printf 'current_head_equal=true\n'
		return 0
	fi

	run_detector_gate || fail "${failure_category:-detector_failed}"
	record_checkpoint root_admitted "$package_capability_digest"
	validate_credential_path_after_detector || fail "${failure_category:-credential_path_invalid}"

	run_flash_boot_a || fail "flash_or_boot_a_failed"
	target_token="$(jq -er '.target_token // .device_url' "$local_root/raw/boot-a-setup.json")" ||
		fail "target_missing"
	validate_target_after_capture || fail "target_invalid"
	original_setting="$(read_setting original)" || fail "original_setting_unavailable"

	local boot_a_pre
	boot_a_pre="$(capture_epoch boot-a-pre)" || fail "boot_a_pre_capture_failed"
	snapshot_setting_matches "$boot_a_pre" "$original_setting" || fail "pre_patch_mismatch"
	record_checkpoint boot_a_observed "$(sha256_file "$boot_a_pre")"

	if [[ -n "$fixture_command" ]]; then
		mutated_setting="$(fixture mutated_setting)"
	else
		mutated_setting="phase35-$(jq -r '.source_commit[0:8]' "$local_root/raw/exact-package-capability.json")-${RANDOM}"
	fi
	mutation_started=1
	patch_setting "$mutated_setting" || fail "patch_not_committed"
	record_checkpoint patch_responded "$(sha256_text "$mutated_setting")"
	local immediate
	immediate="$(read_setting immediate)" || fail "immediate_readback_missing"
	[[ "$immediate" == "$mutated_setting" ]] || fail "immediate_readback_mismatch"
	record_checkpoint storage_confirmed "$(sha256_text "$immediate")"

	local boot_a
	boot_a="$(capture_epoch boot-a)" || fail "boot_a_capture_failed"
	snapshot_setting_matches "$boot_a" "$mutated_setting" || fail "boot_a_value_mismatch"

	record_checkpoint reboot_started "$(hash_fields phase35-reboot-request-v1 true)"
	start_passive_monitor_and_reboot || fail "approved_reboot_failed"
	verify_same_identity || fail "physical_identity_drift"

	local boot_b
	boot_b="$(capture_epoch boot-b)" || fail "boot_b_capture_failed"
	[[ "$(jq -er '.reset_category' "$boot_b")" == "software_cpu" ]] || fail "wrong_reset_category"
	local expected_ordinal
	expected_ordinal="$(($(jq -er '.boot_ordinal' "$boot_a") + 1))"
	[[ "$(jq -er '.boot_ordinal' "$boot_b")" == "$expected_ordinal" ]] || fail "boot_ordinal_mismatch"
	snapshot_setting_matches "$boot_b" "$mutated_setting" || fail "boot_b_value_mismatch"
	record_checkpoint boot_b_observed "$(sha256_file "$boot_b")"

	write_private "$local_root/artifacts/no-actuation.txt" "no_actuation_verified=true"
	record_checkpoint no_actuation_verified "$(sha256_file "$local_root/artifacts/no-actuation.txt")"
	restore_setting_once || fail "restoration_failed"
	cleanup_resources_once || fail "cleanup_failed"
	run_live_rechecks || fail "${failure_category:-live_recheck_failed}"

	local setting_digest
	setting_digest="$(sha256_text "$mutated_setting")"
	build_evidence_root "$boot_a" "$boot_b" "$setting_digest" || fail "evidence_root_build_failed"
	run_validator || fail "${failure_category:-validator_rejected}"
	write_private "$local_root/admitted.seal" \
		"status=eligible_for_downstream_admission" \
		"root_digest=${root_contract_digest}"
	printf 'status=eligible\n'
	printf 'root_digest=%s\n' "$root_contract_digest"
	printf 'result_digest=%s\n' "$(sha256_file "$local_root/result.json")"
}

main "$@"
