#!/usr/bin/env bash
# Detector-gated Phase 28.1.1 accepted-state/lifecycle evidence wrapper.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/process-group.sh
source "$repo_root/scripts/process-group.sh"
mode=""
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
	printf 'usage: %s --mode blocked|hardware --attempt accepted-state|lifecycle --duration-seconds N --port PATH --wifi-credentials PATH --pool-credentials PATH --evidence-out PATH [--manifest PATH] [--reinit-log PATH] [--reattach-timeout-seconds 60] [--attestation-timeout-seconds 300]\n' "$(basename "$0")"
}

while (($#)); do
	case "$1" in
	--mode | --attempt | --duration-seconds | --reattach-timeout-seconds | --attestation-timeout-seconds | --port | --wifi-credentials | --pool-credentials | --evidence-out | --manifest | --reinit-log)
		[[ $# -ge 2 ]] || die "missing option value"
		name="${1#--}"
		case "$name" in
		mode) mode="$2" ;;
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

[[ "$mode" == "blocked" || "$mode" == "hardware" ]] || die "invalid --mode"
[[ "$attempt" == "accepted-state" || "$attempt" == "lifecycle" ]] || die "invalid --attempt"
[[ "$duration_seconds" =~ ^[0-9]+$ ]] || die "--duration-seconds must be an integer"
((duration_seconds >= 360)) || die "--duration-seconds must be at least 360"
[[ "$reattach_timeout_seconds" =~ ^[0-9]+$ ]] || die "--reattach-timeout-seconds must be an integer"
((reattach_timeout_seconds == 60)) || die "--reattach-timeout-seconds must be exactly 60"
[[ "$attestation_timeout_seconds" =~ ^[0-9]+$ ]] || die "--attestation-timeout-seconds must be an integer"
((attestation_timeout_seconds == 300)) || die "--attestation-timeout-seconds must be exactly 300"
[[ -n "$evidence_out" ]] || die "--evidence-out is required"

run_root="${PHASE28_ACCEPTED_STATE_RUN_ROOT:-$repo_root/scratch/phase28.1.1-accepted-state}"
run_dir="$run_root/$(date -u +%Y%m%dT%H%M%SZ)-${attempt}"
mkdir -p "$run_dir" "$(dirname "$evidence_out")"

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

cd "$repo_root"

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
