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
readonly max_handle_attempts=8
lock_held=false
active_effect_pid=""

cleanup_on_exit() {
	if [[ -n "$active_effect_pid" ]] && kill -0 "$active_effect_pid" 2>/dev/null; then
		kill -TERM "$active_effect_pid" 2>/dev/null || true
		wait "$active_effect_pid" 2>/dev/null || true
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
	node --input-type=module -e 'const { createAttemptState } = await import(process.argv[1]); const state=createAttemptState({exactHead:process.argv[2],resumeHandleSha256:process.argv[3],createdMonotonicMs:Number(process.argv[4])}); process.stdout.write(JSON.stringify(state))' "$state_module" "$exact_head" "$resume_digest" "$created_ms"
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
	while (($#)); do
		case "$1" in
		--attempt-generation)
			generation="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	[[ "$generation" =~ ^[0-9]+$ ]] || die "state_malformed"
	ensure_private_root
	require_clean_head
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
	exact_head="$(current_head)"
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
		state_json="$(jq -c --arg state "$PHASE28_TEST_INITIAL_ATTEMPT_STATE" '.attempt_state = $state' <<<"$state_json")"
	fi
	attempt_id="$(jq -er '.attempt_id' <<<"$state_json")"
	boot_digest="$(jq -er '.boot_session_sha256' <<<"$state_json")"
	attempt_dir="$attempts_root/$attempt_id"
	mkdir -m 700 "$attempt_dir"
	atomic_write "$attempt_dir/state.json" "$state_json"
	atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg dir "$attempt_dir" --arg attempt "$attempt_id" --arg boot "$boot_digest" --argjson generation "$generation" '{schema_version:"exact-head-resume-active-v1",status:"active",resume_handle_sha256:$digest,attempt_dir:$dir,attempt_id:$attempt,attempt_generation:$generation,checkpoint_generation:0,boot_session_sha256:$boot}')"
	release_lock
	trap - RETURN
	printf 'resume_handle=%s\n' "$handle"
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
	assert_fresh_identity "$state_path"
	printf 'checkpoint_id=%s\n' "$(jq -r '.checkpoint_id // "none"' "$state_path")"
	printf 'checkpoint_generation=%s\n' "$(jq -r '.checkpoint_generation' "$state_path")"
	printf 'checkpoint_token=%s\n' "$(jq -r '.checkpoint_token // "none"' "$state_path")"
	printf 'expected_response_token=%s\n' "$(jq -r '.expected_response_token // "none"' "$state_path")"
	printf 'expected_user_action=%s\n' "$(jq -r '.expected_user_action // "none"' "$state_path")"
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
	assert_fresh_identity "$state_path"
	[[ "$(jq -r '.checkpoint_token // empty' "$state_path")" == "$checkpoint_token" ]] || die "checkpoint_token_mismatch"
	[[ "$(jq -r '.expected_response_token // empty' "$state_path")" == "$response_token" ]] || die "checkpoint_token_mismatch"
	deadline="$(jq -er '.monotonic_deadline_ms' "$state_path")"
	now="$(monotonic_ms)"
	((now < deadline)) || die "checkpoint_expired"
	assert_fresh_identity "$state_path"
	if [[ "$checkpoint_token" == "plan13-armed-removal-v1" || "$checkpoint_token" == "plan13-barrel-usb-restore-v1" ]]; then
		local sender="${PHASE28_SOCKET_SEND_BIN:-}"
		[[ -n "$sender" && -x "$sender" ]] || die "private_capability_invalid"
		local lease
		local nonce
		lease="$(jq -er '.lifecycle_lease_id' "$state_path")"
		nonce="$(jq -er '.effect_authorization_nonce' "$state_path")"
		"$sender" "$attempt_dir/lifecycle.sock" "$(sha256_text "$handle")" "$(jq -r '.checkpoint_generation' "$state_path")" "$checkpoint_token" "$response_token" "$nonce" "$lease"
	fi
	release_lock
	printf 'checkpoint_delivery=accepted\n'
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
		phase_process_group_terminate "$child_pid" "phase28 attempt cleanup" || die "process_cleanup_unresolved"
	fi
	rm -rf "$attempt_dir"
	atomic_write "$slot" "$(jq -cn --arg digest "$digest" --arg terminal "$terminal" --arg category "$category" --argjson generation "$generation" '{schema_version:"exact-head-resume-tombstone-v1",resume_handle_sha256:$digest,attempt_generation:$generation,terminal_status:"closed",terminal_category:$terminal,cleanup_time_category:$category}')"
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
	assert_fresh_identity "$state_path"
	if [[ -n "${PHASE28_INJECT_INVALID_CATEGORY:-}" ]]; then
		case "$PHASE28_INJECT_INVALID_CATEGORY" in
		expired | token_mismatch | exact_head_mismatch | manifest_mismatch | reference_mismatch | boot_session_mismatch | dirty_head | malformed_state | validator_error | lock_failure | persistence_failure | lease_conflict) die "$PHASE28_INJECT_INVALID_CATEGORY" ;;
		*) die "validator_error" ;;
		esac
	fi
	phase="$(jq -r '.effect_phase' "$state_path")"
	if [[ "$phase" == "authorized" || "$phase" == "invoked" ]]; then
		terminal_cleanup "$slot" "blocked_safe_attempt_prerequisite" "crash"
		release_lock
		die "effect_in_flight_ambiguous"
	fi
	if [[ "$phase" == "completed" || "$phase" == "failed" ]]; then
		atomic_write "$state_path" "$(transform_effect_state normalize "$state_path")"
	fi
	[[ "${PHASE28_CRASH_AT:-}" != "before_authorized_persistence" ]] || exit 97
	local nonce
	local sequence
	nonce="$(random_hex 16)"
	atomic_write "$state_path" "$(transform_effect_state authorize "$state_path" "$effect" "$nonce")"
	sequence="$(jq -er '.effect_sequence' "$state_path")"
	[[ "${PHASE28_CRASH_AT:-}" != "after_authorized_persistence" ]] || exit 97
	local adapter="${PHASE28_EFFECT_ADAPTER_DIR:-}/$effect"
	[[ -x "$adapter" ]] || die "validator_error"
	local ack="$attempt_dir/effect.ack"
	local gate="$attempt_dir/effect.gate"
	rm -f "$ack" "$gate"
	PHASE28_EFFECT_ACK_FILE="$ack" PHASE28_EFFECT_GATE_FILE="$gate" "$adapter" "$attempt_dir" &
	local child_pid=$!
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
	atomic_write "$state_path" "$(transform_effect_state invoke "$state_path" "$effect" "$nonce" "$sequence")"
	[[ "${PHASE28_CRASH_AT:-}" != "after_invoked_persistence" ]] || exit 97
	release_lock
	: >"$gate"
	[[ "${PHASE28_CRASH_AT:-}" != "after_start_gate_release" ]] || exit 97
	set +e
	wait "$child_pid"
	local adapter_status=$?
	set -e
	active_effect_pid=""
	[[ "${PHASE28_CRASH_AT:-}" != "after_adapter_return" ]] || exit 97
	acquire_lock
	assert_fresh_identity "$state_path"
	local succeeded=false
	((adapter_status == 0)) && succeeded=true
	atomic_write "$state_path" "$(transform_effect_state finish "$state_path" "$effect" "$nonce" "$sequence" "$succeeded")"
	rm -f "$attempt_dir/child.pid" "$ack" "$gate"
	[[ "${PHASE28_CRASH_AT:-}" != "after_completed_persistence" ]] || exit 97
	release_lock
	((adapter_status == 0)) || die "${effect}_failed"
	printf 'effect_id=%s\n' "$effect"
	printf 'effect_sequence=%s\n' "$sequence"
	printf 'effect_status=completed\n'
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
*) die "unknown_command" ;;
esac
