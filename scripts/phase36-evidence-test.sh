#!/usr/bin/env bash
set -euo pipefail

readonly report_binary="${1:?missing parity report binary}"
readonly envelope_only_fixture="${2:?missing envelope-only Phase 36 fixture}"
readonly phase36_module="${3:?missing Phase 36 module source}"
readonly phase36_contract="${4:?missing Phase 36 contract source}"
readonly parity_main="${5:?missing parity CLI source}"
readonly effects_fixture="${6:?missing independent effects fixture}"
readonly effects_module="${7:?missing independent effects module source}"
readonly offline_module="${8:?missing offline command source}"
readonly phase35_manifest="${9:?missing Phase 35 manifest}"
readonly phase35_projection="${10:?missing Phase 35 projection}"
readonly phase35_matrix="${11:?missing Phase 35 matrix}"
readonly phase35_verdict="${12:?missing Phase 35 verdict}"
readonly checklist_fixture="${13:?missing parity checklist}"
readonly test_parent="$(mktemp -d "${TEST_TMPDIR:-/tmp}/phase36-evidence.XXXXXX")"
readonly protected_root="${test_parent}/protected"
readonly protected_input="${protected_root}/phase36.json"
readonly protected_note="${protected_root}/opaque-note.txt"
readonly protected_effects="${protected_root}/independent-effects.json"
readonly public_output="${test_parent}/shareable.json"
readonly stderr_output="${test_parent}/stderr.txt"
readonly command_block="${test_parent}/command-source.txt"
readonly effects_command_block="${test_parent}/effects-command-source.txt"
readonly effects_output="${test_parent}/effects-shareable.json"
readonly effects_stderr="${test_parent}/effects-stderr.txt"
readonly insufficient_output="${test_parent}/effects-insufficient.txt"
readonly offline_workspace="${test_parent}/offline-workspace"
readonly offline_output="${test_parent}/offline-output.json"
readonly offline_stderr="${test_parent}/offline-stderr.txt"
readonly offline_command_block="${test_parent}/offline-command-source.txt"
readonly offline_production_module="${test_parent}/offline-production-source.txt"
readonly protected_canary="opaque-phase36-protected-canary"

cleanup() {
	[[ -n "$test_parent" && "$test_parent" == */phase36-evidence.* ]] ||
		return
	chmod -R u+rwX "$test_parent"
	rm -rf "$test_parent"
}
trap cleanup EXIT

fail_test() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
}

assert_absent_literal() {
	local file="$1"
	local literal="$2"
	if rg -Fq -- "$literal" "$file"; then
		fail_test "protected material reached a shareable sink"
	fi
}

umask 077
chmod 700 "$test_parent"
mkdir "$protected_root"
chmod 700 "$protected_root"
cp "$envelope_only_fixture" "$protected_input"
printf '%s\n' "$protected_canary" >"$protected_note"
chmod 600 "$protected_input" "$protected_note"

if "$report_binary" classify-phase36-evidence --root "$protected_root" \
	>"$public_output" 2>"$stderr_output"; then
	fail_test "caller-authored envelope classified without artifact authority"
fi
chmod 600 "$public_output" "$stderr_output"

[[ "$(file_mode "$test_parent")" == 700 ]] ||
	fail_test "protected parent mode is not 0700"
[[ "$(file_mode "$protected_root")" == 700 ]] ||
	fail_test "protected child mode is not 0700"
[[ "$(file_mode "$protected_input")" == 600 ]] ||
	fail_test "protected input mode is not 0600"
[[ "$(file_mode "$protected_note")" == 600 ]] ||
	fail_test "protected note mode is not 0600"
[[ "$(file_mode "$public_output")" == 600 ]] ||
	fail_test "shareable output mode is not 0600"
[[ "$(file_mode "$stderr_output")" == 600 ]] ||
	fail_test "stderr capture mode is not 0600"
[[ ! -s "$public_output" ]] ||
	fail_test "rejected envelope wrote shareable output"
rg -Fq 'category=protected_input_missing' "$stderr_output" ||
	fail_test "envelope-only input did not fail closed on its missing artifact"

for sink in "$public_output" "$stderr_output"; do
	assert_absent_literal "$sink" "$protected_canary"
	assert_absent_literal "$sink" "$protected_root"
	assert_absent_literal "$sink" "$protected_input"
	assert_absent_literal "$sink" "$protected_note"
done

cp "$effects_fixture" "$protected_effects"
chmod 600 "$protected_effects"
if ! "$report_binary" classify-phase36-effects --root "$protected_root" \
	>"$effects_output" 2>"$effects_stderr"; then
	fail_test "independent effect classifier process failed"
fi
chmod 600 "$effects_output" "$effects_stderr"
[[ ! -s "$effects_stderr" ]] ||
	fail_test "successful independent effect classification wrote stderr"
rg -Fq '"status": "validated"' "$effects_output" ||
	fail_test "complete independent effect fixture was not admitted"
rg -Fq '"effect_count": 8' "$effects_output" ||
	fail_test "independent effect projection did not bind all allowed effects"
assert_absent_literal "$effects_output" "$protected_root"
assert_absent_literal "$effects_output" "$protected_canary"

rm "$protected_effects"
if ! "$report_binary" classify-phase36-effects --root "$protected_root" \
	>"$insufficient_output" 2>"$effects_stderr"; then
	fail_test "missing independent effect evidence did not classify"
fi
[[ "$(<"$insufficient_output")" == "category=independent_effect_observation_insufficient" ]] ||
	fail_test "missing independent effect evidence did not emit the exact insufficiency category"
[[ ! -s "$effects_stderr" ]] ||
	fail_test "independent effect insufficiency wrote stderr"

mkdir -p \
	"$offline_workspace/docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion" \
	"$offline_workspace/docs/parity"
cp "$phase35_manifest" \
	"$offline_workspace/docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/.phase35-generation-manifest.json"
cp "$phase35_projection" \
	"$offline_workspace/docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/projection.json"
cp "$phase35_matrix" \
	"$offline_workspace/docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/decision-matrix.json"
cp "$phase35_verdict" \
	"$offline_workspace/docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion/admitted.json"
cp "$checklist_fixture" "$offline_workspace/docs/parity/checklist.md"
if ! "$report_binary" reevaluate-phase36-attempt31 --workspace-root "$offline_workspace" \
	>"$offline_output" 2>"$offline_stderr"; then
	fail_test "offline Attempt 31 insufficiency classification failed"
fi
[[ ! -s "$offline_stderr" ]] ||
	fail_test "offline Attempt 31 insufficiency classification wrote stderr"
rg -Fq '"status": "immutable_artifacts_insufficient"' "$offline_output" ||
	fail_test "missing protected companions did not emit aggregate insufficiency"
for category in \
	snapshot_substance_insufficient \
	runtime_health_insufficient \
	runtime_identity_observation_insufficient \
	independent_effect_observation_insufficient; do
	rg -Fq "\"$category\"" "$offline_output" ||
		fail_test "missing protected companions omitted $category"
done
for file in typed-fact-projection.json decision-matrix.json verdict.json manifest.json; do
	[[ -f "$offline_workspace/docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion/$file" ]] ||
		fail_test "offline Attempt 31 classification omitted $file"
done
assert_absent_literal "$offline_output" "$offline_workspace"
assert_absent_literal "$offline_output" "$protected_root"

sed -n '/^fn run_classify_phase36_evidence_command(/,/^}/p' "$parity_main" \
	>"$command_block"
chmod 600 "$command_block"
[[ -s "$command_block" ]] ||
	fail_test "Phase 36 CLI command block was not found"
sed -n '/^fn run_classify_phase36_effects_command(/,/^}/p' "$parity_main" \
	>"$effects_command_block"
chmod 600 "$effects_command_block"
[[ -s "$effects_command_block" ]] ||
	fail_test "Phase 36 effect CLI command block was not found"
sed -n '/^fn run_reevaluate_phase36_attempt31_command(/,/^}/p' "$parity_main" \
	>"$offline_command_block"
chmod 600 "$offline_command_block"
[[ -s "$offline_command_block" ]] ||
	fail_test "Phase 36 offline Attempt 31 command block was not found"
sed '/^#\[cfg(test)\]/q' "$offline_module" >"$offline_production_module"
chmod 600 "$offline_production_module"
[[ -s "$offline_production_module" ]] ||
	fail_test "Phase 36 offline production source was not found"

readonly effectful_pattern='detect[-_]?ultra205|credential|(^|[^[:alnum:]_])flash([^-[:alnum:]_]|$)|monitor|serial[-_](control|session)|curl[[:space:]].*((--request|-X)[[:space:]]*(PATCH|POST|PUT|DELETE)|--data)|phase28\.1\.1|hardware[-_ ]run'
if rg -q -i "$effectful_pattern" \
	"$phase36_module" "$phase36_contract" "$command_block" "$effects_command_block" \
	"$offline_command_block" "$offline_module"; then
	fail_test "Phase 36 read-only classifier contains an effectful invocation"
fi
readonly effect_invocation_pattern='std::process|ProcessCommand|Command::new|detect[-_]?ultra205|curl[[:space:]]|serial[-_](control|session)|flash[-_]monitor|run_detector|run_flash|run_monitor|credential_path'
if rg -q -i "$effect_invocation_pattern" "$effects_module" "$offline_production_module"; then
	fail_test "Phase 36 independent effect classifier contains an effectful invocation API"
fi
if rg -q 'read_dir|WalkDir|glob::|ignore::Walk' "$offline_module"; then
	fail_test "Phase 36 offline classifier contains filesystem discovery"
fi

printf 'phase36 evidence tests passed\n'
