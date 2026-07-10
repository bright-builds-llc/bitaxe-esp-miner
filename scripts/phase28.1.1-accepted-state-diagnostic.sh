#!/usr/bin/env bash
# Detector-gated Phase 28.1.1 accepted-state/lifecycle evidence wrapper.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode=""
attempt=""
duration_seconds="360"
reattach_timeout_seconds="45"
port=""
wifi_credentials=""
pool_credentials=""
evidence_out=""
manifest=""
reinit_log=""
board="205"

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
	printf 'usage: %s --mode blocked|hardware --attempt accepted-state|lifecycle --duration-seconds N --port PATH --wifi-credentials PATH --pool-credentials PATH --evidence-out PATH [--manifest PATH] [--reinit-log PATH] [--reattach-timeout-seconds N]\n' "$(basename "$0")"
}

while (($#)); do
	case "$1" in
	--mode | --attempt | --duration-seconds | --reattach-timeout-seconds | --port | --wifi-credentials | --pool-credentials | --evidence-out | --manifest | --reinit-log)
		[[ $# -ge 2 ]] || die "missing option value"
		name="${1#--}"
		case "$name" in
		mode) mode="$2" ;;
		attempt) attempt="$2" ;;
		duration-seconds) duration_seconds="$2" ;;
		reattach-timeout-seconds) reattach_timeout_seconds="$2" ;;
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
((reattach_timeout_seconds >= 1 && reattach_timeout_seconds < 90)) || die "--reattach-timeout-seconds must be between 1 and 89"
[[ -n "$evidence_out" ]] || die "--evidence-out is required"

run_root="${PHASE28_ACCEPTED_STATE_RUN_ROOT:-$repo_root/scratch/phase28.1.1-accepted-state}"
run_dir="$run_root/$(date -u +%Y%m%dT%H%M%SZ)-${attempt}"
mkdir -p "$run_dir" "$(dirname "$evidence_out")"

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

run_no_reset_monitor() {
	local output_log="$1"
	local -a args=(
		--port "$port"
		--out "$output_log"
		--seconds "$duration_seconds"
		--no-reset
	)
	trace_event "monitor-no-reset"
	if [[ -n "${PHASE28_ACCEPTED_STATE_MONITOR_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_MONITOR_BIN" "${args[@]}"
	else
		bash scripts/phase13-monitor-capture.sh "${args[@]}"
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

wait_attempts_used=0
wait_for_port_state() {
	local expected="$1"
	local label="$2"
	while ((wait_attempts_used < reattach_timeout_seconds)); do
		if port_is_present; then
			if [[ "$expected" == "present" ]]; then
				return
			fi
		else
			if [[ "$expected" == "absent" ]]; then
				return
			fi
		fi
		wait_attempts_used=$((wait_attempts_used + 1))
		wait_one_second
	done
	die "timed out waiting for selected port to become $label"
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

verify_cold_start_runtime() {
	local raw_log="$1"
	local boot_count
	local listener_count
	if rg -q -i 'stack overflow|stack canary|guru meditation|panic(ed)?|abort\(\)|SW_CPU_RESET|RTC_SW_(SYS|CPU)_RST|software reset' "$raw_log"; then
		die "cold-start capture contains a stack-overflow, panic, or software-reset marker"
	fi
	boot_count="$(rg -c 'bitaxe-rust boot' "$raw_log" || true)"
	listener_count="$(rg -c 'h4_continuous_result=listener_armed' "$raw_log" || true)"
	[[ "$boot_count" == "1" ]] || die "cold-start capture does not contain exactly one stable boot"
	[[ "$listener_count" == "1" ]] || die "cold-start capture does not contain exactly one listener-ready marker"
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
node scripts/phase28.1.1-accepted-state-lifecycle-compare.mjs \
	--reinit-log "$reinit_log" \
	--cold-start-log "$reinit_log" \
	--out "$run_dir/reinit-preflight.redacted.txt"
port_is_present || die "selected port is not present before lifecycle arming"

cold_start_raw_log="$run_dir/cold-start-monitor.raw.log"
trace_event "armed"
printf 'accepted_state_lifecycle_checkpoint=armed\n'
printf 'accepted_state_lifecycle_action=remove-both-power-paths-wait-five-seconds-restore-barrel-then-usb\n'

wait_for_port_state "absent" "absent"
wait_for_port_state "present" "present"
run_no_reset_monitor "$cold_start_raw_log" >"$run_dir/monitor-wrapper.raw.log" 2>&1
trace_event "capture-complete"

verify_cold_start_runtime "$cold_start_raw_log"
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
	printf 'post_capture_detector_status: passed\n'
	cat "$run_dir/lifecycle.redacted.txt"
} >"$evidence_out"

printf 'accepted_state_diagnostic_status=complete\n'
printf 'accepted_state_diagnostic_redacted=true\n'
