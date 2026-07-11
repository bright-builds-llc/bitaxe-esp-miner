#!/usr/bin/env bash
set -euo pipefail

fixture_name="$(basename "$0")"
readonly fixture_name
case "$fixture_name" in
detector_board_info | credential_presence_bind | reference_guard | package | flash_reinit_runtime | lifecycle_start | post_capture_detector_board_info)
	: >"${PHASE28_EFFECT_ACK_FILE:?}"
	for _ in $(seq 1 500); do
		[[ -f "${PHASE28_EFFECT_GATE_FILE:?}" ]] && break
		sleep 0.01
	done
	[[ -f "$PHASE28_EFFECT_GATE_FILE" ]]
	printf '%s\n' "$fixture_name" >>"${PHASE28_EFFECT_TRACE:?}"
	exit "${PHASE28_EFFECT_STATUS:-0}"
	;;
phase28-fake-socket-send)
	printf '%s\n' "$*" >>"${PHASE28_SOCKET_TRACE:?}"
	exit 0
	;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runner="$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt.sh"
temp_base="${TEST_TMPDIR:-$repo_root/scratch}"
mkdir -p "$temp_base"
temp_root="$(mktemp -d "$temp_base/phase28-attempt-test.XXXXXX")"
trap 'rm -rf "$temp_root"' EXIT
control_root="$temp_root/control"
adapter_dir="$temp_root/adapters"
mkdir -p "$adapter_dir"
test_head="1111111111111111111111111111111111111111"

effects=(
	detector_board_info
	credential_presence_bind
	reference_guard
	package
	flash_reinit_runtime
	lifecycle_start
	post_capture_detector_board_info
)
invalid_categories=(
	expired
	token_mismatch
	exact_head_mismatch
	manifest_mismatch
	reference_mismatch
	boot_session_mismatch
	dirty_head
	malformed_state
	validator_error
	lock_failure
	persistence_failure
	lease_conflict
)

for effect in "${effects[@]}"; do
	ln -s "$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt-test.sh" "$adapter_dir/$effect"
done
socket_sender="$temp_root/phase28-fake-socket-send"
ln -s "$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt-test.sh" "$socket_sender"

fail() {
	printf 'phase28_exact_head_attempt_test_error: %s\n' "$1" >&2
	exit 1
}

expect_failure() {
	local expected="$1"
	shift
	if "$@" >"$temp_root/failure.stdout" 2>"$temp_root/failure.stderr"; then
		fail "command unexpectedly succeeded for $expected"
	fi
	rg -q "phase28_attempt_error=${expected}" "$temp_root/failure.stderr" || {
		cat "$temp_root/failure.stderr" >&2
		fail "missing expected failure $expected"
	}
}

expect_exit_failure() {
	if "$@" >"$temp_root/failure.stdout" 2>"$temp_root/failure.stderr"; then
		fail "command unexpectedly succeeded"
	fi
}

mode_of() {
	stat -f '%Lp' "$1"
}

initial_state_for_effect() {
	case "$1" in
	detector_board_info) printf 'connected_entry_waiting\n' ;;
	credential_presence_bind) printf 'detector_passed\n' ;;
	reference_guard) printf 'credentials_bound\n' ;;
	package) printf 'reference_checked\n' ;;
	flash_reinit_runtime) printf 'packaged\n' ;;
	lifecycle_start) printf 'reinit_validated\n' ;;
	post_capture_detector_board_info) printf 'capture_complete\n' ;;
	esac
}

begin_attempt() {
	local initial_state="${1:-new}"
	env \
		PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
		PHASE28_ALLOW_DIRTY_TEST=1 \
		PHASE28_TEST_HEAD="$test_head" \
		PHASE28_TEST_INITIAL_ATTEMPT_STATE="$initial_state" \
		bash "$runner" begin-attempt
}

run_effect() {
	local handle="$1"
	local effect="$2"
	shift 2
	env \
		PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
		PHASE28_ALLOW_DIRTY_TEST=1 \
		PHASE28_TEST_HEAD="$test_head" \
		PHASE28_EFFECT_ADAPTER_DIR="$adapter_dir" \
		PHASE28_EFFECT_TRACE="$temp_root/effects.trace" \
		"$@" \
		bash "$runner" run-validated-effect --resume-handle "$handle" --effect-id "$effect"
}

first_output="$(begin_attempt)"
first_handle="${first_output#resume_handle=}"
[[ "$first_handle" =~ ^[0-9a-f]{64}$ ]] || fail "begin-attempt did not return one opaque handle"
[[ "$first_output" != *"/"* ]] || fail "public handle output exposed a path"
[[ "$(mode_of "$control_root")" == "700" ]]
[[ "$(mode_of "$control_root/resume-index")" == "700" ]]
first_slot="$control_root/resume-index/$(printf '%s' "$first_handle" | shasum -a 256 | awk '{print $1}').json"
[[ "$(mode_of "$first_slot")" == "600" ]]
attempt_dir="$(jq -er '.attempt_dir' "$first_slot")"
[[ "$(mode_of "$attempt_dir")" == "700" ]]
[[ "$(mode_of "$attempt_dir/state.json")" == "600" ]]

resolve_output="$(env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" resolve-checkpoint --resume-handle "$first_handle")"
rg -q '^checkpoint_id=none$' <<<"$resolve_output"
rg -q '^checkpoint_generation=0$' <<<"$resolve_output"
expect_failure resume_handle_missing env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle ""
expect_failure resume_handle_malformed env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle malformed
expect_failure resume_handle_wrong env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "f$(printf '0%.0s' {1..63})"

: >"$temp_root/effects.trace"
for effect in "${effects[@]}"; do
	initial_state="$(initial_state_for_effect "$effect")"
	for invalid in "${invalid_categories[@]}"; do
		handle="$(begin_attempt "$initial_state")"
		handle="${handle#resume_handle=}"
		before_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
		expect_failure "$invalid" run_effect "$handle" "$effect" PHASE28_INJECT_INVALID_CATEGORY="$invalid"
		after_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
		[[ "$before_count" == "$after_count" ]] || fail "$effect/$invalid touched an adapter sentinel"
	done
done

for effect in "${effects[@]}"; do
	initial_state="$(initial_state_for_effect "$effect")"
	handle="$(begin_attempt "$initial_state")"
	handle="${handle#resume_handle=}"
	output="$(run_effect "$handle" "$effect")"
	rg -q "^effect_id=${effect}$" <<<"$output"
	rg -q '^effect_status=completed$' <<<"$output"
done
[[ "$(wc -l <"$temp_root/effects.trace" | tr -d ' ')" == "7" ]]

crash_boundaries=(
	before_authorized_persistence
	after_authorized_persistence
	after_child_creation
	after_start_acknowledgement
	after_invoked_persistence
	after_start_gate_release
	after_adapter_return
	after_completed_persistence
)
for boundary in "${crash_boundaries[@]}"; do
	crash_root="$temp_root/crash-$boundary"
	control_root="$crash_root/control"
	mkdir -p "$crash_root"
	: >"$temp_root/effects.trace"
	handle="$(begin_attempt connected_entry_waiting)"
	handle="${handle#resume_handle=}"
	expect_exit_failure run_effect "$handle" detector_board_info PHASE28_CRASH_AT="$boundary"
	# A persisted authorized/invoked record must not redispatch on restart.
	if [[ "$boundary" != "before_authorized_persistence" && "$boundary" != "after_completed_persistence" ]]; then
		before_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
		expect_failure effect_in_flight_ambiguous run_effect "$handle" detector_board_info
		after_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
		[[ "$before_count" == "$after_count" ]] || fail "$boundary redispatched ambiguous work"
		expect_failure resume_handle_stale env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "$handle"
	fi
done

control_root="$temp_root/control-final"
handle="$(begin_attempt connected_entry_waiting)"
handle="${handle#resume_handle=}"
slot="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
state="$(jq -er '.attempt_dir' "$slot")/state.json"
deadline=$(($(perl -MTime::HiRes=clock_gettime,CLOCK_MONOTONIC -e 'printf "%.0f", clock_gettime(CLOCK_MONOTONIC) * 1000') + 60000))
jq --argjson deadline "$deadline" '.checkpoint_id="plan13-connected-entry" | .checkpoint_token="plan13-connected-entry-v1" | .expected_response_token="ultra205-remains-connected" | .expected_user_action="keep-connected" | .monotonic_deadline_ms=$deadline' "$state" >"$state.next"
mv "$state.next" "$state"
chmod 600 "$state"
delivery="$(env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" deliver-token --resume-handle "$handle" --checkpoint-token plan13-connected-entry-v1 --response-token ultra205-remains-connected)"
rg -q '^checkpoint_delivery=accepted$' <<<"$delivery"
expect_failure checkpoint_token_mismatch env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" deliver-token --resume-handle "$handle" --checkpoint-token wrong --response-token ultra205-remains-connected

if rg -q 'phase28.1.1-accepted-state-diagnostic.sh|just detect-ultra205|espflash|flash-monitor|wifi-credentials.json|pool-credentials' "$runner"; then
	fail "runner contains a direct hardware or credential-content command"
fi
if rg -q 'wifi-secret|pool-secret' "$control_root"; then
	fail "private control state retained a credential sentinel"
fi

printf 'phase28.1.1 exact-head hardware attempt tests: passed (84 invalid cases)\n'
