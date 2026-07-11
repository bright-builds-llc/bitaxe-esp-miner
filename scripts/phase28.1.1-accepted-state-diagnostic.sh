#!/usr/bin/env bash
# Detector-gated Phase 28.1.1 accepted-state/lifecycle evidence wrapper.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=/dev/null
source "$repo_root/scripts/process-group.sh"
mode=""
effect_id=""
attempt=""
duration_seconds="360"
reattach_timeout_seconds="60"
attestation_timeout_seconds="300"
port=""
wifi_credentials=""
pool_credentials=""
evidence_out=""
manifest=""
reinit_log=""
board="205"
readonly minimum_usb_absence_ms=5000
readonly reattach_deadline_ms=60000
readonly monitor_start_reserve_ms=10000
readonly replay_interval_ms=2000
readonly replay_window_ms=180000
readonly latest_safe_replay_ms=72000
active_child_pid=""
active_descendant_group_file=""

die() {
	printf 'accepted_state_diagnostic_error: %s\n' "$1" >&2
	exit 1
}

trace_event() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_CALL_TRACE:-}" ]]; then
		printf '%s\n' "$1" >>"$PHASE28_ACCEPTED_STATE_CALL_TRACE"
	fi
}

usage() {
	printf 'usage: %s --mode blocked|hardware|plan13-prevalidated [--effect-id EFFECT] --attempt accepted-state|lifecycle --duration-seconds N --port PATH --wifi-credentials PATH --pool-credentials PATH --evidence-out PATH [--manifest PATH] [--reinit-log PATH] [--reattach-timeout-seconds 60] [--attestation-timeout-seconds 300]\n' "$(basename "$0")"
}

while (($#)); do
	case "$1" in
	--mode | --effect-id | --attempt | --duration-seconds | --reattach-timeout-seconds | --attestation-timeout-seconds | --port | --wifi-credentials | --pool-credentials | --evidence-out | --manifest | --reinit-log)
		[[ $# -ge 2 ]] || die "missing option value"
		name="${1#--}"
		case "$name" in
		mode) mode="$2" ;;
		effect-id) effect_id="$2" ;;
		attempt) attempt="$2" ;;
		duration-seconds) duration_seconds="$2" ;;
		reattach-timeout-seconds) reattach_timeout_seconds="$2" ;;
		attestation-timeout-seconds) attestation_timeout_seconds="$2" ;;
		port) port="$2" ;;
		wifi-credentials) wifi_credentials="$2" ;;
		pool-credentials) pool_credentials="$2" ;;
		evidence-out) evidence_out="$2" ;;
		manifest) manifest="$2" ;;
		reinit-log) reinit_log="$2" ;;
		esac
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*) die "unknown argument" ;;
	esac
done

if [[ "$mode" == "plan13-prevalidated" ]]; then
	case "$effect_id" in
	detector_board_info | credential_presence_bind | reference_guard | package | flash_reinit_runtime | lifecycle_start | post_capture_detector_board_info) ;;
	*) die "invalid --effect-id" ;;
	esac
	[[ -z "$attempt$port$wifi_credentials$pool_credentials$evidence_out$manifest$reinit_log" ]] || die "plan13-prevalidated rejects caller-owned inputs"
else
	[[ "$mode" == "blocked" || "$mode" == "hardware" ]] || die "invalid --mode"
	[[ -z "$effect_id" ]] || die "--effect-id is reserved for plan13-prevalidated"
	[[ "$attempt" == "accepted-state" || "$attempt" == "lifecycle" ]] || die "invalid --attempt"
	[[ "$duration_seconds" =~ ^[0-9]+$ ]] || die "--duration-seconds must be an integer"
	((duration_seconds >= 360)) || die "--duration-seconds must be at least 360"
	[[ "$reattach_timeout_seconds" =~ ^[0-9]+$ ]] || die "--reattach-timeout-seconds must be an integer"
	((reattach_timeout_seconds == 60)) || die "--reattach-timeout-seconds must be exactly 60"
	[[ "$attestation_timeout_seconds" =~ ^[0-9]+$ ]] || die "--attestation-timeout-seconds must be an integer"
	((attestation_timeout_seconds == 300)) || die "--attestation-timeout-seconds must be exactly 300"
	[[ -n "$evidence_out" ]] || die "--evidence-out is required"
fi

if [[ "$mode" == "plan13-prevalidated" ]]; then
	run_root=""
	run_dir=""
else
	run_root="${PHASE28_ACCEPTED_STATE_RUN_ROOT:-$repo_root/scratch/phase28.1.1-accepted-state}"
	run_dir="$run_root/$(date -u +%Y%m%dT%H%M%SZ)-${attempt}"
	mkdir -p "$run_dir"
	mkdir -p "$(dirname "$evidence_out")"
fi

cleanup_active_child() {
	local pid="${active_child_pid:-$PHASE_PROCESS_GROUP_PID}"
	if [[ -z "$pid" && (-z "$active_descendant_group_file" || ! -s "$active_descendant_group_file") ]]; then
		return 0
	fi
	local cleanup_failed=0

	if [[ -n "$pid" ]] && ! phase_process_group_terminate "$pid" "accepted-state monitor cleanup"; then
		printf 'accepted_state_diagnostic_error: monitor process group remains live\n' >&2
		cleanup_failed=1
	fi
	if [[ -n "$active_descendant_group_file" && -s "$active_descendant_group_file" ]]; then
		local descendant_pid
		descendant_pid="$(sed -n '1p' "$active_descendant_group_file")"
		if [[ ! "$descendant_pid" =~ ^[0-9]+$ ]]; then
			printf 'accepted_state_diagnostic_error: invalid descendant process-group state\n' >&2
			cleanup_failed=1
		elif ! phase_process_group_terminate "$descendant_pid" "accepted-state descendant monitor cleanup"; then
			printf 'accepted_state_diagnostic_error: descendant monitor process group remains live\n' >&2
			cleanup_failed=1
		else
			: >"$active_descendant_group_file"
		fi
	fi
	((cleanup_failed == 0)) || return 1
	active_child_pid=""
	PHASE_PROCESS_GROUP_PID=""
	active_descendant_group_file=""
	if [[ -n "${PHASE28_ACCEPTED_STATE_CHILD_PID_FILE:-}" ]]; then
		: >"$PHASE28_ACCEPTED_STATE_CHILD_PID_FILE"
	fi
	return 0
}

handle_exit() {
	local status=$?
	trap - EXIT INT TERM
	if ! cleanup_active_child; then
		status=1
	fi
	exit "$status"
}

handle_signal() {
	trap - EXIT INT TERM
	if ! cleanup_active_child; then
		exit 1
	fi
	exit 130
}

trap handle_exit EXIT
trap handle_signal INT TERM

write_unavailable_markers() {
	local stage
	for stage in post_enumerate post_mining_ready post_max_baud post_mask_reload post_first_work; do
		printf 'accepted_state_snapshot stage=%s observation=unavailable chip_count_class=unavailable readable_responses=0 error_counter_active=false domain_counter_active=false total_counter_active=false power_delta_class=unavailable result_correlated=false submit_observed=false redacted=true\n' "$stage"
	done
}

normalize_markers() {
	local source_log="$1"
	local normalized_log="$2"
	if [[ -f "$source_log" ]] && rg 'accepted_state_snapshot' "$source_log" >"$normalized_log"; then
		return
	fi
	write_unavailable_markers >"$normalized_log"
}

run_detector() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_DETECT_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_DETECT_BIN"
	else
		just detect-ultra205
	fi
}

run_verify_reference() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_VERIFY_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_VERIFY_BIN"
	else
		just verify-reference
	fi
}

run_package() {
	local -a args=(--investigation accepted_state_snapshot)
	trace_event "package"
	if [[ -n "${PHASE28_ACCEPTED_STATE_PACKAGE_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_PACKAGE_BIN" "${args[@]}"
	else
		bash scripts/phase27-live-hardware-bridge-package.sh "${args[@]}"
	fi
}

run_flash_capture() {
	local package_manifest="$1"
	local evidence_root="$run_dir/hardware"
	local -a args=(
		--evidence-root "$evidence_root"
		--manifest "$package_manifest"
		--mode hardware
		--pool-credentials "$pool_credentials"
		--wifi-credentials "$wifi_credentials"
		--duration-seconds "$duration_seconds"
		--port "$port"
		--redact-evidence=true
	)
	trace_event "flash-capture"
	if [[ -n "${PHASE28_ACCEPTED_STATE_CAPTURE_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_CAPTURE_BIN" "${args[@]}"
	else
		bash scripts/phase27-live-hardware-bridge-evidence.sh "${args[@]}"
	fi
}

port_is_present() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_PORT_PRESENT_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_PORT_PRESENT_BIN" "$port"
	else
		[[ -e "$port" ]]
	fi
}

wait_one_second() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_SLEEP_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_SLEEP_BIN" 1
	else
		sleep 1
	fi
}

monotonic_ms() {
	local value
	if [[ -n "${PHASE28_ACCEPTED_STATE_CLOCK_BIN:-}" ]]; then
		value="$("$PHASE28_ACCEPTED_STATE_CLOCK_BIN")"
	else
		value="$(perl -MTime::HiRes=clock_gettime,CLOCK_MONOTONIC -e 'printf "%.0f\n", clock_gettime(CLOCK_MONOTONIC) * 1000')"
	fi
	[[ "$value" =~ ^[0-9]+$ ]] || die "monotonic clock returned an invalid value"
	printf '%s\n' "$value"
}

read_token_once() {
	received_token=""
	set +e
	if [[ -n "${PHASE28_ACCEPTED_STATE_READ_BIN:-}" ]]; then
		received_token="$("$PHASE28_ACCEPTED_STATE_READ_BIN")"
		token_read_status=$?
	else
		# macOS Bash 3.2 rejects fractional read timeouts and conflates integer timeouts with EOF.
		received_token="$(perl -MIO::Select -MTime::HiRes=time -e '
			my $selector = IO::Select->new(\*STDIN);
			my $deadline = time() + 0.25;
			my $line = q{};
			while (1) {
				my $remaining = $deadline - time();
				exit 142 if $remaining <= 0;
				my @ready = $selector->can_read($remaining);
				exit 142 unless @ready;
				my $bytes_read = sysread(STDIN, my $character, 1);
				exit 1 if !defined($bytes_read) || $bytes_read == 0;
				last if $character eq "\n";
				$line .= $character unless $character eq "\r";
				exit 2 if length($line) > 64;
			}
			print $line;
		')"
		token_read_status=$?
	fi
	set -e
}

read_closed_token_until() {
	local expected_token="$1"
	local deadline_ms="$2"
	local label="$3"
	local now_ms

	while true; do
		now_ms="$(monotonic_ms)"
		((now_ms < deadline_ms)) || die "timed out waiting for $label token"
		read_token_once
		case "$token_read_status" in
		0)
			now_ms="$(monotonic_ms)"
			((now_ms < deadline_ms)) || die "timed out waiting for $label token"
			[[ "$received_token" == "$expected_token" ]] || die "invalid $label token"
			return
			;;
		1) die "stdin closed while waiting for $label token" ;;
		*)
			if ((token_read_status <= 128)); then
				die "unexpected stdin status while waiting for $label token"
			fi
			;;
		esac
	done
}

measure_continuous_usb_absence() {
	local attestation_ms="$1"
	local now_ms
	local elapsed_ms

	while true; do
		port_is_present && die "selected port reappeared before 5000 ms continuous absence"
		now_ms="$(monotonic_ms)"
		elapsed_ms=$((now_ms - attestation_ms))
		((elapsed_ms >= 0)) || die "monotonic clock moved backwards"
		if ((elapsed_ms >= minimum_usb_absence_ms)); then
			printf '%s\n' "$elapsed_ms"
			return
		fi
		wait_one_second
	done
}

wait_for_reappearance() {
	local attestation_ms="$1"
	local now_ms
	local elapsed_ms

	while true; do
		now_ms="$(monotonic_ms)"
		elapsed_ms=$((now_ms - attestation_ms))
		((elapsed_ms >= 0)) || die "monotonic clock moved backwards"
		if port_is_present; then
			((elapsed_ms <= reattach_deadline_ms)) || die "selected port reappeared after 60000 ms deadline"
			printf '%s\n' "$elapsed_ms"
			return
		fi
		((elapsed_ms < reattach_deadline_ms)) || die "selected port remained absent at 60000 ms deadline"
		wait_one_second
	done
}

run_monitored_capture() {
	local output_log="$1"
	local wrapper_log="$2"
	local child_status
	local group_start_status
	local -a args=(
		--port "$port"
		--out "$output_log"
		--seconds "$duration_seconds"
		--no-reset
	)
	local -a monitor_command
	active_descendant_group_file="$run_dir/descendant-monitor-process-group.state"
	: >"$active_descendant_group_file"
	if [[ -n "${PHASE28_ACCEPTED_STATE_MONITOR_BIN:-}" ]]; then
		monitor_command=(env PHASE13_MONITOR_GROUP_STATE_FILE="$active_descendant_group_file" "$PHASE28_ACCEPTED_STATE_MONITOR_BIN" "${args[@]}")
	else
		monitor_command=(env PHASE13_MONITOR_GROUP_STATE_FILE="$active_descendant_group_file" bash scripts/phase13-monitor-capture.sh "${args[@]}")
	fi

	trace_event "monitor-no-reset"
	set +e
	phase_process_group_start "$run_dir/monitor-process-group.ready" "${monitor_command[@]}" >"$wrapper_log" 2>&1
	group_start_status=$?
	set -e
	((group_start_status == 0)) || die "failed to start isolated monitor process group"
	active_child_pid="$PHASE_PROCESS_GROUP_PID"
	if [[ -n "${PHASE28_ACCEPTED_STATE_CHILD_PID_FILE:-}" ]]; then
		printf '%s\n' "$active_child_pid" >"$PHASE28_ACCEPTED_STATE_CHILD_PID_FILE"
	fi
	set +e
	wait "$active_child_pid"
	child_status=$?
	set -e
	cleanup_active_child || die "failed to clean monitor process group"
	((child_status == 0)) || die "no-reset monitor capture failed"
}

verify_source_tree_clean() {
	if [[ "${PHASE28_ACCEPTED_STATE_SKIP_SOURCE_CLEAN_CHECK:-0}" == "1" ]]; then
		return
	fi
	if [[ -n "$(git status --porcelain --untracked-files=no -- crates firmware scripts)" ]]; then
		die "firmware or evidence source tree is dirty"
	fi
}

current_source_commit() {
	git rev-parse HEAD
}

manifest_source_commit() {
	jq -er '.source_commit | select(type == "string" and length > 0)' "$1"
}

verify_manifest_identity() {
	local package_manifest="$1"
	local source_commit
	local package_source_commit
	[[ -f "$package_manifest" ]] || die "phase-correct package manifest is missing"
	source_commit="$(current_source_commit)"
	package_source_commit="$(manifest_source_commit "$package_manifest")" || die "package manifest source_commit is unavailable"
	[[ "$package_source_commit" == "$source_commit" ]] || die "package manifest source_commit does not match HEAD"
}

verify_detector_port() {
	local detector_log="$1"
	run_detector >"$detector_log" 2>&1
	local detected_port
	detected_port="$(sed -n 's/^port=//p' "$detector_log" | tail -1)"
	[[ -n "$detected_port" ]] || die "detect-ultra205 did not report one port"
	[[ "$detected_port" == "$port" ]] || die "detected port does not match --port"
}

count_matches() {
	local pattern="$1"
	local raw_log="$2"
	local count
	local search_status
	set +e
	count="$(rg -c "$pattern" "$raw_log")"
	search_status=$?
	set -e
	case "$search_status" in
	0) printf '%s\n' "$count" ;;
	1) printf '0\n' ;;
	*) die "runtime marker search failed" ;;
	esac
}

verify_stable_runtime() {
	local raw_log="$1"
	local member_label="$2"
	local boot_count
	local listener_count
	local hazard_status
	set +e
	rg -q -i 'stack overflow|stack canary|guru meditation|panic(ed)?|abort\(\)|SW_CPU_RESET|RTC_SW_(SYS|CPU)_RST|software reset' "$raw_log"
	hazard_status=$?
	set -e
	case "$hazard_status" in
	0) die "$member_label capture contains a stack-overflow, panic, or software-reset marker" ;;
	1) ;;
	*) die "$member_label hazard search failed" ;;
	esac
	boot_count="$(count_matches 'bitaxe-rust boot' "$raw_log")"
	listener_count="$(count_matches 'h4_continuous_result=listener_armed' "$raw_log")"
	[[ "$boot_count" == "1" ]] || die "$member_label capture does not contain exactly one stable boot"
	[[ "$listener_count" == "1" ]] || die "$member_label capture does not contain exactly one listener-ready marker"
}

verify_complete_reinit_member() {
	local raw_log="$1"
	local comparison_out="$2"
	verify_stable_runtime "$raw_log" "reinit"
	node scripts/phase28.1.1-accepted-state-lifecycle-compare.mjs \
		--reinit-log "$raw_log" \
		--cold-start-log "$raw_log" \
		--out "$comparison_out"
	if ! rg -q '^lifecycle_status: match$' "$comparison_out"; then
		die "reinit self-comparison did not match"
	fi
}

write_common_evidence_header() {
	local capture_status="$1"
	printf 'mode: %s\n' "$mode"
	printf 'attempt: %s\n' "$attempt"
	printf 'board: %s\n' "$board"
	printf 'selected_port: detector-gated-redacted\n'
	printf 'capture_status: %s\n' "$capture_status"
	printf 'duration_seconds: %s\n' "$duration_seconds"
	printf 'credential_contents_read: false\n'
	printf 'raw_register_values_promoted: false\n'
	printf 'redacted: true\n'
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

sha256_file() {
	shasum -a 256 "$1" | awk '{print $1}'
}

sha256_text() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

private_write() {
	local destination="$1"
	local contents="$2"
	local temporary
	temporary="$(mktemp "$(dirname "$destination")/.plan13-adapter.XXXXXX")"
	printf '%s\n' "$contents" >"$temporary"
	chmod 600 "$temporary"
	mv -f "$temporary" "$destination"
}

adapter_state_path() {
	printf '%s/state.json\n' "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}"
}

assert_adapter_boot_and_head() {
	local state_path
	state_path="$(adapter_state_path)"
	if [[ "${PHASE28_ADAPTER_TEST_MODE:-0}" == "1" && -n "${PHASE28_ADAPTER_TEST_BOOT_RAW:-}" ]]; then
		local platform="${PHASE28_ADAPTER_TEST_BOOT_PLATFORM:-linux}"
		local raw="${PHASE28_ADAPTER_TEST_BOOT_RAW:-11111111-1111-1111-1111-111111111111}"
		node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const platform = process.argv[3];
const raw = process.argv[4];
const observed = platform === "darwin" ? authority.deriveMacBootSessionDigest(raw) : authority.deriveLinuxBootSessionDigest(raw);
authority.validateAttemptState(state);
authority.assertFreshBootSession(state, () => observed);' \
			"$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs" "$state_path" "$platform" "$raw"
		return
	fi
	if [[ "${PHASE28_ADAPTER_TEST_MODE:-0}" == "1" ]]; then
		node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
authority.validateAttemptState(state);
authority.assertFreshBootSession(state);' \
			"$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs" "$state_path"
		return
	fi
	[[ -z "$(git -C "$repo_root" status --porcelain=v1)" ]] || die "plan13 adapter requires a clean exact HEAD"
	[[ "$(git -C "$repo_root" rev-parse HEAD)" == "$(jq -er '.exact_head' "$state_path")" ]] || die "plan13 adapter exact HEAD mismatch"
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
authority.validateAttemptState(state);
authority.assertFreshBootSession(state);' \
		"$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs" "$state_path"
}

validate_adapter_path() {
	local path="$1"
	local attempt_dir="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}"
	[[ "$path" == "$attempt_dir/"* && "$(dirname "$path")" == "$attempt_dir" ]] || die "plan13 adapter private path mismatch"
}

validate_runner_adapter_context() {
	local attempt_dir="${PHASE28_LIFECYCLE_ATTEMPT_DIR:-}"
	local state_path
	[[ -n "$attempt_dir" && -d "$attempt_dir" ]] || die "plan13-prevalidated requires runner-owned attempt state"
	[[ "$(mode_of_path "$attempt_dir")" == "700" ]] || die "plan13 attempt directory is not private"
	state_path="$attempt_dir/state.json"
	[[ -f "$state_path" && "$(mode_of_path "$state_path")" == "600" ]] || die "plan13 state is not private"
	[[ "$(owner_of_path "$attempt_dir")" == "$(id -u)" && "$(owner_of_path "$state_path")" == "$(id -u)" ]] || die "plan13 state owner mismatch"
	if [[ "${PHASE28_ADAPTER_UNIT_FIXTURE:-0}" != "1" ]]; then
		local control_root="${PHASE28_ATTEMPT_CONTROL_ROOT:-$repo_root/hardware-runs/phase28.1.1/attempt-control}"
		local resume_digest
		local active_slot
		resume_digest="$(jq -er '.resume_handle_sha256' "$state_path")"
		active_slot="$control_root/resume-index/$resume_digest.json"
		[[ -f "$active_slot" && "$(mode_of_path "$active_slot")" == "600" && "$(owner_of_path "$active_slot")" == "$(id -u)" ]] || die "plan13 active mapping is unavailable"
		jq -e \
			--arg attempt_dir "$attempt_dir" \
			--arg digest "$resume_digest" \
			--arg attempt_id "$(jq -er '.attempt_id' "$state_path")" \
			--arg boot "$(jq -er '.boot_session_sha256' "$state_path")" \
			--argjson checkpoint_generation "$(jq -er '.checkpoint_generation' "$state_path")" '
			  type == "object" and
			  (keys | sort) == (["attempt_dir","attempt_generation","attempt_id","boot_session_sha256","checkpoint_generation","resume_handle_sha256","schema_version","status"] | sort) and
			  .schema_version == "exact-head-resume-active-v1" and
			  .status == "active" and
			  .attempt_dir == $attempt_dir and
			  .resume_handle_sha256 == $digest and
			  .attempt_id == $attempt_id and
			  .boot_session_sha256 == $boot and
			  .checkpoint_generation == $checkpoint_generation
			' "$active_slot" >/dev/null || die "plan13 active mapping mismatch"
	else
		[[ "${PHASE28_ADAPTER_TEST_MODE:-0}" == "1" ]] || die "adapter unit fixture requires test mode"
	fi
	[[ "${PHASE28_EFFECT_ID:-}" == "$effect_id" ]] || die "plan13 effect identity mismatch"
	validate_adapter_path "${PHASE28_EFFECT_ACK_FILE:-}"
	validate_adapter_path "${PHASE28_EFFECT_GATE_FILE:-}"
	validate_adapter_path "${PHASE28_EFFECT_RESULT_FILE:-}"
	[[ "$(jq -er '.effect_id' "$state_path")" == "$effect_id" ]] || die "plan13 effect is not authorized"
	[[ "$(jq -er '.effect_phase' "$state_path")" == "authorized" ]] || die "plan13 effect is not at the authorization boundary"
	assert_adapter_boot_and_head
	: >"$PHASE28_EFFECT_ACK_FILE"
	chmod 600 "$PHASE28_EFFECT_ACK_FILE"
	for _ in $(seq 1 500); do
		[[ -f "$PHASE28_EFFECT_GATE_FILE" ]] && break
		sleep 0.01
	done
	[[ -f "$PHASE28_EFFECT_GATE_FILE" ]] || die "plan13 effect start gate was not released"
	[[ "$(jq -er '.effect_id' "$state_path")" == "$effect_id" && "$(jq -er '.effect_phase' "$state_path")" == "invoked" ]] || die "plan13 effect invocation was not persisted"
	assert_adapter_boot_and_head
	trace_event "plan13-prevalidated:${effect_id}"
}

write_effect_result() {
	local status="$1"
	local blocker="$2"
	local outputs="$3"
	private_write "$PHASE28_EFFECT_RESULT_FILE" "$(jq -cn --arg effect "$effect_id" --arg status "$status" --arg blocker "$blocker" --argjson outputs "$outputs" '{schema_version:"exact-head-effect-result-v1",effect_id:$effect,status:$status,blocker_reason:$blocker,outputs:$outputs}')"
}

require_test_override_mode() {
	local name
	for name in PHASE28_ACCEPTED_STATE_DETECT_BIN PHASE28_ACCEPTED_STATE_VERIFY_BIN PHASE28_ACCEPTED_STATE_PACKAGE_BIN PHASE28_ACCEPTED_STATE_CAPTURE_BIN PHASE28_ACCEPTED_STATE_MONITOR_BIN PHASE28_ACCEPTED_STATE_PORT_PRESENT_BIN PHASE28_ACCEPTED_STATE_SLEEP_BIN PHASE28_ACCEPTED_STATE_CLOCK_BIN; do
		if [[ -n "${!name:-}" && "${PHASE28_ADAPTER_TEST_MODE:-0}" != "1" ]]; then
			die "plan13 adapter test override rejected"
		fi
	done
}

selected_port_file() {
	printf '%s/selected-port.value\n' "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}"
}

capability_file() {
	printf '%s/credential-capability.json\n' "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}"
}

manifest_path_file() {
	printf '%s/package-manifest.path\n' "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}"
}

reinit_log_path_file() {
	printf '%s/reinit-log.path\n' "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}"
}

read_private_line() {
	local path="$1"
	[[ -f "$path" && "$(mode_of_path "$path")" == "600" ]] || die "plan13 private capability is unavailable"
	sed -n '1p' "$path"
}

is_private_owned_file() {
	local path="$1"
	[[ -f "$path" ]] || return 1
	[[ "$(mode_of_path "$path")" == "600" ]] || return 1
	[[ "$(owner_of_path "$path")" == "$(id -u)" ]]
}

parse_one_detector_port() {
	local detector_log="$1"
	local -a detected_ports=()
	while IFS= read -r candidate; do
		[[ -n "$candidate" ]] && detected_ports+=("$candidate")
	done < <(sed -n 's/^port=//p' "$detector_log")
	[[ "${#detected_ports[@]}" == "1" ]] || return 1
	printf '%s\n' "${detected_ports[0]}"
}

run_prevalidated_detector() {
	local detector_log="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/detector.raw.log"
	if ! run_detector >"$detector_log" 2>&1; then
		write_effect_result failed detector_failed '{}'
		return 1
	fi
	local detected_port
	detected_port="$(parse_one_detector_port "$detector_log")" || {
		write_effect_result failed detector_failed '{}'
		return 1
	}
	private_write "$(selected_port_file)" "$detected_port"
	write_effect_result completed none "$(jq -cn --arg digest "$(sha256_text "plan13-port-v1\0${detected_port}")" '{selected_port_fingerprint_sha256:$digest}')"
}

run_prevalidated_credential_bind() {
	local wifi_path="$repo_root/wifi-credentials.json"
	if ! is_private_owned_file "$wifi_path"; then
		write_effect_result failed credential_binding_failed '{}'
		return 1
	fi
	local -a pool_paths=()
	local candidate
	for candidate in "$repo_root"/pool-credentials*.json; do
		if [[ "$candidate" != *.example ]] && is_private_owned_file "$candidate"; then
			pool_paths+=("$candidate")
		fi
	done
	[[ "${#pool_paths[@]}" == "1" ]] || {
		write_effect_result failed credential_binding_failed '{}'
		return 1
	}
	local wifi_binding
	local pool_binding
	wifi_binding="$(openssl rand -hex 16)"
	pool_binding="$(openssl rand -hex 16)"
	local capability
	capability="$(jq -cn --arg wifi "$wifi_path" --arg pool "${pool_paths[0]}" --arg wifi_binding "$wifi_binding" --arg pool_binding "$pool_binding" '{schema_version:"plan13-credential-capability-v1",wifi_path:$wifi,pool_path:$pool,wifi_binding_id:$wifi_binding,pool_binding_id:$pool_binding}')"
	private_write "$(capability_file)" "$capability"
	write_effect_result completed none "$(jq -cn --arg wifi_binding "$wifi_binding" --arg pool_binding "$pool_binding" --arg digest "$(sha256_file "$(capability_file)")" '{wifi_credential_state:"present",pool_credential_state:"present",wifi_credential_binding_id:$wifi_binding,pool_credential_binding_id:$pool_binding,credential_capability_status:"sealed",credential_capability_sha256:$digest}')"
}

run_prevalidated_reference_guard() {
	local reference_log="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/reference-guard.raw.log"
	if ! run_verify_reference >"$reference_log" 2>&1; then
		write_effect_result failed reference_guard_failed '{}'
		return 1
	fi
	local reference_commit
	reference_commit="$(git -C "$repo_root/reference/esp-miner" rev-parse HEAD)"
	write_effect_result completed none "$(jq -cn --arg commit "$reference_commit" --arg digest "$(sha256_file "$reference_log")" '{reference_commit:$commit,reference_guard_output_sha256:$digest}')"
}

factory_image_path() {
	local package_manifest="$1"
	local relative
	relative="$(jq -er '.artifacts[] | select(.kind == "factory_merged_image") | .path' "$package_manifest")"
	if [[ "$relative" == /* ]]; then
		printf '%s\n' "$relative"
	else
		printf '%s/%s\n' "$(dirname "$package_manifest")" "$relative"
	fi
}

run_prevalidated_package() {
	local package_log="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/package.raw.log"
	if ! run_package >"$package_log" 2>&1; then
		write_effect_result failed package_failed '{}'
		return 1
	fi
	local package_manifest="${PHASE28_ACCEPTED_STATE_MANIFEST:-$repo_root/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json}"
	verify_manifest_identity "$package_manifest"
	local factory_image
	factory_image="$(factory_image_path "$package_manifest")"
	[[ -f "$factory_image" ]] || {
		write_effect_result failed package_failed '{}'
		return 1
	}
	private_write "$(manifest_path_file)" "$package_manifest"
	write_effect_result completed none "$(jq -cn --arg head "$(current_source_commit)" --arg manifest "$(sha256_file "$package_manifest")" --arg image "$(sha256_file "$factory_image")" '{manifest_source_commit:$head,manifest_sha256:$manifest,factory_image_sha256:$image}')"
}

validate_credential_capability() {
	local capability_path="$1"
	local state_path
	local capability
	local wifi_path
	local pool_path
	state_path="$(adapter_state_path)"
	is_private_owned_file "$capability_path" || return 1
	jq -e '
	  if type != "object" then false
	  elif .schema_version != "exact-head-attempt-v2" then false
	  elif .execution_plan != 13 then false
	  elif .credential_capability_status != "sealed" then false
	  elif .wifi_credential_state != "present" then false
	  elif .pool_credential_state != "present" then false
	  else true
	  end
	' "$state_path" >/dev/null || return 1
	[[ "$(sha256_file "$capability_path")" == "$(jq -er '.credential_capability_sha256' "$state_path")" ]] || return 1
	capability="$(jq -ce \
		--arg wifi_binding "$(jq -er '.wifi_credential_binding_id' "$state_path")" \
		--arg pool_binding "$(jq -er '.pool_credential_binding_id' "$state_path")" '
		  if type != "object" then empty
		  elif (keys | sort) != (["pool_binding_id","pool_path","schema_version","wifi_binding_id","wifi_path"] | sort) then empty
		  elif .schema_version != "plan13-credential-capability-v1" then empty
		  elif .wifi_binding_id != $wifi_binding then empty
		  elif .pool_binding_id != $pool_binding then empty
		  elif (.wifi_path | type) != "string" then empty
		  elif (.pool_path | type) != "string" then empty
		  elif (.wifi_path | startswith("/")) != true then empty
		  elif (.pool_path | startswith("/")) != true then empty
		  else .
		  end
		' "$capability_path" 2>/dev/null)" || return 1
	wifi_path="$(jq -er '.wifi_path' <<<"$capability")" || return 1
	pool_path="$(jq -er '.pool_path' <<<"$capability")" || return 1
	is_private_owned_file "$wifi_path" || return 1
	is_private_owned_file "$pool_path" || return 1
	printf '%s\n' "$capability"
}

run_prevalidated_flash_reinit() {
	local capability
	local capability_path
	capability_path="$(capability_file)"
	if ! capability="$(validate_credential_capability "$capability_path")"; then
		write_effect_result failed private_capability_invalid '{}'
		return 1
	fi
	port="$(read_private_line "$(selected_port_file)")"
	manifest="$(read_private_line "$(manifest_path_file)")"
	wifi_credentials="$(jq -er '.wifi_path' <<<"$capability")"
	pool_credentials="$(jq -er '.pool_path' <<<"$capability")"
	duration_seconds=360
	local started_ms
	local ended_ms
	started_ms="$(monotonic_ms)"
	if ! run_flash_capture "$manifest" >"${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/flash-reinit.raw.log" 2>&1; then
		rm -f "$capability_path"
		write_effect_result failed reinit_capture_failed '{}'
		return 1
	fi
	ended_ms="$(monotonic_ms)"
	local raw_reinit_log="$run_dir/hardware/live-capture-runtime/flash-monitor.log"
	[[ -f "$raw_reinit_log" ]] || {
		rm -f "$capability_path"
		write_effect_result failed reinit_capture_failed '{}'
		return 1
	}
	local comparison="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/reinit-self-compare.redacted.txt"
	verify_complete_reinit_member "$raw_reinit_log" "$comparison"
	private_write "$(reinit_log_path_file)" "$raw_reinit_log"
	rm -f "$capability_path"
	local duration_ms=$((ended_ms - started_ms))
	((duration_ms >= 360000)) || {
		write_effect_result failed reinit_capture_short '{}'
		return 1
	}
	write_effect_result completed none "$(jq -cn --argjson start "$started_ms" --argjson end "$ended_ms" --argjson duration "$duration_ms" --arg raw "$(sha256_file "$raw_reinit_log")" --arg classifier "$(sha256_file "$comparison")" '{runtime_credential_consumption:"pass",credential_capability_status:"destroyed",credential_capability_sha256:null,reinit_capture_started_ms:$start,reinit_capture_ended_ms:$end,reinit_capture_duration_ms:$duration,reinit_capture_category:"complete_360s",reinit_raw_log_sha256:$raw,reinit_classifier_input_sha256:$classifier,reinit_five_stage_result:"pass"}')"
}

start_lifecycle_socket_receiver() {
	local frame_path="$1"
	local socket_path="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/lifecycle.sock"
	rm -f "$socket_path" "$frame_path"
	(
		cd "$PHASE28_LIFECYCLE_ATTEMPT_DIR"
		perl -MIO::Socket::UNIX -MSocket -e '
my ($frame_path) = @ARGV;
my $server = IO::Socket::UNIX->new(Type => SOCK_STREAM, Local => "lifecycle.sock", Listen => 1) or die $!;
chmod 0600, "lifecycle.sock" or die $!;
my $client = $server->accept or die $!;
my $length_line = <$client>;
defined $length_line && $length_line =~ /^([0-9]+)\n$/ && $1 <= 4096 or die "invalid frame length";
my $remaining = 0 + $1;
my $frame = q{};
while ($remaining > 0) {
  my $read = sysread($client, my $chunk, $remaining);
  defined $read && $read > 0 or die "short frame";
  $frame .= $chunk;
  $remaining -= $read;
}
open my $out, ">", $frame_path or die $!;
chmod 0600, $frame_path or die $!;
print {$out} $frame or die $!;
close $out or die $!;
' "$frame_path"
	) &
	lifecycle_socket_pid=$!
	for _ in $(seq 1 100); do
		[[ -S "$socket_path" ]] && return
		kill -0 "$lifecycle_socket_pid" 2>/dev/null || break
		sleep 0.01
	done
	die "lifecycle private socket failed to start"
}

validate_lifecycle_frame() {
	local frame_path="$1"
	local expected_generation="$2"
	local expected_token="$3"
	local expected_response="$4"
	local deadline_ms="$5"
	local expected_attempt_state="$6"
	local expected_lifecycle_substate="$7"
	local state_path
	state_path="$(adapter_state_path)"
	assert_adapter_boot_and_head
	(($(monotonic_ms) < deadline_ms)) || die "lifecycle checkpoint expired"
	[[ "$(jq -er '.attempt_state' "$state_path")" == "$expected_attempt_state" ]] || die "lifecycle attempt state mismatch"
	[[ "$(jq -er '.lifecycle_substate' "$state_path")" == "$expected_lifecycle_substate" ]] || die "lifecycle substate mismatch"
	[[ "$(jq -er '.process_running' "$state_path")" == "true" ]] || die "lifecycle owner is not running"
	[[ "$(jq -er '.lifecycle_lease_id' "$state_path")" == "${PHASE28_LIFECYCLE_LEASE_ID:?}" ]] || die "lifecycle lease mismatch"
	[[ "$(jq -er '.lifecycle_capability_sha256' "$state_path")" == "$(sha256_text "${PHASE28_LIFECYCLE_CAPABILITY:?}")" ]] || die "lifecycle capability mismatch"
	[[ "$(jq -er '.lifecycle_owner_pid' "$state_path")" == "$$" ]] || die "lifecycle owner PID mismatch"
	jq -e \
		--arg digest "$(jq -er '.resume_handle_sha256' "$state_path")" \
		--argjson generation "$expected_generation" \
		--arg token "$expected_token" \
		--arg response "$expected_response" \
		--arg nonce "$(jq -er '.effect_authorization_nonce' "$state_path")" \
		--arg lease "$PHASE28_LIFECYCLE_LEASE_ID" '
      (keys | sort) == (["checkpoint_generation","checkpoint_token","effect_authorization_nonce","lifecycle_lease_id","response_token","resume_handle_sha256"] | sort) and
      .resume_handle_sha256 == $digest and
      .checkpoint_generation == $generation and
      .checkpoint_token == $token and
      .response_token == $response and
      .effect_authorization_nonce == $nonce and
      .lifecycle_lease_id == $lease
    ' "$frame_path" >/dev/null || die "lifecycle frame validation failed"
}

receive_lifecycle_token() {
	local expected_token="$1"
	local expected_response="$2"
	local expected_attempt_state="$3"
	local expected_lifecycle_substate="$4"
	local state_path
	local generation
	local deadline
	local frame_path="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/lifecycle-frame.json"
	state_path="$(adapter_state_path)"
	generation="$(jq -er '.checkpoint_generation' "$state_path")"
	deadline="$(jq -er '.monotonic_deadline_ms' "$state_path")"
	start_lifecycle_socket_receiver "$frame_path"
	while kill -0 "$lifecycle_socket_pid" 2>/dev/null; do
		if (($(monotonic_ms) >= deadline)); then
			kill -TERM "$lifecycle_socket_pid" 2>/dev/null || true
			wait "$lifecycle_socket_pid" 2>/dev/null || true
			die "lifecycle checkpoint expired"
		fi
		sleep 0.05
	done
	wait "$lifecycle_socket_pid" || die "lifecycle private socket failed"
	validate_lifecycle_frame "$frame_path" "$generation" "$expected_token" "$expected_response" "$deadline" "$expected_attempt_state" "$expected_lifecycle_substate"
	rm -f "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/lifecycle.sock" "$frame_path"
}

lifecycle_transition() {
	local event="$1"
	local values="$2"
	local values_path="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/lifecycle-values.json"
	private_write "$values_path" "$values"
	bash "${PHASE28_LIFECYCLE_RUNNER:?}" lifecycle-owner-transition \
		--capability "${PHASE28_LIFECYCLE_CAPABILITY:?}" \
		--event "$event" \
		--values-file "$values_path" >/dev/null
	rm -f "$values_path"
}

run_prevalidated_lifecycle() {
	port="$(read_private_line "$(selected_port_file)")"
	manifest="$(read_private_line "$(manifest_path_file)")"
	reinit_log="$(read_private_line "$(reinit_log_path_file)")"
	duration_seconds=360
	verify_manifest_identity "$manifest"
	verify_complete_reinit_member "$reinit_log" "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/reinit-preflight.redacted.txt"
	port_is_present || {
		write_effect_result failed lifecycle_capture_failed '{}'
		return 1
	}

	receive_lifecycle_token plan13-armed-removal-v1 plan13-both-power-paths-removed removal_attested removal_attested
	local attestation_ms
	attestation_ms="$(jq -er '.attestation_accepted_ms' "$(adapter_state_path)")"
	lifecycle_transition absence-observing "$(jq -cn --argjson start "$attestation_ms" '{usb_absence_started_ms:$start}')"
	local usb_absence_measured_ms
	usb_absence_measured_ms="$(measure_continuous_usb_absence "$attestation_ms")"
	local absence_end=$((attestation_ms + usb_absence_measured_ms))
	lifecycle_transition restore-waiting "$(jq -cn --argjson end "$absence_end" --argjson duration "$usb_absence_measured_ms" '{usb_absence_ended_ms:$end,usb_absence_ms:$duration}')"
	receive_lifecycle_token plan13-barrel-usb-restore-v1 plan13-barrel-then-usb-restored restore_attested restore_attested
	lifecycle_transition reappearance-observing '{}'
	local reappearance_elapsed_ms
	reappearance_elapsed_ms="$(wait_for_reappearance "$attestation_ms")"
	local capture_started_ms
	capture_started_ms="$(monotonic_ms)"
	lifecycle_transition capture-running "$(jq -cn --argjson reappearance "$((attestation_ms + reappearance_elapsed_ms))" --argjson elapsed "$reappearance_elapsed_ms" --argjson start "$capture_started_ms" '{usb_reappearance_ms:$reappearance,reappearance_elapsed_ms:$elapsed,capture_started_ms:$start}')"

	local cold_start_raw_log="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/cold-start-monitor.raw.log"
	run_monitored_capture "$cold_start_raw_log" "${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/monitor-wrapper.raw.log"
	local capture_ended_ms
	capture_ended_ms="$(monotonic_ms)"
	local capture_duration_ms=$((capture_ended_ms - capture_started_ms))
	((capture_duration_ms >= 360000)) || {
		write_effect_result failed lifecycle_capture_short '{}'
		return 1
	}
	verify_stable_runtime "$cold_start_raw_log" cold-start
	local comparison="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/lifecycle.redacted.txt"
	node scripts/phase28.1.1-accepted-state-lifecycle-compare.mjs --reinit-log "$reinit_log" --cold-start-log "$cold_start_raw_log" --out "$comparison"
	local lifecycle_status
	lifecycle_status="$(sed -n 's/^lifecycle_status: //p' "$comparison")"
	case "$lifecycle_status" in match | mismatch | incomplete) ;; *) lifecycle_status=incomplete ;; esac
	local result_correlated=false
	local power_delta_class=unavailable
	local share_submission_status=not_observed
	rg -q 'result_correlated=true' "$cold_start_raw_log" && result_correlated=true
	if rg -q 'power_delta_class=rising_hashing' "$cold_start_raw_log"; then
		power_delta_class=rising_hashing
	elif rg -q 'power_delta_class=flat' "$cold_start_raw_log"; then
		power_delta_class=flat
	fi
	if rg -q 'share_submission_status=accepted|share_outcome: accepted' "$cold_start_raw_log"; then
		share_submission_status=accepted
	elif rg -q 'share_submission_status=rejected|share_outcome: rejected' "$cold_start_raw_log"; then
		share_submission_status=rejected
	fi
	local raw_digest
	local same_chain_digest
	raw_digest="$(sha256_file "$cold_start_raw_log")"
	same_chain_digest="$(sha256_text "$(sha256_file "$reinit_log")\0${raw_digest}")"
	lifecycle_transition capture-complete "$(jq -cn --argjson end "$capture_ended_ms" --argjson duration "$capture_duration_ms" --arg raw "$raw_digest" --arg chain "$same_chain_digest" --arg classifier "$(sha256_file "$comparison")" --arg lifecycle "$lifecycle_status" --argjson correlated "$result_correlated" --arg power "$power_delta_class" --arg share "$share_submission_status" '{capture_ended_ms:$end,capture_duration_ms:$duration,lifecycle_raw_log_sha256:$raw,same_chain_raw_log_set_sha256:$chain,classifier_input_sha256:$classifier,lifecycle_status:$lifecycle,result_correlated:$correlated,power_delta_class:$power,share_submission_status:$share}')"
	write_effect_result completed none '{}'
}

run_prevalidated_post_capture() {
	local detector_log="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/post-capture-detector.raw.log"
	if ! run_detector >"$detector_log" 2>&1; then
		write_effect_result failed post_capture_detector_failed '{}'
		return 1
	fi
	local detected_port
	detected_port="$(parse_one_detector_port "$detector_log")" || {
		write_effect_result failed post_capture_detector_failed '{}'
		return 1
	}
	[[ "$(sha256_text "plan13-port-v1\0${detected_port}")" == "$(jq -er '.selected_port_fingerprint_sha256' "$(adapter_state_path)")" ]] || {
		write_effect_result failed post_capture_detector_failed '{}'
		return 1
	}
	local classifier_output
	classifier_output="$(node --input-type=module -e '
import fs from "node:fs";
const strict = await import(process.argv[1]);
const state = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const projection = strict.buildClassifierProjection(state);
const result = strict.classifyStrictProductionEvidence(projection);
process.stdout.write(JSON.stringify({projection,result}));' "$repo_root/scripts/phase28.1.1-strict-production-evidence.mjs" "$(adapter_state_path)")"
	write_effect_result completed none "$(jq -cn --argjson classified "$classifier_output" '{result_correlated:$classified.projection.result_correlated,power_delta_class:$classified.projection.power_delta_class,share_submission_status:$classified.projection.share_submission_status,lifecycle_status:$classified.projection.lifecycle_status,classifier_input_sha256:$classified.projection.classifier_input_sha256,classifier_output_sha256:$classified.result.classifier_output_sha256,classifier_version:$classified.projection.classifier_version}')"
}

run_plan13_prevalidated() {
	require_test_override_mode
	validate_runner_adapter_context
	run_root="${PHASE28_LIFECYCLE_ATTEMPT_DIR:?}/effects"
	run_dir="$run_root/$effect_id"
	mkdir -p "$run_root" "$run_dir"
	chmod 700 "$run_root" "$run_dir"
	case "$effect_id" in
	detector_board_info) run_prevalidated_detector ;;
	credential_presence_bind) run_prevalidated_credential_bind ;;
	reference_guard) run_prevalidated_reference_guard ;;
	package) run_prevalidated_package ;;
	flash_reinit_runtime) run_prevalidated_flash_reinit ;;
	lifecycle_start) run_prevalidated_lifecycle ;;
	post_capture_detector_board_info) run_prevalidated_post_capture ;;
	esac
}

cd "$repo_root"

if [[ "$mode" == "plan13-prevalidated" ]]; then
	run_plan13_prevalidated
	exit $?
fi

if [[ "$mode" == "blocked" ]]; then
	if [[ "$attempt" == "lifecycle" ]]; then
		node scripts/phase28.1.1-accepted-state-lifecycle-compare.mjs \
			--unavailable \
			--out "$run_dir/lifecycle.redacted.txt"
		{
			write_common_evidence_header "blocked_safe_prerequisite"
			cat "$run_dir/lifecycle.redacted.txt"
		} >"$evidence_out"
	else
		write_unavailable_markers >"$run_dir/upstream-markers.redacted.log"
		write_unavailable_markers >"$run_dir/rust-markers.redacted.log"
		node scripts/phase28.1.1-accepted-state-compare.mjs \
			--upstream-log "$run_dir/upstream-markers.redacted.log" \
			--rust-log "$run_dir/rust-markers.redacted.log" \
			--out "$run_dir/comparison.redacted.txt"
		{
			write_common_evidence_header "blocked_safe_prerequisite"
			cat "$run_dir/comparison.redacted.txt"
		} >"$evidence_out"
	fi
	printf 'accepted_state_diagnostic_status=blocked_safe_prerequisite\n'
	printf 'accepted_state_diagnostic_redacted=true\n'
	exit 0
fi

[[ -n "$port" ]] || die "--port is required in hardware mode"
[[ -f "$wifi_credentials" ]] || die "Wi-Fi credential file is missing"
[[ -f "$pool_credentials" ]] || die "pool credential file is missing"
trace_event "credential-paths-checked"
verify_source_tree_clean

run_verify_reference >"$run_dir/verify-reference.raw.log" 2>&1
trace_event "reference-verified"
verify_detector_port "$run_dir/detect-ultra205-preflight.raw.log"
trace_event "preflight-detector"

if [[ "$attempt" == "accepted-state" ]]; then
	run_package >"$run_dir/package.raw.log" 2>&1
	manifest="${manifest:-${PHASE28_ACCEPTED_STATE_MANIFEST:-$repo_root/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json}}"
	verify_manifest_identity "$manifest"
	run_flash_capture "$manifest" >"$run_dir/capture.raw.log" 2>&1
	raw_reinit_log="$run_dir/hardware/live-capture-runtime/flash-monitor.log"
	[[ -f "$raw_reinit_log" ]] || die "reinit capture log is missing"
	verify_complete_reinit_member "$raw_reinit_log" "$run_dir/reinit-self-compare.redacted.txt"
	upstream_raw_log="${PHASE28_ACCEPTED_STATE_UPSTREAM_LOG:-$repo_root/scratch/upstream-wire-capture/captures/upstream-monitor.raw.log}"
	normalize_markers "$upstream_raw_log" "$run_dir/upstream-markers.redacted.log"
	normalize_markers "$raw_reinit_log" "$run_dir/rust-markers.redacted.log"
	node scripts/phase28.1.1-accepted-state-compare.mjs \
		--upstream-log "$run_dir/upstream-markers.redacted.log" \
		--rust-log "$run_dir/rust-markers.redacted.log" \
		--out "$run_dir/comparison.redacted.txt"
	{
		write_common_evidence_header "complete"
		printf 'source_commit: %s\n' "$(current_source_commit)"
		printf 'manifest_source_commit_match: true\n'
		cat "$run_dir/comparison.redacted.txt"
	} >"$evidence_out"
	printf 'accepted_state_manifest=%s\n' "$manifest"
	printf 'accepted_state_reinit_log=%s\n' "$raw_reinit_log"
	printf 'accepted_state_source_commit=%s\n' "$(current_source_commit)"
	printf 'accepted_state_diagnostic_status=complete\n'
	printf 'accepted_state_diagnostic_redacted=true\n'
	exit 0
fi

manifest="${manifest:-${PHASE28_ACCEPTED_STATE_MANIFEST:-}}"
reinit_log="${reinit_log:-${PHASE28_ACCEPTED_STATE_REINIT_LOG:-}}"
[[ -n "$manifest" ]] || die "--manifest is required for lifecycle hardware mode"
[[ -n "$reinit_log" && -f "$reinit_log" ]] || die "--reinit-log is required for lifecycle hardware mode"
verify_manifest_identity "$manifest"
verify_complete_reinit_member "$reinit_log" "$run_dir/reinit-preflight.redacted.txt"
port_is_present || die "selected port is not present before lifecycle arming"

cold_start_raw_log="$run_dir/cold-start-monitor.raw.log"
trace_event "armed"
printf 'accepted_state_lifecycle_checkpoint=armed\n'
printf 'accepted_state_lifecycle_arming_state=armed_waiting_for_attestation\n'
printf 'accepted_state_lifecycle_action=disconnect-both-barrel-and-usb\n'
printf 'accepted_state_lifecycle_expected_token=both-power-paths-removed\n'

armed_ms="$(monotonic_ms)"
attestation_deadline_ms=$((armed_ms + attestation_timeout_seconds * 1000))
printf 'accepted_state_attestation_deadline_ms=%s\n' "$attestation_deadline_ms"
read_closed_token_until "both-power-paths-removed" "$attestation_deadline_ms" "both-power attestation"
attestation_ms="$(monotonic_ms)"
port_is_present && die "selected port is present at both-power attestation"
trace_event "operator-attestation-accepted"
printf 'operator_attested_both_power_paths_removed=true\n'

usb_absence_measured_ms="$(measure_continuous_usb_absence "$attestation_ms")"
trace_event "usb-absence-confirmed"
printf 'accepted_state_usb_absence_confirmed=true\n'
printf 'accepted_state_usb_absence_measured_ms=%s\n' "$usb_absence_measured_ms"
printf 'accepted_state_lifecycle_checkpoint=restore\n'
printf 'accepted_state_lifecycle_arming_state=absence_confirmed_waiting_for_restore\n'
printf 'accepted_state_lifecycle_action=restore-barrel-then-usb\n'
printf 'accepted_state_lifecycle_expected_token=barrel-then-usb-restored\n'

restore_deadline_ms=$((attestation_ms + reattach_timeout_seconds * 1000))
printf 'accepted_state_restore_deadline_ms=%s\n' "$restore_deadline_ms"
read_closed_token_until "barrel-then-usb-restored" "$restore_deadline_ms" "barrel-then-usb restore"
trace_event "restore-token-accepted"
reappearance_elapsed_ms="$(wait_for_reappearance "$attestation_ms")"
monitor_start_ms="$(monotonic_ms)"
monitor_start_elapsed_ms=$((monitor_start_ms - attestation_ms))
((monitor_start_elapsed_ms <= reappearance_elapsed_ms + monitor_start_reserve_ms)) || die "monitor startup exceeded 10000 ms reserve"
next_replay_after_monitor_ms=$((monitor_start_elapsed_ms + replay_interval_ms))
((reattach_deadline_ms + monitor_start_reserve_ms + replay_interval_ms == latest_safe_replay_ms)) || die "lifecycle timing contract drifted from 72000 ms"
((latest_safe_replay_ms < replay_window_ms)) || die "latest safe replay does not precede 180000 ms expiry"
((next_replay_after_monitor_ms < replay_window_ms)) || die "monitor startup left no replay before 180000 ms expiry"

run_monitored_capture "$cold_start_raw_log" "$run_dir/monitor-wrapper.raw.log"
trace_event "capture-complete"

verify_stable_runtime "$cold_start_raw_log" "cold-start"
node scripts/phase28.1.1-accepted-state-lifecycle-compare.mjs \
	--reinit-log "$reinit_log" \
	--cold-start-log "$cold_start_raw_log" \
	--out "$run_dir/lifecycle.redacted.txt"

verify_detector_port "$run_dir/detect-ultra205-post-capture.raw.log"
trace_event "post-capture-detector"

{
	write_common_evidence_header "complete"
	printf 'source_commit: %s\n' "$(current_source_commit)"
	printf 'manifest_source_commit_match: true\n'
	printf 'retained_package_reused: true\n'
	printf 'cold_start_flash_performed: false\n'
	printf 'cold_start_reset_performed: false\n'
	printf 'operator_attested_both_power_paths_removed: true\n'
	printf 'usb_absence_measured_ms: %s\n' "$usb_absence_measured_ms"
	printf 'usb_absence_category: at_least_5000_ms\n'
	printf 'barrel_removal_electronically_verified: false\n'
	printf 'reattach_deadline_ms: %s\n' "$reattach_deadline_ms"
	printf 'reappearance_elapsed_ms: %s\n' "$reappearance_elapsed_ms"
	printf 'monitor_start_reserve_ms: %s\n' "$monitor_start_reserve_ms"
	printf 'replay_interval_ms: %s\n' "$replay_interval_ms"
	printf 'replay_window_ms: %s\n' "$replay_window_ms"
	printf 'latest_safe_replay_ms: %s\n' "$latest_safe_replay_ms"
	printf 'next_replay_after_monitor_ms: %s\n' "$next_replay_after_monitor_ms"
	printf 'post_capture_detector_status: passed\n'
	cat "$run_dir/lifecycle.redacted.txt"
} >"$evidence_out"

printf 'accepted_state_diagnostic_status=complete\n'
printf 'accepted_state_diagnostic_redacted=true\n'
