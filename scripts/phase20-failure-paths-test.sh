#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE20_FAILURE_PATHS_SCRIPT:-${script_dir}/phase20-failure-paths.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase20-failure-paths-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

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

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_no_active_command_bin() {
	local bin_dir="$1"
	local command_log="$2"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/curl" 'printf "curl %s\n" "$*" >>"${PHASE20_COMMAND_LOG:?}"
exit 97
'
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PHASE20_COMMAND_LOG:?}"
exit 98
'
	write_executable "${bin_dir}/just" 'printf "just %s\n" "$*" >>"${PHASE20_COMMAND_LOG:?}"
exit 99
'
	write_executable "${bin_dir}/i2cset" 'printf "i2cset %s\n" "$*" >>"${PHASE20_COMMAND_LOG:?}"
exit 96
'
	write_executable "${bin_dir}/stress" 'printf "stress %s\n" "$*" >>"${PHASE20_COMMAND_LOG:?}"
exit 95
'
	: >"$command_log"
}

write_fake_allow() {
	local path="$1"
	local args_log="$2"

	write_executable "$path" 'printf "%s\n" "$*" >"${PHASE20_ALLOW_ARGS_LOG:?}"
status="${PHASE20_FAKE_ALLOW_STATUS:-passed}"
case "$status" in
passed)
	printf "safety_allow_status: passed\n"
	printf "surface: failure-paths\n"
	printf "claim_tier: unsupported-pending\n"
	printf "evidence_class: deferred\n"
	;;
failed)
	printf "safety_allow_status: failed\n"
	printf "validation_errors:\n- fake failure\n"
	exit 42
	;;
*)
	printf "unknown fake status\n" >&2
	exit 2
	;;
esac
'
	: >"$args_log"
}

write_manifest() {
	local path="$1"

	printf '{"surface":"failure-paths","claim_tier":"unsupported-pending"}\n' >"$path"
}

assert_no_active_commands() {
	local command_log="$1"

	if [[ -s "$command_log" ]]; then
		printf 'Active command log should be empty. Actual content:\n%s\n' "$(cat "$command_log")" >&2
		exit 1
	fi
}

test_missing_manifest_blocks_without_active_commands() {
	local out_dir="${tmp_root}/missing-manifest"
	local bin_dir="${tmp_root}/bin-missing"
	local command_log="${tmp_root}/commands-missing.log"

	create_no_active_command_bin "$bin_dir" "$command_log"

	PATH="${bin_dir}:$PATH" PHASE20_COMMAND_LOG="$command_log" "$BASH" "$wrapper" \
		--manifest "${tmp_root}/missing.json" \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/failure-paths.log" "phase20_failure_paths"
	assert_contains "${out_dir}/failure-paths.log" "failure_paths_status: blocked - missing manifest"
	assert_contains "${out_dir}/failure-paths.log" "fault_stimulus_status: not_run"
	assert_no_active_commands "$command_log"
}

test_failed_safety_allow_validation_blocks() {
	local out_dir="${tmp_root}/failed-validation"
	local bin_dir="${tmp_root}/bin-failed"
	local command_log="${tmp_root}/commands-failed.log"
	local manifest="${tmp_root}/manifest-failed.json"
	local fake_allow="${tmp_root}/fake-allow-failed"
	local allow_args="${tmp_root}/allow-failed-args.log"

	create_no_active_command_bin "$bin_dir" "$command_log"
	write_manifest "$manifest"
	write_fake_allow "$fake_allow" "$allow_args"

	PATH="${bin_dir}:$PATH" PHASE20_COMMAND_LOG="$command_log" PHASE20_ALLOW_ARGS_LOG="$allow_args" PHASE20_FAKE_ALLOW_STATUS=failed PHASE20_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir"

	assert_contains "${out_dir}/failure-paths.log" "safety_allow_status: failed"
	assert_contains "${out_dir}/failure-paths.log" "failure_paths_status: blocked - allow validation failed"
	assert_contains "${out_dir}/failure-paths.log" "fault_stimulus_status: not_run"
	assert_no_active_commands "$command_log"
}

test_unsupported_pending_manifest_records_required_fields() {
	local out_dir="${tmp_root}/unsupported-pending"
	local bin_dir="${tmp_root}/bin-unsupported"
	local command_log="${tmp_root}/commands-unsupported.log"
	local manifest="${tmp_root}/manifest-unsupported.json"
	local fake_allow="${tmp_root}/fake-allow-unsupported"
	local allow_args="${tmp_root}/allow-unsupported-args.log"

	create_no_active_command_bin "$bin_dir" "$command_log"
	write_manifest "$manifest"
	write_fake_allow "$fake_allow" "$allow_args"

	PATH="${bin_dir}:$PATH" PHASE20_COMMAND_LOG="$command_log" PHASE20_ALLOW_ARGS_LOG="$allow_args" PHASE20_SAFETY_ALLOW_BIN="$fake_allow" "$BASH" "$wrapper" \
		--manifest "$manifest" \
		--out-dir "$out_dir"

	assert_contains "$allow_args" "--surface failure-paths"
	assert_contains "$allow_args" "scripts/phase20-failure-paths.sh --manifest ${manifest} --out-dir ${out_dir}"
	assert_contains "${out_dir}/failure-paths.log" "phase20_failure_paths"
	assert_contains "${out_dir}/failure-paths.log" "failure_paths_status: blocked - no production-safe fault stimulus route"
	assert_contains "${out_dir}/failure-paths.log" "fault_stimulus_status: not_run"
	assert_contains "${out_dir}/failure-paths.log" "expected_fault_status: not_observed"
	assert_contains "${out_dir}/failure-paths.log" "api_projection_status: not_run"
	assert_contains "${out_dir}/failure-paths.log" "websocket_projection_status: not_run"
	assert_contains "${out_dir}/failure-paths.log" "final_safe_state_status: required-before-promotion"
	assert_contains "${out_dir}/failure-paths.log" "required_stimulus: missing - future plan must name bounded stimulus"
	assert_contains "${out_dir}/failure-paths.log" "required_expected_fault: missing - future plan must name expected fault"
	assert_contains "${out_dir}/failure-paths.log" "required_recovery_path: missing - future plan must name restore path"
	assert_contains "${out_dir}/failure-paths.log" "required_final_safe_state_marker: missing - future plan must observe final safe-state marker"
	assert_contains "${out_dir}/failure-paths.log" "active_rows_status: below_verified"
	assert_contains "${out_dir}/failure-paths.log" "checklist_rows: PWR-001,PWR-002,THR-001,THR-002,SELF-001,SAFE-04"
	assert_contains "${out_dir}/failure-paths.log" "non_claims: overheat stimulus, fan fault stimulus, power fault stimulus, thermal fault stimulus, ASIC fault stimulus"
	assert_not_contains "${out_dir}/failure-paths.log" "fault_stimulus_status: verified"
	assert_no_active_commands "$command_log"
}

if [[ ! -f "$wrapper" ]]; then
	fail "wrapper script missing: ${wrapper}"
fi

test_missing_manifest_blocks_without_active_commands
test_failed_safety_allow_validation_blocks
test_unsupported_pending_manifest_records_required_fields

printf 'phase20_failure_paths_test passed\n'
