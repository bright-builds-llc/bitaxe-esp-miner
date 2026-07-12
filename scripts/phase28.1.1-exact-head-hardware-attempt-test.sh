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
		socket_pid=""
		frame_path="$PHASE28_LIFECYCLE_ATTEMPT_DIR/fixture-frame.json"
		if [[ "${PHASE28_ORPHAN_RECEIVER_FIXTURE:-0}" == "1" ]]; then
			(
				cd "$PHASE28_LIFECYCLE_ATTEMPT_DIR"
				exec perl "${PHASE28_LIFECYCLE_FRAME_HELPER:?}" receive --socket lifecycle.sock --output fixture-frame.json
			) >/dev/null 2>&1 &
			socket_pid=$!
			printf '%s\n' "$socket_pid" >"${PHASE28_ORPHAN_RECEIVER_PID_FILE:?}"
			for _ in $(seq 1 100); do
				[[ -S "$socket_path" ]] && break
				kill -0 "$socket_pid" 2>/dev/null || exit 1
				sleep 0.01
			done
			[[ -S "$socket_path" ]] || exit 1
			exit 1
		fi
		wait_for_fixture_socket() {
			for _ in $(seq 1 100); do
				[[ -S "$socket_path" ]] && return
				kill -0 "$socket_pid" 2>/dev/null || break
				sleep 0.01
			done
			return 1
		}
		start_fixture_receiver() {
			rm -f "$socket_path" "$frame_path"
			(
				cd "$PHASE28_LIFECYCLE_ATTEMPT_DIR"
				perl "${PHASE28_LIFECYCLE_FRAME_HELPER:?}" receive --socket lifecycle.sock --output fixture-frame.json
			) &
			socket_pid=$!
			wait_for_fixture_socket
		}
		receive_fixture_frame() {
			local expected_token="$1"
			for _ in $(seq 1 500); do
				[[ -f "$frame_path" ]] && break
				kill -0 "$socket_pid" 2>/dev/null || break
				sleep 0.01
			done
			wait "$socket_pid"
			socket_pid=""
			[[ -f "$frame_path" ]]
			[[ "$(jq -er '.response_token' "$frame_path")" == "$expected_token" ]]
			printf '%s\n' "$expected_token" >>"${PHASE28_SOCKET_TRACE:?}"
		}
		trap 'if [[ -n "$socket_pid" ]]; then kill "$socket_pid" 2>/dev/null || true; wait "$socket_pid" 2>/dev/null || true; fi' EXIT
		if [[ "${PHASE28_REAL_SOCKET_FIXTURE:-0}" == "1" ]]; then
			start_fixture_receiver
		else
			(cd "$PHASE28_LIFECYCLE_ATTEMPT_DIR" && node -e 'const fs=require("node:fs"),net=require("node:net"); const path="lifecycle.sock"; const server=net.createServer(()=>{}); server.listen(path,()=>fs.chmodSync(path,0o600));') &
			socket_pid=$!
			wait_for_fixture_socket
		fi
		if [[ "${PHASE28_REAL_SOCKET_FIXTURE:-0}" == "1" ]]; then
			receive_fixture_frame plan13-both-power-paths-removed
		else
			for _ in $(seq 1 500); do
				[[ -f "${PHASE28_SOCKET_TRACE:?}" && "$(wc -l <"$PHASE28_SOCKET_TRACE" | tr -d ' ')" -ge 1 ]] && break
				sleep 0.01
			done
		fi
		attestation_ms="$(jq -er '.attestation_accepted_ms' "$state_path")"
		values="$PHASE28_LIFECYCLE_ATTEMPT_DIR/lifecycle-values.json"
		jq -cn --argjson start "$attestation_ms" '{usb_absence_started_ms:$start}' >"$values"
		chmod 600 "$values"
		bash "${PHASE28_LIFECYCLE_RUNNER:?}" lifecycle-owner-transition --capability "${PHASE28_LIFECYCLE_CAPABILITY:?}" --event absence-observing --values-file "$values" >/dev/null
		if [[ "${PHASE28_REAL_SOCKET_FIXTURE:-0}" == "1" ]]; then
			start_fixture_receiver
		fi
		jq -cn --argjson end "$((attestation_ms + 5000))" '{usb_absence_ended_ms:$end,usb_absence_ms:5000}' >"$values"
		chmod 600 "$values"
		bash "$PHASE28_LIFECYCLE_RUNNER" lifecycle-owner-transition --capability "$PHASE28_LIFECYCLE_CAPABILITY" --event restore-waiting --values-file "$values" >/dev/null
		if [[ "${PHASE28_REAL_SOCKET_FIXTURE:-0}" == "1" ]]; then
			receive_fixture_frame plan13-barrel-then-usb-restored
		else
			for _ in $(seq 1 500); do
				[[ "$(wc -l <"$PHASE28_SOCKET_TRACE" | tr -d ' ')" -ge 2 ]] && break
				sleep 0.01
			done
		fi
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
		post_capture_detector_board_info)
			outputs="${PHASE28_POST_CAPTURE_OUTPUTS:-}"
			if [[ -z "$outputs" ]]; then
				outputs='{"result_correlated":null,"power_delta_class":null,"share_submission_status":null,"lifecycle_status":"absent","classifier_input_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","classifier_output_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","classifier_version":"strict-production-v2"}'
			fi
			;;
		esac
		jq -cn --arg effect "$fixture_name" --argjson outputs "$outputs" '{schema_version:"exact-head-effect-result-v1",effect_id:$effect,status:"completed",blocker_reason:"none",outputs:$outputs}' >"${PHASE28_EFFECT_RESULT_FILE:?}"
		chmod 600 "$PHASE28_EFFECT_RESULT_FILE"
	fi
	exit "${PHASE28_EFFECT_STATUS:-0}"
	;;
phase28-fake-socket-send)
	printf '%s\n' "$*" >>"${PHASE28_SOCKET_TRACE:?}"
	socket_path="${1:?}"
	frame="${2:?}"
	helper="${PHASE28_LIFECYCLE_FRAME_HELPER:-$(dirname "$(readlink "$0")")/phase28.1.1-lifecycle-frame.pl}"
	printf '%s' "$frame" | (
		cd "$(dirname "$socket_path")"
		perl "$helper" send --socket "$(basename "$socket_path")"
	)
	exit $?
	;;
phase28-monotonic-sequence)
	sequence_file="${PHASE28_MONOTONIC_SEQUENCE_FILE:?}"
	value="$(sed -n '1p' "$sequence_file")"
	[[ "$value" =~ ^[0-9]+$ ]]
	sed '1d' "$sequence_file" >"$sequence_file.next"
	mv "$sequence_file.next" "$sequence_file"
	printf '%s\n' "$value"
	exit 0
	;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runner="$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt.sh"
temp_base="${TEST_TMPDIR:-$repo_root/scratch}"
mkdir -p "$temp_base"
temp_root="$(mktemp -d "$temp_base/phase28-attempt-test.XXXXXX")"
orphan_receiver_pid=""
trap 'if [[ -n "$orphan_receiver_pid" ]]; then kill "$orphan_receiver_pid" 2>/dev/null || true; wait "$orphan_receiver_pid" 2>/dev/null || true; fi; rm -rf "$temp_root"' EXIT
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
monotonic_sequence="$temp_root/phase28-monotonic-sequence"
ln -s "$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt-test.sh" "$monotonic_sequence"

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

slot_for_handle() {
	local handle="$1"
	printf '%s/resume-index/%s.json\n' "$control_root" "$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}')"
}

state_for_handle() {
	local slot
	slot="$(slot_for_handle "$1")"
	printf '%s/state.json\n' "$(jq -er '.attempt_dir' "$slot")"
}

cleanup_attempt() {
	local handle="$1"
	shift
	env \
		PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
		PHASE28_TEST_MODE=1 \
		PHASE28_ALLOW_DIRTY_TEST=1 \
		PHASE28_TEST_HEAD="$test_head" \
		"$@" \
		bash "$runner" cleanup-active-attempt --resume-handle "$handle"
}

cleanup_unique_orphan() {
	env \
		PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
		PHASE28_ALLOW_DIRTY_TEST=1 \
		"$@" \
		bash "$runner" cleanup-unique-orphan-attempt \
		--expected-head "$test_head" \
		--expected-state connected_entry_waiting \
		--reason lost_resume_handle
}

expect_orphan_failure() {
	local expected="$1"
	shift
	if "$@" >"$temp_root/orphan-failure.stdout" 2>"$temp_root/orphan-failure.stderr"; then
		fail "orphan cleanup unexpectedly succeeded for $expected"
	fi
	rg -q "^cleanup_category=${expected}$" "$temp_root/orphan-failure.stdout" || fail "missing orphan cleanup category $expected"
	rg -q "^phase28_attempt_error=${expected}$" "$temp_root/orphan-failure.stderr" || fail "missing orphan cleanup error $expected"
	if rg -q "$control_root|resume_handle_sha256|attempt_dir|[0-9a-f]{64}\.json" "$temp_root/orphan-failure.stdout" "$temp_root/orphan-failure.stderr"; then
		fail "orphan cleanup failure exposed a private identity"
	fi
}

setup_unique_orphan() {
	local label="$1"
	control_root="$temp_root/control-orphan-$label"
	local begin_output
	begin_output="$(begin_attempt)"
	orphan_handle="$(sed -n 's/^resume_handle=//p' <<<"$begin_output")"
	orphan_slot="$(slot_for_handle "$orphan_handle")"
	orphan_dir="$(jq -er '.attempt_dir' "$orphan_slot")"
	orphan_state="$orphan_dir/state.json"
}

mutate_orphan_state() {
	local filter="$1"
	jq --arg head "$test_head" "$filter" "$orphan_state" >"$orphan_state.next"
	mv "$orphan_state.next" "$orphan_state"
	chmod 600 "$orphan_state"
}

assert_tombstoned_and_stale() {
	local handle="$1"
	local expected_terminal="$2"
	local attempt_dir="$3"
	local slot
	slot="$(slot_for_handle "$handle")"
	[[ "$(jq -er '.terminal_category' "$slot")" == "$expected_terminal" ]] || fail "cleanup terminal category mismatch"
	[[ "$(jq -r 'keys | sort | join(",")' "$slot")" == "attempt_generation,cleanup_time_category,resume_handle_sha256,schema_version,terminal_category,terminal_status" ]] || fail "cleanup tombstone retained live references"
	[[ ! -e "$attempt_dir" ]] || fail "cleanup retained private attempt references"
	expect_failure resume_handle_stale env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "$handle"
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

prepare_post_capture_state() {
	local handle="$1"
	prepare_lifecycle_state "$handle"
	local slot
	local state_path
	slot="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
	state_path="$(jq -er '.attempt_dir' "$slot")/state.json"
	jq '
    .attempt_state="capture_complete" |
    .lifecycle_substate="complete" |
    .lifecycle_lease_id=("7"*32) |
    .lifecycle_capability_sha256=("a"*64) |
    .lifecycle_owner_pid=123 |
    .lifecycle_owner_start_fingerprint_sha256=("a"*64) |
    .lifecycle_deadline_ms=900000 |
    .attestation_status="accepted" |
    .attestation_accepted_ms=1000 |
    .usb_absence_started_ms=1000 |
    .usb_absence_ended_ms=6000 |
    .usb_absence_ms=5000 |
    .usb_absence_category="continuous_at_least_5000" |
    .restore_token_status="accepted" |
    .restore_accepted_ms=6001 |
    .usb_reappearance_ms=7000 |
    .reappearance_elapsed_ms=6000 |
    .reappearance_category="within_60000" |
    .capture_complete_permitted_lifecycle_counts |= with_entries(.value=1) |
    .armed_prohibited_sentinel_sha256=("a"*64) |
    .capture_complete_prohibited_sentinel_sha256=("a"*64) |
    .armed_permitted_lifecycle_sha256=("b"*64) |
    .capture_complete_permitted_lifecycle_sha256=("c"*64) |
    .capture_started_ms=7000 |
    .capture_ended_ms=367000 |
    .capture_duration_ms=360000 |
    .capture_category="complete_360s" |
    .lifecycle_raw_log_sha256=("a"*64) |
    .same_chain_raw_log_set_sha256=("a"*64) |
    .classifier_input_sha256=("a"*64) |
    .lifecycle_status="match" |
    .process_running=false |
    .result_correlated=false |
    .power_delta_class="flat" |
    .share_submission_status="not_observed"
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
rg -q '^capture_category=not_started$' <<<"$first_output"
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
		if [[ "$invalid" == "expired" ]]; then
			expect_failure resume_handle_stale env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "$handle"
		fi
	done
done

for effect in "${effects[@]}"; do
	[[ "$effect" != "lifecycle_start" ]] || continue
	initial_state="$(initial_state_for_effect "$effect")"
	handle="$(begin_attempt "$initial_state")"
	handle="${handle#resume_handle=}"
	effect_attempt_slot="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
	effect_attempt_dir="$(jq -er '.attempt_dir' "$effect_attempt_slot")"
	output="$(run_effect "$handle" "$effect")"
	rg -q "^effect_id=${effect}$" <<<"$output"
	rg -q '^effect_status=completed$' <<<"$output"
	if [[ "$effect" == "post_capture_detector_board_info" ]]; then
		rg -q '^## TERMINAL CLOSED$' <<<"$output"
		rg -q '^terminal_outcome=blocked_safe_evidence_invalid$' <<<"$output"
		rg -q '^verification_result=gaps_found$' <<<"$output"
		rg -q '^phase30_promotion_input=pending$' <<<"$output"
		terminal_slot="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
		[[ "$(jq -er '.terminal_category' "$terminal_slot")" == "blocked_safe_evidence_invalid" ]] || fail "successful post-capture completion did not tombstone its terminal"
		[[ ! -e "$effect_attempt_dir" ]] || fail "successful post-capture completion retained its live attempt"
		expect_failure resume_handle_stale env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "$handle"
	fi
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
	PHASE28_REAL_SOCKET_FIXTURE=1 \
	PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" \
	PHASE28_SOCKET_TRACE="$socket_trace" \
	bash "$runner" run-validated-effect --resume-handle "$lifecycle_handle" --effect-id lifecycle_start >"$temp_root/lifecycle.stdout" 2>"$temp_root/lifecycle.stderr" &
lifecycle_runner_pid=$!
wait_for_pattern '^checkpoint_id=plan13-lifecycle-removal$' "$temp_root/lifecycle.stdout"
expect_failure lease_dead_or_reused_process env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="b$(printf 'b%.0s' {1..63})" PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" PHASE28_SOCKET_TRACE="$socket_trace" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token plan13-armed-removal-v1 --response-token plan13-both-power-paths-removed
[[ ! -s "$socket_trace" ]] || fail "wrong lifecycle owner reached socket dispatch"
expect_failure checkpoint_token_mismatch env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" PHASE28_SOCKET_TRACE="$socket_trace" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token wrong --response-token plan13-both-power-paths-removed
[[ ! -s "$socket_trace" ]] || fail "wrong lifecycle token reached socket dispatch"
env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token plan13-armed-removal-v1 --response-token plan13-both-power-paths-removed >/dev/null
wait_for_pattern '^checkpoint_id=plan13-lifecycle-restore$' "$temp_root/lifecycle.stdout"
env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_TEST_MODE=1 PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" bash "$runner" deliver-token --resume-handle "$lifecycle_handle" --checkpoint-token plan13-barrel-usb-restore-v1 --response-token plan13-barrel-then-usb-restored >/dev/null
wait "$lifecycle_runner_pid"
rg -q '^effect_status=completed$' "$temp_root/lifecycle.stdout"
[[ "$(rg -c '^plan13-both-power-paths-removed$' "$socket_trace")" == "1" ]] || fail "removal frame was not accepted exactly once"
[[ "$(rg -c '^plan13-barrel-then-usb-restored$' "$socket_trace")" == "1" ]] || fail "restore frame was not accepted exactly once"
lifecycle_slot="$control_root/resume-index/$(printf '%s' "$lifecycle_handle" | shasum -a 256 | awk '{print $1}').json"
lifecycle_state="$(jq -er '.attempt_dir' "$lifecycle_slot")/state.json"
[[ "$(jq -r '.attempt_state' "$lifecycle_state")" == "capture_complete" ]]
[[ "$(jq -r '.process_running' "$lifecycle_state")" == "false" ]]
[[ "$(wc -l <"$temp_root/effects.trace" | tr -d ' ')" == "7" ]]

post_capture_outputs() {
	local correlated="$1"
	local power="$2"
	local share="$3"
	jq -cn \
		--argjson correlated "$correlated" \
		--arg power "$power" \
		--arg share "$share" \
		'{result_correlated:$correlated,power_delta_class:$power,share_submission_status:$share,lifecycle_status:"match",classifier_input_sha256:("d"*64),classifier_output_sha256:("e"*64),classifier_version:"strict-production-v2"}'
}

control_root="$temp_root/control-terminal-gaps"
gaps_handle="$(begin_attempt capture_complete)"
gaps_handle="${gaps_handle#resume_handle=}"
prepare_post_capture_state "$gaps_handle"
gaps_output="$(run_effect "$gaps_handle" post_capture_detector_board_info PHASE28_POST_CAPTURE_OUTPUTS="$(post_capture_outputs false flat not_observed)")"
rg -q '^terminal_outcome=gaps_found_same_chain_production_markers_absent$' <<<"$gaps_output"
rg -q '^verification_result=gaps_found$' <<<"$gaps_output"
rg -q '^blocker_reason=production_markers_absent$' <<<"$gaps_output"

control_root="$temp_root/control-terminal-passed"
passed_handle="$(begin_attempt capture_complete)"
passed_handle="${passed_handle#resume_handle=}"
prepare_post_capture_state "$passed_handle"
passed_output="$(run_effect "$passed_handle" post_capture_detector_board_info PHASE28_POST_CAPTURE_OUTPUTS="$(post_capture_outputs true rising_hashing accepted)")"
rg -q '^terminal_outcome=passed_same_chain_hardware$' <<<"$passed_output"
rg -q '^verification_result=passed$' <<<"$passed_output"
rg -q '^phase30_promotion_input=eligible$' <<<"$passed_output"
rg -q '^blocker_reason=none$' <<<"$passed_output"

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

classification_crash_boundaries=(
	after_classifier_intent_persistence
	after_classifier_invocation
	after_classified_persistence
	after_terminal_persistence
	before_tombstone_persistence
	after_tombstone_persistence
	after_terminal_cleanup
)
for boundary in "${classification_crash_boundaries[@]}"; do
	control_root="$temp_root/classification-crash-$boundary/control"
	mkdir -p "$(dirname "$control_root")"
	: >"$temp_root/effects.trace"
	classifier_trace="$temp_root/classifier-$boundary.trace"
	: >"$classifier_trace"
	handle="$(begin_attempt capture_complete)"
	handle="${handle#resume_handle=}"
	prepare_post_capture_state "$handle"
	expect_exit_failure run_effect "$handle" post_capture_detector_board_info \
		PHASE28_POST_CAPTURE_OUTPUTS="$(post_capture_outputs false flat not_observed)" \
		PHASE28_CLASSIFIER_TRACE="$classifier_trace" \
		PHASE28_CRASH_AT="$boundary"
	effect_count_before="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
	classifier_count_before="$(wc -l <"$classifier_trace" | tr -d ' ')"
	set +e
	run_effect "$handle" post_capture_detector_board_info \
		PHASE28_POST_CAPTURE_OUTPUTS="$(post_capture_outputs true rising_hashing accepted)" \
		PHASE28_CLASSIFIER_TRACE="$classifier_trace" >"$temp_root/classification-restart.stdout" 2>"$temp_root/classification-restart.stderr"
	set -e
	[[ "$(wc -l <"$temp_root/effects.trace" | tr -d ' ')" == "$effect_count_before" ]] || fail "$boundary redispatched the post-capture effect"
	[[ "$(wc -l <"$classifier_trace" | tr -d ' ')" == "$classifier_count_before" ]] || fail "$boundary redispatched the strict classifier"
	expect_failure resume_handle_stale env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" bash "$runner" resolve-checkpoint --resume-handle "$handle"
	tombstone="$control_root/resume-index/$(printf '%s' "$handle" | shasum -a 256 | awk '{print $1}').json"
	[[ "$(jq -r 'keys | sort | join(",")' "$tombstone")" == "attempt_generation,cleanup_time_category,resume_handle_sha256,schema_version,terminal_category,terminal_status" ]] || fail "$boundary terminal tombstone retained a live/private reference"
done

control_root="$temp_root/control-expired-resolve"
: >"$temp_root/effects.trace"
expired_resolve_handle="$(begin_attempt)"
expired_resolve_handle="$(sed -n 's/^resume_handle=//p' <<<"$expired_resolve_handle")"
expired_resolve_slot="$(slot_for_handle "$expired_resolve_handle")"
expired_resolve_dir="$(jq -er '.attempt_dir' "$expired_resolve_slot")"
expired_resolve_state="$expired_resolve_dir/state.json"
jq '.monotonic_deadline_ms=0' "$expired_resolve_state" >"$expired_resolve_state.next"
mv "$expired_resolve_state.next" "$expired_resolve_state"
chmod 600 "$expired_resolve_state"
expect_failure checkpoint_expired env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" resolve-checkpoint --resume-handle "$expired_resolve_handle"
assert_tombstoned_and_stale "$expired_resolve_handle" blocked_safe_attempt_prerequisite "$expired_resolve_dir"
[[ ! -s "$temp_root/effects.trace" ]] || fail "expired resolve invoked an effect sentinel"

control_root="$temp_root/control-expired-deliver"
expired_deliver_handle="$(begin_attempt)"
expired_deliver_handle="$(sed -n 's/^resume_handle=//p' <<<"$expired_deliver_handle")"
expired_deliver_slot="$(slot_for_handle "$expired_deliver_handle")"
expired_deliver_dir="$(jq -er '.attempt_dir' "$expired_deliver_slot")"
expired_deliver_state="$expired_deliver_dir/state.json"
jq '.monotonic_deadline_ms=0' "$expired_deliver_state" >"$expired_deliver_state.next"
mv "$expired_deliver_state.next" "$expired_deliver_state"
chmod 600 "$expired_deliver_state"
expect_failure checkpoint_expired env PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" PHASE28_ALLOW_DIRTY_TEST=1 PHASE28_TEST_HEAD="$test_head" bash "$runner" deliver-token --resume-handle "$expired_deliver_handle" --checkpoint-token malformed-before-parse --response-token malformed-before-parse
assert_tombstoned_and_stale "$expired_deliver_handle" blocked_safe_attempt_prerequisite "$expired_deliver_dir"

control_root="$temp_root/control-expired-effect"
expired_effect_handle="$(begin_attempt)"
expired_effect_handle="$(sed -n 's/^resume_handle=//p' <<<"$expired_effect_handle")"
expired_effect_slot="$(slot_for_handle "$expired_effect_handle")"
expired_effect_dir="$(jq -er '.attempt_dir' "$expired_effect_slot")"
expired_effect_state="$expired_effect_dir/state.json"
jq '.monotonic_deadline_ms=0' "$expired_effect_state" >"$expired_effect_state.next"
mv "$expired_effect_state.next" "$expired_effect_state"
chmod 600 "$expired_effect_state"
before_expired_effect_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
expect_failure checkpoint_expired run_effect "$expired_effect_handle" detector_board_info
after_expired_effect_count="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
[[ "$before_expired_effect_count" == "$after_expired_effect_count" ]] || fail "expired effect reached its adapter"
assert_tombstoned_and_stale "$expired_effect_handle" blocked_safe_attempt_prerequisite "$expired_effect_dir"

control_root="$temp_root/control-cleanup-head-mismatch"
cleanup_head_handle="$(begin_attempt connected_entry_waiting)"
cleanup_head_handle="${cleanup_head_handle#resume_handle=}"
cleanup_head_slot="$(slot_for_handle "$cleanup_head_handle")"
cleanup_head_dir="$(jq -er '.attempt_dir' "$cleanup_head_slot")"
cleanup_output="$(cleanup_attempt "$cleanup_head_handle" PHASE28_TEST_HEAD="2222222222222222222222222222222222222222")"
rg -q '^cleanup_result=closed$' <<<"$cleanup_output"
rg -q '^terminal_outcome=blocked_safe_attempt_prerequisite$' <<<"$cleanup_output"
rg -q '^effect_sentinels_invoked=0$' <<<"$cleanup_output"
rg -q '^positive_evidence_promoted=false$' <<<"$cleanup_output"
assert_tombstoned_and_stale "$cleanup_head_handle" blocked_safe_attempt_prerequisite "$cleanup_head_dir"
expect_failure resume_handle_stale cleanup_attempt "$cleanup_head_handle"

control_root="$temp_root/control-cleanup-boot-mismatch"
cleanup_boot_handle="$(begin_attempt connected_entry_waiting)"
cleanup_boot_handle="${cleanup_boot_handle#resume_handle=}"
cleanup_boot_slot="$(slot_for_handle "$cleanup_boot_handle")"
cleanup_boot_dir="$(jq -er '.attempt_dir' "$cleanup_boot_slot")"
cleanup_boot_state="$cleanup_boot_dir/state.json"
jq '.boot_session_sha256=("f"*64)' "$cleanup_boot_state" >"$cleanup_boot_state.next"
mv "$cleanup_boot_state.next" "$cleanup_boot_state"
chmod 600 "$cleanup_boot_state"
jq '.boot_session_sha256=("f"*64)' "$cleanup_boot_slot" >"$cleanup_boot_slot.next"
mv "$cleanup_boot_slot.next" "$cleanup_boot_slot"
chmod 600 "$cleanup_boot_slot"
cleanup_attempt "$cleanup_boot_handle" >/dev/null
assert_tombstoned_and_stale "$cleanup_boot_handle" blocked_safe_attempt_prerequisite "$cleanup_boot_dir"

control_root="$temp_root/control-cleanup-unresolved"
cleanup_unresolved_handle="$(begin_attempt connected_entry_waiting)"
cleanup_unresolved_handle="${cleanup_unresolved_handle#resume_handle=}"
cleanup_unresolved_slot="$(slot_for_handle "$cleanup_unresolved_handle")"
cleanup_unresolved_dir="$(jq -er '.attempt_dir' "$cleanup_unresolved_slot")"
sleep 30 &
unowned_child_pid=$!
printf '%s\n' "$unowned_child_pid" >"$cleanup_unresolved_dir/child.pid"
chmod 600 "$cleanup_unresolved_dir/child.pid"
cleanup_unresolved_output="$(cleanup_attempt "$cleanup_unresolved_handle")"
rg -q '^terminal_outcome=blocked_safe_unresolved_process$' <<<"$cleanup_unresolved_output"
assert_tombstoned_and_stale "$cleanup_unresolved_handle" blocked_safe_unresolved_process "$cleanup_unresolved_dir"
kill "$unowned_child_pid" 2>/dev/null || true
wait "$unowned_child_pid" 2>/dev/null || true

control_root="$temp_root/control-cleanup-crash"
cleanup_crash_handle="$(begin_attempt connected_entry_waiting)"
cleanup_crash_handle="${cleanup_crash_handle#resume_handle=}"
cleanup_crash_slot="$(slot_for_handle "$cleanup_crash_handle")"
cleanup_crash_dir="$(jq -er '.attempt_dir' "$cleanup_crash_slot")"
expect_exit_failure cleanup_attempt "$cleanup_crash_handle" PHASE28_CRASH_AT=before_tombstone_persistence
[[ "$(jq -er '.status' "$cleanup_crash_slot")" == "active" ]] || fail "cleanup crash lost its recoverable active mapping"
cleanup_attempt "$cleanup_crash_handle" >/dev/null
assert_tombstoned_and_stale "$cleanup_crash_handle" blocked_safe_attempt_prerequisite "$cleanup_crash_dir"

control_root="$temp_root/control-cleanup-after-tombstone-crash"
cleanup_after_tombstone_handle="$(begin_attempt connected_entry_waiting)"
cleanup_after_tombstone_handle="${cleanup_after_tombstone_handle#resume_handle=}"
cleanup_after_tombstone_slot="$(slot_for_handle "$cleanup_after_tombstone_handle")"
cleanup_after_tombstone_dir="$(jq -er '.attempt_dir' "$cleanup_after_tombstone_slot")"
expect_exit_failure cleanup_attempt "$cleanup_after_tombstone_handle" PHASE28_CRASH_AT=after_tombstone_persistence
assert_tombstoned_and_stale "$cleanup_after_tombstone_handle" blocked_safe_attempt_prerequisite "$cleanup_after_tombstone_dir"

control_root="$temp_root/control-orphan-zero"
mkdir -p "$control_root/resume-index" "$control_root/attempts"
chmod 700 "$control_root" "$control_root/resume-index" "$control_root/attempts"
expect_orphan_failure no_active_orphan cleanup_unique_orphan
rg -q '^active_orphan_count_category=zero$' "$temp_root/orphan-failure.stdout"
rg -q '^state_mutated=false$' "$temp_root/orphan-failure.stdout"

setup_unique_orphan multiple
first_orphan_state="$orphan_state"
first_orphan_slot="$orphan_slot"
begin_attempt >"$temp_root/second-orphan-private.out"
first_state_before="$(shasum -a 256 "$first_orphan_state" | awk '{print $1}')"
first_slot_before="$(shasum -a 256 "$first_orphan_slot" | awk '{print $1}')"
expect_orphan_failure orphan_index_ambiguous cleanup_unique_orphan
[[ "$first_state_before" == "$(shasum -a 256 "$first_orphan_state" | awk '{print $1}')" ]] || fail "multiple-orphan rejection mutated state"
[[ "$first_slot_before" == "$(shasum -a 256 "$first_orphan_slot" | awk '{print $1}')" ]] || fail "multiple-orphan rejection mutated its mapping"

setup_unique_orphan collision
collision_digest="$(jq -er '.resume_handle_sha256' "$orphan_slot")"
collision_slot="$control_root/resume-index/$(printf 'f%.0s' {1..64}).json"
jq -cn --arg digest "$collision_digest" '{schema_version:"exact-head-resume-tombstone-v1",resume_handle_sha256:$digest,attempt_generation:1,terminal_status:"closed",terminal_category:"blocked_safe_attempt_prerequisite",cleanup_time_category:"abandoned"}' >"$collision_slot"
chmod 600 "$collision_slot"
expect_orphan_failure orphan_index_ambiguous cleanup_unique_orphan
[[ -f "$orphan_state" ]] || fail "active/tombstone collision rejection mutated state"

setup_unique_orphan wrong-head
expect_orphan_failure orphan_state_ineligible env \
	PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
	PHASE28_ALLOW_DIRTY_TEST=1 \
	bash "$runner" cleanup-unique-orphan-attempt \
	--expected-head "2222222222222222222222222222222222222222" \
	--expected-state connected_entry_waiting \
	--reason lost_resume_handle
[[ -f "$orphan_state" ]] || fail "wrong-head rejection mutated state"

setup_unique_orphan wrong-boot
mutate_orphan_state '.boot_session_sha256=("f"*64)'
jq '.boot_session_sha256=("f"*64)' "$orphan_slot" >"$orphan_slot.next"
mv "$orphan_slot.next" "$orphan_slot"
chmod 600 "$orphan_slot"
expect_orphan_failure orphan_state_ineligible cleanup_unique_orphan
[[ -f "$orphan_state" ]] || fail "wrong-boot rejection mutated state"

setup_unique_orphan wrong-state
mutate_orphan_state '.attempt_state="recovery_waiting"'
expect_orphan_failure orphan_state_ineligible cleanup_unique_orphan

orphan_state_mutations=(
	'.detector_attempt_count=1'
	'.usb_replug_count=1'
	'.both_power_count=1'
	'.reset_count=1'
	'.boot_reset_count=1'
	'.effect_sequence=1'
	'.effect_id="detector_board_info" | .effect_phase="authorized" | .effect_authorization_nonce=("a"*32) | .effect_sequence=1'
	'.armed_prohibited_sentinel_counts.detector_board_info_effect_count=1'
	'.capture_complete_prohibited_sentinel_counts.detector_board_info_effect_count=1'
	'.armed_permitted_lifecycle_counts.removal_pty_write_count=1'
	'.capture_complete_permitted_lifecycle_counts.removal_pty_write_count=1'
	'.wifi_credential_state="present"'
	'.wifi_credential_binding_id=("a"*32)'
	'.reference_guard_state="pass" | .reference_commit=("b"*40) | .reference_guard_output_sha256=("c"*64)'
	'.selected_port_state="one_board205" | .selected_port_fingerprint_sha256=("c"*64)'
	'.manifest_state="pass" | .manifest_source_commit=("1"*40) | .manifest_sha256=("c"*64) | .factory_image_sha256=("d"*64)'
	'.reinit_capture_started_ms=1'
	'.capture_category="running" | .capture_started_ms=1'
	'.classifier_version="strict-production-v2"'
	'.classification_phase="completed"'
	'.result_correlated=false'
	'.process_running=true'
	'.lifecycle_lease_id=("a"*32)'
)
orphan_mutation_index=0
for orphan_mutation in "${orphan_state_mutations[@]}"; do
	setup_unique_orphan "ineligible-$orphan_mutation_index"
	mutate_orphan_state "$orphan_mutation"
	state_before="$(shasum -a 256 "$orphan_state" | awk '{print $1}')"
	expect_orphan_failure orphan_state_ineligible cleanup_unique_orphan
	[[ "$state_before" == "$(shasum -a 256 "$orphan_state" | awk '{print $1}')" ]] || fail "ineligible orphan mutation changed state"
	orphan_mutation_index=$((orphan_mutation_index + 1))
done

setup_unique_orphan capability-present
printf '{}\n' >"$orphan_dir/credential-capability.json"
chmod 600 "$orphan_dir/credential-capability.json"
expect_orphan_failure orphan_live_reference_present cleanup_unique_orphan

setup_unique_orphan pid-present
printf '%s\n' "$$" >"$orphan_dir/child.pid"
chmod 600 "$orphan_dir/child.pid"
expect_orphan_failure orphan_live_reference_present cleanup_unique_orphan

setup_unique_orphan socket-present
(cd "$orphan_dir" && node -e 'const fs=require("node:fs"),net=require("node:net"); const server=net.createServer(()=>{}); server.listen("lifecycle.sock",()=>fs.chmodSync("lifecycle.sock",0o600)); setInterval(()=>{},1000);') &
orphan_socket_pid=$!
for _ in $(seq 1 100); do
	[[ -S "$orphan_dir/lifecycle.sock" ]] && break
	sleep 0.01
done
[[ -S "$orphan_dir/lifecycle.sock" ]] || fail "orphan socket fixture did not start"
expect_orphan_failure orphan_live_reference_present cleanup_unique_orphan
kill "$orphan_socket_pid" 2>/dev/null || true
wait "$orphan_socket_pid" 2>/dev/null || true

setup_unique_orphan malformed-schema
mutate_orphan_state '.schema_version="wrong"'
expect_orphan_failure orphan_state_ineligible cleanup_unique_orphan

setup_unique_orphan malformed-mode
chmod 644 "$orphan_slot"
expect_orphan_failure orphan_index_ambiguous cleanup_unique_orphan

setup_unique_orphan crash-before-tombstone
expect_exit_failure cleanup_unique_orphan PHASE28_CRASH_AT=before_tombstone_persistence
[[ "$(jq -er '.attempt_state' "$orphan_state")" == "terminal" ]] || fail "orphan cleanup crash did not preserve terminal recovery state"
crash_recovery_output="$(cleanup_unique_orphan)"
rg -q '^cleanup_result=closed$' <<<"$crash_recovery_output"
rg -q '^terminal_category=blocked_safe_attempt_prerequisite$' <<<"$crash_recovery_output"
[[ ! -e "$orphan_dir" ]] || fail "orphan cleanup crash recovery retained live references"

setup_unique_orphan crash-after-tombstone
expect_exit_failure cleanup_unique_orphan PHASE28_CRASH_AT=after_tombstone_persistence
[[ ! -e "$orphan_dir" ]] || fail "post-tombstone crash retained live references"
expect_orphan_failure no_active_orphan cleanup_unique_orphan

setup_unique_orphan crash-after-cleanup
expect_exit_failure cleanup_unique_orphan PHASE28_CRASH_AT=after_terminal_cleanup
[[ ! -e "$orphan_dir" ]] || fail "post-cleanup crash retained live references"
expect_orphan_failure no_active_orphan cleanup_unique_orphan

setup_unique_orphan exact-success
: >"$temp_root/orphan-adapter.trace"
historical_digest="$(printf 'e%.0s' {1..64})"
historical_slot="$control_root/resume-index/$historical_digest.json"
jq -cn --arg digest "$historical_digest" '{schema_version:"exact-head-resume-tombstone-v1",resume_handle_sha256:$digest,attempt_generation:1,terminal_status:"closed",terminal_category:"blocked_safe_attempt_prerequisite",cleanup_time_category:"abandoned"}' >"$historical_slot"
chmod 600 "$historical_slot"
orphan_tombstone_count_before="$(find "$control_root/resume-index" -type f -name '*.json' | wc -l | tr -d ' ')"
valid_tombstone_count_before="$(jq -s '[.[] | select(.schema_version == "exact-head-resume-tombstone-v1")] | length' "$control_root"/resume-index/*.json)"
orphan_success_output="$(cleanup_unique_orphan PHASE28_EFFECT_TRACE="$temp_root/orphan-adapter.trace")"
rg -q '^cleanup_result=closed$' <<<"$orphan_success_output"
rg -q '^terminal_category=blocked_safe_attempt_prerequisite$' <<<"$orphan_success_output"
rg -q '^active_orphan_count_category=zero$' <<<"$orphan_success_output"
rg -q '^tombstone_count_increased=true$' <<<"$orphan_success_output"
rg -q '^live_reference_count_category=zero$' <<<"$orphan_success_output"
rg -q '^process_running=false$' <<<"$orphan_success_output"
rg -q '^effect_sentinels_invoked=0$' <<<"$orphan_success_output"
rg -q '^positive_evidence_promoted=false$' <<<"$orphan_success_output"
rg -q '^resume_handle_reconstructed=false$' <<<"$orphan_success_output"
if rg -q "$control_root|resume_handle_sha256|attempt_dir|[0-9a-f]{64}\.json" <<<"$orphan_success_output"; then
	fail "successful orphan cleanup exposed a private identity"
fi
[[ ! -s "$temp_root/orphan-adapter.trace" ]] || fail "orphan cleanup touched an adapter sentinel"
[[ ! -e "$orphan_dir" ]] || fail "successful orphan cleanup retained its attempt directory"
[[ "$(jq -r 'keys | sort | join(",")' "$orphan_slot")" == "attempt_generation,cleanup_time_category,resume_handle_sha256,schema_version,terminal_category,terminal_status" ]] || fail "orphan cleanup tombstone retained live references"
[[ "$(jq -er '.terminal_category' "$orphan_slot")" == "blocked_safe_attempt_prerequisite" ]] || fail "orphan cleanup tombstone category mismatch"
orphan_tombstone_count_after="$(find "$control_root/resume-index" -type f -name '*.json' | wc -l | tr -d ' ')"
valid_tombstone_count_after="$(jq -s '[.[] | select(.schema_version == "exact-head-resume-tombstone-v1")] | length' "$control_root"/resume-index/*.json)"
[[ "$orphan_tombstone_count_after" == "$((orphan_tombstone_count_before + 0))" ]] || fail "orphan cleanup did not replace its active mapping in place"
[[ "$valid_tombstone_count_after" == "$((valid_tombstone_count_before + 1))" ]] || fail "orphan cleanup did not increase the minimal tombstone count"
expect_orphan_failure no_active_orphan cleanup_unique_orphan
rg -q '^state_mutated=false$' "$temp_root/orphan-failure.stdout"

control_root="$temp_root/control-lifecycle-expiry"
socket_trace="$temp_root/lifecycle-expiry-socket.trace"
: >"$socket_trace"
lifecycle_expiry_handle="$(begin_attempt reinit_validated)"
lifecycle_expiry_handle="${lifecycle_expiry_handle#resume_handle=}"
prepare_lifecycle_state "$lifecycle_expiry_handle"
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
	bash "$runner" run-validated-effect --resume-handle "$lifecycle_expiry_handle" --effect-id lifecycle_start >"$temp_root/lifecycle-expiry.stdout" 2>"$temp_root/lifecycle-expiry.stderr" &
lifecycle_expiry_runner_pid=$!
wait_for_pattern '^checkpoint_id=plan13-lifecycle-removal$' "$temp_root/lifecycle-expiry.stdout"
lifecycle_expiry_slot="$(slot_for_handle "$lifecycle_expiry_handle")"
lifecycle_expiry_dir="$(jq -er '.attempt_dir' "$lifecycle_expiry_slot")"
lifecycle_expiry_state="$lifecycle_expiry_dir/state.json"
jq '.lifecycle_deadline_ms=500 | .monotonic_deadline_ms=1000000' "$lifecycle_expiry_state" >"$lifecycle_expiry_state.next"
mv "$lifecycle_expiry_state.next" "$lifecycle_expiry_state"
chmod 600 "$lifecycle_expiry_state"
printf '499\n499\n500\n' >"$temp_root/monotonic-sequence.txt"
before_lifecycle_expiry_effects="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
expect_failure checkpoint_expired env \
	PHASE28_ATTEMPT_CONTROL_ROOT="$control_root" \
	PHASE28_TEST_MODE=1 \
	PHASE28_ALLOW_DIRTY_TEST=1 \
	PHASE28_TEST_HEAD="$test_head" \
	PHASE28_TEST_OWNER_FINGERPRINT="a$(printf 'a%.0s' {1..63})" \
	PHASE28_TEST_SOCKET_SEND_BIN="$socket_sender" \
	PHASE28_SOCKET_TRACE="$socket_trace" \
	PHASE28_MONOTONIC_MS_BIN="$monotonic_sequence" \
	PHASE28_MONOTONIC_SEQUENCE_FILE="$temp_root/monotonic-sequence.txt" \
	bash "$runner" deliver-token --resume-handle "$lifecycle_expiry_handle" --checkpoint-token plan13-armed-removal-v1 --response-token plan13-both-power-paths-removed
set +e
wait "$lifecycle_expiry_runner_pid"
set -e
after_lifecycle_expiry_effects="$(wc -l <"$temp_root/effects.trace" | tr -d ' ')"
[[ "$before_lifecycle_expiry_effects" == "$after_lifecycle_expiry_effects" ]] || fail "lease expiry dispatched another effect"
[[ ! -s "$socket_trace" ]] || fail "deadline crossed after parse but still reached the lifecycle socket"
assert_tombstoned_and_stale "$lifecycle_expiry_handle" blocked_safe_evidence_invalid "$lifecycle_expiry_dir"

control_root="$temp_root/control-orphan-receiver"
orphan_receiver_handle="$(begin_attempt reinit_validated)"
orphan_receiver_handle="${orphan_receiver_handle#resume_handle=}"
prepare_lifecycle_state "$orphan_receiver_handle"
orphan_receiver_pid_file="$temp_root/orphan-receiver.pid"
orphan_receiver_output="$(run_effect "$orphan_receiver_handle" lifecycle_start \
	PHASE28_ORPHAN_RECEIVER_FIXTURE=1 \
	PHASE28_ORPHAN_RECEIVER_PID_FILE="$orphan_receiver_pid_file")"
orphan_receiver_pid="$(sed -n '1p' "$orphan_receiver_pid_file")"
[[ "$orphan_receiver_pid" =~ ^[1-9][0-9]*$ ]] || fail "orphan receiver fixture did not report a PID"
if kill -0 "$orphan_receiver_pid" 2>/dev/null; then
	fail "lifecycle receiver survived terminal process-group cleanup"
fi
orphan_receiver_pid=""
rg -q '^terminal_outcome=blocked_safe_evidence_invalid$' <<<"$orphan_receiver_output"
orphan_receiver_slot="$(slot_for_handle "$orphan_receiver_handle")"
[[ "$(jq -er '.terminal_category' "$orphan_receiver_slot")" == "blocked_safe_evidence_invalid" ]] || fail "orphan receiver cleanup terminal category mismatch"

expect_failure resume_handle_malformed cleanup_attempt malformed
expect_failure resume_handle_wrong cleanup_attempt "f$(printf '0%.0s' {1..63})"
[[ ! -e "$control_root/control.lock" ]] || fail "failed continuation retained the private control lock"

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
rg -q 'lifecycle_frame_helper=.*phase28.1.1-lifecycle-frame.pl' "$runner" || fail "runner lacks the shared lifecycle framing helper"
if rg -q 'IO::Socket::UNIX|length\(\$frame\).*\\n' "$runner" "$repo_root/scripts/phase28.1.1-accepted-state-diagnostic.sh"; then
	fail "runtime scripts retain inline lifecycle framing logic"
fi
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
