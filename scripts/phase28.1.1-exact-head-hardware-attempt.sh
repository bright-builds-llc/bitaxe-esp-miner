#!/usr/bin/env bash
# Crash-closed private control plane for one exact-HEAD Phase 28.1.1 attempt.
set -euo pipefail
umask 077

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=/dev/null
source "$repo_root/scripts/process-group.sh"
readonly private_root="${PHASE28_ATTEMPT_CONTROL_ROOT:-$repo_root/hardware-runs/phase28.1.1/attempt-control}"
readonly resume_index="$private_root/resume-index"
readonly attempts_root="$private_root/attempts"
readonly lock_dir="$private_root/control.lock"
readonly state_module="$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs"
readonly classifier_module="$repo_root/scripts/phase28.1.1-strict-production-evidence.mjs"
readonly production_adapter="$repo_root/scripts/phase28.1.1-accepted-state-diagnostic.sh"
readonly max_handle_attempts=8
lock_held=false
active_effect_pid=""
effect_adapter_command=()

terminate_effect_process() {
	local pid="$1"
	local label="$2"
	if ! kill -0 "$pid" 2>/dev/null; then
		return 0
	fi
	local pgid
	pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')"
	[[ "$pgid" == "$pid" ]] || return 1
	phase_process_group_terminate "$pid" "$label"
}

cleanup_on_exit() {
	if [[ -n "$active_effect_pid" ]] && kill -0 "$active_effect_pid" 2>/dev/null; then
		terminate_effect_process "$active_effect_pid" "phase28 effect cleanup" >/dev/null 2>&1 || true
	fi
	if [[ "$lock_held" == "true" ]]; then
		rmdir "$lock_dir" 2>/dev/null || true
	fi
}

trap cleanup_on_exit EXIT
trap 'cleanup_on_exit; exit 130' INT TERM

die() {
	printf 'phase28_attempt_error=%s\n' "$1" >&2
	exit 1
}

usage() {
	printf 'usage: %s begin-attempt|resolve-checkpoint|deliver-token|run-validated-effect [options]\n' "$(basename "$0")"
}

ensure_private_root() {
	mkdir -p "$private_root" "$resume_index" "$attempts_root"
	chmod 700 "$private_root" "$resume_index" "$attempts_root"
}

acquire_lock() {
	local attempts=0
	while ! mkdir -m 700 "$lock_dir" 2>/dev/null; do
		if [[ -s "$lock_dir/owner.pid" ]]; then
			local owner_pid
			owner_pid="$(sed -n '1p' "$lock_dir/owner.pid")"
			if [[ "$owner_pid" =~ ^[0-9]+$ ]] && ! kill -0 "$owner_pid" 2>/dev/null; then
				rm -rf "$lock_dir"
				continue
			fi
		fi
		attempts=$((attempts + 1))
		((attempts < 100)) || die "lock_failure"
		sleep 0.01
	done
	printf '%s\n' "$$" >"$lock_dir/owner.pid"
	lock_held=true
}

release_lock() {
	rm -f "$lock_dir/owner.pid"
	rmdir "$lock_dir" 2>/dev/null || die "lock_failure"
	lock_held=false
}

sync_path() {
	perl -e 'open my $fh, "<", $ARGV[0] or die $!; $fh->sync or die $!' "$1"
}

mode_of_path() {
	if stat -f '%Lp' "$1" >/dev/null 2>&1; then
		stat -f '%Lp' "$1"
		return
	fi
	stat -c '%a' "$1"
}

atomic_write() {
	local destination="$1"
	local contents="$2"
	local parent
	local temporary
	parent="$(dirname "$destination")"
	temporary="$(mktemp "$parent/.phase28-write.XXXXXX")"
	printf '%s\n' "$contents" >"$temporary"
	chmod 600 "$temporary"
	sync_path "$temporary"
	mv -f "$temporary" "$destination"
	sync_path "$parent"
}

random_hex() {
	local bytes="$1"
	if [[ -n "${PHASE28_RANDOM_HEX_BIN:-}" ]]; then
		"$PHASE28_RANDOM_HEX_BIN" "$bytes"
		return
	fi
	openssl rand -hex "$bytes"
}

sha256_text() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

monotonic_ms() {
	if [[ -n "${PHASE28_MONOTONIC_MS_BIN:-}" ]]; then
		"$PHASE28_MONOTONIC_MS_BIN"
		return
	fi
	perl -MTime::HiRes=clock_gettime,CLOCK_MONOTONIC -e 'printf "%.0f\n", clock_gettime(CLOCK_MONOTONIC) * 1000'
}

current_head() {
	if [[ -n "${PHASE28_TEST_HEAD:-}" ]]; then
		printf '%s\n' "$PHASE28_TEST_HEAD"
		return
	fi
	git -C "$repo_root" rev-parse HEAD
}

require_clean_head() {
	if [[ "${PHASE28_ALLOW_DIRTY_TEST:-0}" != "1" ]] && [[ -n "$(git -C "$repo_root" status --porcelain=v1)" ]]; then
		die "dirty_head"
	fi
}

observe_boot_digest() {
	node --input-type=module -e 'const { observeBootSessionDigest } = await import(process.argv[1]); process.stdout.write(observeBootSessionDigest())' "$state_module"
}

validate_state() {
	local state_path="$1"
	node --input-type=module -e 'import fs from "node:fs"; const { validateAttemptState } = await import(process.argv[1]); validateAttemptState(JSON.parse(fs.readFileSync(process.argv[2], "utf8")))' "$state_module" "$state_path"
}

create_state_json() {
	local exact_head="$1"
	local resume_digest="$2"
	local created_ms="$3"
	node --input-type=module -e 'const { createConnectedEntryState } = await import(process.argv[1]); const state=createConnectedEntryState({exactHead:process.argv[2],resumeHandleSha256:process.argv[3],createdMonotonicMs:Number(process.argv[4])}); process.stdout.write(JSON.stringify(state))' "$state_module" "$exact_head" "$resume_digest" "$created_ms"
}

state_operation() {
	local operation="$1"
	local state_path="$2"
	shift 2
	local args_json="${1:-}"
	if [[ -z "$args_json" ]]; then
		args_json='{}'
	fi
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const operation = process.argv[2];
const state = JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
const args = JSON.parse(process.argv[4]);
let next;
switch (operation) {
  case "consume-checkpoint": next = authority.consumeCheckpoint(state, args); break;
  case "apply-effect-result": next = authority.applyEffectCompletion(state, JSON.parse(fs.readFileSync(args.resultPath, "utf8")), { completedMonotonicMs: args.completedMonotonicMs }); break;
  case "attach-lifecycle": next = authority.attachLifecycleOwner(state, args); break;
  case "lifecycle-event": next = authority.applyLifecycleOwnerEvent(state, args.event, args.values); break;
  case "terminalize": next = authority.terminalizeAttempt(state, args.blockerReason, { cleanupUnresolved: args.cleanupUnresolved }); break;
  case "mark-classifier-invoked": next = authority.markClassifierInvoked(state); break;
  case "classify": {
    const strict = await import(process.argv[5]);
    next = authority.persistStrictClassification(state, strict.classifyStrictPostCaptureState(state));
    break;
  }
  case "finalize-classified": next = authority.finalizeClassifiedAttempt(state); break;
  default: throw new Error("unknown state operation");
}
process.stdout.write(JSON.stringify(next));' "$state_module" "$operation" "$state_path" "$args_json" "$classifier_module"
}

public_checkpoint_output() {
	local state_path="$1"
	local handle="$2"
	# shellcheck disable=SC2016
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const checkpoint = authority.publicCheckpoint(state, process.argv[3]);
process.stdout.write("## CHECKPOINT REACHED\n");
for (const field of authority.PUBLIC_CHECKPOINT_FIELDS) process.stdout.write(`${field}=${checkpoint[field]}\n`);' "$state_module" "$state_path" "$handle"
}

public_terminal_output() {
	local state_path="$1"
	jq -er '
    select(.attempt_state == "terminal") |
    "## TERMINAL CLOSED\n" +
    "terminal_outcome=\(.terminal_outcome)\n" +
    "verification_result=\(.verification_result)\n" +
    "phase30_promotion_input=\(.phase30_promotion_input)\n" +
    "blocker_reason=\(.blocker_reason)\n" +
    "classifier_input_sha256=\(.classifier_input_sha256 // "not_run")\n" +
    "classifier_output_sha256=\(.classifier_output_sha256 // "not_run")"
  ' "$state_path"
}

transform_effect_state() {
	local operation="$1"
	local state_path="$2"
	local effect_id="${3:-}"
	local nonce="${4:-}"
	local sequence="${5:-0}"
	local succeeded="${6:-true}"
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state=JSON.parse(fs.readFileSync(process.argv[3], "utf8"));
let next;
switch(process.argv[2]) {
  case "authorize": next=authority.authorizeEffect(state, process.argv[4], process.argv[5]); break;
  case "invoke": next=authority.markEffectInvoked(state, process.argv[5], Number(process.argv[6])); break;
  case "finish": next=authority.finishEffect(state, process.argv[5], Number(process.argv[6]), process.argv[7] === "true"); break;
  case "normalize": next=authority.normalizeFinishedEffect(state); break;
  default: throw new Error("unknown state operation");
}
process.stdout.write(JSON.stringify(next));' "$state_module" "$operation" "$state_path" "$effect_id" "$nonce" "$sequence" "$succeeded"
}

assert_fresh_identity() {
	local state_path="$1"
	require_clean_head
	local head
	local persisted_head
	local observed_boot
	local persisted_boot
	head="$(current_head)"
	persisted_head="$(jq -er '.exact_head' "$state_path")"
	[[ "$head" == "$persisted_head" ]] || die "exact_head_mismatch"
	observed_boot="$(observe_boot_digest)" || die "boot_session_observation_unavailable"
	persisted_boot="$(jq -er '.boot_session_sha256' "$state_path")"
	[[ "$observed_boot" == "$persisted_boot" ]] || die "boot_session_mismatch"
}

validate_active_slot() {
	local slot="$1"
	local digest="$2"
	jq -e --arg digest "$digest" '
    type == "object" and
    (keys | sort) == (["attempt_dir","attempt_generation","attempt_id","boot_session_sha256","checkpoint_generation","resume_handle_sha256","schema_version","status"] | sort) and
    .schema_version == "exact-head-resume-active-v1" and
    .status == "active" and
    .resume_handle_sha256 == $digest and
    (.attempt_dir | type == "string") and
    (.attempt_id | test("^[0-9a-f]{32}$")) and
    (.boot_session_sha256 | test("^[0-9a-f]{64}$")) and
    (.attempt_generation | type == "number") and
    (.checkpoint_generation | type == "number")
  ' "$slot" >/dev/null
}

validate_tombstone() {
	local slot="$1"
	local digest="$2"
	jq -e --arg digest "$digest" '
    type == "object" and
    (keys | sort) == (["attempt_generation","cleanup_time_category","resume_handle_sha256","schema_version","terminal_category","terminal_status"] | sort) and
    .schema_version == "exact-head-resume-tombstone-v1" and
    .resume_handle_sha256 == $digest and
    .terminal_status == "closed" and
    (.terminal_category | IN("blocked_safe_attempt_prerequisite","blocked_safe_unresolved_process","blocked_safe_evidence_invalid","gaps_found_same_chain_production_markers_absent","passed_same_chain_hardware")) and
    (.cleanup_time_category | IN("normal","blocked","crash","abandoned"))
  ' "$slot" >/dev/null
}

validate_slot_state() {
	local slot="$1"
	local state_path="$2"
	local attempt_dir
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	[[ "$(dirname "$attempt_dir")" == "$attempts_root" ]] || die "resume_handle_ambiguous"
	[[ "$(basename "$attempt_dir")" == "$(jq -er '.attempt_id' "$slot")" ]] || die "resume_handle_ambiguous"
	[[ "$state_path" == "$attempt_dir/state.json" && -f "$state_path" ]] || die "resume_handle_ambiguous"
	[[ "$(mode_of_path "$slot")" == "600" && "$(mode_of_path "$state_path")" == "600" ]] || die "private_capability_invalid"
	[[ "$(jq -er '.attempt_id' "$slot")" == "$(jq -er '.attempt_id' "$state_path")" ]] || die "resume_handle_ambiguous"
	[[ "$(jq -er '.boot_session_sha256' "$slot")" == "$(jq -er '.boot_session_sha256' "$state_path")" ]] || die "resume_handle_ambiguous"
	[[ "$(jq -er '.checkpoint_generation' "$slot")" == "$(jq -er '.checkpoint_generation' "$state_path")" ]] || die "checkpoint_generation_mismatch"
}

persist_state_and_slot() {
	local state_path="$1"
	local slot="$2"
	local state_json="$3"
	atomic_write "$state_path" "$state_json"
	local slot_json
	slot_json="$(jq -c --argjson generation "$(jq -er '.checkpoint_generation' "$state_path")" --arg boot "$(jq -er '.boot_session_sha256' "$state_path")" '.checkpoint_generation=$generation | .boot_session_sha256=$boot' "$slot")"
	atomic_write "$slot" "$slot_json"
}

process_start_fingerprint() {
	local pid="$1"
	if [[ -n "${PHASE28_TEST_OWNER_FINGERPRINT:-}" ]]; then
		printf '%s\n' "$PHASE28_TEST_OWNER_FINGERPRINT"
		return
	fi
	local started
	started="$(ps -o lstart= -p "$pid" 2>/dev/null)"
	[[ -n "$started" ]] || die "lease_dead_or_reused_process"
	sha256_text "pid-start-v1\0${pid}\0${started}"
}

effect_failure_blocker() {
	case "$1" in
	detector_board_info) printf 'detector_failed\n' ;;
	credential_presence_bind) printf 'credential_binding_failed\n' ;;
	reference_guard) printf 'reference_guard_failed\n' ;;
	package) printf 'package_failed\n' ;;
	flash_reinit_runtime) printf 'reinit_capture_failed\n' ;;
	lifecycle_start) printf 'lifecycle_capture_failed\n' ;;
	post_capture_detector_board_info) printf 'post_capture_detector_failed\n' ;;
	esac
}

write_failed_effect_result() {
	local path="$1"
	local effect="$2"
	local blocker="$3"
	atomic_write "$path" "$(jq -cn --arg effect "$effect" --arg blocker "$blocker" '{schema_version:"exact-head-effect-result-v1",effect_id:$effect,status:"failed",blocker_reason:$blocker,outputs:{}}')"
}

adapter_command() {
	local effect="$1"
	if [[ "${PHASE28_TEST_MODE:-0}" == "1" ]]; then
		local adapter="${PHASE28_TEST_EFFECT_ADAPTER_DIR:-}/$effect"
		[[ -x "$adapter" ]] || die "validator_error"
		effect_adapter_command=("$adapter")
		return
	fi
	[[ -z "${PHASE28_TEST_EFFECT_ADAPTER_DIR:-}" ]] || die "validator_error"
	[[ -x "$production_adapter" ]] || die "validator_error"
	effect_adapter_command=("$production_adapter" --mode plan13-prevalidated --effect-id "$effect")
}

resolve_handle() {
	local handle="${1:-}"
	[[ -n "$handle" ]] || die "resume_handle_missing"
	[[ "$handle" =~ ^[0-9a-f]{64}$ ]] || die "resume_handle_malformed"
	local digest
	local slot
	digest="$(sha256_text "$handle")"
	slot="$resume_index/$digest.json"
	[[ -f "$slot" ]] || die "resume_handle_wrong"
	if validate_active_slot "$slot" "$digest"; then
		printf '%s\n' "$slot"
		return
	fi
	if validate_tombstone "$slot" "$digest"; then
		die "resume_handle_stale"
	fi
	die "resume_handle_ambiguous"
}

begin_attempt() {
	local generation=1
	local hardware_exact_head=""
	while (($#)); do
		case "$1" in
		--hardware-exact-head)
			hardware_exact_head="${2:-}"
			shift 2
			;;
		--attempt-generation)
			generation="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	[[ "$generation" =~ ^[0-9]+$ ]] || die "state_malformed"
	[[ "$hardware_exact_head" =~ ^[0-9a-f]{40}$ ]] || die "exact_head_mismatch"
	ensure_private_root
	require_clean_head
	[[ "$(current_head)" == "$hardware_exact_head" ]] || die "exact_head_mismatch"
	acquire_lock
	trap release_lock RETURN
	local exact_head
	local created_ms
	local handle
	local digest
	local slot
	local attempt_id
	local attempt_dir
	local state_json
	local boot_digest
	exact_head="$hardware_exact_head"
	created_ms="$(monotonic_ms)"
	for _ in $(seq 1 "$max_handle_attempts"); do
		handle="$(random_hex 32)"
		[[ "$handle" =~ ^[0-9a-f]{64}$ ]] || continue
		digest="$(sha256_text "$handle")"
		slot="$resume_index/$digest.json"
		[[ ! -e "$slot" ]] && break
		handle=""
	done
	[[ -n "$handle" ]] || die "resume_handle_ambiguous"
	state_json="$(create_state_json "$exact_head" "$digest" "$created_ms")" || die "boot_session_observation_unavailable"
	if [[ -n "${PHASE28_TEST_INITIAL_ATTEMPT_STATE:-}" ]]; then
		[[ "${PHASE28_TEST_MODE:-0}" == "1" ]] || die "validator_error"
		state_json="$(jq -c --arg state "$PHASE28_TEST_INITIAL_ATTEMPT_STATE" '.attempt_state = $state | .checkpoint_id=null | .checkpoint_token=null | .expected_response_token=null | .expected_user_action=null | .monotonic_deadline_ms=null' <<<"$state_json")"
	fi
	attempt_id="$(jq -er '.attempt_id' <<<"$state_json")"
	boot_digest="$(jq -er '.boot_session_sha256' <<<"$state_json")"
	attempt_dir="$attempts_root/$attempt_id"
	mkdir -m 700 "$attempt_dir"
	atomic_write "$attempt_dir/state.json" "$state_json"
	atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg dir "$attempt_dir" --arg attempt "$attempt_id" --arg boot "$boot_digest" --argjson generation "$generation" --argjson checkpoint_generation "$(jq -er '.checkpoint_generation' <<<"$state_json")" '{schema_version:"exact-head-resume-active-v1",status:"active",resume_handle_sha256:$digest,attempt_dir:$dir,attempt_id:$attempt,attempt_generation:$generation,checkpoint_generation:$checkpoint_generation,boot_session_sha256:$boot}')"
	release_lock
	trap - RETURN
	if [[ -n "${PHASE28_TEST_INITIAL_ATTEMPT_STATE:-}" ]]; then
		printf 'resume_handle=%s\n' "$handle"
		return
	fi
	public_checkpoint_output "$attempt_dir/state.json" "$handle"
}

resolve_checkpoint() {
	local handle=""
	while (($#)); do
		case "$1" in
		--resume-handle)
			handle="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	ensure_private_root
	acquire_lock
	local slot
	local state_path
	slot="$(resolve_handle "$handle")"
	state_path="$(jq -er '.attempt_dir' "$slot")/state.json"
	validate_state "$state_path" || die "state_malformed"
	validate_slot_state "$slot" "$state_path"
	assert_fresh_identity "$state_path"
	if [[ "$(jq -r '.attempt_state' "$state_path")" == "terminal" ]]; then
		local retained_terminal
		retained_terminal="$(jq -er '.terminal_outcome' "$state_path")"
		terminal_cleanup "$slot" "$retained_terminal" crash
		release_lock
		die "resume_handle_stale"
	fi
	case "$(jq -er '.attempt_state' "$state_path")" in
	post_capture_validated | classified)
		finalize_successful_classification "$slot" "$state_path"
		release_lock
		return
		;;
	esac
	public_checkpoint_output "$state_path" "$handle"
	release_lock
}

deliver_token() {
	local handle=""
	local checkpoint_token=""
	local response_token=""
	while (($#)); do
		case "$1" in
		--resume-handle)
			handle="${2:-}"
			shift 2
			;;
		--checkpoint-token)
			checkpoint_token="${2:-}"
			shift 2
			;;
		--response-token)
			response_token="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	ensure_private_root
	acquire_lock
	local slot
	local attempt_dir
	local state_path
	local deadline
	local now
	slot="$(resolve_handle "$handle")"
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	state_path="$attempt_dir/state.json"
	validate_state "$state_path" || die "state_malformed"
	validate_slot_state "$slot" "$state_path"
	assert_fresh_identity "$state_path"
	[[ "$(jq -r '.checkpoint_id // empty' "$state_path")" != "" ]] || die "checkpoint_state_mismatch"
	[[ "$(jq -r '.checkpoint_token // empty' "$state_path")" == "$checkpoint_token" ]] || die "checkpoint_token_mismatch"
	[[ "$(jq -r '.expected_response_token // empty' "$state_path")" == "$response_token" ]] || die "checkpoint_token_mismatch"
	deadline="$(jq -er '.monotonic_deadline_ms' "$state_path")"
	now="$(monotonic_ms)"
	((now < deadline)) || die "checkpoint_expired"
	assert_fresh_identity "$state_path"
	local checkpoint_generation
	local lease
	local nonce
	local owner_pid
	local owner_fingerprint
	checkpoint_generation="$(jq -er '.checkpoint_generation' "$state_path")"
	lease="$(jq -r '.lifecycle_lease_id // empty' "$state_path")"
	nonce="$(jq -r '.effect_authorization_nonce // empty' "$state_path")"
	owner_pid="$(jq -r '.lifecycle_owner_pid // empty' "$state_path")"
	owner_fingerprint="$(jq -r '.lifecycle_owner_start_fingerprint_sha256 // empty' "$state_path")"
	local lifecycle_delivery=false
	if [[ "$checkpoint_token" == "plan13-armed-removal-v1" || "$checkpoint_token" == "plan13-barrel-usb-restore-v1" ]]; then
		lifecycle_delivery=true
		[[ "$(jq -r '.process_running' "$state_path")" == "true" ]] || die "lease_dead_or_reused_process"
		if [[ ! "$owner_pid" =~ ^[1-9][0-9]*$ ]] || ! kill -0 "$owner_pid" 2>/dev/null; then
			die "lease_dead_or_reused_process"
		fi
		[[ "$(process_start_fingerprint "$owner_pid")" == "$owner_fingerprint" ]] || die "lease_dead_or_reused_process"
		[[ -S "$attempt_dir/lifecycle.sock" ]] || die "private_capability_invalid"
		[[ "$(mode_of_path "$attempt_dir/lifecycle.sock")" == "600" ]] || die "private_capability_invalid"
	fi
	now="$(monotonic_ms)"
	((now < deadline)) || die "checkpoint_expired"
	local consumed_json
	consumed_json="$(state_operation consume-checkpoint "$state_path" "$(jq -cn --arg token "$checkpoint_token" --arg response "$response_token" --argjson now "$now" '{checkpointToken:$token,responseToken:$response,nowMonotonicMs:$now}')")" || die "checkpoint_state_mismatch"
	persist_state_and_slot "$state_path" "$slot" "$consumed_json"
	if [[ "$lifecycle_delivery" == "true" ]]; then
		local frame
		frame="$(jq -cn --arg digest "$(sha256_text "$handle")" --argjson generation "$checkpoint_generation" --arg token "$checkpoint_token" --arg response "$response_token" --arg nonce "$nonce" --arg lease "$lease" '{resume_handle_sha256:$digest,checkpoint_generation:$generation,checkpoint_token:$token,response_token:$response,effect_authorization_nonce:$nonce,lifecycle_lease_id:$lease}')"
		set +e
		if [[ "${PHASE28_TEST_MODE:-0}" == "1" ]]; then
			local sender="${PHASE28_TEST_SOCKET_SEND_BIN:-}"
			[[ -n "$sender" && -x "$sender" ]] || die "private_capability_invalid"
			"$sender" "$attempt_dir/lifecycle.sock" "$frame"
		else
			(cd "$attempt_dir" && perl -MIO::Socket::UNIX -MSocket -e 'my ($frame)=@ARGV; my $socket=IO::Socket::UNIX->new(Type=>SOCK_STREAM,Peer=>"lifecycle.sock") or exit 1; print {$socket} length($frame)."\n".$frame or exit 1; close $socket or exit 1;' "$frame")
		fi
		local sender_status=$?
		set -e
		if ((sender_status != 0)); then
			local terminal_json
			terminal_json="$(state_operation terminalize "$state_path" '{"blockerReason":"private_capability_invalid","cleanupUnresolved":false}')"
			persist_state_and_slot "$state_path" "$slot" "$terminal_json"
			terminal_cleanup "$slot" "$(jq -er '.terminal_outcome' "$state_path")" blocked
			release_lock
			die "private_capability_invalid"
		fi
	fi
	release_lock
	printf 'checkpoint_delivery=accepted\n'
	printf 'checkpoint_generation=%s\n' "$checkpoint_generation"
}

terminal_cleanup() {
	local slot="$1"
	local terminal="$2"
	local category="$3"
	local attempt_dir
	local generation
	local digest
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	generation="$(jq -er '.attempt_generation' "$slot")"
	digest="$(jq -er '.resume_handle_sha256' "$slot")"
	if [[ -f "$attempt_dir/child.pid" ]]; then
		local child_pid
		child_pid="$(sed -n '1p' "$attempt_dir/child.pid")"
		terminate_effect_process "$child_pid" "phase28 attempt cleanup" || die "process_cleanup_unresolved"
	fi
	rm -f "$attempt_dir/child.pid" "$attempt_dir/lifecycle.sock" "$attempt_dir/credential-capability.json" "$attempt_dir/effect.ack" "$attempt_dir/effect.gate" "$attempt_dir/effect-result.json"
	[[ "${PHASE28_CRASH_AT:-}" != "before_tombstone_persistence" ]] || exit 97
	atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg terminal "$terminal" --arg category "$category" --argjson generation "$generation" '{schema_version:"exact-head-resume-tombstone-v1",resume_handle_sha256:$digest,attempt_generation:$generation,terminal_status:"closed",terminal_category:$terminal,cleanup_time_category:$category}')"
	[[ "${PHASE28_CRASH_AT:-}" != "after_tombstone_persistence" ]] || exit 97
	rm -rf "$attempt_dir"
	[[ "${PHASE28_CRASH_AT:-}" != "after_terminal_cleanup" ]] || exit 97
}

finalize_successful_classification() {
	local slot="$1"
	local state_path="$2"
	local phase
	phase="$(jq -er '.classification_phase' "$state_path")"
	if [[ "$(jq -er '.attempt_state' "$state_path")" == "post_capture_validated" && "$phase" == "invoked" ]]; then
		local interrupted_json
		interrupted_json="$(state_operation terminalize "$state_path" '{"blockerReason":"classifier_input_invalid","cleanupUnresolved":false}')"
		persist_state_and_slot "$state_path" "$slot" "$interrupted_json"
	elif [[ "$(jq -er '.attempt_state' "$state_path")" == "post_capture_validated" ]]; then
		[[ "$phase" == "not_run" ]] || die "classification_inconsistent"
		persist_state_and_slot "$state_path" "$slot" "$(state_operation mark-classifier-invoked "$state_path")"
		[[ "${PHASE28_CRASH_AT:-}" != "after_classifier_intent_persistence" ]] || exit 97
		local classified_json
		if [[ -n "${PHASE28_CLASSIFIER_TRACE:-}" ]]; then
			[[ "${PHASE28_TEST_MODE:-0}" == "1" ]] || die "validator_error"
			printf 'strict-production-v2\n' >>"$PHASE28_CLASSIFIER_TRACE"
		fi
		classified_json="$(state_operation classify "$state_path")"
		[[ "${PHASE28_CRASH_AT:-}" != "after_classifier_invocation" ]] || exit 97
		persist_state_and_slot "$state_path" "$slot" "$classified_json"
		[[ "${PHASE28_CRASH_AT:-}" != "after_classified_persistence" ]] || exit 97
	fi

	if [[ "$(jq -er '.attempt_state' "$state_path")" == "classified" ]]; then
		persist_state_and_slot "$state_path" "$slot" "$(state_operation finalize-classified "$state_path")"
		[[ "${PHASE28_CRASH_AT:-}" != "after_terminal_persistence" ]] || exit 97
	fi
	[[ "$(jq -er '.attempt_state' "$state_path")" == "terminal" ]] || die "classification_inconsistent"
	local terminal
	local closed_output
	terminal="$(jq -er '.terminal_outcome' "$state_path")"
	closed_output="$(public_terminal_output "$state_path")"
	terminal_cleanup "$slot" "$terminal" "$([[ "$terminal" == blocked_safe_* ]] && printf blocked || printf normal)"
	printf '%s\n' "$closed_output"
}

lifecycle_owner_transition() {
	local capability=""
	local event=""
	local values_file=""
	while (($#)); do
		case "$1" in
		--capability)
			capability="${2:-}"
			shift 2
			;;
		--event)
			event="${2:-}"
			shift 2
			;;
		--values-file)
			values_file="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	[[ "$capability" =~ ^[0-9a-f]{64}$ ]] || die "private_capability_invalid"
	case "$event" in
	absence-observing | restore-waiting | reappearance-observing | capture-running | capture-complete) ;;
	*) die "checkpoint_state_mismatch" ;;
	esac
	local attempt_dir="${PHASE28_LIFECYCLE_ATTEMPT_DIR:-}"
	[[ "$attempt_dir" == "$attempts_root/"* && -d "$attempt_dir" ]] || die "private_capability_invalid"
	[[ -n "$values_file" && "$values_file" == "$attempt_dir/"* && -f "$values_file" ]] || die "private_capability_invalid"
	[[ "$(mode_of_path "$values_file")" == "600" ]] || die "private_capability_invalid"
	ensure_private_root
	acquire_lock
	local state_path="$attempt_dir/state.json"
	local digest
	local slot
	digest="$(jq -er '.resume_handle_sha256' "$state_path")"
	slot="$resume_index/$digest.json"
	validate_active_slot "$slot" "$digest" || die "resume_handle_ambiguous"
	validate_state "$state_path" || die "state_malformed"
	validate_slot_state "$slot" "$state_path"
	assert_fresh_identity "$state_path"
	[[ "$(sha256_text "$capability")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] || die "lease_owner_mismatch"
	[[ "$PPID" == "$(jq -er '.lifecycle_owner_pid' "$state_path")" ]] || die "lease_owner_mismatch"
	[[ "$(process_start_fingerprint "$PPID")" == "$(jq -er '.lifecycle_owner_start_fingerprint_sha256' "$state_path")" ]] || die "lease_dead_or_reused_process"
	local now
	now="$(monotonic_ms)"
	((now < $(jq -er '.lifecycle_deadline_ms' "$state_path"))) || die "checkpoint_expired"
	local args
	args="$(jq -cn --arg event "$event" --slurpfile values "$values_file" '{event:$event,values:$values[0]}')"
	local next_json
	next_json="$(state_operation lifecycle-event "$state_path" "$args")" || die "checkpoint_state_mismatch"
	persist_state_and_slot "$state_path" "$slot" "$next_json"
	release_lock
	printf 'lifecycle_transition=%s\n' "$event"
}

run_validated_effect() {
	local handle=""
	local effect=""
	while (($#)); do
		case "$1" in
		--resume-handle)
			handle="${2:-}"
			shift 2
			;;
		--effect-id)
			effect="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	case "$effect" in
	detector_board_info | credential_presence_bind | reference_guard | package | flash_reinit_runtime | lifecycle_start | post_capture_detector_board_info) ;;
	*) die "validator_error" ;;
	esac
	if [[ -n "${PHASE28_CLASSIFIER_TRACE:-}" && "${PHASE28_TEST_MODE:-0}" != "1" ]]; then
		die "validator_error"
	fi
	ensure_private_root
	acquire_lock
	local slot
	local attempt_dir
	local state_path
	local phase
	slot="$(resolve_handle "$handle")"
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	state_path="$attempt_dir/state.json"
	validate_state "$state_path" || die "state_malformed"
	validate_slot_state "$slot" "$state_path"
	assert_fresh_identity "$state_path"
	if [[ "$(jq -r '.attempt_state' "$state_path")" == "terminal" ]]; then
		local retained_terminal
		retained_terminal="$(jq -er '.terminal_outcome' "$state_path")"
		terminal_cleanup "$slot" "$retained_terminal" crash
		release_lock
		die "resume_handle_stale"
	fi
	case "$(jq -er '.attempt_state' "$state_path")" in
	post_capture_validated | classified)
		finalize_successful_classification "$slot" "$state_path"
		release_lock
		return
		;;
	esac
	if [[ -n "${PHASE28_INJECT_INVALID_CATEGORY:-}" ]]; then
		case "$PHASE28_INJECT_INVALID_CATEGORY" in
		expired | token_mismatch | exact_head_mismatch | manifest_mismatch | reference_mismatch | boot_session_mismatch | dirty_head | malformed_state | validator_error | lock_failure | persistence_failure | lease_conflict) die "$PHASE28_INJECT_INVALID_CATEGORY" ;;
		*) die "validator_error" ;;
		esac
	fi
	phase="$(jq -r '.effect_phase' "$state_path")"
	if [[ "$phase" == "authorized" || "$phase" == "invoked" ]]; then
		local ambiguous_json
		ambiguous_json="$(state_operation terminalize "$state_path" '{"blockerReason":"effect_in_flight_ambiguous","cleanupUnresolved":false}')"
		persist_state_and_slot "$state_path" "$slot" "$ambiguous_json"
		terminal_cleanup "$slot" "$(jq -er '.terminal_outcome' "$state_path")" "crash"
		release_lock
		die "effect_in_flight_ambiguous"
	fi
	if [[ "$phase" == "completed" || "$phase" == "failed" ]]; then
		local retained_result="$attempt_dir/effect-result.json"
		[[ -f "$retained_result" ]] || {
			local inconsistent_json
			inconsistent_json="$(state_operation terminalize "$state_path" '{"blockerReason":"effect_in_flight_ambiguous","cleanupUnresolved":false}')"
			persist_state_and_slot "$state_path" "$slot" "$inconsistent_json"
			terminal_cleanup "$slot" "$(jq -er '.terminal_outcome' "$state_path")" crash
			release_lock
			die "effect_in_flight_ambiguous"
		}
		local recovered_json
		set +e
		recovered_json="$(state_operation apply-effect-result "$state_path" "$(jq -cn --arg resultPath "$retained_result" --argjson completedMonotonicMs "$(monotonic_ms)" '{resultPath:$resultPath,completedMonotonicMs:$completedMonotonicMs}')")"
		local recovery_status=$?
		set -e
		if ((recovery_status != 0)); then
			local inconsistent_json
			inconsistent_json="$(state_operation terminalize "$state_path" '{"blockerReason":"effect_in_flight_ambiguous","cleanupUnresolved":false}')"
			persist_state_and_slot "$state_path" "$slot" "$inconsistent_json"
			terminal_cleanup "$slot" "$(jq -er '.terminal_outcome' "$state_path")" crash
			release_lock
			die "effect_in_flight_ambiguous"
		fi
		persist_state_and_slot "$state_path" "$slot" "$recovered_json"
		rm -f "$retained_result"
		if [[ "$(jq -r '.attempt_state' "$state_path")" == "post_capture_validated" ]]; then
			finalize_successful_classification "$slot" "$state_path"
			release_lock
			return
		fi
		if [[ "$(jq -r '.attempt_state' "$state_path")" == "terminal" ]]; then
			local terminal
			terminal="$(jq -er '.terminal_outcome' "$state_path")"
			terminal_cleanup "$slot" "$terminal" crash
			release_lock
			printf 'terminal_outcome=%s\n' "$terminal"
			return
		fi
		release_lock
		printf 'effect_recovery=completed_without_redispatch\n'
		if [[ "$(jq -r '.checkpoint_id // empty' "$state_path")" != "" ]]; then
			public_checkpoint_output "$state_path" "$handle"
		fi
		return
	fi
	[[ "$(jq -r '.checkpoint_id // empty' "$state_path")" == "" ]] || die "checkpoint_state_mismatch"
	[[ "${PHASE28_CRASH_AT:-}" != "before_authorized_persistence" ]] || exit 97
	local nonce
	local sequence
	nonce="$(random_hex 16)"
	atomic_write "$state_path" "$(transform_effect_state authorize "$state_path" "$effect" "$nonce")"
	sequence="$(jq -er '.effect_sequence' "$state_path")"
	[[ "${PHASE28_CRASH_AT:-}" != "after_authorized_persistence" ]] || exit 97
	adapter_command "$effect"
	local ack="$attempt_dir/effect.ack"
	local gate="$attempt_dir/effect.gate"
	local result_path="$attempt_dir/effect-result.json"
	rm -f "$ack" "$gate" "$result_path"
	local lifecycle_lease=""
	local lifecycle_capability=""
	if [[ "$effect" == "lifecycle_start" ]]; then
		lifecycle_lease="$(random_hex 16)"
		lifecycle_capability="$(random_hex 32)"
	fi
	local ready="$attempt_dir/effect-process.ready"
	local -a adapter_env=(env
		PHASE28_EFFECT_ACK_FILE="$ack"
		PHASE28_EFFECT_GATE_FILE="$gate"
		PHASE28_EFFECT_RESULT_FILE="$result_path"
		PHASE28_EFFECT_ID="$effect"
		PHASE28_LIFECYCLE_LEASE_ID="$lifecycle_lease"
		PHASE28_LIFECYCLE_CAPABILITY="$lifecycle_capability"
		PHASE28_LIFECYCLE_ATTEMPT_DIR="$attempt_dir"
		PHASE28_LIFECYCLE_RUNNER="$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt.sh"
		"${effect_adapter_command[@]}")
	phase_process_group_start "$ready" "${adapter_env[@]}" || die "validator_error"
	local child_pid="$PHASE_PROCESS_GROUP_PID"
	active_effect_pid="$child_pid"
	printf '%s\n' "$child_pid" >"$attempt_dir/child.pid"
	[[ "${PHASE28_CRASH_AT:-}" != "after_child_creation" ]] || exit 97
	for _ in $(seq 1 100); do
		[[ -f "$ack" ]] && break
		kill -0 "$child_pid" 2>/dev/null || die "validator_error"
		sleep 0.01
	done
	[[ -f "$ack" ]] || die "validator_error"
	[[ "${PHASE28_CRASH_AT:-}" != "after_start_acknowledgement" ]] || exit 97
	assert_fresh_identity "$state_path"
	persist_state_and_slot "$state_path" "$slot" "$(transform_effect_state invoke "$state_path" "$effect" "$nonce" "$sequence")"
	if [[ "$effect" == "lifecycle_start" ]]; then
		local owner_fingerprint
		owner_fingerprint="$(process_start_fingerprint "$child_pid")"
		local attach_now
		attach_now="$(monotonic_ms)"
		local attach_args
		attach_args="$(jq -cn --arg leaseId "$lifecycle_lease" --arg capabilitySha256 "$(sha256_text "$lifecycle_capability")" --argjson ownerPid "$child_pid" --arg ownerStartFingerprintSha256 "$owner_fingerprint" --argjson lifecycleDeadlineMs "$((attach_now + 1200000))" --argjson checkpointCreatedMonotonicMs "$attach_now" '{leaseId:$leaseId,capabilitySha256:$capabilitySha256,ownerPid:$ownerPid,ownerStartFingerprintSha256:$ownerStartFingerprintSha256,lifecycleDeadlineMs:$lifecycleDeadlineMs,checkpointCreatedMonotonicMs:$checkpointCreatedMonotonicMs}')"
		persist_state_and_slot "$state_path" "$slot" "$(state_operation attach-lifecycle "$state_path" "$attach_args")"
	fi
	[[ "${PHASE28_CRASH_AT:-}" != "after_invoked_persistence" ]] || exit 97
	release_lock
	: >"$gate"
	[[ "${PHASE28_CRASH_AT:-}" != "after_start_gate_release" ]] || exit 97
	set +e
	if [[ "$effect" == "lifecycle_start" ]]; then
		public_checkpoint_output "$state_path" "$handle"
		local emitted_generation
		emitted_generation="$(jq -er '.checkpoint_generation' "$state_path")"
		while kill -0 "$child_pid" 2>/dev/null; do
			if [[ -f "$state_path" ]]; then
				local maybe_generation
				local maybe_checkpoint
				maybe_generation="$(jq -r '.checkpoint_generation' "$state_path" 2>/dev/null)"
				maybe_checkpoint="$(jq -r '.checkpoint_id // empty' "$state_path" 2>/dev/null)"
				if [[ "$maybe_checkpoint" == "plan13-lifecycle-restore" && "$maybe_generation" != "$emitted_generation" ]]; then
					public_checkpoint_output "$state_path" "$handle"
					emitted_generation="$maybe_generation"
				fi
			fi
			sleep 0.05
		done
	fi
	wait "$child_pid"
	local adapter_status=$?
	set -e
	active_effect_pid=""
	[[ "${PHASE28_CRASH_AT:-}" != "after_adapter_return" ]] || exit 97
	if [[ ! -f "$result_path" ]]; then
		write_failed_effect_result "$result_path" "$effect" "$(effect_failure_blocker "$effect")"
		adapter_status=1
	else
		chmod 600 "$result_path"
		sync_path "$result_path"
	fi
	acquire_lock
	assert_fresh_identity "$state_path"
	local succeeded=false
	((adapter_status == 0)) && succeeded=true
	persist_state_and_slot "$state_path" "$slot" "$(transform_effect_state finish "$state_path" "$effect" "$nonce" "$sequence" "$succeeded")"
	rm -f "$attempt_dir/child.pid" "$ack" "$gate"
	[[ "${PHASE28_CRASH_AT:-}" != "after_completed_persistence" ]] || exit 97
	local completed_json
	set +e
	completed_json="$(state_operation apply-effect-result "$state_path" "$(jq -cn --arg resultPath "$result_path" --argjson completedMonotonicMs "$(monotonic_ms)" '{resultPath:$resultPath,completedMonotonicMs:$completedMonotonicMs}')")"
	local completion_status=$?
	set -e
	if ((completion_status != 0)); then
		local inconsistent_json
		inconsistent_json="$(state_operation terminalize "$state_path" '{"blockerReason":"effect_in_flight_ambiguous","cleanupUnresolved":false}')"
		persist_state_and_slot "$state_path" "$slot" "$inconsistent_json"
		terminal_cleanup "$slot" "$(jq -er '.terminal_outcome' "$state_path")" crash
		release_lock
		die "effect_in_flight_ambiguous"
	fi
	persist_state_and_slot "$state_path" "$slot" "$completed_json"
	rm -f "$result_path"
	[[ "${PHASE28_CRASH_AT:-}" != "after_effect_transition_persistence" ]] || exit 97
	local final_state
	final_state="$(jq -er '.attempt_state' "$state_path")"
	if [[ "$final_state" == "post_capture_validated" ]]; then
		printf 'effect_id=%s\n' "$effect"
		printf 'effect_sequence=%s\n' "$sequence"
		printf 'effect_status=%s\n' "$([[ "$succeeded" == true ]] && printf completed || printf failed)"
		finalize_successful_classification "$slot" "$state_path"
		release_lock
		return
	fi
	if [[ "$final_state" == "terminal" ]]; then
		local terminal
		terminal="$(jq -er '.terminal_outcome' "$state_path")"
		terminal_cleanup "$slot" "$terminal" "$([[ "$terminal" == blocked_safe_* ]] && printf blocked || printf normal)"
		release_lock
		printf 'effect_id=%s\n' "$effect"
		printf 'effect_sequence=%s\n' "$sequence"
		printf 'effect_status=%s\n' "$([[ "$succeeded" == true ]] && printf completed || printf failed)"
		printf 'terminal_outcome=%s\n' "$terminal"
		return
	fi
	release_lock
	printf 'effect_id=%s\n' "$effect"
	printf 'effect_sequence=%s\n' "$sequence"
	printf 'effect_status=%s\n' "$([[ "$succeeded" == true ]] && printf completed || printf failed)"
	if [[ "$(jq -r '.checkpoint_id // empty' "$state_path")" != "" ]]; then
		public_checkpoint_output "$state_path" "$handle"
	fi
}

command="${1:-}"
[[ -n "$command" ]] || {
	usage
	exit 2
}
shift
cd "$repo_root"
case "$command" in
begin-attempt) begin_attempt "$@" ;;
resolve-checkpoint) resolve_checkpoint "$@" ;;
deliver-token) deliver_token "$@" ;;
run-validated-effect) run_validated_effect "$@" ;;
lifecycle-owner-transition) lifecycle_owner_transition "$@" ;;
*) die "unknown_command" ;;
esac
