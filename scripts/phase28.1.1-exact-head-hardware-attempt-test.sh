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
	if [[ "$fixture_name" == "lifecycle_start" && "${PHASE28_EFFECT_STATUS:-0}" == "0" ]]; then
		state_path="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/state.json"
		socket_path="$PHASE28_LIFECYCLE_ATTEMPT_DIR/lifecycle.sock"
		(cd "$PHASE28_LIFECYCLE_ATTEMPT_DIR" && node -e 'const fs=require("node:fs"),net=require("node:net"); const path="lifecycle.sock"; const server=net.createServer(()=>{}); server.listen(path,()=>fs.chmodSync(path,0o600));') &
		socket_pid=$!
		trap 'kill "$socket_pid" 2>/dev/null || true; wait "$socket_pid" 2>/dev/null || true' EXIT
		for _ in $(seq 1 100); do
			[[ -S "$socket_path" ]] && break
			sleep 0.01
		done
		[[ -S "$socket_path" ]]
		for _ in $(seq 1 500); do
			[[ -f "${PHASE28_SOCKET_TRACE:?}" && "$(wc -l <"$PHASE28_SOCKET_TRACE" | tr -d ' ')" -ge 1 ]] && break
			sleep 0.01
		done
		attestation_ms="$(jq -er '.attestation_accepted_ms' "$state_path")"
		values="$PHASE28_LIFECYCLE_ATTEMPT_DIR/lifecycle-values.json"
		jq -cn --argjson start "$attestation_ms" '{usb_absence_started_ms:$start}' >"$values"
		chmod 600 "$values"
		bash "${PHASE28_LIFECYCLE_RUNNER:?}" lifecycle-owner-transition --capability "${PHASE28_LIFECYCLE_CAPABILITY:?}" --event absence-observing --values-file "$values" >/dev/null
		jq -cn --argjson end "$((attestation_ms + 5000))" '{usb_absence_ended_ms:$end,usb_absence_ms:5000}' >"$values"
		chmod 600 "$values"
		bash "$PHASE28_LIFECYCLE_RUNNER" lifecycle-owner-transition --capability "$PHASE28_LIFECYCLE_CAPABILITY" --event restore-waiting --values-file "$values" >/dev/null
		for _ in $(seq 1 500); do
			[[ "$(wc -l <"$PHASE28_SOCKET_TRACE" | tr -d ' ')" -ge 2 ]] && break
			sleep 0.01
		done
		printf '{}\n' >"$values"
		chmod 600 "$values"
		bash "$PHASE28_LIFECYCLE_RUNNER" lifecycle-owner-transition --capability "$PHASE28_LIFECYCLE_CAPABILITY" --event reappearance-observing --values-file "$values" >/dev/null
		jq -cn --argjson reappearance "$((attestation_ms + 6000))" '{usb_reappearance_ms:$reappearance,reappearance_elapsed_ms:6000,capture_started_ms:$reappearance}' >"$values"
		chmod 600 "$values"
		bash "$PHASE28_LIFECYCLE_RUNNER" lifecycle-owner-transition --capability "$PHASE28_LIFECYCLE_CAPABILITY" --event capture-running --values-file "$values" >/dev/null
		jq -cn --argjson ended "$((attestation_ms + 366000))" '{capture_ended_ms:$ended,capture_duration_ms:360000,lifecycle_raw_log_sha256:("a"*64),same_chain_raw_log_set_sha256:("a"*64),classifier_input_sha256:("a"*64),lifecycle_status:"match",result_correlated:false,power_delta_class:"flat",share_submission_status:"not_observed"}' >"$values"
		chmod 600 "$values"
		bash "$PHASE28_LIFECYCLE_RUNNER" lifecycle-owner-transition --capability "$PHASE28_LIFECYCLE_CAPABILITY" --event capture-complete --values-file "$values" >/dev/null
		jq -cn --arg effect "$fixture_name" '{schema_version:"exact-head-effect-result-v1",effect_id:$effect,status:"completed",blocker_reason:"none",outputs:{}}' >"${PHASE28_EFFECT_RESULT_FILE:?}"
		chmod 600 "$PHASE28_EFFECT_RESULT_FILE"
	fi
	if [[ "${PHASE28_EFFECT_STATUS:-0}" == "0" && "$fixture_name" != "lifecycle_start" ]]; then
		case "$fixture_name" in
		detector_board_info) outputs='{"selected_port_fingerprint_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}' ;;
		credential_presence_bind) outputs='{"wifi_credential_state":"present","pool_credential_state":"present","wifi_credential_binding_id":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","pool_credential_binding_id":"cccccccccccccccccccccccccccccccc","credential_capability_status":"sealed","credential_capability_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}' ;;
		reference_guard) outputs='{"reference_commit":"2222222222222222222222222222222222222222","reference_guard_output_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}' ;;
		package) outputs="{\"manifest_source_commit\":\"${PHASE28_TEST_HEAD:?}\",\"manifest_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"factory_image_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}" ;;
		flash_reinit_runtime) outputs='{"runtime_credential_consumption":"pass","credential_capability_status":"destroyed","credential_capability_sha256":null,"reinit_capture_started_ms":10,"reinit_capture_ended_ms":360010,"reinit_capture_duration_ms":360000,"reinit_capture_category":"complete_360s","reinit_raw_log_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","reinit_classifier_input_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","reinit_five_stage_result":"pass"}' ;;
		post_capture_detector_board_info) outputs='{"result_correlated":null,"power_delta_class":null,"share_submission_status":null,"lifecycle_status":"absent","classifier_input_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","classifier_output_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","classifier_version":"strict-production-v2"}' ;;
		esac
		jq -cn --arg effect "$fixture_name" --argjson outputs "$outputs" '{schema_version:"exact-head-effect-result-v1",effect_id:$effect,status:"completed",blocker_reason:"none",outputs:$outputs}' >"${PHASE28_EFFECT_RESULT_FILE:?}"
		chmod 600 "$PHASE28_EFFECT_RESULT_FILE"
	fi
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
	local initial_state="${1:-}"
	if [[ -n "$initial_state" ]]; then
		env \
			PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
			PHASE28_TEST_MODE=1 \
			PHASE28_ALLOW_DIRTY_TEST=1 \
			PHASE28_TEST_HEAD="$test_head" \
			PHASE28_TEST_INITIAL_ATTEMPT_STATE="$initial_state" \
			bash "$runner" begin-attempt --hardware-exact-head "$test_head"
		return
	fi
	env \
		PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
		PHASE28_TEST_MODE=1 \
		PHASE28_ALLOW_DIRTY_TEST=1 \
		PHASE28_TEST_HEAD="$test_head" \
		bash "$runner" begin-attempt --hardware-exact-head "$test_head"
}

run_effect() {
	local handle="$1"
	local effect="$2"
	shift 2
	env \
		PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
		PHASE28_TEST_MODE=1 \
		PHASE28_ALLOW_DIRTY_TEST=1 \
		PHASE28_TEST_HEAD="$test_head" \
		PHASE28_TEST_EFFECT_ADAPTER_DIR="$adapter_dir" \
		PHASE28_EFFECT_TRACE="$temp_root/effects.trace" \
		"$@" \
		bash "$runner" run-validated-effect --resume-handle "$handle" --effect-id "$effect"
}

prepare_lifecycle_state() {
	local handle="$1"
	local slot
	slot="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
	local state_path
	state_path="$(jq -er '.attempt_dir' "$slot")/state.json"
	jq --arg head "$test_head" '
    .reference_guard_state="pass" |
    .reference_commit=("2"*40) |
    .reference_guard_output_sha256=("a"*64) |
    .selected_port_state="one_board205" |
    .selected_port_fingerprint_sha256=("a"*64) |
    .manifest_state="pass" |
    .manifest_source_commit=$head |
    .manifest_sha256=("a"*64) |
    .factory_image_sha256=("a"*64) |
    .reinit_capture_started_ms=10 |
    .reinit_capture_ended_ms=360010 |
    .reinit_capture_duration_ms=360000 |
    .reinit_capture_category="complete_360s" |
    .reinit_raw_log_sha256=("a"*64) |
    .reinit_classifier_input_sha256=("a"*64) |
    .reinit_five_stage_result="pass"
  ' "$state_path" >"$state_path.next"
	mv "$state_path.next" "$state_path"
	chmod 600 "$state_path"
}

wait_for_pattern() {
	local pattern="$1"
	local path="$2"
	for _ in $(seq 1 500); do
		rg -q "$pattern" "$path" 2>/dev/null && return
		sleep 0.01
	done
	fail "timed out waiting for $pattern"
}

first_output="$(begin_attempt)"
first_handle="$(sed -n 's/^resume_handle=//p' <<<"$first_output")"
[[ "$first_handle" =~ ^[0-9a-f]{64}$ ]] || fail "begin-attempt did not return one opaque handle"
[[ "$first_output" != *"$control_root"* ]] || fail "public handle output exposed a path"
[[ "$(rg -c '^[a-z_]+=' <<<"$first_output")" == "19" ]] || fail "begin-attempt did not return the exact 19 public fields"
rg -q '^## CHECKPOINT REACHED$' <<<"$first_output"
rg -q '^checkpoint_id=plan13-connected-entry$' <<<"$first_output"
rg -q '^attempt_state=connected_entry_waiting$' <<<"$first_output"
if rg -q 'attempt_dir|selected_port|credential|capability|lease_id|owner_pid|boot_session|effect_authorization|raw_log|device_url|endpoint|password' <<<"$first_output"; then
	fail "public checkpoint exposed a private field"
fi
[[ "$(mode_of "$control_root")" == "700" ]]
[[ "$(mode_of "$control_root/resume-index")" == "700" ]]
first_slot="$control_root/resume-index/$(printf '%s' "$first_handle" | shasum -a 256 | awk '{print $1}').json"
[[ "$(mode_of "$first_slot")" == "600" ]]
attempt_dir="$(jq -er '.attempt_dir' "$first_slot")"
[[ "$(mode_of "$attempt_dir")" == "700" ]]
[[ "$(mode_of "$attempt_dir/state.json")" == "600" ]]

resolve_output="$(env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" resolve-checkpoint --resume-handle "$first_handle")"
rg -q '^checkpoint_id=plan13-connected-entry$' <<<"$resolve_output"
rg -q '^checkpoint_generation=1$' <<<"$resolve_output"
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
	[[ "$effect" != "lifecycle_start" ]] || continue
	initial_state="$(initial_state_for_effect "$effect")"
	handle="$(begin_attempt "$initial_state")"
	handle="${handle#resume_handle=}"
	output="$(run_effect "$handle" "$effect")"
	rg -q "^effect_id=${effect}$" <<<"$output"
	rg -q '^effect_status=completed$' <<<"$output"
done
[[ "$(wc -l <"$temp_root/effects.trace" | tr -d ' ')" == "6" ]]

control_root="$temp_root/control-lifecycle"
socket_trace="$temp_root/lifecycle-socket.trace"
: >"$socket_trace"
lifecycle_handle="$(begin_attempt reinit_validated)"
lifecycle_handle="${lifecycle_handle#resume_handle=}"
prepare_lifecycle_state "$lifecycle_handle"
env \
	PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
	PHASE28_TEST_MODE=1 \
	PHASE28_ALLOW_DIRTY_TEST=1 \
	PHASE28_TEST_HEAD="$test_head" \
	PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" \
	PHASE28_TEST_EFFECT_ADAPTER_DIR="$adapter_dir" \
	PHASE28_EFFECT_TRACE="$temp_root/effects.trace" \
	PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" \
	PHASE28_SOCKET_TRACE="$socket_trace" \
	bash "$runner" run-validated-effect --resume-handle "$lifecycle_handle" --effect-id lifecycle_start >"$temp_root/lifecycle.stdout" 2>"$temp_root/lifecycle.stderr" &
lifecycle_runner_pid=$!
wait_for_pattern '^checkpoint_id=plan13-lifecycle-removal$' "$temp_root/lifecycle.stdout"
expect_failure lease_dead_or_reused_process env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="b$(printf 'b%.0s' {1..63})" PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" PHASE28_SOCKET_TRACE="$socket_trace" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token plan13-armed-removal-v1 --response-token plan13-both-power-paths-removed
[[ ! -s "$socket_trace" ]] || fail "wrong lifecycle owner reached socket dispatch"
expect_failure checkpoint_token_mismatch env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" PHASE28_SOCKET_TRACE="$socket_trace" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token wrong --response-token plan13-both-power-paths-removed
[[ ! -s "$socket_trace" ]] || fail "wrong lifecycle token reached socket dispatch"
env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" PHASE28_SOCKET_TRACE="$socket_trace" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token plan13-armed-removal-v1 --response-token plan13-both-power-paths-removed >/dev/null
wait_for_pattern '^checkpoint_id=plan13-lifecycle-restore$' "$temp_root/lifecycle.stdout"
env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" PHASE28_SOCKET_TRACE="$socket_trace" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token plan13-barrel-usb-restore-v1 --response-token plan13-barrel-then-usb-restored >/dev/null
wait "$lifecycle_runner_pid"
rg -q '^effect_status=completed$' "$temp_root/lifecycle.stdout"
lifecycle_slot="$control_root/resume-index/$(printf '%s' "$lifecycle_handle" | shasum -a 256 | awk '{print $1}').json"
lifecycle_state="$(jq -er '.attempt_dir' "$lifecycle_slot")/state.json"
[[ "$(jq -r '.attempt_state' "$lifecycle_state")" == "capture_complete" ]]
[[ "$(jq -r '.process_running' "$lifecycle_state")" == "false" ]]
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
	after_effect_transition_persistence
)
for effect in "${effects[@]}"; do
	initial_state="$(initial_state_for_effect "$effect")"
	for boundary in "${crash_boundaries[@]}"; do
		crash_root="$temp_root/crash-$effect-$boundary"
		control_root="$crash_root/control"
		mkdir -p "$crash_root"
		: >"$temp_root/effects.trace"
		handle="$(begin_attempt "$initial_state")"
		handle="${handle#resume_handle=}"
		expect_exit_failure run_effect "$handle" "$effect" PHASE28_CRASH_AT="$boundary" PHASE28_EFFECT_STATUS=1
		before_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
		if [[ "$boundary" == "before_authorized_persistence" ]]; then
			[[ "$before_count" == "0" ]] || fail "$effect/$boundary touched the adapter"
			continue
		fi
		if [[ "$boundary" == "after_completed_persistence" ]]; then
			set +e
			run_effect "$handle" "$effect" PHASE28_EFFECT_STATUS=1 >"$temp_root/recovery.stdout" 2>"$temp_root/recovery.stderr"
			set -e
			after_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
			[[ "$before_count" == "$after_count" ]] || fail "$effect/$boundary redispatched completed work"
			continue
		fi
		if [[ "$boundary" == "after_effect_transition_persistence" ]]; then
			set +e
			run_effect "$handle" "$effect" PHASE28_EFFECT_STATUS=1 >"$temp_root/recovery.stdout" 2>"$temp_root/recovery.stderr"
			set -e
			after_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
			[[ "$before_count" == "$after_count" ]] || fail "$effect/$boundary redispatched transitioned work"
			continue
		fi
		expect_failure effect_in_flight_ambiguous run_effect "$handle" "$effect" PHASE28_EFFECT_STATUS=1
		after_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
		[[ "$before_count" == "$after_count" ]] || fail "$effect/$boundary redispatched ambiguous work"
		expect_failure resume_handle_stale env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "$handle"
		tombstone="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
		[[ "$(mode_of "$tombstone")" == "600" ]]
		[[ "$(jq -r 'keys | sort | join(",")' "$tombstone")" == "attempt_generation,cleanup_time_category,resume_handle_sha256,schema_version,terminal_category,terminal_status" ]] || fail "$effect/$boundary tombstone retained live references"
	done
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
expect_failure checkpoint_state_mismatch env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" deliver-token --resume-handle "$handle" --checkpoint-token wrong --response-token ultra205-remains-connected

rg -q 'production_adapter=.*phase28.1.1-accepted-state-diagnostic.sh' "$runner" || fail "runner lacks the fixed repo-owned production adapter"
if rg -q 'just detect-ultra205|espflash|flash-monitor|wifi-credentials.json|pool-credentials' "$runner"; then
	fail "runner contains a direct hardware or credential-content command"
fi
arbitrary_handle="$(begin_attempt connected_entry_waiting)"
arbitrary_handle="${arbitrary_handle#resume_handle=}"
expect_failure validator_error env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_EFFECT_ADAPTER_DIR="$adapter_dir" bash "$runner" run-validated-effect --resume-handle "$arbitrary_handle" --effect-id detector_board_info
if rg -q 'wifi-secret|pool-secret' "$control_root"; then
	fail "private control state retained a credential sentinel"
fi

printf 'phase28.1.1 exact-head hardware attempt tests: passed (84 invalid cases)\n'
