#!/usr/bin/env bash
set -euo pipefail

readonly supervisor="${1:?missing deployed supervisor}"
readonly report_binary="${2:?missing deployed report binary}"
readonly agents_file="${3:?missing workspace marker}"
readonly justfile="${4:?missing Justfile}"
readonly supervisor_source="${5:?missing supervisor source}"
readonly broker_source="${6:?missing broker source}"
readonly capture_source="${7:?missing capture source}"
readonly agents_real="$(perl -MCwd=realpath -e 'print realpath($ARGV[0])' "$agents_file")"
readonly workspace_root="$(dirname "$agents_real")"
readonly phase_dir="${workspace_root}/.planning/phases/36-substantive-evidence-admission-and-exact-re-promotion"
readonly plan_05="${phase_dir}/36-05-PLAN.md"
readonly plan_06="${phase_dir}/36-06-PLAN.md"
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
readonly protected_canary="synthetic-protected-origin"
readonly never_persist_canary="synthetic-never-persist-canary"

cleanup() {
	[[ "$protected_parent" == "$workspace_root/target/private-evidence/phase36-process-"* ]] ||
		return
	[[ "$hardware_parent" == "$workspace_root/target/private-evidence/phase36-hardware-process-"* ]] ||
		return
	chmod -R u+rwX "$protected_parent" "$hardware_parent" 2>/dev/null || true
	rm -rf "$protected_parent" "$hardware_parent"
	rm -f "$candidate_output" "$hardware_candidate" "$test_output" "$test_stderr"
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
[[ -x "$supervisor" && -x "$report_binary" ]] ||
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

run_supervisor \
	mode=preflight \
	board=205 \
	capture-timeout-seconds=360 \
	private-parent="$hardware_parent" \
	attempt-handle-file="$hardware_handle" \
	candidate-output="$hardware_candidate" >"$test_output" 2>"$test_stderr"
expect_failure_category hardware_broker_failed \
	mode=hardware \
	board=205 \
	capture-timeout-seconds=360 \
	private-parent="$hardware_parent" \
	attempt-handle-file="$hardware_handle" \
	candidate-output="$hardware_candidate" \
	wifi-credentials="$never_persist_canary"
[[ ! -e "$hardware_candidate" ]] ||
	fail_test "software-only hardware exercise created a candidate"
readonly hardware_child_name="$(jq -er '.child_name' "$hardware_handle")"
[[ ! -e "$hardware_parent/$hardware_child_name" ]] ||
	fail_test "failed hardware broker exercise created its protected child"

readonly direct_effect_pattern='espflash|detect-ultra205|flash-monitor|serial-session|device-session|curl[[:space:]]|phase17-websocket|phase35-correlated'
if rg -q -i "$direct_effect_pattern" "$supervisor_source"; then
	fail_test "supervisor directly owns an effect-capable adapter"
fi
readonly nested_runner_pattern='Command::new\\(\"(bazel|cargo|just)\"|std::process'
if rg -q "$nested_runner_pattern" "$broker_source" "$capture_source"; then
	fail_test "production broker/capture child invokes a nested build runner"
fi
rg -Fq 'phase36-substantive-evidence *args:' "$justfile" ||
	fail_test "Justfile omits Phase 36 command surface"
rg -Fq './scripts/phase36-substantive-evidence.sh {{ args }}' "$justfile" ||
	fail_test "Justfile does not forward exact mode arguments"

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
readonly expected_graph=$'36-05@5\n36-06@6\n36-07@7\n36-04@8'
[[ "$incomplete_graph" == "$expected_graph" ]] ||
	fail_test "incomplete Phase 36 graph is not the exact wave-ordered contract"
for plan in "$plan_05" "$plan_06" "$plan_07" "$plan_04"; do
	awk '
		NR == 1 && $0 == "---" { in_frontmatter = 1; next }
		in_frontmatter && $0 == "---" { exit }
		in_frontmatter && $0 == "gap_closure: true" { found = 1 }
		END { exit(found ? 0 : 1) }
	' "$plan" || fail_test "incomplete Phase 36 plan is not gap_closure"
done

if command -v lsof >/dev/null 2>&1; then
	[[ -z "$(lsof +D "$protected_parent" 2>/dev/null || true)" ]] ||
		fail_test "a process retains a descriptor below the sealed root"
fi

printf 'phase36 substantive evidence process tests passed\n'
