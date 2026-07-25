#!/usr/bin/env bash
set -euo pipefail

readonly supervisor="${1:?missing deployed supervisor}"
readonly report_binary="${2:?missing deployed report binary}"
readonly deployed_effect_adapter="$(dirname "$supervisor")/phase36_hardware_effect"
readonly agents_file="${3:?missing workspace marker}"
readonly justfile="${4:?missing Justfile}"
readonly supervisor_source="${5:?missing supervisor source}"
readonly broker_source="${6:?missing broker source}"
readonly capture_source="${7:?missing capture source}"
readonly hardware_broker_source="${8:?missing hardware broker source}"
readonly deployed_flash="${9:?missing deployed flash binary}"
readonly agents_real="$(perl -MCwd=realpath -e 'print realpath($ARGV[0])' "$agents_file")"
readonly workspace_root="$(dirname "$agents_real")"
readonly phase_dir="${workspace_root}/.planning/phases/36-substantive-evidence-admission-and-exact-re-promotion"
readonly plan_09="${phase_dir}/36-09-PLAN.md"
readonly plan_09_summary="${phase_dir}/36-09-SUMMARY.md"
readonly plan_10="${phase_dir}/36-10-PLAN.md"
readonly plan_07="${phase_dir}/36-07-PLAN.md"
readonly plan_04="${phase_dir}/36-04-PLAN.md"
readonly gsd_tools="/Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs"
readonly test_nonce="${RANDOM}${RANDOM}"
readonly protected_parent="${workspace_root}/target/private-evidence/phase36-process-${test_nonce}"
readonly candidate_output="${workspace_root}/target/private-evidence/phase36-candidate-${test_nonce}.json"
readonly attempt_handle="${protected_parent}/attempt.handle"
readonly classification_output="${protected_parent}/classification.json"
readonly hardware_parent="${workspace_root}/target/private-evidence/phase36-hardware-process-${test_nonce}"
readonly hardware_candidate="${workspace_root}/target/private-evidence/phase36-hardware-candidate-${test_nonce}.json"
readonly hardware_handle="${hardware_parent}/attempt.handle"
readonly test_output="${TEST_TMPDIR:-/tmp}/phase36-process-${test_nonce}.out"
readonly test_stderr="${TEST_TMPDIR:-/tmp}/phase36-process-${test_nonce}.err"
readonly fake_bin="${TEST_TMPDIR:-/tmp}/phase36-fake-bin-${test_nonce}"
readonly detector_log="${TEST_TMPDIR:-/tmp}/phase36-detector-${test_nonce}.log"
readonly effect_log="${TEST_TMPDIR:-/tmp}/phase36-effect-${test_nonce}.log"
readonly adapter_runfiles="${TEST_TMPDIR:-/tmp}/phase36-adapter-runfiles-${test_nonce}"
readonly adapter_child="${TEST_TMPDIR:-/tmp}/phase36-adapter-child-${test_nonce}"
readonly adapter_manifest="${TEST_TMPDIR:-/tmp}/phase36-adapter-manifest-${test_nonce}.json"
readonly adapter_factory="${TEST_TMPDIR:-/tmp}/phase36-adapter-factory-${test_nonce}.bin"
readonly adapter_log="${TEST_TMPDIR:-/tmp}/phase36-adapter-${test_nonce}.log"
readonly broker_runfiles="${TEST_TMPDIR:-/tmp}/phase36-broker-runfiles-${test_nonce}"
readonly broker_flash_log="${TEST_TMPDIR:-/tmp}/phase36-broker-flash-${test_nonce}.log"
readonly synthetic_credential="${TEST_TMPDIR:-/tmp}/phase36-synthetic-credential-${test_nonce}"
readonly protected_canary="synthetic-protected-origin"
readonly never_persist_canary="synthetic-never-persist-canary"
readonly credential_probe="${TEST_TMPDIR:-/tmp}/synthetic-never-persist-canary-${test_nonce}.json"
case_parents=()
case_candidates=()

cleanup() {
	[[ "$protected_parent" == "$workspace_root/target/private-evidence/phase36-process-"* ]] ||
		return
	[[ "$hardware_parent" == "$workspace_root/target/private-evidence/phase36-hardware-process-"* ]] ||
		return
	chmod -R u+rwX "$protected_parent" "$hardware_parent" 2>/dev/null || true
	rm -rf "$protected_parent" "$hardware_parent"
	rm -rf "$fake_bin" "$adapter_runfiles" "$adapter_child"
	rm -f "$candidate_output" "$hardware_candidate" "$test_output" "$test_stderr" \
		"$detector_log" "$effect_log" "$adapter_manifest" "$adapter_factory" "$adapter_log" \
		"$broker_flash_log" "$synthetic_credential"
	rm -rf "$broker_runfiles"
	local path
	for path in "${case_parents[@]}" "${case_candidates[@]}"; do
		[[ "$path" == "$workspace_root/target/private-evidence/phase36-"* ]] || continue
		chmod -R u+rwX "$path" 2>/dev/null || true
		rm -rf "$path"
	done
}
trap cleanup EXIT

fail_test() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

digest_file() {
	shasum -a 256 "$1" | awk '{print $1}'
}

run_supervisor() {
	BUILD_WORKSPACE_DIRECTORY="$workspace_root" "$supervisor" "$@"
}

expect_failure_category() {
	local category="$1"
	shift
	if run_supervisor "$@" >"$test_output" 2>"$test_stderr"; then
		fail_test "expected failure category ${category}"
	fi
	[[ ! -s "$test_output" ]] || fail_test "failed mode wrote public stdout"
	rg -Fqx "category=${category}" "$test_stderr" ||
		fail_test "failed mode omitted category ${category}"
}

umask 077
[[ -x "$supervisor" && -x "$report_binary" && -x "$deployed_effect_adapter" ]] ||
	fail_test "deployed executables are unavailable"
[[ -x "$gsd_tools" ]] || fail_test "GSD phase index tool is unavailable"

expect_failure_category missing_mode \
	board=205 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output"
expect_failure_category unknown_argument \
	mode=preflight \
	board=205 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output" \
	unknown=value
expect_failure_category duplicate_argument \
	mode=preflight \
	mode=synthetic \
	board=205 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output"

run_supervisor \
	mode=preflight \
	board=205 \
	capture-timeout-seconds=360 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output" >"$test_output" 2>"$test_stderr"
rg -Fqx 'category=preflight_ready' "$test_output" ||
	fail_test "preflight did not report its closed category"
[[ ! -s "$test_stderr" ]] || fail_test "preflight wrote stderr"
[[ "$(file_mode "$protected_parent")" == 700 ]] ||
	fail_test "preflight parent is not mode 0700"
[[ "$(file_mode "$attempt_handle")" == 600 ]] ||
	fail_test "attempt handle is not mode 0600"
readonly child_name="$(jq -er '.child_name' "$attempt_handle")"
readonly attempt_child="${protected_parent}/${child_name}"
[[ ! -e "$attempt_child" ]] ||
	fail_test "preflight created the broker-owned child"
for wrapper_stream in "${attempt_handle}.stdout" "${attempt_handle}.stderr"; do
	[[ -f "$wrapper_stream" && "$(file_mode "$wrapper_stream")" == 600 ]] ||
		fail_test "wrapper stream is not a distinct mode-0600 sibling"
done

run_supervisor \
	mode=synthetic \
	board=205 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output" >"$test_output" 2>"$test_stderr"
rg -Fqx 'category=synthetic_complete' "$test_output" ||
	fail_test "synthetic mode did not report its closed category"
[[ ! -s "$test_stderr" ]] || fail_test "synthetic mode wrote stderr"
[[ -d "$attempt_child" && "$(file_mode "$attempt_child")" == 700 ]] ||
	fail_test "synthetic broker child is not mode 0700"
for private_file in \
	"$attempt_child/private-capture.json" \
	"$attempt_child/seal.json" \
	"${attempt_handle}.stdout" \
	"${attempt_handle}.stderr"; do
	[[ -f "$private_file" && "$(file_mode "$private_file")" == 600 ]] ||
		fail_test "synthetic private file is not mode 0600"
done

readonly private_digest_before="$(digest_file "$attempt_child/private-capture.json")"
readonly candidate_digest_before="$(digest_file "$candidate_output")"
readonly private_mtime_before="$(stat -f '%m' "$attempt_child/private-capture.json")"
readonly candidate_mtime_before="$(stat -f '%m' "$candidate_output")"
readonly tree_before="$(
	find "$protected_parent" -type f -print |
		LC_ALL=C sort |
		while IFS= read -r path; do
			printf '%s %s %s\n' "${path#"$protected_parent/"}" "$(digest_file "$path")" "$(stat -f '%m' "$path")"
		done
)"

run_supervisor \
	mode=inspect-candidate \
	board=205 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output" >"$test_output" 2>"$test_stderr"
rg -Fq '"category": "candidate_eligible"' "$test_output" ||
	fail_test "inspect did not emit candidate_eligible"
[[ ! -s "$test_stderr" ]] || fail_test "inspect wrote stderr"
readonly tree_after_inspect="$(
	find "$protected_parent" -type f -print |
		LC_ALL=C sort |
		while IFS= read -r path; do
			printf '%s %s %s\n' "${path#"$protected_parent/"}" "$(digest_file "$path")" "$(stat -f '%m' "$path")"
		done
)"
[[ "$tree_before" == "$tree_after_inspect" ]] ||
	fail_test "inspect changed bytes, mtimes, or file membership"

run_supervisor \
	mode=classify-candidate \
	board=205 \
	private-parent="$protected_parent" \
	attempt-handle-file="$attempt_handle" \
	candidate-output="$candidate_output" \
	classification-output="$classification_output" >"$test_output" 2>"$test_stderr"
rg -Fqx 'category=classification_complete' "$test_output" ||
	fail_test "classification did not emit classification_complete"
[[ ! -s "$test_stderr" ]] || fail_test "classification wrote stderr"
[[ -f "$classification_output" && "$(file_mode "$classification_output")" == 600 ]] ||
	fail_test "classification output is not the explicit mode-0600 file"
[[ "$(digest_file "$attempt_child/private-capture.json")" == "$private_digest_before" ]] ||
	fail_test "classification changed private input bytes"
[[ "$(digest_file "$candidate_output")" == "$candidate_digest_before" ]] ||
	fail_test "classification changed candidate bytes"
[[ "$(stat -f '%m' "$attempt_child/private-capture.json")" == "$private_mtime_before" ]] ||
	fail_test "classification changed private input mtime"
[[ "$(stat -f '%m' "$candidate_output")" == "$candidate_mtime_before" ]] ||
	fail_test "classification changed candidate mtime"

for public_sink in "$candidate_output" "$test_output" "$test_stderr"; do
	if rg -Fq -- "$protected_canary" "$public_sink"; then
		fail_test "ProtectedOperational canary reached a public sink"
	fi
	if rg -Fq -- "$protected_parent" "$public_sink"; then
		fail_test "private path reached a public sink"
	fi
	if rg -Fq -- "$never_persist_canary" "$public_sink"; then
		fail_test "NeverPersistRaw canary reached a public sink"
	fi
done

mkdir -p \
	"$adapter_runfiles/_main/tools/flash" \
	"$adapter_runfiles/_main/tools/parity" \
	"$adapter_runfiles/_main/scripts" \
	"$adapter_child"
chmod 700 "$adapter_runfiles" "$adapter_child"
printf '{}\n' >"$adapter_manifest"
printf 'factory\n' >"$adapter_factory"
printf '%s\n' \
	'#!/usr/bin/env bash' \
	'set -euo pipefail' \
	'printf "%s\n" "$*" >>"$PHASE36_TEST_EFFECT_LOG"' \
	'jq -cn --arg operation "$PHASE36_EFFECT_OPERATION" --arg package "$PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST" --arg factory "$PHASE36_EFFECT_FACTORY_IMAGE_DIGEST" '"'"'{schema_version:"phase36-effect-result-v1",operation:$operation,status:"completed",failure:null,package_identity_digest:$package,factory_image_digest:$factory}'"'"' >"$PHASE36_EFFECT_RESULT_PATH"' \
	'chmod 600 "$PHASE36_EFFECT_RESULT_PATH"' \
	>"$adapter_runfiles/_main/tools/flash/flash"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' \
	>"$adapter_runfiles/_main/tools/parity/report"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' \
	>"$adapter_runfiles/_main/scripts/phase35_http_boundary_read"
printf '%s\n' '#!/usr/bin/env bash' 'exit 0' \
	>"$adapter_runfiles/_main/scripts/phase17-websocket-capture.mjs"
chmod 700 \
	"$adapter_runfiles/_main/tools/flash/flash" \
	"$adapter_runfiles/_main/tools/parity/report" \
	"$adapter_runfiles/_main/scripts/phase35_http_boundary_read" \
	"$adapter_runfiles/_main/scripts/phase17-websocket-capture.mjs"
readonly adapter_digest="$(printf 'a%.0s' {1..64})"
readonly adapter_common_args=(
	board=205
	port=/dev/private-device
	attempt-child="$adapter_child"
	package-identity-digest="$adapter_digest"
	manifest-path="$adapter_manifest"
	manifest-digest="$adapter_digest"
	firmware-elf-path="$adapter_factory"
	firmware-elf-digest="$adapter_digest"
	executable-image-path="$adapter_factory"
	executable-image-digest="$adapter_digest"
	factory-image-path="$adapter_factory"
	factory-image-digest="$adapter_digest"
	capture-timeout-seconds=360
	wall-clock-timeout-seconds=420
)
if ! RUNFILES_DIR="$adapter_runfiles" \
	BUILD_WORKSPACE_DIRECTORY="$workspace_root" \
	PHASE36_TEST_EFFECT_LOG="$adapter_log" \
	"$deployed_effect_adapter" \
	operation=exact-package-flash \
	result-path="$adapter_child/effect-result-exact-package-flash.json" \
	"${adapter_common_args[@]}" \
	wifi-credentials=opaque; then
	fail_test "deployed exact flash adapter rejected the supported fake boundary"
fi
if ! RUNFILES_DIR="$adapter_runfiles" \
	BUILD_WORKSPACE_DIRECTORY="$workspace_root" \
	PHASE36_TEST_EFFECT_LOG="$adapter_log" \
	"$deployed_effect_adapter" \
	operation=typed-recovery \
	result-path="$adapter_child/effect-result-typed-recovery.json" \
	"${adapter_common_args[@]}"; then
	fail_test "deployed typed recovery adapter rejected the supported fake boundary"
fi
[[ "$(wc -l <"$adapter_log" | tr -d ' ')" == 2 ]] ||
	fail_test "deployed adapter did not invoke exactly two fake flash boundaries"
readonly exact_flash_arguments="$(sed -n '1p' "$adapter_log")"
readonly recovery_arguments="$(sed -n '2p' "$adapter_log")"
for rendered_arguments in "$exact_flash_arguments" "$recovery_arguments"; do
	rg -Fq -- '--redact-evidence' <<<"$rendered_arguments" ||
		fail_test "deployed adapter omitted redacted evidence"
	rg -Fq -- '--evidence-dir' <<<"$rendered_arguments" ||
		fail_test "deployed adapter omitted its evidence directory"
	if rg -Fq -- '--evidence-mode' <<<"$rendered_arguments"; then
		fail_test "deployed adapter passed flash-monitor-only evidence mode to flash"
	fi
done
rg -Fq -- '--wifi-credentials' <<<"$exact_flash_arguments" ||
	fail_test "exact flash omitted the opaque credential input"
if rg -Fq -- '--wifi-credentials' <<<"$recovery_arguments"; then
	fail_test "typed recovery received a credential input"
fi

run_supervisor \
	mode=preflight \
	board=205 \
	capture-timeout-seconds=360 \
	private-parent="$hardware_parent" \
	attempt-handle-file="$hardware_handle" \
	candidate-output="$hardware_candidate" >"$test_output" 2>"$test_stderr"

mkdir -p "$fake_bin"
printf '%s\n' \
	'#!/usr/bin/env bash' \
	'set -euo pipefail' \
	'printf "%s\n" "$*" >>"$PHASE36_TEST_DETECTOR_LOG"' \
	'exit 9' >"$fake_bin/just"
printf '%s\n' \
	'#!/usr/bin/env bash' \
	'set -euo pipefail' \
	'printf "%s\n" "$0 $*" >>"$PHASE36_TEST_EFFECT_LOG"' >"$fake_bin/phase36-effect"
chmod 700 "$fake_bin/just" "$fake_bin/phase36-effect"
ln -s phase36-effect "$fake_bin/espflash"
ln -s phase36-effect "$fake_bin/curl"
ln -s phase36-effect "$fake_bin/flash-monitor"

PATH="$fake_bin:$PATH" \
	PHASE36_TEST_DETECTOR_LOG="$detector_log" \
	PHASE36_TEST_EFFECT_LOG="$effect_log" \
	run_supervisor \
	mode=hardware \
	board=205 \
	capture-timeout-seconds=360 \
	private-parent="$hardware_parent" \
	attempt-handle-file="$hardware_handle" \
	candidate-output="$hardware_candidate" \
	wifi-credentials="$credential_probe" >"$test_output" 2>"$test_stderr"
rg -Fqx 'category=sealed_non_promotion' "$test_output" ||
	fail_test "detector failure omitted typed sealed non-promotion"
[[ ! -s "$test_stderr" ]] || fail_test "detector failure wrote public stderr"
[[ -f "$detector_log" ]] ||
	fail_test "broker did not create detector invocation evidence"
[[ "$(wc -l <"$detector_log" | tr -d ' ')" == 1 ]] ||
	fail_test "broker did not invoke the detector exactly once"
rg -Fqx 'detect-ultra205' "$detector_log" ||
	fail_test "broker invoked a command other than just detect-ultra205"
[[ ! -e "$effect_log" ]] ||
	fail_test "detector failure reached a later device-effect adapter"
[[ ! -e "$hardware_candidate" ]] ||
	fail_test "software-only hardware exercise created a candidate"
readonly hardware_child_name="$(jq -er '.child_name' "$hardware_handle")"
readonly hardware_child="$hardware_parent/$hardware_child_name"
[[ -d "$hardware_child" && "$(file_mode "$hardware_child")" == 700 ]] ||
	fail_test "failed hardware broker exercise omitted its protected child"
for private_file in "$hardware_child/effect-ledger.jsonl" "$hardware_child/seal.json"; do
	[[ -f "$private_file" && "$(file_mode "$private_file")" == 600 ]] ||
		fail_test "failed hardware broker exercise omitted a mode-0600 transaction file"
done
jq -e '
		.status == "sealed_non_promotion" and
		.first_failure == "detector_failed" and
		.secondary_failure == null and
		.recovery_disposition == "not_authorized"
	' "$hardware_child/seal.json" >/dev/null ||
	fail_test "failed hardware broker seal lost typed failure ordering"
[[ "$(jq -s 'length' "$hardware_child/effect-ledger.jsonl")" == 12 ]] ||
	fail_test "failed hardware broker ledger is incomplete"
jq -s -e '
		[.[].operation] |
		(index("exact_package_admission") < index("board205_detector_probe")) and
		(index("board205_detector_probe") < index("cleanup")) and
		(index("typed_recovery") == null)
	' "$hardware_child/effect-ledger.jsonl" >/dev/null ||
	fail_test "failed hardware broker ledger lost operation ordering"

expect_failure_category hardware_output_exists \
	mode=hardware \
	board=205 \
	capture-timeout-seconds=360 \
	private-parent="$hardware_parent" \
	attempt-handle-file="$hardware_handle" \
	candidate-output="$hardware_candidate" \
	wifi-credentials="$credential_probe"
[[ "$(wc -l <"$detector_log" | tr -d ' ')" == 1 ]] ||
	fail_test "replay attempt invoked the detector"

printf '%s\n' \
	'#!/usr/bin/env bash' \
	'set -euo pipefail' \
	'printf "%s\n" "$*" >>"$PHASE36_TEST_DETECTOR_LOG"' \
	'printf "port=/dev/private-device\n"' >"$fake_bin/just"
chmod 700 "$fake_bin/just"
: >"$synthetic_credential"
chmod 600 "$synthetic_credential"
mkdir -p \
	"$broker_runfiles/_main/tools/flash" \
	"$broker_runfiles/_main/tools/parity" \
	"$broker_runfiles/_main/scripts"
ln -s "$report_binary" "$broker_runfiles/_main/tools/parity/report"
ln -s "$deployed_effect_adapter" "$broker_runfiles/_main/scripts/phase36_hardware_effect"
printf '%s\n' '#!/usr/bin/env bash' 'exit 9' \
	>"$broker_runfiles/_main/scripts/phase35_http_boundary_read"
printf '%s\n' '#!/usr/bin/env bash' 'exit 9' \
	>"$broker_runfiles/_main/scripts/phase17-websocket-capture.mjs"
printf '%s\n' \
	'#!/usr/bin/env bash' \
	'set -euo pipefail' \
	'operation="${PHASE36_EFFECT_OPERATION:-monitor}"' \
	'printf "%s %s %s\n" "$PHASE36_TEST_SCENARIO" "$operation" "$*" >>"$PHASE36_TEST_BROKER_FLASH_LOG"' \
	'write_result() {' \
	'  local status="$1" failure="${2:-}" package="$PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST"' \
	'  if [[ "$PHASE36_TEST_SCENARIO" == mismatch ]]; then package="$(printf "c%.0s" {1..64})"; fi' \
	'  jq -cn --arg operation "$PHASE36_EFFECT_OPERATION" --arg status "$status" --arg failure "$failure" --arg package "$package" --arg factory "$PHASE36_EFFECT_FACTORY_IMAGE_DIGEST" '"'"'{schema_version:"phase36-effect-result-v1",operation:$operation,status:$status,failure:(if $failure == "" then null else $failure end),package_identity_digest:$package,factory_image_digest:$factory}'"'"' >"$PHASE36_EFFECT_RESULT_PATH"' \
	'  chmod 600 "$PHASE36_EFFECT_RESULT_PATH"' \
	'}' \
	'case "$PHASE36_TEST_SCENARIO:$operation" in' \
	'parser:exact_package_flash)' \
	'  exec "$PHASE36_TEST_REAL_FLASH" flash --board 205 --port /dev/private-device --manifest /tmp/package.json --image /tmp/factory.bin --evidence-mode dual --redact-evidence --evidence-dir /tmp/private-stage --wifi-credentials /tmp/wifi.json' \
	'  ;;' \
	'partial:exact_package_flash|mismatch:exact_package_flash)' \
	'  write_result failed_confirmed_partial_device_effect flash_failed' \
	'  exit 9' \
	'  ;;' \
	'partial:typed_recovery|completed_then_capture:exact_package_flash|completed_then_capture:typed_recovery)' \
	'  write_result completed' \
	'  ;;' \
	'forged:exact_package_flash)' \
	'  printf '"'"'{"status":"failed_confirmed_partial_device_effect"}\n'"'"'' \
	'  ;;' \
	'esac' >"$broker_runfiles/_main/tools/flash/flash"
chmod 700 \
	"$broker_runfiles/_main/tools/flash/flash" \
	"$broker_runfiles/_main/scripts/phase35_http_boundary_read" \
	"$broker_runfiles/_main/scripts/phase17-websocket-capture.mjs"

run_closed_effect_case() {
	local scenario="$1"
	local expected_failure="$2"
	local expected_recovery="$3"
	local expected_recovery_calls="$4"
	local case_parent="$workspace_root/target/private-evidence/phase36-${scenario}-${test_nonce}"
	local case_candidate="$workspace_root/target/private-evidence/phase36-${scenario}-candidate-${test_nonce}.json"
	local case_handle="$case_parent/attempt.handle"
	case_parents+=("$case_parent")
	case_candidates+=("$case_candidate")

	run_supervisor \
		mode=preflight \
		board=205 \
		capture-timeout-seconds=360 \
		private-parent="$case_parent" \
		attempt-handle-file="$case_handle" \
		candidate-output="$case_candidate" >"$test_output" 2>"$test_stderr"
	RUNFILES_DIR="$broker_runfiles" \
		PATH="$fake_bin:$PATH" \
		PHASE36_TEST_DETECTOR_LOG="$detector_log" \
		PHASE36_TEST_BROKER_FLASH_LOG="$broker_flash_log" \
		PHASE36_TEST_SCENARIO="$scenario" \
		PHASE36_TEST_REAL_FLASH="$deployed_flash" \
		run_supervisor \
		mode=hardware \
		board=205 \
		capture-timeout-seconds=360 \
		private-parent="$case_parent" \
		attempt-handle-file="$case_handle" \
		candidate-output="$case_candidate" \
		wifi-credentials="$synthetic_credential" >"$test_output" 2>"$test_stderr"
	rg -Fqx 'category=sealed_non_promotion' "$test_output" ||
		fail_test "${scenario} did not seal non-promotional"
	[[ ! -s "$test_stderr" ]] ||
		fail_test "${scenario} wrote public stderr"
	local case_child_name case_child
	case_child_name="$(jq -er '.child_name' "$case_handle")"
	case_child="$case_parent/$case_child_name"
	jq -e \
		--arg failure "$expected_failure" \
		--arg recovery "$expected_recovery" \
		'.status == "sealed_non_promotion" and
		.first_failure == $failure and
		.recovery_disposition == $recovery' \
		"$case_child/seal.json" >/dev/null ||
		fail_test "${scenario} seal lost its closed failure/recovery disposition"
	[[ "$(jq -s '[.[] | select(.operation == "typed_recovery" and .transition.status == "invoked")] | length' "$case_child/effect-ledger.jsonl")" == "$expected_recovery_calls" ]] ||
		fail_test "${scenario} recorded an incorrect recovery invocation count"
	[[ "$(jq -s '[.[] | select(.operation == "cleanup" and .transition.status == "invoked")] | length' "$case_child/effect-ledger.jsonl")" == 1 ]] ||
		fail_test "${scenario} did not invoke cleanup exactly once"
	[[ ! -e "$case_candidate" ]] ||
		fail_test "${scenario} created a candidate"
	while IFS= read -r result_file; do
		[[ "$(file_mode "$result_file")" == 600 ]] ||
			fail_test "${scenario} effect result is not mode 0600"
	done < <(find "$case_child" -name 'effect-result-*.json' -type f -print)
}

run_closed_effect_case parser parser_failed not_authorized 0
run_closed_effect_case partial flash_failed attempted_succeeded 1
run_closed_effect_case completed_then_capture capture_failed attempted_succeeded 1
run_closed_effect_case forged flash_failed not_authorized 0
run_closed_effect_case mismatch invocation_construction_failed not_authorized 0
[[ "$(rg -c '^partial typed_recovery ' "$broker_flash_log")" == 1 ]] ||
	fail_test "confirmed partial flash did not use exactly one same-image recovery"
[[ "$(rg -c '^completed_then_capture typed_recovery ' "$broker_flash_log")" == 1 ]] ||
	fail_test "completed flash did not use exactly one same-image recovery"
if rg -q '^parser typed_recovery ' "$broker_flash_log"; then
	fail_test "real parser rejection reached recovery"
fi
if rg -q '^forged typed_recovery ' "$broker_flash_log"; then
	fail_test "forged stdout reached recovery"
fi
if rg -q '^mismatch typed_recovery ' "$broker_flash_log"; then
	fail_test "mismatched closed result reached recovery"
fi

readonly direct_effect_pattern='espflash|flash-monitor|serial-session|device-session|curl[[:space:]]|phase17-websocket|phase35-correlated'
if rg -q -i "$direct_effect_pattern" "$supervisor_source"; then
	fail_test "supervisor directly owns an effect-capable adapter"
fi
[[ "$(rg -Fc 'just detect-ultra205' "$supervisor_source")" == 1 ]] ||
	fail_test "supervisor does not expose exactly one broker-owned detector contract"
rg -Fq 'const DETECTOR_PROGRAM: &str = "just";' "$hardware_broker_source" ||
	fail_test "hardware broker does not own the detector program"
rg -Fq 'const DETECTOR_ARGUMENT: &str = "detect-ultra205";' "$hardware_broker_source" ||
	fail_test "hardware broker does not own the detector argument"
rg -Fq 'Command::new(DETECTOR_PROGRAM)' "$hardware_broker_source" ||
	fail_test "hardware broker does not invoke its detector adapter"
readonly nested_runner_pattern='Command::new\("(bazel|cargo|just)"|std::process'
if rg -q "$nested_runner_pattern" "$broker_source" "$capture_source"; then
	fail_test "production broker/capture child invokes a nested build runner"
fi
rg -Fq 'phase36-substantive-evidence *args:' "$justfile" ||
	fail_test "Justfile omits Phase 36 command surface"
rg -Fq 'bazel run //scripts:phase36_substantive_evidence -- {{ args }}' "$justfile" ||
	fail_test "Justfile does not use the deployed supervisor runfiles"

readonly graph_json="$("$gsd_tools" phase-plan-index 36)"
readonly incomplete_graph="$(
	jq -r '
		. as $root
		| .plans[]
		| select(.id as $id | ($root.incomplete | index($id)) != null)
		| [.wave, .id]
		| @tsv
	' <<<"$graph_json" |
		LC_ALL=C sort -n -k1,1 |
		awk -F '\t' '{print $2 "@" $1}'
)"
if [[ -f "$plan_09_summary" ]]; then
	readonly expected_graph=$'36-10@9\n36-07@10\n36-04@11'
	readonly incomplete_plans=("$plan_10" "$plan_07" "$plan_04")
else
	readonly expected_graph=$'36-09@8\n36-10@9\n36-07@10\n36-04@11'
	readonly incomplete_plans=("$plan_09" "$plan_10" "$plan_07" "$plan_04")
fi
[[ "$incomplete_graph" == "$expected_graph" ]] ||
	fail_test "incomplete Phase 36 graph is not the exact wave-ordered contract"
for plan in "${incomplete_plans[@]}"; do
	awk '
		NR == 1 && $0 == "---" { in_frontmatter = 1; next }
		in_frontmatter && $0 == "---" { exit }
		in_frontmatter && $0 == "gap_closure: true" { found = 1 }
		END { exit(found ? 0 : 1) }
	' "$plan" || fail_test "incomplete Phase 36 plan is not gap_closure"
done

require_frontmatter_dependency() {
	local plan="$1"
	local dependency="$2"
	awk -v expected="  - \"${dependency}\"" '
		NR == 1 && $0 == "---" { in_frontmatter = 1; next }
		in_frontmatter && $0 == "---" { exit }
		in_frontmatter && $0 == "depends_on:" { in_dependencies = 1; next }
		in_dependencies && $0 !~ /^  / { in_dependencies = 0 }
		in_dependencies && $0 == expected { found = 1 }
		END { exit(found ? 0 : 1) }
	' "$plan" ||
		fail_test "Phase 36 plan omits required dependency ${dependency}"
}

require_frontmatter_dependency "$plan_10" "36-09"
require_frontmatter_dependency "$plan_07" "36-10"
require_frontmatter_dependency "$plan_04" "36-07"

if command -v lsof >/dev/null 2>&1; then
	[[ -z "$(lsof +D "$protected_parent" 2>/dev/null || true)" ]] ||
		fail_test "a process retains a descriptor below the sealed root"
fi

printf 'phase36 substantive evidence process tests passed\n'
