#!/usr/bin/env bash
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Crash-closed private control plane for one exact-HEAD Phase 28.1.1 attempt.
set -euo pipefail
umask 077

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=/dev/null
source "$repo_root/scripts/process-group.sh"
# shellcheck source=/dev/null
source "$repo_root/scripts/serial-session-trace.sh"
readonly private_root="${PHASE28_ATTEMPT_CONTROL_ROOT:-$repo_root/hardware-runs/phase28.1.1/attempt-control}"
readonly resume_index="$private_root/resume-index"
readonly attempts_root="$private_root/attempts"
readonly lock_dir="$private_root/control.lock"
readonly state_module="$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs"
readonly classifier_module="$repo_root/scripts/phase28.1.1-strict-production-evidence.mjs"
readonly production_adapter="$repo_root/scripts/phase28.1.1-accepted-state-diagnostic.sh"
readonly lifecycle_frame_helper="$repo_root/scripts/phase28.1.1-lifecycle-frame.pl"
readonly transport_qualification_helper="$repo_root/scripts/ultra205-transport-qualification.sh"
readonly phase_lifecycle_id="28.1.1-2026-07-09T19-24-27"
readonly max_handle_attempts=8
lock_held=false
active_effect_pid=""
effect_adapter_command=()

terminate_effect_process() {
	local pid="$1"
	local label="$2"
	local leader_alive=false
	local group_alive=false
	phase_process_is_alive "$pid" && leader_alive=true
	phase_process_group_is_alive "$pid" && group_alive=true
	if [[ "$leader_alive" == "false" && "$group_alive" == "false" ]]; then
		return 0
	fi
	if [[ "$leader_alive" == "true" ]]; then
		local pgid
		pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')"
		[[ "$pgid" == "$pid" ]] || return 1
	fi
	phase_process_group_terminate "$pid" "$label"
}

cleanup_on_exit() {
	if [[ -n "$active_effect_pid" ]]; then
		terminate_effect_process "$active_effect_pid" "phase28 effect cleanup" >/dev/null 2>&1 || true
	fi
	if [[ "$lock_held" == "true" ]]; then
		rm -f "$lock_dir/owner.pid"
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
	printf 'usage: %s begin-attempt --hardware-exact-head HEAD --transport-qualification PATH | resolve-checkpoint|deliver-token|run-validated-effect|verify-terminal|cleanup-active-attempt|cleanup-unique-orphan-attempt [options]\n' "$(basename "$0")"
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

owner_of_path() {
	if stat -f '%u' "$1" >/dev/null 2>&1; then
		stat -f '%u' "$1"
		return
	fi
	stat -c '%u' "$1"
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

lifecycle_lease_duration_ms() {
	# shellcheck disable=SC2016 # The template literal is evaluated by Node, not Bash.
	node --input-type=module -e 'const { PLAN13_LIFECYCLE_LEASE_DURATION_MS } = await import(process.argv[1]); process.stdout.write(`${PLAN13_LIFECYCLE_LEASE_DURATION_MS}\n`)' "$state_module"
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

active_expiry_blocker() {
	local state_path="$1"
	local now="$2"
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const maybeBlocker = authority.activeAttemptExpiryBlocker(state, Number(process.argv[3]));
if (maybeBlocker !== null) process.stdout.write(maybeBlocker);' "$state_module" "$state_path" "$now"
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

public_action_output() {
	local state_path="$1"
	# shellcheck disable=SC2016
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const action = authority.publicAction(state);
process.stdout.write("## ACTION READY\n");
for (const field of authority.PUBLIC_ACTION_FIELDS) process.stdout.write(`${field}=${action[field]}\n`);' "$state_module" "$state_path"
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
	if jq -e --arg digest "$digest" '
    type == "object" and
    (keys | sort) == (["attempt_generation","cleanup_time_category","resume_handle_sha256","schema_version","terminal_category","terminal_status"] | sort) and
    .schema_version == "exact-head-resume-tombstone-v1" and
    .resume_handle_sha256 == $digest and
    .terminal_status == "closed" and
    (.terminal_category | IN("blocked_safe_attempt_prerequisite","blocked_safe_unresolved_process","blocked_safe_evidence_invalid","gaps_found_same_chain_production_markers_absent","passed_same_chain_hardware")) and
    (.cleanup_time_category | IN("normal","blocked","crash","abandoned"))
  ' "$slot" >/dev/null; then
		return 0
	fi
	jq -e --arg digest "$digest" '
    type == "object" and
    (keys | sort) == (["attempt_generation","blocker_reason","cleanup_complete","cleanup_time_category","exact_head","formal_attempt_id_sha256","live_process_count","live_socket_count","phase30_promotion_input","phase_lifecycle_id","qualification_contract_digest_sha256","resume_handle_sha256","schema_version","serial_holder_count","terminal_category","terminal_status","verification_result"] | sort) and
    .schema_version == "exact-head-resume-tombstone-v2" and
    .resume_handle_sha256 == $digest and
    .terminal_status == "closed" and
    (.exact_head | test("^[0-9a-f]{40}$")) and
    (.formal_attempt_id_sha256 | test("^[0-9a-f]{64}$")) and
    .phase_lifecycle_id == "28.1.1-2026-07-09T19-24-27" and
    (.qualification_contract_digest_sha256 | test("^[0-9a-f]{64}$")) and
    (.terminal_category | IN("blocked_safe_attempt_prerequisite","blocked_safe_unresolved_process","blocked_safe_evidence_invalid","gaps_found_same_chain_production_markers_absent","passed_same_chain_hardware")) and
    (.verification_result | IN("gaps_found","passed")) and
    (.phase30_promotion_input | IN("pending","eligible")) and
    (.blocker_reason | type == "string") and
    (.cleanup_complete | type == "boolean") and
    (.live_process_count | type == "number" and floor == . and . >= 0) and
    (.serial_holder_count | type == "number" and floor == . and . >= 0) and
    (.live_socket_count | type == "number" and floor == . and . >= 0) and
    (.cleanup_time_category | IN("normal","blocked","crash","abandoned"))
  ' "$slot" >/dev/null
}

validate_formal_tombstone() {
	local slot="$1"
	local digest="$2"
	validate_tombstone "$slot" "$digest" || return 1
	[[ "$(jq -er '.schema_version' "$slot")" == "exact-head-resume-tombstone-v2" ]]
}

count_category() {
	case "$1" in
	0) printf 'zero\n' ;;
	1) printf 'one\n' ;;
	*) printf 'multiple\n' ;;
	esac
}

public_orphan_cleanup_failure() {
	local category="$1"
	local active_count_category="$2"
	local tombstone_count_category="$3"
	if [[ "$lock_held" == "true" ]]; then
		release_lock
	fi
	printf 'cleanup_result=not_closed\n'
	printf 'cleanup_category=%s\n' "$category"
	printf 'active_orphan_count_category=%s\n' "$active_count_category"
	printf 'tombstone_count_category=%s\n' "$tombstone_count_category"
	printf 'state_mutated=false\n'
	printf 'effect_sentinels_invoked=0\n'
	printf 'positive_evidence_promoted=false\n'
	die "$category"
}

classify_orphan_state() {
	local state_path="$1"
	local expected_head="$2"
	local expected_state="$3"
	local reason="$4"
	local observed_boot="$5"
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const category = authority.classifyLostResumeHandleOrphanState(state, {
  expectedHead: process.argv[3],
  expectedState: process.argv[4],
  reason: process.argv[5],
  observedBootSessionSha256: process.argv[6],
});
process.stdout.write(category);' "$state_module" "$state_path" "$expected_head" "$expected_state" "$reason" "$observed_boot"
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

qualification_parent_is_private() {
	local qualification_path="$1"
	local parent
	parent="$(dirname "$qualification_path")"
	[[ "$qualification_path" == /* && -d "$parent" && ! -L "$parent" ]] || return 1
	[[ "$(mode_of_path "$parent")" == "700" && "$(owner_of_path "$parent")" == "$(id -u)" ]]
}

formal_credential_paths_ready() {
	local wifi_path="$repo_root/wifi-credentials.json"
	local pool_path=""
	if [[ "${PHASE28_TEST_MODE:-0}" == "1" ]]; then
		wifi_path="${PHASE28_TEST_WIFI_CREDENTIAL_PATH:-$wifi_path}"
		pool_path="${PHASE28_TEST_POOL_CREDENTIAL_PATH:-}"
	elif [[ -n "${PHASE28_TEST_WIFI_CREDENTIAL_PATH:-}${PHASE28_TEST_POOL_CREDENTIAL_PATH:-}" ]]; then
		return 1
	fi
	[[ -f "$wifi_path" && ! -L "$wifi_path" ]] || return 1
	[[ "$(mode_of_path "$wifi_path")" == "600" && "$(owner_of_path "$wifi_path")" == "$(id -u)" ]] || return 1
	if [[ -z "$pool_path" ]]; then
		local -a pool_paths=()
		local candidate
		for candidate in "$repo_root"/pool-credentials*.json; do
			if [[ "$candidate" != *.example && -f "$candidate" && ! -L "$candidate" && "$(mode_of_path "$candidate")" == "600" && "$(owner_of_path "$candidate")" == "$(id -u)" ]]; then
				pool_paths+=("$candidate")
			fi
		done
		((${#pool_paths[@]} == 1)) || return 1
		pool_path="${pool_paths[0]}"
	fi
	[[ -f "$pool_path" && ! -L "$pool_path" ]] || return 1
	[[ "$(mode_of_path "$pool_path")" == "600" && "$(owner_of_path "$pool_path")" == "$(id -u)" ]]
}

write_consumed_qualification_tombstone() {
	local qualification_path="$1"
	local qualification_json="$2"
	local qualification_source_digest="$3"
	local formal_attempt_id="$4"
	local exact_head="$5"
	local qualification_attempt formal_attempt
	qualification_attempt="$(jq -er '.attempt_id' <<<"$qualification_json")"
	formal_attempt="$formal_attempt_id"
	atomic_write "$qualification_path" "$(jq -cn \
		--arg exact_head "$exact_head" \
		--arg qualification_attempt_sha256 "$(sha256_text "$qualification_attempt")" \
		--arg formal_attempt_sha256 "$(sha256_text "$formal_attempt")" \
		--arg source_digest "$qualification_source_digest" \
		--arg contract "$(jq -er '.diagnostic_contract_digest_sha256' <<<"$qualification_json")" \
		'{schema_version:"ultra205-transport-qualification-consumed-v1",status:"consumed",exact_head:$exact_head,qualification_attempt_id_sha256:$qualification_attempt_sha256,formal_attempt_id_sha256:$formal_attempt_sha256,qualification_source_digest_sha256:$source_digest,qualification_contract_digest_sha256:$contract,cleanup_complete:true,product_projection_imported:false}')"
}

build_qualification_handoff() {
	local qualification_json="$1"
	local qualification_source_digest="$2"
	local formal_attempt_id="$3"
	local exact_head="$4"
	local qualification_attempt
	qualification_attempt="$(jq -er '.attempt_id' <<<"$qualification_json")"
	jq -cn \
		--arg exact_head "$exact_head" \
		--arg qualification_attempt_sha256 "$(sha256_text "$qualification_attempt")" \
		--arg formal_attempt_sha256 "$(sha256_text "$formal_attempt_id")" \
		--arg source_digest "$qualification_source_digest" \
		--arg contract "$(jq -er '.diagnostic_contract_digest_sha256' <<<"$qualification_json")" \
		'{schema_version:"phase28.1.1-native-qualification-handoff-v1",qualification_status:"consumed",exact_head:$exact_head,qualification_attempt_id_sha256:$qualification_attempt_sha256,formal_attempt_id_sha256:$formal_attempt_sha256,qualification_source_digest_sha256:$source_digest,qualification_contract_digest_sha256:$contract,source_root_distinct:true,product_projection_empty:true,cleanup_complete:true,live_process_count:0,serial_holder_count:0,live_socket_count:0}'
}

validate_qualification_handoff() {
	local handoff_path="$1"
	local expected_head="$2"
	local expected_formal_attempt_id="$3"
	[[ -f "$handoff_path" && ! -L "$handoff_path" ]] || return 1
	[[ "$(mode_of_path "$handoff_path")" == "600" && "$(owner_of_path "$handoff_path")" == "$(id -u)" ]] || return 1
	jq -e \
		--arg expected_head "$expected_head" \
		--arg expected_formal_attempt_sha256 "$(sha256_text "$expected_formal_attempt_id")" '
		  type == "object" and
		  (keys | sort) == (["cleanup_complete","exact_head","formal_attempt_id_sha256","live_process_count","live_socket_count","product_projection_empty","qualification_attempt_id_sha256","qualification_contract_digest_sha256","qualification_source_digest_sha256","qualification_status","schema_version","serial_holder_count","source_root_distinct"] | sort) and
		  .schema_version == "phase28.1.1-native-qualification-handoff-v1" and
		  .qualification_status == "consumed" and
		  .exact_head == $expected_head and
		  .formal_attempt_id_sha256 == $expected_formal_attempt_sha256 and
		  (.qualification_attempt_id_sha256 | test("^[0-9a-f]{64}$")) and
		  .qualification_attempt_id_sha256 != .formal_attempt_id_sha256 and
		  (.qualification_source_digest_sha256 | test("^[0-9a-f]{64}$")) and
		  (.qualification_contract_digest_sha256 | test("^[0-9a-f]{64}$")) and
		  .source_root_distinct == true and
		  .product_projection_empty == true and
		  .cleanup_complete == true and
		  .live_process_count == 0 and
		  .serial_holder_count == 0 and
		  .live_socket_count == 0
		' "$handoff_path" >/dev/null
}

begin_attempt() {
	local generation=1
	local hardware_exact_head=""
	local transport_qualification=""
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
		--transport-qualification)
			transport_qualification="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	[[ "$generation" =~ ^[0-9]+$ ]] || die "state_malformed"
	[[ "$hardware_exact_head" =~ ^[0-9a-f]{40}$ ]] || die "exact_head_mismatch"
	[[ -n "$transport_qualification" ]] || die "transport_qualification_missing"
	ensure_private_root
	require_clean_head
	[[ "$(current_head)" == "$hardware_exact_head" ]] || die "exact_head_mismatch"
	[[ -x "$transport_qualification_helper" ]] || die "transport_qualification_invalid"
	qualification_parent_is_private "$transport_qualification" || die "transport_qualification_invalid"
	local qualification_source_digest
	local qualification_json
	qualification_source_digest="$(shasum -a 256 "$transport_qualification" 2>/dev/null | awk '{print $1}')" || die "transport_qualification_invalid"
	"$transport_qualification_helper" validate-native "$transport_qualification" "$hardware_exact_head" >/dev/null 2>&1 || die "transport_qualification_invalid"
	qualification_json="$(jq -ce . "$transport_qualification" 2>/dev/null)" || die "transport_qualification_invalid"
	[[ "$(shasum -a 256 "$transport_qualification" 2>/dev/null | awk '{print $1}')" == "$qualification_source_digest" ]] || die "transport_qualification_invalid"
	formal_credential_paths_ready || die "credential_binding_failed"
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
	local handoff_json
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
	[[ "$attempt_id" != "$(jq -er '.attempt_id' <<<"$qualification_json")" ]] || die "resume_handle_ambiguous"
	boot_digest="$(jq -er '.boot_session_sha256' <<<"$state_json")"
	attempt_dir="$attempts_root/$attempt_id"
	[[ "$(dirname "$transport_qualification")" != "$attempt_dir" ]] || die "transport_qualification_invalid"
	[[ "$(shasum -a 256 "$transport_qualification" 2>/dev/null | awk '{print $1}')" == "$qualification_source_digest" ]] || die "transport_qualification_invalid"
	handoff_json="$(build_qualification_handoff "$qualification_json" "$qualification_source_digest" "$attempt_id" "$exact_head")"
	write_consumed_qualification_tombstone "$transport_qualification" "$qualification_json" "$qualification_source_digest" "$attempt_id" "$exact_head"
	mkdir -m 700 "$attempt_dir"
	atomic_write "$attempt_dir/state.json" "$state_json"
	atomic_write "$attempt_dir/qualification-handoff.json" "$handoff_json"
	validate_qualification_handoff "$attempt_dir/qualification-handoff.json" "$exact_head" "$attempt_id" || die "transport_qualification_invalid"
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
	local now
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
	assert_fresh_identity "$state_path"
	if [[ "$(jq -r '.attempt_state' "$state_path")" == "terminal" ]]; then
		cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
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
	if [[ "$(jq -er '.attempt_state' "$state_path")" == "restore_watcher_armed" ]]; then
		public_action_output "$state_path"
	else
		public_checkpoint_output "$state_path" "$handle"
	fi
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
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
	assert_fresh_identity "$state_path"
	[[ "$(jq -r '.checkpoint_id // empty' "$state_path")" != "" ]] || die "checkpoint_state_mismatch"
	[[ "$(jq -r '.checkpoint_token // empty' "$state_path")" == "$checkpoint_token" ]] || die "checkpoint_token_mismatch"
	[[ "$(jq -r '.expected_response_token // empty' "$state_path")" == "$response_token" ]] || die "checkpoint_token_mismatch"
	deadline="$(jq -er '.monotonic_deadline_ms' "$state_path")"
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
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
	if [[ "$checkpoint_token" == "plan13-armed-removal-v1" ]]; then
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
	fail_closed_if_expired "$slot" "$state_path" "$now"
	local consumed_json
	consumed_json="$(state_operation consume-checkpoint "$state_path" "$(jq -cn --arg token "$checkpoint_token" --arg response "$response_token" --argjson now "$now" '{checkpointToken:$token,responseToken:$response,nowMonotonicMs:$now}')")" || die "checkpoint_state_mismatch"
	persist_state_and_slot "$state_path" "$slot" "$consumed_json"
	now="$(monotonic_ms)"
	if ((now >= deadline)); then
		fail_closed_active "$slot" "$state_path" checkpoint_expired checkpoint_expired
	fi
	if [[ "$lifecycle_delivery" == "true" ]]; then
		local frame
		frame="$(jq -cn --arg digest "$(sha256_text "$handle")" --argjson generation "$checkpoint_generation" --arg token "$checkpoint_token" --arg response "$response_token" --arg nonce "$nonce" --arg lease "$lease" '{resume_handle_sha256:$digest,checkpoint_generation:$generation,checkpoint_token:$token,response_token:$response,effect_authorization_nonce:$nonce,lifecycle_lease_id:$lease}')"
		set +e
		local sender="${PHASE28_TEST_SOCKET_SEND_BIN:-}"
		if [[ -n "$sender" ]]; then
			[[ "${PHASE28_TEST_MODE:-0}" == "1" && -x "$sender" ]] || die "private_capability_invalid"
			"$sender" "$attempt_dir/lifecycle.sock" "$frame"
		else
			printf '%s' "$frame" | (cd "$attempt_dir" && perl "$lifecycle_frame_helper" send --socket lifecycle.sock)
		fi
		local sender_status=$?
		set -e
		if ((sender_status != 0)); then
			local terminal_json
			terminal_json="$(state_operation terminalize "$state_path" '{"blockerReason":"private_capability_invalid","cleanupUnresolved":false}')"
			persist_state_and_slot "$state_path" "$slot" "$terminal_json"
			cleanup_persisted_terminal_active "$slot" "$state_path" blocked >/dev/null
			release_lock
			die "private_capability_invalid"
		fi
	fi
	release_lock
	printf 'checkpoint_delivery=accepted\n'
	printf 'checkpoint_generation=%s\n' "$checkpoint_generation"
}

cleanup_attempt_process() {
	local slot="$1"
	local state_path="$2"
	local attempt_dir
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	local child_pid=""
	if [[ -e "$attempt_dir/child.pid" ]]; then
		[[ -f "$attempt_dir/child.pid" && "$(mode_of_path "$attempt_dir/child.pid")" == "600" ]] || return 1
		child_pid="$(sed -n '1p' "$attempt_dir/child.pid")"
		[[ "$child_pid" =~ ^[1-9][0-9]*$ ]] || return 1
	fi
	if [[ -z "$child_pid" ]]; then
		[[ "$(jq -er '.process_running' "$state_path")" == "false" ]] || return 1
		return 0
	fi
	local leader_alive=false
	local group_alive=false
	phase_process_is_alive "$child_pid" && leader_alive=true
	phase_process_group_is_alive "$child_pid" && group_alive=true
	if [[ "$leader_alive" == "false" && "$group_alive" == "false" ]]; then
		return 0
	fi
	if [[ "$leader_alive" == "true" ]]; then
		local pgid
		pgid="$(ps -o pgid= -p "$child_pid" 2>/dev/null | tr -d ' ')"
		[[ "$pgid" == "$child_pid" ]] || return 1
	fi
	if [[ "$(jq -er '.process_running' "$state_path")" == "true" ]]; then
		[[ "$(jq -er '.lifecycle_owner_pid' "$state_path")" == "$child_pid" ]] || return 1
		if [[ "$leader_alive" == "true" ]]; then
			local observed_fingerprint
			observed_fingerprint="$(process_start_fingerprint "$child_pid" 2>/dev/null)" || return 1
			[[ "$observed_fingerprint" == "$(jq -er '.lifecycle_owner_start_fingerprint_sha256' "$state_path")" ]] || return 1
		fi
	fi
	terminate_effect_process "$child_pid" "phase28 attempt cleanup"
}

escrow_attempt_traces() {
	local attempt_dir="$1"
	local attempt_id
	attempt_id="$(basename "$attempt_dir")"
	local escrow_root="${PHASE28_TRACE_ESCROW_ROOT:-$repo_root/scratch/phase28.1.1-plan13-private-traces}"
	local escrow_dir="$escrow_root/$attempt_id"
	local -a trace_files=()
	local candidate
	for candidate in \
		"$attempt_dir/cold-start-monitor.raw.log" \
		"$attempt_dir/monitor-wrapper.raw.log"; do
		[[ ! -f "$candidate" ]] || trace_files+=("$candidate")
	done
	if [[ -d "$attempt_dir/private-session-traces" ]]; then
		while IFS= read -r -d '' candidate; do
			trace_files+=("$candidate")
		done < <(find "$attempt_dir/private-session-traces" -type f -print0)
	fi
	((${#trace_files[@]} > 0)) || return 0
	umask 077
	mkdir -p "$escrow_root"
	chmod 700 "$escrow_root"
	if [[ ! -d "$escrow_dir" ]]; then
		mkdir "$escrow_dir"
	fi
	chmod 700 "$escrow_dir"
	local index=0
	for candidate in "${trace_files[@]}"; do
		index=$((index + 1))
		local destination
		destination="$escrow_dir/trace-$(printf '%03d' "$index").raw"
		cp "$candidate" "$destination" || return 1
		chmod 600 "$destination" || return 1
	done
	local digest_manifest="$escrow_dir/trace-digests.sha256"
	: >"$digest_manifest"
	chmod 600 "$digest_manifest"
	for candidate in "$escrow_dir"/trace-*.raw; do
		shasum -a 256 "$candidate" >>"$digest_manifest" || return 1
	done
}

replace_active_with_tombstone() {
	local slot="$1"
	local terminal="$2"
	local category="$3"
	local attempt_dir
	local generation
	local digest
	local state_path
	local handoff_path
	local exact_head
	local attempt_id
	local lifecycle_id
	local verification_result
	local phase30_promotion_input
	local blocker_reason
	local contract_digest
	local cleanup_complete=false
	local live_resource_count=1
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	generation="$(jq -er '.attempt_generation' "$slot")"
	digest="$(jq -er '.resume_handle_sha256' "$slot")"
	state_path="$attempt_dir/state.json"
	handoff_path="$attempt_dir/qualification-handoff.json"
	validate_state "$state_path" || die "state_malformed"
	exact_head="$(jq -er '.exact_head' "$state_path")"
	attempt_id="$(jq -er '.attempt_id' "$state_path")"
	lifecycle_id="$(jq -er '.phase_lifecycle_id' "$state_path")"
	verification_result="$(jq -er '.verification_result' "$state_path")"
	phase30_promotion_input="$(jq -er '.phase30_promotion_input' "$state_path")"
	blocker_reason="$(jq -er '.blocker_reason' "$state_path")"
	contract_digest="$(jq -er '.qualification_contract_digest_sha256 | select(type == "string" and test("^[0-9a-f]{64}$"))' "$handoff_path" 2>/dev/null || true)"
	if [[ -z "$contract_digest" ]]; then
		contract_digest="$(sha256_text "invalid-qualification-handoff-v1")"
	fi
	if [[ "$(jq -er '.cleanup_state' "$state_path")" == "complete" ]]; then
		cleanup_complete=true
		live_resource_count=0
	fi
	if ! escrow_attempt_traces "$attempt_dir"; then
		printf 'phase28_attempt_warning=trace_escrow_incomplete\n' >&2
	fi
	rm -f "$attempt_dir/child.pid" "$attempt_dir/lifecycle.sock" "$attempt_dir/credential-capability.json" "$attempt_dir/effect.ack" "$attempt_dir/effect.gate" "$attempt_dir/effect-result.json"
	[[ "${PHASE28_CRASH_AT:-}" != "before_tombstone_persistence" ]] || exit 97
	atomic_write "$slot" "$(jq -cn \
		--arg digest "$digest" \
		--arg terminal "$terminal" \
		--arg category "$category" \
		--arg exact_head "$exact_head" \
		--arg formal_attempt_id_sha256 "$(sha256_text "$attempt_id")" \
		--arg lifecycle_id "$lifecycle_id" \
		--arg verification_result "$verification_result" \
		--arg phase30 "$phase30_promotion_input" \
		--arg blocker "$blocker_reason" \
		--arg contract "$contract_digest" \
		--argjson generation "$generation" \
		--argjson cleanup_complete "$cleanup_complete" \
		--argjson live_resource_count "$live_resource_count" \
		'{schema_version:"exact-head-resume-tombstone-v2",resume_handle_sha256:$digest,attempt_generation:$generation,terminal_status:"closed",terminal_category:$terminal,cleanup_time_category:$category,exact_head:$exact_head,formal_attempt_id_sha256:$formal_attempt_id_sha256,phase_lifecycle_id:$lifecycle_id,verification_result:$verification_result,phase30_promotion_input:$phase30,blocker_reason:$blocker,qualification_contract_digest_sha256:$contract,cleanup_complete:$cleanup_complete,live_process_count:$live_resource_count,serial_holder_count:$live_resource_count,live_socket_count:$live_resource_count}')"
	rm -rf "$attempt_dir"
	[[ "${PHASE28_CRASH_AT:-}" != "after_tombstone_persistence" ]] || exit 97
	[[ "${PHASE28_CRASH_AT:-}" != "after_terminal_cleanup" ]] || exit 97
}

terminalize_and_cleanup_active() {
	local slot="$1"
	local state_path="$2"
	local blocker="$3"
	local category="$4"
	local cleanup_unresolved=false
	if ! cleanup_attempt_process "$slot" "$state_path"; then
		cleanup_unresolved=true
	fi
	local terminal_json
	terminal_json="$(state_operation terminalize "$state_path" "$(jq -cn --arg blockerReason "$blocker" --argjson cleanupUnresolved "$cleanup_unresolved" '{blockerReason:$blockerReason,cleanupUnresolved:$cleanupUnresolved}')")"
	persist_state_and_slot "$state_path" "$slot" "$terminal_json"
	local terminal
	terminal="$(jq -er '.terminal_outcome' "$state_path")"
	replace_active_with_tombstone "$slot" "$terminal" "$category"
	printf '%s\n' "$terminal"
}

cleanup_persisted_terminal_active() {
	local slot="$1"
	local state_path="$2"
	local category="$3"
	[[ "$(jq -er '.attempt_state' "$state_path")" == "terminal" ]] || die "state_malformed"
	if ! cleanup_attempt_process "$slot" "$state_path"; then
		local unresolved_json
		unresolved_json="$(state_operation terminalize "$state_path" '{"blockerReason":"process_cleanup_unresolved","cleanupUnresolved":true}')"
		persist_state_and_slot "$state_path" "$slot" "$unresolved_json"
	fi
	local terminal
	terminal="$(jq -er '.terminal_outcome' "$state_path")"
	replace_active_with_tombstone "$slot" "$terminal" "$category"
	printf '%s\n' "$terminal"
}

fail_closed_active() {
	local slot="$1"
	local state_path="$2"
	local blocker="$3"
	local error="$4"
	local category="${5:-blocked}"
	terminalize_and_cleanup_active "$slot" "$state_path" "$blocker" "$category" >/dev/null
	release_lock
	die "$error"
}

fail_closed_if_expired() {
	local slot="$1"
	local state_path="$2"
	local now="$3"
	local maybe_blocker
	maybe_blocker="$(active_expiry_blocker "$state_path" "$now")"
	if [[ -n "$maybe_blocker" ]]; then
		fail_closed_active "$slot" "$state_path" "$maybe_blocker" "$maybe_blocker"
	fi
}

finalize_successful_classification() {
	local slot="$1"
	local state_path="$2"
	local now
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
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
	cleanup_persisted_terminal_active "$slot" "$state_path" "$([[ "$terminal" == blocked_safe_* ]] && printf blocked || printf normal)" >/dev/null
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
	absence-observing | restore-watcher-armed | usb-reappearance-observed | capture-running | capture-complete) ;;
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
	local now
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
	assert_fresh_identity "$state_path"
	[[ "$(sha256_text "$capability")" == "$(jq -er '.lifecycle_capability_sha256' "$state_path")" ]] || die "lease_owner_mismatch"
	[[ "$PPID" == "$(jq -er '.lifecycle_owner_pid' "$state_path")" ]] || die "lease_owner_mismatch"
	[[ "$(process_start_fingerprint "$PPID")" == "$(jq -er '.lifecycle_owner_start_fingerprint_sha256' "$state_path")" ]] || die "lease_dead_or_reused_process"
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
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
	local now
	now="$(monotonic_ms)"
	fail_closed_if_expired "$slot" "$state_path" "$now"
	assert_fresh_identity "$state_path"
	if [[ "$(jq -r '.attempt_state' "$state_path")" == "terminal" ]]; then
		cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
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
		expired) fail_closed_active "$slot" "$state_path" checkpoint_expired expired ;;
		token_mismatch | exact_head_mismatch | manifest_mismatch | reference_mismatch | boot_session_mismatch | dirty_head | malformed_state | validator_error | lock_failure | persistence_failure | lease_conflict) die "$PHASE28_INJECT_INVALID_CATEGORY" ;;
		*) die "validator_error" ;;
		esac
	fi
	phase="$(jq -r '.effect_phase' "$state_path")"
	if [[ "$phase" == "authorized" || "$phase" == "invoked" ]]; then
		local ambiguous_json
		ambiguous_json="$(state_operation terminalize "$state_path" '{"blockerReason":"effect_in_flight_ambiguous","cleanupUnresolved":false}')"
		persist_state_and_slot "$state_path" "$slot" "$ambiguous_json"
		cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
		release_lock
		die "effect_in_flight_ambiguous"
	fi
	if [[ "$phase" == "completed" || "$phase" == "failed" ]]; then
		local retained_result="$attempt_dir/effect-result.json"
		[[ -f "$retained_result" ]] || {
			local inconsistent_json
			inconsistent_json="$(state_operation terminalize "$state_path" '{"blockerReason":"effect_in_flight_ambiguous","cleanupUnresolved":false}')"
			persist_state_and_slot "$state_path" "$slot" "$inconsistent_json"
			cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
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
			cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
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
			cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
			release_lock
			printf 'terminal_outcome=%s\n' "$terminal"
			return
		fi
		rm -f "$attempt_dir/child.pid"
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
		PHASE28_LIFECYCLE_FRAME_HELPER="$lifecycle_frame_helper"
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
		local lease_duration_ms
		lease_duration_ms="$(lifecycle_lease_duration_ms)"
		local attach_args
		attach_args="$(jq -cn --arg leaseId "$lifecycle_lease" --arg capabilitySha256 "$(sha256_text "$lifecycle_capability")" --argjson ownerPid "$child_pid" --arg ownerStartFingerprintSha256 "$owner_fingerprint" --argjson lifecycleDeadlineMs "$((attach_now + lease_duration_ms))" --argjson checkpointCreatedMonotonicMs "$attach_now" '{leaseId:$leaseId,capabilitySha256:$capabilitySha256,ownerPid:$ownerPid,ownerStartFingerprintSha256:$ownerStartFingerprintSha256,lifecycleDeadlineMs:$lifecycleDeadlineMs,checkpointCreatedMonotonicMs:$checkpointCreatedMonotonicMs}')"
		persist_state_and_slot "$state_path" "$slot" "$(state_operation attach-lifecycle "$state_path" "$attach_args")"
	fi
	[[ "${PHASE28_CRASH_AT:-}" != "after_invoked_persistence" ]] || exit 97
	release_lock
	: >"$gate"
	[[ "${PHASE28_CRASH_AT:-}" != "after_start_gate_release" ]] || exit 97
	set +e
	if [[ "$effect" == "lifecycle_start" ]]; then
		public_checkpoint_output "$state_path" "$handle"
		local emitted_restore_action=false
		while kill -0 "$child_pid" 2>/dev/null; do
			if [[ -f "$state_path" ]]; then
				local maybe_attempt_state
				maybe_attempt_state="$(jq -r '.attempt_state // empty' "$state_path" 2>/dev/null)"
				if [[ "$maybe_attempt_state" == "restore_watcher_armed" && "$emitted_restore_action" == "false" ]]; then
					public_action_output "$state_path"
					emitted_restore_action=true
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
	rm -f "$ack" "$gate"
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
		cleanup_persisted_terminal_active "$slot" "$state_path" crash >/dev/null
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
		cleanup_persisted_terminal_active "$slot" "$state_path" "$([[ "$terminal" == blocked_safe_* ]] && printf blocked || printf normal)" >/dev/null
		release_lock
		printf 'effect_id=%s\n' "$effect"
		printf 'effect_sequence=%s\n' "$sequence"
		printf 'effect_status=%s\n' "$([[ "$succeeded" == true ]] && printf completed || printf failed)"
		printf 'terminal_outcome=%s\n' "$terminal"
		return
	fi
	rm -f "$attempt_dir/child.pid"
	release_lock
	printf 'effect_id=%s\n' "$effect"
	printf 'effect_sequence=%s\n' "$sequence"
	printf 'effect_status=%s\n' "$([[ "$succeeded" == true ]] && printf completed || printf failed)"
	if [[ "$(jq -r '.checkpoint_id // empty' "$state_path")" != "" ]]; then
		public_checkpoint_output "$state_path" "$handle"
	fi
}

cleanup_unique_orphan_attempt() {
	local expected_head=""
	local expected_state=""
	local reason=""
	while (($#)); do
		case "$1" in
		--expected-head)
			expected_head="${2:-}"
			shift 2
			;;
		--expected-state)
			expected_state="${2:-}"
			shift 2
			;;
		--reason)
			reason="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	[[ "$expected_head" =~ ^[0-9a-f]{40}$ ]] || die "orphan_cleanup_request_invalid"
	[[ "$expected_state" == "connected_entry_waiting" ]] || die "orphan_cleanup_request_invalid"
	[[ "$reason" == "lost_resume_handle" ]] || die "orphan_cleanup_request_invalid"

	if [[ ! -e "$private_root" ]]; then
		public_orphan_cleanup_failure "no_active_orphan" "zero" "zero"
	fi
	for private_directory in "$private_root" "$resume_index" "$attempts_root"; do
		if [[ ! -d "$private_directory" || -L "$private_directory" || "$(mode_of_path "$private_directory")" != "700" || "$(owner_of_path "$private_directory")" != "$(id -u)" ]]; then
			public_orphan_cleanup_failure "orphan_index_ambiguous" "unknown" "unknown"
		fi
	done
	require_clean_head
	acquire_lock

	local active_count=0
	local active_slot=""
	local tombstone_count=0
	local entry
	while IFS= read -r -d '' entry; do
		local entry_name
		local entry_digest
		entry_name="$(basename "$entry")"
		if [[ ! -f "$entry" || -L "$entry" || "$entry_name" != *.json || "$(mode_of_path "$entry")" != "600" || "$(owner_of_path "$entry")" != "$(id -u)" ]]; then
			public_orphan_cleanup_failure "orphan_index_ambiguous" "unknown" "unknown"
		fi
		entry_digest="${entry_name%.json}"
		if [[ ! "$entry_digest" =~ ^[0-9a-f]{64}$ ]]; then
			public_orphan_cleanup_failure "orphan_index_ambiguous" "unknown" "unknown"
		fi
		if validate_active_slot "$entry" "$entry_digest"; then
			active_count=$((active_count + 1))
			if ((active_count == 1)); then
				active_slot="$entry"
			fi
		elif validate_tombstone "$entry" "$entry_digest"; then
			tombstone_count=$((tombstone_count + 1))
		else
			public_orphan_cleanup_failure "orphan_index_ambiguous" "unknown" "unknown"
		fi
	done < <(find "$resume_index" -mindepth 1 -maxdepth 1 -print0)

	local active_count_category
	local tombstone_count_category
	active_count_category="$(count_category "$active_count")"
	tombstone_count_category="$(count_category "$tombstone_count")"
	if ((active_count == 0)); then
		public_orphan_cleanup_failure "no_active_orphan" "$active_count_category" "$tombstone_count_category"
	fi
	if ((active_count != 1)); then
		public_orphan_cleanup_failure "orphan_index_ambiguous" "$active_count_category" "$tombstone_count_category"
	fi

	local slot="$active_slot"
	local attempt_dir
	local state_path
	local qualification_path
	attempt_dir="$(jq -er '.attempt_dir' "$slot" 2>/dev/null)" || public_orphan_cleanup_failure "orphan_index_ambiguous" "one" "$tombstone_count_category"
	state_path="$attempt_dir/state.json"
	qualification_path="$attempt_dir/qualification-handoff.json"
	if ! validate_slot_state "$slot" "$state_path" 2>/dev/null || ! validate_state "$state_path" 2>/dev/null; then
		public_orphan_cleanup_failure "orphan_state_ineligible" "one" "$tombstone_count_category"
	fi
	if ! validate_qualification_handoff "$qualification_path" "$expected_head" "$(jq -er '.attempt_id' "$state_path")"; then
		public_orphan_cleanup_failure "orphan_state_ineligible" "one" "$tombstone_count_category"
	fi

	local attempt_entry_count=0
	local observed_attempt_entry=""
	while IFS= read -r -d '' entry; do
		attempt_entry_count=$((attempt_entry_count + 1))
		observed_attempt_entry="$entry"
	done < <(find "$attempts_root" -mindepth 1 -maxdepth 1 -print0)
	if ((attempt_entry_count != 1)) || [[ "$observed_attempt_entry" != "$attempt_dir" || ! -d "$attempt_dir" || -L "$attempt_dir" || "$(mode_of_path "$attempt_dir")" != "700" || "$(owner_of_path "$attempt_dir")" != "$(id -u)" ]]; then
		public_orphan_cleanup_failure "orphan_index_ambiguous" "one" "$tombstone_count_category"
	fi
	local live_entry_count=0
	local observed_state=false
	local observed_qualification=false
	while IFS= read -r -d '' entry; do
		live_entry_count=$((live_entry_count + 1))
		[[ "$entry" == "$state_path" ]] && observed_state=true
		[[ "$entry" == "$qualification_path" ]] && observed_qualification=true
	done < <(find "$attempt_dir" -mindepth 1 -maxdepth 1 -print0)
	if ((live_entry_count != 2)) || [[ "$observed_state" != "true" || "$observed_qualification" != "true" || ! -f "$state_path" || -L "$state_path" || ! -f "$qualification_path" || -L "$qualification_path" ]]; then
		public_orphan_cleanup_failure "orphan_live_reference_present" "one" "$tombstone_count_category"
	fi

	local observed_boot
	if ! observed_boot="$(observe_boot_digest 2>/dev/null)"; then
		public_orphan_cleanup_failure "orphan_state_ineligible" "one" "$tombstone_count_category"
	fi
	local orphan_state_category
	if ! orphan_state_category="$(classify_orphan_state "$state_path" "$expected_head" "$expected_state" "$reason" "$observed_boot" 2>/dev/null)"; then
		public_orphan_cleanup_failure "orphan_state_ineligible" "one" "$tombstone_count_category"
	fi

	local terminal
	if [[ "$orphan_state_category" == "connected_entry_waiting" ]]; then
		terminal="$(terminalize_and_cleanup_active "$slot" "$state_path" "cancelled_or_abandoned" abandoned)"
	elif [[ "$orphan_state_category" == "terminal_recovery" ]]; then
		terminal="$(cleanup_persisted_terminal_active "$slot" "$state_path" abandoned)"
	else
		public_orphan_cleanup_failure "orphan_state_ineligible" "one" "$tombstone_count_category"
	fi
	if [[ "$terminal" != "blocked_safe_attempt_prerequisite" ]]; then
		release_lock
		die "orphan_cleanup_terminal_inconsistent"
	fi
	local slot_digest
	slot_digest="$(basename "$slot" .json)"
	if ! validate_tombstone "$slot" "$slot_digest" || [[ -e "$attempt_dir" ]]; then
		release_lock
		die "orphan_cleanup_terminal_inconsistent"
	fi
	release_lock
	printf 'cleanup_result=closed\n'
	printf 'terminal_category=blocked_safe_attempt_prerequisite\n'
	printf 'active_orphan_count_category=zero\n'
	printf 'tombstone_count_increased=true\n'
	printf 'live_reference_count_category=zero\n'
	printf 'process_running=false\n'
	printf 'effect_sentinels_invoked=0\n'
	printf 'positive_evidence_promoted=false\n'
	printf 'resume_handle_reconstructed=false\n'
}

cleanup_active_attempt() {
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
	local attempt_dir
	local state_path
	slot="$(resolve_handle "$handle")"
	attempt_dir="$(jq -er '.attempt_dir' "$slot")"
	state_path="$attempt_dir/state.json"
	validate_state "$state_path" || die "state_malformed"
	validate_slot_state "$slot" "$state_path"
	if [[ "$(jq -er '.attempt_state' "$state_path")" == "terminal" ]]; then
		local retained_terminal
		retained_terminal="$(cleanup_persisted_terminal_active "$slot" "$state_path" blocked)"
		release_lock
		printf 'cleanup_result=closed\n'
		printf 'terminal_outcome=%s\n' "$retained_terminal"
		printf 'process_running=false\n'
		printf 'effect_sentinels_invoked=0\n'
		printf 'positive_evidence_promoted=false\n'
		return
	fi
	local blocker=""
	local now
	now="$(monotonic_ms)"
	blocker="$(active_expiry_blocker "$state_path" "$now")"
	if [[ -z "$blocker" && "${PHASE28_ALLOW_DIRTY_TEST:-0}" != "1" && -n "$(git -C "$repo_root" status --porcelain=v1)" ]]; then
		blocker="dirty_head"
	fi
	if [[ -z "$blocker" && "$(current_head)" != "$(jq -er '.exact_head' "$state_path")" ]]; then
		blocker="exact_head_mismatch"
	fi
	if [[ -z "$blocker" ]]; then
		local observed_boot=""
		if ! observed_boot="$(observe_boot_digest 2>/dev/null)"; then
			blocker="boot_session_observation_unavailable"
		elif [[ "$observed_boot" != "$(jq -er '.boot_session_sha256' "$state_path")" ]]; then
			blocker="boot_session_mismatch"
		fi
	fi
	if [[ -z "$blocker" ]]; then
		blocker="cancelled_or_abandoned"
	fi
	local terminal
	terminal="$(terminalize_and_cleanup_active "$slot" "$state_path" "$blocker" blocked)"
	release_lock
	printf 'cleanup_result=closed\n'
	printf 'terminal_outcome=%s\n' "$terminal"
	printf 'process_running=false\n'
	printf 'effect_sentinels_invoked=0\n'
	printf 'positive_evidence_promoted=false\n'
}

parse_terminal_artifact() {
	local artifact_path="$1"
	node --input-type=module -e '
import fs from "node:fs";
const text = fs.readFileSync(process.argv[1], "utf8");
const lines = text.split(/\r?\n/u);
const delimiters = lines.flatMap((line, index) => line === "---" ? [index] : []);
if (delimiters.length !== 2 || delimiters[0] !== 0) throw new Error("frontmatter delimiters are not closed");
if (lines.slice(delimiters[1] + 1).some((line) => line.trim() !== "")) throw new Error("artifact body is not closed");
const value = {};
const numericFields = new Set(["live_process_count", "serial_holder_count", "live_socket_count"]);
for (const line of lines.slice(1, delimiters[1])) {
  const match = /^([a-z0-9_]+): (.+)$/u.exec(line);
  if (!match || Object.hasOwn(value, match[1])) throw new Error("frontmatter field is malformed");
  const raw = match[2];
  value[match[1]] = raw === "true" ? true : raw === "false" ? false : numericFields.has(match[1]) && /^[0-9]+$/u.test(raw) ? Number(raw) : raw;
}
const expected = ["blocker_reason","cleanup_complete","exact_head","formal_attempt_id_sha256","formal_status","live_process_count","live_socket_count","phase","phase30_promotion_input","phase_lifecycle_id","qualification_contract_digest_sha256","redacted","serial_holder_count","terminal_outcome","verification_result"].sort();
const actual = Object.keys(value).sort();
if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) throw new Error("frontmatter key set is not closed");
process.stdout.write(JSON.stringify(value));' "$artifact_path"
}

verify_terminal() {
	local handle=""
	local artifact_path=""
	while (($#)); do
		case "$1" in
		--resume-handle)
			handle="${2:-}"
			shift 2
			;;
		--redacted-artifact)
			artifact_path="${2:-}"
			shift 2
			;;
		*) die "unknown_argument" ;;
		esac
	done
	[[ "$handle" =~ ^[0-9a-f]{64}$ ]] || die "resume_handle_malformed"
	[[ -f "$artifact_path" && ! -L "$artifact_path" ]] || die "terminal_artifact_invalid"
	[[ -d "$private_root" && -d "$resume_index" && -d "$attempts_root" ]] || die "terminal_artifact_invalid"
	local digest
	local slot
	digest="$(sha256_text "$handle")"
	slot="$resume_index/$digest.json"
	[[ -f "$slot" && ! -L "$slot" && "$(mode_of_path "$slot")" == "600" && "$(owner_of_path "$slot")" == "$(id -u)" ]] || die "resume_handle_wrong"
	validate_formal_tombstone "$slot" "$digest" || die "terminal_artifact_invalid"
	[[ "$(jq -er '.exact_head' "$slot")" == "$(current_head)" ]] || die "exact_head_mismatch"
	[[ "$(jq -er '.phase_lifecycle_id' "$slot")" == "$phase_lifecycle_id" ]] || die "terminal_artifact_invalid"
	[[ "$(jq -er '.cleanup_complete' "$slot")" == "true" ]] || die "terminal_artifact_invalid"
	[[ "$(jq -er '.live_process_count' "$slot")" == "0" && "$(jq -er '.serial_holder_count' "$slot")" == "0" && "$(jq -er '.live_socket_count' "$slot")" == "0" ]] || die "terminal_artifact_invalid"
	[[ -z "$(find "$attempts_root" -mindepth 1 -maxdepth 1 -print -quit)" ]] || die "terminal_resources_live"
	[[ -z "$(find "$private_root" \( -type s -o -name child.pid \) -print -quit)" ]] || die "terminal_resources_live"
	local artifact_json
	artifact_json="$(parse_terminal_artifact "$artifact_path" 2>/dev/null)" || die "terminal_artifact_invalid"
	jq -e \
		--arg phase_lifecycle_id "$phase_lifecycle_id" \
		--arg exact_head "$(jq -er '.exact_head' "$slot")" \
		--arg formal_attempt_id_sha256 "$(jq -er '.formal_attempt_id_sha256' "$slot")" \
		--arg terminal "$(jq -er '.terminal_category' "$slot")" \
		--arg verification "$(jq -er '.verification_result' "$slot")" \
		--arg phase30 "$(jq -er '.phase30_promotion_input' "$slot")" \
		--arg blocker "$(jq -er '.blocker_reason' "$slot")" \
		--arg contract "$(jq -er '.qualification_contract_digest_sha256' "$slot")" '
		  .phase == "28.1.1" and
		  .phase_lifecycle_id == $phase_lifecycle_id and
		  .exact_head == $exact_head and
		  .formal_attempt_id_sha256 == $formal_attempt_id_sha256 and
		  .formal_status == "closed" and
		  .terminal_outcome == $terminal and
		  .verification_result == $verification and
		  .phase30_promotion_input == $phase30 and
		  .blocker_reason == $blocker and
		  .qualification_contract_digest_sha256 == $contract and
		  .cleanup_complete == true and
		  .live_process_count == 0 and
		  .serial_holder_count == 0 and
		  .live_socket_count == 0 and
		  .redacted == true and
		  (if .terminal_outcome == "passed_same_chain_hardware" then
		     .verification_result == "passed" and .phase30_promotion_input == "eligible" and .blocker_reason == "none"
		   else
		     .verification_result == "gaps_found" and .phase30_promotion_input == "pending"
		   end)
		' <<<"$artifact_json" >/dev/null || die "terminal_artifact_invalid"
	printf 'terminal_verification=passed\n'
	printf 'terminal_outcome=%s\n' "$(jq -er '.terminal_category' "$slot")"
	printf 'cleanup_complete=true\n'
	printf 'hardware_effects_invoked=0\n'
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
verify-terminal) verify_terminal "$@" ;;
cleanup-active-attempt) cleanup_active_attempt "$@" ;;
cleanup-unique-orphan-attempt) cleanup_unique_orphan_attempt "$@" ;;
*) die "unknown_command" ;;
esac
