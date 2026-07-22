#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE35_TEST_STUB_DISPATCH:-false}" == true ]]; then
	case "${0##*/}" in
	flash)
		subcommand="${1:-}"
		case "$subcommand" in
		flash-monitor)
			printf 'CALL\n' >>"${PHASE35_DIRECT_FLASH_CALLS:?}"
			printf 'arg=%s\n' "$@" >>"${PHASE35_DIRECT_FLASH_CALLS:?}"
			printf 'direct_flash\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
			;;
		finalize-evidence)
			printf 'CALL\n' >>"${PHASE35_FINALIZER_CALLS:?}"
			printf 'arg=%s\n' "$@" >>"${PHASE35_FINALIZER_CALLS:?}"
			printf 'finalize_evidence\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
			;;
		*) exit 98 ;;
		esac
		evidence_dir=""
		expected_private_sha256=""
		while (($#)); do
			if [[ "$1" == "--evidence-dir" ]]; then
				evidence_dir="$2"
				shift 2
				continue
			fi
			if [[ "$1" == "--expected-private-sha256" ]]; then
				expected_private_sha256="$2"
				shift 2
				continue
			fi
			shift
		done
		[[ -n "$evidence_dir" ]]
		if [[ "$subcommand" == flash-monitor ]]; then
			mkdir -p "$evidence_dir"
			chmod 700 "$evidence_dir"
			if [[ "${PHASE35_TEST_PRIVATE_INPUT:?}" == valid ]]; then
				printf 'device_%s=%s%s password=[redacted]\n' url http '://fixture-target' \
					>"${evidence_dir}/flash-monitor.classifier-input.log"
			else
				printf 'fixture-monitor-without-origin\n' \
					>"${evidence_dir}/flash-monitor.classifier-input.log"
			fi
			private_digest="$(shasum -a 256 \
				"${evidence_dir}/flash-monitor.classifier-input.log" | awk '{print $1}')"
			jq -cn \
				--arg private_monitor_log_sha256 "$private_digest" \
				'{redaction_mode:"dual",commit_ready:false,private_monitor_log_sha256:$private_monitor_log_sha256}' \
				>"${evidence_dir}/flash-command-evidence.private.json"
			chmod 600 \
				"${evidence_dir}/flash-monitor.classifier-input.log" \
				"${evidence_dir}/flash-command-evidence.private.json"
			exit 0
		fi
		[[ -n "$expected_private_sha256" ]]
		[[ ! -e "${evidence_dir}/flash-monitor.log" ]]
		[[ ! -e "${evidence_dir}/flash-command-evidence.json" ]]
		actual_private_sha256="$(shasum -a 256 \
			"${evidence_dir}/flash-monitor.classifier-input.log" | awk '{print $1}')"
		[[ "$actual_private_sha256" == "$expected_private_sha256" ]]
		printf 'device_url=[redacted-url] password=[redacted]\n' \
			>"${evidence_dir}/flash-monitor.log"
		jq -cn \
			--arg monitor_log_sha256 "$(shasum -a 256 \
				"${evidence_dir}/flash-monitor.log" | awk '{print $1}')" \
			'{redaction_mode:"dual",commit_ready:true,monitor_log_path:"flash-monitor.log",monitor_log_sha256:$monitor_log_sha256}' \
			>"${evidence_dir}/flash-command-evidence.json"
		chmod 600 \
			"${evidence_dir}/flash-monitor.log" \
			"${evidence_dir}/flash-command-evidence.json"
		;;
	report)
		printf 'CALL\n' >>"${PHASE35_CLASSIFIER_CALLS:?}"
		printf 'arg=%s\n' "$@" >>"${PHASE35_CLASSIFIER_CALLS:?}"
		trace=""
		mode=""
		while (($#)); do
			if [[ "$1" == "--trace" ]]; then
				trace="$2"
				shift 2
				continue
			fi
			if [[ "$1" == "--mode" ]]; then
				mode="$2"
				shift 2
				continue
			fi
			shift
		done
		printf 'trace=%s\n' "$trace" >>"${PHASE35_CLASSIFIER_CALLS:?}"
		printf 'classifier\n' >>"${PHASE35_FIXTURE_STATE:?}/calls.log"
		if [[ "${trace##*/}" == flash-monitor.classifier-input.log ]]; then
			[[ ! -e "$(dirname "$trace")/flash-monitor.log" ]]
			[[ ! -e "$(dirname "$trace")/flash-command-evidence.json" ]]
		fi
		case "${PHASE35_TEST_PARITY_OUTCOME:?}" in
		passed)
			private_origin_pattern="$(printf 'device_%s=%s%s' url http '://fixture-target')"
			if [[ "$mode" == post-restart ]]; then
				jq -cn '{status:"passed",category:"none",session:"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",boot_ordinal:8,device_url:"fixture-target"}'
			elif [[ "${trace##*/}" == flash-monitor.classifier-input.log ]] &&
				rg -Fq "$private_origin_pattern" "$trace"; then
				jq -cn '{status:"passed",category:"none",session:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",boot_ordinal:7,device_url:"fixture-target"}'
			else
				jq -cn '{status:"failed",category:"baseline_origin_missing",session:null,boot_ordinal:null,device_url:null}'
			fi
			;;
		rejected)
			jq -cn '{status:"failed",category:"baseline_multiple_sessions",session:null,boot_ordinal:null,device_url:null}'
			;;
		*)
			exit 98
			;;
		esac
		;;
	just | bazel)
		printf '%s\n' "${0##*/}" >>"${PHASE35_NESTED_TOOL_CALLS:?}"
		exit 97
		;;
	*)
		exit 98
		;;
	esac
	exit 0
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly test_entrypoint="${script_dir}/phase35-correlated-evidence-test.sh"
readonly supervisor="${script_dir}/phase35-correlated-evidence.sh"
readonly fixture="${script_dir}/phase35-correlated-evidence-fixture.sh"
readonly justfile="${script_dir}/../Justfile"
readonly sdkconfig_defaults="${script_dir}/../firmware/bitaxe/sdkconfig.defaults"
readonly test_root="${TEST_TMPDIR:-$(mktemp -d)}/phase35"
readonly workspace="${test_root}/workspace"
readonly source_commit="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
readonly reference_commit="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
readonly minimum_main_task_stack_bytes=16384
active_scenario=""

mkdir -p "$workspace"

fail_test() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local file="$1"
	local pattern="$2"
	rg -q "$pattern" "$file" || fail_test "expected ${pattern} in ${file##*/}"
}

assert_line() {
	local file="$1"
	local expected="$2"
	rg -Fqx -- "$expected" "$file" || fail_test "expected exact line in ${file##*/}: ${expected}"
}

assert_absent() {
	local file="$1"
	local pattern="$2"
	[[ ! -f "$file" ]] || ! rg -q "$pattern" "$file" ||
		fail_test "unexpected ${pattern} in ${file##*/}"
}

assert_count() {
	local expected="$1"
	local pattern="$2"
	local file="$3"
	local actual=0
	if [[ -f "$file" ]]; then
		actual="$(rg -c "^${pattern}$" "$file" || printf '0')"
	fi
	[[ "$actual" == "$expected" ]] ||
		fail_test "expected ${expected} ${pattern} calls, found ${actual}"
}

line_number() {
	local pattern="$1"
	local file="$2"
	rg -n "^${pattern}$" "$file" | head -1 | cut -d: -f1
}

file_mode() {
	local file="$1"
	stat -f '%Lp' "$file" 2>/dev/null || stat -c '%a' "$file"
}

path_metadata() {
	local path="$1"
	stat -f '%HT:%Lp:%z:%m:%c' "$path" 2>/dev/null ||
		stat -c '%F:%a:%s:%Y:%Z' "$path"
}

file_digest() {
	shasum -a 256 "$1" | awk '{print $1}'
}

text_digest() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

test_main_task_stack_capacity() {
	# Arrange
	local assignment_count
	assignment_count="$(rg -c '^CONFIG_ESP_MAIN_TASK_STACK_SIZE=[0-9]+$' "$sdkconfig_defaults")"
	[[ "$assignment_count" == 1 ]] ||
		fail_test "expected one ESP main-task stack assignment"
	local configured_stack_bytes
	configured_stack_bytes="$(
		sed -n 's/^CONFIG_ESP_MAIN_TASK_STACK_SIZE=//p' "$sdkconfig_defaults"
	)"

	# Act
	local capacity_is_sufficient=false
	if [[ "$configured_stack_bytes" =~ ^[0-9]+$ ]] &&
		((configured_stack_bytes >= minimum_main_task_stack_bytes)); then
		capacity_is_sufficient=true
	fi

	# Assert
	[[ "$capacity_is_sufficient" == true ]] ||
		fail_test "ESP main-task stack is below the Phase 35 runtime minimum"
}

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
	supervisor_path="$PATH"
	supervisor_stdout="${case_dir}/stdout.log"
	supervisor_stderr="${case_dir}/stderr.log"
	direct_flash_calls="${state_dir}/direct-flash-calls.log"
	finalizer_calls="${state_dir}/finalizer-calls.log"
	classifier_calls="${state_dir}/classifier-calls.log"
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
		"${script_dir}/phase35-correlated-evidence-root.sh" \
		"${script_dir}/phase35-correlated-evidence-effects.sh" \
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
			PHASE35_NESTED_TOOL_CALLS="$nested_tool_calls" \
			PHASE35_TEST_PRIVATE_INPUT="$stub_private_input" \
			PHASE35_TEST_PARITY_OUTCOME="$stub_parity_outcome" \
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
	local blocked_bin="${case_dir}/blocked-bin"
	mkdir -p "$(dirname "$flash_bin")" "$(dirname "$parity_bin")" "$blocked_bin"
	rm -f "$flash_bin" "$parity_bin"
	ln -s "$test_entrypoint" "$flash_bin"
	ln -s "$test_entrypoint" "$parity_bin"
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

test_runfiles_invokes_direct_flash_once_without_nested_build_tools() {
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
	assert_count 1 credential_path "$calls"
	assert_count 1 direct_flash "$calls"
	assert_count 1 finalize_evidence "$calls"
	assert_count 1 CALL "$direct_flash_calls"
	assert_count 1 CALL "$finalizer_calls"
	[[ "$(rg -c '^arg=' "$direct_flash_calls")" == 15 ]] ||
		fail_test "direct flash received unexpected or missing arguments"
	[[ ! -s "$nested_tool_calls" ]] || fail_test "direct flash path invoked nested just or Bazel"
	assert_line "$direct_flash_calls" 'arg=flash-monitor'
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

run_production_capture_case() {
	local scenario="$1"
	local capture_root="${test_root}/production-capture-${scenario}"
	rm -rf "$capture_root"
	mkdir -p "$capture_root/raw"
	chmod 700 "$capture_root" "$capture_root/raw"
	printf '%s\n' 1000 >"$capture_root/raw/monotonic-state"
	chmod 600 "$capture_root/raw/monotonic-state"
	jq -cn \
		--arg session 0123456789abcdef0011223344556677 \
		'{status:"passed",category:"none",session:$session,boot_ordinal:51}' \
		>"$capture_root/raw/boot-a-setup.json"
	chmod 600 "$capture_root/raw/boot-a-setup.json"

	(
		local fixture_command=""
		local workspace_dir="$workspace"
		local local_root="$capture_root"
		local target_token=synthetic-origin
		local failure_category=""
		local capture_scenario="$scenario"

		# shellcheck source=scripts/phase35-correlated-evidence-root.sh
		source "${script_dir}/phase35-correlated-evidence-root.sh"
		# shellcheck source=scripts/phase35-correlated-evidence-effects.sh
		source "${script_dir}/phase35-correlated-evidence-effects.sh"

		monotonic_millis() {
			local state_file="$local_root/raw/monotonic-state"
			local current
			current="$(<"$state_file")"
			printf '%s\n' "$((current + 100))" >"$state_file"
			printf '%s\n' "$current"
		}

		curl() {
			local output=""
			local endpoint=""
			local http1=false
			local no_proxy=""
			local proto=""
			local connect_timeout=""
			local max_time=""
			local max_filesize=""
			while (($#)); do
				case "$1" in
				--output)
					output="$2"
					shift 2
					;;
				--http1.1)
					http1=true
					shift
					;;
				--noproxy)
					no_proxy="$2"
					shift 2
					;;
				--proto)
					proto="$2"
					shift 2
					;;
				--connect-timeout)
					connect_timeout="$2"
					shift 2
					;;
				--max-time)
					max_time="$2"
					shift 2
					;;
				--max-filesize)
					max_filesize="$2"
					shift 2
					;;
				--silent | --show-error | --fail)
					shift
					;;
				*)
					endpoint="$1"
					shift
					;;
				esac
			done
			[[ "$http1" == true && "$no_proxy" == '*' && "$proto" == '=http,https' ]] ||
				return 92
			[[ "$connect_timeout" == 5 && "$max_time" == 10 ]] || return 92
			case "$endpoint" in
			*/api/system/info)
				[[ "$max_filesize" == 65536 ]] || return 92
				if [[ "$capture_scenario" == missing_hostname ]]; then
					jq -cn \
						--arg bootSession 0123456789abcdef0011223344556677 \
						'{bootSession:$bootSession,operatorSnapshotRevision:20}' >"$output"
				else
					jq -cn \
						--arg bootSession 0123456789abcdef0011223344556677 \
						'{bootSession:$bootSession,operatorSnapshotRevision:20,hostname:"synthetic-host"}' >"$output"
				fi
				;;
			*/api/system/logs)
				[[ "$max_filesize" == 524288 ]] || return 92
				printf '%s\n' \
					'operator_snapshot session=0123456789abcdef0011223344556677 revision=20 redacted=true' \
					>"$output"
				if [[ "$capture_scenario" != missing_websocket_marker ]]; then
					printf '%s\n' \
						'operator_snapshot session=0123456789abcdef0011223344556677 revision=21 redacted=true' \
						>>"$output"
				fi
				;;
			*) return 91 ;;
			esac
		}

		node() {
			local output=""
			while (($#)); do
				if [[ "$1" == --out ]]; then
					output="$2"
					shift 2
					continue
				fi
				shift
			done
			local revision=21
			[[ "$capture_scenario" != same_revision ]] || revision=20
			jq -cn \
				--arg session 0123456789abcdef0011223344556677 \
				--argjson revision "$revision" \
				'{event:"system_info",data:{bootSession:$session,operatorSnapshotRevision:$revision}}' |
				sed 's/^/websocket_frame_1=/' >"$output"
			printf 'websocket_close_status=closed\n' >>"$output"
		}

		capture_epoch boot-a-pre
	)
}

test_production_capture_preserves_real_epoch_boundaries() {
	# Arrange and Act
	local snapshot
	snapshot="$(run_production_capture_case success)" ||
		fail_test "production capture fixture failed"

	# Assert
	[[ "$(jq -r '.boot_ordinal' "$snapshot")" == 51 ]] ||
		fail_test "capture did not use the serial-classified boot ordinal"
	[[ "$(jq -r '.storage_revision' "$snapshot")" == 20 ]] ||
		fail_test "capture did not retain the API storage revision"
	[[ "$(jq -r '.setting_digest' "$snapshot")" == "$(text_digest synthetic-host)" ]] ||
		fail_test "capture did not hash the validated private setting"
	assert_contains "$snapshot" 'operatorSnapshotRevision\\\":21'
	assert_absent "$snapshot" 'live_websocket_json: .*event'
	assert_line "${test_root}/production-capture-success/raw/boot-a-pre-retained.log" \
		'operator_snapshot session=0123456789abcdef0011223344556677 revision=20 redacted=true'
	assert_line "${test_root}/production-capture-success/raw/boot-a-pre-retained.log" \
		'operator_snapshot session=0123456789abcdef0011223344556677 revision=21 redacted=true'
	(("$(jq -r '.ended_millis' "$snapshot")" > "$(jq -r '.started_millis' "$snapshot")")) ||
		fail_test "capture did not preserve a positive real interval"
	local protected_file
	for protected_file in \
		"${test_root}/production-capture-success/raw/boot-a-pre-api.json" \
		"${test_root}/production-capture-success/raw/boot-a-pre-api.stderr" \
		"${test_root}/production-capture-success/raw/boot-a-pre-websocket.log" \
		"${test_root}/production-capture-success/raw/boot-a-pre-websocket.stderr" \
		"${test_root}/production-capture-success/raw/boot-a-pre-retained.log" \
		"${test_root}/production-capture-success/raw/boot-a-pre-retained.stderr"; do
		[[ "$(file_mode "$protected_file")" == 600 ]] ||
			fail_test "production capture artifact is not mode 0600"
	done
}

test_production_capture_rejects_incoherent_boundaries() {
	local scenario
	for scenario in same_revision missing_websocket_marker missing_hostname; do
		# Arrange and Act
		local output="${test_root}/${scenario}.stdout"
		local error="${test_root}/${scenario}.stderr"
		set +e
		run_production_capture_case "$scenario" >"$output" 2>"$error"
		local result_code=$?
		set -e

		# Assert
		[[ "$result_code" != 0 ]] ||
			fail_test "${scenario} production capture was accepted"
		[[ ! -s "$output" && ! -s "$error" ]] ||
			fail_test "${scenario} production failure exposed private diagnostics"
		[[ ! -e "${test_root}/production-capture-${scenario}/raw/boot-a-pre.json" ]] ||
			fail_test "${scenario} production failure created a usable snapshot"
	done
}

test_production_reboot_classifies_only_post_loss_bytes() {
	# Arrange
	prepare_case production_reboot
	prepare_direct_flash_stubs
	local reboot_root="${case_dir}/reboot-root"
	mkdir -p "$reboot_root/raw"
	chmod 700 "$reboot_root" "$reboot_root/raw"
	jq -cn \
		--arg session aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
		'{status:"passed",category:"none",session:$session,boot_ordinal:7,device_url:"synthetic-origin"}' \
		>"$reboot_root/raw/boot-a-setup.json"
	chmod 600 "$reboot_root/raw/boot-a-setup.json"
	local baseline_bytes
	baseline_bytes="$(printf 'baseline-before-restart\n' | wc -c | tr -d ' ')"

	# Act
	(
		export PHASE35_FIXTURE_STATE="$state_dir"
		export PHASE35_CLASSIFIER_CALLS="$classifier_calls"
		export PHASE35_TEST_PARITY_OUTCOME=passed
		export PHASE35_TEST_PRIVATE_INPUT=valid
		export PHASE35_TEST_STUB_DISPATCH=true
		local fixture_command=""
		local workspace_dir="$workspace"
		local local_root="$reboot_root"
		local target_token=synthetic-origin
		local port=synthetic-port
		local capture_timeout_seconds=360
		local passive_monitor_pid=""
		local failure_category=""
		local -a PASSIVE_MONITOR_ARGS=(
			--chip esp32s3
			--before no-reset-no-sync
			--after no-reset
			--no-reset
			--non-interactive
		)

		# shellcheck source=scripts/phase35-correlated-evidence-root.sh
		source "${script_dir}/phase35-correlated-evidence-root.sh"
		# shellcheck source=scripts/phase35-correlated-evidence-effects.sh
		source "${script_dir}/phase35-correlated-evidence-effects.sh"

		bash() {
			local monitor_out=""
			local raw_out=""
			while (($#)); do
				case "$1" in
				--out)
					monitor_out="$2"
					shift 2
					;;
				--raw-out)
					raw_out="$2"
					shift 2
					;;
				*) shift ;;
				esac
			done
			printf 'baseline-before-restart\n' >"$raw_out"
			: >"$monitor_out"
			printf 'ready\n' >"${PHASE13_MONITOR_ACTIVE_READY_FILE:?}"
		}

		curl() {
			local output=""
			local method=GET
			while (($#)); do
				case "$1" in
				--output)
					output="$2"
					shift 2
					;;
				--request)
					method="$2"
					shift 2
					;;
				*) shift ;;
				esac
			done
			if [[ "$method" == POST ]]; then
				printf '{}\n' >"$output"
				return 0
			fi
			return 7
		}

		start_passive_monitor_and_reboot
		[[ "$target_token" == fixture-target ]]
	)

	# Assert
	assert_line "$classifier_calls" 'arg=phase33-classify'
	assert_line "$classifier_calls" 'arg=--mode'
	assert_line "$classifier_calls" 'arg=post-restart'
	assert_line "$classifier_calls" 'arg=--start-byte'
	assert_line "$classifier_calls" "arg=${baseline_bytes}"
	assert_line "$classifier_calls" 'arg=--expected-session'
	assert_line "$classifier_calls" 'arg=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
	assert_line "$classifier_calls" 'arg=--expected-ordinal'
	assert_line "$classifier_calls" 'arg=7'
	[[ "$(file_mode "$reboot_root/raw/boot-b-setup.json")" == 600 ]] ||
		fail_test "post-restart classification is not mode 0600"
	[[ "$(file_mode "$reboot_root/raw/boot-b-setup.stderr")" == 600 ]] ||
		fail_test "post-restart classifier diagnostics are not mode 0600"
}

test_just_entrypoint_builds_the_current_package_before_supervisor() {
	# Arrange
	local expected_recipe
	expected_recipe=$'phase35-evidence *args:\n    bazel build //firmware/bitaxe:firmware_image\n    bazel run //scripts:phase35_correlated_evidence -- {{ args }}'

	# Act
	local actual_recipe
	actual_recipe="$(awk '
		/^phase35-evidence \*args:$/ { capture = 1 }
		capture && /^[^[:space:]]/ && $0 !~ /^phase35-evidence \*args:$/ { exit }
		capture { print }
	' "$justfile")"

	# Assert
	[[ "$actual_recipe" == "$expected_recipe" ]] ||
		fail_test "phase35-evidence did not build the exact current package first"
}

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

test_main_task_stack_capacity
test_runfiles_rejects_existing_child_before_admission_or_effects
test_runfiles_preserves_caller_owned_parent_and_sibling_outputs
test_runfiles_entrypoint_resolves_sibling_helpers
test_runfiles_resolves_repo_root_credential_only_after_detector
test_runfiles_invokes_direct_flash_once_without_nested_build_tools
test_commit_redacted_copy_reproduces_attempt_11_origin_loss
test_invalid_private_classifier_input_stops_before_mutation
test_direct_flash_classifier_rejection_preserves_typed_category
test_production_capture_preserves_real_epoch_boundaries
test_production_capture_rejects_incoherent_boundaries
test_production_reboot_classifies_only_post_loss_bytes
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
