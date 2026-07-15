#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly wrapper="${PHASE33_WRAPPER:-${script_dir}/phase33-confirmed-settings-durability.sh}"
readonly fixture="${script_dir}/phase33-confirmed-settings-durability-fixture.sh"
tmp_root="$(realpath "$(mktemp -d "${TMPDIR:-/tmp}/phase33-confirmed-settings-test.XXXXXX")")"
readonly tmp_root
trap 'rm -rf "$tmp_root"' EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

fake_bin="${tmp_root}/fake-bin"
mkdir -p "$fake_bin"
for command in just curl classifier identity monitor checkpoint; do
	ln -s "$fixture" "${fake_bin}/${command}"
done

manifest="${tmp_root}/package.json"
jq -cn \
	--arg source_commit "test-source-commit" \
	--arg reference_commit "test-reference-commit" \
	'{source_commit:$source_commit,reference_commit:$reference_commit}' >"$manifest"

scenario_status=0
scenario_output=""
scenario_summary=""
scenario_state=""

run_scenario() {
	local scenario="$1"
	local maybe_manifest="${2:-$manifest}"
	scenario_output="${tmp_root}/${scenario}.out"
	scenario_summary="${tmp_root}/${scenario}.md"
	scenario_state="${tmp_root}/${scenario}-state"
	local raw_root="${tmp_root}/${scenario}-raw"
	rm -rf "$scenario_state" "$raw_root" "$scenario_output" "$scenario_summary"
	set +e
	PHASE33_ALLOW_TEST_MODE=1 \
		PHASE33_FAKE_STATE_ROOT="$scenario_state" \
		PHASE33_JUST_COMMAND="${fake_bin}/just" \
		PHASE33_CURL_COMMAND="${fake_bin}/curl" \
		PHASE33_CLASSIFIER="${fake_bin}/classifier" \
		PHASE33_IDENTITY_COMMAND="${fake_bin}/identity" \
		PHASE33_PASSIVE_MONITOR_COMMAND="${fake_bin}/monitor" \
		PHASE33_CHECKPOINT_COMMAND="${fake_bin}/checkpoint" \
		PHASE33_POLL_INTERVAL_SECONDS=0.001 \
		bash "$wrapper" --mode simulate --scenario "$scenario" \
		--capture-seconds 1 --manifest "$maybe_manifest" \
		--shareable-out "$scenario_summary" --local-root "$raw_root" \
		>"$scenario_output" 2>&1
	scenario_status=$?
	set -e
}

assert_call_count() {
	local expected="$1"
	local pattern="$2"
	local calls_log="${scenario_state}/calls.log"
	local actual=0
	if [[ -f "$calls_log" ]]; then
		actual="$(grep -Ec "$pattern" "$calls_log" || true)"
	fi
	[[ "$actual" == "$expected" ]] || fail "${pattern} count ${actual}, expected ${expected}"
}

assert_restored() {
	grep -Fq 'curl PATCH restore' "${scenario_state}/calls.log" || fail "restoration PATCH missing"
	grep -Fq 'restoration_category=confirmed_restored' "$scenario_output" || fail "confirmed restoration category missing"
}

run_scenario success
[[ "$scenario_status" == "0" ]] || fail "success orchestration failed"
grep -Fq -- '- status: passed' "$scenario_summary" || fail "success status missing"
grep -Fq -- '- application_restart_count: 1' "$scenario_summary" || fail "restart count missing"
grep -Fq -- 'simulation-only; no hardware or parity claim' "$scenario_summary" || fail "simulation non-claim missing"
assert_call_count 1 '^just detect-ultra205$'
assert_call_count 1 '^just flash-monitor '
assert_call_count 1 '^curl PATCH proof$'
assert_call_count 1 '^curl POST restart$'
assert_call_count 1 '^curl PATCH restore$'

failure_cases='detector_ambiguous detector_ambiguous
board_info_failure board_info_failure
failed_flash required_package_flash_failed
zero_origin runtime_origin_missing
multiple_origin runtime_origin_multiple
identity_change physical_identity_changed
unchanged_session post_restart_session_unchanged
multiple_session post_restart_multiple_sessions
ordinal_n_plus_two post_restart_ordinal_nonmonotonic
wrong_reset post_restart_reset_reason_wrong
wrong_session_origin runtime_origin_wrong_session
extra_reset post_restart_multiple_sessions
response_reversal response_before_effect_unproved
immediate_missing immediate_readback_missing
immediate_mismatch immediate_readback_mismatch
post_missing post_reboot_readback_missing
post_mismatch post_reboot_readback_mismatch
holder_leak serial_holder_cleanup_failed
process_leak monitor_process_cleanup_failed
timeout proof_timeout
restore_failure restoration_failed
sensitive_output sensitive_output_detected'

while read -r scenario expected_category; do
	run_scenario "$scenario"
	[[ "$scenario_status" != "0" ]] || fail "${scenario} unexpectedly passed"
	grep -Fq "failure_category=${expected_category}" "$scenario_output" || fail "${scenario} category mismatch"
	assert_call_count 1 '^just detect-ultra205$'
	case "$scenario" in
	identity_change | unchanged_session | multiple_session | ordinal_n_plus_two | wrong_reset | wrong_session_origin | extra_reset | response_reversal | immediate_missing | immediate_mismatch | post_missing | post_mismatch | holder_leak | process_leak | timeout)
		assert_restored
		;;
	sensitive_output)
		grep -Fq 'curl PATCH restore' "${scenario_state}/calls.log" || fail "sensitive-output restoration PATCH missing"
		;;
	esac
done <<<"$failure_cases"

run_scenario missing_flash "${tmp_root}/missing-package.json"
[[ "$scenario_status" != "0" ]] || fail "missing package unexpectedly passed"
grep -Fq 'failure_category=required_package_missing' "$scenario_output" || fail "missing package category mismatch"
assert_call_count 0 '^just '
assert_call_count 0 '^curl '

for scenario in cancel_after_patch errexit_after_patch; do
	run_scenario "$scenario"
	case "$scenario" in
	cancel_after_patch) [[ "$scenario_status" == "143" ]] || fail "signal status was not preserved" ;;
	errexit_after_patch) [[ "$scenario_status" == "73" ]] || fail "errexit status was not preserved" ;;
	esac
	assert_restored
	assert_call_count 0 '^curl POST restart$'
done

unsafe_state="${tmp_root}/unsafe-state"
unsafe_output="${tmp_root}/unsafe-root.out"
set +e
PHASE33_ALLOW_TEST_MODE=1 PHASE33_FAKE_STATE_ROOT="$unsafe_state" \
	PHASE33_JUST_COMMAND="${fake_bin}/just" PHASE33_CURL_COMMAND="${fake_bin}/curl" \
	PHASE33_CLASSIFIER="${fake_bin}/classifier" PHASE33_IDENTITY_COMMAND="${fake_bin}/identity" \
	PHASE33_PASSIVE_MONITOR_COMMAND="${fake_bin}/monitor" \
	bash "$wrapper" --mode simulate --scenario success --capture-seconds 1 \
	--manifest "$manifest" --shareable-out "${tmp_root}/unsafe.md" \
	--local-root ".planning/phases/33-confirmed-settings-durability" >"$unsafe_output" 2>&1
unsafe_status=$?
set -e
[[ "$unsafe_status" != "0" ]] || fail "tracked local root unexpectedly passed"
grep -Fq 'failure_category=local_raw_root_unsafe' "$unsafe_output" || fail "tracked local root category mismatch"
[[ ! -e "${unsafe_state}/calls.log" ]] || fail "unsafe root reached a detector, flash, or HTTP command"

tracked_ignored_repo="${tmp_root}/tracked-ignored-repo"
tracked_ignored_state="${tmp_root}/tracked-ignored-state"
tracked_ignored_output="${tmp_root}/tracked-ignored-root.out"
git init -q "$tracked_ignored_repo"
mkdir "${tracked_ignored_repo}/raw"
printf 'raw/\n' >"${tracked_ignored_repo}/.gitignore"
printf 'tracked raw evidence sentinel\n' >"${tracked_ignored_repo}/raw/tracked.txt"
git -C "$tracked_ignored_repo" add .gitignore
git -C "$tracked_ignored_repo" add -f raw/tracked.txt
set +e
(
	cd "$tracked_ignored_repo"
	PHASE33_ALLOW_TEST_MODE=1 PHASE33_FAKE_STATE_ROOT="$tracked_ignored_state" \
		PHASE33_JUST_COMMAND="${fake_bin}/just" PHASE33_CURL_COMMAND="${fake_bin}/curl" \
		PHASE33_CLASSIFIER="${fake_bin}/classifier" PHASE33_IDENTITY_COMMAND="${fake_bin}/identity" \
		PHASE33_PASSIVE_MONITOR_COMMAND="${fake_bin}/monitor" \
		bash "$wrapper" --mode simulate --scenario success --capture-seconds 1 \
		--manifest "$manifest" --shareable-out "${tmp_root}/tracked-ignored.md" \
		--local-root raw
) >"$tracked_ignored_output" 2>&1
tracked_ignored_status=$?
set -e
[[ "$tracked_ignored_status" != "0" ]] || fail "tracked-but-ignored local root unexpectedly passed"
grep -Fq 'failure_category=local_raw_root_unsafe' "$tracked_ignored_output" || fail "tracked-but-ignored root category mismatch"
[[ ! -e "${tracked_ignored_state}/calls.log" ]] || fail "tracked-but-ignored root reached a detector, flash, or HTTP command"

mkdir -m 700 "${tmp_root}/real-root"
ln -s "${tmp_root}/real-root" "${tmp_root}/symlink-root"
symlink_output="${tmp_root}/symlink-root.out"
set +e
PHASE33_ALLOW_TEST_MODE=1 PHASE33_FAKE_STATE_ROOT="${tmp_root}/symlink-state" \
	bash "$wrapper" --mode simulate --scenario success --capture-seconds 1 \
	--manifest "$manifest" --shareable-out "${tmp_root}/symlink.md" \
	--local-root "${tmp_root}/symlink-root" >"$symlink_output" 2>&1
symlink_status=$?
set -e
[[ "$symlink_status" != "0" ]] || fail "symlink local root unexpectedly passed"
grep -Fq 'failure_category=local_raw_root_unsafe' "$symlink_output" || fail "symlink root category mismatch"

source_text="$(<"$wrapper")"
for forbidden_call in \
	'espflash erase' \
	'esptool.py' \
	'pool-credentials=' \
	'just phase28' \
	'just diagnose-ultra205-uart' \
	'just parity'; do
	if grep -Fq "$forbidden_call" <<<"$source_text"; then
		fail "forbidden command surface found: ${forbidden_call}"
	fi
done
grep -Fq -- '--before no-reset-no-sync --after no-reset --no-reset' "$wrapper" || fail "passive reset contract missing"
# The single-quoted needles intentionally inspect literal wrapper source.
# shellcheck disable=SC2016
[[ "$(grep -Fc '"$just_command" detect-ultra205' "$wrapper")" == "1" ]] || fail "detector source count is not exactly one"
grep -Fq 'capture_seconds >= 360' "$wrapper" || fail "minimum capture gate missing"
# shellcheck disable=SC2016
[[ "$(grep -Fc '"$just_command" flash-monitor' "$wrapper")" == "1" ]] || fail "required package flash source count is not exactly one"
grep -Fq 'phase33-classify' "$wrapper" || fail "typed classifier missing"
grep -Fq 'passive_byte_delivery_unproved' "$wrapper" || fail "pre-POST byte delivery gate missing"
[[ "$(grep -Fc '/api/system/restart' "$wrapper")" == "1" ]] || fail "restart POST source count is not exactly one"
service_loss_line="$(grep -nF '((service_lost == 1)) || fail_proof service_loss_unproved' "$wrapper" | cut -d: -f1)"
# The command substitution is intentionally literal because this inspects wrapper source.
# shellcheck disable=SC2016
proof_offset_line="$(grep -nF 'proof_offset="$(wc -c <"$passive_raw"' "$wrapper" | cut -d: -f1)"
post_classifier_line="$(grep -nF -- '--mode post-restart' "$wrapper" | cut -d: -f1)"
[[ "$service_loss_line" -lt "$proof_offset_line" && "$proof_offset_line" -lt "$post_classifier_line" ]] || fail "post-restart byte boundary is not service-loss scoped"
if grep -Eq -- '--skip-flash|retained-runtime' "$wrapper"; then
	fail "wrapper exposes an unauthorized skip-flash path"
fi

if rg -n -i 'https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|/dev/|ssid|password|worker|device_url|api[_ -]?(token|key)' "${tmp_root}/success.md"; then
	fail "success summary leaked a sensitive value"
fi
if ! rg -q 'https?://|password' "${tmp_root}/sensitive_output.md"; then
	fail "sensitive-output fixture did not exercise the denylist"
fi

printf 'phase33 confirmed settings durability tests passed\n'
